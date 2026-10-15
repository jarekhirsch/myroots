//! Removes hidden metadata from photos published on the family website:
//! EXIF (GPS position, camera, dates), XMP, IPTC and comments. The picture
//! itself is not re-encoded; the EXIF orientation is kept so photos are not
//! turned on their side.

/// Photo bytes without metadata (unknown formats are returned unchanged).
pub fn strip(bytes: &[u8], mime: &str) -> Vec<u8> {
    let out = match mime {
        "image/jpeg" | "image/jpg" | "image/pjpeg" => strip_jpeg(bytes),
        "image/png" => strip_png(bytes),
        _ => None,
    };
    out.unwrap_or_else(|| bytes.to_vec())
}

fn strip_jpeg(b: &[u8]) -> Option<Vec<u8>> {
    if b.len() < 4 || b[0] != 0xFF || b[1] != 0xD8 {
        return None;
    }
    let mut out = Vec::with_capacity(b.len());
    out.extend_from_slice(&b[..2]);
    let mut i = 2;
    let mut orientation: Option<u16> = None;
    let mut wrote_exif = false;
    while i + 4 <= b.len() {
        if b[i] != 0xFF {
            return None; // not a marker where one is expected: leave the file alone
        }
        let marker = b[i + 1];
        if marker == 0xFF {
            i += 1; // fill byte
            continue;
        }
        if marker == 0xDA || (0xD0..=0xD9).contains(&marker) || marker == 0x01 {
            // start of the image data (or a marker without length): copy the rest
            if marker == 0xDA {
                if let (Some(o), false) = (orientation, wrote_exif) {
                    out.extend_from_slice(&exif_orientation(o));
                }
            }
            out.extend_from_slice(&b[i..]);
            return Some(out);
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        if len < 2 || i + 2 + len > b.len() {
            return None;
        }
        let seg = &b[i..i + 2 + len];
        let data = &seg[4..];
        let keep = match marker {
            0xE1 => {
                if data.starts_with(b"Exif\0\0") {
                    orientation = orientation.or_else(|| read_orientation(&data[6..]));
                }
                false
            }
            0xE0 | 0xE2 | 0xEE => true, // JFIF, ICC colour profile, Adobe colour transform
            0xE3..=0xED | 0xEF | 0xFE => false, // other APPn (IPTC, maker data) and comments
            _ => true,                          // tables, frame headers …
        };
        if keep {
            // the orientation goes right after JFIF/at the first non-APP segment
            if !(0xE0..=0xEF).contains(&marker) && !wrote_exif {
                if let Some(o) = orientation {
                    out.extend_from_slice(&exif_orientation(o));
                    wrote_exif = true;
                }
            }
            out.extend_from_slice(seg);
        }
        i += 2 + len;
    }
    None
}

/// Orientation (tag 0x0112) from a TIFF block.
fn read_orientation(t: &[u8]) -> Option<u16> {
    if t.len() < 8 {
        return None;
    }
    let le = match &t[..2] {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let u16_at = |p: usize| -> Option<u16> {
        let s = t.get(p..p + 2)?;
        Some(if le { u16::from_le_bytes([s[0], s[1]]) } else { u16::from_be_bytes([s[0], s[1]]) })
    };
    let u32_at = |p: usize| -> Option<u32> {
        let s = t.get(p..p + 4)?;
        Some(if le { u32::from_le_bytes([s[0], s[1], s[2], s[3]]) } else { u32::from_be_bytes([s[0], s[1], s[2], s[3]]) })
    };
    let ifd = u32_at(4)? as usize;
    let n = u16_at(ifd)? as usize;
    for k in 0..n.min(512) {
        let e = ifd + 2 + k * 12;
        if u16_at(e)? == 0x0112 {
            let v = u16_at(e + 8)?;
            return (1..=8).contains(&v).then_some(v);
        }
    }
    None
}

/// A minimal EXIF segment holding only the orientation.
fn exif_orientation(o: u16) -> Vec<u8> {
    let mut p: Vec<u8> = b"Exif\0\0MM\0\x2a\0\0\0\x08".to_vec();
    p.extend_from_slice(&1u16.to_be_bytes()); // one entry
    p.extend_from_slice(&0x0112u16.to_be_bytes());
    p.extend_from_slice(&3u16.to_be_bytes()); // SHORT
    p.extend_from_slice(&1u32.to_be_bytes());
    p.extend_from_slice(&o.to_be_bytes());
    p.extend_from_slice(&[0, 0]);
    p.extend_from_slice(&0u32.to_be_bytes()); // no next IFD
    let mut seg = vec![0xFF, 0xE1];
    seg.extend_from_slice(&((p.len() + 2) as u16).to_be_bytes());
    seg.extend_from_slice(&p);
    seg
}

fn strip_png(b: &[u8]) -> Option<Vec<u8>> {
    const SIG: &[u8] = b"\x89PNG\r\n\x1a\n";
    if !b.starts_with(SIG) {
        return None;
    }
    let mut out = SIG.to_vec();
    let mut i = SIG.len();
    while i + 12 <= b.len() {
        let len = u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize;
        let end = i.checked_add(12)?.checked_add(len)?;
        if end > b.len() {
            return None;
        }
        let kind = &b[i + 4..i + 8];
        let drop = matches!(kind, b"eXIf" | b"tEXt" | b"zTXt" | b"iTXt" | b"tIME");
        if !drop {
            out.extend_from_slice(&b[i..end]);
        }
        if kind == b"IEND" {
            return Some(out);
        }
        i = end;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg_with_gps() -> Vec<u8> {
        // SOI, APP0 JFIF, APP1 Exif (little endian, orientation 6 + a GPS pointer), COM, DQT, SOS, data, EOI
        let mut exif = b"Exif\0\0II\x2a\0\x08\0\0\0".to_vec();
        exif.extend_from_slice(&2u16.to_le_bytes());
        exif.extend_from_slice(&[0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0]); // orientation 6
        exif.extend_from_slice(&[0x25, 0x88, 4, 0, 1, 0, 0, 0, 0x26, 0, 0, 0]); // GPS IFD pointer
        exif.extend_from_slice(&0u32.to_le_bytes());
        exif.extend_from_slice(b"GPS 52.2297N 21.0122E secret");
        let mut j = vec![0xFF, 0xD8];
        let jfif = b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0";
        j.extend_from_slice(&[0xFF, 0xE0]);
        j.extend_from_slice(&((jfif.len() + 2) as u16).to_be_bytes());
        j.extend_from_slice(jfif);
        j.extend_from_slice(&[0xFF, 0xE1]);
        j.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
        j.extend_from_slice(&exif);
        j.extend_from_slice(&[0xFF, 0xFE, 0, 9]);
        j.extend_from_slice(b"comment");
        j.extend_from_slice(&[0xFF, 0xDB, 0, 4, 1, 2]);
        j.extend_from_slice(&[0xFF, 0xDA, 0, 4, 9, 9, 0x11, 0x22, 0xFF, 0x00, 0x33, 0xFF, 0xD9]);
        j
    }

    #[test]
    fn jpeg_loses_gps_but_keeps_orientation() {
        let j = jpeg_with_gps();
        let s = strip(&j, "image/jpeg");
        let text = String::from_utf8_lossy(&s);
        assert!(!text.contains("GPS") && !text.contains("comment"));
        assert!(s.starts_with(&[0xFF, 0xD8, 0xFF, 0xE0]));
        assert!(s.ends_with(&[0xFF, 0xDA, 0, 4, 9, 9, 0x11, 0x22, 0xFF, 0x00, 0x33, 0xFF, 0xD9]));
        // the new EXIF block says "orientation 6"
        let p = s.windows(6).position(|w| w == b"Exif\0\0").unwrap() + 6;
        assert_eq!(read_orientation(&s[p..]), Some(6));
        // idempotent
        assert_eq!(strip(&s, "image/jpeg"), s);
    }

    #[test]
    fn png_text_chunks_go() {
        let chunk = |t: &[u8], d: &[u8]| {
            let mut c = (d.len() as u32).to_be_bytes().to_vec();
            c.extend_from_slice(t);
            c.extend_from_slice(d);
            c.extend_from_slice(&[0, 0, 0, 0]);
            c
        };
        let mut p = b"\x89PNG\r\n\x1a\n".to_vec();
        p.extend(chunk(b"IHDR", &[0; 13]));
        p.extend(chunk(b"tEXt", b"Author\0secret"));
        p.extend(chunk(b"IDAT", &[1, 2, 3]));
        p.extend(chunk(b"IEND", &[]));
        let s = strip(&p, "image/png");
        assert!(!String::from_utf8_lossy(&s).contains("secret"));
        assert!(String::from_utf8_lossy(&s).contains("IDAT"));
    }

    #[test]
    fn garbage_is_left_alone() {
        let mut r = 0x1234_5678u32;
        for n in 0..3000 {
            let mut j = jpeg_with_gps();
            for _ in 0..1 + n % 5 {
                r ^= r << 13;
                r ^= r >> 17;
                r ^= r << 5;
                let k = (r as usize) % j.len();
                j[k] = (r >> 8) as u8;
            }
            let _ = strip(&j, "image/jpeg");
            let _ = strip(&j, "image/png");
            j.truncate((r as usize) % j.len());
            let _ = strip(&j, "image/jpeg");
        }
    }
}
