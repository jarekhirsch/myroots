#!/usr/bin/env python3
"""Adds photos to a test GEDCOM and packs both into a ZIP (import with photos).

python3 gen_media.py big.ged 2000 big-photos.zip

Every photo is a 1600x2000 JPEG (about 300 KB, like a scanned print); one in
ten is a group photo linked to three people.
"""
import io
import random
import re
import sys
import zipfile

from PIL import Image, ImageDraw, ImageFilter

src, count, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
random.seed(7)
ged = open(src, encoding="utf-8").read()
ids = re.findall(r"^0 @(I\d+)@ INDI", ged, re.M)
links = {}
for n in range(count):
    who = random.sample(ids, 3 if n % 10 == 0 else 1)
    for i in who:
        links.setdefault(i, []).append(f"zdjecia/p{n:05d}.jpg")


def photo(n):
    w, h = 1600, 2000
    base = [random.randint(60, 200) for _ in range(3)]
    im = Image.new("RGB", (w, h), tuple(base))
    d = ImageDraw.Draw(im)
    for _ in range(40):
        c = tuple(max(0, min(255, b + random.randint(-70, 70))) for b in base)
        x, y = random.randint(0, w), random.randint(0, h)
        r = random.randint(60, 500)
        d.ellipse([x - r, y - r, x + r, y + r], fill=c)
    im = im.filter(ImageFilter.GaussianBlur(6))
    noise = Image.effect_noise((w, h), 24).convert("RGB")
    im = Image.blend(im, noise, 0.12)
    d = ImageDraw.Draw(im)
    d.text((80, 80), f"Zdjęcie {n}", fill=(255, 255, 255))
    b = io.BytesIO()
    im.save(b, "JPEG", quality=82)
    return b.getvalue()


lines = []
cur = None
for line in ged.splitlines():
    m = re.match(r"^0 @(I\d+)@ INDI", line)
    if line.startswith("0 ") and cur in links:
        for k, f in enumerate(links[cur]):
            lines += ["1 OBJE", f"2 FILE {f}", "3 FORM jpg", f"2 TITL Zdjęcie {f[9:-4]}"] + (["2 _PRIM Y"] if k == 0 else [])
    cur = m.group(1) if m else (None if line.startswith("0 ") else cur)
    lines.append(line)
with zipfile.ZipFile(out, "w", zipfile.ZIP_STORED) as z:
    z.writestr("drzewo.ged", "\n".join(lines) + "\n")
    for n in range(count):
        z.writestr(f"zdjecia/p{n:05d}.jpg", photo(n))
        if n % 250 == 0:
            print(n, file=sys.stderr)
