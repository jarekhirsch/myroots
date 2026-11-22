#!/usr/bin/env python3
"""Generates a large, realistic GEDCOM file for performance tests.

python3 gen_big_ged.py 20000 > big.ged
"""
import random
import sys

N = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
random.seed(42)

M_NAMES = "Jan Stanisław Józef Antoni Franciszek Piotr Andrzej Tomasz Michał Wojciech Kazimierz Adam Paweł Marcin Jakub Ignacy Walenty Szymon Maciej Bartłomiej".split()
F_NAMES = "Maria Anna Katarzyna Zofia Agnieszka Barbara Jadwiga Helena Marianna Franciszka Józefa Rozalia Elżbieta Teresa Magdalena Apolonia Julianna Ewa Regina Wiktoria".split()
SURN = "Kowalski Nowak Wiśniewski Wójcik Kamiński Lewandowski Zieliński Szymański Woźniak Dąbrowski Kozłowski Jankowski Mazur Kwiatkowski Krawczyk Piotrowski Grabowski Nowakowski Pawłowski Michalski Hirsch Piechowski Zawadzki Sikora Baran Rutkowski Michalak Ostrowski Tomaszewski Pietrzak".split()
TOWNS = []
for i in range(400):
    TOWNS.append((f"Miejscowość {i:03d}", 49.0 + random.random() * 5.5, 14.2 + random.random() * 9.8))
CEM = [f"Cmentarz parafialny, {TOWNS[i][0]}" for i in range(0, 400, 3)]
OCC = "rolnik kowal nauczyciel szewc krawiec młynarz kupiec stolarz lekarz urzędnik żołnierz gospodyni".split()
EVENTS = [("BAPM", None), ("RESI", None), ("EMIG", None), ("CONF", None), ("GRAD", None)]
MON = "JAN FEB MAR APR MAY JUN JUL AUG SEP OCT NOV DEC".split()


def fem(s):
    if s.endswith("ski"):
        return s[:-1] + "a"
    if s.endswith("cki") or s.endswith("dzki"):
        return s[:-1] + "a"
    return s


def gdate(y):
    r = random.random()
    if r < 0.6:
        return f"{random.randint(1, 28)} {random.choice(MON)} {y}"
    if r < 0.85:
        return f"{y}"
    return f"ABT {y}"


people = []  # (id, sex, given, surname, birth year)
fams = []  # (id, husb, wife, children, marr year)

# Founders, then generations
pid = 0
fid = 0
gen = []
for _ in range(60):
    pid += 1
    people.append([pid, "M", random.choice(M_NAMES), random.choice(SURN), random.randint(1650, 1700)])
    gen.append(pid)

while len(people) < N:
    nxt = []
    for h in gen:
        if len(people) >= N:
            break
        hp = people[h - 1]
        if hp[1] != "M":
            continue
        # spouse from outside
        pid += 1
        wy = hp[4] + random.randint(-6, 3)
        people.append([pid, "F", random.choice(F_NAMES), random.choice(SURN), wy])
        w = pid
        fid += 1
        kids = []
        for k in range(random.randint(1, 7)):
            if len(people) >= N:
                break
            pid += 1
            sex = random.choice("MF")
            people.append([pid, sex, random.choice(M_NAMES if sex == "M" else F_NAMES), hp[3], hp[4] + 22 + k * 2 + random.randint(0, 6)])
            kids.append(pid)
            if sex == "M" or random.random() < 0.3:
                nxt.append(pid)
        fams.append([fid, h, w, kids, hp[4] + 21 + random.randint(0, 5)])
    if not nxt:
        break
    # daughters marrying in: give them husbands from the outside next round
    gen = []
    for x in nxt:
        p = people[x - 1]
        if p[1] == "M":
            gen.append(x)
        elif len(people) < N:
            pid += 1
            people.append([pid, "M", random.choice(M_NAMES), random.choice(SURN), p[4] + random.randint(0, 6)])
            gen.append(pid)
            # her family: husband pid + wife x, children go to husband loop
            # mark wife so the next family uses her instead of a stranger
            people[pid - 1].append(x)

famc = {}
fams_of = {}
for f in fams:
    for c in f[3]:
        famc[c] = f[0]
    fams_of.setdefault(f[1], []).append(f[0])
    fams_of.setdefault(f[2], []).append(f[0])

out = []
w = out.append
w("0 HEAD\n1 SOUR BIGGEN\n1 GEDC\n2 VERS 5.5.1\n2 FORM LINEAGE-LINKED\n1 CHAR UTF-8")
for i in range(1, 61):
    w(f"0 @S{i}@ SOUR\n1 TITL Księga metrykalna parafii {TOWNS[i][0]} {1700 + i}\n1 AUTH Parafia")
for p in people:
    i, sex, given, sur, by = p[:5]
    if sex == "F":
        sur = fem(sur)
    w(f"0 @I{i}@ INDI\n1 NAME {given} /{sur}/\n1 SEX {sex}")
    t = random.choice(TOWNS)
    w(f"1 BIRT\n2 DATE {gdate(by)}\n2 PLAC {t[0]}")
    if random.random() < 0.3:
        w(f"2 SOUR @S{random.randint(1, 60)}@\n3 PAGE akt {random.randint(1, 300)}")
    dy = by + random.randint(1, 90)
    if dy < 2020:
        t2 = random.choice(TOWNS)
        w(f"1 DEAT\n2 DATE {gdate(dy)}\n2 PLAC {t2[0]}")
        if random.random() < 0.5:
            w(f"1 BURI\n2 PLAC {random.choice(CEM)}")
    if random.random() < 0.4:
        w(f"1 OCCU {random.choice(OCC)}")
    for _ in range(random.randint(0, 2)):
        tag = random.choice(EVENTS)[0]
        w(f"1 {tag}\n2 DATE {by + random.randint(0, 40)}\n2 PLAC {random.choice(TOWNS)[0]}")
    if random.random() < 0.1:
        w(f"1 NOTE Notatka o osobie {given} {sur}. Mieszkał w okolicy przez wiele lat i prowadził gospodarstwo.")
    if i in famc:
        w(f"1 FAMC @F{famc[i]}@")
    for f in fams_of.get(i, []):
        w(f"1 FAMS @F{f}@")
for f in fams:
    i, h, wi, kids, my = f
    w(f"0 @F{i}@ FAM\n1 HUSB @I{h}@\n1 WIFE @I{wi}@\n1 MARR\n2 DATE {gdate(my)}\n2 PLAC {random.choice(TOWNS)[0]}")
    for c in kids:
        w(f"1 CHIL @I{c}@")
w("0 TRLR")
sys.stdout.write("\n".join(out) + "\n")
sys.stderr.write(f"{len(people)} people, {len(fams)} families\n")
