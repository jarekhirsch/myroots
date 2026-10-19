// MyRoots © 2026 Jarek Hirsch, https://hirsch.fi
// SPDX-License-Identifier: GPL-3.0-or-later (with additional terms, see NOTICE.md)

//! MyRoots desktop / mobile shell.
//!
//! The web UI talks to the same Rust engine as the browser version, but here
//! it runs natively: records are kept in SQLite and photos as files in the
//! app's data folder (see `myroots-store`).

use myroots_core::{archive, site, Ctx, Engine};
use myroots_store::{Store, StoreMedia};
use serde_json::Value;
use std::fs;
use std::io::Cursor;
use std::sync::Mutex;
use tauri::http::{header, Response as HttpResponse, StatusCode};
use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{AppHandle, Manager, State};

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Inner {
    engine: Engine,
    store: Store,
}

#[derive(Default)]
struct AppState(Mutex<Option<Inner>>);

fn ctx() -> Ctx {
    let now = chrono::Local::now();
    Ctx::new(now.timestamp_millis(), now.date_naive())
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn open_inner(app: &AppHandle) -> Result<Inner, String> {
    let dir = app.path().app_data_dir().map_err(err)?;
    let mut store = Store::open(&dir).map_err(err)?;
    let mut engine = Engine::new(store.database(ctx().now).map_err(err)?, VERSION);
    // a brand-new tree: persist the initial meta / settings records
    store.apply(engine.take_changes()).map_err(err)?;
    Ok(Inner { engine, store })
}

fn with<T>(app: &AppHandle, state: &State<AppState>, f: impl FnOnce(&mut Inner) -> Result<T, String>) -> Result<T, String> {
    let mut g = state.0.lock().map_err(|_| "state poisoned".to_string())?;
    if g.is_none() {
        *g = Some(open_inner(app)?);
    }
    f(g.as_mut().unwrap())
}

#[tauri::command]
fn engine_open(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    with(&app, &state, |_| Ok(()))
}

/// Every engine method (see crates/myroots-core/src/engine.rs). Errors are `{code, message}`.
#[tauri::command]
fn engine_call(app: AppHandle, state: State<AppState>, method: String, params: Value) -> Result<Value, Value> {
    let mut g = state.0.lock().map_err(|_| Value::String("state poisoned".into()))?;
    if g.is_none() {
        *g = Some(open_inner(&app).map_err(Value::String)?);
    }
    let inner = g.as_mut().unwrap();
    let res = inner.engine.call(&method, params, &ctx());
    let changes = inner.engine.take_changes();
    inner.store.apply(changes).map_err(|e| serde_json::json!({ "code": "storage", "message": e.to_string() }))?;
    res.map_err(|e| serde_json::to_value(&e).unwrap_or(Value::Null))
}

fn header_str<'a>(req: &'a Request<'_>, name: &str) -> Option<&'a str> {
    req.headers().get(name).and_then(|v| v.to_str().ok())
}

fn raw_body<'a>(req: &'a Request<'_>) -> Result<&'a [u8], String> {
    match req.body() {
        InvokeBody::Raw(b) => Ok(b),
        _ => Err("expected raw bytes".into()),
    }
}

/// Stores media bytes. Headers: `x-id`, `x-thumb` ("1" for the thumbnail).
#[tauri::command]
fn media_put(app: AppHandle, state: State<AppState>, request: Request) -> Result<(), String> {
    let id: u64 = header_str(&request, "x-id").and_then(|v| v.parse().ok()).ok_or("missing x-id")?;
    let thumb = header_str(&request, "x-thumb") == Some("1");
    let bytes = raw_body(&request)?.to_vec();
    with(&app, &state, |i| i.store.put_media(id, thumb, &bytes).map_err(err))
}

/// Media bytes (empty when missing).
#[tauri::command]
fn media_get(app: AppHandle, state: State<AppState>, id: u64, thumb: bool) -> Result<Response, String> {
    with(&app, &state, |i| Ok(Response::new(i.store.get_media(id, thumb).unwrap_or_default())))
}

/// The whole tree as a `.myroots` archive (the UI saves it where the user chose).
#[tauri::command]
fn archive_bytes(app: AppHandle, state: State<AppState>) -> Result<Response, String> {
    with(&app, &state, |i| {
        let mut m = StoreMedia(&i.store);
        archive::write_vec(&i.engine.db, &mut m, VERSION, ctx().now).map(Response::new).map_err(|e| serde_json::to_string(&e).unwrap_or_default())
    })
}

/// Replaces everything with the content of a `.myroots` archive (raw body).
#[tauri::command]
fn archive_import(app: AppHandle, state: State<AppState>, request: Request) -> Result<(), String> {
    let bytes = raw_body(&request)?.to_vec();
    with(&app, &state, |i| {
        let dir = i.store.dir().to_path_buf();
        let tmp = dir.join("media-import");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).map_err(err)?;
        let db = archive::read_slice(&bytes, |id, thumb, b| {
            let name = if thumb { format!("{id}_t") } else { id.to_string() };
            fs::write(tmp.join(name), b).map_err(|e| myroots_core::Error::new("archive", e.to_string()))
        })
        .map_err(|e| {
            let _ = fs::remove_dir_all(&tmp);
            serde_json::to_string(&e).unwrap_or_default()
        })?;
        // Archive is valid: swap the media folder and the records.
        let media = dir.join(myroots_store::MEDIA_DIR);
        let _ = fs::remove_dir_all(&media);
        fs::rename(&tmp, &media).map_err(err)?;
        i.store.replace_all(&db.all_records()).map_err(err)?;
        i.engine = Engine::new(db, VERSION);
        let _ = i.engine.take_changes();
        Ok(())
    })
}

/// The read-only family website as a ZIP (viewer template + public data + media).
/// The print dialog for the page (window.print() does nothing in WKWebView).
#[tauri::command]
fn print_page(webview: tauri::Webview) -> Result<(), String> {
    #[cfg(desktop)]
    {
        webview.print().map_err(|e| e.to_string())
    }
    #[cfg(not(desktop))]
    {
        let _ = webview;
        Err("not supported".into())
    }
}

#[tauri::command]
fn site_bytes(app: AppHandle, state: State<AppState>) -> Result<Response, String> {
    let resolver = app.asset_resolver();
    let list = resolver.get("viewer/files.json".into()).ok_or("viewer template is missing")?;
    let names: Vec<String> = serde_json::from_slice(&list.bytes).map_err(err)?;
    let mut files = Vec::new();
    for n in names {
        let a = resolver.get(format!("viewer/{n}")).ok_or_else(|| format!("missing viewer/{n}"))?;
        files.push((n, a.bytes));
    }
    with(&app, &state, |i| {
        let mut m = StoreMedia(&i.store);
        let (w, _) = site::write(Cursor::new(Vec::new()), &i.engine.db, &ctx(), &files, &mut m, VERSION).map_err(|e| serde_json::to_string(&e).unwrap_or_default())?;
        Ok(Response::new(w.into_inner()))
    })
}

/// `media://localhost/<id>` and `<id>_t` → image bytes for <img src>.
fn media_protocol(app: &AppHandle, path: &str) -> HttpResponse<Vec<u8>> {
    let not_found = || HttpResponse::builder().status(StatusCode::NOT_FOUND).body(Vec::new()).unwrap();
    let name = path.trim_start_matches('/');
    let (id_s, thumb) = match name.strip_suffix("_t") {
        Some(x) => (x, true),
        None => (name, false),
    };
    let Ok(id) = id_s.parse::<u64>() else { return not_found() };
    let state = app.state::<AppState>();
    let Ok(g) = state.0.lock() else { return not_found() };
    let Some(inner) = g.as_ref() else { return not_found() };
    let bytes = inner.store.get_media(id, thumb).or_else(|| if thumb { inner.store.get_media(id, false) } else { None });
    let Some(bytes) = bytes else { return not_found() };
    let mime = if thumb && inner.store.media_path(id, true).exists() {
        "image/jpeg".to_string()
    } else {
        inner.engine.db.media.get(&id).map(|m| m.mime.clone()).filter(|m| !m.is_empty()).unwrap_or_else(|| "application/octet-stream".into())
    };
    // only pictures, sound, video and PDF are served as such; never as a page or a script
    let mime = myroots_core::model::safe_mime(&mime);
    HttpResponse::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header(header::CONTENT_SECURITY_POLICY, "default-src 'none'; img-src 'self'; media-src 'self'; style-src 'unsafe-inline'; sandbox")
        .header(header::CACHE_CONTROL, "max-age=31536000, immutable")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(bytes)
        .unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .register_uri_scheme_protocol("media", |ctx, req| media_protocol(ctx.app_handle(), req.uri().path()))
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();
            let inner = open_inner(&handle).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
            *state.0.lock().unwrap() = Some(inner);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![engine_open, engine_call, media_put, media_get, archive_bytes, archive_import, site_bytes, print_page])
        .run(tauri::generate_context!())
        .expect("error while running MyRoots");
}
