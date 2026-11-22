//! Privacy for the published family website.
//!
//! The owner of the tree always sees everything inside the app. The policy
//! below is applied when the family website is exported:
//!
//! * `full`          – selected sections are not published at all,
//! * `hide_living`   – living people are reduced to anonymous placeholders
//!                     (the tree keeps its shape, no data leaks),
//! * `censor_living` – living people are visible, selected fields are censored.

use crate::life;
use crate::model::*;
use chrono::NaiveDate;

pub const LEVELS: &[&str] = &["full", "hide_living", "censor_living"];
pub const SECTIONS: &[&str] = &["profile", "list", "tree", "map", "stats", "gallery"];
pub const FIELDS: &[&str] = &["dates", "places", "bio_events", "photos_sensitive"];

#[derive(Clone, Debug)]
pub struct Policy {
    pub active: bool,
    pub level: String,
    pub sections: Vec<String>,
    pub fields: Vec<String>,
    pub max_age: u32,
    pub today: NaiveDate,
}

impl Policy {
    /// No restrictions (the owner).
    pub fn owner(max_age: u32, today: NaiveDate) -> Policy {
        Policy { active: false, level: "full".into(), sections: vec![], fields: vec![], max_age, today }
    }

    /// Policy for the public website, from the settings.
    pub fn public(s: &Settings, today: NaiveDate) -> Policy {
        let level = if LEVELS.contains(&s.privacy.level.as_str()) { s.privacy.level.clone() } else { "full".into() };
        Policy {
            active: s.privacy.enabled,
            level,
            sections: s.privacy.sections.iter().filter(|x| SECTIONS.contains(&x.as_str())).cloned().collect(),
            fields: s.privacy.fields.iter().filter(|x| FIELDS.contains(&x.as_str())).cloned().collect(),
            max_age: s.max_age,
            today,
        }
    }

    pub fn is_living(&self, p: &Person) -> bool {
        life::is_alive(p, self.max_age, self.today)
    }

    /// Is the whole section locked (level "full")?
    pub fn blocks_section(&self, section: &str) -> bool {
        self.active && self.level == "full" && self.sections.iter().any(|s| s == section)
    }

    /// Should this person be reduced to an anonymous placeholder?
    pub fn should_hide(&self, p: &Person) -> bool {
        if !self.active {
            return false;
        }
        match self.level.as_str() {
            "hide_living" => self.is_living(p),
            // "full" with both people-revealing sections locked: keep the structure only.
            "full" => self.blocks_section("profile") && self.blocks_section("list"),
            _ => false,
        }
    }

    pub fn should_censor(&self) -> bool {
        self.active && self.level == "censor_living"
    }

    fn censors(&self, field: &str) -> bool {
        self.fields.iter().any(|f| f == field)
    }

    pub fn photo_hidden(&self, p: &Person) -> bool {
        self.should_censor() && self.is_living(p) && self.censors("photos_sensitive")
    }

    /// The kinship calculator reveals names and the family structure.
    pub fn kinship_allowed(&self) -> bool {
        !self.active || self.level == "censor_living"
    }

    /// Copy of the person with censored fields (when `censor_living` applies).
    /// Returns `(person, censored)`.
    /// The person as this policy shows it, without a copy when nothing is
    /// censored (lists of a whole big tree call this for every person).
    pub fn view<'a>(&self, p: &'a Person) -> std::borrow::Cow<'a, Person> {
        if !self.should_censor() || !self.is_living(p) {
            std::borrow::Cow::Borrowed(p)
        } else {
            std::borrow::Cow::Owned(self.redact(p).0)
        }
    }

    pub fn redact(&self, p: &Person) -> (Person, bool) {
        if !self.should_censor() || !self.is_living(p) {
            return (p.clone(), false);
        }
        let mut d = p.clone();
        if self.censors("dates") {
            d.birth_date = year_only(&d.birth_date);
            d.death_date = year_only(&d.death_date);
            for e in &mut d.events {
                e.date = year_only(&e.date);
            }
        }
        if self.censors("places") {
            d.birth_place.clear();
            d.death_place.clear();
            d.residence.clear();
            d.grave = Grave::default();
            for e in &mut d.events {
                e.place.clear();
            }
        }
        if self.censors("bio_events") {
            d.biography.clear();
            d.notes.clear();
            d.research.clear();
            d.events.clear();
        }
        if self.censors("photos_sensitive") {
            d.gallery.clear();
            d.aging.clear();
            d.recordings.clear();
            d.aging_audio = None;
            d.photo = None;
            d.grave.photo = None;
            d.height.clear();
            d.eye_color.clear();
            d.blood_type.clear();
            d.faith.clear();
        }
        (d, true)
    }

    /// Union details are censored when either partner is a living, censored person.
    pub fn redact_union(&self, u: &Union, a_living: bool, b_living: bool) -> Union {
        let mut u = u.clone();
        if self.should_censor() && (a_living || b_living) {
            if self.censors("dates") {
                u.date = year_only(&u.date);
                u.end_date = year_only(&u.end_date);
            }
            if self.censors("places") {
                u.place.clear();
            }
        }
        u
    }
}

/// "1985-03-12" → "1985"; "~1985" → "1985"; "" → "".
pub fn year_only(v: &str) -> String {
    crate::date::year(v).map(|y| format!("{y:04}")).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn living() -> Person {
        Person { id: 1, first_name: "Ala".into(), birth_date: "1990-05-03".into(), birth_place: "Kraków".into(), faith: "x".into(), ..Default::default() }
    }

    #[test]
    fn censor() {
        let mut s = Settings::default();
        s.privacy.enabled = true;
        s.privacy.level = "censor_living".into();
        let pol = Policy::public(&s, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let (r, c) = pol.redact(&living());
        assert!(c);
        assert_eq!(r.birth_date, "1990");
        assert_eq!(r.birth_place, "");
        assert_eq!(r.faith, "");
        let mut dead = living();
        dead.death_date = "2020".into();
        assert!(!pol.redact(&dead).1);
    }

    #[test]
    fn hide() {
        let mut s = Settings::default();
        s.privacy.level = "hide_living".into();
        let pol = Policy::public(&s, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert!(pol.should_hide(&living()));
        s.privacy.enabled = false;
        let pol = Policy::public(&s, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert!(!pol.should_hide(&living()));
    }
}
