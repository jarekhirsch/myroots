//! Mutating operations (people, relations, unions, pets, sources, media …).
//! Every function keeps the data consistent and records revisions.

use crate::date;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::life::Ctx;
use crate::model::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// How many revisions are kept per person.
pub const HISTORY_LIMIT: usize = 20;

/* ------------------------------------------------------------------ */
/*  Sanitization                                                        */
/* ------------------------------------------------------------------ */

fn clean(s: &str) -> String {
    // single-line text field
    s.replace(['\r', '\n', '\t'], " ").split(' ').filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" ")
}

fn clean_multiline(s: &str) -> String {
    s.replace('\r', "").trim().to_string()
}

/// Minimal server-side guard for rich text: the UI sanitizes with an
/// allow-list before rendering; here we just drop script/style blocks and
/// event-handler attributes so that stored data is never dangerous.
pub fn clean_html(s: &str) -> String {
    let mut out = s.replace('\r', "");
    for tag in ["script", "style", "iframe", "object", "embed"] {
        loop {
            let lower = out.to_ascii_lowercase();
            let Some(start) = lower.find(&format!("<{tag}")) else { break };
            let end = lower[start..]
                .find(&format!("</{tag}>"))
                .map(|e| start + e + tag.len() + 3)
                .or_else(|| lower[start..].find('>').map(|e| start + e + 1))
                .unwrap_or(out.len());
            out.replace_range(start..end, "");
        }
    }
    // on*= attributes and javascript: URLs
    let mut res = String::with_capacity(out.len());
    let mut in_tag = false;
    let chars: Vec<char> = out.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        }
        if in_tag && c.is_whitespace() && i + 3 < chars.len() && chars[i + 1].eq_ignore_ascii_case(&'o') && chars[i + 2].eq_ignore_ascii_case(&'n') && chars[i + 3].is_ascii_alphabetic() {
            // skip attribute name=value
            let mut j = i + 1;
            while j < chars.len() && !chars[j].is_whitespace() && chars[j] != '=' && chars[j] != '>' {
                j += 1;
            }
            if j < chars.len() && chars[j] == '=' {
                j += 1;
                if j < chars.len() && (chars[j] == '"' || chars[j] == '\'') {
                    let q = chars[j];
                    j += 1;
                    while j < chars.len() && chars[j] != q {
                        j += 1;
                    }
                    j += 1;
                } else {
                    while j < chars.len() && !chars[j].is_whitespace() && chars[j] != '>' {
                        j += 1;
                    }
                }
            }
            i = j;
            continue;
        }
        res.push(c);
        i += 1;
    }
    let lower = res.to_ascii_lowercase();
    if lower.contains("javascript:") {
        // case-insensitive removal
        let mut r = String::new();
        let mut rest = res.as_str();
        loop {
            let l = rest.to_ascii_lowercase();
            match l.find("javascript:") {
                Some(p) => {
                    r.push_str(&rest[..p]);
                    rest = &rest[p + "javascript:".len()..];
                }
                None => {
                    r.push_str(rest);
                    break;
                }
            }
        }
        res = r;
    }
    res.trim().to_string()
}

fn dedup_strings(v: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    v.iter().map(|s| clean(s)).filter(|s| !s.is_empty() && seen.insert(s.to_lowercase())).collect()
}

fn clean_events(events: &[LifeEvent], db: &Database) -> Vec<LifeEvent> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for e in events {
        let ev = LifeEvent {
            name: clean(&e.name),
            date: date::sanitize(&e.date),
            place: clean(&e.place),
            description: clean_html(&e.description),
            image: e.image.filter(|m| db.media.contains_key(m)),
        };
        if ev.name.is_empty() && ev.date.is_empty() && ev.description.is_empty() && ev.image.is_none() {
            continue;
        }
        let sig = format!("{}|{}|{}", ev.name, ev.date, ev.description);
        if seen.insert(sig) {
            out.push(ev);
        }
    }
    out
}

fn clean_gallery(items: &[GalleryItem], db: &Database) -> Vec<GalleryItem> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|g| db.media.contains_key(&g.media) && seen.insert(g.media))
        .map(|g| GalleryItem { media: g.media, caption: clean(&g.caption) })
        .collect()
}

fn clean_grave(g: &Grave, db: &Database) -> Grave {
    let lat = g.lat.filter(|v| v.is_finite() && (-90.0..=90.0).contains(v));
    let lng = g.lng.filter(|v| v.is_finite() && (-180.0..=180.0).contains(v));
    let (lat, lng) = if lat.is_some() && lng.is_some() { (lat, lng) } else { (None, None) };
    Grave {
        cemetery: clean(&g.cemetery),
        location: clean(&g.location),
        lat,
        lng,
        photo: g.photo.filter(|m| db.media.contains_key(m)),
        url: safe_url(&g.url),
        notes: clean_multiline(&g.notes),
    }
}

fn clean_recordings(items: &[Recording], db: &Database) -> Vec<Recording> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter(|r| db.media.contains_key(&r.media) && seen.insert(r.media))
        .map(|r| Recording { media: r.media, title: clean(&r.title), date: date::sanitize(&r.date) })
        .collect()
}

fn clean_aging(items: &[AgingFrame], db: &Database) -> Vec<AgingFrame> {
    let mut seen = HashSet::new();
    let mut out: Vec<AgingFrame> = items
        .iter()
        .filter(|f| db.media.contains_key(&f.media) && seen.insert(f.media))
        .map(|f| AgingFrame {
            media: f.media,
            date: date::sanitize(&f.date),
            eyes: f.eyes.filter(|e| e.iter().all(|v| v.is_finite())).map(|e| e.map(|v| v.clamp(0.0, 1.0))),
        })
        .collect();
    // oldest photo first (undated ones keep their place at the end)
    out.sort_by_key(|f| if f.date.is_empty() { (1, String::new()) } else { (0, date::sort_key(&f.date)) });
    out
}

/// Normalizes a person coming from the UI (ids of relations are validated separately).
pub fn sanitize_person(p: &Person, db: &Database) -> Person {
    let mut d = p.clone();
    d.first_name = clean(&d.first_name);
    d.last_name = clean(&d.last_name);
    d.maiden_name = clean(&d.maiden_name);
    d.birth_date = date::sanitize(&d.birth_date);
    d.death_date = date::sanitize(&d.death_date);
    d.birth_place = clean(&d.birth_place);
    d.death_place = clean(&d.death_place);
    d.death_cause = clean(&d.death_cause);
    d.biography = clean_html(&d.biography);
    d.notes = clean_html(&d.notes);
    for f in [&mut d.occupation, &mut d.education, &mut d.faith, &mut d.height, &mut d.eye_color, &mut d.blood_type] {
        *f = clean(f);
    }
    d.events = clean_events(&d.events, db);
    d.clans = dedup_strings(&d.clans);
    d.photo = d.photo.filter(|m| db.media.contains_key(m));
    d.gallery = clean_gallery(&d.gallery, db);
    d.recordings = clean_recordings(&d.recordings, db);
    d.aging = clean_aging(&d.aging, db);
    d.aging_audio = d.aging_audio.filter(|a| d.recordings.iter().any(|r| r.media == *a));
    d.grave = clean_grave(&d.grave, db);
    let mut custom = BTreeMap::new();
    let url_fields: HashSet<(String, String)> = db
        .settings
        .custom_schema
        .iter()
        .flat_map(|g| g.fields.iter().filter(|f| f.kind == "url").map(move |f| (g.id.clone(), f.id.clone())))
        .collect();
    for (g, fields) in &d.custom {
        let gid = key(g);
        if gid.is_empty() {
            continue;
        }
        let mut m = BTreeMap::new();
        for (f, v) in fields {
            let fid = key(f);
            let mut v = clean_multiline(v);
            if url_fields.contains(&(gid.clone(), fid.clone())) {
                v = safe_url(&v);
            }
            if !fid.is_empty() && !v.is_empty() {
                m.insert(fid, v);
            }
        }
        if !m.is_empty() {
            custom.insert(gid, m);
        }
    }
    d.custom = custom;
    let mut cit = BTreeMap::new();
    for (f, ids) in &d.citations {
        let fk = key(f);
        let mut v: Vec<Id> = Vec::new();
        for id in ids {
            if db.sources.contains_key(id) && !v.contains(id) {
                v.push(*id);
            }
        }
        if !fk.is_empty() && !v.is_empty() {
            cit.insert(fk, v);
        }
    }
    d.citations = cit;
    d
}

/// sanitize_key(): lowercase [a-z0-9_-].
pub fn key(s: &str) -> String {
    s.trim().to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').collect()
}

/* ------------------------------------------------------------------ */
/*  Relations guard                                                     */
/* ------------------------------------------------------------------ */

/// Would setting `parent` as a parent of `child` create an ancestor loop?
pub fn creates_cycle(db: &Database, child: Id, parent: Id) -> bool {
    if child == parent {
        return true;
    }
    // Is `child` an ancestor of `parent`?
    let mut stack = vec![parent];
    let mut seen = HashSet::new();
    while let Some(n) = stack.pop() {
        if n == child {
            return true;
        }
        if !seen.insert(n) {
            continue;
        }
        if let Some(p) = db.persons.get(&n) {
            stack.extend(p.parents());
        }
    }
    false
}

/// True when one person is an ancestor of the other (or they are the same person).
pub fn in_direct_line(db: &Database, a: Id, b: Id) -> bool {
    creates_cycle(db, a, b) || creates_cycle(db, b, a)
}

/// Partners added by hand cannot be the same person, or ancestor and descendant.
pub fn check_partners(db: &Database, a: Id, b: Id) -> Result<()> {
    if in_direct_line(db, a, b) {
        return Err(Error::new("relation", "partners cannot be a parent and a child, or an ancestor and a descendant"));
    }
    Ok(())
}

fn validate_parent(db: &Database, child: Id, parent: Option<Id>) -> Result<Option<Id>> {
    match parent {
        None | Some(0) => Ok(None),
        Some(pid) => {
            if !db.persons.contains_key(&pid) {
                return Err(Error::not_found("person", pid));
            }
            if child != 0 && creates_cycle(db, child, pid) {
                return Err(Error::new("cycle", "this relation would make the person their own ancestor"));
            }
            Ok(Some(pid))
        }
    }
}

/* ------------------------------------------------------------------ */
/*  History                                                             */
/* ------------------------------------------------------------------ */

fn unions_of(db: &Database, id: Id) -> Vec<Union> {
    db.unions.values().filter(|u| u.involves(id)).cloned().collect()
}

fn changed_fields(old: &Person, new: &Person, old_unions: &[Union], new_unions: &[Union]) -> Vec<String> {
    let a = serde_json::to_value(old).unwrap();
    let b = serde_json::to_value(new).unwrap();
    let mut out = Vec::new();
    if let (Some(a), Some(b)) = (a.as_object(), b.as_object()) {
        for (k, va) in a {
            if matches!(k.as_str(), "updated" | "created" | "id") {
                continue;
            }
            if b.get(k) != Some(va) {
                out.push(k.clone());
            }
        }
    }
    let norm = |v: &[Union]| {
        let mut v: Vec<String> = v.iter().map(|u| serde_json::to_string(u).unwrap()).collect();
        v.sort();
        v
    };
    if norm(old_unions) != norm(new_unions) {
        out.push("spouses".into());
    }
    out
}

/// Records a snapshot of the previous state (only when something changed).
pub fn record_revision(db: &mut Database, ctx: &Ctx, old: &Person, old_unions: Vec<Union>, new: &Person, new_unions: &[Union]) {
    let fields = changed_fields(old, new, &old_unions, new_unions);
    if fields.is_empty() {
        return;
    }
    let id = db.next_id();
    db.put_revision(Revision { id, person: old.id, time: ctx.now, fields, data: old.clone(), unions: old_unions });
    prune_history(db, old.id);
}

fn prune_history(db: &mut Database, person: Id) {
    let mut revs: Vec<(i64, Id)> = db.revisions.values().filter(|r| r.person == person).map(|r| (r.time, r.id)).collect();
    if revs.len() <= HISTORY_LIMIT {
        return;
    }
    revs.sort_by(|a, b| b.cmp(a));
    for (_, id) in revs.into_iter().skip(HISTORY_LIMIT) {
        db.del_revision(id);
    }
}

/* ------------------------------------------------------------------ */
/*  People                                                              */
/* ------------------------------------------------------------------ */

/// Optional relation edits sent together with a person.
#[derive(Deserialize, Serialize, Default, Debug, Clone)]
#[serde(default)]
pub struct RelationEdits {
    /// Full desired list of unions of this person (None = leave unchanged).
    pub unions: Option<Vec<Union>>,
    /// Full desired list of children (None = leave unchanged).
    pub children: Option<Vec<Id>>,
}

pub fn create_person(db: &mut Database, ctx: &Ctx, mut p: Person, rel: RelationEdits) -> Result<Id> {
    p.mother = validate_parent(db, 0, p.mother)?;
    p.father = validate_parent(db, 0, p.father)?;
    let mut p = sanitize_person(&p, db);
    auto_maiden_name(db, &mut p);
    let id = db.next_id();
    p.id = id;
    p.created = ctx.now;
    p.updated = ctx.now;
    db.put_person(p);
    apply_relation_edits(db, id, &rel)?;
    Ok(id)
}

/// Updates only the recordings and the "Through the years" photos of a person.
pub fn set_person_media(
    db: &mut Database,
    ctx: &Ctx,
    id: Id,
    recordings: Option<Vec<Recording>>,
    aging: Option<Vec<AgingFrame>>,
    aging_audio: Option<Option<Id>>,
    add_to_gallery: &[Id],
) -> Result<Person> {
    let old = db.persons.get(&id).cloned().ok_or_else(|| Error::not_found("person", id))?;
    let mut new = old.clone();
    for m in add_to_gallery {
        if db.media.contains_key(m) && new.photo != Some(*m) && !new.gallery.iter().any(|g| g.media == *m) {
            new.gallery.push(GalleryItem { media: *m, caption: String::new() });
        }
    }
    if let Some(r) = recordings {
        new.recordings = clean_recordings(&r, db);
    }
    if let Some(a) = aging {
        new.aging = clean_aging(&a, db);
    }
    if let Some(a) = aging_audio {
        new.aging_audio = a;
    }
    new.aging_audio = new.aging_audio.filter(|a| new.recordings.iter().any(|r| r.media == *a));
    new.updated = ctx.now;
    db.put_person(new.clone());
    let unions = unions_of(db, id);
    record_revision(db, ctx, &old, unions.clone(), &new, &unions);
    Ok(new)
}

pub fn update_person(db: &mut Database, ctx: &Ctx, p: Person, rel: RelationEdits) -> Result<()> {
    let id = p.id;
    let old = db.persons.get(&id).cloned().ok_or_else(|| Error::not_found("person", id))?;
    let old_unions = unions_of(db, id);
    let mut p = p;
    p.mother = validate_parent(db, id, p.mother)?;
    p.father = validate_parent(db, id, p.father)?;
    let mut new = sanitize_person(&p, db);
    auto_maiden_name(db, &mut new);
    new.created = old.created;
    new.updated = ctx.now;
    db.put_person(new.clone());
    apply_relation_edits(db, id, &rel)?;
    let new_unions = unions_of(db, id);
    record_revision(db, ctx, &old, old_unions, &new, &new_unions);
    Ok(())
}

/// A woman with a father and an empty maiden name gets the father's surname
/// (behaviour of the WordPress plugin).
pub fn fill_maiden_name(db: &Database, p: &mut Person) {
    auto_maiden_name(db, p)
}

fn auto_maiden_name(db: &Database, p: &mut Person) {
    if p.gender == Gender::Female && p.maiden_name.is_empty() {
        if let Some(f) = p.father.and_then(|f| db.persons.get(&f)) {
            let fem = feminine_surname(f.last_name.trim());
            if !fem.is_empty() && norm(&fem) != norm(&p.last_name) && norm(f.last_name.trim()) != norm(&p.last_name) {
                p.maiden_name = fem;
            }
        }
    }
}

/// Feminine form of a surname in languages that inflect surnames by gender
/// (Polish -ski → -ska, -cki → -cka, -dzki → -dzka, Czech/Slovak -ý → -á).
/// Other surnames are returned unchanged.
pub fn feminine_surname(s: &str) -> String {
    let parts: Vec<String> = s
        .split('-')
        .map(|w| {
            let lower = w.to_lowercase();
            for (m, f) in [("ski", "ska"), ("cki", "cka"), ("dzki", "dzka"), ("ský", "ská"), ("cký", "cká")] {
                if lower.ends_with(m) && w.chars().count() > m.chars().count() + 1 {
                    let cut: String = w.chars().take(w.chars().count() - m.chars().count()).collect();
                    return format!("{cut}{f}");
                }
            }
            if lower.ends_with('ý') && w.chars().count() > 3 {
                let cut: String = w.chars().take(w.chars().count() - 1).collect();
                return format!("{cut}á");
            }
            w.to_string()
        })
        .collect();
    parts.join("-")
}

fn apply_relation_edits(db: &mut Database, id: Id, rel: &RelationEdits) -> Result<()> {
    if let Some(children) = &rel.children {
        set_children(db, id, children)?;
    }
    if let Some(unions) = &rel.unions {
        set_unions(db, id, unions)?;
    }
    Ok(())
}

/// Makes `children` exactly the list of children of `parent`.
pub fn set_children(db: &mut Database, parent: Id, children: &[Id]) -> Result<()> {
    let gender = db.persons.get(&parent).map(|p| p.gender).unwrap_or_default();
    let wanted: BTreeSet<Id> = children.iter().copied().filter(|c| *c != parent && db.persons.contains_key(c)).collect();
    let current: Vec<Id> = db.persons.values().filter(|c| c.mother == Some(parent) || c.father == Some(parent)).map(|c| c.id).collect();
    for cid in &current {
        if !wanted.contains(cid) {
            let mut c = db.persons[cid].clone();
            if c.mother == Some(parent) {
                c.set_parent(ParentRole::Mother, None, ParentRel::Bio);
            }
            if c.father == Some(parent) {
                c.set_parent(ParentRole::Father, None, ParentRel::Bio);
            }
            db.put_person(c);
        }
    }
    for cid in wanted {
        if current.contains(&cid) {
            continue;
        }
        if creates_cycle(db, cid, parent) {
            return Err(Error::new("cycle", "this relation would make the person their own ancestor"));
        }
        let mut c = db.persons[&cid].clone();
        let role = ParentRole::for_gender(gender);
        if c.parent(role).is_none() {
            c.set_parent(role, Some(parent), ParentRel::Bio);
        } else if gender == Gender::Unset || gender == Gender::Unknown {
            if c.parent(role.other()).is_none() {
                c.set_parent(role.other(), Some(parent), ParentRel::Bio);
            } else {
                c.set_parent(role, Some(parent), ParentRel::Bio);
            }
        } else {
            c.set_parent(role, Some(parent), ParentRel::Bio);
        }
        db.put_person(c);
    }
    Ok(())
}

/// Makes `unions` exactly the set of unions involving `person`.
pub fn set_unions(db: &mut Database, person: Id, unions: &[Union]) -> Result<()> {
    let existing: Vec<Id> = db.unions.values().filter(|u| u.involves(person)).map(|u| u.id).collect();
    let mut kept = HashSet::new();
    let mut partners_seen = HashSet::new();
    for u in unions {
        let mut u = sanitize_union(u);
        // Normalize: `person` is one of the partners.
        if !u.involves(person) {
            let partner = if u.a != 0 && u.a != person { Some(u.a) } else { u.b };
            u.a = person;
            u.b = partner;
        }
        if let Some(b) = u.b {
            if b == person || !db.persons.contains_key(&b) {
                u.b = None;
            }
        }
        if u.b.is_none() && u.partner_first_name.is_empty() && u.partner_last_name.is_empty() && u.date.is_empty() && u.place.is_empty() {
            continue; // empty row
        }
        if let Some(b) = u.b {
            if !partners_seen.insert(b) {
                continue; // one union per couple
            }
            check_partners(db, person, b)?;
        }
        if u.id != 0 && existing.contains(&u.id) {
            kept.insert(u.id);
            db.put_union(u);
        } else {
            // Reuse an existing union of the same couple (created from the other side).
            let dup = u.b.and_then(|b| db.unions.values().find(|x| x.involves(person) && x.involves(b)).map(|x| x.id));
            match dup {
                Some(did) => {
                    kept.insert(did);
                    u.id = did;
                    db.put_union(u);
                }
                None => {
                    u.id = db.next_id();
                    kept.insert(u.id);
                    db.put_union(u);
                }
            }
        }
    }
    for uid in existing {
        if !kept.contains(&uid) {
            db.del_union(uid);
        }
    }
    Ok(())
}

pub fn sanitize_union(u: &Union) -> Union {
    let mut u = u.clone();
    u.partner_first_name = clean(&u.partner_first_name);
    u.partner_last_name = clean(&u.partner_last_name);
    u.date = date::sanitize(&u.date);
    u.end_date = date::sanitize(&u.end_date);
    u.place = clean(&u.place);
    u.end_cause = end_cause::normalize(&u.end_cause);
    if u.end_cause == end_cause::ONGOING {
        u.end_cause.clear();
    }
    u
}

pub fn save_union(db: &mut Database, u: Union) -> Result<Id> {
    let mut u = sanitize_union(&u);
    if !db.persons.contains_key(&u.a) {
        return Err(Error::not_found("person", u.a));
    }
    if let Some(b) = u.b {
        if b == u.a {
            return Err(Error::invalid("a person cannot be their own spouse"));
        }
        if !db.persons.contains_key(&b) {
            return Err(Error::not_found("person", b));
        }
        if let Some(other) = db.unions.values().find(|x| x.id != u.id && x.involves(u.a) && x.involves(b)) {
            u.id = other.id;
        }
    }
    if u.id == 0 {
        u.id = db.next_id();
    }
    let id = u.id;
    db.put_union(u);
    Ok(id)
}

/// Links two people as partners (no-op when already linked).
pub fn link_partners(db: &mut Database, a: Id, b: Id, details: Option<&Union>) -> Result<Id> {
    if let Some(u) = db.unions.values().find(|x| x.involves(a) && x.involves(b)).cloned() {
        if let Some(d) = details {
            let mut u2 = u.clone();
            u2.absorb(d);
            if u2 != u {
                db.put_union(u2);
            }
        }
        return Ok(u.id);
    }
    let mut u = details.cloned().unwrap_or_default();
    u.id = 0;
    u.a = a;
    u.b = Some(b);
    save_union(db, u)
}

pub fn set_parent(db: &mut Database, ctx: &Ctx, child: Id, role: ParentRole, parent: Option<Id>, rel: ParentRel) -> Result<()> {
    let old = db.persons.get(&child).cloned().ok_or_else(|| Error::not_found("person", child))?;
    let parent = validate_parent(db, child, parent)?;
    let mut c = old.clone();
    c.set_parent(role, parent, rel);
    c.updated = ctx.now;
    auto_maiden_name(db, &mut c);
    let unions = unions_of(db, child);
    db.put_person(c.clone());
    record_revision(db, ctx, &old, unions.clone(), &c, &unions);
    Ok(())
}

/// Adds a new person related to `of` (tree "add relative").
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum NewRelation {
    Father,
    Mother,
    Child,
    Spouse,
    Sibling,
}

pub fn add_relative(db: &mut Database, ctx: &Ctx, of: Id, relation: NewRelation, mut p: Person, other_parent: Option<Id>) -> Result<Id> {
    let base = db.persons.get(&of).cloned().ok_or_else(|| Error::not_found("person", of))?;
    match relation {
        NewRelation::Father | NewRelation::Mother => {
            let role = if matches!(relation, NewRelation::Father) { ParentRole::Father } else { ParentRole::Mother };
            if p.gender == Gender::Unset {
                p.gender = if role == ParentRole::Father { Gender::Male } else { Gender::Female };
            }
            let id = create_person(db, ctx, p, RelationEdits::default())?;
            set_parent(db, ctx, of, role, Some(id), ParentRel::Bio)?;
            // The new parent becomes the partner of the other existing parent.
            if let Some(other) = base.parent(role.other()) {
                link_partners(db, id, other, None)?;
            }
            Ok(id)
        }
        NewRelation::Child => {
            let role = ParentRole::for_gender(base.gender);
            match role {
                ParentRole::Mother => p.mother = Some(of),
                ParentRole::Father => p.father = Some(of),
            }
            let second = other_parent.or_else(|| {
                // single partner → assume the other parent
                let ix = db.index();
                let partners = ix.partners(db, of);
                if partners.len() == 1 {
                    Some(partners[0])
                } else {
                    None
                }
            });
            if let Some(s) = second {
                if db.persons.contains_key(&s) && s != of {
                    match role.other() {
                        ParentRole::Mother => p.mother = Some(s),
                        ParentRole::Father => p.father = Some(s),
                    }
                }
            }
            create_person(db, ctx, p, RelationEdits::default())
        }
        NewRelation::Spouse => {
            if p.gender == Gender::Unset {
                p.gender = match base.gender {
                    Gender::Male => Gender::Female,
                    Gender::Female => Gender::Male,
                    g => g,
                };
            }
            let id = create_person(db, ctx, p, RelationEdits::default())?;
            link_partners(db, of, id, None)?;
            Ok(id)
        }
        NewRelation::Sibling => {
            p.mother = base.mother;
            p.father = base.father;
            create_person(db, ctx, p, RelationEdits::default())
        }
    }
}

/// Deletes a person and every reference to them.
pub fn delete_person(db: &mut Database, id: Id) -> Result<()> {
    let Some(gone) = db.del_person_raw(id) else {
        return Err(Error::not_found("person", id));
    };
    // Marks on group photos keep the name only.
    let name = gone.full_name();
    let tagged: Vec<Id> = db.media.values().filter(|m| m.tags.iter().any(|t| t.person == Some(id))).map(|m| m.id).collect();
    for mid in tagged {
        let mut m = db.media[&mid].clone();
        for t in m.tags.iter_mut().filter(|t| t.person == Some(id)) {
            t.person = None;
            if t.name.is_empty() {
                t.name = name.clone();
            }
        }
        db.put_media_meta(m);
    }
    let children: Vec<Id> = db.persons.values().filter(|c| c.mother == Some(id) || c.father == Some(id)).map(|c| c.id).collect();
    for cid in children {
        let mut c = db.persons[&cid].clone();
        if c.mother == Some(id) {
            c.set_parent(ParentRole::Mother, None, ParentRel::Bio);
        }
        if c.father == Some(id) {
            c.set_parent(ParentRole::Father, None, ParentRel::Bio);
        }
        db.put_person(c);
    }
    let unions: Vec<Id> = db.unions.values().filter(|u| u.involves(id)).map(|u| u.id).collect();
    for u in unions {
        db.del_union(u);
    }
    let pets: Vec<Id> = db.pets.values().filter(|p| p.owner == Some(id)).map(|p| p.id).collect();
    for pid in pets {
        let mut p = db.pets[&pid].clone();
        p.owner = None;
        db.put_pet(p);
    }
    let sources: Vec<Id> = db.sources.values().filter(|s| s.persons.contains(&id)).map(|s| s.id).collect();
    for sid in sources {
        let mut s = db.sources[&sid].clone();
        s.persons.retain(|x| *x != id);
        db.put_source(s);
    }
    let revs: Vec<Id> = db.revisions.values().filter(|r| r.person == id).map(|r| r.id).collect();
    for r in revs {
        db.del_revision(r);
    }
    let sugg: Vec<Id> = db.suggestions.values().filter(|s| s.person == Some(id)).map(|s| s.id).collect();
    for sid in sugg {
        let mut s = db.suggestions[&sid].clone();
        s.person = None;
        db.put_suggestion(s);
    }
    if db.settings.tree_start_person == Some(id) {
        let mut s = db.settings.clone();
        s.tree_start_person = None;
        db.put_settings(s);
    }
    Ok(())
}

/// Restores a revision (the current state is recorded as a new revision first).
pub fn restore_revision(db: &mut Database, ctx: &Ctx, rev_id: Id) -> Result<()> {
    let rev = db.revisions.get(&rev_id).cloned().ok_or_else(|| Error::not_found("revision", rev_id))?;
    let id = rev.person;
    if !db.persons.contains_key(&id) {
        return Err(Error::not_found("person", id));
    }
    let mut p = rev.data.clone();
    p.id = id;
    // Relations pointing to people that no longer exist are dropped.
    if p.mother.map(|m| !db.persons.contains_key(&m) || creates_cycle(db, id, m)).unwrap_or(false) {
        p.mother = None;
    }
    if p.father.map(|m| !db.persons.contains_key(&m) || creates_cycle(db, id, m)).unwrap_or(false) {
        p.father = None;
    }
    let unions: Vec<Union> = rev.unions.iter().filter(|u| u.b.map(|b| db.persons.contains_key(&b)).unwrap_or(true)).cloned().collect();
    update_person(db, ctx, p, RelationEdits { unions: Some(unions), children: None })
}

/* ------------------------------------------------------------------ */
/*  Pets                                                                */
/* ------------------------------------------------------------------ */

pub fn save_pet(db: &mut Database, ctx: &Ctx, p: Pet) -> Result<Id> {
    let mut p = p;
    p.name = clean(&p.name);
    p.species = key(&p.species);
    p.breed = clean(&p.breed);
    p.birth_date = date::sanitize(&p.birth_date);
    p.death_date = date::sanitize(&p.death_date);
    p.biography = clean_html(&p.biography);
    p.notes = clean_html(&p.notes);
    p.owner = p.owner.filter(|o| db.persons.contains_key(o));
    p.photo = p.photo.filter(|m| db.media.contains_key(m));
    p.gallery = clean_gallery(&p.gallery, db);
    if p.gender == Gender::Unknown {
        p.gender = Gender::Unset;
    }
    if p.id == 0 || !db.pets.contains_key(&p.id) {
        p.id = db.next_id();
        p.created = ctx.now;
    } else {
        p.created = db.pets[&p.id].created;
    }
    p.updated = ctx.now;
    let id = p.id;
    db.put_pet(p);
    Ok(id)
}

/* ------------------------------------------------------------------ */
/*  Sources                                                             */
/* ------------------------------------------------------------------ */

pub fn save_source(db: &mut Database, ctx: &Ctx, s: Source) -> Result<Id> {
    let mut s = s;
    s.title = clean(&s.title);
    s.date = clean(&s.date);
    s.url = safe_url(&clean(&s.url));
    s.media_url = safe_url(&clean(&s.media_url));
    s.notes = clean_multiline(&s.notes);
    s.photo = s.photo.filter(|m| db.media.contains_key(m));
    let mut seen = HashSet::new();
    s.scans.retain(|m| db.media.contains_key(m) && seen.insert(*m));
    let mut seen = HashSet::new();
    s.persons.retain(|p| db.persons.contains_key(p) && seen.insert(*p));
    if s.title.is_empty() {
        return Err(Error::invalid("the source needs a title"));
    }
    if s.id == 0 || !db.sources.contains_key(&s.id) {
        s.id = db.next_id();
        s.created = ctx.now;
    } else {
        s.created = db.sources[&s.id].created;
    }
    s.updated = ctx.now;
    let id = s.id;
    db.put_source(s);
    Ok(id)
}

pub fn delete_source(db: &mut Database, id: Id) -> Result<()> {
    if db.del_source(id).is_none() {
        return Err(Error::not_found("source", id));
    }
    let affected: Vec<Id> = db.persons.values().filter(|p| p.citations.values().any(|v| v.contains(&id))).map(|p| p.id).collect();
    for pid in affected {
        let mut p = db.persons[&pid].clone();
        for v in p.citations.values_mut() {
            v.retain(|x| *x != id);
        }
        p.citations.retain(|_, v| !v.is_empty());
        db.put_person(p);
    }
    Ok(())
}

/// Sets the general source links of a person (the person side of the M:N relation).
pub fn set_person_sources(db: &mut Database, person: Id, sources: &[Id]) -> Result<()> {
    if !db.persons.contains_key(&person) {
        return Err(Error::not_found("person", person));
    }
    let wanted: HashSet<Id> = sources.iter().copied().filter(|s| db.sources.contains_key(s)).collect();
    let ids: Vec<Id> = db.sources.keys().copied().collect();
    for sid in ids {
        let s = &db.sources[&sid];
        let has = s.persons.contains(&person);
        let want = wanted.contains(&sid);
        if has != want {
            let mut s = s.clone();
            if want {
                s.persons.push(person);
            } else {
                s.persons.retain(|x| *x != person);
            }
            db.put_source(s);
        }
    }
    Ok(())
}

/* ------------------------------------------------------------------ */
/*  Media                                                               */
/* ------------------------------------------------------------------ */

pub fn register_media(db: &mut Database, ctx: &Ctx, mut m: Media) -> Id {
    // The platform usually reserves the id first (bytes are stored under it).
    if m.id == 0 {
        m.id = db.next_id();
    }
    m.file_name = clean(&m.file_name);
    m.caption = clean(&m.caption);
    m.mime = safe_mime(&m.mime);
    m.tags = clean_tags(&m.tags, db);
    m.date = date::sanitize(&m.date);
    if m.created == 0 {
        m.created = ctx.now;
    }
    let id = m.id;
    db.put_media_meta(m);
    id
}

/// Reserves a new media id (the platform stores the bytes under it first).
pub fn reserve_media_id(db: &mut Database) -> Id {
    db.next_id()
}

/// Deletes a media item and every reference to it.
pub fn delete_media(db: &mut Database, id: Id) {
    let persons: Vec<Id> = db
        .persons
        .values()
        .filter(|p| {
            p.photo == Some(id)
                || p.gallery.iter().any(|g| g.media == id)
                || p.events.iter().any(|e| e.image == Some(id))
                || p.recordings.iter().any(|r| r.media == id)
                || p.aging.iter().any(|f| f.media == id)
                || p.grave.photo == Some(id)
        })
        .map(|p| p.id)
        .collect();
    for pid in persons {
        let mut p = db.persons[&pid].clone();
        if p.photo == Some(id) {
            p.photo = None;
        }
        p.gallery.retain(|g| g.media != id);
        p.recordings.retain(|r| r.media != id);
        p.aging.retain(|f| f.media != id);
        if p.aging_audio == Some(id) {
            p.aging_audio = None;
        }
        if p.grave.photo == Some(id) {
            p.grave.photo = None;
        }
        for e in &mut p.events {
            if e.image == Some(id) {
                e.image = None;
            }
        }
        db.put_person(p);
    }
    let pets: Vec<Id> = db.pets.values().filter(|p| p.photo == Some(id) || p.gallery.iter().any(|g| g.media == id)).map(|p| p.id).collect();
    for pid in pets {
        let mut p = db.pets[&pid].clone();
        if p.photo == Some(id) {
            p.photo = None;
        }
        p.gallery.retain(|g| g.media != id);
        db.put_pet(p);
    }
    let sources: Vec<Id> = db.sources.values().filter(|s| s.photo == Some(id) || s.scans.contains(&id)).map(|s| s.id).collect();
    for sid in sources {
        let mut s = db.sources[&sid].clone();
        if s.photo == Some(id) {
            s.photo = None;
        }
        s.scans.retain(|m| *m != id);
        db.put_source(s);
    }
    db.delete_media(id);
}

/// Cleans the people marked on a photo: boxes inside the image, known people
/// only (unknown ids keep the name), no empty marks.
pub fn clean_tags(tags: &[PhotoTag], db: &Database) -> Vec<PhotoTag> {
    let mut out: Vec<PhotoTag> = Vec::new();
    for t in tags.iter().take(300) {
        let r = t.rect;
        if r.iter().any(|v| !v.is_finite()) {
            continue;
        }
        let x = r[0].clamp(0.0, 1.0);
        let y = r[1].clamp(0.0, 1.0);
        let w = r[2].clamp(0.0, 1.0 - x);
        let h = r[3].clamp(0.0, 1.0 - y);
        if w < 0.005 || h < 0.005 {
            continue;
        }
        let person = t.person.filter(|p| db.persons.contains_key(p));
        let name: String = clean(&t.name).chars().take(200).collect();
        if person.is_none() && name.is_empty() {
            continue;
        }
        if person.is_some() && out.iter().any(|o| o.person == person) {
            continue;
        }
        let face = if t.face.len() == 128 && t.face.iter().all(|v| v.is_finite() && v.abs() < 10.0) { t.face.clone() } else { Vec::new() };
        out.push(PhotoTag { person, name: if person.is_some() { String::new() } else { name }, rect: [x, y, w, h], face });
    }
    out
}

/// Sets the people marked on a photo.
pub fn set_media_tags(db: &mut Database, id: Id, tags: &[PhotoTag]) -> Result<Vec<PhotoTag>> {
    let mut m = db.media.get(&id).cloned().ok_or_else(|| Error::not_found("media", id))?;
    // the client does not get the face descriptors back: keep them for unchanged boxes
    let mut tags = tags.to_vec();
    for t in tags.iter_mut().filter(|t| t.face.is_empty()) {
        if let Some(old) = m.tags.iter().find(|o| o.rect.iter().zip(t.rect.iter()).all(|(a, b)| (a - b).abs() < 1e-4)) {
            t.face = old.face.clone();
        }
    }
    m.tags = clean_tags(&tags, db);
    let out = m.tags.clone();
    db.put_media_meta(m);
    Ok(out)
}

/// Ids of media not referenced anywhere.
pub fn unused_media(db: &Database) -> Vec<Id> {
    let mut used = HashSet::new();
    for m in db.media.values() {
        if m.tags.iter().any(|t| t.person.is_some()) {
            used.insert(m.id);
        }
    }
    for p in db.persons.values() {
        used.extend(p.photo);
        used.extend(p.gallery.iter().map(|g| g.media));
        used.extend(p.events.iter().filter_map(|e| e.image));
        used.extend(p.recordings.iter().map(|r| r.media));
        used.extend(p.aging.iter().map(|f| f.media));
        used.extend(p.grave.photo);
    }
    for p in db.pets.values() {
        used.extend(p.photo);
        used.extend(p.gallery.iter().map(|g| g.media));
    }
    for s in db.sources.values() {
        used.extend(s.photo);
        used.extend(s.scans.iter().copied());
    }
    for r in db.revisions.values() {
        used.extend(r.data.photo);
        used.extend(r.data.gallery.iter().map(|g| g.media));
        used.extend(r.data.recordings.iter().map(|x| x.media));
        used.extend(r.data.aging.iter().map(|f| f.media));
        used.extend(r.data.grave.photo);
    }
    db.media.keys().copied().filter(|m| !used.contains(m)).collect()
}

/* ------------------------------------------------------------------ */
/*  Clans                                                               */
/* ------------------------------------------------------------------ */

pub fn rename_clan(db: &mut Database, from: &str, to: &str) -> Result<usize> {
    let to = clean(to);
    if to.is_empty() {
        return Err(Error::invalid("empty clan name"));
    }
    let ids: Vec<Id> = db.persons.values().filter(|p| p.clans.iter().any(|c| c == from)).map(|p| p.id).collect();
    for id in &ids {
        let mut p = db.persons[id].clone();
        for c in &mut p.clans {
            if c == from {
                *c = to.clone();
            }
        }
        p.clans = dedup_strings(&p.clans);
        db.put_person(p);
    }
    let mut s = db.settings.clone();
    for c in &mut s.clans {
        if c == from {
            *c = to.clone();
        }
    }
    s.clans = dedup_strings(&s.clans);
    db.put_settings(s);
    Ok(ids.len())
}

pub fn delete_clan(db: &mut Database, name: &str) -> usize {
    let ids: Vec<Id> = db.persons.values().filter(|p| p.clans.iter().any(|c| c == name)).map(|p| p.id).collect();
    for id in &ids {
        let mut p = db.persons[id].clone();
        p.clans.retain(|c| c != name);
        db.put_person(p);
    }
    let mut s = db.settings.clone();
    s.clans.retain(|c| c != name);
    db.put_settings(s);
    ids.len()
}

/// Adds / removes a clan for many people at once.
pub fn assign_clan(db: &mut Database, name: &str, persons: &[Id], add: bool) -> usize {
    let name = clean(name);
    let mut n = 0;
    for id in persons {
        let Some(p) = db.persons.get(id) else { continue };
        let has = p.clans.iter().any(|c| *c == name);
        if has == add {
            continue;
        }
        let mut p = p.clone();
        if add {
            p.clans.push(name.clone());
        } else {
            p.clans.retain(|c| *c != name);
        }
        db.put_person(p);
        n += 1;
    }
    n
}

/* ------------------------------------------------------------------ */
/*  Places                                                              */
/* ------------------------------------------------------------------ */

pub fn set_place_coords(db: &mut Database, ctx: &Ctx, name: &str, lat: Option<f64>, lng: Option<f64>, origin: &str) -> Id {
    let name = clean(name);
    let existing = db.place_by_name(&name).cloned();
    let mut p = existing.unwrap_or_default();
    if p.id == 0 {
        p.id = db.next_id();
        p.name = name;
    }
    let (old_lat, old_lng) = (p.lat, p.lng);
    p.lat = lat.filter(|v| v.is_finite() && (-90.0..=90.0).contains(v));
    p.lng = lng.filter(|v| v.is_finite() && (-180.0..=180.0).contains(v));
    // the weather belonged to the old position
    if p.lat != old_lat || p.lng != old_lng {
        p.weather.clear();
    }
    p.origin = if p.lat.is_some() && p.lng.is_some() { origin.to_string() } else { "failed".into() };
    p.updated = ctx.now;
    let id = p.id;
    db.put_place(p);
    id
}

/// Stores the weather of one day at a place that has coordinates.
pub fn set_place_weather(db: &mut Database, name: &str, day: &str, w: DayWeather) -> Result<()> {
    let d = date::parse_iso_day(day).ok_or_else(|| Error::invalid("date must be YYYY-MM-DD"))?;
    let mut p = db.place_by_name(name).cloned().ok_or_else(|| Error::not_found("place", 0))?;
    if p.lat.is_none() || p.lng.is_none() {
        return Err(Error::invalid("the place has no coordinates"));
    }
    let num = |v: Option<f32>, lo: f32, hi: f32| v.filter(|x| x.is_finite() && (lo..=hi).contains(x));
    let w = DayWeather {
        code: w.code.clamp(0, 99),
        tmax: num(w.tmax, -90.0, 60.0),
        tmin: num(w.tmin, -90.0, 60.0),
        rain: num(w.rain, 0.0, 2000.0),
        snow: num(w.snow, 0.0, 1000.0),
    };
    if p.weather.len() >= 5000 && !p.weather.contains_key(day) {
        return Err(Error::invalid("too many days"));
    }
    p.weather.insert(d.format("%Y-%m-%d").to_string(), w);
    db.put_place(p);
    Ok(())
}

/// Renames a place everywhere it is used (people, events, unions).
pub fn rename_place(db: &mut Database, from: &str, to: &str) -> usize {
    let to = clean(to);
    let mut n = 0;
    let ids: Vec<Id> = db.persons.keys().copied().collect();
    for id in ids {
        let p = &db.persons[&id];
        if p.birth_place == from || p.death_place == from || p.events.iter().any(|e| e.place == from) {
            let mut p = p.clone();
            if p.birth_place == from {
                p.birth_place = to.clone();
            }
            if p.death_place == from {
                p.death_place = to.clone();
            }
            for e in &mut p.events {
                if e.place == from {
                    e.place = to.clone();
                }
            }
            db.put_person(p);
            n += 1;
        }
    }
    let uids: Vec<Id> = db.unions.values().filter(|u| u.place == from).map(|u| u.id).collect();
    for uid in uids {
        let mut u = db.unions[&uid].clone();
        u.place = to.clone();
        db.put_union(u);
        n += 1;
    }
    if let Some(pl) = db.place_by_name(from).cloned() {
        if db.place_by_name(&to).is_none() {
            let mut pl = pl;
            pl.name = to;
            db.put_place(pl);
        }
    }
    n
}

/* ------------------------------------------------------------------ */
/*  Settings                                                            */
/* ------------------------------------------------------------------ */

pub fn save_settings(db: &mut Database, mut s: Settings) {
    s.suggestions.email = clean(&s.suggestions.email);
    let ep = clean(&s.suggestions.endpoint);
    s.suggestions.endpoint = if ep.to_ascii_lowercase().starts_with("https://") { ep } else { String::new() };
    s.clans = dedup_strings(&s.clans);
    s.clans.sort_by_key(|c| crate::model::norm(c));
    s.history.off = dedup_strings(&s.history.off).into_iter().filter(|x| x.len() <= 40).take(500).collect();
    let mut own: Vec<HistoryEvent> = s
        .history
        .own
        .iter()
        .map(|e| HistoryEvent { id: clean(&e.id).chars().take(40).collect(), date: date::sanitize(&e.date), title: clean(&e.title).chars().take(300).collect() })
        .filter(|e| !e.title.is_empty() && !e.date.is_empty())
        .take(1000)
        .collect();
    for (i, e) in own.iter_mut().enumerate() {
        if e.id.is_empty() {
            e.id = format!("own-{i}");
        }
    }
    own.sort_by(|a, b| date::sort_key(&a.date).cmp(&date::sort_key(&b.date)));
    s.history.own = own;
    s.tree_start_person = s.tree_start_person.filter(|p| db.persons.contains_key(p));
    s.privacy.sections.retain(|x| crate::privacy::SECTIONS.contains(&x.as_str()));
    s.privacy.fields.retain(|x| crate::privacy::FIELDS.contains(&x.as_str()));
    if !crate::privacy::LEVELS.contains(&s.privacy.level.as_str()) {
        s.privacy.level = "full".into();
    }
    s.people_per_page = s.people_per_page.clamp(5, 500);
    s.notifications.days_before = s.notifications.days_before.clamp(1, 90);
    let mut groups = Vec::new();
    let allowed = ["text", "textarea", "number", "date", "url", "email", "select"];
    for g in &s.custom_schema {
        let gid = key(&g.id);
        if gid.is_empty() {
            continue;
        }
        let fields = g
            .fields
            .iter()
            .filter_map(|f| {
                let fid = key(&f.id);
                if fid.is_empty() {
                    return None;
                }
                Some(CustomField {
                    id: fid,
                    name: clean(&f.name),
                    kind: if allowed.contains(&f.kind.as_str()) { f.kind.clone() } else { "text".into() },
                    options: f.options.iter().map(|o| clean(o)).filter(|o| !o.is_empty()).collect(),
                })
            })
            .collect();
        groups.push(CustomGroup { id: gid, name: clean(&g.name), fields });
    }
    s.custom_schema = groups;
    db.put_settings(s);
}

#[cfg(test)]
mod tests {
    #[test]
    fn feminine() {
        assert_eq!(super::feminine_surname("Nowakowski"), "Nowakowska");
        assert_eq!(super::feminine_surname("Kowalski-Zawadzki"), "Kowalska-Zawadzka");
        assert_eq!(super::feminine_surname("Nowak"), "Nowak");
        assert_eq!(super::feminine_surname("Novotný"), "Novotná");
    }

    use super::*;
    use chrono::NaiveDate;

    pub fn ctx() -> Ctx {
        Ctx::new(1_700_000_000_000, NaiveDate::from_ymd_opt(2026, 10, 14).unwrap())
    }

    fn person(first: &str, last: &str, g: Gender) -> Person {
        Person { first_name: first.into(), last_name: last.into(), gender: g, ..Default::default() }
    }

    #[test]
    fn relatives_and_cycles() {
        let c = ctx();
        let mut db = Database::new(c.now);
        let jan = create_person(&mut db, &c, person("Jan", "Kowalski", Gender::Male), RelationEdits::default()).unwrap();
        let anna = add_relative(&mut db, &c, jan, NewRelation::Spouse, person("Anna", "Nowak", Gender::Unset), None).unwrap();
        assert_eq!(db.persons[&anna].gender, Gender::Female);
        let kid = add_relative(&mut db, &c, jan, NewRelation::Child, person("Ewa", "Kowalska", Gender::Female), None).unwrap();
        assert_eq!(db.persons[&kid].father, Some(jan));
        assert_eq!(db.persons[&kid].mother, Some(anna));
        // Kowalska is the feminine form of Kowalski: no maiden name needed.
        assert_eq!(db.persons[&kid].maiden_name, "");
        let married = add_relative(&mut db, &c, jan, NewRelation::Child, person("Ola", "Wiśniewska", Gender::Female), None).unwrap();
        assert_eq!(db.persons[&married].maiden_name, "Kowalska");
        // cycle: jan as child of kid
        assert_eq!(set_parent(&mut db, &c, jan, ParentRole::Father, Some(kid), ParentRel::Bio).unwrap_err().code, "cycle");
        // revisions recorded on change
        let mut p = db.persons[&kid].clone();
        p.birth_date = "2000-01-01".into();
        update_person(&mut db, &c, p, RelationEdits::default()).unwrap();
        assert_eq!(db.revisions.values().filter(|r| r.person == kid).count(), 1);
        // delete removes references
        delete_person(&mut db, anna).unwrap();
        assert_eq!(db.persons[&kid].mother, None);
        assert!(db.unions.is_empty());
    }

    #[test]
    fn html_cleaning() {
        assert_eq!(clean_html("<p onclick=\"x()\">a</p><script>alert(1)</script>"), "<p>a</p>");
        assert_eq!(clean_html("<a href=\"JavaScript:alert(1)\">x</a>"), "<a href=\"alert(1)\">x</a>");
    }
}
