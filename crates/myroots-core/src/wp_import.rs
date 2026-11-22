//! Import of a backup made by the MyRoots WordPress plugin
//! (MyRoots → Backup → "Download backup (JSON)", format 1).
//!
//! Relations stored twice in WordPress are normalized into the canonical
//! model. The backup contains only *ids* of WordPress attachments (photos,
//! scans); they are returned as "pending media" which the UI can download
//! from the site (WordPress REST API: /wp-json/wp/v2/media/<id>) and attach.

use crate::date;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::life::Ctx;
use crate::model::*;
use crate::ops;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum MediaRef {
    PersonPhoto { id: Id },
    PersonGallery { id: Id, caption: String },
    PersonEvent { id: Id, index: usize },
    PetPhoto { id: Id },
    PetGallery { id: Id, caption: String },
    SourcePhoto { id: Id },
    SourceScan { id: Id },
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(default)]
pub struct PendingMedia {
    pub wp_id: u64,
    pub refs: Vec<MediaRef>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct WpPending {
    pub site: String,
    pub items: Vec<PendingMedia>,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct WpReport {
    pub persons: usize,
    pub unions: usize,
    pub pets: usize,
    pub sources: usize,
    pub media_pending: usize,
    pub site: String,
    pub plugin: String,
    pub warnings: Vec<(String, String)>,
}

fn s(v: &Value, k: &str) -> String {
    match v.get(k) {
        Some(Value::String(x)) => x.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

fn n(v: &Value, k: &str) -> u64 {
    match v.get(k) {
        Some(Value::Number(x)) => x.as_u64().unwrap_or(0),
        Some(Value::String(x)) => x.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

fn arr<'a>(v: &'a Value, k: &str) -> Vec<&'a Value> {
    match v.get(k) {
        Some(Value::Array(a)) => a.iter().collect(),
        Some(Value::Object(o)) => o.values().collect(), // PHP arrays with holes
        _ => vec![],
    }
}

fn id_list(v: &Value, k: &str) -> Vec<u64> {
    arr(v, k)
        .into_iter()
        .filter_map(|x| match x {
            Value::Number(n) => n.as_u64(),
            Value::String(s) => s.trim().parse().ok(),
            _ => None,
        })
        .filter(|x| *x > 0)
        .collect()
}

pub fn import(db: &mut Database, ctx: &Ctx, json: &str) -> Result<WpReport> {
    let data: Value = serde_json::from_str(json).map_err(|e| Error::new("wp_invalid", e.to_string()))?;
    if data.get("format").and_then(|f| f.as_u64()) != Some(1) || data.get("persons").is_none() {
        return Err(Error::new("wp_invalid", "this is not a valid MyRoots backup file"));
    }
    let mut rep = WpReport { site: s(&data, "site"), plugin: s(&data, "plugin"), ..Default::default() };
    let mut pending: BTreeMap<u64, Vec<MediaRef>> = BTreeMap::new();
    let mut add_media = |wp: u64, r: MediaRef| {
        if wp > 0 {
            pending.entry(wp).or_default().push(r);
        }
    };

    let persons = arr(&data, "persons");
    let sources = arr(&data, "sources");
    let mut pmap: HashMap<u64, Id> = HashMap::new();
    let mut smap: HashMap<u64, Id> = HashMap::new();

    // 1. Sources (citations of people point to them).
    for row in &sources {
        let m = row.get("meta").cloned().unwrap_or(Value::Null);
        let title = s(row, "title");
        let src = Source {
            title: if title.is_empty() { "—".into() } else { title },
            date: s(&m, "date"),
            url: s(&m, "url"),
            notes: s(&m, "notes"),
            media_url: s(&m, "media_url"),
            ..Default::default()
        };
        let id = ops::save_source(db, ctx, src)?;
        smap.insert(n(row, "id"), id);
        rep.sources += 1;
        add_media(n(row, "thumbnail"), MediaRef::SourcePhoto { id });
        for g in id_list(&m, "gallery") {
            add_media(g, MediaRef::SourceScan { id });
        }
    }

    // 2. People (without relations).
    for row in &persons {
        let d = row.get("data").cloned().unwrap_or(Value::Null);
        let id = db.next_id();
        pmap.insert(n(row, "id"), id);
        let mut p = Person {
            id,
            status: Status::from_wp(&s(row, "status")),
            first_name: s(&d, "first_name"),
            last_name: s(&d, "last_name"),
            maiden_name: s(&d, "maiden_name"),
            gender: Gender::from_code(&s(&d, "gender")),
            birth_date: s(&d, "birth_date"),
            birth_place: s(&d, "birth_place"),
            death_date: s(&d, "death_date"),
            death_place: s(&d, "death_place"),
            death_cause: s(&d, "death_cause"),
            biography: s(&d, "biography"),
            notes: s(row, "content"),
            occupation: s(&d, "occupation"),
            education: s(&d, "education"),
            faith: s(&d, "faith"),
            height: s(&d, "height"),
            eye_color: s(&d, "eye_color"),
            blood_type: s(&d, "blood_type"),
            clans: arr(&d, "clans").into_iter().filter_map(|c| c.as_str().map(|x| x.to_string())).collect(),
            created: ctx.now,
            updated: ctx.now,
            ..Default::default()
        };
        if p.first_name.is_empty() && p.last_name.is_empty() {
            let t = s(row, "title");
            let mut it = t.splitn(2, ' ');
            p.first_name = it.next().unwrap_or("").to_string();
            p.last_name = it.next().unwrap_or("").to_string();
        }
        for (i, ev) in arr(&d, "events").into_iter().enumerate() {
            p.events.push(LifeEvent {
                name: s(ev, "event_name"),
                date: s(ev, "event_date"),
                place: String::new(),
                description: s(ev, "event_description"),
                image: None,
                citations: vec![],
            });
            add_media(n(ev, "event_image"), MediaRef::PersonEvent { id, index: i });
        }
        add_media(n(row, "thumbnail"), MediaRef::PersonPhoto { id });
        for g in arr(&d, "gallery") {
            add_media(n(g, "id"), MediaRef::PersonGallery { id, caption: s(g, "caption") });
        }
        if let Some(Value::Object(custom)) = d.get("custom") {
            for (gid, fields) in custom {
                if let Value::Object(f) = fields {
                    let m: BTreeMap<String, String> = f.iter().map(|(k, v)| (k.clone(), v.as_str().map(|x| x.to_string()).unwrap_or_else(|| v.to_string()))).collect();
                    p.custom.insert(gid.clone(), m);
                }
            }
        }
        if let Some(Value::Object(cit)) = d.get("citations") {
            for (field, ids) in cit {
                let v: Vec<Id> = match ids {
                    Value::Array(a) => a.iter().filter_map(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.parse().ok()))).filter_map(|x| smap.get(&x).copied()).collect(),
                    _ => vec![],
                };
                if !v.is_empty() {
                    p.citations.insert(field.clone(), v);
                }
            }
        }
        // Events are deduplicated / sanitized; keep the indexes stable for media refs.
        let p = {
            let mut clean = ops::sanitize_person(&p, db);
            // sanitize_person may drop empty events → re-check indexes later.
            clean.events = p.events.iter().map(|e| LifeEvent { date: date::sanitize(&e.date), description: ops::clean_html(&e.description), ..e.clone() }).collect();
            clean
        };
        db.put_person(p);
        rep.persons += 1;
    }

    // 3. Relations.
    let remap = |x: u64| pmap.get(&x).copied();
    for row in &persons {
        let d = row.get("data").cloned().unwrap_or(Value::Null);
        let Some(id) = remap(n(row, "id")) else { continue };
        let mut p = db.persons[&id].clone();
        let mother = remap(n(&d, "parent_mother"));
        let father = remap(n(&d, "parent_father"));
        if let Some(m) = mother {
            if m != id {
                p.set_parent(ParentRole::Mother, Some(m), ParentRel::parse(&s(&d, "parent_mother_type")));
            }
        }
        if let Some(f) = father {
            if f != id {
                p.set_parent(ParentRole::Father, Some(f), ParentRel::parse(&s(&d, "parent_father_type")));
            }
        }
        db.put_person(p);
    }
    // children[] lists: fill missing parent slots (old data before relation sync).
    for row in &persons {
        let d = row.get("data").cloned().unwrap_or(Value::Null);
        let Some(pid) = remap(n(row, "id")) else { continue };
        let pg = db.persons[&pid].gender;
        for c in id_list(&d, "children") {
            let Some(cid) = remap(c) else { continue };
            if cid == pid {
                continue;
            }
            let child = &db.persons[&cid];
            if child.parents().any(|x| x == pid) {
                continue;
            }
            let role = ParentRole::for_gender(pg);
            let slot = if child.parent(role).is_none() {
                Some(role)
            } else if !pg.is_known() && child.parent(role.other()).is_none() {
                Some(role.other())
            } else {
                None
            };
            match slot {
                Some(r) if !ops::creates_cycle(db, cid, pid) => {
                    let mut c2 = child.clone();
                    c2.set_parent(r, Some(pid), ParentRel::Bio);
                    db.put_person(c2);
                }
                _ => rep.warnings.push(("child_points_elsewhere".into(), format!("{} → {}", db.name_of(pid), db.name_of(cid)))),
            }
        }
    }
    // Remove cycles that might come from inconsistent data.
    let ids: Vec<Id> = db.persons.keys().copied().collect();
    for id in ids {
        let p = db.persons[&id].clone();
        for role in [ParentRole::Mother, ParentRole::Father] {
            if let Some(par) = p.parent(role) {
                let mut tmp = p.clone();
                tmp.set_parent(role, None, ParentRel::Bio);
                db.persons.insert(id, tmp);
                let cyc = ops::creates_cycle(db, id, par);
                db.persons.insert(id, p.clone());
                db.touch();
                if cyc {
                    let mut fixed = db.persons[&id].clone();
                    fixed.set_parent(role, None, ParentRel::Bio);
                    db.put_person(fixed);
                    rep.warnings.push(("ancestor_cycle".into(), db.name_of(id)));
                }
            }
        }
    }
    // Unions.
    for row in &persons {
        let d = row.get("data").cloned().unwrap_or(Value::Null);
        let Some(id) = remap(n(row, "id")) else { continue };
        for sp in arr(&d, "spouses") {
            let partner = remap(n(sp, "person_id")).filter(|x| *x != id);
            let u = Union {
                id: 0,
                a: id,
                b: partner,
                partner_first_name: if partner.is_none() { s(sp, "first_name") } else { String::new() },
                partner_last_name: if partner.is_none() { s(sp, "last_name") } else { String::new() },
                date: date::sanitize(&s(sp, "marriage_date")),
                place: s(sp, "marriage_place"),
                end_cause: end_cause::normalize(&s(sp, "marriage_end_cause")),
                end_date: date::sanitize(&s(sp, "marriage_end")),
                informal: matches!(sp.get("informal"), Some(Value::Number(n)) if n.as_u64() == Some(1)) || matches!(sp.get("informal"), Some(Value::Bool(true))) || s(sp, "informal") == "1",
                citations: vec![],
            };
            match partner {
                Some(b) => {
                    let before = db.unions.len();
                    ops::link_partners(db, id, b, Some(&u))?;
                    if db.unions.len() > before {
                        rep.unions += 1;
                    }
                }
                None => {
                    if u.partner_first_name.is_empty() && u.partner_last_name.is_empty() && u.date.is_empty() {
                        continue;
                    }
                    ops::save_union(db, u)?;
                    rep.unions += 1;
                }
            }
        }
    }
    // Normalize the end cause "ongoing" → "".
    let uids: Vec<Id> = db.unions.keys().copied().collect();
    for uid in uids {
        let u = &db.unions[&uid];
        if u.end_cause == end_cause::ONGOING {
            let mut u = u.clone();
            u.end_cause.clear();
            db.put_union(u);
        }
    }

    // 4. Source links to people.
    for row in &sources {
        let Some(sid) = smap.get(&n(row, "id")).copied() else { continue };
        let mut src = db.sources[&sid].clone();
        for p in id_list(row, "persons") {
            if let Some(pid) = remap(p) {
                if !src.persons.contains(&pid) {
                    src.persons.push(pid);
                }
            }
        }
        db.put_source(src);
    }

    // 5. Pets.
    for row in arr(&data, "pets") {
        let d = row.get("data").cloned().unwrap_or(Value::Null);
        let mut name = s(&d, "name");
        if name.is_empty() {
            name = s(row, "title");
        }
        let pet = Pet {
            status: Status::from_wp(&s(row, "status")),
            name,
            species: s(&d, "species"),
            breed: s(&d, "breed"),
            gender: Gender::from_code(&s(&d, "gender")),
            birth_date: s(&d, "birth_date"),
            death_date: s(&d, "death_date"),
            owner: remap(n(&d, "owner_id")),
            biography: s(&d, "biography"),
            notes: s(row, "content"),
            ..Default::default()
        };
        let id = ops::save_pet(db, ctx, pet)?;
        rep.pets += 1;
        add_media(n(row, "thumbnail"), MediaRef::PetPhoto { id });
        for g in arr(&d, "gallery") {
            add_media(n(g, "id"), MediaRef::PetGallery { id, caption: s(g, "caption") });
        }
    }

    // 6. Options: clans, custom field schema.
    if let Some(opts) = data.get("options") {
        let mut st = db.settings.clone();
        for c in arr(opts, "clans") {
            if let Some(c) = c.as_str() {
                if !st.clans.iter().any(|x| x == c) {
                    st.clans.push(c.to_string());
                }
            }
        }
        if st.custom_schema.is_empty() {
            let raw = s(opts, "custom_schema");
            if let Ok(schema) = serde_json::from_str::<Vec<CustomGroup>>(&raw) {
                st.custom_schema = schema;
            }
        }
        if st.tree_name.is_empty() {
            st.tree_name = rep.site.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/').to_string();
        }
        ops::save_settings(db, st);
    }

    // Pending media.
    let mut wp = load_pending(db);
    if !rep.site.is_empty() {
        wp.site = rep.site.clone();
    }
    for (wp_id, refs) in pending {
        rep.media_pending += 1;
        match wp.items.iter_mut().find(|i| i.wp_id == wp_id) {
            Some(i) => i.refs.extend(refs),
            None => wp.items.push(PendingMedia { wp_id, refs }),
        }
    }
    save_pending(db, &wp);
    Ok(rep)
}

const PENDING_KEY: &str = "wp_pending";

pub fn load_pending(db: &Database) -> WpPending {
    db.meta_extra(PENDING_KEY).and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default()
}

pub fn save_pending(db: &mut Database, p: &WpPending) {
    let v = if p.items.is_empty() { None } else { Some(serde_json::to_value(p).unwrap()) };
    db.set_meta_extra(PENDING_KEY, v);
}

/// Attaches a downloaded attachment (already registered as `media`) to every
/// place that referenced the WordPress attachment `wp_id`.
pub fn attach(db: &mut Database, wp_id: u64, media: Id) -> Result<usize> {
    if !db.media.contains_key(&media) {
        return Err(Error::not_found("media", media));
    }
    let mut wp = load_pending(db);
    let Some(pos) = wp.items.iter().position(|i| i.wp_id == wp_id) else { return Ok(0) };
    let item = wp.items.remove(pos);
    let mut n = 0;
    for r in &item.refs {
        match r {
            MediaRef::PersonPhoto { id } | MediaRef::PersonGallery { id, .. } | MediaRef::PersonEvent { id, .. } => {
                let Some(p) = db.persons.get(id) else { continue };
                let mut p = p.clone();
                match r {
                    MediaRef::PersonPhoto { .. } => p.photo = Some(media),
                    MediaRef::PersonGallery { caption, .. } => {
                        if !p.gallery.iter().any(|g| g.media == media) {
                            p.gallery.push(GalleryItem { media, caption: caption.clone() });
                        }
                    }
                    MediaRef::PersonEvent { index, .. } => {
                        if let Some(e) = p.events.get_mut(*index) {
                            e.image = Some(media);
                        }
                    }
                    _ => {}
                }
                db.put_person(p);
            }
            MediaRef::PetPhoto { id } | MediaRef::PetGallery { id, .. } => {
                let Some(p) = db.pets.get(id) else { continue };
                let mut p = p.clone();
                match r {
                    MediaRef::PetPhoto { .. } => p.photo = Some(media),
                    MediaRef::PetGallery { caption, .. } => {
                        if !p.gallery.iter().any(|g| g.media == media) {
                            p.gallery.push(GalleryItem { media, caption: caption.clone() });
                        }
                    }
                    _ => {}
                }
                db.put_pet(p);
            }
            MediaRef::SourcePhoto { id } | MediaRef::SourceScan { id } => {
                let Some(s) = db.sources.get(id) else { continue };
                let mut s = s.clone();
                if matches!(r, MediaRef::SourcePhoto { .. }) {
                    s.photo = Some(media);
                } else if !s.scans.contains(&media) {
                    s.scans.push(media);
                }
                db.put_source(s);
            }
        }
        n += 1;
    }
    save_pending(db, &wp);
    Ok(n)
}

/// Forget the pending downloads (user decided not to fetch the photos).
pub fn clear_pending(db: &mut Database) {
    save_pending(db, &WpPending::default());
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn import_backup() {
        let json = r#"{"format":1,"plugin":"2.6.0","site":"https://example.org","persons":[
          {"id":10,"title":"Jan Kowalski","status":"publish","content":"","thumbnail":501,"data":{"first_name":"Jan","last_name":"Kowalski","gender":"M","birth_date":"~1900","parent_mother":0,"parent_father":0,"spouses":[{"person_id":11,"marriage_date":"1925-06-12","marriage_place":"Kraków","marriage_end_cause":"Trwa","marriage_end":"","informal":0}],"children":[12],"events":[{"event_name":"Wojsko","event_date":"1920","event_description":"","event_image":502}],"clans":["Kowalscy"],"gallery":[{"id":503,"caption":"x"}],"custom":{"g1":{"f1":"v"}},"citations":{"birth_date":[30]}}},
          {"id":11,"title":"Anna Nowak","status":"publish","content":"","thumbnail":0,"data":{"first_name":"Anna","last_name":"Nowak","gender":"F","spouses":[{"person_id":10,"marriage_date":"","marriage_place":"","marriage_end_cause":"","marriage_end":"","informal":0}],"children":[12]}},
          {"id":12,"title":"Ewa","status":"publish","content":"","thumbnail":0,"data":{"first_name":"Ewa","last_name":"Kowalska","gender":"F","parent_mother":11,"parent_father":0,"parent_mother_type":"adopted"}}
        ],"pets":[{"id":20,"title":"Burek","thumbnail":504,"data":{"name":"Burek","species":"dog","owner_id":10}}],
        "sources":[{"id":30,"title":"Księga","thumbnail":0,"meta":{"date":"1900","url":"","notes":"","media_url":"","gallery":[505]},"persons":[10]}],
        "options":{"clans":["Kowalscy","Nowakowie"],"custom_schema":"[{\"id\":\"g1\",\"name\":\"Wojsko\",\"fields\":[{\"id\":\"f1\",\"name\":\"Stopień\",\"type\":\"text\"}]}]"}}"#;
        let c = Ctx::new(0, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let mut db = Database::new(0);
        let rep = import(&mut db, &c, json).unwrap();
        assert_eq!((rep.persons, rep.unions, rep.pets, rep.sources, rep.media_pending), (3, 1, 1, 1, 5));
        let jan = db.persons.values().find(|p| p.first_name == "Jan").unwrap().clone();
        let ewa = db.persons.values().find(|p| p.first_name == "Ewa").unwrap().clone();
        assert_eq!(ewa.father, Some(jan.id));
        assert_eq!(ewa.mother_rel, ParentRel::Adopted);
        let u = db.unions.values().next().unwrap();
        assert_eq!(u.date, "1925-06-12");
        assert_eq!(u.end_cause, "");
        assert_eq!(jan.citations["birth_date"].len(), 1);
        assert_eq!(db.settings.custom_schema.len(), 1);
        assert_eq!(db.settings.clans.len(), 2);
        // attach a downloaded photo
        let m = crate::ops::register_media(&mut db, &c, Media { file_name: "a.jpg".into(), mime: "image/jpeg".into(), ..Default::default() });
        assert_eq!(attach(&mut db, 501, m).unwrap(), 1);
        assert_eq!(db.persons[&jan.id].photo, Some(m));
        assert_eq!(load_pending(&db).items.len(), 4);
        let m2 = crate::ops::register_media(&mut db, &c, Media::default());
        attach(&mut db, 502, m2).unwrap();
        assert_eq!(db.persons[&jan.id].events[0].image, Some(m2));
    }
}
