//! Reads a .myroots backup the way the desktop app does and prints what is in it.
//! cargo run -p myroots-core --example check_archive -- file.myroots
fn main() {
    let path = std::env::args().nth(1).expect("path to a .myroots file");
    let f = std::fs::File::open(&path).expect("open");
    let (mut n, mut bytes) = (0usize, 0usize);
    let db = myroots_core::archive::read(std::io::BufReader::new(f), |_, _, b| {
        n += 1;
        bytes += b.len();
        Ok(())
    })
    .expect("read archive");
    println!("people {}, media records {}, files {} ({} MB)", db.persons.len(), db.media.len(), n, bytes / 1_000_000);
}
