//! `.myroots` archive: a ZIP with the full database and all media.
//!
//! ```text
//! myroots.json        { format, version, app, exported, records: [...] }
//! media/<id>          original file
//! media/<id>_t        thumbnail (optional)
//! ```
//! The same format moves a tree between devices (desktop ⇄ phone ⇄ browser)
//! and serves as a backup.

use crate::db::{Database, Record};
use crate::error::{Error, Result};
use crate::model::Id;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const FORMAT: &str = "myroots-archive";
pub const VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct Manifest {
    pub format: String,
    pub version: u32,
    pub app: String,
    pub exported: i64,
    pub records: Vec<Record>,
}

/// Provides media bytes to archive / site writers.
pub trait MediaSource {
    fn original(&mut self, id: Id) -> Option<Vec<u8>>;
    fn thumb(&mut self, id: Id) -> Option<Vec<u8>>;
}

/// In-memory media source (used by the WASM build: JS loads the blobs first).
#[derive(Default)]
pub struct MemMedia {
    pub map: std::collections::HashMap<Id, (Option<Vec<u8>>, Option<Vec<u8>>)>,
}

impl MediaSource for MemMedia {
    fn original(&mut self, id: Id) -> Option<Vec<u8>> {
        self.map.get(&id).and_then(|v| v.0.clone())
    }
    fn thumb(&mut self, id: Id) -> Option<Vec<u8>> {
        self.map.get(&id).and_then(|v| v.1.clone())
    }
}

fn zerr(e: impl std::fmt::Display) -> Error {
    Error::new("archive", e.to_string())
}

pub fn write<W: Write + Seek>(w: W, db: &Database, media: &mut dyn MediaSource, app: &str, now: i64) -> Result<W> {
    let mut z = ZipWriter::new(w);
    let deflate = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let manifest = Manifest { format: FORMAT.into(), version: VERSION, app: app.into(), exported: now, records: db.all_records() };
    z.start_file("myroots.json", deflate).map_err(zerr)?;
    z.write_all(&serde_json::to_vec(&manifest)?).map_err(zerr)?;
    for id in db.media.keys() {
        if let Some(b) = media.original(*id) {
            z.start_file(format!("media/{id}"), stored).map_err(zerr)?;
            z.write_all(&b).map_err(zerr)?;
        }
        if let Some(t) = media.thumb(*id) {
            z.start_file(format!("media/{id}_t"), stored).map_err(zerr)?;
            z.write_all(&t).map_err(zerr)?;
        }
    }
    z.finish().map_err(zerr)
}

pub fn write_vec(db: &Database, media: &mut dyn MediaSource, app: &str, now: i64) -> Result<Vec<u8>> {
    Ok(write(std::io::Cursor::new(Vec::new()), db, media, app, now)?.into_inner())
}

/// Reads an archive; media are passed to `sink(id, is_thumb, bytes)`.
pub fn read<R: Read + Seek>(r: R, mut sink: impl FnMut(Id, bool, Vec<u8>) -> Result<()>) -> Result<Database> {
    let mut z = ZipArchive::new(r).map_err(|_| Error::new("archive_invalid", "not a MyRoots archive"))?;
    let manifest: Manifest = {
        let mut f = z.by_name("myroots.json").map_err(|_| Error::new("archive_invalid", "missing myroots.json"))?;
        let mut s = Vec::new();
        f.read_to_end(&mut s).map_err(zerr)?;
        serde_json::from_slice(&s).map_err(|e| Error::new("archive_invalid", e.to_string()))?
    };
    if manifest.format != FORMAT {
        return Err(Error::new("archive_invalid", "not a MyRoots archive"));
    }
    if manifest.version > VERSION {
        return Err(Error::new("archive_newer", "the archive was made by a newer version of MyRoots"));
    }
    let db = Database::from_records(manifest.records).map_err(|e| Error::new("archive_invalid", e))?;
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(zerr)?;
        let name = f.name().to_string();
        let Some(rest) = name.strip_prefix("media/") else { continue };
        let (id, thumb) = match rest.strip_suffix("_t") {
            Some(x) => (x, true),
            None => (rest, false),
        };
        let Ok(id) = id.parse::<Id>() else { continue };
        let mut b = Vec::with_capacity(f.size() as usize);
        f.read_to_end(&mut b).map_err(zerr)?;
        sink(id, thumb, b)?;
    }
    Ok(db)
}

pub fn read_slice(bytes: &[u8], sink: impl FnMut(Id, bool, Vec<u8>) -> Result<()>) -> Result<Database> {
    read(std::io::Cursor::new(bytes), sink)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::life::Ctx;
    use crate::model::*;
    use crate::ops;
    use chrono::NaiveDate;

    #[test]
    fn roundtrip() {
        let c = Ctx::new(5, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let mut db = Database::new(0);
        let m = ops::register_media(&mut db, &c, Media { mime: "image/jpeg".into(), ..Default::default() });
        let p = ops::create_person(&mut db, &c, Person { first_name: "Jan".into(), photo: Some(m), ..Default::default() }, Default::default()).unwrap();
        let mut mem = MemMedia::default();
        mem.map.insert(m, (Some(vec![1, 2, 3]), Some(vec![9])));
        let bytes = write_vec(&db, &mut mem, "test", 5).unwrap();
        let mut got = vec![];
        let db2 = read_slice(&bytes, |id, t, b| {
            got.push((id, t, b));
            Ok(())
        })
        .unwrap();
        assert_eq!(db2.persons[&p].photo, Some(m));
        assert_eq!(got.len(), 2);
        assert!(db2.meta.next_id > p);
    }
}
