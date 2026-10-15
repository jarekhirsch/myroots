// MyRoots © 2026 Jarek Hirsch, https://hirsch.fi
// SPDX-License-Identifier: GPL-3.0-or-later (with additional terms, see NOTICE.md)

//! Native storage of a MyRoots tree.
//!
//! * Records (people, unions, settings…) live in one SQLite file as JSON rows
//!   keyed by `(kind, id)`; the engine keeps the whole tree in memory and the
//!   store only persists the change sets it produces.
//! * Photos and scans are plain files in `media/` next to the database
//!   (`<id>` = original, `<id>_t` = thumbnail), so the OS can back them up and
//!   the web view can load them directly.

use myroots_core::archive::MediaSource;
use myroots_core::db::{ChangeSet, Database, Kind, Record};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const DB_FILE: &str = "myroots.sqlite";
pub const MEDIA_DIR: &str = "media";

#[derive(Debug)]
pub struct StoreError(pub String);

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for StoreError {}
impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError(format!("database: {e}"))
    }
}
impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        StoreError(format!("file: {e}"))
    }
}
impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError(format!("json: {e}"))
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;

pub struct Store {
    conn: Connection,
    dir: PathBuf,
}

fn kind_from(s: &str) -> Option<Kind> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
}

impl Store {
    /// Opens (or creates) the store in `dir`.
    pub fn open(dir: impl AsRef<Path>) -> Result<Store> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(dir.join(MEDIA_DIR))?;
        let conn = Connection::open(dir.join(DB_FILE))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS records (
                 kind TEXT NOT NULL,
                 id   INTEGER NOT NULL,
                 data TEXT NOT NULL,
                 PRIMARY KEY (kind, id)
             ) WITHOUT ROWID;",
        )?;
        Ok(Store { conn, dir })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// All stored records.
    pub fn load(&self) -> Result<Vec<Record>> {
        let mut st = self.conn.prepare("SELECT kind, id, data FROM records")?;
        let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?)))?;
        let mut out = Vec::new();
        for row in rows {
            let (k, id, data) = row?;
            let Some(kind) = kind_from(&k) else { continue };
            out.push(Record { kind, id: id as u64, data: Some(serde_json::from_str(&data)?) });
        }
        Ok(out)
    }

    /// Opens the database held by the store (a new, empty tree when there is none).
    pub fn database(&self, now: i64) -> Result<Database> {
        let recs = self.load()?;
        if recs.is_empty() {
            Ok(Database::new(now))
        } else {
            Database::from_records(recs).map_err(StoreError)
        }
    }

    /// Persists a change set in one transaction and removes deleted media files.
    pub fn apply(&mut self, changes: ChangeSet) -> Result<()> {
        if changes.is_empty() {
            return Ok(());
        }
        let (records, deleted_media) = changes.records();
        self.write_records(&records)?;
        for id in deleted_media {
            for thumb in [false, true] {
                let p = self.media_path(id, thumb);
                if p.exists() {
                    let _ = fs::remove_file(p);
                }
            }
        }
        Ok(())
    }

    fn write_records(&mut self, records: &[Record]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut up = tx.prepare_cached("INSERT INTO records (kind, id, data) VALUES (?1, ?2, ?3) ON CONFLICT(kind, id) DO UPDATE SET data = excluded.data")?;
            let mut del = tx.prepare_cached("DELETE FROM records WHERE kind = ?1 AND id = ?2")?;
            for r in records {
                match &r.data {
                    Some(v) => {
                        up.execute(params![r.kind.as_str(), r.id as i64, serde_json::to_string(v)?])?;
                    }
                    None => {
                        del.execute(params![r.kind.as_str(), r.id as i64])?;
                    }
                }
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Replaces everything (restoring an archive): records and media.
    pub fn replace_all(&mut self, records: &[Record]) -> Result<()> {
        self.conn.execute("DELETE FROM records", [])?;
        self.write_records(records)
    }

    /// Removes every media file (before restoring an archive).
    pub fn clear_media(&self) -> Result<()> {
        let d = self.dir.join(MEDIA_DIR);
        if d.exists() {
            fs::remove_dir_all(&d)?;
        }
        fs::create_dir_all(&d)?;
        Ok(())
    }

    pub fn media_path(&self, id: u64, thumb: bool) -> PathBuf {
        self.dir.join(MEDIA_DIR).join(if thumb { format!("{id}_t") } else { id.to_string() })
    }

    pub fn put_media(&self, id: u64, thumb: bool, bytes: &[u8]) -> Result<()> {
        let p = self.media_path(id, thumb);
        let tmp = p.with_extension("part");
        fs::write(&tmp, bytes)?;
        fs::rename(tmp, p)?;
        Ok(())
    }

    pub fn get_media(&self, id: u64, thumb: bool) -> Option<Vec<u8>> {
        fs::read(self.media_path(id, thumb)).ok()
    }

    /// Size of the database file (diagnostics).
    pub fn size(&self) -> u64 {
        fs::metadata(self.dir.join(DB_FILE)).map(|m| m.len()).unwrap_or(0)
    }

    pub fn has_record(&self, kind: Kind, id: u64) -> Result<bool> {
        Ok(self.conn.query_row("SELECT 1 FROM records WHERE kind = ?1 AND id = ?2", params![kind.as_str(), id as i64], |_| Ok(())).optional()?.is_some())
    }
}

/// Media of the store as a source for archives and the family website.
pub struct StoreMedia<'a>(pub &'a Store);

impl MediaSource for StoreMedia<'_> {
    fn original(&mut self, id: u64) -> Option<Vec<u8>> {
        self.0.get_media(id, false)
    }
    fn thumb(&mut self, id: u64) -> Option<Vec<u8>> {
        self.0.get_media(id, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use myroots_core::{Ctx, Engine};
    use serde_json::json;

    #[test]
    fn persist_and_reload() {
        let dir = std::env::temp_dir().join(format!("myroots-store-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let c = Ctx::new(1, chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let id;
        {
            let mut st = Store::open(&dir).unwrap();
            let mut e = Engine::new(st.database(1).unwrap(), "3.0.0");
            id = e.call("person.create", json!({"person": {"first_name": "Jan", "last_name": "Kowalski"}}), &c).unwrap()["id"].as_u64().unwrap();
            let m = e.call("media.reserve", json!({}), &c).unwrap()["id"].as_u64().unwrap();
            st.put_media(m, false, b"jpeg").unwrap();
            e.call("media.register", json!({"media": {"id": m, "file_name": "a.jpg", "mime": "image/jpeg"}}), &c).unwrap();
            st.apply(e.take_changes()).unwrap();
            assert_eq!(st.get_media(m, false).unwrap(), b"jpeg");
            e.call("media.delete", json!({"id": m}), &c).unwrap();
            st.apply(e.take_changes()).unwrap();
            assert!(st.get_media(m, false).is_none());
        }
        let st = Store::open(&dir).unwrap();
        let mut e = Engine::new(st.database(2).unwrap(), "3.0.0");
        let p = e.call("person.profile", json!({"id": id}), &c).unwrap();
        assert_eq!(p["person"]["last_name"], "Kowalski");
        assert!(st.has_record(Kind::Person, id).unwrap());
        let _ = fs::remove_dir_all(&dir);
    }
}
