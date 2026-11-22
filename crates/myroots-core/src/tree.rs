//! Data for the interactive family tree (family-chart format) and the
//! whole-family graph.

use crate::db::Database;
use crate::life;
use crate::model::*;
use crate::privacy::Policy;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Debug, Clone)]
pub struct NodeData {
    pub gender: String,
    #[serde(rename = "first name")]
    pub first_name: String,
    #[serde(rename = "last name")]
    pub last_name: String,
    pub maiden_name: String,
    pub birthday: String,
    pub birth_place: String,
    pub death_date: String,
    pub death_place: String,
    pub years: String,
    pub avatar: Option<Id>,
    pub living: bool,
    pub anon: bool,
    pub clans: Vec<String>,
    /// partner id → marriage date
    pub marriages: BTreeMap<String, String>,
    pub father_rel: String,
    pub mother_rel: String,
    pub pets: Vec<PetChip>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PetChip {
    pub id: Id,
    pub name: String,
    pub species: String,
    pub photo: Option<Id>,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct Rels {
    pub parents: Vec<String>,
    pub spouses: Vec<String>,
    pub children: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Node {
    pub id: String,
    pub data: NodeData,
    pub rels: Rels,
}

pub fn nodes(db: &Database, pol: &Policy) -> Vec<Node> {
    let ix = db.index();
    let include = |p: &Person| !pol.active || p.status == Status::Published;
    let mut nodes: Vec<Node> = Vec::new();
    let mut pos: HashMap<Id, usize> = HashMap::new();
    for raw in db.persons.values() {
        if !include(raw) {
            continue;
        }
        let anon = pol.should_hide(raw);
        let d = pol.view(raw);
        let photo_off = anon || pol.photo_hidden(raw);
        let mut marriages = BTreeMap::new();
        let mut spouses = vec![];
        for uid in ix.unions_of(raw.id) {
            let u = &db.unions[uid];
            if let Some(p) = u.partner_of(raw.id) {
                if db.persons.get(&p).map(|x| include(x)).unwrap_or(false) {
                    if !spouses.contains(&p.to_string()) {
                        spouses.push(p.to_string());
                    }
                    if !anon && !u.date.is_empty() {
                        let pl = db.persons.get(&p).map(|x| pol.is_living(x)).unwrap_or(false);
                        let ur = pol.redact_union(u, pol.is_living(raw), pl);
                        marriages.insert(p.to_string(), ur.date);
                    }
                }
            }
        }
        let pets = if anon || (pol.active && pol.level != "censor_living") {
            vec![]
        } else {
            ix.pets_of(raw.id)
                .iter()
                .filter_map(|i| db.pets.get(i))
                .filter(|p| !pol.active || p.status == Status::Published)
                .map(|p| PetChip { id: p.id, name: p.name.clone(), species: p.species.clone(), photo: p.photo })
                .collect()
        };
        let data = if anon {
            NodeData {
                gender: String::new(),
                first_name: String::new(),
                last_name: String::new(),
                maiden_name: String::new(),
                birthday: String::new(),
                birth_place: String::new(),
                death_date: String::new(),
                death_place: String::new(),
                years: String::new(),
                avatar: None,
                living: true,
                anon: true,
                clans: vec![],
                marriages,
                father_rel: String::new(),
                mother_rel: String::new(),
                pets,
            }
        } else {
            NodeData {
                gender: d.gender.code().to_string(),
                first_name: d.first_name.clone(),
                last_name: d.last_name.clone(),
                maiden_name: d.maiden_name.clone(),
                birthday: d.birth_date.clone(),
                birth_place: d.birth_place.clone(),
                death_date: d.death_date.clone(),
                death_place: d.death_place.clone(),
                years: life::years(&d),
                avatar: if photo_off { None } else { d.photo },
                living: pol.is_living(raw),
                anon: false,
                clans: d.clans.clone(),
                marriages,
                father_rel: if raw.father.is_some() { rel_code(raw.father_rel) } else { String::new() },
                mother_rel: if raw.mother.is_some() { rel_code(raw.mother_rel) } else { String::new() },
                pets,
            }
        };
        let mut parents = vec![];
        for p in [raw.father, raw.mother].into_iter().flatten() {
            if db.persons.get(&p).map(|x| include(x)).unwrap_or(false) {
                parents.push(p.to_string());
            }
        }
        let children: Vec<String> = ix
            .children_of(raw.id)
            .iter()
            .filter(|c| db.persons.get(c).map(|x| include(x)).unwrap_or(false))
            .map(|c| c.to_string())
            .collect();
        pos.insert(raw.id, nodes.len());
        nodes.push(Node { id: raw.id.to_string(), data, rels: Rels { parents, spouses, children } });
    }
    // family-chart requires both parents of a child to be partners.
    let pairs: Vec<(String, String)> = nodes
        .iter()
        .filter(|n| n.rels.parents.len() == 2)
        .map(|n| (n.rels.parents[0].clone(), n.rels.parents[1].clone()))
        .collect();
    for (a, b) in pairs {
        let (ia, ib) = (a.parse::<Id>().ok().and_then(|x| pos.get(&x).copied()), b.parse::<Id>().ok().and_then(|x| pos.get(&x).copied()));
        if let (Some(ia), Some(ib)) = (ia, ib) {
            if !nodes[ia].rels.spouses.contains(&b) {
                nodes[ia].rels.spouses.push(b.clone());
            }
            if !nodes[ib].rels.spouses.contains(&a) {
                nodes[ib].rels.spouses.push(a.clone());
            }
        }
    }
    nodes
}

fn rel_code(r: ParentRel) -> String {
    match r {
        ParentRel::Bio => "bio",
        ParentRel::Adopted => "adopted",
        ParentRel::Foster => "foster",
        ParentRel::Step => "step",
    }
    .into()
}

/// Default person for the tree view according to the settings.
pub fn start_person(db: &Database) -> Option<Id> {
    let s = &db.settings;
    if let Some(p) = s.tree_start_person.filter(|p| db.persons.contains_key(p)) {
        if s.tree_start_mode == "person" || s.tree_start_mode.is_empty() {
            return Some(p);
        }
    }
    let key = |p: &Person| crate::date::sort_key(&p.birth_date);
    match s.tree_start_mode.as_str() {
        "youngest" => db.persons.values().filter(|p| !p.birth_date.is_empty()).max_by_key(|p| key(p)).map(|p| p.id),
        "oldest" => db.persons.values().filter(|p| !p.birth_date.is_empty()).min_by_key(|p| key(p)).map(|p| p.id),
        "last" => db.persons.values().max_by_key(|p| (p.updated, p.id)).map(|p| p.id),
        _ => s.tree_start_person.filter(|p| db.persons.contains_key(p)),
    }
    .or_else(|| db.persons.keys().next().copied())
}
