//! Family statistics.

use crate::date;
use crate::db::Database;
use crate::life;
use crate::model::*;
use crate::privacy::Policy;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Serialize, Debug, Clone)]
pub struct NamedAge {
    pub id: Id,
    pub name: String,
    pub age: i32,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct Stats {
    pub total: usize,
    pub males: usize,
    pub females: usize,
    pub living: usize,
    pub pets_alive: usize,
    pub pets_dead: usize,
    pub pets_total: usize,
    pub avg_age: Option<f64>,
    pub oldest: Option<NamedAge>,
    pub youngest: Option<NamedAge>,
    pub marriages: usize,
    pub avg_children: Option<f64>,
    pub avg_marriage_age_m: Option<f64>,
    pub avg_marriage_age_f: Option<f64>,
    pub avg_gen_gap: Option<f64>,
    pub names_top10: Vec<(String, usize)>,
    pub surnames_top10: Vec<(String, usize)>,
    pub birth_places_top10: Vec<(String, usize)>,
    pub birth_decades: Vec<(String, usize)>,
    pub death_decades: Vec<(String, usize)>,
    pub age_groups: Vec<(String, usize)>,
    pub children_dist: Vec<(String, usize)>,
    pub months_of_birth: Vec<usize>,
    pub occupations_top10: Vec<(String, usize)>,
    pub generations: usize,
}

fn top(map: HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

fn avg(v: &[i32]) -> Option<f64> {
    if v.is_empty() {
        None
    } else {
        Some(v.iter().map(|x| *x as f64).sum::<f64>() / v.len() as f64)
    }
}

pub fn compute(db: &Database, pol: &Policy, clan: &str) -> Stats {
    let max = db.settings.max_age;
    let today = pol.today;
    let ix = db.index();
    let persons: Vec<&Person> = db
        .persons
        .values()
        .filter(|p| clan.is_empty() || p.clans.iter().any(|c| c == clan))
        .filter(|p| !pol.active || (p.status == Status::Published && !pol.should_hide(p)))
        .collect();
    let ids: HashSet<Id> = persons.iter().map(|p| p.id).collect();

    let mut s = Stats { total: persons.len(), ..Default::default() };
    let mut ages = vec![];
    let mut oldest: Option<NamedAge> = None;
    let mut youngest: Option<NamedAge> = None;
    let (mut names, mut surnames, mut places, mut occ) = (HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new());
    let (mut bdec, mut ddec): (BTreeMap<i32, usize>, BTreeMap<i32, usize>) = (BTreeMap::new(), BTreeMap::new());
    let mut groups = [0usize; 5];
    let mut dist = [0usize; 5];
    let mut months = vec![0usize; 12];
    let mut child_counts = vec![];
    let mut birth_year: HashMap<Id, i32> = HashMap::new();

    for raw in &persons {
        let d = pol.view(raw);
        match d.gender {
            Gender::Male => s.males += 1,
            Gender::Female => s.females += 1,
            _ => {}
        }
        let alive = life::is_alive(raw, max, today);
        if alive {
            s.living += 1;
        }
        // A presumed death (no date, older than the limit) has no real age.
        let presumed = !alive && d.death_date.trim().is_empty();
        if let Some(a) = life::age(&d, max, today).filter(|_| !presumed) {
            ages.push(a);
            if oldest.as_ref().map(|o| a > o.age).unwrap_or(true) {
                oldest = Some(NamedAge { id: d.id, name: d.full_name(), age: a });
            }
            if alive && d.death_date.is_empty() && youngest.as_ref().map(|o| a < o.age).unwrap_or(true) {
                youngest = Some(NamedAge { id: d.id, name: d.full_name(), age: a });
            }
            let gi = match a {
                ..=20 => 0,
                21..=40 => 1,
                41..=60 => 2,
                61..=80 => 3,
                _ => 4,
            };
            groups[gi] += 1;
        }
        if !d.first_name.is_empty() {
            *names.entry(d.first_name.clone()).or_insert(0) += 1;
        }
        if !d.last_name.is_empty() {
            *surnames.entry(d.last_name.clone()).or_insert(0) += 1;
        }
        if !d.maiden_name.is_empty() && d.maiden_name != d.last_name {
            *surnames.entry(d.maiden_name.clone()).or_insert(0) += 1;
        }
        if !d.birth_place.is_empty() {
            *places.entry(d.birth_place.clone()).or_insert(0) += 1;
        }
        if !d.occupation.is_empty() {
            *occ.entry(d.occupation.clone()).or_insert(0) += 1;
        }
        if let Some(y) = date::year(&d.birth_date) {
            birth_year.insert(d.id, y);
            *bdec.entry(y.div_euclid(10) * 10).or_insert(0) += 1;
        }
        if let Some(y) = date::year(&d.death_date) {
            *ddec.entry(y.div_euclid(10) * 10).or_insert(0) += 1;
        }
        let core = date::core(&d.birth_date);
        if core.len() >= 7 {
            if let Ok(m) = core[5..7].parse::<usize>() {
                if (1..=12).contains(&m) {
                    months[m - 1] += 1;
                }
            }
        }
        let kc = ix.children_of(d.id).len();
        child_counts.push(kc as i32);
        dist[kc.min(4)] += 1;
    }

    // Marriages and age at marriage.
    let (mut mm, mut mf) = (vec![], vec![]);
    let mut pairs = 0;
    for u in db.unions.values() {
        let a_in = ids.contains(&u.a);
        let b_in = u.b.map(|b| ids.contains(&b)).unwrap_or(false);
        if !a_in && !b_in {
            continue;
        }
        if u.b.is_some() {
            pairs += 1;
        }
        let Some(md) = date::to_naive(&u.date) else { continue };
        for pid in [Some(u.a), u.b].into_iter().flatten() {
            if !ids.contains(&pid) {
                continue;
            }
            let p = &db.persons[&pid];
            let r = pol.view(p);
            if let Some(bd) = date::to_naive(&r.birth_date) {
                let a = date::years_between(bd, md);
                match p.gender {
                    Gender::Male => mm.push(a),
                    Gender::Female => mf.push(a),
                    _ => {}
                }
            }
        }
    }
    s.marriages = pairs;

    // Generation gap.
    let mut gaps = vec![];
    for p in &persons {
        if let Some(cy) = birth_year.get(&p.id) {
            for par in p.parents() {
                if let Some(py) = birth_year.get(&par) {
                    let g = cy - py;
                    if (12..=60).contains(&g) {
                        gaps.push(g);
                    }
                }
            }
        }
    }

    // Number of generations (longest ancestor chain inside the set).
    let mut depth: HashMap<Id, usize> = HashMap::new();
    fn gen_of(db: &Database, ids: &HashSet<Id>, id: Id, depth: &mut HashMap<Id, usize>, guard: &mut HashSet<Id>) -> usize {
        if let Some(d) = depth.get(&id) {
            return *d;
        }
        if !guard.insert(id) {
            return 1;
        }
        let p = &db.persons[&id];
        let mut g = 1;
        for par in p.parents() {
            if ids.contains(&par) {
                g = g.max(gen_of(db, ids, par, depth, guard) + 1);
            }
        }
        guard.remove(&id);
        depth.insert(id, g);
        g
    }
    let mut guard = HashSet::new();
    for p in &persons {
        let g = gen_of(db, &ids, p.id, &mut depth, &mut guard);
        s.generations = s.generations.max(g);
    }

    // Pets.
    for pet in db.pets.values() {
        if pol.active && pet.status != Status::Published {
            continue;
        }
        if !clan.is_empty() && !pet.owner.map(|o| ids.contains(&o)).unwrap_or(false) {
            continue;
        }
        if pet.death_date.is_empty() {
            s.pets_alive += 1;
        } else {
            s.pets_dead += 1;
        }
    }
    s.pets_total = s.pets_alive + s.pets_dead;

    s.avg_age = avg(&ages);
    s.oldest = oldest;
    s.youngest = youngest;
    s.avg_children = avg(&child_counts);
    s.avg_marriage_age_m = avg(&mm);
    s.avg_marriage_age_f = avg(&mf);
    s.avg_gen_gap = avg(&gaps);
    s.names_top10 = top(names, 10);
    s.surnames_top10 = top(surnames, 10);
    s.birth_places_top10 = top(places, 10);
    s.occupations_top10 = top(occ, 10);
    s.birth_decades = bdec.into_iter().map(|(k, v)| (format!("{k}s"), v)).collect();
    s.death_decades = ddec.into_iter().map(|(k, v)| (format!("{k}s"), v)).collect();
    s.age_groups = ["0-20", "21-40", "41-60", "61-80", "80+"].iter().zip(groups).map(|(k, v)| (k.to_string(), v)).collect();
    s.children_dist = ["0", "1", "2", "3", "4+"].iter().zip(dist).map(|(k, v)| (k.to_string(), v)).collect();
    s.months_of_birth = months;
    s
}
