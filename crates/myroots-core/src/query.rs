//! Read-only views assembled for the UI.

use crate::date;
use crate::db::{Database, Index};
use crate::life::{self, Ctx};
use crate::model::*;
use crate::privacy::Policy;
use chrono::{Datelike, Duration, NaiveDate};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

/* ------------------------------------------------------------------ */
/*  Summaries                                                           */
/* ------------------------------------------------------------------ */

/// Compact person record used by lists, pickers and searches.
#[derive(Serialize, Clone, Debug)]
pub struct Summary {
    pub id: Id,
    pub name: String,
    pub first_name: String,
    pub last_name: String,
    pub maiden_name: String,
    pub gender: Gender,
    pub birth_date: String,
    pub birth_place: String,
    pub death_date: String,
    pub death_place: String,
    pub years: String,
    pub clans: Vec<String>,
    pub photo: Option<Id>,
    pub living: bool,
    pub status: Status,
    pub occupation: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub death_cause: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub residence: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub nickname: String,
    pub updated: i64,
    /// Anonymous placeholder (privacy) — no data, no profile.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub anon: bool,
    /// Some fields were censored (privacy).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub censored: bool,
}

pub fn summary(p: &Person, max_age: u32, today: NaiveDate) -> Summary {
    Summary {
        id: p.id,
        name: p.full_name(),
        first_name: p.first_name.clone(),
        last_name: p.last_name.clone(),
        maiden_name: p.maiden_name.clone(),
        gender: p.gender,
        birth_date: p.birth_date.clone(),
        birth_place: p.birth_place.clone(),
        death_date: p.death_date.clone(),
        death_place: p.death_place.clone(),
        years: life::years(&p),
        clans: p.clans.clone(),
        photo: p.photo,
        living: life::is_alive(p, max_age, today),
        status: p.status,
        occupation: p.occupation.clone(),
        death_cause: p.death_cause.clone(),
        residence: p.residence.clone(),
        nickname: p.nickname.clone(),
        updated: p.updated,
        anon: false,
        censored: false,
    }
}

pub fn summaries(db: &Database, ctx: &Ctx) -> Vec<Summary> {
    let mut v: Vec<(String, Summary)> = db.persons.values().map(|p| summary(p, db.settings.max_age, ctx.today)).map(|s| (crate::model::norm(&s.name), s)).collect();
    v.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.id.cmp(&b.1.id)));
    v.into_iter().map(|(_, s)| s).collect()
}

/* ------------------------------------------------------------------ */
/*  Overview (dashboard)                                                */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug)]
pub struct Overview {
    pub tree_name: String,
    pub persons: usize,
    pub unions: usize,
    pub pets: usize,
    pub sources: usize,
    pub media: usize,
    pub places: usize,
    pub clans: usize,
    pub living: usize,
    pub new_suggestions: usize,
    pub version: u64,
    pub recent: Vec<Summary>,
    pub empty: bool,
}

pub fn overview(db: &Database, ctx: &Ctx) -> Overview {
    let max = db.settings.max_age;
    let mut recent: Vec<&Person> = db.persons.values().collect();
    let k = recent.len().min(8);
    if k > 0 && k < recent.len() {
        recent.select_nth_unstable_by(k - 1, |a, b| b.updated.cmp(&a.updated).then(b.id.cmp(&a.id)));
        recent.truncate(k);
    }
    recent.sort_by(|a, b| b.updated.cmp(&a.updated).then(b.id.cmp(&a.id)));
    Overview {
        tree_name: db.settings.tree_name.clone(),
        persons: db.persons.len(),
        unions: db.unions.len(),
        pets: db.pets.len(),
        sources: db.sources.len(),
        media: db.media.len(),
        places: db.places.values().filter(|p| p.lat.is_some()).count(),
        clans: all_clans(db).len(),
        living: db.persons.values().filter(|p| life::is_alive(p, max, ctx.today)).count(),
        new_suggestions: db.suggestions.values().filter(|s| s.status == "new" || s.status.is_empty()).count(),
        version: db.meta.version,
        recent: recent.into_iter().take(8).map(|p| summary(p, max, ctx.today)).collect(),
        empty: db.persons.is_empty(),
    }
}

/* ------------------------------------------------------------------ */
/*  Clans                                                               */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug, Clone)]
pub struct ClanInfo {
    pub name: String,
    pub count: usize,
    pub defined: bool,
}

pub fn all_clans(db: &Database) -> Vec<String> {
    clans(db).into_iter().map(|c| c.name).collect()
}

pub fn clans(db: &Database) -> Vec<ClanInfo> {
    let mut map: BTreeMap<String, ClanInfo> = BTreeMap::new();
    for c in &db.settings.clans {
        map.entry(c.clone()).or_insert(ClanInfo { name: c.clone(), count: 0, defined: true });
    }
    for p in db.persons.values() {
        for c in &p.clans {
            map.entry(c.clone()).or_insert(ClanInfo { name: c.clone(), count: 0, defined: false }).count += 1;
        }
    }
    let mut v: Vec<ClanInfo> = map.into_values().collect();
    v.sort_by_key(|c| crate::model::norm(&c.name));
    v
}

/* ------------------------------------------------------------------ */
/*  Person profile / editor                                             */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug, Clone)]
pub struct UnionView {
    pub union: Union,
    /// The partner from the viewpoint of the profile owner.
    pub partner: Option<Summary>,
    pub partner_name: String,
    /// Children of this couple.
    pub children: Vec<Id>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SourceRef {
    pub id: Id,
    pub title: String,
    pub date: String,
    pub url: String,
    pub notes: String,
    pub media_url: String,
    pub photo: Option<Id>,
    pub scans: Vec<Id>,
    /// Fields this source is cited for (empty for a general link).
    pub fields: Vec<String>,
    pub linked: bool,
    /// Every citation of this source in the profile, with its details.
    pub cites: Vec<CiteRef>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub repository: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub call_number: String,
}

/// One citation shown in a profile: what it confirms and where in the source.
#[derive(Serialize, Debug, Clone)]
pub struct CiteRef {
    /// A field of `CITABLE_FIELDS`, "event" or "marriage".
    pub fact: String,
    /// Name of the event or the partner (events and marriages).
    pub label: String,
    /// Index of the event in `person.events` / id of the union.
    pub index: Option<usize>,
    pub union: Option<Id>,
    pub page: String,
    pub quality: Option<u8>,
    pub note: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct PetRef {
    pub id: Id,
    pub name: String,
    pub species: String,
    pub breed: String,
    pub photo: Option<Id>,
    pub years: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Profile {
    pub person: Person,
    pub summary: Summary,
    pub age: Option<i32>,
    pub living: bool,
    pub presumed_dead: bool,
    pub father: Option<Summary>,
    pub mother: Option<Summary>,
    pub unions: Vec<UnionView>,
    pub children: Vec<Summary>,
    pub siblings: Vec<Summary>,
    pub half_siblings: Vec<Id>,
    pub pets: Vec<PetRef>,
    pub sources: Vec<SourceRef>,
    pub grandparents: Vec<Summary>,
    pub grandchildren: Vec<Summary>,
    pub revisions: usize,
    pub censored: bool,
    /// Photos of others where this person is marked (group photos).
    pub tagged: Vec<GalleryItem>,
    /// Godparents, witnesses… of this person, and (reverse) the people this
    /// person is a godparent, witness… of.
    pub associations: Vec<AssocView>,
}

#[derive(Serialize, Debug, Clone)]
pub struct AssocView {
    pub person: Summary,
    pub role: String,
    pub event: String,
    /// true: this person is the `role` of `person` (e.g. godparent of).
    pub reverse: bool,
}

pub fn profile(db: &Database, ctx: &Ctx, id: Id, pol: &Policy) -> Option<Profile> {
    let raw = db.persons.get(&id)?;
    let ix = db.index();
    let max = db.settings.max_age;
    let sum = |pid: Id| -> Option<Summary> { public_summary(db, pol, pid) };
    let (person, censored) = pol.redact(raw);

    let mut unions = Vec::new();
    for uid in ix.unions_of(id) {
        let u = &db.unions[uid];
        let partner_id = u.partner_of(id);
        let a_l = db.persons.get(&u.a).map(|p| pol.is_living(p)).unwrap_or(false);
        let b_l = u.b.and_then(|b| db.persons.get(&b)).map(|p| pol.is_living(p)).unwrap_or(false);
        let u2 = pol.redact_union(u, a_l, b_l);
        let partner = partner_id.and_then(sum);
        let partner_name = match &partner {
            Some(s) => s.name.clone(),
            None => format!("{} {}", u.partner_first_name, u.partner_last_name).trim().to_string(),
        };
        let children: Vec<Id> = ix
            .children_of(id)
            .iter()
            .copied()
            .filter(|c| {
                let cp = &db.persons[c];
                match partner_id {
                    Some(pid) => cp.parents().any(|x| x == pid),
                    None => false,
                }
            })
            .collect();
        unions.push(UnionView { union: u2, partner, partner_name, children });
    }

    let children: Vec<Summary> = ix.children_of(id).iter().filter_map(|c| sum(*c)).collect();
    let siblings_ids = ix.siblings(db, id);
    let half: Vec<Id> = siblings_ids
        .iter()
        .copied()
        .filter(|s| {
            let sp = &db.persons[s];
            !(sp.father == raw.father && sp.mother == raw.mother && raw.father.is_some() && raw.mother.is_some())
        })
        .collect();
    let siblings: Vec<Summary> = siblings_ids.iter().filter_map(|s| sum(*s)).collect();

    let mut grandparents = Vec::new();
    for p in raw.parents() {
        if let Some(pp) = db.persons.get(&p) {
            for g in pp.parents() {
                if let Some(s) = sum(g) {
                    grandparents.push(s);
                }
            }
        }
    }
    let mut grandchildren = Vec::new();
    for c in ix.children_of(id) {
        for g in ix.children_of(*c) {
            if let Some(s) = sum(*g) {
                grandchildren.push(s);
            }
        }
    }

    let pets = ix
        .pets_of(id)
        .iter()
        .filter_map(|pid| db.pets.get(pid))
        .filter(|p| !pol.active || p.status == Status::Published)
        .map(|p| PetRef {
            id: p.id,
            name: p.name.clone(),
            species: p.species.clone(),
            breed: p.breed.clone(),
            photo: p.photo,
            years: life::years_label(&p.birth_date, &p.death_date),
        })
        .collect();

    // Sources: general links + citations.
    let mut src: BTreeMap<Id, SourceRef> = BTreeMap::new();
    let mk = |s: &Source| SourceRef {
        id: s.id,
        title: s.title.clone(),
        date: s.date.clone(),
        url: s.url.clone(),
        notes: s.notes.clone(),
        media_url: s.media_url.clone(),
        photo: s.photo,
        scans: s.scans.clone(),
        fields: vec![],
        linked: false,
        cites: vec![],
        repository: s.repository.clone(),
        call_number: s.call_number.clone(),
    };
    for sid in ix.sources_of(id) {
        if let Some(s) = db.sources.get(sid) {
            src.entry(*sid).or_insert_with(|| mk(s)).linked = true;
        }
    }
    for (field, ids) in &person.citations {
        for sid in ids {
            if let Some(s) = db.sources.get(sid) {
                let e = src.entry(*sid).or_insert_with(|| mk(s));
                e.fields.push(field.clone());
                let info = person.citation_info.get(&citation_key(field, *sid)).cloned().unwrap_or_default();
                e.cites.push(CiteRef { fact: field.clone(), label: String::new(), index: None, union: None, page: info.page, quality: info.quality, note: info.note });
            }
        }
    }
    for (i, ev) in person.events.iter().enumerate() {
        for c in &ev.citations {
            if let Some(s) = db.sources.get(&c.source) {
                let e = src.entry(c.source).or_insert_with(|| mk(s));
                e.cites.push(CiteRef { fact: "event".into(), label: ev.name.clone(), index: Some(i), union: None, page: c.page.clone(), quality: c.quality, note: c.note.clone() });
            }
        }
    }

    for uv in &unions {
        let u: &UnionView = uv;
        for c in &u.union.citations {
            if let Some(s) = db.sources.get(&c.source) {
                let e = src.entry(c.source).or_insert_with(|| mk(s));
                e.cites.push(CiteRef { fact: "marriage".into(), label: u.partner_name.clone(), index: None, union: Some(u.union.id), page: c.page.clone(), quality: c.quality, note: c.note.clone() });
            }
        }
    }

    let living = life::is_alive(raw, max, ctx.today);
    Some(Profile {
        summary: {
            let mut s = summary(&person, max, ctx.today);
            s.living = living;
            s.censored = censored;
            s
        },
        age: if censored && pol.fields.iter().any(|f| f == "dates") { None } else { life::age(raw, max, ctx.today) },
        living,
        presumed_dead: life::presumed_dead(raw, max, ctx.today),
        father: raw.father.and_then(sum),
        mother: raw.mother.and_then(sum),
        unions,
        children,
        siblings,
        half_siblings: half,
        pets,
        sources: src.into_values().collect(),
        grandparents,
        grandchildren,
        revisions: db.revisions.values().filter(|r| r.person == id).count(),
        censored,
        tagged: tagged_photos(db, pol, raw),
        associations: associations(db, pol, raw),
        person,
    })
}

fn associations(db: &Database, pol: &Policy, p: &Person) -> Vec<AssocView> {
    let mut out: Vec<AssocView> = p
        .associations
        .iter()
        .filter_map(|a| Some(AssocView { person: public_summary(db, pol, a.person).filter(|s| !s.anon)?, role: a.role.clone(), event: a.event.clone(), reverse: false }))
        .collect();
    for o in db.persons.values() {
        for a in o.associations.iter().filter(|a| a.person == p.id) {
            if let Some(s) = public_summary(db, pol, o.id).filter(|s| !s.anon) {
                out.push(AssocView { person: s, role: a.role.clone(), event: a.event.clone(), reverse: true });
            }
        }
    }
    out
}

/// Summary respecting the policy (anonymous placeholder when hidden).
/// Photos (not the person's own) on which the person is marked.
fn tagged_photos(db: &Database, pol: &Policy, p: &Person) -> Vec<GalleryItem> {
    if pol.active && pol.photo_hidden(p) {
        return vec![];
    }
    let own: std::collections::HashSet<Id> = p.photo.into_iter().chain(p.gallery.iter().map(|g| g.media)).collect();
    let mut v: Vec<&Media> = db.media.values().filter(|m| !own.contains(&m.id) && m.tags.iter().any(|t| t.person == Some(p.id))).collect();
    v.sort_by_key(|m| (m.created, m.id));
    v.into_iter().map(|m| GalleryItem { media: m.id, caption: m.caption.clone() }).collect()
}

#[derive(Serialize, Debug, Clone)]
pub struct TagView {
    pub person: Option<Id>,
    pub name: String,
    pub rect: [f32; 4],
    /// Name to show (the person's current name, or the name written on the mark).
    pub label: String,
    /// Dates of the person (to estimate when the photo was taken).
    pub birth_date: String,
    pub death_date: String,
    /// The face descriptor is known.
    pub has_face: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct MediaView {
    pub media: Media,
    pub tags: Vec<TagView>,
}

/// A photo with the people marked on it (people the reader may not see are left out).
pub fn media_view(db: &Database, pol: &Policy, id: Id) -> Option<MediaView> {
    let m = db.media.get(&id)?;
    let mut tags = Vec::new();
    for t in &m.tags {
        let (label, birth_date, death_date) = match t.person {
            Some(pid) => {
                let Some(s) = public_summary(db, pol, pid) else { continue };
                if pol.active && db.persons.get(&pid).map(|p| pol.photo_hidden(p)).unwrap_or(true) {
                    continue;
                }
                (s.name, s.birth_date, s.death_date)
            }
            None => (t.name.clone(), String::new(), String::new()),
        };
        tags.push(TagView { person: t.person, name: t.name.clone(), rect: t.rect, label, birth_date, death_date, has_face: !t.face.is_empty() });
    }
    let mut media = m.clone();
    media.tags = tags.iter().map(|t| PhotoTag { person: t.person, name: t.name.clone(), rect: t.rect, face: Vec::new() }).collect();
    Some(MediaView { media, tags })
}

pub fn public_summary(db: &Database, pol: &Policy, id: Id) -> Option<Summary> {
    let p = db.persons.get(&id)?;
    if pol.active && p.status != Status::Published {
        return None;
    }
    if pol.should_hide(p) {
        return Some(anon_summary(p));
    }
    let (r, censored) = pol.redact(p);
    let mut s = summary(&r, pol.max_age, pol.today);
    s.living = pol.is_living(p);
    s.censored = censored;
    if pol.photo_hidden(p) {
        s.photo = None;
    }
    Some(s)
}

pub fn anon_summary(p: &Person) -> Summary {
    Summary {
        id: p.id,
        name: String::new(),
        first_name: String::new(),
        last_name: String::new(),
        maiden_name: String::new(),
        gender: Gender::Unset,
        birth_date: String::new(),
        birth_place: String::new(),
        death_date: String::new(),
        death_place: String::new(),
        years: String::new(),
        clans: vec![],
        photo: None,
        living: true,
        status: Status::Published,
        occupation: String::new(),
        death_cause: String::new(),
        residence: String::new(),
        nickname: String::new(),
        updated: 0,
        anon: true,
        censored: false,
    }
}

/// Data for the person editor.
#[derive(Serialize, Debug)]
pub struct EditModel {
    pub person: Person,
    pub unions: Vec<UnionView>,
    pub children: Vec<Id>,
    pub sources: Vec<Id>,
}

pub fn edit_model(db: &Database, ctx: &Ctx, id: Id) -> Option<EditModel> {
    let pol = Policy::owner(db.settings.max_age, ctx.today);
    let prof = profile(db, ctx, id, &pol)?;
    let ix = db.index();
    Some(EditModel {
        person: db.persons[&id].clone(),
        unions: prof.unions,
        children: ix.children_of(id).to_vec(),
        sources: ix.sources_of(id).to_vec(),
    })
}

/* ------------------------------------------------------------------ */
/*  Pets / sources lists                                                */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug)]
pub struct PetView {
    pub pet: Pet,
    pub owner: Option<Summary>,
    pub years: String,
    pub alive: bool,
}

pub fn pets(db: &Database, ctx: &Ctx) -> Vec<PetView> {
    let mut v: Vec<PetView> = db
        .pets
        .values()
        .map(|p| PetView {
            pet: p.clone(),
            owner: p.owner.and_then(|o| db.persons.get(&o)).map(|o| summary(o, db.settings.max_age, ctx.today)),
            years: life::years_label(&p.birth_date, &p.death_date),
            alive: p.death_date.is_empty(),
        })
        .collect();
    v.sort_by_key(|p| crate::model::norm(&p.pet.name));
    v
}

#[derive(Serialize, Debug)]
pub struct SourceView {
    pub source: Source,
    pub persons: Vec<Summary>,
    /// People citing this source for specific facts: (person, fields).
    pub citations: Vec<(Summary, Vec<String>)>,
}

/// Facts of every person citing each source: source → person → facts
/// (`CITABLE_FIELDS` keys, "event:<name>", "marriage:<partner>").
fn citing(db: &Database) -> HashMap<Id, BTreeMap<Id, Vec<String>>> {
    let mut m: HashMap<Id, BTreeMap<Id, Vec<String>>> = HashMap::new();
    let mut add = |sid: Id, pid: Id, fact: String| {
        let v = m.entry(sid).or_default().entry(pid).or_default();
        if !v.contains(&fact) {
            v.push(fact);
        }
    };
    for p in db.persons.values() {
        for (field, ids) in &p.citations {
            for sid in ids {
                add(*sid, p.id, field.clone());
            }
        }
        for e in &p.events {
            for c in &e.citations {
                add(c.source, p.id, format!("event:{}", e.name));
            }
        }
    }
    for u in db.unions.values() {
        for c in &u.citations {
            for (pid, other) in [(Some(u.a), u.b), (u.b, Some(u.a))] {
                let Some(pid) = pid else { continue };
                let partner = other.and_then(|o| db.persons.get(&o)).map(|o| o.full_name()).unwrap_or_else(|| format!("{} {}", u.partner_first_name, u.partner_last_name).trim().to_string());
                add(c.source, pid, format!("marriage:{partner}"));
            }
        }
    }
    m
}

fn source_view_with(db: &Database, ctx: &Ctx, s: &Source, cited: Option<&BTreeMap<Id, Vec<String>>>) -> SourceView {
    let max = db.settings.max_age;
    let citations = cited
        .into_iter()
        .flatten()
        .filter_map(|(pid, facts)| db.persons.get(pid).map(|p| (summary(p, max, ctx.today), facts.clone())))
        .collect();
    SourceView {
        source: s.clone(),
        persons: s.persons.iter().filter_map(|p| db.persons.get(p)).map(|p| summary(p, max, ctx.today)).collect(),
        citations,
    }
}

pub fn source_view(db: &Database, ctx: &Ctx, s: &Source) -> SourceView {
    let all = citing(db);
    source_view_with(db, ctx, s, all.get(&s.id))
}

pub fn sources(db: &Database, ctx: &Ctx) -> Vec<SourceView> {
    let all = citing(db);
    let mut v: Vec<SourceView> = db.sources.values().map(|s| source_view_with(db, ctx, s, all.get(&s.id))).collect();
    v.sort_by_cached_key(|s| crate::model::norm(&s.source.title));
    v
}

/* ------------------------------------------------------------------ */
/*  Shared gallery                                                      */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug, Clone)]
pub struct GalleryEntry {
    pub media: Id,
    pub caption: String,
    /// "photo" (profile), "gallery", "event", "pet", "source"
    pub kind: String,
    pub owner_id: Id,
    pub owner_name: String,
    /// "person" | "pet" | "source"
    pub owner_type: String,
    pub clans: Vec<String>,
    pub date: String,
}

pub fn gallery(db: &Database, pol: &Policy) -> Vec<GalleryEntry> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for p in db.persons.values() {
        if pol.active && (p.status != Status::Published || pol.should_hide(p) || pol.photo_hidden(p)) {
            continue;
        }
        let name = p.full_name();
        let mut push = |m: Id, caption: &str, kind: &str, date: &str| {
            if seen.insert((m, p.id)) {
                out.push(GalleryEntry {
                    media: m,
                    caption: caption.to_string(),
                    kind: kind.into(),
                    owner_id: p.id,
                    owner_name: name.clone(),
                    owner_type: "person".into(),
                    clans: p.clans.clone(),
                    date: date.into(),
                });
            }
        };
        if let Some(m) = p.photo {
            push(m, "", "photo", "");
        }
        for g in &p.gallery {
            push(g.media, &g.caption, "gallery", "");
        }
        let r = pol.view(p);
        for e in &r.events {
            if let Some(m) = e.image {
                push(m, &e.name, "event", &e.date);
            }
        }
    }
    for p in db.pets.values() {
        if pol.active && p.status != Status::Published {
            continue;
        }
        for m in p.photo.iter().copied().chain(p.gallery.iter().map(|g| g.media)) {
            let caption = p.gallery.iter().find(|g| g.media == m).map(|g| g.caption.clone()).unwrap_or_default();
            out.push(GalleryEntry {
                media: m,
                caption,
                kind: "pet".into(),
                owner_id: p.id,
                owner_name: p.name.clone(),
                owner_type: "pet".into(),
                clans: vec![],
                date: String::new(),
            });
        }
    }
    out.retain(|e| db.media.contains_key(&e.media));
    out
}

/* ------------------------------------------------------------------ */
/*  Events of the whole family (timeline)                               */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug, Clone)]
pub struct FamilyEvent {
    /// birth | death | marriage | event
    pub kind: String,
    /// Event name for custom events.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    pub date: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub place: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Id>,
    pub person: Id,
    pub person_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo: Option<Id>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub clans: Vec<String>,
    /// Partner for marriages / cause of death.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub with_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_id: Option<Id>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub cause: String,
}

pub fn family_events(db: &Database, pol: &Policy) -> Vec<FamilyEvent> {
    let mut out = Vec::new();
    let visible = |p: &Person| !(pol.active && (p.status != Status::Published || pol.should_hide(p)));
    for raw in db.persons.values() {
        if !visible(raw) {
            continue;
        }
        let p = pol.view(raw);
        let photo = if pol.photo_hidden(raw) { None } else { p.photo };
        let base = |kind: &str, date: &str, place: &str| FamilyEvent {
            kind: kind.into(),
            name: String::new(),
            date: date.into(),
            place: place.into(),
            description: String::new(),
            image: None,
            person: p.id,
            person_name: p.full_name(),
            photo,
            clans: p.clans.clone(),
            with_name: String::new(),
            with_id: None,
            cause: String::new(),
        };
        if !p.birth_date.is_empty() {
            out.push(base("birth", &p.birth_date, &p.birth_place));
        }
        if !p.death_date.is_empty() {
            let mut e = base("death", &p.death_date, &p.death_place);
            e.cause = p.death_cause.clone();
            out.push(e);
        }
        for ev in &p.events {
            if ev.name.is_empty() && ev.date.is_empty() {
                continue;
            }
            let mut e = base("event", &ev.date, &ev.place);
            e.name = ev.name.clone();
            e.description = ev.description.clone();
            e.image = ev.image;
            out.push(e);
        }
    }
    for u in db.unions.values() {
        if u.date.is_empty() {
            continue;
        }
        let Some(a) = db.persons.get(&u.a) else { continue };
        if !visible(a) {
            continue;
        }
        let b = u.b.and_then(|b| db.persons.get(&b));
        if let Some(b) = b {
            if !visible(b) {
                continue;
            }
        }
        let u2 = pol.redact_union(u, pol.is_living(a), b.map(|b| pol.is_living(b)).unwrap_or(false));
        let ar = pol.view(a);
        out.push(FamilyEvent {
            kind: "marriage".into(),
            name: String::new(),
            date: u2.date.clone(),
            place: u2.place.clone(),
            description: String::new(),
            image: None,
            person: a.id,
            person_name: ar.full_name(),
            photo: if pol.photo_hidden(a) { None } else { a.photo },
            clans: a.clans.clone(),
            with_name: b.map(|b| b.full_name()).unwrap_or_else(|| format!("{} {}", u.partner_first_name, u.partner_last_name).trim().to_string()),
            with_id: u.b,
            cause: String::new(),
        });
    }
    out.sort_by_cached_key(|e| date::sort_key(&e.date));
    out
}

/* ------------------------------------------------------------------ */
/*  Anniversaries                                                       */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug, Clone)]
pub struct Anniversary {
    /// Next occurrence (YYYY-MM-DD).
    pub date: String,
    pub days_left: i64,
    /// birth | death | marriage
    pub kind: String,
    /// Which anniversary (years).
    pub years: i32,
    /// Birthday of a living person (vs. birth anniversary of the deceased).
    pub living: bool,
    pub person: Id,
    pub name: String,
    pub partner: Option<Id>,
    pub partner_name: String,
    pub photo: Option<Id>,
}

fn next_occurrence(orig: NaiveDate, today: NaiveDate) -> Option<NaiveDate> {
    // 29 February is celebrated on 28 February in common years.
    let on = |y: i32| NaiveDate::from_ymd_opt(y, orig.month(), orig.day()).or_else(|| NaiveDate::from_ymd_opt(y, 2, 28));
    match on(today.year()) {
        Some(d) if d >= today => Some(d),
        _ => on(today.year() + 1),
    }
}

pub fn anniversaries(db: &Database, ctx: &Ctx, pol: &Policy, days: i64, limit: usize) -> Vec<Anniversary> {
    let today = ctx.today;
    let until = today + Duration::days(days.clamp(1, 366));
    let hide_living = pol.active;
    let mut items = Vec::new();
    for p in db.persons.values() {
        if pol.active && p.status != Status::Published {
            continue;
        }
        let living = pol.is_living(p);
        let photo = if pol.photo_hidden(p) || (hide_living && living && pol.level != "censor_living") { None } else { p.photo };
        if !(hide_living && living) {
            if let Some(b) = date::exact_day(&p.birth_date) {
                if let Some(next) = next_occurrence(b, today) {
                    if next <= until {
                        items.push(Anniversary {
                            date: next.format("%Y-%m-%d").to_string(),
                            days_left: (next - today).num_days(),
                            kind: "birth".into(),
                            years: next.year() - b.year(),
                            living,
                            person: p.id,
                            name: p.full_name(),
                            partner: None,
                            partner_name: String::new(),
                            photo,
                        });
                    }
                }
            }
        }
        if let Some(d) = date::exact_day(&p.death_date) {
            if let Some(next) = next_occurrence(d, today) {
                if next <= until {
                    items.push(Anniversary {
                        date: next.format("%Y-%m-%d").to_string(),
                        days_left: (next - today).num_days(),
                        kind: "death".into(),
                        years: next.year() - d.year(),
                        living: false,
                        person: p.id,
                        name: p.full_name(),
                        partner: None,
                        partner_name: String::new(),
                        photo,
                    });
                }
            }
        }
    }
    for u in db.unions.values() {
        if u.has_ended() {
            continue;
        }
        let Some(m) = date::exact_day(&u.date) else { continue };
        let Some(a) = db.persons.get(&u.a) else { continue };
        let b = u.b.and_then(|b| db.persons.get(&b));
        if pol.active {
            if a.status != Status::Published || b.map(|b| b.status != Status::Published).unwrap_or(false) {
                continue;
            }
            if pol.is_living(a) || b.map(|b| pol.is_living(b)).unwrap_or(false) {
                continue;
            }
        }
        // A union of two deceased people is not celebrated as a wedding anniversary
        // unless at least one partner lives (the plugin skipped ended unions only).
        if let Some(next) = next_occurrence(m, today) {
            if next <= until {
                items.push(Anniversary {
                    date: next.format("%Y-%m-%d").to_string(),
                    days_left: (next - today).num_days(),
                    kind: "marriage".into(),
                    years: next.year() - m.year(),
                    living: true,
                    person: a.id,
                    name: a.full_name(),
                    partner: u.b,
                    partner_name: b.map(|b| b.full_name()).unwrap_or_else(|| format!("{} {}", u.partner_first_name, u.partner_last_name).trim().to_string()),
                    photo: a.photo,
                });
            }
        }
    }
    items.sort_by(|a, b| a.date.cmp(&b.date).then(a.name.cmp(&b.name)));
    if limit > 0 {
        items.truncate(limit);
    }
    items
}

/* ------------------------------------------------------------------ */
/*  Places                                                              */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug, Clone)]
pub struct PlaceUse {
    pub name: String,
    pub count: usize,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub origin: String,
    pub place_id: Option<Id>,
    pub weather: BTreeMap<String, DayWeather>,
}

/// All place names used in the tree with their usage count and coordinates.
pub fn places(db: &Database) -> Vec<PlaceUse> {
    let mut counts: HashMap<String, (String, usize)> = HashMap::new();
    let mut add = |s: &str| {
        let t = s.trim();
        if !t.is_empty() {
            counts.entry(t.to_lowercase()).or_insert((t.to_string(), 0)).1 += 1;
        }
    };
    for p in db.persons.values() {
        add(&p.birth_place);
        add(&p.death_place);
        for e in &p.events {
            add(&e.place);
        }
    }
    for u in db.unions.values() {
        add(&u.place);
    }
    let by_name = db.places_by_name();
    let mut out: Vec<PlaceUse> = counts
        .into_values()
        .map(|(name, count)| {
            let pl = by_name.get(&name.trim().to_lowercase()).copied();
            PlaceUse {
                count,
                lat: pl.and_then(|p| p.lat),
                lng: pl.and_then(|p| p.lng),
                origin: pl.map(|p| p.origin.clone()).unwrap_or_default(),
                place_id: pl.map(|p| p.id),
                weather: pl.map(|p| p.weather.clone()).unwrap_or_default(),
                name,
            }
        })
        .collect();
    out.sort_by_key(|p| crate::model::norm(&p.name));
    out
}

/// Place names that have never been geocoded (no record at all).
pub fn missing_places(db: &Database) -> Vec<String> {
    places(db).into_iter().filter(|p| p.place_id.is_none()).map(|p| p.name).collect()
}

/// Unused helper to keep Index in the public API docs.
pub fn index(db: &Database) -> std::sync::Arc<Index> {
    db.index()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_occ() {
        let t = NaiveDate::from_ymd_opt(2026, 10, 14).unwrap();
        assert_eq!(next_occurrence(NaiveDate::from_ymd_opt(1950, 10, 20).unwrap(), t), NaiveDate::from_ymd_opt(2026, 10, 20));
        assert_eq!(next_occurrence(NaiveDate::from_ymd_opt(1950, 1, 2).unwrap(), t), NaiveDate::from_ymd_opt(2027, 1, 2));
        assert_eq!(next_occurrence(NaiveDate::from_ymd_opt(1952, 2, 29).unwrap(), t), NaiveDate::from_ymd_opt(2027, 2, 28));
        let leap = NaiveDate::from_ymd_opt(2027, 12, 1).unwrap();
        assert_eq!(next_occurrence(NaiveDate::from_ymd_opt(1952, 2, 29).unwrap(), leap), NaiveDate::from_ymd_opt(2028, 2, 29));
    }
}

/* ------------------------------------------------------------------ */
/*  Graves                                                              */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Debug, Clone)]
pub struct GraveEntry {
    pub person: Summary,
    pub grave: Grave,
    pub death_date: String,
}

/// Everyone with a recorded grave, by cemetery and name.
pub fn graves(db: &Database, pol: &Policy) -> Vec<GraveEntry> {
    let mut v: Vec<GraveEntry> = db
        .persons
        .values()
        .filter(|p| !p.grave.is_empty())
        .filter_map(|p| {
            let s = public_summary(db, pol, p.id)?;
            let r = pol.view(p);
            if r.grave.is_empty() {
                return None;
            }
            Some(GraveEntry { person: s, grave: r.grave.clone(), death_date: r.death_date.clone() })
        })
        .collect();
    v.sort_by_cached_key(|g| (crate::model::norm(&g.grave.cemetery), crate::model::norm(&g.person.last_name), crate::model::norm(&g.person.first_name)));
    v
}

/// A cemetery used in the tree (suggestions in the grave form).
#[derive(Serialize, Debug, Clone)]
pub struct CemeteryUse {
    pub name: String,
    pub count: usize,
}

/// Cemeteries of the family with the number of graves (owner view).
pub fn cemeteries(db: &Database) -> Vec<CemeteryUse> {
    let mut m: HashMap<String, CemeteryUse> = HashMap::new();
    for p in db.persons.values() {
        let c = p.grave.cemetery.trim();
        if !c.is_empty() {
            m.entry(crate::model::norm(c)).or_insert_with(|| CemeteryUse { name: c.to_string(), count: 0 }).count += 1;
        }
    }
    for p in db.places.values().filter(|p| p.kind == "cemetery") {
        m.entry(crate::model::norm(&p.name)).or_insert_with(|| CemeteryUse { name: p.name.clone(), count: 0 });
    }
    let mut v: Vec<(String, CemeteryUse)> = m.into_iter().collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v.into_iter().map(|(_, c)| c).collect()
}

#[derive(Serialize, Debug, Clone)]
pub struct KnownFace {
    pub person: Id,
    pub media: Id,
    pub face: Vec<f32>,
}

/// Faces of people marked on photos (for suggestions on other photos).
pub fn face_index(db: &Database) -> Vec<KnownFace> {
    let mut v = Vec::new();
    for m in db.media.values() {
        for t in &m.tags {
            if let (Some(p), false) = (t.person, t.face.is_empty()) {
                if db.persons.contains_key(&p) {
                    v.push(KnownFace { person: p, media: m.id, face: t.face.clone() });
                }
            }
        }
    }
    v
}

