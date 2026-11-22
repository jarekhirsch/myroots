//! GEDCOM 5.5.1 import and export.
//!
//! Compatible with the files produced by the WordPress plugin (including the
//! MyRoots extensions `_CLAN`, `_ATTR`, `_MARNM`, `_PET`, `_FREL/_MREL`) and with
//! Ancestry, MyHeritage, Gramps, MacFamilyTree and others. In addition to the
//! plugin, sources (SOUR records + citations), place coordinates (MAP) and the
//! common standard events (BAPM, BURI, RESI …) are imported.

use crate::date;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::life::Ctx;
use crate::model::*;
use crate::ops;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

/* ------------------------------------------------------------------ */
/*  Parser                                                              */
/* ------------------------------------------------------------------ */

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub xref: Option<String>,
    pub tag: String,
    pub value: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn child(&self, tag: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.tag == tag)
    }
    pub fn val(&self, tag: &str) -> String {
        self.child(tag).map(|c| c.value.trim().to_string()).unwrap_or_default()
    }
    pub fn all<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.children.iter().filter(move |c| c.tag == tag)
    }
}

/// Decodes the raw bytes of a GEDCOM file to text (see [`crate::charsets`]).
pub fn decode(raw: &[u8]) -> String {
    crate::charsets::decode(raw)
}

/// A GEDCOM file, or a zipped one (GEDZIP `.gdz` of GEDCOM 7, or a .zip with a .ged inside).
pub fn decode_file(raw: &[u8]) -> Result<String> {
    if !raw.starts_with(b"PK\x03\x04") {
        return Ok(decode(raw));
    }
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(raw)).map_err(|_| Error::new("gedcom_zip", "the archive cannot be read"))?;
    let names: Vec<String> = z.file_names().filter_map(|s| s.ok().map(|s| s.to_string())).collect();
    let pick = names
        .iter()
        .find(|n| n.eq_ignore_ascii_case("gedcom.ged"))
        .or_else(|| names.iter().find(|n| n.to_lowercase().ends_with(".ged") && !n.starts_with("__MACOSX")))
        .ok_or_else(|| Error::new("gedcom_no_people", "the archive does not contain a GEDCOM file"))?
        .clone();
    let mut f = z.by_name(&pick).map_err(|_| Error::new("gedcom_zip", "the archive cannot be read"))?;
    let buf = crate::archive::read_limited(&mut f, 512 << 20).map_err(|_| Error::new("gedcom_zip", "the archive cannot be read"))?;
    drop(f);
    Ok(decode(&buf))
}

pub fn parse(text: &str) -> Vec<Node> {
    struct Flat {
        node: Node,
        level: usize,
        parent: Option<usize>,
        kids: Vec<usize>,
    }
    let mut flat: Vec<Flat> = Vec::new();
    let mut parent_at: Vec<Option<usize>> = Vec::new();
    for line in text.split(['\n', '\r']) {
        let line = line.trim_start_matches('\u{feff}');
        if line.trim().is_empty() {
            continue;
        }
        let t = line.trim_start();
        let mut it = t.splitn(2, ' ');
        let Some(level) = it.next().and_then(|l| l.parse::<usize>().ok()) else { continue };
        let rest = it.next().unwrap_or("").trim_start();
        let (xref, rest) = if rest.starts_with('@') {
            match rest[1..].find('@') {
                Some(end) => (Some(rest[1..end + 1].to_string()), rest[end + 2..].trim_start()),
                None => (None, rest),
            }
        } else {
            (None, rest)
        };
        let (tag, value) = match rest.find(' ') {
            Some(p) => (&rest[..p], &rest[p + 1..]),
            None => (rest, ""),
        };
        if tag.is_empty() {
            continue;
        }
        if tag == "CONC" || tag == "CONT" {
            if level > 0 {
                if let Some(Some(pi)) = parent_at.get(level - 1) {
                    let v = &mut flat[*pi].node.value;
                    if tag == "CONT" {
                        v.push('\n');
                    }
                    v.push_str(&value.replace("@@", "@"));
                }
            }
            continue;
        }
        // GEDCOM 7: shared notes are SNOTE records/pointers; "@@" stands for "@" in text
        let tag = if tag == "SNOTE" { "NOTE" } else { tag };
        let value = if value.contains("@@") { value.replace("@@", "@") } else { value.to_string() };
        let idx = flat.len();
        let parent = if level > 0 { parent_at.get(level - 1).copied().flatten() } else { None };
        flat.push(Flat { node: Node { xref, tag: tag.to_string(), value, children: vec![] }, level, parent, kids: vec![] });
        if let Some(p) = parent {
            flat[p].kids.push(idx);
        }
        if parent_at.len() <= level {
            parent_at.resize(level + 1, None);
        }
        parent_at[level] = Some(idx);
        parent_at.truncate(level + 1);
    }
    let _ = flat.iter().map(|f| f.level).max();
    // Build nested tree bottom-up.
    let mut built: Vec<Option<Node>> = vec![None; flat.len()];
    for i in (0..flat.len()).rev() {
        let mut n = std::mem::take(&mut flat[i].node);
        n.children = flat[i].kids.iter().map(|k| built[*k].take().unwrap_or_default()).collect();
        built[i] = Some(n);
    }
    flat.iter().enumerate().filter(|(_, f)| f.parent.is_none()).filter_map(|(i, _)| built[i].take()).collect()
}

fn xref_of(v: &str) -> String {
    v.trim().trim_matches('@').to_string()
}

/* ------------------------------------------------------------------ */
/*  Import                                                              */
/* ------------------------------------------------------------------ */

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ImportOptions {
    /// update | skip | new
    pub mode: String,
    pub dry_run: bool,
    /// Translated names for standard events (BAPM → "Chrzest" …), from the UI.
    pub event_names: HashMap<String, String>,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct ImportReport {
    pub dry_run: bool,
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub families: usize,
    pub pets_created: usize,
    pub pets_skipped: usize,
    pub sources: usize,
    pub places: usize,
    /// (code, detail) — e.g. ("indi_no_name", "@I12@")
    pub warnings: Vec<(String, String)>,
}

/// Standard individual events imported as life events.
const STD_EVENTS: &[(&str, &str)] = &[
    ("BAPM", "Baptism"),
    ("CHR", "Christening"),
    ("CONF", "Confirmation"),
    ("FCOM", "First communion"),
    ("BURI", "Burial"),
    ("CREM", "Cremation"),
    ("ADOP", "Adoption"),
    ("BARM", "Bar mitzvah"),
    ("BASM", "Bat mitzvah"),
    ("GRAD", "Graduation"),
    ("RETI", "Retirement"),
    ("EMIG", "Emigration"),
    ("IMMI", "Immigration"),
    ("NATU", "Naturalization"),
    ("CENS", "Census"),
    ("RESI", "Residence"),
    ("ORDN", "Ordination"),
    ("PROB", "Probate"),
    ("WILL", "Will"),
    ("_MILT", "Military service"),
    ("_MILI", "Military service"),
    ("CHRA", "Adult christening"),
    ("BLES", "Blessing"),
    ("TITL", "Title"),
    ("NATI", "Nationality"),
    ("DSCR", "Physical description"),
    ("PROP", "Property"),
];

/// Family events other than marriage and divorce, kept as life events of both partners.
const FAM_EVENTS: &[(&str, &str)] = &[
    ("ENGA", "Engagement"),
    ("MARB", "Marriage banns"),
    ("MARC", "Marriage contract"),
    ("MARL", "Marriage license"),
    ("MARS", "Marriage settlement"),
    ("DIVF", "Divorce filed"),
];

struct Ctx2<'a> {
    notes: HashMap<String, String>,
    /// source xref (or "inline:<title>") → source id
    sources: HashMap<String, Id>,
    opts: &'a ImportOptions,
    /// The file comes from MyRoots (the WordPress plugin or this app): `_MARNM`
    /// and `NPFX` hold the maiden name there, elsewhere `_MARNM` is the married name.
    myroots: bool,
}

/// Key of a source reference: a pointer, or the text of an inline source.
fn source_key(s: &Node) -> Option<String> {
    let v = s.value.trim();
    if v.starts_with('@') {
        let x = xref_of(v);
        return (!x.is_empty() && x != "VOID").then_some(x);
    }
    let title: String = v.lines().next().unwrap_or("").trim().chars().take(200).collect();
    (!title.is_empty()).then(|| format!("inline:{title}"))
}

/// The place of an event: PLAC, else the address (often used for residences).
fn place_of(ev: &Node) -> String {
    let p = ev.val("PLAC");
    if !p.is_empty() {
        return p;
    }
    let Some(a) = ev.child("ADDR") else { return String::new() };
    let mut parts: Vec<String> = vec![];
    for v in std::iter::once(a.value.replace('\n', ", ")).chain(["CITY", "STAE", "CTRY"].iter().map(|k| a.val(k))) {
        let v = v.trim().trim_matches(',').trim().to_string();
        if !v.is_empty() && !parts.iter().any(|x| x.contains(&v)) {
            parts.push(v);
        }
    }
    parts.join(", ")
}

fn note_text(n: &Node, c: &Ctx2) -> String {
    let v = n.value.trim();
    if v.starts_with('@') && v.ends_with('@') {
        return c.notes.get(&xref_of(v)).cloned().unwrap_or_default();
    }
    n.value.trim().to_string()
}

fn text_to_html(t: &str) -> String {
    let t = t.trim();
    if t.is_empty() {
        return String::new();
    }
    if t.contains('<') && t.contains('>') {
        return t.to_string();
    }
    t.split("\n\n")
        .map(|p| format!("<p>{}</p>", p.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('\n', "<br>")))
        .collect::<Vec<_>>()
        .join("")
}

fn place_coords(n: &Node) -> Option<(String, f64, f64)> {
    let plac = n.child("PLAC")?;
    let map = plac.child("MAP")?;
    let parse = |s: String| -> Option<f64> {
        let s = s.trim().to_uppercase();
        let (sign, num) = match s.chars().next()? {
            'N' | 'E' => (1.0, &s[1..]),
            'S' | 'W' => (-1.0, &s[1..]),
            _ => (1.0, s.as_str()),
        };
        num.trim().parse::<f64>().ok().map(|v| v * sign)
    };
    Some((plac.value.trim().to_string(), parse(map.val("LATI"))?, parse(map.val("LONG"))?))
}

struct Parsed {
    person: Person,
    /// field → source key (xref or "inline:<title>") with the citation details
    citations: Vec<(String, String, CitationInfo)>,
    /// index in `person.events` → source key with details
    event_cites: Vec<(usize, String, CitationInfo)>,
    sources: Vec<String>,
    coords: Vec<(String, f64, f64)>,
}

/// Details of a citation (`n SOUR @S1@` with `PAGE`, `QUAY`, `DATA.TEXT`, `NOTE`).
fn cite_info(s: &Node, c: &Ctx2) -> CitationInfo {
    let mut notes: Vec<String> = vec![];
    if let Some(d) = s.child("DATA") {
        for t in d.all("TEXT") {
            let v = t.value.trim();
            if !v.is_empty() {
                notes.push(v.to_string());
            }
        }
    }
    // GEDCOM 5.5.1 inline source: "n SOUR <title>" + "n+1 TEXT …"
    for t in s.all("TEXT") {
        let v = t.value.trim();
        if !v.is_empty() {
            notes.push(v.to_string());
        }
    }
    for n in s.all("NOTE") {
        let v = note_text(n, c);
        if !v.is_empty() {
            notes.push(v);
        }
    }
    CitationInfo {
        page: s.val("PAGE"),
        quality: s.val("QUAY").trim().parse::<u8>().ok().filter(|q| *q <= 3),
        note: notes.join("\n\n"),
    }
}

/// Citations under a node: source key + details.
fn cites_of(n: &Node, c: &Ctx2) -> Vec<(String, CitationInfo)> {
    n.all("SOUR").filter_map(|s| source_key(s).map(|k| (k, cite_info(s, c)))).collect()
}

fn extract(node: &Node, c: &Ctx2) -> Parsed {
    let mut p = Person::default();
    let mut cits: Vec<(String, String, CitationInfo)> = vec![];
    let mut ev_cits: Vec<(usize, String, CitationInfo)> = vec![];
    let mut coords = vec![];
    // Names: the primary one (first without TYPE married/aka), the married
    // surname (NAME TYPE married or _MARNM) and the birth surname.
    let split = |n: &Node| -> (String, String) {
        let nv = n.value.trim();
        let (mut given, mut surname) = match (nv.find('/'), nv.rfind('/')) {
            (Some(a), Some(b)) if b > a => (format!("{} {}", nv[..a].trim(), nv[b + 1..].trim()).trim().to_string(), nv[a + 1..b].trim().to_string()),
            (Some(_), _) => (nv.replace('/', "").trim().to_string(), String::new()),
            _ => {
                let mut it = nv.splitn(2, ' ');
                (it.next().unwrap_or("").to_string(), it.next().unwrap_or("").trim().to_string())
            }
        };
        let (g, sn, spfx) = (n.val("GIVN"), n.val("SURN"), n.val("SPFX"));
        if !g.is_empty() {
            given = g;
        }
        if !sn.is_empty() {
            surname = if !spfx.is_empty() && !norm(&sn).starts_with(&norm(&spfx)) { format!("{spfx} {sn}") } else { sn };
        }
        (given, surname)
    };
    let names: Vec<&Node> = node.all("NAME").collect();
    let ntype = |n: &Node| n.val("TYPE").to_lowercase();
    let primary = names.iter().copied().find(|n| !matches!(ntype(n).as_str(), "married" | "aka" | "nickname")).or(names.first().copied());
    if let Some(name) = primary {
        let (given, surname) = split(name);
        p.first_name = given;
        p.last_name = surname;
        let marnm = [name.val("_MARNM"), node.val("_MARNM")].into_iter().find(|v| !v.is_empty()).unwrap_or_default();
        let married_name = names.iter().filter(|n| ntype(n) == "married").map(|n| split(n).1).filter(|s| !s.is_empty()).last();
        if c.myroots && married_name.is_none() {
            // files of the WordPress plugin: _MARNM (or NPFX) = maiden name
            p.maiden_name = if !marnm.is_empty() { marnm } else { name.val("NPFX") };
        } else {
            let married = married_name.unwrap_or(marnm);
            if !married.is_empty() && norm(&married) != norm(&p.last_name) {
                p.maiden_name = std::mem::replace(&mut p.last_name, married);
            }
        }
        for n in &names {
            if std::ptr::eq(*n, name) || !matches!(ntype(n).as_str(), "birth" | "maiden") || !p.maiden_name.is_empty() {
                continue;
            }
            let s = split(n).1;
            if !s.is_empty() && norm(&s) != norm(&p.last_name) {
                p.maiden_name = s;
            }
        }
        if p.first_name.is_empty() && p.last_name.is_empty() && !p.maiden_name.is_empty() {
            p.last_name = std::mem::take(&mut p.maiden_name);
        }
    }
    p.gender = match node.val("SEX").to_uppercase().as_str() {
        "M" => Gender::Male,
        "F" => Gender::Female,
        _ => Gender::Unknown,
    };
    let cite = |n: &Node, fields: &[&str], cits: &mut Vec<(String, String, CitationInfo)>| {
        for (k, info) in cites_of(n, c) {
            for f in fields {
                cits.push((f.to_string(), k.clone(), info.clone()));
            }
        }
    };
    if let Some(name) = primary {
        cite(name, &["name"], &mut cits);
    }
    if let Some(b) = node.child("BIRT") {
        p.birth_date = date::from_gedcom(&b.val("DATE"));
        p.birth_place = place_of(b);
        cite(b, &["birth_date", "birth_place"], &mut cits);
        coords.extend(place_coords(b));
    }
    if let Some(d) = node.child("DEAT") {
        p.death_date = date::from_gedcom(&d.val("DATE"));
        p.death_place = place_of(d);
        p.death_cause = d.val("CAUS");
        let fields: &[&str] = if p.death_cause.is_empty() { &["death_date", "death_place"] } else { &["death_date", "death_place", "death_cause"] };
        cite(d, fields, &mut cits);
        coords.extend(place_coords(d));
    }
    // "1 DEAT Y", or a burial: died, date unknown
    p.deceased = p.death_date.is_empty()
        && (node.child("DEAT").map(|d| !d.value.trim().eq_ignore_ascii_case("N")).unwrap_or(false) || node.child("BURI").is_some() || node.child("CREM").is_some());
    p.occupation = node.all("OCCU").map(|o| o.value.trim().to_string()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(", ");
    for o in node.all("OCCU") {
        cite(o, &["occupation"], &mut cits);
    }
    p.education = node.val("EDUC");
    p.faith = node.val("RELI");
    if let Some(e) = node.child("EDUC") {
        cite(e, &["education"], &mut cits);
    }
    if let Some(r) = node.child("RELI") {
        cite(r, &["faith"], &mut cits);
    }
    // MyRoots writes the free notes as a NOTE marked "_TYPE notes" and the
    // unconfirmed information as "_TYPE research"; every other NOTE is the biography
    let is_free = |n: &&Node| n.val("_TYPE").eq_ignore_ascii_case("notes");
    let is_research = |n: &&Node| n.val("_TYPE").eq_ignore_ascii_case("research");
    let notes: Vec<String> = node.all("NOTE").filter(|n| !is_free(n) && !is_research(n)).map(|n| note_text(n, c)).filter(|s| !s.is_empty()).collect();
    if !notes.is_empty() {
        p.biography = text_to_html(&notes.join("\n\n"));
    }
    let free: Vec<String> = node.all("NOTE").filter(is_free).map(|n| note_text(n, c)).filter(|s| !s.is_empty()).collect();
    if !free.is_empty() {
        p.notes = text_to_html(&free.join("\n\n"));
    }
    let research: Vec<String> = node.all("NOTE").filter(is_research).map(|n| note_text(n, c)).filter(|s| !s.is_empty()).collect();
    if !research.is_empty() {
        p.research = text_to_html(&research.join("\n\n"));
    }
    for a in node.all("_ATTR") {
        let v = a.value.trim().to_string();
        if v.is_empty() {
            continue;
        }
        match a.val("TYPE").to_uppercase().as_str() {
            "HEIGHT" => p.height = v,
            "EYE_COLOR" => p.eye_color = v,
            "BLOOD" => p.blood_type = v,
            _ => {}
        }
    }
    p.clans = node.all("_CLAN").map(|c| c.value.trim().to_string()).filter(|s| !s.is_empty()).collect();
    for ev in node.all("EVEN").chain(node.all("FACT")) {
        let mut name = ev.val("TYPE");
        if name.is_empty() {
            name = ev.value.trim().to_string();
        }
        // "1 FACT Blue eyes / 2 TYPE Eye colour": the value is the content
        let value = ev.value.trim();
        let mut desc = if value.is_empty() || value == name || value == "Y" { String::new() } else { value.to_string() };
        let note = ev.child("NOTE").map(|n| note_text(n, c)).unwrap_or_default();
        if !note.is_empty() {
            desc = if desc.is_empty() { note } else { format!("{desc}\n\n{note}") };
        }
        if name.is_empty() {
            continue;
        }
        coords.extend(place_coords(ev));
        for (k, info) in cites_of(ev, c) {
            ev_cits.push((p.events.len(), k, info));
        }
        p.events.push(LifeEvent {
            name,
            date: date::from_gedcom(&ev.val("DATE")),
            place: place_of(ev),
            description: text_to_html(&desc),
            image: None,
            citations: vec![],
        });
    }
    for (tag, def) in STD_EVENTS {
        for ev in node.all(tag) {
            // a residence without a date is the address of the person
            if *tag == "RESI" && p.residence.is_empty() && ev.val("DATE").is_empty() {
                let at = place_of(ev);
                let at = if at.is_empty() { ev.value.trim().to_string() } else { at };
                if !at.is_empty() && at != "Y" {
                    p.residence = at;
                    cite(ev, &["residence"], &mut cits);
                    continue;
                }
            }
            let name = c.opts.event_names.get(*tag).cloned().unwrap_or_else(|| def.to_string());
            let note = ev.child("NOTE").map(|n| note_text(n, c)).unwrap_or_default();
            let value = ev.value.trim();
            let mut desc = if value.is_empty() || value == "Y" { String::new() } else { value.to_string() };
            // "1 EVEN / 2 TYPE …" style detail of a standard event
            let t = ev.val("TYPE");
            if !t.is_empty() && desc.is_empty() {
                desc = t;
            }
            if !note.is_empty() {
                desc = if desc.is_empty() { note } else { format!("{desc}\n\n{note}") };
            }
            coords.extend(place_coords(ev));
            for (k, info) in cites_of(ev, c) {
                ev_cits.push((p.events.len(), k, info));
            }
            p.events.push(LifeEvent { name, date: date::from_gedcom(&ev.val("DATE")), place: place_of(ev), description: text_to_html(&desc), image: None, citations: vec![] });
        }
    }
    // the burial place is also the grave
    if let Some(b) = node.child("BURI") {
        cite(b, &["burial"], &mut cits);
        p.grave.cemetery = place_of(b);
        if let Some((_, lat, lng)) = place_coords(b) {
            p.grave.lat = Some(lat);
            p.grave.lng = Some(lng);
        }
    }
    let sources: Vec<String> = node.all("SOUR").filter_map(source_key).collect();
    p.ext_ids = ext_ids(node);
    Parsed { person: p, citations: cits, event_cites: ev_cits, sources, coords }
}

/// Merges parsed values into an existing person (no overwrite with empty values).
fn merge_into(base: &mut Person, parsed: &Person) {
    let (old_birth, old_death) = (base.birth_date.clone(), base.death_date.clone());
    macro_rules! f {
        ($($x:ident),*) => {$(
            if !parsed.$x.is_empty() { base.$x = parsed.$x.clone(); }
        )*};
    }
    f!(first_name, last_name, maiden_name, birth_date, birth_place, death_date, death_place, death_cause, occupation, education, faith, height, eye_color, blood_type, biography, notes, research, residence, grave);
    // a more exact date entered in MyRoots stays ("1902-04-11" over "about 1902")
    let exact = |d: &str| d.len() == 10 && !d.starts_with(['~', '<', '>']);
    if exact(&old_birth) && !exact(&base.birth_date) && dates_agree(&old_birth, &base.birth_date).is_some() {
        base.birth_date = old_birth;
    }
    if exact(&old_death) && !exact(&base.death_date) && dates_agree(&old_death, &base.death_date).is_some() {
        base.death_date = old_death;
    }
    if parsed.deceased && base.death_date.is_empty() {
        base.deceased = true;
    }
    for e in &parsed.ext_ids {
        if !base.ext_ids.contains(e) {
            base.ext_ids.push(e.clone());
        }
    }
    if parsed.gender.is_known() || base.gender == Gender::Unset {
        base.gender = parsed.gender;
    }
    for c in &parsed.clans {
        if !base.clans.contains(c) {
            base.clans.push(c.clone());
        }
    }
    for e in &parsed.events {
        match base.events.iter_mut().find(|x| x.name == e.name && x.date == e.date && x.description == e.description) {
            Some(x) => {
                for c in &e.citations {
                    if !x.citations.iter().any(|y| y.source == c.source) {
                        x.citations.push(c.clone());
                    }
                }
            }
            None => base.events.push(e.clone()),
        }
    }
}

/// Id of a source cited in the file: a SOUR record, or an inline source
/// ("1 SOUR Parish register of …"), which becomes a source record.
fn resolve_source(db: &mut Database, ctx: &Ctx, known: &mut HashMap<String, Id>, by_title: &mut HashMap<String, Id>, rep: &mut ImportReport, key: &str) -> Result<Option<Id>> {
    if let Some(id) = known.get(key) {
        return Ok(Some(*id));
    }
    let Some(title) = key.strip_prefix("inline:") else { return Ok(None) };
    let id = match by_title.get(&norm(title)) {
        Some(id) => *id,
        None => {
            rep.sources += 1;
            let id = ops::save_source(db, ctx, Source { title: title.to_string(), ..Default::default() })?;
            by_title.insert(norm(title), id);
            id
        }
    };
    known.insert(key.to_string(), id);
    Ok(Some(id))
}

/// A person already in the tree before this import (`taken`: people created
/// or matched by this import, so two records of one file never collapse).
/// Existing people by name (first name + surname, also + maiden name) and
/// by identifiers of other programs, for imports into a tree that is not empty.
struct Matcher {
    by_name: HashMap<String, Vec<Id>>,
    by_ext: HashMap<String, Id>,
}

impl Matcher {
    fn new(db: &Database) -> Matcher {
        let mut by_name: HashMap<String, Vec<Id>> = HashMap::new();
        let mut by_ext = HashMap::new();
        for p in db.persons.values() {
            for k in name_keys(p) {
                let v = by_name.entry(k).or_default();
                if !v.contains(&p.id) {
                    v.push(p.id);
                }
            }
            for e in &p.ext_ids {
                by_ext.insert(e.clone(), p.id);
            }
        }
        Matcher { by_name, by_ext }
    }
}

fn name_keys(p: &Person) -> Vec<String> {
    let f = p.first_name.trim();
    let mut v: Vec<String> = [p.last_name.trim(), p.maiden_name.trim()].iter().map(|l| norm(&format!("{f} {l}"))).filter(|k| !k.trim().is_empty()).collect();
    v.dedup();
    v
}

/// Identifiers of a person in the program that wrote the file.
fn ext_ids(node: &Node) -> Vec<String> {
    let mut v = vec![];
    for tag in ["_UID", "UID", "RIN", "REFN", "EXID", "AFN", "_FSFTID"] {
        for n in node.all(tag) {
            let x = n.value.trim();
            if !x.is_empty() && x.len() <= 120 {
                v.push(format!("{tag}:{x}"));
            }
        }
    }
    v
}

/// Two dates can describe the same event: one is missing, or the years
/// differ by at most one (estimates, "about"), and full dates are equal.
fn dates_agree(a: &str, b: &str) -> Option<u32> {
    if a.is_empty() || b.is_empty() {
        return Some(0);
    }
    if a == b {
        return Some(3);
    }
    let (ya, yb) = (crate::date::year(a)?, crate::date::year(b)?);
    if (ya - yb).abs() > 1 {
        return None;
    }
    // two exact, different days are different people
    let exact = |d: &str| d.len() == 10 && d.chars().all(|c| c.is_ascii_digit() || c == '-');
    if exact(a) && exact(b) {
        return None;
    }
    Some(if ya == yb { 2 } else { 1 })
}

fn find_existing(db: &Database, m: &Matcher, p: &Person, taken: &HashSet<Id>) -> Option<Id> {
    for e in &p.ext_ids {
        if let Some(id) = m.by_ext.get(e) {
            if !taken.contains(id) && db.persons.contains_key(id) {
                return Some(*id);
            }
        }
    }
    let mut cands: Vec<Id> = name_keys(p).iter().flat_map(|k| m.by_name.get(k).into_iter().flatten().copied()).filter(|x| !taken.contains(x)).collect();
    cands.sort_unstable();
    cands.dedup();
    let mut best: Option<(u32, Id)> = None;
    for id in cands {
        let Some(x) = db.persons.get(&id) else { continue };
        if p.gender.is_known() && x.gender.is_known() && p.gender != x.gender {
            continue;
        }
        let (Some(b), Some(d)) = (dates_agree(&p.birth_date, &x.birth_date), dates_agree(&p.death_date, &x.death_date)) else { continue };
        let mut score = 1 + b * 3 + d * 2;
        if !p.birth_place.is_empty() && norm(&p.birth_place) == norm(&x.birth_place) {
            score += 2;
        }
        // the oldest record first among equals (re-imported files keep the order)
        if best.map(|(s, bid)| score > s || (score == s && id < bid)).unwrap_or(true) {
            best = Some((score, id));
        }
    }
    best.map(|(_, id)| id)
}

pub fn import(db: &mut Database, ctx: &Ctx, text: &str, opts: &ImportOptions) -> Result<ImportReport> {
    let nodes = parse(text);
    let mut rep = ImportReport { dry_run: opts.dry_run, ..Default::default() };
    let mut indi: Vec<&Node> = vec![];
    let mut fams: Vec<&Node> = vec![];
    let mut pets: Vec<&Node> = vec![];
    let myroots = nodes.iter().find(|n| n.tag == "HEAD").map(|h| h.val("SOUR").to_uppercase().contains("MYROOTS")).unwrap_or(false);
    let mut c = Ctx2 { notes: HashMap::new(), sources: HashMap::new(), opts, myroots };
    let mut src_nodes: Vec<&Node> = vec![];
    for n in &nodes {
        let Some(x) = &n.xref else { continue };
        match n.tag.as_str() {
            "INDI" => indi.push(n),
            "FAM" => fams.push(n),
            "_PET" => pets.push(n),
            "NOTE" => {
                c.notes.insert(x.clone(), n.value.trim().to_string());
            }
            "SOUR" => src_nodes.push(n),
            _ => {}
        }
    }
    if indi.is_empty() {
        return Err(Error::new("gedcom_no_people", "the file does not contain any people (INDI)"));
    }
    let mode = match opts.mode.as_str() {
        "skip" | "new" => opts.mode.as_str(),
        _ => "update",
    };

    // Sources
    let mut by_title: HashMap<String, Id> = db.sources.values().map(|x| (norm(&x.title), x.id)).collect();
    for s in src_nodes {
        let title = {
            let t = s.val("TITL");
            if t.is_empty() {
                let a = s.val("ABBR");
                if a.is_empty() {
                    s.value.trim().to_string()
                } else {
                    a
                }
            } else {
                t
            }
        };
        if title.is_empty() {
            continue;
        }
        let existing = by_title.get(&norm(&title)).copied();
        let id = match existing {
            Some(id) => id,
            None => {
                let mut notes = vec![];
                for k in ["AUTH", "PUBL"] {
                    let v = s.val(k);
                    if !v.is_empty() {
                        notes.push(v);
                    }
                }
                for n in s.all("NOTE") {
                    let t = note_text(n, &c);
                    if !t.is_empty() {
                        notes.push(t);
                    }
                }
                let url = s.all("_LINK").chain(s.all("WWW")).map(|n| n.value.trim().to_string()).find(|v| !v.is_empty()).unwrap_or_default();
                let date = s.child("DATA").and_then(|d| d.child("EVEN")).map(|e| date::from_gedcom(&e.val("DATE"))).unwrap_or_default();
                rep.sources += 1;
                let key = norm(&title);
                let id = ops::save_source(db, ctx, Source { title, url, notes: notes.join("\n"), date, ..Default::default() })?;
                by_title.insert(key, id);
                id
            }
        };
        c.sources.insert(s.xref.clone().unwrap_or_default(), id);
    }

    // People
    let matcher = Matcher::new(db);
    let indi_by_x: HashMap<&str, &Node> = indi.iter().filter_map(|n| n.xref.as_deref().map(|x| (x, *n))).collect();
    let mut xmap: HashMap<String, Id> = HashMap::new();
    let mut taken: HashSet<Id> = HashSet::new();
    let mut coords_all = vec![];
    for n in &indi {
        let xref = n.xref.clone().unwrap_or_default();
        let parsed = extract(n, &c);
        let mut p = parsed.person;
        if p.first_name.is_empty() && p.last_name.is_empty() {
            rep.warnings.push(("indi_no_name".into(), format!("@{xref}@")));
            continue;
        }
        // sources of the person: SOUR records and inline sources
        let mut resolved: HashMap<String, Id> = HashMap::new();
        for key in parsed.citations.iter().map(|(_, k, _)| k).chain(parsed.event_cites.iter().map(|(_, k, _)| k)).chain(parsed.sources.iter()) {
            if let Some(id) = resolve_source(db, ctx, &mut c.sources, &mut by_title, &mut rep, key)? {
                resolved.insert(key.clone(), id);
            }
        }
        for (i, key, info) in &parsed.event_cites {
            if let (Some(sid), Some(ev)) = (resolved.get(key), p.events.get_mut(*i)) {
                if !ev.citations.iter().any(|x| x.source == *sid) {
                    ev.citations.push(Citation::new(*sid, info.clone()));
                }
            }
        }
        let existing = if mode == "new" { None } else { find_existing(db, &matcher, &p, &taken) };
        if let Some(id) = existing {
            taken.insert(id);
        }
        if let (Some(id), "skip") = (existing, mode) {
            xmap.insert(xref, id);
            rep.skipped += 1;
            continue;
        }
        let id = match existing {
            Some(id) => {
                let mut base = db.persons[&id].clone();
                merge_into(&mut base, &p);
                base.updated = ctx.now;
                let base = ops::sanitize_person(&base, db);
                db.put_person(base);
                rep.updated += 1;
                id
            }
            None => {
                p.id = 0;
                let id = ops::create_person(db, ctx, p, ops::RelationEdits::default())?;
                rep.created += 1;
                taken.insert(id);
                id
            }
        };
        // citations + general source links
        if !parsed.citations.is_empty() {
            let mut person = db.persons[&id].clone();
            for (field, sx, info) in &parsed.citations {
                if let Some(sid) = resolved.get(sx) {
                    let e = person.citations.entry(field.clone()).or_default();
                    if !e.contains(sid) {
                        e.push(*sid);
                    }
                    if !info.is_empty() {
                        person.citation_info.entry(citation_key(field, *sid)).or_insert_with(|| info.clone());
                    }
                }
            }
            db.put_person(person);
        }
        for sx in &parsed.sources {
            if let Some(sid) = resolved.get(sx) {
                let mut s = db.sources[sid].clone();
                if !s.persons.contains(&id) {
                    s.persons.push(id);
                    db.put_source(s);
                }
            }
        }
        coords_all.extend(parsed.coords);
        xmap.insert(xref, id);
    }

    // Families
    for f in &fams {
        let husb = xmap.get(&xref_of(&f.val("HUSB"))).copied();
        // same-sex couples in 5.5.1 files: two HUSB (or two WIFE) lines
        let wife = xmap
            .get(&xref_of(&f.val("WIFE")))
            .copied()
            .or_else(|| f.all("HUSB").nth(1).and_then(|n| xmap.get(&xref_of(&n.value)).copied()));
        let husb = husb.or_else(|| f.all("WIFE").nth(1).and_then(|n| xmap.get(&xref_of(&n.value)).copied()));
        let marr = f.child("MARR");
        let div = f.child("DIV").or_else(|| f.child("ANUL"));
        // other family events: life events of both partners
        let mut fam_events: Vec<LifeEvent> = vec![];
        for (tag, def) in FAM_EVENTS {
            for ev in f.all(tag) {
                let name = c.opts.event_names.get(*tag).cloned().unwrap_or_else(|| def.to_string());
                let note = ev.child("NOTE").map(|n| note_text(n, &c)).unwrap_or_default();
                let mut cites = vec![];
                for (k, info) in cites_of(ev, &c) {
                    if let Some(sid) = resolve_source(db, ctx, &mut c.sources, &mut by_title, &mut rep, &k)? {
                        cites.push(Citation::new(sid, info));
                    }
                }
                fam_events.push(LifeEvent { name, date: date::from_gedcom(&ev.val("DATE")), place: place_of(ev), description: text_to_html(&note), image: None, citations: cites });
            }
        }
        for ev in f.all("EVEN").chain(f.all("FACT")) {
            let name = [ev.val("TYPE"), ev.value.trim().to_string()].into_iter().find(|v| !v.is_empty()).unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let note = ev.child("NOTE").map(|n| note_text(n, &c)).unwrap_or_default();
            let mut cites = vec![];
            for (k, info) in cites_of(ev, &c) {
                if let Some(sid) = resolve_source(db, ctx, &mut c.sources, &mut by_title, &mut rep, &k)? {
                    cites.push(Citation::new(sid, info));
                }
            }
            fam_events.push(LifeEvent { name, date: date::from_gedcom(&ev.val("DATE")), place: place_of(ev), description: text_to_html(&note), image: None, citations: cites });
        }
        if !fam_events.is_empty() {
            for pid in [husb, wife].into_iter().flatten() {
                let mut person = db.persons[&pid].clone();
                for e in &fam_events {
                    if !person.events.iter().any(|x| x.name == e.name && x.date == e.date) {
                        person.events.push(e.clone());
                    }
                }
                db.put_person(person);
            }
        }
        let mut touched = false;
        // sources of the marriage (MARR.SOUR) and of the family record (FAM.SOUR)
        let mut union_cites: Vec<Citation> = vec![];
        for n in marr.into_iter().chain(std::iter::once(*f)) {
            for (k, info) in cites_of(n, &c) {
                if let Some(sid) = resolve_source(db, ctx, &mut c.sources, &mut by_title, &mut rep, &k)? {
                    if !union_cites.iter().any(|x| x.source == sid) {
                        union_cites.push(Citation::new(sid, info));
                    }
                }
            }
        }
        if let (Some(h), Some(w)) = (husb, wife) {
            if h != w {
                let u = Union {
                    citations: union_cites.clone(),
                    date: marr.map(|m| date::from_gedcom(&m.val("DATE"))).unwrap_or_default(),
                    place: marr.map(place_of).unwrap_or_default(),
                    end_date: div.map(|d| date::from_gedcom(&d.val("DATE"))).unwrap_or_default(),
                    end_cause: if div.is_some() { end_cause::DIVORCED.into() } else { String::new() },
                    informal: marr.is_none() && f.child("_UST").map(|u| u.value.to_uppercase().contains("PARTNER")).unwrap_or(false),
                    ..Default::default()
                };
                if let Some(m) = marr {
                    coords_all.extend(place_coords(m));
                }
                ops::link_partners(db, h, w, Some(&u))?;
                touched = true;
            }
        }
        for ch in f.all("CHIL") {
            let cx = xref_of(&ch.value);
            let Some(child) = xmap.get(&cx).copied() else { continue };
            let mut frel = ParentRel::parse(&ch.val("_FREL"));
            let mut mrel = ParentRel::parse(&ch.val("_MREL"));
            if frel == ParentRel::Bio || mrel == ParentRel::Bio {
                // fallback: standard PEDI in INDI.FAMC
                if let Some(ind) = indi_by_x.get(cx.as_str()) {
                    let fx = f.xref.clone().unwrap_or_default();
                    for famc in ind.all("FAMC") {
                        if xref_of(&famc.value) != fx {
                            continue;
                        }
                        let pedi = ParentRel::parse(&famc.val("PEDI"));
                        if pedi != ParentRel::Bio {
                            if frel == ParentRel::Bio {
                                frel = pedi;
                            }
                            if mrel == ParentRel::Bio {
                                mrel = pedi;
                            }
                        }
                    }
                }
            }
            let mut cp = db.persons[&child].clone();
            let mut changed = false;
            if let Some(h) = husb {
                if h != child && !ops::creates_cycle(db, child, h) {
                    cp.set_parent(ParentRole::Father, Some(h), frel);
                    changed = true;
                }
            }
            if let Some(w) = wife {
                if w != child && !ops::creates_cycle(db, child, w) {
                    cp.set_parent(ParentRole::Mother, Some(w), mrel);
                    changed = true;
                }
            }
            if changed {
                // maiden name from the father (plugin behaviour)
                ops::fill_maiden_name(db, &mut cp);
                db.put_person(cp);
                touched = true;
            }
        }
        if touched {
            rep.families += 1;
        }
    }

    // Pets (_PET)
    for n in &pets {
        let name = n.val("NAME");
        if name.is_empty() {
            rep.warnings.push(("pet_no_name".into(), String::new()));
            continue;
        }
        let owner = xmap.get(&xref_of(&n.val("_OWNER"))).copied();
        if db.pets.values().any(|p| norm(&p.name) == norm(&name) && p.owner == owner) {
            rep.pets_skipped += 1;
            continue;
        }
        let sp = ops::key(&n.val("_SPECIES"));
        let species = if sp.is_empty() || ["dog", "cat", "bird", "horse", "other"].contains(&sp.as_str()) { sp } else { "other".into() };
        let pet = Pet {
            name,
            species,
            breed: n.val("_BREED"),
            gender: match n.val("SEX").to_uppercase().as_str() {
                "M" => Gender::Male,
                "F" => Gender::Female,
                _ => Gender::Unset,
            },
            birth_date: n.child("BIRT").map(|b| date::from_gedcom(&b.val("DATE"))).unwrap_or_default(),
            death_date: n.child("DEAT").map(|b| date::from_gedcom(&b.val("DATE"))).unwrap_or_default(),
            owner,
            biography: n.child("NOTE").map(|x| text_to_html(&note_text(x, &c))).unwrap_or_default(),
            ..Default::default()
        };
        ops::save_pet(db, ctx, pet)?;
        rep.pets_created += 1;
    }

    // Coordinates
    for (name, lat, lng) in coords_all {
        if name.is_empty() {
            continue;
        }
        if db.place_by_name(&name).map(|p| p.lat.is_some()).unwrap_or(false) {
            continue;
        }
        ops::set_place_coords(db, ctx, &name, Some(lat), Some(lng), "gedcom");
        rep.places += 1;
    }
    Ok(rep)
}

/* ------------------------------------------------------------------ */
/*  Export                                                              */
/* ------------------------------------------------------------------ */

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ExportOptions {
    pub notes: bool,
    pub attrs: bool,
    pub sources: bool,
    pub places: bool,
    /// Reference photos as `media/<id>.<ext>` (for an archive export).
    pub media: bool,
    pub pets: bool,
    /// Only published records.
    pub published_only: bool,
    pub app_version: String,
    pub tree_name: String,
    /// Translated names of standard events (as for the import), so that
    /// "Chrzest" is written back as BAPM.
    pub event_names: HashMap<String, String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            notes: true,
            attrs: true,
            sources: true,
            places: true,
            media: false,
            pets: true,
            published_only: false,
            app_version: String::new(),
            tree_name: String::new(),
            event_names: HashMap::new(),
        }
    }
}

/// Splits a long value into CONC lines (never next to a space, so that no
/// program loses it when trimming lines).
fn conc_lines(text: &str, max: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = vec![];
    let mut i = 0;
    while chars.len() - i > max {
        let mut cut = i + max;
        while cut > i + max / 2 && (chars[cut - 1] == ' ' || chars[cut] == ' ') {
            cut -= 1;
        }
        out.push(chars[i..cut].iter().collect());
        i = cut;
    }
    out.push(chars[i..].iter().collect());
    out
}

/// One value line, continued with CONC when it is too long for GEDCOM (255 chars a line).
fn gline(level: usize, tag: &str, value: &str) -> String {
    let v = gv(value);
    let mut parts = conc_lines(&v, 200).into_iter();
    let mut out = format!("{level} {tag} {}\n", parts.next().unwrap_or_default());
    for p in parts {
        out += &format!("{} CONC {p}\n", level + 1);
    }
    out
}

/// A one-line value: no line breaks, "@" escaped as "@@" (both 5.5.1 and 7).
fn gv(v: &str) -> String {
    v.replace(['\r', '\n'], " ").trim().replace('@', "@@")
}

fn gnote(text: &str, level: usize, tag: &str) -> String {
    let text = text.replace('\r', "").replace('@', "@@");
    if text.trim().is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for (i, para) in text.trim().split('\n').enumerate() {
        for (j, chunk) in conc_lines(para, 200).into_iter().enumerate() {
            let line = match (i, j) {
                (0, 0) => format!("{level} {tag}"),
                (_, 0) => format!("{} CONT", level + 1),
                _ => format!("{} CONC", level + 1),
            };
            out += &if chunk.is_empty() { format!("{line}\n") } else { format!("{line} {chunk}\n") };
        }
    }
    out
}

pub fn ext_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/avif" => "avif",
        "image/svg+xml" => "svg",
        "application/pdf" => "pdf",
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/mp4" | "audio/x-m4a" | "audio/aac" | "audio/m4a" => "m4a",
        "audio/webm" => "webm",
        "audio/ogg" | "audio/opus" => "ogg",
        "audio/wav" | "audio/x-wav" | "audio/wave" => "wav",
        "audio/flac" | "audio/x-flac" => "flac",
        _ => "jpg",
    }
}

pub fn export(db: &Database, ctx: &Ctx, o: &ExportOptions) -> String {
    let persons: BTreeMap<Id, &Person> = db.persons.iter().filter(|(_, p)| !o.published_only || p.status == Status::Published).map(|(k, v)| (*k, v)).collect();
    // Families: (father, mother) → children
    #[derive(Default)]
    struct Fam {
        husb: Option<Id>,
        wife: Option<Id>,
        children: Vec<Id>,
        union: Option<Union>,
    }
    let mut fams: BTreeMap<(Id, Id), Fam> = BTreeMap::new();
    for p in persons.values() {
        let f = p.father.filter(|x| persons.contains_key(x));
        let m = p.mother.filter(|x| persons.contains_key(x));
        if f.is_none() && m.is_none() {
            continue;
        }
        let e = fams.entry((f.unwrap_or(0), m.unwrap_or(0))).or_default();
        e.husb = f;
        e.wife = m;
        e.children.push(p.id);
    }
    for u in db.unions.values() {
        let Some(b) = u.b else { continue };
        if !persons.contains_key(&u.a) || !persons.contains_key(&b) {
            continue;
        }
        let (h, w) = if persons[&u.a].gender == Gender::Female { (b, u.a) } else { (u.a, b) };
        let key = if fams.contains_key(&(w, h)) && !fams.contains_key(&(h, w)) { (w, h) } else { (h, w) };
        let e = fams.entry(key).or_default();
        if e.husb.is_none() && e.wife.is_none() {
            e.husb = Some(key.0);
            e.wife = Some(key.1);
        }
        e.union = Some(u.clone());
    }
    let mut fam_x: HashMap<(Id, Id), String> = HashMap::new();
    let mut famc: HashMap<Id, String> = HashMap::new();
    let mut fams_of: HashMap<Id, Vec<String>> = HashMap::new();
    for (i, (k, f)) in fams.iter().enumerate() {
        let x = format!("F{}", i + 1);
        fam_x.insert(*k, x.clone());
        for c in &f.children {
            famc.insert(*c, x.clone());
        }
        for p in [f.husb, f.wife].into_iter().flatten() {
            fams_of.entry(p).or_default().push(x.clone());
        }
    }

    let today = ctx.today.format("%d %b %Y").to_string().to_uppercase();
    let mut out = String::new();
    out += "0 HEAD\n1 SOUR MyRoots\n";
    out += &format!("2 VERS {}\n2 NAME MyRoots\n", gv(&o.app_version));
    if !o.tree_name.is_empty() {
        out += &format!("2 CORP {}\n", gv(&o.tree_name));
    }
    out += &format!("1 DEST ANY\n1 DATE {today}\n1 SUBM @U1@\n1 CHAR UTF-8\n1 GEDC\n2 VERS 5.5.1\n2 FORM LINEAGE-LINKED\n");
    // standard event tags by their (translated) names
    let mut std_tag: HashMap<String, &str> = HashMap::new();
    for (tag, def) in STD_EVENTS {
        if tag.starts_with('_') {
            continue;
        }
        std_tag.entry(norm(def)).or_insert(tag);
        if let Some(n) = o.event_names.get(*tag) {
            std_tag.entry(norm(n)).or_insert(tag);
        }
    }
    const ATTRS: [&str; 4] = ["TITL", "NATI", "DSCR", "PROP"];

    let by_name = db.places_by_name();
    let plac = |level: usize, name: &str| -> String {
        if name.is_empty() {
            return String::new();
        }
        let mut s = format!("{level} PLAC {}\n", gv(name));
        if o.places {
            if let Some(p) = by_name.get(&name.trim().to_lowercase()) {
                if let (Some(lat), Some(lng)) = (p.lat, p.lng) {
                    s += &format!("{} MAP\n{} LATI {}{:.6}\n{} LONG {}{:.6}\n", level + 1, level + 2, if lat < 0.0 { 'S' } else { 'N' }, lat.abs(), level + 2, if lng < 0.0 { 'W' } else { 'E' }, lng.abs());
                }
            }
        }
        s
    };
    // "n SOUR @S1@" with PAGE, QUAY and a NOTE of the citation
    let cite_block = |level: usize, sid: Id, info: Option<&CitationInfo>| -> String {
        let mut s = format!("{level} SOUR @S{sid}@\n");
        if let Some(i) = info {
            if !i.page.trim().is_empty() {
                s += &gline(level + 1, "PAGE", &i.page);
            }
            if let Some(q) = i.quality {
                s += &format!("{} QUAY {}\n", level + 1, q.min(3));
            }
            if !i.note.trim().is_empty() {
                s += &gnote(&i.note, level + 1, "NOTE");
            }
        }
        s
    };
    let cites = |p: &Person, fields: &[&str], level: usize| -> String {
        if !o.sources {
            return String::new();
        }
        let mut ids: Vec<(Id, Option<&CitationInfo>)> = vec![];
        for f in fields {
            if let Some(v) = p.citations.get(*f) {
                for i in v {
                    if !db.sources.contains_key(i) {
                        continue;
                    }
                    let info = p.citation_info.get(&citation_key(f, *i));
                    match ids.iter_mut().find(|(x, _)| x == i) {
                        Some(e) => {
                            if e.1.is_none() {
                                e.1 = info;
                            }
                        }
                        None => ids.push((*i, info)),
                    }
                }
            }
        }
        ids.iter().map(|(i, info)| cite_block(level, *i, *info)).collect()
    };
    let cites_list = |list: &[Citation], level: usize| -> String {
        if !o.sources {
            return String::new();
        }
        list.iter().filter(|c| db.sources.contains_key(&c.source)).map(|c| cite_block(level, c.source, Some(&c.info()))).collect()
    };
    let ix = db.index();

    for p in persons.values() {
        out += &format!("0 @I{}@ INDI\n", p.id);
        // The birth name first (the genealogical convention), the current
        // surname as a second NAME of TYPE married.
        let (f, l, m) = (gv(&p.first_name).replace('/', ""), gv(&p.last_name).replace('/', ""), gv(&p.maiden_name).replace('/', ""));
        let name = |surname: &str, typ: &str| -> String {
            let mut s = format!("1 NAME {f} /{surname}/\n");
            if !typ.is_empty() {
                s += &format!("2 TYPE {typ}\n");
            }
            if !f.is_empty() {
                s += &format!("2 GIVN {f}\n");
            }
            if !surname.is_empty() {
                s += &format!("2 SURN {surname}\n");
            }
            s
        };
        if !m.is_empty() && norm(&m) != norm(&l) {
            out += &name(&m, "birth");
            out += &cites(p, &["name"], 2);
            out += &name(&l, "married");
        } else {
            out += &name(&l, "");
            out += &cites(p, &["name"], 2);
        }
        out += &format!("1 SEX {}\n", match p.gender { Gender::Male => "M", Gender::Female => "F", _ => "U" });
        if !p.birth_date.is_empty() || !p.birth_place.is_empty() {
            out += "1 BIRT\n";
            if !p.birth_date.is_empty() {
                out += &format!("2 DATE {}\n", date::to_gedcom(&p.birth_date));
            }
            out += &plac(2, &p.birth_place);
            out += &cites(p, &["birth_date", "birth_place"], 2);
        }
        if !p.death_date.is_empty() || !p.death_place.is_empty() || !p.death_cause.is_empty() {
            out += "1 DEAT\n";
            if !p.death_date.is_empty() {
                out += &format!("2 DATE {}\n", date::to_gedcom(&p.death_date));
            }
            out += &plac(2, &p.death_place);
            if !p.death_cause.is_empty() {
                out += &format!("2 CAUS {}\n", gv(&p.death_cause));
            }
            out += &cites(p, &["death_date", "death_place", "death_cause"], 2);
        } else if p.deceased {
            out += "1 DEAT Y\n";
        }
        if !p.occupation.is_empty() {
            out += &gline(1, "OCCU", &p.occupation);
            out += &cites(p, &["occupation"], 2);
        }
        if !p.education.is_empty() {
            out += &gline(1, "EDUC", &p.education);
            out += &cites(p, &["education"], 2);
        }
        if !p.faith.is_empty() {
            out += &gline(1, "RELI", &p.faith);
            out += &cites(p, &["faith"], 2);
        }
        if !p.residence.is_empty() {
            out += "1 RESI\n";
            out += &gline(2, "ADDR", &p.residence);
            out += &cites(p, &["residence"], 2);
        }
        if o.attrs {
            for (v, t) in [(&p.height, "HEIGHT"), (&p.eye_color, "EYE_COLOR"), (&p.blood_type, "BLOOD")] {
                if !v.is_empty() {
                    out += &format!("1 _ATTR {}\n2 TYPE {t}\n", gv(v));
                }
            }
        }
        for c in &p.clans {
            out += &format!("1 _CLAN {}\n", gv(c));
        }
        for e in &p.events {
            let name = gv(&e.name);
            if name.is_empty() {
                continue;
            }
            let mut d = strip_tags(&e.description).trim().to_string();
            match std_tag.get(&norm(&name)).copied() {
                // attributes carry their value on the line: "1 TITL hrabia"
                Some(tag) if ATTRS.contains(&tag) && !d.is_empty() && !d.contains('\n') => {
                    out += &gline(1, tag, &d);
                    d.clear();
                }
                Some(tag) if !ATTRS.contains(&tag) => out += &format!("1 {tag}\n"),
                _ => out += &format!("1 EVEN\n2 TYPE {name}\n"),
            }
            if !e.date.is_empty() {
                out += &format!("2 DATE {}\n", date::to_gedcom(&e.date));
            }
            out += &plac(2, &e.place);
            if !d.is_empty() {
                out += &gnote(&d, 2, "NOTE");
            }
            out += &cites_list(&e.citations, 2);
            // sources of the grave go with a burial event
            if std_tag.get(&norm(&name)).copied() == Some("BURI") {
                out += &cites(p, &["burial"], 2);
            }
        }
        // the grave as a burial, unless a burial event says it already
        let g = &p.grave;
        if (!g.cemetery.is_empty() || g.lat.is_some() || p.citations.contains_key("burial")) && !p.events.iter().any(|e| std_tag.get(&norm(&gv(&e.name))).copied() == Some("BURI")) {
            out += "1 BURI\n";
            if !g.cemetery.is_empty() {
                out += &gline(2, "PLAC", &g.cemetery);
            }
            if let (Some(lat), Some(lng)) = (g.lat, g.lng) {
                if g.cemetery.is_empty() {
                    out += "2 PLAC\n";
                }
                out += &format!("3 MAP\n4 LATI {}{:.6}\n4 LONG {}{:.6}\n", if lat < 0.0 { 'S' } else { 'N' }, lat.abs(), if lng < 0.0 { 'W' } else { 'E' }, lng.abs());
            }
            if !g.location.is_empty() {
                out += &gnote(&g.location, 2, "NOTE");
            }
            out += &cites(p, &["burial"], 2);
        }
        if let Some(x) = famc.get(&p.id) {
            out += &format!("1 FAMC @{x}@\n");
            let pedi = if p.father_rel == ParentRel::Adopted || p.mother_rel == ParentRel::Adopted {
                "adopted"
            } else if p.father_rel == ParentRel::Foster || p.mother_rel == ParentRel::Foster {
                "foster"
            } else {
                ""
            };
            if !pedi.is_empty() {
                out += &format!("2 PEDI {pedi}\n");
            }
        }
        if let Some(v) = fams_of.get(&p.id) {
            let mut v = v.clone();
            v.dedup();
            for x in v {
                out += &format!("1 FAMS @{x}@\n");
            }
        }
        if o.notes {
            let b = strip_tags(&p.biography);
            if !b.is_empty() {
                out += &gnote(&b, 1, "NOTE");
                out += &cites(p, &["biography"], 2);
            }
            let n = strip_tags(&p.notes);
            if !n.trim().is_empty() {
                out += &gnote(&n, 1, "NOTE");
                out += "2 _TYPE notes\n";
            }
            let r = strip_tags(&p.research);
            if !r.trim().is_empty() {
                out += &gnote(&r, 1, "NOTE");
                out += "2 _TYPE research\n";
            }
        }
        if o.sources {
            for sid in ix.sources_of(p.id) {
                out += &format!("1 SOUR @S{sid}@\n");
            }
        }
        if o.media {
            for m in p.photo.iter().chain(p.gallery.iter().map(|g| &g.media)) {
                if let Some(md) = db.media.get(m) {
                    out += &format!("1 OBJE\n2 FILE media/{}.{}\n3 FORM {}\n", md.id, ext_for_mime(&md.mime), ext_for_mime(&md.mime));
                    if Some(*m) == p.photo {
                        out += "2 _PRIM Y\n";
                    }
                }
            }
            for r in &p.recordings {
                if let Some(md) = db.media.get(&r.media) {
                    let ext = ext_for_mime(&md.mime);
                    out += &format!("1 OBJE\n2 FILE media/{}.{ext}\n3 FORM {ext}\n3 TYPE audio\n", md.id);
                    if !r.title.is_empty() {
                        out += &gline(2, "TITL", &r.title);
                    }
                }
            }
        }
    }
    for (k, f) in &fams {
        out += &format!("0 @{}@ FAM\n", fam_x[k]);
        if let Some(h) = f.husb {
            out += &format!("1 HUSB @I{h}@\n");
        }
        if let Some(w) = f.wife {
            out += &format!("1 WIFE @I{w}@\n");
        }
        if let Some(u) = &f.union {
            if !u.date.is_empty() || !u.place.is_empty() || !u.informal {
                out += "1 MARR\n";
                if !u.date.is_empty() {
                    out += &format!("2 DATE {}\n", date::to_gedcom(&u.date));
                }
                out += &plac(2, &u.place);
                out += &cites_list(&u.citations, 2);
            } else {
                // an informal relationship: the sources of the family record
                out += &cites_list(&u.citations, 1);
            }
            if u.informal {
                out += "1 _UST PARTNERS\n";
            }
            if !u.end_date.is_empty() && u.end_cause == end_cause::DIVORCED {
                out += &format!("1 DIV\n2 DATE {}\n", date::to_gedcom(&u.end_date));
            } else if u.end_cause == end_cause::DIVORCED {
                out += "1 DIV Y\n";
            }
        }
        for c in &f.children {
            out += &format!("1 CHIL @I{c}@\n");
            let cp = &persons[c];
            let code = |r: ParentRel| match r {
                ParentRel::Adopted => Some("Adopted"),
                ParentRel::Foster => Some("Foster"),
                ParentRel::Step => Some("Step"),
                ParentRel::Bio => None,
            };
            if f.husb.is_some() && cp.father == f.husb {
                if let Some(c) = code(cp.father_rel) {
                    out += &format!("2 _FREL {c}\n");
                }
            }
            if f.wife.is_some() && cp.mother == f.wife {
                if let Some(c) = code(cp.mother_rel) {
                    out += &format!("2 _MREL {c}\n");
                }
            }
        }
    }
    if o.sources {
        for s in db.sources.values() {
            out += &format!("0 @S{}@ SOUR\n1 TITL {}\n", s.id, gv(&s.title));
            let d = date::to_gedcom(&s.date);
            if !d.is_empty() {
                out += &format!("1 DATA\n2 EVEN\n3 DATE {d}\n");
            }
            if !s.url.is_empty() {
                out += &format!("1 WWW {}\n", gv(&s.url));
            }
            if !s.notes.is_empty() {
                out += &gnote(&s.notes, 1, "NOTE");
            }
        }
    }
    if o.pets {
        let mut n = 0;
        for p in db.pets.values() {
            if o.published_only && p.status != Status::Published {
                continue;
            }
            let name = gv(&p.name);
            if name.is_empty() {
                continue;
            }
            n += 1;
            out += &format!("0 @PET{n}@ _PET\n1 NAME {name}\n");
            if !p.species.is_empty() {
                out += &format!("1 _SPECIES {}\n", gv(&p.species));
            }
            if !p.breed.is_empty() {
                out += &format!("1 _BREED {}\n", gv(&p.breed));
            }
            if p.gender.is_known() {
                out += &format!("1 SEX {}\n", p.gender.code());
            }
            if !p.birth_date.is_empty() {
                out += &format!("1 BIRT\n2 DATE {}\n", date::to_gedcom(&p.birth_date));
            }
            if !p.death_date.is_empty() {
                out += &format!("1 DEAT\n2 DATE {}\n", date::to_gedcom(&p.death_date));
            }
            if let Some(o) = p.owner.filter(|o| persons.contains_key(o)) {
                out += &format!("1 _OWNER @I{o}@\n");
            }
            if o.notes {
                let b = strip_tags(&p.biography);
                if !b.is_empty() {
                    out += &gnote(&b, 1, "NOTE");
                }
            }
        }
    }
    let subm = if o.tree_name.trim().is_empty() { "MyRoots".to_string() } else { gv(&o.tree_name) };
    out += &format!("0 @U1@ SUBM\n1 NAME {subm}\n");
    out += "0 TRLR\n";
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use crate::life;

    const SAMPLE: &str = "0 HEAD\n1 CHAR UTF-8\n0 @N1@ NOTE Shared note\n1 CONT second line\n0 @S1@ SOUR\n1 TITL Parish book\n0 @I1@ INDI\n1 NAME Jan /Kowalski/\n1 SEX M\n1 BIRT\n2 DATE ABT 1900\n2 PLAC Kraków\n3 MAP\n4 LATI N50.06\n4 LONG E19.94\n2 SOUR @S1@\n1 NOTE @N1@\n1 BURI\n2 DATE 1970\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME Anna /Nowak/\n1 SEX F\n1 FAMS @F1@\n0 @I3@ INDI\n1 NAME Ewa /Kowalska/\n1 SEX F\n1 FAMC @F1@\n2 PEDI adopted\n0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n1 MARR\n2 DATE 12 JUN 1925\n1 CHIL @I3@\n0 @PET1@ _PET\n1 NAME Burek\n1 _SPECIES dog\n1 _OWNER @I1@\n0 TRLR\n";

    const CITED: &str = "0 HEAD\n1 CHAR UTF-8\n0 @S1@ SOUR\n1 TITL Księga urodzeń\n0 @S2@ SOUR\n1 TITL Księga ślubów\n0 @I1@ INDI\n1 NAME Jan /Kowalski/\n2 SOUR @S1@\n1 SEX M\n1 BIRT\n2 DATE 3 MAR 1900\n2 PLAC Kraków\n2 SOUR @S1@\n3 PAGE akt 112\n3 QUAY 3\n3 DATA\n4 TEXT Jan, syn Józefa\n1 OCCU kowal\n2 SOUR Spis ludności 1921\n3 PAGE s. 4\n1 RESI\n2 DATE 1930\n2 PLAC Tarnów\n2 SOUR @S1@\n3 PAGE k. 7\n1 BURI\n2 PLAC Cmentarz Rakowicki\n2 SOUR @S1@\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME Anna /Nowak/\n1 SEX F\n1 FAMS @F1@\n0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n1 MARR\n2 DATE 1925\n2 SOUR @S2@\n3 PAGE akt 5\n3 QUAY 2\n0 TRLR\n";

    #[test]
    fn reimport_with_new_people_does_not_duplicate() {
        let c = Ctx::new(0, NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
        let mut db = Database::new(0);
        let v1 = "0 HEAD\n0 @I1@ INDI\n1 NAME Jan /Kowalski/\n1 SEX M\n1 _UID AAA1\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME Anna /Nowak/\n1 SEX F\n1 BIRT\n2 DATE ABT 1902\n1 FAMS @F1@\n0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n0 TRLR\n";
        import(&mut db, &c, v1, &ImportOptions::default()).unwrap();
        // the user corrects data in MyRoots
        let anna = db.persons.values().find(|p| p.first_name == "Anna").unwrap().id;
        let mut a = db.persons[&anna].clone();
        a.birth_date = "1902-04-11".into();
        db.put_person(a);
        // a newer file from the other program: Jan renamed there, a birth date
        // where there was none, Anna with "about", and a new child
        let v2 = "0 HEAD\n0 @I1@ INDI\n1 NAME Janek /Kowalski/\n1 SEX M\n1 _UID AAA1\n1 BIRT\n2 DATE 1899\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME Anna /Nowak/\n1 SEX F\n1 BIRT\n2 DATE ABT 1902\n1 FAMS @F1@\n0 @I3@ INDI\n1 NAME Piotr /Kowalski/\n1 BIRT\n2 DATE 1925\n1 FAMC @F1@\n0 @F1@ FAM\n1 HUSB @I1@\n1 WIFE @I2@\n1 CHIL @I3@\n0 TRLR\n";
        let r = import(&mut db, &c, v2, &ImportOptions::default()).unwrap();
        assert_eq!((r.created, r.updated), (1, 2), "{r:?}");
        assert_eq!(db.persons.len(), 3);
        assert_eq!(db.unions.len(), 1);
        assert_eq!(db.persons[&anna].birth_date, "1902-04-11");
        let piotr = db.persons.values().find(|p| p.first_name == "Piotr").unwrap();
        assert_eq!(piotr.mother, Some(anna));
    }

    #[test]
    fn citations_of_facts_events_and_marriages() {
        let c = Ctx::new(0, NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
        let check = |db: &Database| {
            let jan = db.persons.values().find(|p| p.first_name == "Jan").unwrap();
            let s1 = db.sources.values().find(|s| s.title == "Księga urodzeń").unwrap().id;
            let s2 = db.sources.values().find(|s| s.title == "Księga ślubów").unwrap().id;
            let census = db.sources.values().find(|s| s.title == "Spis ludności 1921").unwrap().id;
            assert_eq!(jan.citations["name"], vec![s1]);
            assert_eq!(jan.citations["birth_date"], vec![s1]);
            assert_eq!(jan.citations["birth_place"], vec![s1]);
            assert_eq!(jan.citations["burial"], vec![s1]);
            let b = &jan.citation_info[&citation_key("birth_date", s1)];
            assert_eq!((b.page.as_str(), b.quality), ("akt 112", Some(3)));
            assert!(b.note.contains("Jan, syn Józefa"));
            assert_eq!(jan.citations["occupation"], vec![census]);
            assert_eq!(jan.citation_info[&citation_key("occupation", census)].page, "s. 4");
            let resi = jan.events.iter().find(|e| e.place == "Tarnów").unwrap();
            assert_eq!(resi.citations.len(), 1);
            assert_eq!((resi.citations[0].source, resi.citations[0].page.as_str()), (s1, "k. 7"));
            let u = db.unions.values().next().unwrap();
            assert_eq!(u.citations.len(), 1);
            assert_eq!((u.citations[0].source, u.citations[0].page.as_str(), u.citations[0].quality), (s2, "akt 5", Some(2)));
        };
        let mut db = Database::new(0);
        import(&mut db, &c, CITED, &ImportOptions::default()).unwrap();
        check(&db);
        // the same after an export and an import into an empty tree
        let text = export(&db, &c, &ExportOptions { sources: true, notes: true, places: true, attrs: true, ..Default::default() });
        assert!(text.contains("3 PAGE akt 112\n3 QUAY 3\n"), "{text}");
        assert!(text.contains("1 MARR\n2 DATE 1925\n2 SOUR @S"), "{text}");
        let mut db2 = Database::new(0);
        import(&mut db2, &c, &text, &ImportOptions::default()).unwrap();
        check(&db2);
        // deleting a source removes its citations everywhere
        let s1 = db.sources.values().find(|s| s.title == "Księga urodzeń").unwrap().id;
        crate::ops::delete_source(&mut db, s1).unwrap();
        let jan = db.persons.values().find(|p| p.first_name == "Jan").unwrap();
        assert!(!jan.citations.contains_key("birth_date"));
        assert!(jan.citation_info.keys().all(|k| !k.ends_with(&format!("#{s1}"))));
        assert!(jan.events.iter().all(|e| e.citations.iter().all(|x| x.source != s1)));
    }

    #[test]
    fn roundtrip() {
        let c = Ctx::new(0, NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
        let mut db = Database::new(0);
        let rep = import(&mut db, &c, SAMPLE, &ImportOptions::default()).unwrap();
        assert_eq!((rep.created, rep.families, rep.pets_created, rep.sources, rep.places), (3, 1, 1, 1, 1));
        let jan = db.persons.values().find(|p| p.first_name == "Jan").unwrap().clone();
        assert_eq!(jan.birth_date, "~1900");
        assert!(jan.biography.contains("Shared note<br>second line"));
        assert_eq!(jan.citations.get("birth_date").map(|v| v.len()), Some(1));
        assert_eq!(jan.events.len(), 1);
        let ewa = db.persons.values().find(|p| p.first_name == "Ewa").unwrap();
        assert_eq!(ewa.father, Some(jan.id));
        assert_eq!(ewa.father_rel, ParentRel::Adopted);
        assert_eq!(ewa.maiden_name, ""); // Kowalska = feminine form of Kowalski
        assert_eq!(db.unions.len(), 1);
        assert_eq!(db.unions.values().next().unwrap().date, "1925-06-12");
        let out = export(&db, &c, &ExportOptions::default());
        assert!(out.contains("2 DATE ABT 1900"));
        assert!(out.contains("1 MARR\n2 DATE 12 JUN 1925"));
        assert!(out.contains("_FREL Adopted"));
        // re-import in update mode → no new people
        let rep2 = import(&mut db, &c, &out, &ImportOptions::default()).unwrap();
        assert_eq!(rep2.created, 0);
        assert_eq!(rep2.updated, 3);
        assert_eq!(db.unions.len(), 1);
    }

    fn run(text: &str) -> Database {
        let c = Ctx::new(0, NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
        let mut db = Database::new(0);
        import(&mut db, &c, text, &ImportOptions::default()).unwrap();
        db
    }
    fn who<'a>(db: &'a Database, first: &str) -> &'a Person {
        db.persons.values().find(|p| p.first_name == first).unwrap_or_else(|| panic!("no {first}"))
    }

    #[test]
    fn myheritage_style() {
        let db = run("0 HEAD\n1 SOUR MYHERITAGE\n1 CHAR UTF-8\n0 @I1@ INDI\n1 _UPD 14 OCT 2026\n1 NAME Anna /Kowalska/\n2 GIVN Anna\n2 SURN Kowalska\n2 _MARNM Nowak\n1 SEX F\n1 BIRT\n2 DATE ABT 1901\n2 PLAC Kraków, małopolskie, Polska\n2 SOUR @S1@\n3 PAGE str. 12\n1 RESI\n2 ADDR ul. Długa 5\n3 CITY Gdańsk\n3 CTRY Polska\n2 DATE 1930\n1 EVEN Order Virtuti Militari\n2 TYPE Odznaczenie\n1 DEAT Y\n1 RIN MH:I1\n1 _UID 5F1E\n0 @S1@ SOUR\n1 TITL Księga chrztów parafii Mariackiej\n1 AUTH Parafia\n0 TRLR\n");
        let a = who(&db, "Anna");
        assert_eq!((a.last_name.as_str(), a.maiden_name.as_str()), ("Nowak", "Kowalska"));
        assert!(a.deceased && !life::is_alive(a, 0, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()));
        assert_eq!(life::years(a), "1901–?");
        let resi = a.events.iter().find(|e| e.name == "Residence").unwrap();
        assert_eq!(resi.place, "ul. Długa 5, Gdańsk, Polska");
        let medal = a.events.iter().find(|e| e.name == "Odznaczenie").unwrap();
        assert!(medal.description.contains("Order Virtuti Militari"));
        assert_eq!(a.citations.get("birth_date").map(|v| v.len()), Some(1));
    }

    #[test]
    fn ansel_legacy_file() {
        // Legacy/PAF style: ANSEL, CONC split in the middle of a word, Julian and dual dates,
        // inline source, surname prefix, NAME TYPE birth
        let mut raw = b"0 HEAD\n1 SOUR Legacy\n1 CHAR ANSEL\n0 @I1@ INDI\n1 NAME J\xe2ozef /Wi\xe2sniewski/\n1 SEX M\n1 BIRT\n2 DATE @#DJULIAN@ 12 FEB 1699/00\n2 SOUR Ksi\xf1ega metrykalna Z\xe7z\xf1ary\n1 NOTE Urodzi\xb1 si\xf1e w ma\xb1ym dwor".to_vec();
        raw.extend(b"ku\n2 CONC  pod lasem.\n2 CONT Druga linia @@ znak.\n0 @I2@ INDI\n1 NAME Maria /van der Berg/\n2 SPFX van der\n2 SURN Berg\n1 SEX F\n1 BURI\n2 PLAC Amsterdam\n0 @I3@ INDI\n1 NAME Zofia /Kowalska/\n2 TYPE married\n1 NAME Zofia /Wi\xe2sniewska/\n2 TYPE birth\n1 SEX F\n0 TRLR\n");
        let c = Ctx::new(0, NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
        let mut db = Database::new(0);
        let rep = import(&mut db, &c, &decode(&raw), &ImportOptions::default()).unwrap();
        let j = who(&db, "Józef");
        assert_eq!(j.last_name, "Wiśniewski");
        assert_eq!(j.birth_date, "1700-02-12");
        assert!(j.biography.contains("Urodził się w małym dworku pod lasem.<br>Druga linia @ znak."), "{}", j.biography);
        assert_eq!(rep.sources, 1);
        assert!(db.sources.values().any(|s| s.title == "Księga metrykalna Zżąry" || s.title.starts_with("Księga metrykalna")));
        assert_eq!(j.citations.get("birth_date").map(|v| v.len()), Some(1));
        let m = who(&db, "Maria");
        assert_eq!(m.last_name, "van der Berg");
        assert!(m.deceased);
        let z = who(&db, "Zofia");
        assert_eq!((z.last_name.as_str(), z.maiden_name.as_str()), ("Kowalska", "Wiśniewska"));
    }

    #[test]
    fn gedcom7_and_families() {
        let db = run("0 HEAD\n1 GEDC\n2 VERS 7.0\n0 @N1@ SNOTE Shared @@ note\n0 @I1@ INDI\n1 NAME Alex /Smith/\n1 SEX X\n1 SNOTE @N1@\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME Sam /Smith/\n1 SEX M\n1 FAMS @F1@\n0 @I3@ INDI\n1 NAME Kid /Smith/\n1 FAMC @F1@\n2 PEDI ADOPTED\n0 @F1@ FAM\n1 HUSB @I2@\n1 WIFE @I1@\n1 ENGA\n2 DATE 3 MAR 1990\n1 MARR\n2 DATE BET 1990 AND 1991\n1 ANUL\n2 DATE 1995\n1 CHIL @I3@\n1 CHIL @VOID@\n0 TRLR\n");
        let a = who(&db, "Alex");
        assert!(a.biography.contains("Shared @ note"));
        assert!(a.events.iter().any(|e| e.name == "Engagement" && e.date == "1990-03-03"));
        assert!(who(&db, "Sam").events.iter().any(|e| e.name == "Engagement"));
        let u = db.unions.values().next().unwrap();
        assert_eq!((u.date.as_str(), u.end_date.as_str(), u.end_cause.as_str()), ("1990..1991", "1995", end_cause::DIVORCED));
        assert_eq!(who(&db, "Kid").mother_rel, ParentRel::Adopted);
    }

    #[test]
    fn export_is_standard() {
        let c = Ctx::new(0, NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
        let mut db = run("0 HEAD\n0 @I1@ INDI\n1 NAME Anna /Nowak/\n1 SEX F\n1 DEAT Y\n1 BAPM\n2 DATE 2 MAY 1900\n1 TITL hrabina\n1 NOTE a\n0 TRLR\n");
        let id = who(&db, "Anna").id;
        let mut a = db.persons[&id].clone();
        a.maiden_name = "Kowalska".into();
        a.notes = "<p>Wolne notatki</p>".into();
        a.research = "<p>Podobno z Wilna</p>".into();
        a.residence = "ul. Długa 5, Gdańsk".into();
        a.biography = format!("<p>{}</p>", "słowo ".repeat(80));
        a.occupation = "ab@cd".into();
        db.put_person(a);
        let mut o = ExportOptions::default();
        o.event_names.insert("BAPM".into(), "Chrzest".into());
        let out = export(&db, &c, &o);
        assert!(out.contains("1 SUBM @U1@") && out.contains("0 @U1@ SUBM"));
        assert!(out.contains("1 NAME Anna /Kowalska/\n2 TYPE birth"));
        assert!(out.contains("1 NAME Anna /Nowak/\n2 TYPE married"));
        assert!(!out.contains("_MARNM"));
        assert!(out.contains("1 DEAT Y"));
        assert!(out.contains("1 BAPM\n2 DATE 2 MAY 1900"));
        assert!(out.contains("1 TITL hrabina"));
        assert!(out.contains("1 OCCU ab@@cd"));
        assert!(out.lines().all(|l| l.chars().count() <= 255));
        assert!(out.contains("2 CONC "));
        for l in out.lines().filter(|l| l.starts_with("2 CONC")) {
            assert!(!l.ends_with(' ') && !l["2 CONC ".len()..].starts_with(' '), "{l}");
        }
        // and back: same person, names, notes
        let mut db2 = Database::new(0);
        import(&mut db2, &c, &out, &ImportOptions::default()).unwrap();
        let b = who(&db2, "Anna");
        assert_eq!((b.last_name.as_str(), b.maiden_name.as_str()), ("Nowak", "Kowalska"));
        assert_eq!(b.residence, "ul. Długa 5, Gdańsk");
        assert!(b.research.contains("Podobno z Wilna") && !b.biography.contains("Wilna"));
        assert!(!b.events.iter().any(|e| e.name == "Residence"));
        assert!(b.deceased);
        assert!(b.notes.contains("Wolne notatki") && !b.biography.contains("Wolne"));
        assert_eq!(strip_tags(&b.biography).trim(), "słowo ".repeat(80).trim());
        assert_eq!(b.occupation, "ab@cd");
        assert!(b.events.iter().any(|e| e.name == "Baptism" && e.date == "1900-05-02"));
        assert!(b.events.iter().any(|e| e.name == "Title" && e.description.contains("hrabina")));
    }
}
