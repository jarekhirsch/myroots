//! Robustness tests: random and mangled input must never crash the engine
//! (a panic would stop the whole app). Deterministic, so failures reproduce.
#![cfg(test)]

use crate::db::Database;
use crate::engine::Engine;
use crate::life::Ctx;
use chrono::NaiveDate;
use serde_json::{json, Value};
use std::panic::{catch_unwind, AssertUnwindSafe};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
    fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[self.below(v.len())]
    }
}

const ATOMS: &[&str] = &[
    "", " ", "0", "-1", "1", "12", "1920", "15 MAY 1920", "~1900", "<1800", ">2000", "1900..1910", "ABT 1850", "BET 1 AND", "@", "@@", "@I1@", "/", "//", "\n", "\r\n",
    "<", ">", "<script>alert(1)</script>", "<img src=x onerror=alert(1)>", "javascript:alert(1)", "İstanbul", "ẞ", "K", "Łódź", "ąęść", "🙂", "\u{202e}", "\u{0}", "\u{feff}",
    "1 CONC", "2 CONT x", "0 TRLR", "0 @F1@ FAM", "1 CHIL @I2@", "1 HUSB @I1@", "%s", "%1$d", "9999999999999999999999", "1e309", "NaN", "null", "true", "[]", "{}",
];

fn rand_str(r: &mut Rng) -> String {
    let n = r.below(5);
    (0..n).map(|_| r.pick(ATOMS).to_string()).collect::<Vec<_>>().join(if r.below(2) == 0 { "" } else { " " })
}

fn rand_value(r: &mut Rng, depth: usize) -> Value {
    match r.below(if depth > 2 { 5 } else { 8 }) {
        0 => Value::Null,
        1 => json!(r.below(2) == 0),
        2 => json!(r.below(40) as i64 - 5),
        3 => json!(r.next() as i64),
        4 => json!(rand_str(r)),
        5 => Value::Array((0..r.below(4)).map(|_| rand_value(r, depth + 1)).collect()),
        _ => {
            const KEYS: &[&str] = &[
                "id", "person", "a", "b", "of", "relation", "first_name", "last_name", "birth_date", "death_date", "gender", "mother", "father", "date", "name", "title", "media",
                "eyes", "aging", "recordings", "union", "children", "unions", "settings", "text", "base64", "options", "mode", "dry_run", "keep", "query", "q", "from", "to", "url",
                "events", "gallery", "custom", "citations", "status", "role", "revision", "sources", "clan", "places", "lat", "lng", "json", "token", "message",
            ];
            let mut m = serde_json::Map::new();
            for _ in 0..r.below(6) {
                m.insert(r.pick(KEYS).to_string(), rand_value(r, depth + 1));
            }
            Value::Object(m)
        }
    }
}

fn ctx() -> Ctx {
    Ctx::new(1, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap())
}

#[test]
fn dates_and_text_never_panic() {
    let mut r = Rng(0x9e3779b97f4a7c15);
    for _ in 0..20000 {
        let s = rand_str(&mut r);
        let res = catch_unwind(|| {
            let _ = crate::date::sanitize(&s);
            let _ = crate::date::from_gedcom(&s);
            let _ = crate::date::to_gedcom(&crate::date::sanitize(&s));
            let _ = crate::date::year(&s);
            let _ = crate::date::sort_key(&s);
            let _ = crate::ops::clean_html(&s);
            let _ = crate::model::norm(&s);
            let _ = crate::model::safe_url(&s);
            let _ = crate::model::strip_tags(&s);
            let _ = crate::ops::feminine_surname(&s);
        });
        assert!(res.is_ok(), "panic on {s:?}");
    }
}

#[test]
fn gedcom_import_never_panics() {
    let base = "0 HEAD\n1 CHAR UTF-8\n0 @N1@ NOTE x\n0 @S1@ SOUR\n1 TITL Book\n0 @I1@ INDI\n1 NAME Jan /Kowalski/\n2 GIVN Jan\n1 SEX M\n1 BIRT\n2 DATE ABT 1900\n2 PLAC Kraków\n3 MAP\n4 LATI N50.1\n4 LONG E19.9\n2 SOUR @S1@\n1 NOTE @N1@\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME Anna /Nowak/\n2 _MARNM X\n1 SEX F\n1 DEAT Y\n1 FAMS @F1@\n0 @I3@ INDI\n1 NAME Ewa\n1 FAMC @F1@\n2 PEDI adopted\n0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n1 MARR\n2 DATE 1925\n1 CHIL @I3@\n1 CHIL @I1@\n0 @PET1@ _PET\n1 NAME Burek\n1 _OWNER @I1@\n0 TRLR\n";
    let lines: Vec<&str> = base.lines().collect();
    let mut r = Rng(0x2545f4914f6cdd1d);
    for round in 0..1500 {
        let mut ls: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        for _ in 0..1 + r.below(6) {
            let i = r.below(ls.len());
            match r.below(6) {
                0 => {
                    ls.remove(i);
                }
                1 => ls.insert(i, format!("{} {}", r.below(5), rand_str(&mut r))),
                2 => ls[i].push_str(&rand_str(&mut r)),
                3 => {
                    let j = r.below(ls.len());
                    ls.swap(i, j);
                }
                4 => {
                    let cut = r.below(ls[i].len() + 1);
                    let cut = (0..=cut).rev().find(|c| ls[i].is_char_boundary(*c)).unwrap_or(0);
                    ls[i].truncate(cut);
                }
                _ => ls[i] = ls[i].replacen(char::is_numeric, &r.below(9).to_string(), 1),
            }
        }
        let text = ls.join(if round % 3 == 0 { "\r\n" } else { "\n" });
        let res = catch_unwind(AssertUnwindSafe(|| {
            let mut db = Database::new(0);
            let _ = crate::gedcom::import(&mut db, &ctx(), &text, &crate::gedcom::ImportOptions::default());
            let _ = crate::gedcom::export(&db, &ctx(), &crate::gedcom::ExportOptions::default());
            let mut bytes = text.clone().into_bytes();
            if !bytes.is_empty() {
                let k = r.below(bytes.len());
                bytes[k] = (r.next() & 0xff) as u8;
            }
            let _ = crate::gedcom::decode(&bytes);
            let _ = crate::gedcom::decode_file(&bytes);
        }));
        assert!(res.is_ok(), "panic on GEDCOM:\n{text}");
    }
}

#[test]
fn engine_methods_never_panic() {
    let methods = [
        "app.info", "overview", "settings.get", "settings.save", "people.summaries", "person.profile", "person.edit", "person.create", "person.update", "person.delete",
        "person.add_relative", "person.link", "person.set_parent", "person.history", "person.restore", "person.set_media", "person.set_sources", "union.save", "union.delete",
        "pets.list", "pet.get", "pet.save", "pet.delete", "sources.list", "source.get", "source.save", "source.delete", "media.reserve", "media.register", "media.update",
        "media.delete", "media.list", "media.unused", "clans.list", "clans.rename", "clans.delete", "clans.assign", "places.list", "places.missing", "places.set", "places.rename",
        "tree.data", "lifeline", "map.data", "stats", "anniversaries", "events.list", "gallery", "kinship", "health.issues", "health.completeness", "merge.suggestions",
        "merge.dismiss", "merge.compare", "merge.execute", "book.build", "suggestions.list", "suggestion.encode", "suggestions.import", "suggestions.add",
        "suggestions.set_status", "suggestions.delete", "gedcom.import", "gedcom.export", "wp.import", "wp.pending", "wp.attach", "wp.clear", "no.such.method",
    ];
    let mut r = Rng(0xdeadbeefcafebabe);
    let mut e = Engine::new(Database::new(0), "3.0.0");
    // a few people so that ids hit something
    for i in 0..6 {
        let _ = e.call("person.create", json!({"person": {"first_name": format!("P{i}"), "birth_date": format!("19{i}0")}}), &ctx());
    }
    let mut ok: std::collections::BTreeMap<&str, u32> = Default::default();
    for n in 0..30000 {
        let m = *r.pick(&methods);
        let mut p = rand_value(&mut r, 0);
        if let Value::Object(o) = &mut p {
            if r.below(2) == 0 {
                o.insert("id".into(), json!(1 + r.below(12)));
            }
        }
        if n % 5000 == 0 {
            e.viewer = !e.viewer;
        }
        let res = catch_unwind(AssertUnwindSafe(|| e.call(m, p.clone(), &ctx()).is_ok()));
        assert!(res.is_ok(), "panic in {m} with {p}");
        if res.unwrap() {
            *ok.entry(m).or_insert(0) += 1;
        }
        let _ = e.take_changes();
    }
    eprintln!("successful calls: {ok:?}");
}

#[test]
fn archives_never_panic() {
    let mut db = Database::new(0);
    let mut e = Engine::new(Database::new(0), "3.0.0");
    let _ = e.call("person.create", json!({"person": {"first_name": "Jan"}}), &ctx());
    db = std::mem::replace(&mut e.db, db);
    struct NoMedia;
    impl crate::archive::MediaSource for NoMedia {
        fn original(&mut self, _: crate::model::Id) -> Option<Vec<u8>> {
            Some(vec![1, 2, 3])
        }
        fn thumb(&mut self, _: crate::model::Id) -> Option<Vec<u8>> {
            None
        }
    }
    let good = crate::archive::write_vec(&db, &mut NoMedia, "3.0.0", 0).unwrap();
    let mut r = Rng(0x1234567887654321);
    for _ in 0..3000 {
        let mut b = good.clone();
        for _ in 0..1 + r.below(8) {
            let k = r.below(b.len());
            b[k] = (r.next() & 0xff) as u8;
        }
        if r.below(4) == 0 {
            b.truncate(r.below(b.len()));
        }
        let res = catch_unwind(|| {
            let _ = crate::archive::read_slice(&b, |_, _, _| Ok(()));
        });
        assert!(res.is_ok(), "panic reading a damaged archive");
    }
    let res = catch_unwind(|| {
        for s in ["", "{", "[]", "{\"posts\":[{\"id\":\"x\"}]}", "{\"persons\":[{\"id\":1,\"data\":{\"spouses\":[{\"person_id\":1}],\"children\":[1]}}]}"] {
            let mut d = Database::new(0);
            let _ = crate::wp_import::import(&mut d, &ctx(), s);
        }
    });
    assert!(res.is_ok());
}

#[test]
fn structured_edits_never_panic() {
    let mut r = Rng(0x0badc0ffee123457);
    let mut e = Engine::new(Database::new(0), "3.0.0");
    let mut ids: Vec<u64> = vec![];
    let rels = ["father", "mother", "spouse", "child", "sibling", "x"];
    let person = |r: &mut Rng, ids: &[u64]| {
        let id_or_null = |r: &mut Rng| if ids.is_empty() || r.below(3) == 0 { Value::Null } else { json!(ids[r.below(ids.len())]) };
        json!({
            "first_name": rand_str(r), "last_name": rand_str(r), "maiden_name": rand_str(r), "gender": *r.pick(&["M", "F", "U", ""]),
            "birth_date": rand_str(r), "death_date": rand_str(r), "birth_place": rand_str(r), "biography": rand_str(r), "deceased": r.below(2) == 0,
            "mother": id_or_null(r), "father": id_or_null(r), "mother_rel": *r.pick(&["bio", "adopted", "step"]),
            "events": [{"name": rand_str(r), "date": rand_str(r), "place": rand_str(r), "description": rand_str(r), "image": id_or_null(r)}],
            "clans": [rand_str(r)], "gallery": [{"media": r.below(300), "caption": rand_str(r)}],
            "aging": [{"media": r.below(300), "date": rand_str(r), "eyes": [r.below(3) as f64 - 0.5, 0.4, 1e9, -3]}],
            "custom": {"g": {"f": rand_str(r)}}, "citations": {"birth_date": [r.below(300)]}
        })
    };
    for n in 0..6000 {
        let pick = |r: &mut Rng, ids: &[u64]| if ids.is_empty() { 1 } else { ids[r.below(ids.len())] };
        let (m, p) = match r.below(9) {
            0 | 1 => ("person.create", json!({ "person": person(&mut r, &ids) })),
            2 => {
                let mut pp = person(&mut r, &ids);
                pp["id"] = json!(pick(&mut r, &ids));
                ("person.update", json!({ "person": pp, "relations": {"children": [pick(&mut r, &ids)], "unions": [{"a": pick(&mut r, &ids), "b": pick(&mut r, &ids), "date": rand_str(&mut r)}]} }))
            }
            3 => ("person.add_relative", json!({"of": pick(&mut r, &ids), "relation": *r.pick(&rels), "person": person(&mut r, &ids)})),
            4 => ("person.link", json!({"of": pick(&mut r, &ids), "id": pick(&mut r, &ids), "other": pick(&mut r, &ids), "relation": *r.pick(&rels)})),
            5 => ("union.save", json!({"union": {"a": pick(&mut r, &ids), "b": pick(&mut r, &ids), "date": rand_str(&mut r), "end_cause": rand_str(&mut r)}})),
            6 => ("merge.execute", json!({"a": pick(&mut r, &ids), "b": pick(&mut r, &ids), "keep": pick(&mut r, &ids), "selection": {}})),
            7 => (*r.pick(&["kinship", "lifeline", "tree.data", "person.profile", "book.build", "stats", "health.issues", "gedcom.export", "map.data"]), json!({"a": pick(&mut r, &ids), "b": pick(&mut r, &ids), "id": pick(&mut r, &ids), "person": pick(&mut r, &ids), "root": pick(&mut r, &ids)})),
            _ => (if n % 50 == 0 { "person.delete" } else { "person.restore" }, json!({"id": pick(&mut r, &ids), "revision": r.below(500)})),
        };
        let res = catch_unwind(AssertUnwindSafe(|| e.call(m, p.clone(), &ctx())));
        assert!(res.is_ok(), "panic in {m} with {p}");
        if n < 3 {
            if let Ok(Err(er)) = &res {
                eprintln!("{m}: {er:?}");
            }
        }
        if let Ok(Ok(v)) = res {
            if let Some(id) = v.get("id").and_then(|x| x.as_u64()) {
                if m == "person.create" || m == "person.add_relative" {
                    ids.push(id);
                }
            }
        }
        let _ = e.take_changes();
    }
    assert!(ids.len() > 100, "too few people created: {}", ids.len());
}
