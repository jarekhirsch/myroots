//! Genealogical map: events grouped by place.

use crate::db::Database;
use crate::model::*;
use crate::privacy::Policy;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize, Debug, Clone)]
pub struct MapEvent {
    pub person: Id,
    pub name: String,
    pub photo: Option<Id>,
    pub clans: Vec<String>,
    /// birth | death | marriage | event
    pub kind: String,
    pub label: String,
    pub date: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct MapPlace {
    pub place: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub events: Vec<MapEvent>,
}

#[derive(Serialize, Debug, Clone)]
pub struct MapData {
    pub geocoded: Vec<MapPlace>,
    /// Places without coordinates (never geocoded or not found).
    pub uncached: Vec<MapPlace>,
}

/// `only`: restrict to one person (mini map in a profile).
pub fn build(db: &Database, pol: &Policy, only: Option<Id>) -> MapData {
    let mut by: BTreeMap<String, (String, Vec<MapEvent>)> = BTreeMap::new();
    let mut push = |place: &str, ev: MapEvent| {
        let t = place.trim();
        if t.is_empty() {
            return;
        }
        by.entry(t.to_lowercase()).or_insert_with(|| (t.to_string(), vec![])).1.push(ev);
    };
    let visible = |p: &Person| !(pol.active && (p.status != Status::Published || pol.should_hide(p)));
    for raw in db.persons.values() {
        if only.map(|o| o != raw.id).unwrap_or(false) || !visible(raw) {
            continue;
        }
        let (d, _) = pol.redact(raw);
        let photo = if pol.photo_hidden(raw) { None } else { d.photo };
        let mk = |kind: &str, label: &str, date: &str| MapEvent {
            person: d.id,
            name: d.full_name(),
            photo,
            clans: d.clans.clone(),
            kind: kind.into(),
            label: label.into(),
            date: date.into(),
        };
        push(&d.birth_place, mk("birth", "", &d.birth_date));
        push(&d.death_place, mk("death", "", &d.death_date));
        for e in &d.events {
            push(&e.place, mk("event", &e.name, &e.date));
        }
    }
    for u in db.unions.values() {
        if u.place.trim().is_empty() {
            continue;
        }
        for pid in [Some(u.a), u.b].into_iter().flatten() {
            if only.map(|o| o != pid).unwrap_or(false) {
                continue;
            }
            let Some(raw) = db.persons.get(&pid) else { continue };
            if !visible(raw) {
                continue;
            }
            let other_l = u.partner_of(pid).and_then(|o| db.persons.get(&o)).map(|o| pol.is_living(o)).unwrap_or(false);
            let ur = pol.redact_union(u, pol.is_living(raw), other_l);
            if ur.place.is_empty() {
                continue;
            }
            let (d, _) = pol.redact(raw);
            let partner = u.partner_of(pid).and_then(|o| db.persons.get(&o)).map(|o| o.full_name()).unwrap_or_else(|| format!("{} {}", u.partner_first_name, u.partner_last_name).trim().to_string());
            push(
                &ur.place,
                MapEvent {
                    person: d.id,
                    name: d.full_name(),
                    photo: if pol.photo_hidden(raw) { None } else { d.photo },
                    clans: d.clans.clone(),
                    kind: "marriage".into(),
                    label: partner,
                    date: ur.date.clone(),
                },
            );
        }
    }
    let mut out = MapData { geocoded: vec![], uncached: vec![] };
    for (_, (name, events)) in by {
        let pl = db.place_by_name(&name);
        let (lat, lng) = (pl.and_then(|p| p.lat), pl.and_then(|p| p.lng));
        let mp = MapPlace { place: name, lat, lng, events };
        if lat.is_some() && lng.is_some() {
            out.geocoded.push(mp);
        } else {
            out.uncached.push(mp);
        }
    }
    out
}
