//! In-memory database with change tracking.
//!
//! The whole tree lives in memory (genealogies are small: even 50 000 people
//! fit comfortably). Every mutation is recorded in a `ChangeSet` that the
//! platform layer persists (SQLite natively, IndexedDB in the browser).

use crate::model::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Person,
    Union,
    Pet,
    Source,
    Media,
    Place,
    Revision,
    Suggestion,
    Settings,
    Meta,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Person => "person",
            Kind::Union => "union",
            Kind::Pet => "pet",
            Kind::Source => "source",
            Kind::Media => "media",
            Kind::Place => "place",
            Kind::Revision => "revision",
            Kind::Suggestion => "suggestion",
            Kind::Settings => "settings",
            Kind::Meta => "meta",
        }
    }
    pub fn parse(s: &str) -> Option<Kind> {
        Some(match s {
            "person" => Kind::Person,
            "union" => Kind::Union,
            "pet" => Kind::Pet,
            "source" => Kind::Source,
            "media" => Kind::Media,
            "place" => Kind::Place,
            "revision" => Kind::Revision,
            "suggestion" => Kind::Suggestion,
            "settings" => Kind::Settings,
            "meta" => Kind::Meta,
            _ => return None,
        })
    }
}

/// One stored record (JSON payload).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Record {
    pub kind: Kind,
    pub id: Id,
    /// `None` = deleted.
    pub data: Option<Value>,
}

/// Pending changes since the last `take_changes()`.
#[derive(Default, Debug, Clone)]
pub struct ChangeSet {
    map: BTreeMap<(Kind, Id), Option<Value>>,
    /// Media blobs to delete (the platform owns the bytes).
    pub deleted_media: Vec<Id>,
}

impl ChangeSet {
    pub fn is_empty(&self) -> bool {
        self.map.is_empty() && self.deleted_media.is_empty()
    }
    pub fn records(self) -> (Vec<Record>, Vec<Id>) {
        (
            self.map.into_iter().map(|((kind, id), data)| Record { kind, id, data }).collect(),
            self.deleted_media,
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct Meta {
    pub next_id: Id,
    pub created: i64,
    /// Data version, bumped on every change (cache keys in the UI).
    pub version: u64,
    /// Small auxiliary state (e.g. pending WordPress media downloads).
    pub extra: BTreeMap<String, Value>,
}

#[derive(Default, Debug, Clone)]
pub struct Database {
    pub persons: BTreeMap<Id, Person>,
    pub unions: BTreeMap<Id, Union>,
    pub pets: BTreeMap<Id, Pet>,
    pub sources: BTreeMap<Id, Source>,
    pub media: BTreeMap<Id, Media>,
    pub places: BTreeMap<Id, Place>,
    pub revisions: BTreeMap<Id, Revision>,
    pub suggestions: BTreeMap<Id, Suggestion>,
    pub settings: Settings,
    pub meta: Meta,
    changes: ChangeSet,
}

macro_rules! store_fns {
    ($put:ident, $del:ident, $field:ident, $ty:ty, $kind:expr) => {
        pub fn $put(&mut self, v: $ty) {
            let id = v.id;
            self.track($kind, id, Some(serde_json::to_value(&v).expect("serializable")));
            self.$field.insert(id, v);
        }
        pub fn $del(&mut self, id: Id) -> Option<$ty> {
            let r = self.$field.remove(&id);
            if r.is_some() {
                self.track($kind, id, None);
            }
            r
        }
    };
}

impl Database {
    pub fn new(now: i64) -> Database {
        let mut db = Database::default();
        db.meta.next_id = 1;
        db.meta.created = now;
        db.track(Kind::Meta, 1, Some(serde_json::to_value(&db.meta).unwrap()));
        db.track(Kind::Settings, 1, Some(serde_json::to_value(&db.settings).unwrap()));
        db
    }

    /// Builds the database from stored records (no changes are tracked).
    pub fn from_records(records: impl IntoIterator<Item = Record>) -> Result<Database, String> {
        let mut db = Database::default();
        db.meta.next_id = 1;
        for r in records {
            let Some(data) = r.data else { continue };
            macro_rules! load {
                ($field:ident) => {{
                    let v = serde_json::from_value(data).map_err(|e| format!("{} #{}: {e}", r.kind.as_str(), r.id))?;
                    db.$field.insert(r.id, v);
                }};
            }
            match r.kind {
                Kind::Person => load!(persons),
                Kind::Union => load!(unions),
                Kind::Pet => load!(pets),
                Kind::Source => load!(sources),
                Kind::Media => load!(media),
                Kind::Place => load!(places),
                Kind::Revision => load!(revisions),
                Kind::Suggestion => load!(suggestions),
                Kind::Settings => db.settings = serde_json::from_value(data).unwrap_or_default(),
                Kind::Meta => db.meta = serde_json::from_value(data).unwrap_or_default(),
            }
        }
        // Data from archives and backups: never trust links and media types.
        for m in db.media.values_mut() {
            m.mime = crate::model::safe_mime(&m.mime);
        }
        for s in db.sources.values_mut() {
            s.url = crate::model::safe_url(&s.url);
            s.media_url = crate::model::safe_url(&s.media_url);
        }
        // Make sure the id counter is above every stored id.
        let max = [
            db.persons.keys().last(),
            db.unions.keys().last(),
            db.pets.keys().last(),
            db.sources.keys().last(),
            db.media.keys().last(),
            db.places.keys().last(),
            db.revisions.keys().last(),
            db.suggestions.keys().last(),
        ]
        .into_iter()
        .flatten()
        .copied()
        .max()
        .unwrap_or(0);
        if db.meta.next_id <= max {
            db.meta.next_id = max + 1;
        }
        Ok(db)
    }

    /// All records (full dump), e.g. for an archive.
    pub fn all_records(&self) -> Vec<Record> {
        fn push<T: Serialize>(out: &mut Vec<Record>, kind: Kind, m: &BTreeMap<Id, T>) {
            for (id, v) in m {
                out.push(Record { kind, id: *id, data: Some(serde_json::to_value(v).unwrap()) });
            }
        }
        let mut out = Vec::new();
        out.push(Record { kind: Kind::Meta, id: 1, data: Some(serde_json::to_value(&self.meta).unwrap()) });
        out.push(Record { kind: Kind::Settings, id: 1, data: Some(serde_json::to_value(&self.settings).unwrap()) });
        push(&mut out, Kind::Person, &self.persons);
        push(&mut out, Kind::Union, &self.unions);
        push(&mut out, Kind::Pet, &self.pets);
        push(&mut out, Kind::Source, &self.sources);
        push(&mut out, Kind::Media, &self.media);
        push(&mut out, Kind::Place, &self.places);
        push(&mut out, Kind::Revision, &self.revisions);
        push(&mut out, Kind::Suggestion, &self.suggestions);
        out
    }

    fn track(&mut self, kind: Kind, id: Id, data: Option<Value>) {
        self.changes.map.insert((kind, id), data);
        if kind != Kind::Meta {
            self.meta.version += 1;
            let meta = serde_json::to_value(&self.meta).unwrap();
            self.changes.map.insert((Kind::Meta, 1), Some(meta));
        }
    }

    pub fn take_changes(&mut self) -> ChangeSet {
        std::mem::take(&mut self.changes)
    }

    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// Marks every record as changed (after a bulk replace).
    pub fn mark_all_dirty(&mut self) {
        for r in self.all_records() {
            self.changes.map.insert((r.kind, r.id), r.data);
        }
    }

    pub fn next_id(&mut self) -> Id {
        let id = self.meta.next_id.max(1);
        self.meta.next_id = id + 1;
        let meta = serde_json::to_value(&self.meta).unwrap();
        self.changes.map.insert((Kind::Meta, 1), Some(meta));
        id
    }

    store_fns!(put_person, del_person_raw, persons, Person, Kind::Person);
    store_fns!(put_union, del_union, unions, Union, Kind::Union);
    store_fns!(put_pet, del_pet, pets, Pet, Kind::Pet);
    store_fns!(put_source, del_source, sources, Source, Kind::Source);
    store_fns!(put_media_meta, del_media_meta, media, Media, Kind::Media);
    store_fns!(put_place, del_place, places, Place, Kind::Place);
    store_fns!(put_revision, del_revision, revisions, Revision, Kind::Revision);
    store_fns!(put_suggestion, del_suggestion, suggestions, Suggestion, Kind::Suggestion);

    pub fn put_settings(&mut self, s: Settings) {
        self.settings = s;
        self.track(Kind::Settings, 1, Some(serde_json::to_value(&self.settings).unwrap()));
    }

    pub fn delete_media(&mut self, id: Id) {
        if self.del_media_meta(id).is_some() {
            self.changes.deleted_media.push(id);
        }
    }

    pub fn meta_extra(&self, key: &str) -> Option<Value> {
        self.meta.extra.get(key).cloned()
    }

    pub fn set_meta_extra(&mut self, key: &str, v: Option<Value>) {
        match v {
            Some(v) => {
                self.meta.extra.insert(key.to_string(), v);
            }
            None => {
                self.meta.extra.remove(key);
            }
        }
        let meta = serde_json::to_value(&self.meta).unwrap();
        self.changes.map.insert((Kind::Meta, 1), Some(meta));
    }

    pub fn person(&self, id: Id) -> Option<&Person> {
        self.persons.get(&id)
    }

    pub fn name_of(&self, id: Id) -> String {
        self.persons.get(&id).map(|p| p.full_name()).unwrap_or_else(|| format!("#{id}"))
    }

    pub fn place_by_name(&self, name: &str) -> Option<&Place> {
        let key = name.trim().to_lowercase();
        self.places.values().find(|p| p.name.trim().to_lowercase() == key)
    }

    pub fn index(&self) -> Index {
        Index::build(self)
    }
}

/* ------------------------------------------------------------------ */
/*  Derived index                                                       */
/* ------------------------------------------------------------------ */

/// Relations derived from the canonical data (children lists, unions per person …).
#[derive(Default, Debug)]
pub struct Index {
    pub children: HashMap<Id, Vec<Id>>,
    pub unions_of: HashMap<Id, Vec<Id>>,
    pub pets_of: HashMap<Id, Vec<Id>>,
    pub sources_of: HashMap<Id, Vec<Id>>,
}

impl Index {
    pub fn build(db: &Database) -> Index {
        let mut ix = Index::default();
        for p in db.persons.values() {
            for parent in p.parents() {
                if db.persons.contains_key(&parent) {
                    ix.children.entry(parent).or_default().push(p.id);
                }
            }
        }
        // Children sorted by birth date, then id.
        for list in ix.children.values_mut() {
            list.sort_by(|a, b| {
                let da = crate::date::sort_key(&db.persons[a].birth_date);
                let dbb = crate::date::sort_key(&db.persons[b].birth_date);
                da.cmp(&dbb).then(a.cmp(b))
            });
        }
        let mut unions: Vec<&Union> = db.unions.values().collect();
        unions.sort_by(|a, b| crate::date::sort_key(&a.date).cmp(&crate::date::sort_key(&b.date)).then(a.id.cmp(&b.id)));
        for u in unions {
            ix.unions_of.entry(u.a).or_default().push(u.id);
            if let Some(b) = u.b {
                if b != u.a {
                    ix.unions_of.entry(b).or_default().push(u.id);
                }
            }
        }
        for pet in db.pets.values() {
            if let Some(o) = pet.owner {
                ix.pets_of.entry(o).or_default().push(pet.id);
            }
        }
        for s in db.sources.values() {
            for pid in &s.persons {
                ix.sources_of.entry(*pid).or_default().push(s.id);
            }
        }
        ix
    }

    pub fn children_of(&self, id: Id) -> &[Id] {
        self.children.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn unions_of(&self, id: Id) -> &[Id] {
        self.unions_of.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Partners (people in the tree) of `id`, in union order, deduplicated.
    pub fn partners(&self, db: &Database, id: Id) -> Vec<Id> {
        let mut out = Vec::new();
        for uid in self.unions_of(id) {
            if let Some(p) = db.unions.get(uid).and_then(|u| u.partner_of(id)) {
                if p != id && !out.contains(&p) {
                    out.push(p);
                }
            }
        }
        out
    }

    /// Siblings (at least one common parent).
    pub fn siblings(&self, db: &Database, id: Id) -> Vec<Id> {
        let Some(p) = db.persons.get(&id) else { return vec![] };
        let mut out = Vec::new();
        for parent in p.parents() {
            for c in self.children_of(parent) {
                if *c != id && !out.contains(c) {
                    out.push(*c);
                }
            }
        }
        out.sort_by(|a, b| crate::date::sort_key(&db.persons[a].birth_date).cmp(&crate::date::sort_key(&db.persons[b].birth_date)));
        out
    }

    pub fn pets_of(&self, id: Id) -> &[Id] {
        self.pets_of.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn sources_of(&self, id: Id) -> &[Id] {
        self.sources_of.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }
}
