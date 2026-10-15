//! Data model of a MyRoots family tree.
//!
//! Unlike the WordPress plugin (which stored every relation twice: `parent_*`
//! on the child plus `children[]` on the parent, `spouses[]` on both partners),
//! relations are stored exactly once here:
//!
//! * a child points to its mother and father (`Person::mother`, `Person::father`);
//!   lists of children are derived,
//! * a marriage / partnership is a separate `Union` record shared by both partners.
//!
//! This removes the whole class of "one-sided relation" inconsistencies.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Id = u64;

/* ------------------------------------------------------------------ */
/*  Enumerations                                                        */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Gender {
    #[serde(rename = "M")]
    Male,
    #[serde(rename = "F")]
    Female,
    #[serde(rename = "U")]
    Unknown,
    #[default]
    #[serde(rename = "")]
    Unset,
}

impl Gender {
    pub fn from_code(s: &str) -> Gender {
        match s.trim().to_ascii_uppercase().as_str() {
            "M" => Gender::Male,
            "F" => Gender::Female,
            "U" => Gender::Unknown,
            _ => Gender::Unset,
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Gender::Male => "M",
            Gender::Female => "F",
            Gender::Unknown => "U",
            Gender::Unset => "",
        }
    }
    pub fn is_known(self) -> bool {
        matches!(self, Gender::Male | Gender::Female)
    }
}

/// Kind of the parent-child relation.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum ParentRel {
    #[default]
    Bio,
    Adopted,
    Foster,
    Step,
}

impl ParentRel {
    pub fn parse(s: &str) -> ParentRel {
        match s.trim().to_ascii_lowercase().as_str() {
            "adopted" | "adoptive" | "adoption" => ParentRel::Adopted,
            "foster" => ParentRel::Foster,
            "step" | "stepchild" | "step-child" => ParentRel::Step,
            _ => ParentRel::Bio,
        }
    }
}

/// Publication status (kept from the WordPress posts).
/// Only `Published` records end up in the exported family website.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Published,
    Draft,
    Private,
}

impl Status {
    pub fn from_wp(s: &str) -> Status {
        match s {
            "draft" | "pending" | "auto-draft" => Status::Draft,
            "private" => Status::Private,
            _ => Status::Published,
        }
    }
}

/* ------------------------------------------------------------------ */
/*  Person                                                              */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct LifeEvent {
    pub name: String,
    /// Qualified date (see `date` module).
    pub date: String,
    pub place: String,
    /// Rich text (sanitized HTML subset).
    pub description: String,
    pub image: Option<Id>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct GalleryItem {
    pub media: Id,
    pub caption: String,
}

/// A voice or sound recording of a person (audio media).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Recording {
    pub media: Id,
    pub title: String,
    /// When it was recorded (qualified date, may be empty).
    pub date: String,
}

/// One photo of the "Through the years" animation.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct AgingFrame {
    pub media: Id,
    /// When the photo was taken (qualified date; a year is enough).
    pub date: String,
    /// Centres of the eyes, as fractions of the image size: left x, left y,
    /// right x, right y (left/right as seen on the photo). None = not set.
    pub eyes: Option<[f32; 4]>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Person {
    pub id: Id,
    pub status: Status,

    pub first_name: String,
    pub last_name: String,
    pub maiden_name: String,
    pub gender: Gender,

    pub birth_date: String,
    pub birth_place: String,
    pub death_date: String,
    pub death_place: String,
    pub death_cause: String,
    /// Known to have died although the date is unknown (GEDCOM `1 DEAT Y`).
    pub deceased: bool,

    /// Rich text (sanitized HTML subset).
    pub biography: String,
    /// Free content (the body of the WordPress post), rich text.
    pub notes: String,

    pub occupation: String,
    pub education: String,
    pub faith: String,
    pub height: String,
    pub eye_color: String,
    pub blood_type: String,

    pub mother: Option<Id>,
    pub mother_rel: ParentRel,
    pub father: Option<Id>,
    pub father_rel: ParentRel,

    pub events: Vec<LifeEvent>,
    pub clans: Vec<String>,

    pub photo: Option<Id>,
    pub gallery: Vec<GalleryItem>,
    /// Voice recordings.
    pub recordings: Vec<Recording>,
    /// "Through the years": photos from different periods, aligned on the eyes.
    pub aging: Vec<AgingFrame>,
    /// Recording played along with the animation (one of `recordings`).
    pub aging_audio: Option<Id>,

    /// Custom fields: group id → field id → value.
    pub custom: BTreeMap<String, BTreeMap<String, String>>,
    /// Source citations pinned to facts: field key → source ids.
    pub citations: BTreeMap<String, Vec<Id>>,

    /// Unix time in milliseconds.
    pub created: i64,
    pub updated: i64,
}

impl Person {
    pub fn full_name(&self) -> String {
        let n = format!("{} {}", self.first_name.trim(), self.last_name.trim());
        let n = n.trim().to_string();
        if n.is_empty() {
            format!("#{}", self.id)
        } else {
            n
        }
    }

    pub fn parent(&self, role: ParentRole) -> Option<Id> {
        match role {
            ParentRole::Mother => self.mother,
            ParentRole::Father => self.father,
        }
    }

    pub fn parent_rel(&self, role: ParentRole) -> ParentRel {
        match role {
            ParentRole::Mother => self.mother_rel,
            ParentRole::Father => self.father_rel,
        }
    }

    pub fn set_parent(&mut self, role: ParentRole, id: Option<Id>, rel: ParentRel) {
        match role {
            ParentRole::Mother => {
                self.mother = id;
                self.mother_rel = if id.is_some() { rel } else { ParentRel::Bio };
            }
            ParentRole::Father => {
                self.father = id;
                self.father_rel = if id.is_some() { rel } else { ParentRel::Bio };
            }
        }
    }

    pub fn parents(&self) -> impl Iterator<Item = Id> + '_ {
        self.father.into_iter().chain(self.mother)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ParentRole {
    Mother,
    Father,
}

impl ParentRole {
    /// Natural parent slot for a person of the given gender.
    pub fn for_gender(g: Gender) -> ParentRole {
        if g == Gender::Female {
            ParentRole::Mother
        } else {
            ParentRole::Father
        }
    }
    pub fn other(self) -> ParentRole {
        match self {
            ParentRole::Mother => ParentRole::Father,
            ParentRole::Father => ParentRole::Mother,
        }
    }
}

/* ------------------------------------------------------------------ */
/*  Union (marriage / partnership)                                      */
/* ------------------------------------------------------------------ */

/// Fixed keys of how a union ended. Unknown legacy values are kept verbatim.
pub mod end_cause {
    pub const ONGOING: &str = "ongoing";
    pub const WIDOWED: &str = "widowed";
    pub const DIVORCED: &str = "divorced";
    pub const OTHER: &str = "other";

    /// Normalizes legacy WordPress values ("Trwa", "Rozwód", …) and GEDCOM words.
    pub fn normalize(v: &str) -> String {
        let t = v.trim();
        let l = t.to_lowercase();
        match l.as_str() {
            "" | "trwa" | "ongoing" => ONGOING.into(),
            "śmierć małżonka" | "widowed" | "death of spouse" => WIDOWED.into(),
            "rozwód" | "divorce" | "divorced" => DIVORCED.into(),
            "inne/nieznane" | "other" | "other/unknown" | "unknown" => OTHER.into(),
            _ => t.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Union {
    pub id: Id,
    /// First partner (always a person in the tree).
    pub a: Id,
    /// Second partner, if recorded as a person in the tree.
    pub b: Option<Id>,
    /// Name of the second partner when they are not in the tree.
    pub partner_first_name: String,
    pub partner_last_name: String,

    pub date: String,
    pub place: String,
    /// One of `end_cause::*` or a legacy free-text value.
    pub end_cause: String,
    pub end_date: String,
    /// Informal relationship (not a marriage).
    pub informal: bool,
}

impl Union {
    pub fn involves(&self, id: Id) -> bool {
        self.a == id || self.b == Some(id)
    }
    /// The partner of `id` in this union (None when unknown / external).
    pub fn partner_of(&self, id: Id) -> Option<Id> {
        if self.a == id {
            self.b
        } else if self.b == Some(id) {
            Some(self.a)
        } else {
            None
        }
    }
    pub fn has_ended(&self) -> bool {
        !self.end_date.trim().is_empty()
            || (!self.end_cause.is_empty() && self.end_cause != end_cause::ONGOING)
    }
    fn filled(&self) -> usize {
        [&self.date, &self.place, &self.end_date, &self.end_cause]
            .iter()
            .filter(|s| !s.trim().is_empty() && s.as_str() != end_cause::ONGOING)
            .count()
    }
    /// Merges details of another record of the same couple (keeps filled values).
    pub fn absorb(&mut self, other: &Union) {
        let richer = other.filled() > self.filled();
        macro_rules! take {
            ($f:ident) => {
                if self.$f.trim().is_empty() || (richer && !other.$f.trim().is_empty()) {
                    if !other.$f.trim().is_empty() {
                        self.$f = other.$f.clone();
                    }
                }
            };
        }
        take!(date);
        take!(place);
        take!(end_date);
        if (self.end_cause.is_empty() || self.end_cause == end_cause::ONGOING)
            && !other.end_cause.is_empty()
        {
            self.end_cause = other.end_cause.clone();
        }
        self.informal = self.informal && other.informal;
    }
}

/* ------------------------------------------------------------------ */
/*  Pets                                                                */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Pet {
    pub id: Id,
    pub status: Status,
    pub name: String,
    /// Key: dog, cat, bird, horse, other (or a custom value).
    pub species: String,
    pub breed: String,
    pub gender: Gender,
    pub birth_date: String,
    pub death_date: String,
    pub owner: Option<Id>,
    pub biography: String,
    pub notes: String,
    pub photo: Option<Id>,
    pub gallery: Vec<GalleryItem>,
    pub created: i64,
    pub updated: i64,
}

/* ------------------------------------------------------------------ */
/*  Sources                                                             */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Source {
    pub id: Id,
    pub title: String,
    pub date: String,
    pub url: String,
    pub notes: String,
    /// Link to an audio / video recording.
    pub media_url: String,
    /// Cover image.
    pub photo: Option<Id>,
    /// Document scans.
    pub scans: Vec<Id>,
    /// General links between the source and people.
    pub persons: Vec<Id>,
    pub created: i64,
    pub updated: i64,
}

/* ------------------------------------------------------------------ */
/*  Media                                                               */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Media {
    pub id: Id,
    pub file_name: String,
    pub mime: String,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub has_thumb: bool,
    pub caption: String,
    /// Original location (e.g. the WordPress attachment URL) — informational.
    pub origin: String,
    pub created: i64,
}

/* ------------------------------------------------------------------ */
/*  Places (geocoding cache + manual coordinates)                       */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Place {
    pub id: Id,
    pub name: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    /// "nominatim" | "manual" | "gedcom" | "failed"
    pub origin: String,
    pub updated: i64,
}

/* ------------------------------------------------------------------ */
/*  History (person revisions)                                          */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Revision {
    pub id: Id,
    pub person: Id,
    pub time: i64,
    /// Keys of the fields that changed.
    pub fields: Vec<String>,
    /// Full snapshot of the PREVIOUS state of the person.
    pub data: Person,
    /// Snapshot of the person's unions at that time (relations can be restored too).
    pub unions: Vec<Union>,
}

/* ------------------------------------------------------------------ */
/*  Correction suggestions from family members                          */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Suggestion {
    pub id: Id,
    pub person: Option<Id>,
    pub person_name: String,
    pub message: String,
    pub author: String,
    pub email: String,
    /// "new" | "done" | "rejected"
    pub status: String,
    pub created: i64,
    /// Opaque token from the family website (deduplication of re-imports).
    pub token: String,
}

/* ------------------------------------------------------------------ */
/*  Settings                                                            */
/* ------------------------------------------------------------------ */

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct PrivacySettings {
    /// Privacy applies to the exported family website.
    pub enabled: bool,
    /// "full" | "hide_living" | "censor_living"
    pub level: String,
    /// Sections locked at the "full" level: profile, list, tree, map, stats, gallery.
    pub sections: Vec<String>,
    /// Censored fields at "censor_living": dates, places, bio_events, photos_sensitive.
    pub fields: Vec<String>,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        PrivacySettings {
            enabled: true,
            level: "censor_living".into(),
            sections: crate::privacy::SECTIONS.iter().map(|s| s.to_string()).collect(),
            fields: crate::privacy::FIELDS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Appearance {
    pub primary: String,
    pub accent: String,
    pub bg: String,
    pub text: String,
    pub border: String,
    pub font: String,
    pub font_size: String,
    pub radius: String,
    pub shadow: String,
    /// "light" | "dark" | "" (follow the system)
    pub theme: String,
    /// Map tiles style: "osm" | "light" | "dark" | "topo"
    pub map_style: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct CustomField {
    pub id: String,
    pub name: String,
    /// text | textarea | number | date | url | email | select
    #[serde(rename = "type")]
    pub kind: String,
    pub options: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct CustomGroup {
    pub id: String,
    pub name: String,
    pub fields: Vec<CustomField>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub days_before: u32,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        NotificationSettings { enabled: true, days_before: 14 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct SuggestionSettings {
    /// Show the "Suggest a correction" form on the family website.
    pub enabled: bool,
    /// Where suggestions go: an e-mail address (mailto) …
    pub email: String,
    /// … and/or an HTTPS endpoint accepting a JSON POST (e.g. a form service).
    pub endpoint: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct SiteSettings {
    pub title: String,
    pub subtitle: String,
    /// Welcome text on the home page of the family website (rich text).
    pub intro: String,
    /// Default language of the website ("" = visitor's browser language).
    pub language: String,
    /// Include the original-size photos (otherwise only reduced copies).
    pub full_photos: bool,
}

impl Default for SiteSettings {
    fn default() -> Self {
        SiteSettings { title: String::new(), subtitle: String::new(), intro: String::new(), language: String::new(), full_photos: true }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub tree_name: String,
    /// Presumed death: a person older than this without a death date is
    /// treated as deceased (0 = off).
    pub max_age: u32,
    pub privacy: PrivacySettings,
    /// Opt-in: OpenStreetMap tiles and Nominatim geocoding.
    pub map_external: bool,
    pub appearance: Appearance,
    pub custom_schema: Vec<CustomGroup>,
    /// Clans defined manually (in addition to the ones used by people).
    pub clans: Vec<String>,
    /// People list order: "name_asc" | "name_desc" | "newest" | "birth_asc" | "birth_desc"
    pub sort_order: String,
    pub people_per_page: u32,
    /// "person" | "oldest" | "youngest" | "last"
    pub tree_start_mode: String,
    pub tree_start_person: Option<Id>,
    /// Dismissed duplicate pairs ("min:max").
    pub merge_dismissed: Vec<String>,
    pub notifications: NotificationSettings,
    pub suggestions: SuggestionSettings,
    pub site: SiteSettings,
    /// UI language ("" = system).
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            tree_name: String::new(),
            max_age: 110,
            privacy: PrivacySettings::default(),
            map_external: false,
            appearance: Appearance::default(),
            custom_schema: Vec::new(),
            clans: Vec::new(),
            sort_order: "name_asc".into(),
            people_per_page: 50,
            tree_start_mode: "person".into(),
            tree_start_person: None,
            merge_dismissed: Vec::new(),
            notifications: NotificationSettings::default(),
            suggestions: SuggestionSettings::default(),
            site: SiteSettings::default(),
            language: String::new(),
        }
    }
}

/* ------------------------------------------------------------------ */
/*  Helpers                                                             */
/* ------------------------------------------------------------------ */

/// Fields that can be pinned to sources (citations).
pub const CITABLE_FIELDS: &[&str] = &[
    "birth_date",
    "birth_place",
    "death_date",
    "death_place",
    "occupation",
    "faith",
    "education",
    "biography",
];

/// Strips HTML tags and decodes the most common entities (for plain-text use).
pub fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag = String::new();
    for c in html.chars() {
        if in_tag {
            if c == '>' {
                in_tag = false;
                let t = tag.trim_start_matches('/').to_ascii_lowercase();
                let name: String = t.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
                if matches!(name.as_str(), "p" | "br" | "div" | "li" | "h1" | "h2" | "h3" | "h4" | "tr")
                    && !out.ends_with('\n')
                    && !out.is_empty()
                {
                    out.push('\n');
                }
                tag.clear();
            } else {
                tag.push(c);
            }
        } else if c == '<' {
            in_tag = true;
        } else {
            out.push(c);
        }
    }
    let out = out
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    out.lines().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n").trim().to_string()
}

/// Folds Latin letters with diacritics to ASCII (ł → l, ż → z, ß → ss …).
pub fn fold_char(c: char, out: &mut String) {
    let r: &str = match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'ǎ' => "a",
        'æ' => "ae",
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c",
        'ď' | 'đ' | 'ð' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => "e",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g",
        'ĥ' | 'ħ' => "h",
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => "i",
        'ĵ' => "j",
        'ķ' => "k",
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l",
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => "o",
        'œ' => "oe",
        'ŕ' | 'ŗ' | 'ř' => "r",
        'ś' | 'ŝ' | 'ş' | 'š' | 'ș' => "s",
        'ß' => "ss",
        'ţ' | 'ť' | 'ŧ' | 'ț' => "t",
        'þ' => "th",
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u",
        'ŵ' => "w",
        'ý' | 'ÿ' | 'ŷ' => "y",
        'ź' | 'ż' | 'ž' => "z",
        _ => {
            out.push(c);
            return;
        }
    };
    out.push_str(r);
}

/// Lowercase, Latin accents folded, only letters, digits and single spaces
/// kept (comparisons, search, sorting). Non-Latin scripts are preserved.
/// A link that is safe to open: http(s) or mailto only ("www.x.pl" becomes
/// https://www.x.pl); anything else (javascript:, data:, file: …) is dropped.
pub fn safe_url(s: &str) -> String {
    let t = s.trim();
    if t.is_empty() {
        return String::new();
    }
    let lower = t.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:") {
        return t.chars().filter(|c| !c.is_control()).collect();
    }
    if !lower.contains(':') && !t.contains(char::is_whitespace) && t.contains('.') && !t.starts_with('.') && !t.starts_with('/') {
        return format!("https://{t}");
    }
    String::new()
}

/// Media types the app stores and serves; anything else is plain bytes
/// (never served as a page or a script).
pub fn safe_mime(m: &str) -> String {
    let m = m.trim().to_ascii_lowercase();
    let base = m.split(';').next().unwrap_or("").trim();
    let ok = (base.starts_with("image/") && base != "image/svg+xml") || base.starts_with("audio/") || base.starts_with("video/") || base == "application/pdf";
    if ok && base.chars().all(|c| c.is_ascii_alphanumeric() || "/.+-".contains(c)) {
        base.to_string()
    } else {
        "application/octet-stream".into()
    }
}

pub fn norm(s: &str) -> String {
    let mut folded = String::with_capacity(s.len());
    for c in s.trim().chars().flat_map(|c| c.to_lowercase()) {
        fold_char(c, &mut folded);
    }
    folded
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip() {
        assert_eq!(strip_tags("<p>Ala &amp; kot</p><p>B</p>"), "Ala & kot\nB");
    }

    #[test]
    fn normalize() {
        assert_eq!(norm("  Łukasz-Żółć "), "lukaszzolc");
        assert_eq!(norm("שָׁלוֹם Иван"), norm("שָׁלוֹם Иван"));
        assert_eq!(norm("Иван"), "иван");
        assert_eq!(end_cause::normalize("Rozwód"), "divorced");
        assert_eq!(end_cause::normalize("Śmierć małżonka"), "widowed");
    }
}
