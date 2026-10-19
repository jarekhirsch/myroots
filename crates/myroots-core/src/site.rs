//! Export of the family website: a static, read-only copy of the app
//! (works on any web hosting and even opened straight from a disk).
//!
//! Privacy is applied HERE, before anything leaves the device: the website
//! contains only published records, anonymous placeholders instead of hidden
//! people and censored copies of living people.

use crate::archive::MediaSource;
use crate::db::{Database, Kind, Record};
use crate::error::{Error, Result};
use crate::life::Ctx;
use crate::model::*;
use crate::privacy::Policy;
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};
use std::io::{Seek, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Public copy of the database.
pub fn public_db(db: &Database, ctx: &Ctx) -> Database {
    let pol = Policy::public(&db.settings, ctx.today);
    let mut out = Database::default();
    let mut hidden = HashSet::new();
    for p in db.persons.values() {
        if p.status != Status::Published {
            continue;
        }
        if pol.should_hide(p) {
            hidden.insert(p.id);
            out.persons.insert(
                p.id,
                Person { id: p.id, mother: p.mother, father: p.father, mother_rel: p.mother_rel, father_rel: p.father_rel, ..Default::default() },
            );
            continue;
        }
        let (mut r, _) = pol.redact(p);
        if pol.photo_hidden(p) {
            r.photo = None;
            r.gallery.clear();
            r.aging.clear();
            r.recordings.clear();
            r.aging_audio = None;
        }
        r.created = 0;
        out.persons.insert(p.id, r);
    }
    // Drop parent links to people that are not exported.
    let ids: HashSet<Id> = out.persons.keys().copied().collect();
    for p in out.persons.values_mut() {
        if p.mother.map(|m| !ids.contains(&m)).unwrap_or(false) {
            p.mother = None;
        }
        if p.father.map(|m| !ids.contains(&m)).unwrap_or(false) {
            p.father = None;
        }
    }
    for u in db.unions.values() {
        if !ids.contains(&u.a) || u.b.map(|b| !ids.contains(&b)).unwrap_or(false) {
            continue;
        }
        let a_l = pol.is_living(&db.persons[&u.a]);
        let b_l = u.b.and_then(|b| db.persons.get(&b)).map(|p| pol.is_living(p)).unwrap_or(false);
        let mut r = pol.redact_union(u, a_l, b_l);
        if hidden.contains(&u.a) || u.b.map(|b| hidden.contains(&b)).unwrap_or(false) {
            r.date.clear();
            r.place.clear();
            r.end_date.clear();
            if u.b.is_none() {
                r.partner_first_name.clear();
                r.partner_last_name.clear();
            }
        }
        out.unions.insert(u.id, r);
    }
    for p in db.pets.values() {
        if p.status != Status::Published {
            continue;
        }
        let mut p = p.clone();
        if p.owner.map(|o| !ids.contains(&o) || hidden.contains(&o)).unwrap_or(false) {
            p.owner = None;
        }
        p.created = 0;
        out.pets.insert(p.id, p);
    }
    // Sources used by visible people.
    let mut used_sources: HashSet<Id> = HashSet::new();
    for p in out.persons.values() {
        for v in p.citations.values() {
            used_sources.extend(v.iter().copied());
        }
    }
    for s in db.sources.values() {
        let visible_links: Vec<Id> = s.persons.iter().copied().filter(|p| ids.contains(p) && !hidden.contains(p)).collect();
        if visible_links.is_empty() && !used_sources.contains(&s.id) && pol.active {
            continue;
        }
        let mut s = s.clone();
        s.persons = visible_links;
        s.created = 0;
        out.sources.insert(s.id, s);
    }
    for p in out.persons.values_mut() {
        for v in p.citations.values_mut() {
            v.retain(|s| db.sources.contains_key(s));
        }
    }
    // Media referenced by exported data.
    let mut media: BTreeSet<Id> = BTreeSet::new();
    for p in out.persons.values() {
        media.extend(p.photo);
        media.extend(p.gallery.iter().map(|g| g.media));
        media.extend(p.events.iter().filter_map(|e| e.image));
        media.extend(p.recordings.iter().map(|r| r.media));
        media.extend(p.aging.iter().map(|f| f.media));
    }
    for p in out.pets.values() {
        media.extend(p.photo);
        media.extend(p.gallery.iter().map(|g| g.media));
    }
    for s in out.sources.values() {
        media.extend(s.photo);
        media.extend(s.scans.iter().copied());
    }
    // People marked on photos: only the ones shown with their photos.
    let markable: HashSet<Id> = db.persons.values().filter(|p| ids.contains(&p.id) && !hidden.contains(&p.id) && !pol.photo_hidden(p)).map(|p| p.id).collect();
    for m in db.media.values() {
        if m.tags.iter().any(|t| t.person.map(|p| markable.contains(&p)).unwrap_or(false)) {
            media.insert(m.id);
        }
    }
    for m in media {
        if let Some(md) = db.media.get(&m) {
            let mut md = md.clone();
            md.origin.clear();
            md.tags.retain(|t| t.person.map(|p| markable.contains(&p)).unwrap_or(true));
            out.media.insert(m, md);
        }
    }
    // Places with coordinates (unless the map is locked or disabled).
    if db.settings.map_external && !pol.blocks_section("map") {
        let used: HashSet<String> = crate::query::places(&out).into_iter().map(|p| p.name.to_lowercase()).collect();
        // weather only for days that are shown (it would reveal hidden dates)
        let mut days: HashSet<(String, String)> = HashSet::new();
        let mut day = |place: &str, d: &str| {
            if let Some(x) = crate::date::exact_day(d) {
                days.insert((place.trim().to_lowercase(), x.format("%Y-%m-%d").to_string()));
            }
        };
        for p in out.persons.values() {
            day(&p.birth_place, &p.birth_date);
            day(&p.death_place, &p.death_date);
            for e in &p.events {
                day(&e.place, &e.date);
            }
        }
        for u in out.unions.values() {
            day(&u.place, &u.date);
        }
        for pl in db.places.values() {
            if pl.lat.is_some() && used.contains(&pl.name.trim().to_lowercase()) {
                let mut pl = pl.clone();
                let key = pl.name.trim().to_lowercase();
                pl.weather.retain(|d, _| days.contains(&(key.clone(), d.clone())));
                out.places.insert(pl.id, pl);
            }
        }
    }
    let mut s = db.settings.clone();
    s.merge_dismissed.clear();
    s.notifications = NotificationSettings::default();
    if !s.map_external || pol.blocks_section("map") {
        s.map_external = false;
    }
    s.tree_start_person = s.tree_start_person.filter(|p| ids.contains(p) && !hidden.contains(p));
    // the owner's e-mail / form address only when the suggestion form is on
    if !s.suggestions.enabled {
        s.suggestions = Default::default();
    }
    if !s.suggestions.endpoint.trim().to_ascii_lowercase().starts_with("https://") {
        s.suggestions.endpoint.clear();
    }
    out.settings = s;
    out.meta.version = db.meta.version;
    out.meta.next_id = db.meta.next_id;
    out
}

#[derive(Serialize, Debug)]
pub struct SiteStats {
    pub persons: usize,
    pub hidden: usize,
    pub media: usize,
}

/// Contents of `data.js` (works when the website is opened from a disk).
pub fn data_js(db: &Database, ctx: &Ctx, app: &str) -> String {
    #[derive(Serialize)]
    struct Payload<'a> {
        app: &'a str,
        generated: i64,
        today: String,
        records: Vec<Record>,
    }
    let p = Payload { app, generated: ctx.now, today: ctx.today.format("%Y-%m-%d").to_string(), records: db.all_records().into_iter().filter(|r| r.kind != Kind::Revision && r.kind != Kind::Suggestion).collect() };
    // `</` is escaped so the JSON can never close a <script> element.
    let json = serde_json::to_string(&p).unwrap().replace("</", "<\\/");
    format!("window.MYROOTS_SITE={json};\n")
}

/// Writes the website ZIP: viewer assets + data.js + media.
/// `viewer` = (relative path, bytes) of the prebuilt read-only app.
pub fn write<W: Write + Seek>(w: W, db: &Database, ctx: &Ctx, viewer: &[(String, Vec<u8>)], media: &mut dyn MediaSource, app: &str) -> Result<(W, SiteStats)> {
    if viewer.is_empty() {
        return Err(Error::new("site_no_viewer", "the website template is missing"));
    }
    let pubdb = public_db(db, ctx);
    let mut z = ZipWriter::new(w);
    let deflate = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let ze = |e: zip::result::ZipError| Error::new("archive", e.to_string());
    let ie = |e: std::io::Error| Error::new("archive", e.to_string());
    for (path, bytes) in viewer {
        z.start_file(path.as_str(), deflate).map_err(ze)?;
        z.write_all(bytes).map_err(ie)?;
    }
    z.start_file("data.js", deflate).map_err(ze)?;
    z.write_all(data_js(&pubdb, ctx, app).as_bytes()).map_err(ie)?;
    let full = db.settings.site.full_photos;
    let mut count = 0;
    for (id, m) in &pubdb.media {
        let ext = crate::gedcom::ext_for_mime(&m.mime);
        if !m.mime.starts_with("image/") {
            // sound and documents: the file itself, no thumbnail
            if let Some(o) = media.original(*id) {
                z.start_file(format!("media/{id}.{ext}"), stored).map_err(ze)?;
                z.write_all(&o).map_err(ie)?;
                count += 1;
            }
            continue;
        }
        let mut thumb = media.thumb(*id);
        if thumb.is_none() {
            // no thumbnail stored: the original serves as one (browsers sniff the format)
            thumb = media.original(*id);
        }
        let orig = if full || thumb.is_none() { media.original(*id) } else { None };
        // no GPS position, camera or other hidden data on the public website
        let orig = orig.map(|o| crate::metadata::strip(&o, &m.mime));
        let thumb = thumb.map(|t| crate::metadata::strip(&t, "image/jpeg"));
        if let Some(t) = &thumb {
            z.start_file(format!("media/{id}_t.jpg"), stored).map_err(ze)?;
            z.write_all(t).map_err(ie)?;
        }
        // Without full photos the "original" is the thumbnail-sized copy.
        match orig {
            Some(o) => {
                z.start_file(format!("media/{id}.{ext}"), stored).map_err(ze)?;
                z.write_all(&o).map_err(ie)?;
            }
            None => {
                if let Some(t) = thumb {
                    z.start_file(format!("media/{id}.{ext}"), stored).map_err(ze)?;
                    z.write_all(&t).map_err(ie)?;
                }
            }
        }
        count += 1;
    }
    let hidden = pubdb.persons.values().filter(|p| p.first_name.is_empty() && p.last_name.is_empty() && p.birth_date.is_empty()).count();
    let stats = SiteStats { persons: pubdb.persons.len(), hidden, media: count };
    let w = z.finish().map_err(ze)?;
    Ok((w, stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops;
    use chrono::NaiveDate;

    #[test]
    fn privacy_applied() {
        let c = Ctx::new(1, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let mut db = Database::new(0);
        let old = ops::create_person(&mut db, &c, Person { first_name: "Old".into(), birth_date: "1900".into(), death_date: "1980".into(), ..Default::default() }, Default::default()).unwrap();
        let kid = ops::create_person(&mut db, &c, Person { first_name: "Kid".into(), birth_date: "1990-02-03".into(), birth_place: "X".into(), father: Some(old), ..Default::default() }, Default::default()).unwrap();
        let draft = ops::create_person(&mut db, &c, Person { first_name: "Draft".into(), status: Status::Draft, ..Default::default() }, Default::default()).unwrap();
        let mut s = db.settings.clone();
        s.privacy.enabled = true;
        s.privacy.level = "hide_living".into();
        ops::save_settings(&mut db, s);
        let p = public_db(&db, &c);
        assert_eq!(p.persons[&kid].first_name, "");
        assert_eq!(p.persons[&kid].father, Some(old));
        assert!(!p.persons.contains_key(&draft));
        let mut s = db.settings.clone();
        s.privacy.level = "censor_living".into();
        ops::save_settings(&mut db, s);
        let p = public_db(&db, &c);
        assert_eq!(p.persons[&kid].first_name, "Kid");
        assert_eq!(p.persons[&kid].birth_date, "1990");
        assert_eq!(p.persons[&kid].birth_place, "");
        let js = data_js(&p, &c, "t");
        assert!(js.starts_with("window.MYROOTS_SITE="));
        // voice and photos of the living stay private; the owner's e-mail is not published while the form is off
        let m = ops::register_media(&mut db, &c, Media { file_name: "v.mp3".into(), mime: "audio/mpeg".into(), ..Default::default() });
        let mut k = db.persons[&kid].clone();
        k.recordings = vec![Recording { media: m, title: "Głos".into(), date: String::new() }];
        k.aging = vec![AgingFrame { media: m, date: "2000".into(), eyes: None }];
        db.put_person(k);
        let mut s = db.settings.clone();
        s.suggestions.enabled = false;
        s.suggestions.email = "owner@example.com".into();
        s.suggestions.endpoint = "javascript:alert(1)".into();
        ops::save_settings(&mut db, s);
        assert_eq!(db.settings.suggestions.endpoint, "");
        let p = public_db(&db, &c);
        assert!(p.persons[&kid].recordings.is_empty() && p.persons[&kid].aging.is_empty());
        assert!(!p.media.contains_key(&m));
        assert!(!data_js(&p, &c, "t").contains("owner@example.com"));
    }

    #[test]
    fn photo_tags_and_weather() {
        let c = Ctx::new(1, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let mut db = Database::new(0);
        let gran = ops::create_person(&mut db, &c, Person { first_name: "Gran".into(), birth_date: "1920-05-04".into(), birth_place: "Gdańsk".into(), death_date: "1990".into(), ..Default::default() }, Default::default()).unwrap();
        let kid = ops::create_person(&mut db, &c, Person { first_name: "Kid".into(), birth_date: "1990-02-03".into(), birth_place: "Gdańsk".into(), ..Default::default() }, Default::default()).unwrap();
        let other = ops::create_person(&mut db, &c, Person { first_name: "Other".into(), birth_date: "1921".into(), death_date: "1999".into(), ..Default::default() }, Default::default()).unwrap();
        let m = ops::register_media(&mut db, &c, Media { file_name: "g.jpg".into(), mime: "image/jpeg".into(), ..Default::default() });
        let tags = vec![
            PhotoTag { person: Some(gran), name: String::new(), rect: [0.1, 0.1, 0.2, 0.2] },
            PhotoTag { person: Some(kid), name: String::new(), rect: [0.5, 0.1, 0.2, 0.2] },
            PhotoTag { person: Some(gran), name: String::new(), rect: [0.0, 0.0, 0.1, 0.1] },
            PhotoTag { person: None, name: "  Ciocia Ala ".into(), rect: [0.8, 0.5, 0.5, 0.9] },
            PhotoTag { person: None, name: String::new(), rect: [0.1, 0.1, 0.2, 0.2] },
            PhotoTag { person: Some(9999), name: "Nobody".into(), rect: [0.1, 0.1, 0.0, 0.2] },
        ];
        let t = ops::set_media_tags(&mut db, m, &tags).unwrap();
        assert_eq!(t.len(), 3);
        assert_eq!(t[2].name, "Ciocia Ala");
        assert!((t[2].rect[2] - 0.2).abs() < 1e-6 && (t[2].rect[3] - 0.5).abs() < 1e-6);
        // the photo shows up on the profile and counts as used
        let pol = Policy::owner(110, c.today);
        let prof = crate::query::profile(&db, &c, gran, &pol).unwrap();
        assert_eq!(prof.tagged.len(), 1);
        assert!(!ops::unused_media(&db).contains(&m));
        // merging keeps the marks
        let sel = crate::merge::Selection { gallery: vec![], ..Default::default() };
        crate::merge::execute(&mut db, &c, gran, other, gran, sel).unwrap();
        assert!(db.media[&m].tags.iter().any(|t| t.person == Some(gran)));

        // weather: stored per day, cleared when the place moves
        ops::set_place_coords(&mut db, &c, "Gdańsk", Some(54.35), Some(18.65), "manual");
        let w = DayWeather { code: 3, tmax: Some(18.0), tmin: Some(9.5), rain: Some(1.2), snow: None };
        ops::set_place_weather(&mut db, "gdańsk", "1920-05-04", w.clone()).unwrap();
        ops::set_place_weather(&mut db, "Gdańsk", "1990-02-03", w.clone()).unwrap();
        assert!(ops::set_place_weather(&mut db, "Gdańsk", "1990-02", w.clone()).is_err());
        assert!(ops::set_place_weather(&mut db, "Nowhere", "1990-02-03", w.clone()).is_err());
        assert_eq!(crate::query::places(&db).iter().find(|p| p.name == "Gdańsk").unwrap().weather.len(), 2);

        // family website: the living child is censored, so neither the mark nor the weather of the birthday is published
        let mut s = db.settings.clone();
        s.map_external = true;
        s.privacy.enabled = true;
        s.privacy.level = "censor_living".into();
        s.privacy.fields = vec!["dates".into(), "photos_sensitive".into()];
        ops::save_settings(&mut db, s);
        let p = public_db(&db, &c);
        let pm = &p.media[&m];
        assert!(pm.tags.iter().any(|t| t.person == Some(gran)));
        assert!(!pm.tags.iter().any(|t| t.person == Some(kid)));
        let pl = p.places.values().find(|x| x.name == "Gdańsk").unwrap();
        assert!(pl.weather.contains_key("1920-05-04"));
        assert!(!pl.weather.contains_key("1990-02-03"));
        let pv = crate::query::media_view(&p, &Policy::public(&p.settings, c.today), m).unwrap();
        assert_eq!(pv.tags.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(), vec!["Gran", "Ciocia Ala"]);

        ops::set_place_coords(&mut db, &c, "Gdańsk", Some(54.0), Some(18.65), "manual");
        assert!(db.place_by_name("Gdańsk").unwrap().weather.is_empty());
        // deleting a person keeps the name on the photo
        ops::delete_person(&mut db, gran).unwrap();
        assert!(db.media[&m].tags.iter().any(|t| t.person.is_none() && t.name == "Gran"));
    }
}
