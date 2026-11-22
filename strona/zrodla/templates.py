# Free family tree templates for myroots.cc (print and fill-in PDFs).
import math
from reportlab.lib.pagesizes import A4, A3, landscape
from reportlab.lib.units import mm
from reportlab.pdfgen import canvas
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.lib.colors import HexColor, white

import os
F = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'fonts-src') + '/'
for n in ['Literata-SemiBold', 'Literata-Regular', 'Onest-Regular', 'Onest-SemiBold']:
    pdfmetrics.registerFont(TTFont(n, F + n + '.ttf'))
pdfmetrics.registerFont(TTFont('DejaVu', '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf'))
pdfmetrics.registerFont(TTFont('DejaVu-Bold', '/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf'))
T = {}
RTL = False
FN = {'Literata-SemiBold': 'Literata-SemiBold', 'Onest-Regular': 'Onest-Regular', 'Onest-SemiBold': 'Onest-SemiBold'}


def tx(s):
    if RTL and s:
        from bidi.algorithm import get_display
        return get_display(s)
    return s


def font(c, name, size):
    c.setFont(FN[name], size)


def fit(c, name, size, text, width, low=4.5):
    """Sets the largest font size (up to size) at which text fits in width."""
    while size > low and c.stringWidth(tx(text), FN[name], size) > width:
        size -= 0.25
    c.setFont(FN[name], size)


INK = HexColor('#1f2a24')
MUTED = HexColor('#6b746e')
LINE = HexColor('#c9cfc9')
GREEN = HexColor('#1d5a44')
SAND = HexColor('#f1e7d6')
SAGE = HexColor('#dfe9e2')
PAPER = HexColor('#fbf8f2')



def header(c, W, H, title, sub):
    c.setFillColor(PAPER)
    c.rect(0, 0, W, H, stroke=0, fill=1)
    c.setFillColor(INK)
    font(c, 'Literata-SemiBold', 22)
    c.drawCentredString(W / 2, H - 20 * mm, tx(title))
    c.setFillColor(MUTED)
    font(c, 'Onest-Regular', 9.5)
    c.drawCentredString(W / 2, H - 27 * mm, tx(sub))


def footer(c, W):
    c.setFillColor(MUTED)
    font(c, 'Onest-Regular', 7.5)
    c.drawCentredString(W / 2, 8 * mm, tx(T['footer']))


def meta(c, title, subject):
    c.setTitle(title)
    c.setAuthor('MyRoots (myroots.cc)')
    c.setSubject(subject)
    c.setKeywords(T['keywords'])
    c.setCreator('myroots.cc')


def box(c, x, y, w, h, role, form=None, key='', fill=white, photo=False):
    """A person box: role label, name line, born, died. form: acroForm or None."""
    c.setFillColor(fill)
    c.setStrokeColor(LINE)
    c.setLineWidth(0.8)
    c.roundRect(x, y, w, h, 3 * mm, stroke=1, fill=1)
    px = x + 3 * mm
    if photo:
        pw = min(h - 6 * mm, w * 0.36)
        c.setFillColor(HexColor('#efe9dd'))
        c.setStrokeColor(LINE)
        c.roundRect(x + 3 * mm, y + h - 3 * mm - pw, pw, pw, 2 * mm, stroke=1, fill=1)
        c.setFillColor(MUTED)
        font(c, 'Onest-Regular', 6.5)
        c.drawCentredString(x + 3 * mm + pw / 2, y + h - 3 * mm - pw / 2 - 1, tx(T['photo']))
        px = x + 6 * mm + pw
    c.setFillColor(GREEN)
    fit(c, 'Onest-SemiBold', 7.5, role.upper(), x + w - 3 * mm - px)
    c.drawString(px, y + h - 6 * mm, tx(role.upper()))
    rows = [('', T['name']), (T['b'], T['born']), (T['d'], T['died'])]
    lh = (h - 9 * mm) / 3
    for i, (lab, hint) in enumerate(rows):
        ly = y + h - 9 * mm - lh * (i + 1) + 1.8 * mm
        c.setStrokeColor(LINE)
        c.setLineWidth(0.5)
        lx = px + ((c.stringWidth(lab, FN['Onest-Regular'], 7) + 2 * mm) if lab else 0)
        c.line(lx, ly, x + w - 3 * mm, ly)
        c.setFillColor(MUTED)
        font(c, 'Onest-Regular', 7)
        if lab:
            c.drawString(px, ly + 0.8 * mm, tx(lab))
        if i == 0:
            font(c, 'Onest-Regular', 5.5)
            c.setFillColor(HexColor('#a3aaa4'))
            c.drawRightString(x + w - 3 * mm, ly - 2.2 * mm, tx(hint))
        if form:
            fs = 10 if i == 0 else 8
            form.textfield(name=f'{key}_{i}', tooltip=f'{role}: {hint}', x=lx, y=ly + 0.3 * mm, width=x + w - 3 * mm - lx, height=lh - 2.6 * mm,
                           fontName='Helvetica', fontSize=fs, borderWidth=0, fillColor=None, textColor=INK, forceBorder=False)


def connector(c, x1, y1, x2, y2):
    """Child (x1,y1 top-centre) to parent (x2,y2 bottom-centre), elbow line."""
    c.setStrokeColor(HexColor('#b9c2bb'))
    c.setLineWidth(1)
    my = (y1 + y2) / 2
    p = c.beginPath()
    p.moveTo(x1, y1)
    p.lineTo(x1, my)
    p.lineTo(x2, my)
    p.lineTo(x2, y2)
    c.drawPath(p, stroke=1, fill=0)


def four_generations(path, fillable):
    """Pyramid: me, parents, grandparents, 8 great-grandparents (od pradziadków)."""
    W, H = landscape(A4)
    c = canvas.Canvas(path, pagesize=(W, H))
    meta(c, T['four_fill' if fillable else 'four_print'], T['four_subj'])
    header(c, W, H, T['four_h'], T['four_sub'] + (T['fill_note'] if fillable else ''))
    form = c.acroForm if fillable else None
    margin = 10 * mm
    rows = [8, 4, 2, 1]
    roles = [
        [T['ggf'], T['ggm']] * 4,
        [T['gf'], T['gm']] * 2,
        [T['father'], T['mother']],
        [T['me']],
    ]
    fills = [SAND, SAGE, SAND, HexColor('#e4ece6')]
    top = H - 34 * mm
    bottom = 16 * mm
    bh = 27 * mm
    gap = (top - bottom - 4 * bh) / 3
    centres = []
    for r, n in enumerate(rows):
        slot = (W - 2 * margin) / 8
        w = slot * 8 / n - (3 * mm if n == 8 else 8 * mm)
        w = min(w, 92 * mm)
        y = top - bh - r * (bh + gap)
        row = []
        for i in range(n):
            cx = margin + (W - 2 * margin) * (i + 0.5) / n
            box(c, cx - w / 2, y, w, bh, roles[r][i], form, f'g{r}_{i}', fill=fills[r] if r < 3 else HexColor('#e4ece6'))
            row.append((cx, y))
        centres.append(row)
    for r in range(1, 4):
        for i, (cx, y) in enumerate(centres[r]):
            for j in (2 * i, 2 * i + 1):
                px, py = centres[r - 1][j]
                connector(c, cx, y + bh, px, py)
    footer(c, W)
    c.save()


def with_photos(path):
    """Three generations with photo frames (ze zdjęciami): me, parents, grandparents."""
    W, H = landscape(A4)
    c = canvas.Canvas(path, pagesize=(W, H))
    meta(c, T['ph_title'], T['ph_subj'])
    header(c, W, H, T['ph_h'], T['ph_sub'])
    rows = [4, 2, 1]
    roles = [[T['gf'], T['gm']] * 2, [T['father'], T['mother']], [T['me']]]
    bh, bw = 34 * mm, 64 * mm
    top = H - 36 * mm
    gap = (top - 16 * mm - 3 * bh) / 2
    centres = []
    for r, n in enumerate(rows):
        y = top - bh - r * (bh + gap)
        row = []
        for i in range(n):
            cx = 12 * mm + (W - 24 * mm) * (i + 0.5) / n
            box(c, cx - bw / 2, y, bw, bh, roles[r][i], photo=True, fill=[SAND, SAGE, HexColor('#e4ece6')][r])
            row.append((cx, y))
        centres.append(row)
    for r in range(1, 3):
        for i, (cx, y) in enumerate(centres[r]):
            for j in (2 * i, 2 * i + 1):
                px, py = centres[r - 1][j]
                connector(c, cx, y + bh, px, py)
    footer(c, W)
    c.save()


def fan(path):
    """Fan of ancestors, 5 generations, A3 landscape."""
    W, H = landscape(A3)
    c = canvas.Canvas(path, pagesize=(W, H))
    meta(c, T['fan_title'], T['fan_subj'])
    header(c, W, H, T['fan_h'], T['fan_sub'])
    cx, cy = W / 2, 26 * mm
    R = min(W / 2 - 14 * mm, H - cy - 36 * mm)
    gens = 5
    r0 = R / (gens + 0.3)
    ring = (R - r0) / (gens - 1)
    labels = {1: (T['father'], T['mother']), 2: (T['gf'], T['gm']), 3: (T['ggf'], T['ggm'])}
    for g in range(1, gens):
        n = 2 ** g
        a_in, a_out = r0 + ring * (g - 1), r0 + ring * g
        for i in range(n):
            a0 = 180 - 180 * i / n
            a1 = 180 - 180 * (i + 1) / n
            side = i < n / 2
            light = 0.0 + g * 0.06
            c.setFillColor(SAND if side else SAGE)
            c.setStrokeColor(white)
            c.setLineWidth(1.5)
            p = c.beginPath()
            p.moveTo(cx + a_in * math.cos(math.radians(a0)), cy + a_in * math.sin(math.radians(a0)))
            p.arcTo(cx - a_out, cy - a_out, cx + a_out, cy + a_out, startAng=a0, extent=a1 - a0)
            p.arcTo(cx - a_in, cy - a_in, cx + a_in, cy + a_in, startAng=a1, extent=a0 - a1)
            p.close()
            c.drawPath(p, stroke=1, fill=1)
            # writing lines, radial in narrow cells, tangential in wide ones
            am = math.radians((a0 + a1) / 2)
            mid = (a_in + a_out) / 2
            c.saveState()
            c.translate(cx + mid * math.cos(am), cy + mid * math.sin(am))
            if g <= 2:
                c.rotate(math.degrees(am) - 90)
                span = mid * math.radians(abs(a1 - a0)) * (0.5 if g == 1 else 0.6)
                lines = 3
                for k in range(lines):
                    yy = ring * 0.22 - k * ring * 0.2
                    c.setStrokeColor(HexColor('#c2b59a') if side else HexColor('#a9bfb1'))
                    c.setLineWidth(0.5)
                    c.line(-span / 2, yy, span / 2, yy)
                c.setFillColor(MUTED)
                fit(c, 'Onest-Regular', 6.5, labels[g][i % 2], span)
                c.drawCentredString(0, -ring * 0.36, tx(labels[g][i % 2]))
            else:
                rot = math.degrees(am)
                if math.cos(am) < 0:
                    rot += 180
                c.rotate(rot)
                span = ring * 0.8
                cell = mid * math.radians(abs(a1 - a0))
                for k in range(2 if g == 3 else 1):
                    yy = (cell * 0.12) - k * cell * 0.3 if g == 3 else 0
                    c.setStrokeColor(HexColor('#c2b59a') if side else HexColor('#a9bfb1'))
                    c.setLineWidth(0.5)
                    c.line(-span / 2 + (6 * mm if g == 3 else 0), yy - 1, span / 2, yy - 1)
                if g == 3:
                    c.setFillColor(MUTED)
                    lab = labels[3][i % 2]
                    fit(c, 'Onest-Regular', 5.5, lab, span - 1 * mm)
                    if math.cos(am) < 0:
                        c.drawString(-span / 2, -cell * 0.38, tx(lab))
                    else:
                        c.drawString(-span / 2, -cell * 0.38, tx(lab))
            c.restoreState()
    # centre: me
    c.setFillColor(GREEN)
    p = c.beginPath()
    p.moveTo(cx - r0 + 2, cy)
    p.arcTo(cx - r0 + 2, cy - r0 + 2, cx + r0 - 2, cy + r0 - 2, startAng=180, extent=-180)
    p.close()
    c.drawPath(p, stroke=0, fill=1)
    c.setStrokeColor(HexColor('#7fa996'))
    for k in range(2):
        c.line(cx - r0 * 0.6, cy + r0 * (0.5 - k * 0.22), cx + r0 * 0.6, cy + r0 * (0.5 - k * 0.22))
    c.setFillColor(white)
    font(c, 'Onest-SemiBold', 9)
    c.drawCentredString(cx, cy + r0 * 0.12, tx(T['me'].upper()))
    c.setFillColor(MUTED)
    font(c, 'Onest-Regular', 8)
    c.drawString(20 * mm, cy - 3, tx(T['dad_side']))
    c.drawRightString(W - 20 * mm, cy - 3, tx(T['mum_side']))
    footer(c, W)
    c.save()


def person_card(path, fillable):
    """A4 portrait: everything about one person."""
    W, H = A4
    c = canvas.Canvas(path, pagesize=A4)
    meta(c, T['card_fill' if fillable else 'card_print'], T['card_subj'])
    header(c, W, H, T['card_h'], T['card_sub'] + (T['fill_note'] if fillable else ''))
    form = c.acroForm if fillable else None
    x0, x1 = 16 * mm, W - 16 * mm
    y = H - 40 * mm
    # photo
    c.setFillColor(HexColor('#efe9dd'))
    c.setStrokeColor(LINE)
    c.roundRect(x1 - 36 * mm, y - 40 * mm, 36 * mm, 40 * mm, 3 * mm, stroke=1, fill=1)
    c.setFillColor(MUTED)
    font(c, 'Onest-Regular', 8)
    c.drawCentredString(x1 - 18 * mm, y - 21 * mm, tx(T['photo']))
    def field(label, key, x, y, w, h=8 * mm):
        c.setFillColor(GREEN)
        font(c, 'Onest-SemiBold', 7)
        c.drawString(x, y + h - 1, tx(label.upper()))
        c.setStrokeColor(LINE)
        c.setLineWidth(0.6)
        c.line(x, y, x + w, y)
        if form:
            form.textfield(name=key, tooltip=label, x=x, y=y + 0.5, width=w, height=h - 3.5 * mm, fontName='Helvetica', fontSize=9, borderWidth=0, fillColor=None, textColor=INK)
    def section(title, y):
        c.setFillColor(INK)
        font(c, 'Literata-SemiBold', 12)
        c.drawString(x0, y, tx(title))
        c.setStrokeColor(GREEN)
        c.setLineWidth(1.2)
        c.line(x0, y - 2.5 * mm, x0 + 14 * mm, y - 2.5 * mm)
    wl = x1 - 42 * mm - x0
    section(T['s1'], y)
    y -= 15 * mm
    for lab, key in [(T['first'], 'imie'), (T['last'], 'nazwisko'), (T['maiden'], 'rodowe')]:
        field(lab, key, x0, y, wl)
        y -= 11 * mm
    half = (x1 - x0 - 6 * mm) / 2
    for (l1, k1), (l2, k2) in [((T['bdate'], 'ur_data'), (T['bplace'], 'ur_miejsce')), ((T['ddate'], 'zm_data'), (T['dplace'], 'zm_miejsce')),
                               ((T['occ'], 'zawod'), (T['rel'], 'wyznanie')), ((T['burial'], 'grob'), (T['cause'], 'przyczyna'))]:
        field(l1, k1, x0, y, half)
        field(l2, k2, x0 + half + 6 * mm, y, half)
        y -= 11 * mm
    y -= 4 * mm
    section(T['s2'], y)
    y -= 15 * mm
    for (l1, k1), (l2, k2) in [((T['fa'], 'ojciec'), (T['mo'], 'matka')), ((T['spouse'], 'malzonek'), (T['wed'], 'slub'))]:
        field(l1, k1, x0, y, half)
        field(l2, k2, x0 + half + 6 * mm, y, half)
        y -= 11 * mm
    field(T['children'], 'dzieci', x0, y, x1 - x0)
    y -= 11 * mm
    field(T['siblings'], 'rodzenstwo', x0, y, x1 - x0)
    y -= 15 * mm
    section(T['s3'], y)
    y -= 15 * mm
    for i in range(4):
        field(T['memories'] if i == 0 else '', f'historia{i}', x0, y, x1 - x0)
        y -= 9 * mm
    y -= 2 * mm
    for i in range(2):
        field(T['sources'] if i == 0 else '', f'zrodla{i}', x0, y, x1 - x0)
        y -= 9 * mm
    footer(c, W)
    c.save()


def build(code, strings, outdir, files):
    """files: the six PDF names (no extension) in the order of the templates page."""
    global T, RTL
    import os
    T = strings
    RTL = code == 'he'
    if RTL:
        FN.update({'Literata-SemiBold': 'DejaVu-Bold', 'Onest-Regular': 'DejaVu', 'Onest-SemiBold': 'DejaVu-Bold'})
    else:
        FN.update({'Literata-SemiBold': 'Literata-SemiBold', 'Onest-Regular': 'Onest-Regular', 'Onest-SemiBold': 'Onest-SemiBold'})
    os.makedirs(outdir, exist_ok=True)
    four_generations(outdir + files[0] + '.pdf', False)
    four_generations(outdir + files[1] + '.pdf', True)
    with_photos(outdir + files[2] + '.pdf')
    fan(outdir + files[3] + '.pdf')
    person_card(outdir + files[4] + '.pdf', False)
    person_card(outdir + files[5] + '.pdf', True)
