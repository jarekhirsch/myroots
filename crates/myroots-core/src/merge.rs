//! Duplicate detection and merging of two people.

use crate::db::Database;
use crate::error::{Error, Result};
use crate::life::{self, Ctx};
use crate::model::*;
use crate::ops;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

pub const SCALAR_FIELDS: &[&str] = &[
    "first_name", "last_name", "maiden_name", "gender", "birth_date", "birth_place", "death_date", "death_place",
    "death_cause", "occupation", "education", "faith", "height", "eye_color", "blood_type", "biography", "notes",
];

#[derive(Serialize, Debug, Clone)]
pub struct Suggestion {
    pub a: Id,
    pub b: Id,
    pub score: u32,
    pub name_a: String,
    pub name_b: String,
    pub years_a: String,
    pub years_b: String,
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Score of a pair; None = certainly not a duplicate. Threshold: 45.
fn pair_score(a: &Person, b: &Person) -> Option<u32> {
    if a.gender.is_known() && b.gender.is_known() && a.gender != b.gender {
        return None;
    }
    let mut score = 0;
    let (fa, fb) = (norm(&a.first_name), norm(&b.first_name));
    if !fa.is_empty() && fa == fb {
        score += 50;
    } else if !fa.is_empty() && !fb.is_empty() && levenshtein(&fa, &fb) <= 2 {
        score += 35;
    } else if fa.is_empty() || fb.is_empty() {
        score += 15;
    } else {
        return None;
    }
    match (crate::date::year(&a.birth_date), crate::date::year(&b.birth_date)) {
        (Some(x), Some(y)) => {
            let d = (x - y).abs();
            if d == 0 {
                score += 40;
            } else if d <= 2 {
                score += 25;
            } else {
                return None;
            }
        }
        (Some(_), None) | (None, Some(_)) => score += 10,
        _ => score += 5,
    }
    let pa = norm(&a.birth_place);
    if !pa.is_empty() && pa == norm(&b.birth_place) {
        score += 10;
    }
    if score >= 45 {
        Some(score)
    } else {
        None
    }
}

pub fn suggestions(db: &Database, limit: usize) -> Vec<Suggestion> {
    let mut buckets: HashMap<String, Vec<Id>> = HashMap::new();
    for p in db.persons.values() {
        let mut keys: Vec<String> = [norm(&p.last_name), norm(&p.maiden_name)].into_iter().filter(|k| !k.is_empty()).collect();
        keys.dedup();
        for k in keys {
            buckets.entry(k).or_default().push(p.id);
        }
    }
    let dismissed: HashSet<&str> = db.settings.merge_dismissed.iter().map(|s| s.as_str()).collect();
    let mut pairs: BTreeMap<(Id, Id), u32> = BTreeMap::new();
    for ids in buckets.values() {
        let n = ids.len();
        if !(2..=300).contains(&n) {
            continue;
        }
        for i in 0..n {
            for j in i + 1..n {
                let (a, b) = (ids[i].min(ids[j]), ids[i].max(ids[j]));
                if pairs.contains_key(&(a, b)) || dismissed.contains(format!("{a}:{b}").as_str()) {
                    continue;
                }
                // Parent/child or partners are not duplicates.
                let (pa, pb) = (&db.persons[&a], &db.persons[&b]);
                if pa.parents().any(|x| x == b) || pb.parents().any(|x| x == a) {
                    continue;
                }
                if db.unions.values().any(|u| u.involves(a) && u.involves(b)) {
                    continue;
                }
                if let Some(s) = pair_score(pa, pb) {
                    pairs.insert((a, b), s);
                }
            }
        }
    }
    let mut v: Vec<Suggestion> = pairs
        .into_iter()
        .map(|((a, b), score)| {
            let (pa, pb) = (&db.persons[&a], &db.persons[&b]);
            Suggestion {
                a,
                b,
                score,
                name_a: pa.full_name(),
                name_b: pb.full_name(),
                years_a: life::years(&pa),
                years_b: life::years(&pb),
            }
        })
        .collect();
    v.sort_by(|x, y| y.score.cmp(&x.score).then(x.a.cmp(&y.a)));
    v.truncate(limit);
    v
}

pub fn dismiss(db: &mut Database, a: Id, b: Id) {
    let key = format!("{}:{}", a.min(b), a.max(b));
    let mut s = db.settings.clone();
    if !s.merge_dismissed.contains(&key) {
        s.merge_dismissed.push(key);
        db.put_settings(s);
    }
}

/* ------------------------------------------------------------------ */
/*  Compare                                                             */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug, Clone)]
pub struct Item {
    pub value: String,
    pub label: String,
    /// "A" | "B" | "A+B"
    #[serde(rename = "in")]
    pub side: String,
    pub media: Option<Id>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Comparison {
    pub a: Person,
    pub b: Person,
    pub mother_a: Option<(Id, String)>,
    pub mother_b: Option<(Id, String)>,
    pub father_a: Option<(Id, String)>,
    pub father_b: Option<(Id, String)>,
    pub children: Vec<Item>,
    pub partners: Vec<Item>,
    pub events: Vec<Item>,
    pub gallery: Vec<Item>,
    pub clans: Vec<Item>,
    pub sources: Vec<Item>,
    pub fields: Vec<String>,
}

fn side(in_a: bool, in_b: bool) -> String {
    match (in_a, in_b) {
        (true, true) => "A+B".into(),
        (true, false) => "A".into(),
        _ => "B".into(),
    }
}

pub fn compare(db: &Database, a: Id, b: Id) -> Result<Comparison> {
    if a == b {
        return Err(Error::invalid("select two different people"));
    }
    let pa = db.persons.get(&a).ok_or_else(|| Error::not_found("person", a))?;
    let pb = db.persons.get(&b).ok_or_else(|| Error::not_found("person", b))?;
    let ix = db.index();
    let label = |id: Id| {
        let p = &db.persons[&id];
        let y = life::years(&p);
        if y.is_empty() {
            p.full_name()
        } else {
            format!("{} ({y})", p.full_name())
        }
    };
    let par = |x: Option<Id>| x.filter(|i| db.persons.contains_key(i)).map(|i| (i, label(i)));

    let ca: HashSet<Id> = ix.children_of(a).iter().copied().collect();
    let cb: HashSet<Id> = ix.children_of(b).iter().copied().collect();
    let mut children: Vec<Id> = ca.union(&cb).copied().filter(|c| *c != a && *c != b).collect();
    children.sort();
    let pa_set: HashSet<Id> = ix.partners(db, a).into_iter().collect();
    let pb_set: HashSet<Id> = ix.partners(db, b).into_iter().collect();
    let mut partners: Vec<Id> = pa_set.union(&pb_set).copied().filter(|p| *p != a && *p != b).collect();
    partners.sort();

    let mut events = vec![];
    for (tok, p) in [("a", pa), ("b", pb)] {
        for (i, e) in p.events.iter().enumerate() {
            events.push(Item {
                value: format!("{tok}:{i}"),
                label: format!("{} {}", e.date, e.name).trim().to_string(),
                side: tok.to_uppercase(),
                media: e.image,
            });
        }
    }
    let mut gallery = vec![];
    let mut seen = HashSet::new();
    for p in [pa, pb] {
        for g in &p.gallery {
            if seen.insert(g.media) {
                gallery.push(Item {
                    value: g.media.to_string(),
                    label: g.caption.clone(),
                    side: side(pa.gallery.iter().any(|x| x.media == g.media), pb.gallery.iter().any(|x| x.media == g.media)),
                    media: Some(g.media),
                });
            }
        }
    }
    let mut clans = vec![];
    for c in pa.clans.iter().chain(pb.clans.iter()) {
        if !clans.iter().any(|x: &Item| x.value == *c) {
            clans.push(Item { value: c.clone(), label: c.clone(), side: side(pa.clans.contains(c), pb.clans.contains(c)), media: None });
        }
    }
    let sa: HashSet<Id> = ix.sources_of(a).iter().copied().collect();
    let sb: HashSet<Id> = ix.sources_of(b).iter().copied().collect();
    let mut srcs: Vec<Id> = sa.union(&sb).copied().collect();
    srcs.sort();
    Ok(Comparison {
        mother_a: par(pa.mother),
        mother_b: par(pb.mother),
        father_a: par(pa.father),
        father_b: par(pb.father),
        children: children.iter().map(|c| Item { value: c.to_string(), label: label(*c), side: side(ca.contains(c), cb.contains(c)), media: None }).collect(),
        partners: partners.iter().map(|c| Item { value: c.to_string(), label: label(*c), side: side(pa_set.contains(c), pb_set.contains(c)), media: None }).collect(),
        events,
        gallery,
        clans,
        sources: srcs.iter().map(|s| Item { value: s.to_string(), label: db.sources[s].title.clone(), side: side(sa.contains(s), sb.contains(s)), media: None }).collect(),
        fields: SCALAR_FIELDS.iter().map(|s| s.to_string()).collect(),
        a: pa.clone(),
        b: pb.clone(),
    })
}

/* ------------------------------------------------------------------ */
/*  Execute                                                             */
/* ------------------------------------------------------------------ */

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Selection {
    /// field → "a" | "b"
    pub scalar: BTreeMap<String, String>,
    pub mother: Option<Id>,
    pub father: Option<Id>,
    pub children: Vec<Id>,
    pub partners: Vec<Id>,
    /// "a:0", "b:2" …
    pub events: Vec<String>,
    pub gallery: Vec<Id>,
    pub clans: Vec<String>,
    pub sources: Vec<Id>,
    /// "a" | "b" | "none"
    pub avatar: String,
}

fn scalar_get(p: &Person, f: &str) -> serde_json::Value {
    serde_json::to_value(p).unwrap().get(f).cloned().unwrap_or(serde_json::Value::Null)
}

pub fn execute(db: &mut Database, ctx: &Ctx, a: Id, b: Id, keep: Id, sel: Selection) -> Result<Id> {
    if !((keep == a) || (keep == b)) || a == b {
        return Err(Error::invalid("keep must be one of the two people"));
    }
    let remove = if keep == a { b } else { a };
    let pa = db.persons.get(&a).cloned().ok_or_else(|| Error::not_found("person", a))?;
    let pb = db.persons.get(&b).cloned().ok_or_else(|| Error::not_found("person", b))?;
    let (pk, pr) = if keep == a { (pa.clone(), pb.clone()) } else { (pb.clone(), pa.clone()) };
    let old_unions: Vec<Union> = db.unions.values().filter(|u| u.involves(keep)).cloned().collect();

    // Scalars
    let mut v = serde_json::to_value(&pk).unwrap();
    for f in SCALAR_FIELDS {
        let src = if sel.scalar.get(*f).map(|s| s == "b").unwrap_or(false) { &pb } else { &pa };
        v[*f] = scalar_get(src, f);
    }
    let mut new: Person = serde_json::from_value(v)?;
    let ok_parent = |x: Option<Id>| x.filter(|p| *p != keep && *p != remove && db.persons.contains_key(p));
    new.mother = ok_parent(sel.mother);
    new.father = ok_parent(sel.father);
    new.mother_rel = if new.mother == pa.mother { pa.mother_rel } else if new.mother == pb.mother { pb.mother_rel } else { ParentRel::Bio };
    new.deceased = (pa.deceased || pb.deceased) && new.death_date.is_empty();
    // recordings and "through the years" photos of both
    for r in pa.recordings.iter().chain(pb.recordings.iter()) {
        if !new.recordings.iter().any(|x| x.media == r.media) {
            new.recordings.push(r.clone());
        }
    }
    for f in pa.aging.iter().chain(pb.aging.iter()) {
        if !new.aging.iter().any(|x| x.media == f.media) {
            new.aging.push(f.clone());
        }
    }
    new.aging_audio = new.aging_audio.or(pa.aging_audio).or(pb.aging_audio);
    if new.grave.is_empty() {
        new.grave = if pk.grave.is_empty() { pr.grave.clone() } else { pk.grave.clone() };
    }
    new.father_rel = if new.father == pa.father { pa.father_rel } else if new.father == pb.father { pb.father_rel } else { ParentRel::Bio };
    new.events = sel
        .events
        .iter()
        .filter_map(|r| {
            let (tok, idx) = r.split_once(':')?;
            let idx: usize = idx.parse().ok()?;
            match tok {
                "a" => pa.events.get(idx).cloned(),
                "b" => pb.events.get(idx).cloned(),
                _ => None,
            }
        })
        .collect();
    let mut gal: Vec<GalleryItem> = vec![];
    for m in &sel.gallery {
        let cap = pa.gallery.iter().chain(pb.gallery.iter()).filter(|g| g.media == *m).map(|g| g.caption.clone()).find(|c| !c.is_empty()).unwrap_or_default();
        gal.push(GalleryItem { media: *m, caption: cap });
    }
    new.gallery = gal;
    new.clans = sel.clans.clone();
    // custom: kept record first, gaps from the other
    for (g, fields) in &pr.custom {
        for (f, val) in fields {
            let e = new.custom.entry(g.clone()).or_default().entry(f.clone()).or_default();
            if e.is_empty() {
                *e = val.clone();
            }
        }
    }
    // citations: union
    for (k, ids) in pa.citations.iter().chain(pb.citations.iter()) {
        let e = new.citations.entry(k.clone()).or_default();
        for i in ids {
            if !e.contains(i) {
                e.push(*i);
            }
        }
    }
    new.photo = match sel.avatar.as_str() {
        "none" => None,
        "b" => pb.photo.or(pa.photo),
        "a" => pa.photo.or(pb.photo),
        _ => pk.photo.or(pr.photo),
    };
    new.created = match (pk.created, pr.created) {
        (0, x) | (x, 0) => x,
        (x, y) => x.min(y),
    };
    new.id = keep;
    new.updated = ctx.now;
    if new.mother.map(|m| ops::creates_cycle(db, keep, m)).unwrap_or(false) {
        new.mother = None;
    }
    if new.father.map(|m| ops::creates_cycle(db, keep, m)).unwrap_or(false) {
        new.father = None;
    }
    let new = ops::sanitize_person(&new, db);
    db.put_person(new.clone());

    // Children: everything pointing to `remove` now points to `keep`.
    let kids: Vec<Id> = db.persons.values().filter(|c| c.mother == Some(remove) || c.father == Some(remove)).map(|c| c.id).collect();
    for cid in kids {
        let mut c = db.persons[&cid].clone();
        if c.mother == Some(remove) {
            c.mother = Some(keep);
        }
        if c.father == Some(remove) {
            c.father = Some(keep);
        }
        if c.mother == Some(keep) && c.father == Some(keep) {
            let role = ParentRole::for_gender(new.gender).other();
            c.set_parent(role, None, ParentRel::Bio);
        }
        db.put_person(c);
    }
    // Children not selected lose the link to `keep`.
    let selected_kids: HashSet<Id> = sel.children.iter().copied().collect();
    let linked: Vec<Id> = db.persons.values().filter(|c| c.mother == Some(keep) || c.father == Some(keep)).map(|c| c.id).collect();
    for cid in linked {
        if selected_kids.contains(&cid) {
            continue;
        }
        let mut c = db.persons[&cid].clone();
        if c.mother == Some(keep) {
            c.set_parent(ParentRole::Mother, None, ParentRel::Bio);
        }
        if c.father == Some(keep) {
            c.set_parent(ParentRole::Father, None, ParentRel::Bio);
        }
        db.put_person(c);
    }
    // Selected children that were not linked yet (should not happen, but be safe).
    for cid in &sel.children {
        if let Some(c) = db.persons.get(cid) {
            if c.mother != Some(keep) && c.father != Some(keep) && *cid != keep && !ops::creates_cycle(db, *cid, keep) {
                let mut c = c.clone();
                let role = ParentRole::for_gender(new.gender);
                if c.parent(role).is_none() {
                    c.set_parent(role, Some(keep), ParentRel::Bio);
                } else if c.parent(role.other()).is_none() {
                    c.set_parent(role.other(), Some(keep), ParentRel::Bio);
                }
                db.put_person(c);
            }
        }
    }

    // Unions: repoint, deduplicate, drop the ones not selected.
    let uids: Vec<Id> = db.unions.values().filter(|u| u.involves(remove)).map(|u| u.id).collect();
    for uid in uids {
        let mut u = db.unions[&uid].clone();
        if u.a == remove {
            u.a = keep;
        }
        if u.b == Some(remove) {
            u.b = Some(keep);
        }
        if u.b == Some(u.a) {
            db.del_union(uid);
            continue;
        }
        let dup = db.unions.values().find(|x| x.id != uid && x.involves(keep) && u.b.is_some() && x.involves(u.partner_of(keep).unwrap_or(0))).map(|x| x.id);
        match dup {
            Some(did) => {
                let mut d = db.unions[&did].clone();
                d.absorb(&u);
                db.put_union(d);
                db.del_union(uid);
            }
            None => db.put_union(u),
        }
    }
    let selected_partners: HashSet<Id> = sel.partners.iter().copied().collect();
    let uids: Vec<Id> = db.unions.values().filter(|u| u.involves(keep)).map(|u| u.id).collect();
    for uid in uids {
        let u = &db.unions[&uid];
        if let Some(p) = u.partner_of(keep) {
            if !selected_partners.contains(&p) {
                db.del_union(uid);
            }
        }
    }

    // Marks on photos.
    let tagged: Vec<Id> = db.media.values().filter(|m| m.tags.iter().any(|t| t.person == Some(remove))).map(|m| m.id).collect();
    for mid in tagged {
        let mut m = db.media[&mid].clone();
        for t in m.tags.iter_mut() {
            if t.person == Some(remove) {
                t.person = Some(keep);
            }
        }
        let mut seen = HashSet::new();
        m.tags.retain(|t| t.person.map(|p| seen.insert(p)).unwrap_or(true));
        db.put_media_meta(m);
    }

    // Pets, sources, suggestions, settings.
    let pets: Vec<Id> = db.pets.values().filter(|p| p.owner == Some(remove)).map(|p| p.id).collect();
    for pid in pets {
        let mut p = db.pets[&pid].clone();
        p.owner = Some(keep);
        db.put_pet(p);
    }
    let selected_sources: HashSet<Id> = sel.sources.iter().copied().collect();
    let sids: Vec<Id> = db.sources.keys().copied().collect();
    for sid in sids {
        let s = &db.sources[&sid];
        let had = s.persons.contains(&keep) || s.persons.contains(&remove);
        let want = selected_sources.contains(&sid);
        if !had && !want {
            continue;
        }
        let mut s = s.clone();
        s.persons.retain(|p| *p != keep && *p != remove);
        if want {
            s.persons.push(keep);
        }
        db.put_source(s);
    }
    let sg: Vec<Id> = db.suggestions.values().filter(|s| s.person == Some(remove)).map(|s| s.id).collect();
    for id in sg {
        let mut s = db.suggestions[&id].clone();
        s.person = Some(keep);
        db.put_suggestion(s);
    }
    let revs: Vec<Id> = db.revisions.values().filter(|r| r.person == remove).map(|r| r.id).collect();
    for r in revs {
        db.del_revision(r);
    }
    if db.settings.tree_start_person == Some(remove) {
        let mut s = db.settings.clone();
        s.tree_start_person = Some(keep);
        db.put_settings(s);
    }
    db.del_person_raw(remove);
    let new_unions: Vec<Union> = db.unions.values().filter(|u| u.involves(keep)).cloned().collect();
    ops::record_revision(db, ctx, &pk, old_unions, &new, &new_unions);
    Ok(keep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::*;
    use chrono::NaiveDate;

    #[test]
    fn detect_and_merge() {
        let c = Ctx::new(1, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let mut db = Database::new(0);
        let mk = |n: &str, by: &str| Person { first_name: n.into(), last_name: "Kowalski".into(), gender: Gender::Male, birth_date: by.into(), ..Default::default() };
        let a = create_person(&mut db, &c, mk("Jan", "1900"), RelationEdits::default()).unwrap();
        let b = create_person(&mut db, &c, mk("Jann", "1901"), RelationEdits::default()).unwrap();
        let _x = create_person(&mut db, &c, mk("Piotr", "1950"), RelationEdits::default()).unwrap();
        let kid = add_relative(&mut db, &c, b, NewRelation::Child, Person { first_name: "Kid".into(), ..Default::default() }, None).unwrap();
        let w = add_relative(&mut db, &c, b, NewRelation::Spouse, Person { first_name: "W".into(), ..Default::default() }, None).unwrap();
        let s = suggestions(&db, 30);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].a, s[0].b), (a, b));
        let cmp = compare(&db, a, b).unwrap();
        assert_eq!(cmp.children.len(), 1);
        let sel = Selection { children: vec![kid], partners: vec![w], ..Default::default() };
        execute(&mut db, &c, a, b, a, sel).unwrap();
        assert!(!db.persons.contains_key(&b));
        assert_eq!(db.persons[&kid].father, Some(a));
        assert!(db.unions.values().any(|u| u.involves(a) && u.involves(w)));
    }
}
