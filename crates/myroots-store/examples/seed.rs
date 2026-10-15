//! Fills a store with a GEDCOM file and one photo (manual testing of the apps).
//! cargo run -p myroots-store --example seed -- <data dir> <file.ged> [photo.jpg]
use myroots_core::{Ctx, Engine};
use myroots_store::Store;
use serde_json::json;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut st = Store::open(&a[1]).expect("open");
    let now = 1_792_000_000_000;
    let c = Ctx::new(now, chrono::NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
    let mut e = Engine::new(st.database(now).unwrap(), "3.0.0");
    let ged = std::fs::read_to_string(&a[2]).expect("ged");
    let r = e.call("gedcom.import", json!({"text": ged, "options": {}}), &c).unwrap();
    println!("{r}");
    if let Some(photo) = a.get(3) {
        let bytes = std::fs::read(photo).unwrap();
        let id = e.call("media.reserve", json!({}), &c).unwrap()["id"].as_u64().unwrap();
        st.put_media(id, false, &bytes).unwrap();
        e.call("media.register", json!({"media": {"id": id, "file_name": "p.jpg", "mime": "image/jpeg", "size": bytes.len()}}), &c).unwrap();
        let people = e.call("people.summaries", json!({}), &c).unwrap();
        for p in people.as_array().unwrap().iter().take(3) {
            let pid = p["id"].as_u64().unwrap();
            let mut m = e.call("person.edit", json!({"id": pid}), &c).unwrap();
            m["person"]["photo"] = json!(id);
            e.call("person.update", json!({"person": m["person"]}), &c).unwrap();
        }
    }
    st.apply(e.take_changes()).unwrap();
}
