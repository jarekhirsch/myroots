//! Genealogical map: events grouped by place.

use crate::db::Database;
use crate::model::*;
use crate::privacy::Policy;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Debug, Clone)]
pub struct MapEvent {
    pub person: Id,
    pub name: String,
    pub photo: Option<Id>,
    pub clans: Vec<String>,
    /// birth | death | marriage | event | burial
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

/// A cemetery with the people buried there (filter of the map).
#[derive(Serialize, Debug, Clone)]
pub struct MapCemetery {
    pub name: String,
    /// Position: the average of the graves with coordinates, or the place of
    /// the same name when it has coordinates.
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub persons: Vec<Id>,
    /// Graves shown on the map (with their own position or the cemetery's).
    pub located: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct MapData {
    pub geocoded: Vec<MapPlace>,
    /// Places without coordinates (never geocoded or not found).
    pub uncached: Vec<MapPlace>,
    /// Cemeteries of the family, sorted by name.
    pub cemeteries: Vec<MapCemetery>,
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
    // places by lowercase name, looked up once (db.place_by_name scans all places)
    let places: HashMap<String, &Place> = db.places.values().map(|p| (p.name.trim().to_lowercase(), p)).collect();
    let place = |name: &str| places.get(&name.trim().to_lowercase()).copied();
    // graves: (cemetery key) -> name, people; points with their own position
    let mut cems: BTreeMap<String, (String, Vec<Id>, Vec<(f64, f64)>)> = BTreeMap::new();
    let mut grave_points: BTreeMap<String, MapPlace> = BTreeMap::new();
    for raw in db.persons.values() {
        if only.map(|o| o != raw.id).unwrap_or(false) || !visible(raw) {
            continue;
        }
        let d = pol.view(raw);
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
        // the grave: on its own position, or at the cemetery when that place has coordinates
        let g = &d.grave;
        let cem = g.cemetery.trim();
        if !cem.is_empty() || (g.lat.is_some() && g.lng.is_some()) {
            let key = norm(cem);
            let entry = cems.entry(key.clone()).or_insert_with(|| (cem.to_string(), vec![], vec![]));
            entry.1.push(d.id);
            let label = if g.location.trim().is_empty() { cem.to_string() } else if cem.is_empty() { g.location.trim().to_string() } else { format!("{cem}, {}", g.location.trim()) };
            let ev = mk("burial", &label, &d.death_date);
            if let (Some(lat), Some(lng)) = (g.lat, g.lng) {
                entry.2.push((lat, lng));
                // graves a few metres apart share one pin
                let k = format!("{key}|{:.5}|{:.5}", lat, lng);
                grave_points.entry(k).or_insert_with(|| MapPlace { place: if cem.is_empty() { label.clone() } else { cem.to_string() }, lat: Some(lat), lng: Some(lng), events: vec![] }).events.push(ev);
            } else if !cem.is_empty() {
                if let Some(pl) = place(cem).filter(|p| p.lat.is_some() && p.lng.is_some()) {
                    let k = format!("{key}|place");
                    grave_points.entry(k).or_insert_with(|| MapPlace { place: cem.to_string(), lat: pl.lat, lng: pl.lng, events: vec![] }).events.push(ev);
                }
            }
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
            let d = pol.view(raw);
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
    let mut out = MapData { geocoded: vec![], uncached: vec![], cemeteries: vec![] };
    let mut located: HashMap<String, usize> = HashMap::new();
    for (k, mp) in grave_points {
        let key = k.split('|').next().unwrap_or("").to_string();
        *located.entry(key).or_default() += mp.events.len();
        out.geocoded.push(mp);
    }
    for (key, (name, persons, pts)) in cems {
        let (lat, lng) = if pts.is_empty() {
            place(&name).map(|p| (p.lat, p.lng)).filter(|(a, b)| a.is_some() && b.is_some()).unwrap_or((None, None))
        } else {
            let n = pts.len() as f64;
            (Some(pts.iter().map(|p| p.0).sum::<f64>() / n), Some(pts.iter().map(|p| p.1).sum::<f64>() / n))
        };
        out.cemeteries.push(MapCemetery { name, lat, lng, located: located.get(&key).copied().unwrap_or(0), persons });
    }
    out.cemeteries.sort_by_cached_key(|c| (c.name.is_empty(), norm(&c.name)));
    for (_, (name, events)) in by {
        let pl = place(&name);
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
