//! Personal lifeline: a timeline of the events of a person's life,
//! including births and deaths of close relatives within the lifespan.

use crate::date;
use crate::db::Database;
use crate::life;
use crate::model::*;
use crate::privacy::Policy;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct LifelineEvent {
    /// birth | event | marriage | marriage_end | child_birth | grandchild_birth |
    /// sibling_birth | sibling_death | parent_death | grandparent_death | death | presumed_death
    pub kind: String,
    pub date: String,
    pub place: String,
    /// Custom event name.
    pub name: String,
    pub description: String,
    pub image: Option<Id>,
    /// Related person (partner, child, sibling …).
    pub related: Option<Id>,
    pub related_name: String,
    /// mother | father (parent_death); end cause key (marriage_end).
    pub role: String,
    /// Age of the person at the event (negative = before birth).
    pub age: Option<i32>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Lifeline {
    pub person: Id,
    pub name: String,
    pub gender: Gender,
    pub photo: Option<Id>,
    pub years: String,
    pub events: Vec<LifelineEvent>,
}

fn within(event_date: &str, p: &Person) -> bool {
    let Some(ev) = date::to_naive(event_date) else { return false };
    if let Some(b) = date::to_naive(&p.birth_date) {
        if ev < b {
            return false;
        }
    }
    if let Some(d) = date::to_naive(&p.death_date) {
        if ev > d {
            return false;
        }
    }
    true
}

pub fn build(db: &Database, pol: &Policy, id: Id) -> Option<Lifeline> {
    let raw = db.persons.get(&id)?;
    if pol.active && (raw.status != Status::Published || pol.should_hide(raw)) {
        return None;
    }
    let (d, _) = pol.redact(raw);
    let ix = db.index();
    let birth = date::to_naive(&d.birth_date);
    let age_at = |ds: &str| -> Option<i32> {
        let b = birth?;
        let e = date::to_naive(ds)?;
        let y = date::years_between(b, e);
        Some(if e < b { -y } else { y })
    };
    let visible = |pid: Id| -> Option<Person> {
        let p = db.persons.get(&pid)?;
        if pol.active && (p.status != Status::Published || pol.should_hide(p)) {
            return None;
        }
        Some(pol.redact(p).0)
    };
    let mk = |kind: &str, ds: &str, place: &str| LifelineEvent {
        kind: kind.into(),
        date: ds.into(),
        place: place.into(),
        name: String::new(),
        description: String::new(),
        image: None,
        related: None,
        related_name: String::new(),
        role: String::new(),
        age: age_at(ds),
    };
    let mut ev = Vec::new();
    if !d.birth_date.is_empty() {
        let mut e = mk("birth", &d.birth_date, &d.birth_place);
        e.age = Some(0);
        ev.push(e);
    }
    for x in &d.events {
        if x.date.is_empty() {
            continue;
        }
        let mut e = mk("event", &x.date, &x.place);
        e.name = x.name.clone();
        e.description = x.description.clone();
        e.image = x.image;
        ev.push(e);
    }
    for uid in ix.unions_of(id) {
        let u = &db.unions[uid];
        let partner = u.partner_of(id);
        let pv = partner.and_then(visible);
        if partner.is_some() && pv.is_none() && pol.active {
            continue;
        }
        let a_l = pol.is_living(raw);
        let b_l = partner.and_then(|p| db.persons.get(&p)).map(|p| pol.is_living(p)).unwrap_or(false);
        let u = pol.redact_union(u, a_l, b_l);
        let name = match &pv {
            Some(p) => p.full_name(),
            None => format!("{} {}", u.partner_first_name, u.partner_last_name).trim().to_string(),
        };
        if !u.date.is_empty() {
            let mut e = mk("marriage", &u.date, &u.place);
            e.related = partner;
            e.related_name = name.clone();
            ev.push(e);
        }
        if !u.end_date.is_empty() && !u.end_cause.is_empty() && u.end_cause != end_cause::ONGOING {
            let mut e = mk("marriage_end", &u.end_date, "");
            e.related = partner;
            e.related_name = name;
            e.role = u.end_cause.clone();
            ev.push(e);
        }
    }
    for c in ix.children_of(id) {
        let Some(cp) = visible(*c) else { continue };
        if !cp.birth_date.is_empty() && within(&cp.birth_date, &d) {
            let mut e = mk("child_birth", &cp.birth_date, &cp.birth_place);
            e.related = Some(cp.id);
            e.related_name = cp.full_name();
            ev.push(e);
        }
        for g in ix.children_of(*c) {
            let Some(gp) = visible(*g) else { continue };
            if !gp.birth_date.is_empty() && within(&gp.birth_date, &d) {
                let mut e = mk("grandchild_birth", &gp.birth_date, &gp.birth_place);
                e.related = Some(gp.id);
                e.related_name = gp.full_name();
                ev.push(e);
            }
        }
    }
    for s in ix.siblings(db, id) {
        let Some(sp) = visible(s) else { continue };
        if !sp.birth_date.is_empty() && within(&sp.birth_date, &d) {
            let mut e = mk("sibling_birth", &sp.birth_date, &sp.birth_place);
            e.related = Some(s);
            e.related_name = sp.full_name();
            ev.push(e);
        }
        if !sp.death_date.is_empty() && within(&sp.death_date, &d) {
            let mut e = mk("sibling_death", &sp.death_date, &sp.death_place);
            e.related = Some(s);
            e.related_name = sp.full_name();
            ev.push(e);
        }
    }
    for (role, pid) in [("mother", raw.mother), ("father", raw.father)] {
        let Some(pid) = pid else { continue };
        let Some(pp) = visible(pid) else { continue };
        if !pp.death_date.is_empty() && within(&pp.death_date, &d) {
            let mut e = mk("parent_death", &pp.death_date, &pp.death_place);
            e.related = Some(pid);
            e.related_name = pp.full_name();
            e.role = role.into();
            ev.push(e);
        }
        for g in pp.parents() {
            let Some(gp) = visible(g) else { continue };
            if !gp.death_date.is_empty() && within(&gp.death_date, &d) {
                let mut e = mk("grandparent_death", &gp.death_date, &gp.death_place);
                e.related = Some(g);
                e.related_name = gp.full_name();
                ev.push(e);
            }
        }
    }
    if !d.death_date.is_empty() {
        ev.push(mk("death", &d.death_date, &d.death_place));
    } else if life::known_dead(&d) {
        let mut e = mk("death", "", &d.death_place);
        e.age = None;
        ev.push(e);
    } else if life::presumed_dead(raw, db.settings.max_age, pol.today) {
        let mut e = mk("presumed_death", "", "");
        e.age = None;
        ev.push(e);
    }
    ev.sort_by(|a, b| date::sort_key(&a.date).cmp(&date::sort_key(&b.date)));
    Some(Lifeline {
        person: id,
        name: d.full_name(),
        gender: d.gender,
        photo: if pol.photo_hidden(raw) { None } else { d.photo },
        years: life::years(&d),
        events: ev,
    })
}
