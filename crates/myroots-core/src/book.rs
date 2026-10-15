//! Printable genealogy book: selects people and splits them into chapters.
//! The UI renders every person's profile page from `query::profile`.

use crate::date;
use crate::db::Database;
use crate::model::*;
use crate::privacy::Policy;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct BookRequest {
    /// clans | ancestors | descendants
    pub mode: String,
    /// generations | clans | families
    pub structure: String,
    pub person: Option<Id>,
    pub max_gen: u32,
    pub year_from: Option<i32>,
    pub year_to: Option<i32>,
    pub clans: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Chapter {
    /// generation | clan | no_clan | family | other
    pub kind: String,
    /// Generation number, clan name, or the names of the parents.
    pub title: String,
    pub number: u32,
    pub names: Vec<String>,
    pub persons: Vec<Id>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Book {
    pub chapters: Vec<Chapter>,
    pub total: usize,
}

fn walk(db: &Database, start: Id, up: bool, max_gen: u32) -> BTreeSet<Id> {
    let ix = db.index();
    let mut ids = BTreeSet::from([start]);
    let mut q = VecDeque::from([(start, 0u32)]);
    while let Some((id, d)) = q.pop_front() {
        if max_gen > 0 && d >= max_gen {
            continue;
        }
        let next: Vec<Id> = if up { db.persons[&id].parents().filter(|p| db.persons.contains_key(p)).collect() } else { ix.children_of(id).to_vec() };
        for n in next {
            if ids.insert(n) {
                q.push_back((n, d + 1));
            }
        }
    }
    ids
}

fn sort_ids(db: &Database, list: &mut [Id]) {
    list.sort_by(|a, b| {
        let ya = date::year(&db.persons[a].birth_date).unwrap_or(9999);
        let yb = date::year(&db.persons[b].birth_date).unwrap_or(9999);
        ya.cmp(&yb).then(a.cmp(b))
    });
}

pub fn build(db: &Database, pol: &Policy, req: &BookRequest) -> Book {
    let ix = db.index();
    let mut ids: BTreeSet<Id> = match (req.mode.as_str(), req.person.filter(|p| db.persons.contains_key(p))) {
        ("ancestors", Some(p)) => walk(db, p, true, req.max_gen),
        ("descendants", Some(p)) => {
            let mut s = walk(db, p, false, req.max_gen);
            let extra: Vec<Id> = s.iter().flat_map(|id| ix.partners(db, *id)).collect();
            s.extend(extra);
            s
        }
        _ => db
            .persons
            .values()
            .filter(|p| req.clans.is_empty() || p.clans.iter().any(|c| req.clans.contains(c)))
            .map(|p| p.id)
            .collect(),
    };
    if pol.active {
        ids.retain(|id| {
            let p = &db.persons[id];
            p.status == Status::Published && !pol.should_hide(p)
        });
    }
    if req.year_from.is_some() || req.year_to.is_some() {
        ids.retain(|id| match date::year(&db.persons[id].birth_date) {
            None => true,
            Some(y) => req.year_from.map(|f| y >= f).unwrap_or(true) && req.year_to.map(|t| y <= t).unwrap_or(true),
        });
    }
    let total = ids.len();
    let set: HashSet<Id> = ids.iter().copied().collect();
    let mut chapters = vec![];

    match req.structure.as_str() {
        "clans" => {
            let mut by: BTreeMap<String, Vec<Id>> = BTreeMap::new();
            let mut none = vec![];
            for id in &ids {
                match db.persons[id].clans.first() {
                    Some(c) => by.entry(c.clone()).or_default().push(*id),
                    None => none.push(*id),
                }
            }
            let mut keys: Vec<String> = by.keys().cloned().collect();
            keys.sort_by_key(|k| norm(k));
            for (i, k) in keys.into_iter().enumerate() {
                let mut list = by.remove(&k).unwrap();
                sort_ids(db, &mut list);
                chapters.push(Chapter { kind: "clan".into(), title: k, number: i as u32 + 1, names: vec![], persons: list });
            }
            if !none.is_empty() {
                sort_ids(db, &mut none);
                chapters.push(Chapter { kind: "no_clan".into(), title: String::new(), number: 0, names: vec![], persons: none });
            }
        }
        "families" => {
            let mut has_children = HashSet::new();
            for id in &ids {
                for p in db.persons[id].parents() {
                    if set.contains(&p) {
                        has_children.insert(p);
                    }
                }
            }
            let mut families: BTreeMap<(Id, Id), BTreeSet<Id>> = BTreeMap::new();
            let mut assigned: HashMap<Id, (Id, Id)> = HashMap::new();
            for id in &ids {
                if !has_children.contains(id) {
                    continue;
                }
                let spouse = ix.partners(db, *id).into_iter().find(|s| set.contains(s));
                let key = match spouse {
                    Some(s) => ((*id).min(s), (*id).max(s)),
                    None => (*id, 0),
                };
                let fam = families.entry(key).or_default();
                fam.insert(*id);
                if let Some(s) = spouse {
                    fam.insert(s);
                }
            }
            for (k, fam) in &families {
                for p in fam {
                    assigned.entry(*p).or_insert(*k);
                }
            }
            let mut members: BTreeMap<(Id, Id), Vec<Id>> = families.iter().map(|(k, v)| (*k, v.iter().copied().collect())).collect();
            for id in &ids {
                if assigned.contains_key(id) {
                    continue;
                }
                let parent_fam = db.persons[id].parents().find_map(|p| assigned.get(&p).copied());
                if let Some(k) = parent_fam {
                    members.get_mut(&k).unwrap().push(*id);
                    assigned.insert(*id, k);
                }
            }
            let mut rest: Vec<Id> = ids.iter().copied().filter(|i| !assigned.contains_key(i)).collect();
            let mut fams: Vec<((Id, Id), Vec<Id>)> = members.into_iter().collect();
            fams.sort_by_key(|(k, _)| {
                families[k].iter().map(|p| date::year(&db.persons[p].birth_date).unwrap_or(9999)).min().unwrap_or(9999)
            });
            for (i, (k, mut list)) in fams.into_iter().enumerate() {
                let names: Vec<String> = families[&k].iter().map(|p| db.persons[p].full_name()).collect();
                list.sort();
                list.dedup();
                sort_ids(db, &mut list);
                chapters.push(Chapter { kind: "family".into(), title: names.join(" & "), number: i as u32 + 1, names, persons: list });
            }
            if !rest.is_empty() {
                sort_ids(db, &mut rest);
                chapters.push(Chapter { kind: "other".into(), title: String::new(), number: 0, names: vec![], persons: rest });
            }
        }
        _ => {
            // Generations: generation = max(parents) + 1 inside the set (cycle-safe).
            let mut gen: HashMap<Id, u32> = HashMap::new();
            fn calc(db: &Database, set: &HashSet<Id>, id: Id, gen: &mut HashMap<Id, u32>, visiting: &mut HashSet<Id>) -> u32 {
                if let Some(g) = gen.get(&id) {
                    return *g;
                }
                if !visiting.insert(id) {
                    return 0;
                }
                let mut g = 0;
                for p in db.persons[&id].parents() {
                    if set.contains(&p) {
                        g = g.max(calc(db, set, p, gen, visiting) + 1);
                    }
                }
                visiting.remove(&id);
                gen.insert(id, g);
                g
            }
            let mut visiting = HashSet::new();
            let mut by: BTreeMap<u32, Vec<Id>> = BTreeMap::new();
            for id in &ids {
                let g = calc(db, &set, *id, &mut gen, &mut visiting);
                by.entry(g).or_default().push(*id);
            }
            for (g, mut list) in by {
                sort_ids(db, &mut list);
                chapters.push(Chapter { kind: "generation".into(), title: roman(g + 1), number: g + 1, names: vec![], persons: list });
            }
        }
    }
    Book { chapters, total }
}

pub fn roman(mut n: u32) -> String {
    let map = [(1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"), (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I")];
    let mut out = String::new();
    for (v, r) in map {
        while n >= v {
            out.push_str(r);
            n -= v;
        }
    }
    if out.is_empty() {
        "I".into()
    } else {
        out
    }
}
