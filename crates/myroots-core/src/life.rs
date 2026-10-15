//! Life helpers: age, living / presumed death.

use crate::date;
use crate::model::Person;
use chrono::NaiveDate;

/// Execution context: time is always supplied by the platform layer
/// (the core never reads the clock, so it behaves the same in WASM and natively).
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    /// Unix time in milliseconds.
    pub now: i64,
    /// Today's local date.
    pub today: NaiveDate,
}

impl Ctx {
    pub fn new(now: i64, today: NaiveDate) -> Ctx {
        Ctx { now, today }
    }
}

/// Presumed death: no death date, but age >= max_age (0 = feature off).
/// Died, even if the date is unknown: marked as deceased, or a place or
/// cause of death is recorded.
pub fn known_dead(p: &Person) -> bool {
    !p.death_date.trim().is_empty() || p.deceased || !p.death_place.trim().is_empty() || !p.death_cause.trim().is_empty()
}

pub fn presumed_dead(p: &Person, max_age: u32, today: NaiveDate) -> bool {
    if max_age == 0 || known_dead(p) {
        return false;
    }
    match date::to_naive(&p.birth_date) {
        Some(b) => date::years_between(b, today) >= max_age as i32,
        None => false,
    }
}

pub fn is_alive(p: &Person, max_age: u32, today: NaiveDate) -> bool {
    !known_dead(p) && !presumed_dead(p, max_age, today)
}

/// Age in years (final age for the deceased; capped at max_age for presumed death).
pub fn age(p: &Person, max_age: u32, today: NaiveDate) -> Option<i32> {
    let birth = date::to_naive(&p.birth_date)?;
    if !p.death_date.trim().is_empty() {
        let end = date::to_naive(&p.death_date)?;
        return Some(date::years_between(birth, end));
    }
    if known_dead(p) {
        return None;
    }
    let a = date::years_between(birth, today);
    if max_age > 0 && a >= max_age as i32 {
        return Some(max_age as i32);
    }
    Some(a)
}

/// Years label of a person: like [`years_label`], "1900–?" when they died on an unknown date.
pub fn years(p: &Person) -> String {
    if p.death_date.trim().is_empty() && known_dead(p) {
        if let Some(b) = date::year(&p.birth_date) {
            return format!("{b}–?");
        }
    }
    years_label(&p.birth_date, &p.death_date)
}

/// "1900–1980", "*1900", "†1980" or "".
pub fn years_label(birth: &str, death: &str) -> String {
    match (date::year(birth), date::year(death)) {
        (Some(b), Some(d)) => format!("{b}–{d}"),
        (Some(b), None) => format!("*{b}"),
        (None, Some(d)) => format!("†{d}"),
        _ => String::new(),
    }
}
