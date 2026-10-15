//! Data health: consistency checks and completeness report.
//!
//! Relations are stored once, so the plugin's "one-sided relation" warnings
//! cannot happen anymore. The remaining checks are about plausibility.

use crate::date;
use crate::db::Database;
use crate::life;
use crate::model::*;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};

pub const PARENT_MIN_AGE: i32 = 12;
pub const PARENT_MAX_AGE: i32 = 70;
pub const MAX_LIFESPAN: i32 = 120;
pub const MARRIAGE_MIN_AGE: i32 = 12;

#[derive(Serialize, Debug, Clone)]
pub struct Issue {
    /// "error" | "warn"
    pub severity: String,
    /// Stable code, translated by the UI.
    #[serde(rename = "type")]
    pub kind: String,
    pub person: Id,
    pub related: Option<Id>,
    /// Message arguments (numbers, names, roles), in the order of the message.
    pub args: Vec<Value>,
}

fn add(v: &mut Vec<Issue>, sev: &str, kind: &str, person: Id, related: Option<Id>, args: Vec<Value>) {
    v.push(Issue { severity: sev.into(), kind: kind.into(), person, related, args });
}

pub fn analyze(db: &Database) -> Vec<Issue> {
    let mut v = Vec::new();
    for p in db.persons.values() {
        let by = date::year(&p.birth_date);
        let dy = date::year(&p.death_date);
        if let (Some(b), Some(d)) = (by, dy) {
            if d < b {
                add(&mut v, "error", "death_before_birth", p.id, None, vec![json!(d), json!(b)]);
            } else if d - b > MAX_LIFESPAN {
                add(&mut v, "warn", "implausible_age", p.id, None, vec![json!(d - b)]);
            }
        }
        // Generations
        if let Some(cb) = by {
            for (role, pid, rel) in [("father", p.father, p.father_rel), ("mother", p.mother, p.mother_rel)] {
                let Some(pid) = pid else { continue };
                if rel != ParentRel::Bio {
                    continue;
                }
                let Some(par) = db.persons.get(&pid) else { continue };
                let pname = par.full_name();
                if let Some(pb) = date::year(&par.birth_date) {
                    let gap = cb - pb;
                    if gap <= 0 {
                        add(&mut v, "error", "parent_younger", p.id, Some(pid), vec![json!(role), json!(pname)]);
                    } else if gap < PARENT_MIN_AGE {
                        add(&mut v, "warn", "parent_too_young", p.id, Some(pid), vec![json!(role), json!(pname), json!(gap)]);
                    } else if gap > PARENT_MAX_AGE {
                        add(&mut v, "warn", "parent_too_old", p.id, Some(pid), vec![json!(role), json!(pname), json!(gap)]);
                    }
                }
                if let Some(pd) = date::year(&par.death_date) {
                    let margin = if role == "father" { 1 } else { 0 };
                    if pd + margin < cb {
                        add(&mut v, "error", "parent_dead_before_birth", p.id, Some(pid), vec![json!(role), json!(pname), json!(pd), json!(cb)]);
                    }
                }
            }
        }
        // Gender roles
        if let Some(f) = p.father.and_then(|f| db.persons.get(&f)) {
            if f.gender == Gender::Female {
                add(&mut v, "warn", "gender_role_conflict_father", p.id, Some(f.id), vec![json!(f.full_name())]);
            }
        }
        if let Some(m) = p.mother.and_then(|m| db.persons.get(&m)) {
            if m.gender == Gender::Male {
                add(&mut v, "warn", "gender_role_conflict_mother", p.id, Some(m.id), vec![json!(m.full_name())]);
            }
        }
        if p.mother.is_some() && p.mother == p.father {
            add(&mut v, "error", "same_parent_twice", p.id, p.mother, vec![]);
        }
    }
    // Marriages
    for u in db.unions.values() {
        for (me, other) in [(Some(u.a), u.b), (u.b, Some(u.a))] {
            let Some(me) = me.and_then(|m| db.persons.get(&m)) else { continue };
            let who = match other.and_then(|o| db.persons.get(&o)) {
                Some(o) => o.full_name(),
                None => format!("{} {}", u.partner_first_name, u.partner_last_name).trim().to_string(),
            };
            let by = date::year(&me.birth_date);
            let dy = date::year(&me.death_date);
            let my = date::year(&u.date);
            let ey = date::year(&u.end_date);
            if let (Some(m), Some(b)) = (my, by) {
                if m < b {
                    add(&mut v, "error", "marriage_before_birth", me.id, other, vec![json!(m), json!(who), json!(b)]);
                } else if m - b < MARRIAGE_MIN_AGE {
                    add(&mut v, "warn", "marriage_too_young", me.id, other, vec![json!(m - b), json!(who)]);
                }
            }
            if let (Some(m), Some(d)) = (my, dy) {
                if m > d {
                    add(&mut v, "error", "marriage_after_death", me.id, other, vec![json!(m), json!(who), json!(d)]);
                }
            }
            if me.id == u.a {
                if let (Some(m), Some(e)) = (my, ey) {
                    if e < m {
                        add(&mut v, "error", "marriage_end_before_start", me.id, other, vec![json!(e), json!(m), json!(who)]);
                    }
                }
            }
        }
    }
    // Ancestor cycles (should be impossible through the app, but imports may contain them)
    let mut color: HashMap<Id, u8> = HashMap::new();
    let mut reported = HashSet::new();
    let ids: Vec<Id> = db.persons.keys().copied().collect();
    for start in ids {
        if color.get(&start).copied().unwrap_or(0) != 0 {
            continue;
        }
        let mut stack: Vec<(Id, usize)> = vec![(start, 0)];
        let mut path: Vec<Id> = vec![];
        while let Some((node, pi)) = stack.pop() {
            if pi == 0 {
                color.insert(node, 1);
                path.push(node);
            }
            let parents: Vec<Id> = db.persons.get(&node).map(|p| p.parents().filter(|x| db.persons.contains_key(x)).collect()).unwrap_or_default();
            if pi < parents.len() {
                stack.push((node, pi + 1));
                let next = parents[pi];
                match color.get(&next).copied().unwrap_or(0) {
                    1 => {
                        if let Some(ci) = path.iter().position(|x| *x == next) {
                            let cyc: Vec<Id> = path[ci..].to_vec();
                            let k = *cyc.iter().min().unwrap();
                            if reported.insert(k) {
                                let names: Vec<Value> = cyc.iter().map(|i| json!(db.name_of(*i))).collect();
                                add(&mut v, "error", "ancestor_cycle", cyc[0], None, vec![json!(names)]);
                            }
                        }
                    }
                    0 => stack.push((next, 0)),
                    _ => {}
                }
            } else {
                color.insert(node, 2);
                path.pop();
            }
        }
    }
    v.sort_by(|a, b| (a.severity != "error").cmp(&(b.severity != "error")).then(a.person.cmp(&b.person)));
    v
}

#[derive(Serialize, Debug)]
pub struct Completeness {
    pub criteria: Vec<String>,
    pub missing: BTreeMap<String, Vec<Id>>,
    pub applicable: BTreeMap<String, usize>,
    pub overall: u32,
    pub checked: usize,
    /// Per-person score (0–100).
    pub scores: BTreeMap<Id, u32>,
}

pub const CRITERIA: &[&str] = &["birth_date", "birth_place", "gender", "death_date", "death_place", "biography", "photo", "source", "parent"];

pub fn completeness(db: &Database, today: chrono::NaiveDate) -> Completeness {
    let ix = db.index();
    let mut missing: BTreeMap<String, Vec<Id>> = CRITERIA.iter().map(|c| (c.to_string(), vec![])).collect();
    let mut applicable: BTreeMap<String, usize> = CRITERIA.iter().map(|c| (c.to_string(), 0)).collect();
    let mut scores = BTreeMap::new();
    let (mut filled, mut total) = (0usize, 0usize);
    for p in db.persons.values() {
        let deceased = !life::is_alive(p, db.settings.max_age, today);
        let has_source = !ix.sources_of(p.id).is_empty() || !p.citations.is_empty();
        let checks: [(&str, bool, bool); 9] = [
            ("birth_date", true, !p.birth_date.is_empty()),
            ("birth_place", true, !p.birth_place.is_empty()),
            ("gender", true, p.gender != Gender::Unset),
            ("death_date", deceased, !p.death_date.is_empty()),
            ("death_place", deceased, !p.death_place.is_empty()),
            ("biography", true, !strip_tags(&p.biography).is_empty()),
            ("photo", true, p.photo.is_some()),
            ("source", true, has_source),
            ("parent", true, p.mother.is_some() || p.father.is_some()),
        ];
        let (mut pf, mut pt) = (0, 0);
        for (k, app, ok) in checks {
            if !app {
                continue;
            }
            *applicable.get_mut(k).unwrap() += 1;
            total += 1;
            pt += 1;
            if ok {
                filled += 1;
                pf += 1;
            } else {
                missing.get_mut(k).unwrap().push(p.id);
            }
        }
        scores.insert(p.id, if pt > 0 { (pf * 100 / pt) as u32 } else { 100 });
    }
    Completeness {
        criteria: CRITERIA.iter().map(|s| s.to_string()).collect(),
        missing,
        applicable,
        overall: if total > 0 { ((filled as f64 / total as f64) * 100.0).round() as u32 } else { 100 },
        checked: db.persons.len(),
        scores,
    }
}
