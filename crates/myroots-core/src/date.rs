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

/* ---------- other calendars (GEDCOM: JULIAN, HEBREW, FRENCH_R) ---------- */
// Day numbers are "fixed" days (RD) as in Reingold & Dershowitz,
// "Calendrical Calculations": day 1 = 1 January 1 (Gregorian), the same
// count as chrono's `num_days_from_ce`.

#[derive(Clone, Copy, PartialEq, Debug)]
enum Cal {
    Gregorian,
    Julian,
    Hebrew,
    French,
}

fn from_fixed(rd: i64) -> Option<NaiveDate> {
    NaiveDate::from_num_days_from_ce_opt(i32::try_from(rd).ok()?)
}

fn fixed_from_julian(y: i64, m: i64, d: i64) -> i64 {
    let leap = y.rem_euclid(4) == 0;
    -2 + 365 * (y - 1) + (y - 1).div_euclid(4) + (367 * m - 362).div_euclid(12) + if m <= 2 { 0 } else if leap { -1 } else { -2 } + d
}

const HEBREW_EPOCH: i64 = -1_373_427;
fn hebrew_leap(y: i64) -> bool {
    (7 * y + 1).rem_euclid(19) < 7
}
fn hebrew_elapsed(y: i64) -> i64 {
    let months = (235 * y - 234).div_euclid(19);
    let parts = 12084 + 13753 * months;
    let day = 29 * months + parts.div_euclid(25920);
    if (3 * (day + 1)).rem_euclid(7) < 3 {
        day + 1
    } else {
        day
    }
}
fn hebrew_new_year(y: i64) -> i64 {
    let (a, b, c) = (hebrew_elapsed(y - 1), hebrew_elapsed(y), hebrew_elapsed(y + 1));
    let delay = if c - b == 356 { 2 } else if b - a == 382 { 1 } else { 0 };
    HEBREW_EPOCH + b + delay
}
/// Months counted from Nisan (1) to Adar (12) and Adar II (13).
fn hebrew_month_days(m: i64, y: i64) -> i64 {
    let len = hebrew_new_year(y + 1) - hebrew_new_year(y);
    let short = matches!(m, 2 | 4 | 6 | 10 | 13) || (m == 12 && !hebrew_leap(y)) || (m == 8 && len % 10 != 5) || (m == 9 && len % 10 == 3);
    if short {
        29
    } else {
        30
    }
}
fn fixed_from_hebrew(y: i64, m: i64, d: i64) -> i64 {
    let last = if hebrew_leap(y) { 13 } else { 12 };
    let mut days = hebrew_new_year(y) + d - 1;
    if m < 7 {
        days += (7..=last).map(|x| hebrew_month_days(x, y)).sum::<i64>();
        days += (1..m).map(|x| hebrew_month_days(x, y)).sum::<i64>();
    } else {
        days += (7..m).map(|x| hebrew_month_days(x, y)).sum::<i64>();
    }
    days
}
fn hebrew_month(s: &str) -> Option<i64> {
    Some(match s {
        "NSN" => 1,
        "IYR" => 2,
        "SVN" => 3,
        "TMZ" => 4,
        "AAV" => 5,
        "ELL" => 6,
        "TSH" => 7,
        "CSH" => 8,
        "KSL" => 9,
        "TVT" => 10,
        "SHV" => 11,
        "ADR" => 12,
        "ADS" => 13,
        _ => return None,
    })
}

/// French Republican calendar: year I began on 22 September 1792; the
/// sextile (leap) years of its short use were III, VII and XI.
fn fixed_from_french(y: i64, m: i64, d: i64) -> i64 {
    let start = 654_415; // 22 September 1792
    let before = 365 * (y - 1) + [3, 7, 11, 15].iter().filter(|&&s| s < y).count() as i64;
    start + before + (m - 1) * 30 + d - 1
}
fn french_month(s: &str) -> Option<i64> {
    ["VEND", "BRUM", "FRIM", "NIVO", "PLUV", "VENT", "GERM", "FLOR", "PRAI", "MESS", "THER", "FRUC", "COMP"].iter().position(|m| *m == s).map(|i| i as i64 + 1)
}

/// A date of another calendar → Gregorian YYYY[-MM[-DD]]. A day is converted
/// exactly; a month or a year only approximately (its first day).
fn other_calendar(cal: Cal, parts: &[&str]) -> String {
    let month = |m: &str| -> Option<i64> {
        match cal {
            Cal::Hebrew => hebrew_month(m),
            Cal::French => french_month(m),
            _ => month_num(m).map(|x| x as i64),
        }
    };
    let fixed = |y: i64, m: i64, d: i64| -> i64 {
        match cal {
            Cal::Hebrew => fixed_from_hebrew(y, m, d),
            Cal::French => fixed_from_french(y, m, d),
            _ => fixed_from_julian(y, m, d),
        }
    };
    // a dual year ("1699/00", the year still began in March) is the later one
    let year = |y: &str| -> Option<i64> {
        let (first, dual) = match y.split_once('/') {
            Some((a, _)) => (a, 1),
            None => (y, 0),
        };
        (!first.is_empty() && first.len() <= 4 && first.bytes().all(|b| b.is_ascii_digit())).then(|| first.parse::<i64>().ok().map(|v| v + dual)).flatten()
    };
    let (y, m, d) = match parts {
        [d, m, y] => match (d.parse::<i64>().ok(), month(m), year(y)) {
            (Some(d), Some(m), Some(y)) if (1..=31).contains(&d) => (y, Some(m), Some(d)),
            _ => return String::new(),
        },
        [m, y] => match (month(m), year(y)) {
            (Some(m), Some(y)) => (y, Some(m), None),
            _ => return String::new(),
        },
        [y] => match year(y) {
            Some(y) => (y, None, None),
            None => return String::new(),
        },
        _ => return String::new(),
    };
    if cal == Cal::Hebrew && (y < 3800 || y > 6000) || cal == Cal::French && !(1..=20).contains(&y) {
        return String::new();
    }
    match (m, d) {
        (Some(m), Some(d)) => from_fixed(fixed(y, m, d)).map(|g| g.format("%Y-%m-%d").to_string()).unwrap_or_default(),
        (Some(m), None) => from_fixed(fixed(y, m, 1)).map(|g| if cal == Cal::Julian { format!("{y:04}-{m:02}") } else { g.format("%Y-%m").to_string() }).unwrap_or_default(),
        _ => match cal {
            Cal::Julian => format!("{y:04}"),
            // a Hebrew or Republican year spans two Gregorian years: the one it begins in
            Cal::Hebrew => from_fixed(fixed_from_hebrew(y, 7, 1)).map(|g| g.year().to_string()).unwrap_or_default(),
            _ => from_fixed(fixed_from_french(y, 1, 1)).map(|g| g.year().to_string()).unwrap_or_default(),
        },
    }
}

/// Bare GEDCOM date (DD MON YYYY / MON YYYY / YYYY) → YYYY[-MM[-DD]].
/// Also accepts calendar escapes, dual years, "1.05.1920", "1920-05-01" and
/// "May 1, 1920", which other programs write despite the standard. Julian,
/// Hebrew and French Republican dates are converted to the Gregorian calendar.
fn gedcom_bare(raw: &str) -> String {
    let mut s = raw.trim().to_uppercase().replace(',', " ");
    let mut cal = Cal::Gregorian;
    // calendar escapes (5.5.1 "@#DJULIAN@", GEDCOM 7 "JULIAN")
    while let Some(a) = s.find("@#") {
        match s[a + 2..].find('@') {
            Some(b) => {
                let esc = s[a + 2..a + 2 + b].to_string();
                cal = match esc.trim_start_matches('D').trim() {
                    "JULIAN" => Cal::Julian,
                    "HEBREW" => Cal::Hebrew,
                    "FRENCH R" => Cal::French,
                    _ => cal,
                };
                s.replace_range(a..a + 3 + b, " ");
            }
            None => break,
        }
    }
    for (name, c) in [("GREGORIAN ", Cal::Gregorian), ("JULIAN ", Cal::Julian), ("HEBREW ", Cal::Hebrew), ("FRENCH_R ", Cal::French)] {
        if let Some(r) = s.trim_start().strip_prefix(name) {
            s = r.to_string();
            cal = c;
        }
    }
    if cal != Cal::Gregorian {
        let parts: Vec<&str> = s.split_whitespace().collect();
        return other_calendar(cal, &parts);
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
        assert_eq!(from_gedcom("@#DJULIAN@ 5 FEB 1700"), "1700-02-15");
        assert_eq!(from_gedcom("JULIAN 5 FEB 1700"), "1700-02-15");
        assert_eq!(from_gedcom("@#DJULIAN@ FEB 1700"), "1700-02");
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
    fn other_calendars() {
        // the Gregorian reform: 4 October 1582 (Julian) was followed by 15 October
        assert_eq!(from_gedcom("@#DJULIAN@ 5 OCT 1582"), "1582-10-15");
        assert_eq!(from_gedcom("JULIAN 1 JAN 1900"), "1900-01-13");
        assert_eq!(from_gedcom("ABT @#DJULIAN@ 1890"), "~1890");
        // Rosh Hashana 5780 = 30 September 2019; 15 Nisan 5779 = 20 April 2019
        assert_eq!(from_gedcom("@#DHEBREW@ 1 TSH 5780"), "2019-09-30");
        assert_eq!(from_gedcom("HEBREW 15 NSN 5779"), "2019-04-20");
        // 14 Adar II 5776 (a leap year) = 24 March 2016
        assert_eq!(from_gedcom("@#DHEBREW@ 14 ADS 5776"), "2016-03-24");
        assert_eq!(from_gedcom("@#DHEBREW@ 5680"), "1919");
        // 18 Brumaire VIII = 9 November 1799; 1 Vendémiaire XIII = 23 September 1804
        assert_eq!(from_gedcom("@#DFRENCH R@ 18 BRUM 8"), "1799-11-09");
        assert_eq!(from_gedcom("FRENCH_R 1 VEND 13"), "1804-09-23");
        assert_eq!(from_gedcom("@#DHEBREW@ 1 XYZ 5780"), "");
    }

    #[test]
    fn diff() {
        let a = NaiveDate::from_ymd_opt(1950, 6, 10).unwrap();
        let b = NaiveDate::from_ymd_opt(2020, 6, 9).unwrap();
        assert_eq!(years_between(a, b), 69);
    }
}
