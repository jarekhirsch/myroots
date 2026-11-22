// MyRoots © 2026 Jarek Hirsch, https://hirsch.lt
// SPDX-License-Identifier: GPL-3.0-or-later (with additional terms, see NOTICE.md)

//! WebAssembly bindings: the browser app (PWA) and the exported family website
//! run the very same engine as the desktop and mobile apps.
//!
//! Persistence stays in JavaScript (IndexedDB): after each call the UI takes
//! the pending changes (`take_changes`) and writes them asynchronously.

use myroots_core::archive::{self, MediaSource};
use myroots_core::db::{Database, Record};
use myroots_core::{site, Ctx, Engine};
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn err(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn core_err(e: myroots_core::Error) -> JsValue {
    JsValue::from_str(&serde_json::to_string(&e).unwrap_or_else(|_| e.to_string()))
}

fn ctx(now: f64, today: &str) -> Result<Ctx, JsValue> {
    let today = chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d").map_err(err)?;
    Ok(Ctx::new(now as i64, today))
}

/// Media provider backed by a synchronous JS callback `(id, thumb) => Uint8Array | undefined`.
struct JsMedia<'a> {
    f: &'a js_sys::Function,
}

impl<'a> JsMedia<'a> {
    fn get(&self, id: u64, thumb: bool) -> Option<Vec<u8>> {
        let r = self.f.call2(&JsValue::NULL, &JsValue::from_f64(id as f64), &JsValue::from_bool(thumb)).ok()?;
        if r.is_undefined() || r.is_null() {
            return None;
        }
        Some(js_sys::Uint8Array::new(&r).to_vec())
    }
}

impl<'a> MediaSource for JsMedia<'a> {
    fn original(&mut self, id: u64) -> Option<Vec<u8>> {
        self.get(id, false)
    }
    fn thumb(&mut self, id: u64) -> Option<Vec<u8>> {
        self.get(id, true)
    }
}

#[derive(Serialize)]
struct Changes {
    records: Vec<Record>,
    deleted_media: Vec<u64>,
}

#[wasm_bindgen]
pub struct WasmEngine {
    engine: Engine,
}

#[wasm_bindgen]
impl WasmEngine {
    /// `records_json`: JSON array of stored records ("" or "[]" = new, empty tree).
    #[wasm_bindgen(constructor)]
    pub fn new(records_json: &str, app_version: &str, viewer: bool, now: f64) -> Result<WasmEngine, JsValue> {
        let records: Vec<Record> = if records_json.trim().is_empty() { vec![] } else { serde_json::from_str(records_json).map_err(err)? };
        let db = if records.is_empty() { Database::new(now as i64) } else { Database::from_records(records).map_err(err)? };
        let mut engine = Engine::new(db, app_version);
        engine.viewer = viewer;
        Ok(WasmEngine { engine })
    }

    /// Calls an engine method. Returns the JSON result; throws a JSON error `{code, message}`.
    pub fn call(&mut self, method: &str, params_json: &str, now: f64, today: &str) -> Result<String, JsValue> {
        let params: serde_json::Value = if params_json.trim().is_empty() { serde_json::Value::Null } else { serde_json::from_str(params_json).map_err(err)? };
        let c = ctx(now, today)?;
        let v = self.engine.call(method, params, &c).map_err(core_err)?;
        serde_json::to_string(&v).map_err(err)
    }

    /// Pending changes as JSON `{records: [...], deleted_media: [...]}`.
    pub fn take_changes(&mut self) -> String {
        let (records, deleted_media) = self.engine.take_changes().records();
        serde_json::to_string(&Changes { records, deleted_media }).unwrap()
    }

    /// Every stored record (full dump) as JSON.
    pub fn all_records(&self) -> String {
        serde_json::to_string(&self.engine.db.all_records()).unwrap()
    }

    /// Builds a `.myroots` archive. `media(id, thumb)` must return the bytes synchronously.
    pub fn export_archive(&self, media: &js_sys::Function, now: f64) -> Result<Vec<u8>, JsValue> {
        let mut m = JsMedia { f: media };
        archive::write_vec(&self.engine.db, &mut m, &self.engine.app_version, now as i64).map_err(core_err)
    }

    /// Replaces the database with the archive content. Media are passed to
    /// `sink(id, thumb, bytes)`. Returns the new records as JSON (to persist).
    pub fn import_archive(&mut self, bytes: &[u8], sink: &js_sys::Function) -> Result<String, JsValue> {
        let db = archive::read_slice(bytes, |id, thumb, b| {
            let arr = js_sys::Uint8Array::from(b.as_slice());
            sink.call3(&JsValue::NULL, &JsValue::from_f64(id as f64), &JsValue::from_bool(thumb), &arr)
                .map(|_| ())
                .map_err(|e| myroots_core::Error::new("archive", format!("{e:?}")))
        })
        .map_err(core_err)?;
        let viewer = self.engine.viewer;
        let version = self.engine.app_version.clone();
        self.engine = Engine::new(db, &version);
        self.engine.viewer = viewer;
        let _ = self.engine.take_changes();
        Ok(self.all_records())
    }

    /// Media ids needed by the family website (to preload them).
    pub fn site_media(&self, now: f64, today: &str) -> Result<Vec<f64>, JsValue> {
        let c = ctx(now, today)?;
        Ok(site::public_db(&self.engine.db, &c).media.keys().map(|k| *k as f64).collect())
    }

    /// Builds the family website ZIP. `viewer_paths` / `viewer_files` are the
    /// prebuilt read-only app files.
    pub fn export_site(&self, viewer_paths: Vec<String>, viewer_files: Vec<js_sys::Uint8Array>, media: &js_sys::Function, now: f64, today: &str) -> Result<Vec<u8>, JsValue> {
        let c = ctx(now, today)?;
        let files: Vec<(String, Vec<u8>)> = viewer_paths.into_iter().zip(viewer_files.into_iter().map(|a| a.to_vec())).collect();
        let mut m = JsMedia { f: media };
        let (w, _) = site::write(std::io::Cursor::new(Vec::new()), &self.engine.db, &c, &files, &mut m, &self.engine.app_version).map_err(core_err)?;
        Ok(w.into_inner())
    }

    /// GEDCOM bytes (any encoding) → UTF-8 text.
    pub fn decode_gedcom(bytes: &[u8]) -> String {
        myroots_core::gedcom::decode(bytes)
    }
}
