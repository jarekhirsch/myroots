//! Timing of engine calls on a large tree (performance check).
//! cargo run --release -p myroots-core --example bench -- big.ged
use myroots_core::{Ctx, Database, Engine};
use serde_json::json;
use std::time::Instant;

fn main() {
    let path = std::env::args().nth(1).expect("path to a .ged file");
    let text = std::fs::read_to_string(path).unwrap();
    let today = chrono::NaiveDate::from_ymd_opt(2026, 11, 22).unwrap();
    let ctx = Ctx::new(1_795_000_000_000, today);
    let mut e = Engine::new(Database::new(ctx.now), "bench");
    let t = Instant::now();
    let r = e.call("gedcom.import", json!({"text": text, "options": {"mode": "new"}}), &ctx).unwrap();
    println!("{:>8.1} ms  gedcom.import {}", t.elapsed().as_secs_f64() * 1e3, r.to_string().chars().take(120).collect::<String>());
    let ids: Vec<u64> = e.db.persons.keys().copied().collect();
    let a = ids[ids.len() / 2];
    let b = ids[ids.len() / 3];
    let calls: Vec<(&str, serde_json::Value)> = vec![
        ("app.info", json!({})),
        ("overview", json!({})),
        ("people.summaries", json!({})),
        ("person.profile", json!({"id": a})),
        ("person.profile", json!({"id": b})),
        ("person.edit", json!({"id": a})),
        ("tree.data", json!({})),
        ("lifeline", json!({"id": a})),
        ("map.data", json!({})),
        ("map.data", json!({"person": a})),
        ("stats", json!({})),
        ("anniversaries", json!({"days": 30})),
        ("events.list", json!({})),
        ("gallery", json!({})),
        ("kinship", json!({"a": a, "b": b})),
        ("health.issues", json!({})),
        ("health.completeness", json!({})),
        ("merge.suggestions", json!({})),
        ("graves.list", json!({})),
        ("places.list", json!({})),
        ("places.missing", json!({})),
        ("clans.list", json!({})),
        ("sources.list", json!({})),
        ("book.build", json!({"person": a, "mode": "descendants", "structure": "generations", "max_gen": 4, "clans": []})),
        ("gedcom.export", json!({})),
    ];
    for (m, p) in calls {
        let t = Instant::now();
        let r = e.call(m, p, &ctx);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        let size = r.as_ref().map(|v| v.to_string().len()).unwrap_or(0);
        let t2 = Instant::now();
        let _ = r.as_ref().map(|v| v.to_string());
        println!("{:>8.1} ms  {:<22} {:>9} B json ({:.1} ms) {}", ms, m, size, t2.elapsed().as_secs_f64() * 1e3, r.err().map(|e| format!("{e:?}")).unwrap_or_default());
    }
    // a small edit, as the UI does it
    let t = Instant::now();
    let mut ed = e.call("person.edit", json!({"id": a}), &ctx).unwrap();
    let person = ed["person"].take();
    e.call("person.update", json!({"person": person}), &ctx).unwrap();
    let ch = e.take_changes();
    println!("{:>8.1} ms  edit+save ({} changed records)", t.elapsed().as_secs_f64() * 1e3, ch.records().0.len());
    let t = Instant::now();
    let recs = e.db.all_records();
    let n = recs.len();
    let db2 = Database::from_records(recs).unwrap();
    println!("{:>8.1} ms  all_records + from_records ({} records, {} persons)", t.elapsed().as_secs_f64() * 1e3, n, db2.persons.len());
    let t = Instant::now();
    let mut mm = myroots_core::archive::MemMedia::default();
    let z = myroots_core::archive::write_vec(&e.db, &mut mm, "bench", ctx.now);
    println!("{:>8.1} ms  archive export {:?}", t.elapsed().as_secs_f64() * 1e3, z.as_ref().map(|v| v.len()).map_err(|e| e.to_string()));
}
