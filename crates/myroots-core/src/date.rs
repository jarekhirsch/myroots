//! Dates with an uncertainty qualifier, stored in a single string field
//! (format kept from the WordPress plugin for compatibility):
//!
//! | value          | meaning            | GEDCOM        |
//! |----------------|--------------------|---------------|
//! | `1920`         | exact (year)       | `1920`        |
//! | `1920-05`      | exact (month)      | `MAY 1920`    |
//! | `1920-05-15`   | exact (day)        | `15 MAY 1920` |
//! | `~1920`        | about              | `ABT 1920`    |
//! | `<1920`        | before             | `BEF 1920`    |
//! | `>1850`        | after              | `AFT 1850`    |
//! | `1900..1910`   | between            | `BET … AND …` |

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Qualifier {
    Exact,
    Abt,
    Bef,
    Aft,
    Bet,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct ParsedDate {
    pub q: Qualifier,
    pub d1: String,
    pub d2: String,
}

fn is_digits(s: &str, n: usize) -> bool {
    s.len() == n && s.bytes().all(|b| b.is_ascii_digit())
}

/// A bare date (YYYY, YYYY-MM or YYYY-MM-DD) or "" when invalid.
pub fn bare(v: &str) -> String {
    let v = v.trim();
    let parts: Vec<&str> = v.split('-').collect();
    let ok = match parts.len() {
        1 => is_digits(parts[0], 4),
        2 => is_digits(parts[0], 4) && is_digits(parts[1], 2),
        3 => is_digits(parts[0], 4) && is_digits(parts[1], 2) && is_digits(parts[2], 2),
        _ => false,
    };
    if ok {
        v.to_string()
    } else {
        String::new()
    }
}

pub fn parse(v: &str) -> ParsedDate {
    let v = v.trim();
    if v.is_empty() {
        return ParsedDate { q: Qualifier::Exact, d1: String::new(), d2: String::new() };
    }
    if let Some(pos) = v.find("..") {
        let (a, b) = (&v[..pos], &v[pos + 2..]);
        return ParsedDate { q: Qualifier::Bet, d1: bare(a), d2: bare(b) };
    }
    let (q, rest) = match v.chars().next() {
        Some('~') => (Qualifier::Abt, &v[1..]),
        Some('<') => (Qualifier::Bef, &v[1..]),
        Some('>') => (Qualifier::Aft, &v[1..]),
        _ => (Qualifier::Exact, v),
    };
    ParsedDate { q, d1: bare(rest.trim_start()), d2: String::new() }
}

/// Normalizes a qualified date; returns "" for invalid input.
pub fn sanitize(v: &str) -> String {
    let p = parse(v);
    if p.q == Qualifier::Bet {
        if p.d1.is_empty() || p.d2.is_empty() {
            return if p.d1.is_empty() { p.d2 } else { p.d1 };
        }
        return format!("{}..{}", p.d1, p.d2);
    }
    if p.d1.is_empty() {
        return String::new();
    }
    let prefix = match p.q {
        Qualifier::Abt => "~",
        Qualifier::Bef => "<",
        Qualifier::Aft => ">",
        _ => "",
    };
    format!("{prefix}{}", p.d1)
}

/// The first bare date of the value (for computations and sorting).
pub fn core(v: &str) -> String {
    parse(v).d1
}

/// Year of the date or None.
pub fn year(v: &str) -> Option<i32> {
    let c = core(v);
    if c.len() >= 4 {
        c[..4].parse().ok()
    } else {
        None
    }
}

/// Full `YYYY-MM-DD` (missing month/day filled with 01) or "".
pub fn full(v: &str) -> String {
    let c = core(v);
    match c.len() {
        4 => format!("{c}-01-01"),
        7 => format!("{c}-01"),
        10 => c,
        _ => String::new(),
    }
}

pub fn to_naive(v: &str) -> Option<NaiveDate> {
    let f = full(v);
    if f.is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(&f, "%Y-%m-%d").ok().or_else(|| {
        // e.g. 31 in a 30-day month: clamp to a valid day
        let y: i32 = f[..4].parse().ok()?;
        let m: u32 = f[5..7].parse().ok()?;
        NaiveDate::from_ymd_opt(y, m.clamp(1, 12), 1)
    })
}

/// Whole years between two dates (like PHP DateTime::diff()->y, always >= 0).
pub fn years_between(a: NaiveDate, b: NaiveDate) -> i32 {
    let (from, to) = if a <= b { (a, b) } else { (b, a) };
    let mut y = to.year() - from.year();
    if (to.month(), to.day()) < (from.month(), from.day()) {
        y -= 1;
    }
    y.max(0)
}

/// Sort key: dates without a value go last.
pub fn sort_key(v: &str) -> String {
    let f = full(v);
    if f.is_empty() {
        "9999-99-99".into()
    } else {
        f
    }
}

/// Exact (full) date only — used for anniversaries.
pub fn exact_day(v: &str) -> Option<NaiveDate> {
    let p = parse(v);
    if p.q != Qualifier::Exact || p.d1.len() != 10 {
        return None;
    }
    NaiveDate::parse_from_str(&p.d1, "%Y-%m-%d").ok()
}

pub fn parse_iso_day(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()
}

/* ------------------------------------------------------------------ */
/*  GEDCOM                                                              */
/* ------------------------------------------------------------------ */

const MONTHS: [&str; 12] = ["JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC"];

fn month_num(s: &str) -> Option<u32> {
    let s = s.trim_end_matches('.').to_uppercase();
    let k: String = s.chars().take(3).collect();
    if s.chars().count() < 3 {
        return None;
    }
    if let Some(i) = MONTHS.iter().position(|m| *m == k) {
        return Some(i as u32 + 1);
    }
    // German and Dutch abbreviations seen in GEDCOM files from local programs
    Some(match k.as_str() {
        "MÄR" | "MRZ" | "MAA" => 3,
        "MAI" | "MEI" => 5,
        "OKT" => 10,
        "DEZ" => 12,
        _ => return None,
    })
}

/// A year: 3–4 digits, or a dual year "1700/01" (the later, New Style year).
fn year_of(s: &str) -> Option<i32> {
    let (y, dual) = match s.split_once('/') {
        Some((a, b)) if !b.is_empty() && b.len() <= 2 && b.bytes().all(|c| c.is_ascii_digit()) => (a, true),
        Some(_) => return None,
        None => (s, false),
    };
    if !(y.len() == 3 || y.len() == 4) || !y.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let y: i32 = y.parse().ok()?;
    Some(if dual { y + 1 } else { y })
}

fn ymd(y: i32, m: u32, d: u32) -> String {
    if d >= 1 && NaiveDate::from_ymd_opt(y, m, d).is_some() {
        format!("{y:04}-{m:02}-{d:02}")
    } else {
        format!("{y:04}-{m:02}")
    }
}

/// Bare GEDCOM date (DD MON YYYY / MON YYYY / YYYY) → YYYY[-MM[-DD]].
/// Also accepts calendar escapes, dual years, "1.05.1920", "1920-05-01" and
/// "May 1, 1920", which other programs write despite the standard.
fn gedcom_bare(raw: &str) -> String {
    let mut s = raw.trim().to_uppercase().replace(',', " ");
    // calendar escapes (5.5.1 "@#DJULIAN@", GEDCOM 7 "JULIAN"): the date is kept as written
    while let Some(a) = s.find("@#") {
        match s[a + 2..].find('@') {
            Some(b) => s.replace_range(a..a + 3 + b, " "),
            None => break,
        }
    }
    for cal in ["GREGORIAN ", "JULIAN "] {
        if let Some(r) = s.trim_start().strip_prefix(cal) {
            s = r.to_string();
        }
    }
    if s.contains("B.C") || s.ends_with(" BC") || s.ends_with("BCE") {
        return String::new();
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    let num = |x: &str| -> Option<u32> { if !x.is_empty() && x.len() <= 2 && x.bytes().all(|b| b.is_ascii_digit()) { x.parse().ok() } else { None } };
    match parts.as_slice() {
        [d, m, y] if num(d).is_some() => match (month_num(m), year_of(y)) {
            (Some(mn), Some(y)) => ymd(y, mn, num(d).unwrap()),
            _ => String::new(),
        },
        // "MAY 1 1920" (US style)
        [m, d, y] if num(d.trim_end_matches('.')).is_some() => match (month_num(m), year_of(y)) {
            (Some(mn), Some(y)) => ymd(y, mn, num(d.trim_end_matches('.')).unwrap()),
            _ => String::new(),
        },
        [m, y] => match (month_num(m), year_of(y)) {
            (Some(mn), Some(y)) => format!("{y:04}-{mn:02}"),
            _ => String::new(),
        },
        [one] => {
            if let Some(y) = year_of(one) {
                return format!("{y:04}");
            }
            // 1920-05-01 / 1920-05
            let iso: Vec<&str> = one.split('-').collect();
            if let [y, m, rest @ ..] = iso.as_slice() {
                if let (Some(y), Some(m)) = (year_of(y), num(m).filter(|m| (1..=12).contains(m))) {
                    return match rest {
                        [d] => num(d).map(|d| ymd(y, m, d)).unwrap_or_default(),
                        [] => format!("{y:04}-{m:02}"),
                        _ => String::new(),
                    };
                }
            }
            // 1.05.1920 / 05.1920
            let dots: Vec<&str> = one.split('.').collect();
            match dots.as_slice() {
                [d, m, y] => match (num(d), num(m).filter(|m| (1..=12).contains(m)), year_of(y)) {
                    (Some(d), Some(m), Some(y)) => ymd(y, m, d),
                    _ => String::new(),
                },
                [m, y] => match (num(m).filter(|m| (1..=12).contains(m)), year_of(y)) {
                    (Some(m), Some(y)) => format!("{y:04}-{m:02}"),
                    _ => String::new(),
                },
                _ => String::new(),
            }
        }
        _ => String::new(),
    }
}

/// GEDCOM DATE value → internal qualified date.
pub fn from_gedcom(raw: &str) -> String {
    // date phrases: "INT 1900 (from the gravestone)", "(about the time of the war)"
    let mut raw = raw.trim().to_uppercase();
    while let (Some(a), Some(b)) = (raw.find('('), raw.rfind(')')) {
        if b <= a {
            break;
        }
        raw.replace_range(a..=b, " ");
    }
    let raw = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if raw.is_empty() {
        return String::new();
    }
    let range = |a: &str, b: &str| -> String {
        let (a, b) = (gedcom_bare(a), gedcom_bare(b));
        match (a.is_empty(), b.is_empty()) {
            (false, false) => format!("{a}..{b}"),
            (false, true) => a,
            _ => b,
        }
    };
    let (first, rest) = raw.split_once(' ').unwrap_or((raw.as_str(), ""));
    match first {
        "BET" | "BET." | "BETWEEN" => {
            if let Some((a, b)) = rest.split_once(" AND ").or_else(|| rest.split_once(" - ")) {
                return range(a, b);
            }
            let b = gedcom_bare(rest);
            return if b.is_empty() { b } else { format!(">{b}") };
        }
        "FROM" => {
            if let Some((a, b)) = rest.split_once(" TO ") {
                return range(a, b);
            }
            return gedcom_bare(rest);
        }
        _ => {}
    }
    let pre = match first {
        "ABT" | "ABT." | "ABOUT" | "CA" | "CA." | "C." | "CIRCA" | "APPROX" | "APPROX." | "CAL" | "CAL." | "EST" | "EST." | "CIR" | "INT" => Some("~"),
        "BEF" | "BEF." | "BEFORE" => Some("<"),
        "AFT" | "AFT." | "AFTER" => Some(">"),
        "TO" => Some(""),
        _ => None,
    };
    if let Some(pre) = pre {
        let b = gedcom_bare(rest);
        return if b.is_empty() { b } else { format!("{pre}{b}") };
    }
    // 1900-1910 (both full years)
    if let Some((a, b)) = raw.split_once('-') {
        if a.len() == 4 && b.len() == 4 && a.bytes().chain(b.bytes()).all(|c| c.is_ascii_digit()) {
            return range(a, b);
        }
    }
    gedcom_bare(&raw)
}

fn gedcom_bare_out(d: &str) -> String {
    match d.len() {
        10 => {
            let m: usize = d[5..7].parse().unwrap_or(1);
            let day: u32 = d[8..10].parse().unwrap_or(1);
            format!("{} {} {}", day, MONTHS[(m.clamp(1, 12)) - 1], &d[..4])
        }
        7 => {
            let m: usize = d[5..7].parse().unwrap_or(1);
            format!("{} {}", MONTHS[(m.clamp(1, 12)) - 1], &d[..4])
        }
        4 => d.to_string(),
        _ => String::new(),
    }
}

/// Internal qualified date → GEDCOM DATE value.
pub fn to_gedcom(v: &str) -> String {
    let p = parse(v);
    if p.q == Qualifier::Bet {
        let a = gedcom_bare_out(&p.d1);
        let b = gedcom_bare_out(&p.d2);
        return match (a.is_empty(), b.is_empty()) {
            (false, false) => format!("BET {a} AND {b}"),
            (false, true) => a,
            _ => b,
        };
    }
    let b = gedcom_bare_out(&p.d1);
    if b.is_empty() {
        return String::new();
    }
    let pre = match p.q {
        Qualifier::Abt => "ABT ",
        Qualifier::Bef => "BEF ",
        Qualifier::Aft => "AFT ",
        _ => "",
    };
    format!("{pre}{b}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualified() {
        assert_eq!(sanitize("~1920"), "~1920");
        assert_eq!(sanitize(" < 1920-05 "), "<1920-05");
        assert_eq!(sanitize("1900..1910"), "1900..1910");
        assert_eq!(sanitize("1900.."), "1900");
        assert_eq!(sanitize("abc"), "");
        assert_eq!(year("~1920-05-01"), Some(1920));
        assert_eq!(full("1909"), "1909-01-01");
        assert_eq!(full("1909-03"), "1909-03-01");
    }

    #[test]
    fn gedcom_roundtrip() {
        assert_eq!(from_gedcom("15 MAY 1920"), "1920-05-15");
        assert_eq!(from_gedcom("abt 1920"), "~1920");
        assert_eq!(from_gedcom("BET 1900 AND 1910"), "1900..1910");
        assert_eq!(from_gedcom("FROM MAR 1900 TO 1910"), "1900-03..1910");
        assert_eq!(from_gedcom("BEF 3 JAN 1800"), "<1800-01-03");
        assert_eq!(to_gedcom("1920-05-15"), "15 MAY 1920");
        assert_eq!(to_gedcom("~1920"), "ABT 1920");
        assert_eq!(to_gedcom("1900..1910-02"), "BET 1900 AND FEB 1910");
    }

    #[test]
    fn gedcom_dates_from_other_programs() {
        assert_eq!(from_gedcom("@#DJULIAN@ 5 FEB 1700"), "1700-02-05");
        assert_eq!(from_gedcom("JULIAN 5 FEB 1700"), "1700-02-05");
        assert_eq!(from_gedcom("12 FEB 1699/00"), "1700-02-12");
        assert_eq!(from_gedcom("INT 1900 (gravestone)"), "~1900");
        assert_eq!(from_gedcom("(during the war)"), "");
        assert_eq!(from_gedcom("Abt. 1850"), "~1850");
        assert_eq!(from_gedcom("circa 1850"), "~1850");
        assert_eq!(from_gedcom("Before 1850"), "<1850");
        assert_eq!(from_gedcom("between 1850 and 1860"), "1850..1860");
        assert_eq!(from_gedcom("1850-1860"), "1850..1860");
        assert_eq!(from_gedcom("1920-05-01"), "1920-05-01");
        assert_eq!(from_gedcom("1.05.1920"), "1920-05-01");
        assert_eq!(from_gedcom("May 1, 1920"), "1920-05-01");
        assert_eq!(from_gedcom("1 MAI 1920"), "1920-05-01");
        assert_eq!(from_gedcom("31 FEB 1920"), "1920-02");
        assert_eq!(from_gedcom("100 B.C."), "");
        assert_eq!(from_gedcom("1 MAŁ 1920"), "");
        assert_eq!(from_gedcom("BET 1900"), ">1900");
    }

    #[test]
    fn diff() {
        let a = NaiveDate::from_ymd_opt(1950, 6, 10).unwrap();
        let b = NaiveDate::from_ymd_opt(2020, 6, 9).unwrap();
        assert_eq!(years_between(a, b), 69);
    }
}
