//! JSON command dispatcher shared by every platform:
//! Tauri commands (desktop / mobile), WASM (browser PWA) and the read-only
//! family website all call `Engine::call(method, params)`.

use crate::db::{ChangeSet, Database};
use crate::error::{Error, Result};
use crate::life::Ctx;
use crate::model::*;
use crate::privacy::Policy;
use crate::*;
use base64::Engine as _;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct Engine {
    pub db: Database,
    pub app_version: String,
    /// Read-only family website: mutating calls are rejected and the public
    /// privacy policy applies.
    pub viewer: bool,
}

fn p<T: DeserializeOwned>(params: &Value) -> Result<T> {
    serde_json::from_value(params.clone()).map_err(|e| Error::invalid(format!("bad parameters: {e}")))
}

fn id_param(params: &Value, key: &str) -> Result<Id> {
    params.get(key).and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))).ok_or_else(|| Error::invalid(format!("missing `{key}`")))
}

fn ok<T: serde::Serialize>(v: T) -> Result<Value> {
    Ok(serde_json::to_value(v)?)
}

const READ_ONLY: &[&str] = &[
    "app.info", "overview", "settings.get", "people.summaries", "person.profile", "person.history", "pets.list", "pet.get",
    "sources.list", "source.get", "media.list", "media.get", "graves.list", "clans.list", "places.list", "tree.data", "lifeline", "map.data", "stats",
    "anniversaries", "events.list", "gallery", "kinship", "book.build", "suggestion.encode",
];

pub const SUGGESTION_PREFIX: &str = "MYROOTS-SUGGESTION:";
pub const SUGGESTION_SUFFIX: &str = ":END";

impl Engine {
    pub fn new(db: Database, app_version: &str) -> Engine {
        Engine { db, app_version: app_version.into(), viewer: false }
    }

    pub fn take_changes(&mut self) -> ChangeSet {
        self.db.take_changes()
    }

    fn policy(&self, ctx: &Ctx) -> Policy {
        if self.viewer {
            Policy::public(&self.db.settings, ctx.today)
        } else {
            Policy::owner(self.db.settings.max_age, ctx.today)
        }
    }

    fn sections(&self, ctx: &Ctx) -> Value {
        let pol = self.policy(ctx);
        let mut m = serde_json::Map::new();
        for s in privacy::SECTIONS {
            m.insert(s.to_string(), json!(!pol.blocks_section(s)));
        }
        m.insert("kinship".into(), json!(pol.kinship_allowed()));
        m.insert("map".into(), json!(!pol.blocks_section("map") && self.db.settings.map_external));
        Value::Object(m)
    }

    pub fn call(&mut self, method: &str, params: Value, ctx: &Ctx) -> Result<Value> {
        if self.viewer && !READ_ONLY.contains(&method) {
            return Err(Error::new("read_only", "the family website is read-only"));
        }
        let pol = self.policy(ctx);
        let sections = self.sections(ctx);
        let viewer = self.viewer;
        let app_version = self.app_version.clone();
        let db = &mut self.db;
        match method {
            /* ---------------- general ---------------- */
            "app.info" => ok(json!({
                "version": app_version,
                "viewer": viewer,
                "data_version": db.meta.version,
                "sections": sections,
                "tree_name": db.settings.tree_name,
                "site": db.settings.site,
                "appearance": db.settings.appearance,
                "language": db.settings.language,
                "suggestions": db.settings.suggestions,
                "map_external": db.settings.map_external,
                "weather_external": !db.settings.weather_off && !viewer,
                "empty": db.persons.is_empty(),
            })),
            "overview" => ok(query::overview(db, ctx)),
            "settings.get" => {
                let mut s = db.settings.clone();
                if viewer {
                    s.merge_dismissed.clear();
                }
                ok(s)
            }
            "settings.save" => {
                let s: Settings = p(params.get("settings").unwrap_or(&params))?;
                ops::save_settings(db, s);
                ok(&db.settings)
            }
            "data.reset" => {
                if params.get("confirm").and_then(|v| v.as_str()) != Some("DELETE") {
                    return Err(Error::invalid("confirmation missing"));
                }
                let settings = db.settings.clone();
                for r in db.all_records() {
                    match r.kind {
                        db::Kind::Person => drop(db.del_person_raw(r.id)),
                        db::Kind::Union => drop(db.del_union(r.id)),
                        db::Kind::Pet => drop(db.del_pet(r.id)),
                        db::Kind::Source => drop(db.del_source(r.id)),
                        db::Kind::Place => drop(db.del_place(r.id)),
                        db::Kind::Revision => drop(db.del_revision(r.id)),
                        db::Kind::Suggestion => drop(db.del_suggestion(r.id)),
                        db::Kind::Media => db.delete_media(r.id),
                        _ => {}
                    }
                }
                let s = if params.get("keep_settings").and_then(|v| v.as_bool()).unwrap_or(true) {
                    let mut s = settings;
                    s.tree_start_person = None;
                    s.merge_dismissed.clear();
                    s
                } else {
                    Settings::default()
                };
                db.put_settings(s);
                wp_import::clear_pending(db);
                ok(json!({"ok": true}))
            }

            /* ---------------- people ---------------- */
            "people.summaries" => {
                if viewer {
                    let ids: Vec<Id> = db.persons.keys().copied().collect();
                    let v: Vec<query::Summary> = ids.into_iter().filter_map(|i| query::public_summary(db, &pol, i)).collect();
                    ok(v)
                } else {
                    ok(query::summaries(db, ctx))
                }
            }
            "person.profile" => {
                let id = id_param(&params, "id")?;
                if viewer {
                    if pol.blocks_section("profile") {
                        return Err(Error::new("private", "private content"));
                    }
                    if let Some(pp) = db.persons.get(&id) {
                        if pol.should_hide(pp) {
                            return Err(Error::new("private", "private content"));
                        }
                    }
                }
                query::profile(db, ctx, id, &pol).map(|v| serde_json::to_value(v).unwrap()).ok_or_else(|| Error::not_found("person", id))
            }
            "person.edit" => {
                let id = id_param(&params, "id")?;
                query::edit_model(db, ctx, id).map(|v| serde_json::to_value(v).unwrap()).ok_or_else(|| Error::not_found("person", id))
            }
            "person.create" => {
                #[derive(Deserialize)]
                struct P {
                    person: Person,
                    #[serde(default)]
                    relations: ops::RelationEdits,
                }
                let a: P = p(&params)?;
                let id = ops::create_person(db, ctx, a.person, a.relations)?;
                ok(json!({ "id": id }))
            }
            "person.update" => {
                #[derive(Deserialize)]
                struct P {
                    person: Person,
                    #[serde(default)]
                    relations: ops::RelationEdits,
                }
                let a: P = p(&params)?;
                let id = a.person.id;
                ops::update_person(db, ctx, a.person, a.relations)?;
                ok(json!({ "id": id }))
            }
            "person.delete" => {
                ops::delete_person(db, id_param(&params, "id")?)?;
                ok(json!({"ok": true}))
            }
            "person.add_relative" => {
                #[derive(Deserialize)]
                struct P {
                    of: Id,
                    relation: ops::NewRelation,
                    #[serde(default)]
                    person: Person,
                    #[serde(default)]
                    other_parent: Option<Id>,
                }
                let a: P = p(&params)?;
                let id = ops::add_relative(db, ctx, a.of, a.relation, a.person, a.other_parent)?;
                ok(json!({ "id": id }))
            }
            "person.link" => {
                // Link two existing people: relation of `other` to `of`.
                #[derive(Deserialize)]
                struct P {
                    of: Id,
                    other: Id,
                    relation: ops::NewRelation,
                }
                let a: P = p(&params)?;
                if !db.persons.contains_key(&a.other) {
                    return Err(Error::not_found("person", a.other));
                }
                match a.relation {
                    ops::NewRelation::Father => ops::set_parent(db, ctx, a.of, ParentRole::Father, Some(a.other), ParentRel::Bio)?,
                    ops::NewRelation::Mother => ops::set_parent(db, ctx, a.of, ParentRole::Mother, Some(a.other), ParentRel::Bio)?,
                    ops::NewRelation::Child => {
                        let g = db.persons.get(&a.of).map(|x| x.gender).unwrap_or_default();
                        let role = ParentRole::for_gender(g);
                        let child = db.persons.get(&a.other).cloned().unwrap();
                        let role = if child.parent(role).is_some() && child.parent(role.other()).is_none() && !g.is_known() { role.other() } else { role };
                        ops::set_parent(db, ctx, a.other, role, Some(a.of), ParentRel::Bio)?
                    }
                    ops::NewRelation::Spouse => {
                        ops::check_partners(db, a.of, a.other)?;
                        ops::link_partners(db, a.of, a.other, None)?;
                    }
                    ops::NewRelation::Sibling => {
                        let base = db.persons.get(&a.of).cloned().ok_or_else(|| Error::not_found("person", a.of))?;
                        if let Some(f) = base.father {
                            ops::set_parent(db, ctx, a.other, ParentRole::Father, Some(f), ParentRel::Bio)?;
                        }
                        if let Some(m) = base.mother {
                            ops::set_parent(db, ctx, a.other, ParentRole::Mother, Some(m), ParentRel::Bio)?;
                        }
                    }
                }
                ok(json!({"ok": true}))
            }
            "person.set_parent" => {
                #[derive(Deserialize)]
                struct P {
                    child: Id,
                    role: ParentRole,
                    parent: Option<Id>,
                    #[serde(default)]
                    rel: ParentRel,
                }
                let a: P = p(&params)?;
                ops::set_parent(db, ctx, a.child, a.role, a.parent, a.rel)?;
                ok(json!({"ok": true}))
            }
            "person.history" => {
                let id = id_param(&params, "id")?;
                let mut v: Vec<&Revision> = db.revisions.values().filter(|r| r.person == id).collect();
                v.sort_by(|a, b| b.time.cmp(&a.time).then(b.id.cmp(&a.id)));
                ok(v)
            }
            "person.restore" => {
                ops::restore_revision(db, ctx, id_param(&params, "revision")?)?;
                ok(json!({"ok": true}))
            }
            "person.set_media" => {
                #[derive(Deserialize)]
                struct P {
                    id: Id,
                    recordings: Option<Vec<Recording>>,
                    aging: Option<Vec<AgingFrame>>,
                    #[serde(default, with = "double_option")]
                    aging_audio: Option<Option<Id>>,
                    #[serde(default)]
                    add_to_gallery: Vec<Id>,
                }
                let a: P = p(&params)?;
                let person = ops::set_person_media(db, ctx, a.id, a.recordings, a.aging, a.aging_audio, &a.add_to_gallery)?;
                ok(json!({ "recordings": person.recordings, "aging": person.aging, "aging_audio": person.aging_audio, "gallery": person.gallery }))
            }
            "person.set_sources" => {
                #[derive(Deserialize)]
                struct P {
                    id: Id,
                    sources: Vec<Id>,
                }
                let a: P = p(&params)?;
                ops::set_person_sources(db, a.id, &a.sources)?;
                ok(json!({"ok": true}))
            }
            "union.save" => {
                let u: Union = p(params.get("union").unwrap_or(&params))?;
                if let Some(b) = u.b {
                    ops::check_partners(db, u.a, b)?;
                }
                let id = ops::save_union(db, u)?;
                ok(json!({ "id": id }))
            }
            "union.delete" => {
                let id = id_param(&params, "id")?;
                db.del_union(id).ok_or_else(|| Error::not_found("union", id))?;
                ok(json!({"ok": true}))
            }

            /* ---------------- pets / sources ---------------- */
            "pets.list" => {
                let mut v = query::pets(db, ctx);
                if viewer {
                    v.retain(|x| x.pet.status == Status::Published);
                }
                ok(v)
            }
            "pet.get" => {
                let id = id_param(&params, "id")?;
                let v = query::pets(db, ctx).into_iter().find(|x| x.pet.id == id).ok_or_else(|| Error::not_found("pet", id))?;
                ok(v)
            }
            "pet.save" => {
                let pet: Pet = p(params.get("pet").unwrap_or(&params))?;
                ok(json!({ "id": ops::save_pet(db, ctx, pet)? }))
            }
            "pet.delete" => {
                let id = id_param(&params, "id")?;
                db.del_pet(id).ok_or_else(|| Error::not_found("pet", id))?;
                ok(json!({"ok": true}))
            }
            "sources.list" => ok(query::sources(db, ctx)),
            // id and title of every source (pickers; sources.list is heavy in big trees)
            "sources.brief" => {
                let mut v: Vec<(String, Value)> = db.sources.values().map(|s| (crate::model::norm(&s.title), json!({ "id": s.id, "title": s.title, "date": s.date }))).collect();
                v.sort_by(|a, b| a.0.cmp(&b.0));
                ok(v.into_iter().map(|x| x.1).collect::<Vec<_>>())
            }
            "source.get" => {
                let id = id_param(&params, "id")?;
                let s = db.sources.get(&id).cloned().ok_or_else(|| Error::not_found("source", id))?;
                ok(query::source_view(db, ctx, &s))
            }
            "source.save" => {
                let s: Source = p(params.get("source").unwrap_or(&params))?;
                ok(json!({ "id": ops::save_source(db, ctx, s)? }))
            }
            "source.delete" => {
                ops::delete_source(db, id_param(&params, "id")?)?;
                ok(json!({"ok": true}))
            }

            /* ---------------- media ---------------- */
            "media.reserve" => ok(json!({ "id": ops::reserve_media_id(db) })),
            "media.register" => {
                let m: Media = p(params.get("media").unwrap_or(&params))?;
                ok(json!({ "id": ops::register_media(db, ctx, m) }))
            }
            "media.update" => {
                let id = id_param(&params, "id")?;
                let mut m = db.media.get(&id).cloned().ok_or_else(|| Error::not_found("media", id))?;
                if let Some(c) = params.get("caption").and_then(|v| v.as_str()) {
                    m.caption = c.trim().to_string();
                }
                if let Some(d) = params.get("date").and_then(|v| v.as_str()) {
                    m.date = crate::date::sanitize(d);
                }
                db.put_media_meta(m);
                ok(json!({"ok": true}))
            }
            "media.link_imported" => {
                let links: Vec<(Id, crate::gedcom::MediaLink)> = params
                    .get("links")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|x| Some((x.get("media")?.as_u64()? as Id, serde_json::from_value(x.clone()).ok()?))).collect())
                    .unwrap_or_default();
                ok(json!({ "linked": ops::link_imported_media(db, ctx, &links) }))
            }
            "media.get" => {
                let id = id_param(&params, "id")?;
                query::media_view(db, &pol, id).map(|v| serde_json::to_value(v).unwrap()).ok_or_else(|| Error::not_found("media", id))
            }
            "media.tags" => {
                let id = id_param(&params, "id")?;
                let tags: Vec<PhotoTag> = p(params.get("tags").unwrap_or(&json!([])))?;
                ops::set_media_tags(db, id, &tags)?;
                query::media_view(db, &pol, id).map(|v| serde_json::to_value(v).unwrap()).ok_or_else(|| Error::not_found("media", id))
            }
            "media.delete" => {
                ops::delete_media(db, id_param(&params, "id")?);
                ok(json!({"ok": true}))
            }
            "media.list" => ok(db.media.values().collect::<Vec<_>>()),
            "media.faces" => ok(query::face_index(db)),
            "media.unused" => ok(ops::unused_media(db)),

            /* ---------------- clans / places ---------------- */
            "clans.list" => ok(query::clans(db)),
            "clans.rename" => {
                let from = params.get("from").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let to = params.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string();
                ok(json!({ "count": ops::rename_clan(db, &from, &to)? }))
            }
            "clans.delete" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                ok(json!({ "count": ops::delete_clan(db, &name) }))
            }
            "clans.assign" => {
                #[derive(Deserialize)]
                struct P {
                    name: String,
                    persons: Vec<Id>,
                    add: bool,
                }
                let a: P = p(&params)?;
                ok(json!({ "count": ops::assign_clan(db, &a.name, &a.persons, a.add) }))
            }
            "graves.list" => ok(query::graves(db, &pol)),
            "graves.cemeteries" => ok(query::cemeteries(db)),
            "places.list" => ok(query::places(db)),
            "places.missing" => ok(query::missing_places(db)),
            "places.set" => {
                #[derive(Deserialize)]
                struct P {
                    name: String,
                    lat: Option<f64>,
                    lng: Option<f64>,
                    #[serde(default)]
                    origin: String,
                }
                let a: P = p(&params)?;
                let origin = if a.origin.is_empty() { "manual".to_string() } else { a.origin };
                ok(json!({ "id": ops::set_place_coords(db, ctx, &a.name, a.lat, a.lng, &origin) }))
            }
            "cemeteries.save" => {
                #[derive(Deserialize)]
                struct P {
                    #[serde(default)]
                    id: Option<Id>,
                    name: String,
                    #[serde(default)]
                    address: String,
                    lat: Option<f64>,
                    lng: Option<f64>,
                    #[serde(default)]
                    origin: String,
                }
                let a: P = p(&params)?;
                ok(json!({ "id": ops::save_cemetery(db, ctx, a.id, &a.name, &a.address, a.lat, a.lng, &a.origin)? }))
            }
            "cemeteries.remove" => {
                ops::remove_cemetery(db, ctx, id_param(&params, "id")?)?;
                ok(json!({"ok": true}))
            }
            "places.weather" => {
                #[derive(Deserialize)]
                struct P {
                    name: String,
                    date: String,
                    weather: DayWeather,
                }
                let a: P = p(&params)?;
                ops::set_place_weather(db, &a.name, &a.date, a.weather)?;
                ok(json!({"ok": true}))
            }
            "places.rename" => {
                let from = params.get("from").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let to = params.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string();
                ok(json!({ "count": ops::rename_place(db, &from, &to) }))
            }

            /* ---------------- views ---------------- */
            "tree.data" => {
                let nodes = tree::nodes(db, &pol);
                let start = tree::start_person(db).filter(|s| !pol.should_hide(&db.persons[s]));
                // lite: only what the family timeline draws
                if params.get("lite").and_then(|v| v.as_bool()).unwrap_or(false) {
                    ok(json!({ "nodes": tree::lite(&nodes), "start": start }))
                } else {
                    ok(json!({ "nodes": nodes, "start": start }))
                }
            }
            "lifeline" => {
                let id = id_param(&params, "id")?;
                lifeline::build(db, &pol, id).map(|v| serde_json::to_value(v).unwrap()).ok_or_else(|| Error::not_found("person", id))
            }
            "map.data" => {
                let only = params.get("person").and_then(|v| v.as_u64());
                ok(map::build(db, &pol, only))
            }
            "stats" => {
                let clan = params.get("clan").and_then(|v| v.as_str()).unwrap_or("");
                ok(stats::compute(db, &pol, clan))
            }
            "anniversaries" => {
                let days = params.get("days").and_then(|v| v.as_i64()).unwrap_or(30);
                let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                ok(query::anniversaries(db, ctx, &pol, days, limit))
            }
            "events.list" => ok(query::family_events(db, &pol)),
            "gallery" => ok(query::gallery(db, &pol)),
            "kinship" => {
                if !pol.kinship_allowed() {
                    return Err(Error::new("private", "private content"));
                }
                let a = id_param(&params, "a")?;
                let b = id_param(&params, "b")?;
                kinship::compute(db, &pol, a, b).map(|v| serde_json::to_value(v).unwrap()).ok_or_else(|| Error::not_found("person", a))
            }
            "health.issues" => ok(health::analyze(db)),
            "health.count" => ok(json!({ "count": health::analyze(db).len() })),
            "health.completeness" => ok(health::completeness(db, ctx.today)),
            "merge.suggestions" => ok(merge::suggestions(db, 30)),
            "merge.dismiss" => {
                merge::dismiss(db, id_param(&params, "a")?, id_param(&params, "b")?);
                ok(json!({"ok": true}))
            }
            "merge.compare" => ok(merge::compare(db, id_param(&params, "a")?, id_param(&params, "b")?)?),
            "merge.execute" => {
                let sel: merge::Selection = p(params.get("selection").unwrap_or(&Value::Null)).unwrap_or_default();
                let id = merge::execute(db, ctx, id_param(&params, "a")?, id_param(&params, "b")?, id_param(&params, "keep")?, sel)?;
                ok(json!({ "id": id }))
            }
            "book.build" => {
                let req: book::BookRequest = p(&params)?;
                ok(book::build(db, &pol, &req))
            }

            /* ---------------- suggestions ---------------- */
            "suggestions.list" => {
                let mut v: Vec<&Suggestion> = db.suggestions.values().collect();
                v.sort_by(|a, b| b.created.cmp(&a.created));
                ok(v)
            }
            "suggestion.encode" => {
                // Used by the family website to build the code sent by e-mail.
                let s: Suggestion = p(&params)?;
                let json = serde_json::to_vec(&s)?;
                ok(json!({ "code": format!("{SUGGESTION_PREFIX}{}{SUGGESTION_SUFFIX}", base64::engine::general_purpose::STANDARD.encode(json)) }))
            }
            "suggestions.import" => {
                let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let mut added = 0;
                let mut dup = 0;
                for part in text.split(SUGGESTION_PREFIX).skip(1) {
                    let Some(end) = part.find(SUGGESTION_SUFFIX) else { continue };
                    // e-mail clients may wrap long lines or quote them with "> "
                    let code: String = part[..end].chars().filter(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '=').collect();
                    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(code) else { continue };
                    let Ok(mut s) = serde_json::from_slice::<Suggestion>(&bytes) else { continue };
                    if !s.token.is_empty() && db.suggestions.values().any(|x| x.token == s.token) {
                        dup += 1;
                        continue;
                    }
                    s.id = db.next_id();
                    s.status = "new".into();
                    if s.created == 0 {
                        s.created = ctx.now;
                    }
                    s.person = s.person.filter(|p| db.persons.contains_key(p));
                    s.message = s.message.chars().take(5000).collect();
                    db.put_suggestion(s);
                    added += 1;
                }
                ok(json!({ "added": added, "duplicates": dup }))
            }
            "suggestions.add" => {
                let mut s: Suggestion = p(&params)?;
                s.id = db.next_id();
                s.status = "new".into();
                s.created = ctx.now;
                db.put_suggestion(s);
                ok(json!({"ok": true}))
            }
            "suggestions.set_status" => {
                let id = id_param(&params, "id")?;
                let mut s = db.suggestions.get(&id).cloned().ok_or_else(|| Error::not_found("suggestion", id))?;
                s.status = params.get("status").and_then(|v| v.as_str()).unwrap_or("new").to_string();
                db.put_suggestion(s);
                ok(json!({"ok": true}))
            }
            "suggestions.delete" => {
                let id = id_param(&params, "id")?;
                db.del_suggestion(id);
                ok(json!({"ok": true}))
            }

            /* ---------------- import / export ---------------- */
            "gedcom.import" => {
                // Raw file bytes (any encoding: UTF-8/16, ANSEL, Windows code pages; or a zipped GEDCOM) as base64, or decoded text.
                let text = match params.get("base64").and_then(|v| v.as_str()) {
                    Some(b) => gedcom::decode_file(&base64::engine::general_purpose::STANDARD.decode(b.trim()).map_err(|_| Error::invalid("bad base64"))?)?,
                    None => params.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                };
                let opts: gedcom::ImportOptions = p(params.get("options").unwrap_or(&json!({})))?;
                if opts.dry_run {
                    let mut copy = db.clone();
                    let r = gedcom::import(&mut copy, ctx, &text, &opts)?;
                    return ok(r);
                }
                ok(gedcom::import(db, ctx, &text, &opts)?)
            }
            "gedcom.export" => {
                let mut opts: gedcom::ExportOptions = p(params.get("options").unwrap_or(&json!({})))?;
                opts.app_version = app_version.clone();
                if opts.tree_name.is_empty() {
                    opts.tree_name = db.settings.tree_name.clone();
                }
                ok(json!({ "text": gedcom::export(db, ctx, &opts) }))
            }
            "wp.import" => {
                let text = params.get("json").and_then(|v| v.as_str()).unwrap_or("");
                ok(wp_import::import(db, ctx, text)?)
            }
            "wp.pending" => ok(wp_import::load_pending(db)),
            "wp.attach" => {
                let n = wp_import::attach(db, id_param(&params, "wp_id")?, id_param(&params, "media")?)?;
                ok(json!({ "count": n }))
            }
            "wp.clear" => {
                wp_import::clear_pending(db);
                ok(json!({"ok": true}))
            }
            _ => Err(Error::new("unknown_method", format!("unknown method `{method}`"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn dispatch() {
        let c = Ctx::new(1, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let mut e = Engine::new(Database::new(0), "3.0.0");
        let r = e.call("person.create", json!({"person": {"first_name": "Jan", "gender": "M"}}), &c).unwrap();
        let id = r["id"].as_u64().unwrap();
        let r = e.call("person.add_relative", json!({"of": id, "relation": "spouse", "person": {"first_name": "Anna"}}), &c).unwrap();
        assert!(r["id"].as_u64().is_some());
        let prof = e.call("person.profile", json!({"id": id}), &c).unwrap();
        assert_eq!(prof["unions"].as_array().unwrap().len(), 1);
        assert!(e.take_changes().records().0.len() >= 3);
        let code = e.call("suggestion.encode", json!({"person": id, "message": "Data urodzenia to 1901", "token": "t1"}), &c).unwrap();
        let text = format!("Hello\n{}\nbye", code["code"].as_str().unwrap());
        let r = e.call("suggestions.import", json!({ "text": text }), &c).unwrap();
        assert_eq!(r["added"], 1);
        let r = e.call("suggestions.import", json!({ "text": text }), &c).unwrap();
        assert_eq!(r["duplicates"], 1);
        e.viewer = true;
        assert_eq!(e.call("person.delete", json!({"id": id}), &c).unwrap_err().code, "read_only");
    }

    #[test]
    fn recordings_and_years() {
        let c = Ctx::new(1, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let mut e = Engine::new(Database::new(0), "3.0.0");
        let id = e.call("person.create", json!({"person": {"first_name": "Jan"}}), &c).unwrap()["id"].as_u64().unwrap();
        let mut media = vec![];
        for (n, mime) in [(1, "audio/mpeg"), (2, "image/jpeg"), (3, "image/jpeg")] {
            let r = e.call("media.register", json!({"media": {"file_name": format!("f{n}"), "mime": mime}}), &c).unwrap();
            media.push(r["id"].as_u64().unwrap());
        }
        let r = e
            .call(
                "person.set_media",
                json!({"id": id, "recordings": [{"media": media[0], "title": " Kołysanka ", "date": "1987"}], "aging_audio": media[0],
                       "aging": [{"media": media[2], "date": "1990", "eyes": [0.4, 0.4, 0.6, 2.0]}, {"media": media[1], "date": "1950", "eyes": null}, {"media": 999, "date": "1960"}]}),
                &c,
            )
            .unwrap();
        assert_eq!(r["recordings"][0]["title"], "Kołysanka");
        assert_eq!(r["aging"].as_array().unwrap().len(), 2);
        assert_eq!(r["aging"][0]["date"], "1950"); // sorted, unknown media dropped
        assert_eq!(r["aging"][1]["eyes"][3], 1.0); // clamped
        assert_eq!(r["aging_audio"], media[0]);
        // only the audio is cleared, the photos stay
        e.call("person.set_media", json!({"id": id, "aging_audio": null}), &c).unwrap();
        let prof = e.call("person.profile", json!({"id": id}), &c).unwrap();
        assert!(prof["person"]["aging_audio"].is_null());
        assert_eq!(prof["person"]["aging"].as_array().unwrap().len(), 2);
        // deleting a media removes it everywhere
        e.call("media.delete", json!({"id": media[0]}), &c).unwrap();
        let prof = e.call("person.profile", json!({"id": id}), &c).unwrap();
        assert!(prof["person"]["recordings"].as_array().unwrap().is_empty());
        let unused = e.call("media.unused", json!({}), &c).unwrap();
        assert!(!unused.as_array().unwrap().iter().any(|v| v.as_u64() == Some(media[1])));
    }
}

/// Tells a missing field (leave unchanged) from `null` (clear).
mod double_option {
    use serde::{Deserialize, Deserializer};
    pub fn deserialize<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Option::<T>::deserialize(d).map(Some)
    }
}
