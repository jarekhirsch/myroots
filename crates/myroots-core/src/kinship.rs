//! Kinship calculator.
//!
//! 1. Blood relation: common ancestor (BFS up the parents), with the depth of
//!    both people below it — the UI turns (dA, dB, genders, full/half) into a
//!    translated term (parent, grandparent, sibling, uncle, N-th cousin …).
//! 2. No blood relation: shortest route through parent / child / spouse edges
//!    (affinity: in-laws, step-relations …).
//! 3. Always: the path connecting both people, step by step.

use crate::db::Database;
use crate::life;
use crate::model::*;
use crate::privacy::Policy;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};

#[derive(Serialize, Debug, Clone)]
pub struct PathNode {
    pub id: Id,
    pub name: String,
    pub years: String,
    pub gender: Gender,
    pub photo: Option<Id>,
    /// Edge used to reach this node: "" | parent | child | spouse
    pub edge: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Kinship {
    /// same | blood | affinity | none
    #[serde(rename = "type")]
    pub kind: String,
    pub a: Id,
    pub b: Id,
    pub name_a: String,
    pub name_b: String,
    pub gender_a: Gender,
    pub gender_b: Gender,
    /// Generations from A / B up to the common ancestor.
    pub depth_a: u32,
    pub depth_b: u32,
    /// Both parents in common (full siblings / cousins through a couple).
    pub full: bool,
    pub ancestor: Option<Id>,
    /// Edge signature of the affinity route, e.g. "spouse,parent".
    pub signature: String,
    pub path: Vec<PathNode>,
}

fn ancestors(db: &Database, id: Id) -> HashMap<Id, (u32, Id)> {
    // id -> (depth, via child)
    let mut out: HashMap<Id, (u32, Id)> = HashMap::new();
    out.insert(id, (0, 0));
    let mut q = VecDeque::from([(id, 0u32)]);
    while let Some((n, d)) = q.pop_front() {
        let Some(p) = db.persons.get(&n) else { continue };
        for par in [p.father, p.mother].into_iter().flatten() {
            if !db.persons.contains_key(&par) || out.contains_key(&par) {
                continue;
            }
            out.insert(par, (d + 1, n));
            q.push_back((par, d + 1));
        }
    }
    out
}

fn node(db: &Database, pol: &Policy, id: Id, edge: &str) -> PathNode {
    let p = &db.persons[&id];
    let (r, _) = pol.redact(p);
    PathNode {
        id,
        name: p.full_name(),
        years: life::years(&r),
        gender: p.gender,
        photo: if pol.photo_hidden(p) { None } else { p.photo },
        edge: edge.into(),
    }
}

pub fn compute(db: &Database, pol: &Policy, a: Id, b: Id) -> Option<Kinship> {
    let pa = db.persons.get(&a)?;
    let pb = db.persons.get(&b)?;
    let mut res = Kinship {
        kind: "none".into(),
        a,
        b,
        name_a: pa.full_name(),
        name_b: pb.full_name(),
        gender_a: pa.gender,
        gender_b: pb.gender,
        depth_a: 0,
        depth_b: 0,
        full: false,
        ancestor: None,
        signature: String::new(),
        path: vec![],
    };
    if a == b {
        res.kind = "same".into();
        res.path = vec![node(db, pol, a, "")];
        return Some(res);
    }

    // 1. Blood
    let anc_a = ancestors(db, a);
    let anc_b = ancestors(db, b);
    let mut best_sum = u32::MAX;
    let mut best: Vec<Id> = vec![];
    let mut keys: Vec<&Id> = anc_a.keys().collect();
    keys.sort();
    for cid in keys {
        if let Some((db_, _)) = anc_b.get(cid) {
            let s = anc_a[cid].0 + db_;
            if s < best_sum {
                best_sum = s;
                best = vec![*cid];
            } else if s == best_sum {
                best.push(*cid);
            }
        }
    }
    if let Some(&cid) = best.first() {
        let da = anc_a[&cid].0;
        let dbb = anc_b[&cid].0;
        res.kind = "blood".into();
        res.depth_a = da;
        res.depth_b = dbb;
        res.full = best.len() >= 2;
        res.ancestor = Some(cid);
        // Path: A → … → ancestor → … → B
        let mut up = vec![];
        let mut cur = cid;
        while cur != a {
            up.push(cur);
            match anc_a.get(&cur) {
                Some((_, via)) if *via != 0 => cur = *via,
                _ => break,
            }
        }
        up.reverse();
        let mut down = vec![];
        let mut cur = cid;
        while cur != b {
            match anc_b.get(&cur) {
                Some((_, via)) if *via != 0 => {
                    down.push(*via);
                    cur = *via;
                }
                _ => break,
            }
        }
        let mut path = vec![node(db, pol, a, "")];
        for id in up {
            path.push(node(db, pol, id, "parent"));
        }
        for id in down {
            path.push(node(db, pol, id, "child"));
        }
        res.path = path;
        return Some(res);
    }

    // 2. Affinity — BFS over parent / child / spouse edges.
    let ix = db.index();
    let mut prev: HashMap<Id, Option<(Id, &'static str)>> = HashMap::new();
    prev.insert(a, None);
    let mut q = VecDeque::from([a]);
    let mut steps = 0;
    while let Some(n) = q.pop_front() {
        steps += 1;
        if n == b || steps > 200_000 {
            break;
        }
        let p = &db.persons[&n];
        let mut edges: Vec<(Id, &'static str)> = vec![];
        for par in [p.father, p.mother].into_iter().flatten() {
            if db.persons.contains_key(&par) {
                edges.push((par, "parent"));
            }
        }
        for c in ix.children_of(n) {
            edges.push((*c, "child"));
        }
        for s in ix.partners(db, n) {
            edges.push((s, "spouse"));
        }
        for (next, t) in edges {
            if prev.contains_key(&next) {
                continue;
            }
            prev.insert(next, Some((n, t)));
            q.push_back(next);
        }
    }
    if !prev.contains_key(&b) {
        return Some(res);
    }
    let mut route = vec![];
    let mut cur = Some(b);
    while let Some(c) = cur {
        let entry = prev[&c];
        route.push((c, entry.map(|e| e.1).unwrap_or("")));
        cur = entry.map(|e| e.0);
    }
    route.reverse();
    res.kind = "affinity".into();
    res.signature = route.iter().skip(1).map(|(_, e)| *e).collect::<Vec<_>>().join(",");
    res.path = route.into_iter().map(|(id, e)| node(db, pol, id, e)).collect();
    Some(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::life::Ctx;
    use crate::ops::*;
    use chrono::NaiveDate;

    #[test]
    fn cousins_and_inlaws() {
        let c = Ctx::new(0, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let mut db = Database::new(0);
        let p = |n: &str, g: Gender| Person { first_name: n.into(), gender: g, ..Default::default() };
        let gf = create_person(&mut db, &c, p("GF", Gender::Male), RelationEdits::default()).unwrap();
        let gm = add_relative(&mut db, &c, gf, NewRelation::Spouse, p("GM", Gender::Female), None).unwrap();
        let s1 = add_relative(&mut db, &c, gf, NewRelation::Child, p("S1", Gender::Male), None).unwrap();
        let s2 = add_relative(&mut db, &c, gf, NewRelation::Child, p("S2", Gender::Female), None).unwrap();
        let c1 = add_relative(&mut db, &c, s1, NewRelation::Child, p("C1", Gender::Male), None).unwrap();
        let c2 = add_relative(&mut db, &c, s2, NewRelation::Child, p("C2", Gender::Female), None).unwrap();
        let w = add_relative(&mut db, &c, s1, NewRelation::Spouse, p("W", Gender::Female), None).unwrap();
        let pol = Policy::owner(0, c.today);
        let k = compute(&db, &pol, c1, c2).unwrap();
        assert_eq!((k.kind.as_str(), k.depth_a, k.depth_b, k.full), ("blood", 2, 2, true));
        assert_eq!(k.path.len(), 5);
        let k = compute(&db, &pol, s1, s2).unwrap();
        assert_eq!((k.depth_a, k.depth_b, k.full), (1, 1, true));
        let k = compute(&db, &pol, w, gm).unwrap();
        assert_eq!(k.kind, "affinity");
        assert_eq!(k.signature, "spouse,parent");
    }
}
