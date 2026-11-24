# Builds the static myroots.cc in 12 languages. Polish at the root, the others in /<code>/.
# All texts live in lang/<code>.py; this file holds the HTML structure only.
# Relative links everywhere, so the site works from the domain root and from a preview.
import json, os, html, re, shutil, importlib.util

SITE = 'https://myroots.cc'
HERE = os.path.dirname(os.path.abspath(__file__))
# Where the site is written (MYROOTS_SITE_OUT), by default "out" next to this file.
OUT = os.path.abspath(os.environ.get('MYROOTS_SITE_OUT') or os.path.join(HERE, 'out')) + '/'
# Files that are not generated (images, fonts, style, templates, .htaccess) are
# copied from ../static when that folder exists (the layout of the repository).
STATIC = os.path.join(HERE, '..', 'static')
# The lists of changes of the app ("What's new" page): CHANGELOG.md (Polish)
# and CHANGELOG.en.md (all other languages), by default two levels up.
CHANGELOG_DIR = os.path.abspath(os.environ.get('MYROOTS_CHANGELOG_DIR') or os.path.join(HERE, '..', '..'))
DEMO = 'https://demo.myroots.cc'


def read_changelog(name):
    p = os.path.join(CHANGELOG_DIR, name)
    if not os.path.exists(p):
        return []
    rels, cur = [], None
    for line in open(p, encoding='utf-8').read().splitlines():
        m = re.match(r'^##\s+v?(\d+\.\d+\.\d+)\s*\((\d{4}-\d{2}-\d{2})\)', line)
        if m:
            cur = {'version': m.group(1), 'date': m.group(2), 'lines': []}
            rels.append(cur)
        elif cur is not None and not line.startswith('# '):
            cur['lines'].append(line)
    return rels


CHANGES = {'pl': read_changelog('CHANGELOG.md'), 'en': read_changelog('CHANGELOG.en.md')}
# Current version (MYROOTS_VERSION), by default the newest in the changelog.
VERSION = os.environ.get('MYROOTS_VERSION') or (CHANGES['pl'] or CHANGES['en'] or [{'version': '3.0.0'}])[0]['version']
# Download files keep the same names in every version (the release workflow
# uploads new files under these names; .htaccess sends the old versioned
# names of 3.0.0 here).
DL = {
    'mac': 'downloads/MyRoots-mac.dmg',
    'win': 'downloads/MyRoots-windows-setup.exe',
    'lin': 'downloads/MyRoots-linux.AppImage',
    'deb': 'downloads/MyRoots-linux.deb',
    'rpm': 'downloads/MyRoots-linux.rpm',
    'apk': 'downloads/MyRoots-android.apk',
    'wp': 'downloads/myroots.zip',
    'srv': 'downloads/myroots-server.zip',
}
DL_EXT = {'mac': '.dmg', 'win': '.exe', 'lin': '.AppImage', 'apk': '.apk', 'wp': '.zip', 'srv': '.zip'}
SUPPORT_PL = 'https://suppi.pl/jarekhirsch'
SUPPORT_INTL = 'https://ko-fi.com/jarekhirsch'


def support(L):
    """Suppi for the Polish site, Ko-fi for every other language."""
    return SUPPORT_PL if L['code'] == 'pl' else SUPPORT_INTL

AUTHOR_URL = 'https://hirsch.lt/'
STORE_URL = 'https://apps.microsoft.com/detail/9MX7K9QKVVCW'
# Umami Cloud (cookieless visit counting). Empty UMAMI_ID = no script on the site.
UMAMI_ID = '98f82dd4-ad2a-4f8f-8da0-1e54c28b820e'
UMAMI_SRC = 'https://cloud.umami.is/script.js'
GITHUB_USER = 'https://github.com/jarekhirsch'
GITHUB = GITHUB_USER + '/myroots'
UPDATED = '2026-11-13'
SITEMAP_DATE = '2026-11-19'  # lastmod in sitemap.xml: date of the last change of the site
SRC = {
    'SRC_MH_FREE': 'https://www.myheritage.com/help/en/articles/12852341-is-myheritage-free',
    'SRC_MH_DN': 'https://techlicious.com/blog/myheritage-photo-animation',
    'SRC_GRAMPS': 'https://gramps-project.org/',
    'SRC_GRAMPS_WEB': 'https://www.grampsweb.org/',
    'SRC_GRAMPS_G7': 'https://gramps.discourse.group/t/gramps-6-0-and-gedcom-7/6950',
}
LANGS = ['pl', 'en', 'de', 'fr', 'es', 'it', 'pt', 'nl', 'sv', 'fi', 'da', 'he']
PAGES = ['home', 'tpl', 'guide', 'wp', 'priv', 'sign']
SHOW_IMG = ['drzewo-genealogiczne-z-datami-os-czasu', 'profil-osoby-ze-zdjeciami', 'album-rodzinny-galeria', 'zwierzeta-w-drzewie-genealogicznym', 'plakat-wachlarz-przodkow', 'animacja-przez-lata', 'drzewo-genealogiczne-aplikacja-telefon']
SHOW_KEY = ['os', 'profil', 'galeria', 'zwierzeta', 'plakat', 'lata', 'telefon']
FEAT_IMG = ['animacja-przez-lata', 'ksiega-rodu-do-druku', 'zwierzeta-w-drzewie-genealogicznym', 'oznaczanie-osob-na-zdjeciach']
HERO_IMG = 'drzewo-genealogiczne-online-myroots'

# own line icons, 24x24, drawn with the current colour
ICONS = {
    'laptop': '<rect x="4" y="5" width="16" height="11" rx="1.5"/><path d="M2 19h20"/>',
    'desktop': '<rect x="3" y="4" width="18" height="12" rx="1.5"/><path d="M9 20h6M12 16v4"/>',
    'phone': '<rect x="7" y="2.5" width="10" height="19" rx="2"/><path d="M11 18.5h2"/>',
    'globe': '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c3 3.2 3 14.8 0 18M12 3c-3 3.2-3 14.8 0 18"/>',
    'server': '<rect x="4" y="4" width="16" height="7" rx="1.5"/><rect x="4" y="13" width="16" height="7" rx="1.5"/><path d="M8 7.5h.01M8 16.5h.01M12 7.5h4M12 16.5h4"/>',
    'calendar': '<rect x="3.5" y="5" width="17" height="15" rx="2"/><path d="M3.5 10h17M8 3v4M16 3v4M8.5 15c1.2-1.2 2.3-1.2 3.5 0s2.3 1.2 3.5 0"/>',
    'fan': '<path d="M3 18a9 9 0 0 1 18 0z"/><path d="M12 18V9M12 18 5.6 11.6M12 18l6.4-6.4M7.5 18a4.5 4.5 0 0 1 9 0"/>',
    'timeline': '<path d="M3 12h18"/><circle cx="7" cy="12" r="1.6"/><circle cx="15" cy="12" r="1.6"/><path d="M4 7h7M11 17h8"/>',
    'pin': '<path d="M12 21s-6.5-5.6-6.5-10.5a6.5 6.5 0 0 1 13 0C18.5 15.4 12 21 12 21z"/><circle cx="12" cy="10.5" r="2.3"/>',
    'site': '<rect x="3" y="4.5" width="18" height="15" rx="2"/><path d="M3 9h18M6.5 6.8h.01M9 6.8h.01M7 13h6M7 16h10"/>',
    'lock': '<rect x="5" y="10.5" width="14" height="10" rx="2"/><path d="M8 10.5V8a4 4 0 0 1 8 0v2.5M12 14.5v2"/>',
    'trees': '<circle cx="7.5" cy="10" r="4"/><path d="M7.5 14v7"/><circle cx="17" cy="12.5" r="3"/><path d="M17 15.5V21M3 21h18"/>',
    'album': '<rect x="3" y="4" width="18" height="16" rx="1.5"/><rect x="6" y="7" width="5" height="4.5"/><rect x="13" y="7" width="5" height="4.5"/><rect x="6" y="14" width="12" height="3"/>',
    'kin': '<circle cx="6" cy="6.5" r="2.5"/><circle cx="18" cy="6.5" r="2.5"/><circle cx="12" cy="18" r="2.5"/><path d="M7.3 8.7l3.4 7M16.7 8.7l-3.4 7"/>',
    'lang': '<path d="M4 5h16v11H10l-4 3.5V16H4z"/><path d="M9 13.5l3-6 3 6M10 11.5h4"/>',
    'home': '<path d="M4 11 12 4l8 7v9H4z"/><path d="M10 20v-5h4v5"/>',
    'block': '<rect x="4" y="4" width="16" height="16" rx="2"/><path d="M8 9h8M8 12h8M8 15h5"/>',
    'eyeoff': '<path d="M3 12s3.5-6 9-6 9 6 9 6-3.5 6-9 6-9-6-9-6z"/><circle cx="12" cy="12" r="2.5"/><path d="M4 20 20 4"/>',
    'pen': '<path d="M4 20l1-4L16 5l3 3L8 19z"/><path d="M14 7l3 3"/>',
    'user': '<circle cx="12" cy="8" r="3.5"/><path d="M5 20c.8-3.6 3.6-5.5 7-5.5s6.2 1.9 7 5.5"/>',
    'sync': '<path d="M5 9a7 7 0 0 1 12.5-3M19 15a7 7 0 0 1-12.5 3"/><path d="M17.5 2.5V6H14M6.5 21.5V18H10"/>',
}
MORE_IC = ['calendar', 'fan', 'timeline', 'pin', 'site', 'lock', 'trees', 'album', 'kin', 'lang']
WP_IC = ['home', 'block', 'eyeoff', 'pen', 'user', 'sync', 'trees', 'lock', 'phone']
DL_IC = {'mac': 'laptop', 'win': 'desktop', 'lin': 'desktop', 'apk': 'phone', 'wp': 'globe', 'srv': 'server'}


def icon(name):
    return f'<svg class="ic" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">{ICONS[name]}</svg>'


def load(code):
    path = os.path.join(HERE, 'lang', code + '.py')
    if not os.path.exists(path):
        return None
    spec = importlib.util.spec_from_file_location('lang_' + code, path)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m.L


ALL = {c: load(c) for c in LANGS}
ALL = {c: l for c, l in ALL.items() if l}

# "What's new" and the Linux download: texts in lang_news.py
from lang_news import NEWS, STACK, STACK_MORE  # noqa: E402
from stack_logos import LOGOS  # noqa: E402
for _c, _L in ALL.items():
    _N = NEWS[_c]
    _L['slug']['news'] = _N['slug']
    _L['news'] = _N
    _L['home']['dl']['lin'] = _N['lin']
PAGES = PAGES + ['news']

if os.path.isdir(STATIC):
    shutil.copytree(STATIC, OUT, dirs_exist_ok=True)
os.makedirs(OUT, exist_ok=True)


def size(rel):
    p = OUT + rel
    if not os.path.exists(p):
        return ''
    return f'{os.path.getsize(p) / 1e6:.0f} MB'


def ld(obj):
    return '<script type="application/ld+json">' + json.dumps(obj, ensure_ascii=False).replace('</', '<\\/') + '</script>'


def strip(s):
    return html.unescape(re.sub(r'<[^>]+>', '', s))


def url(L, page):
    return SITE + '/' + L['prefix'] + L['slug'][page]


class Ctx:
    """Placeholders of one page: where the site root and the language root are, relative to it."""

    def __init__(self, L, page):
        self.L, self.page = L, page
        path = L['prefix'] + L['slug'][page]
        self.R = '../' * path.count('/')
        self.Lr = self.R + L['prefix']
        self.img_dir = L['img']

    def fill(self, s):
        L = self.L
        rep = {'{R}': self.R, '{L}': self.Lr or './', '{DEMO}': DEMO, '{SUPPORT}': support(L), '{AUTHOR_URL}': AUTHOR_URL,
               '{GITHUB}': GITHUB, '{GITHUB_USER}': GITHUB_USER, '{UPDATED}': L['updated'], '{DL_WP}': self.R + DL['wp'],
               '{STORE}': STORE_URL, '{DL_WIN}': self.R + DL['win'], '{DL_DEB}': self.R + DL['deb'], '{DL_RPM}': self.R + DL['rpm']}
        for k in PAGES:
            rep['{S_' + k + '}'] = (self.Lr + L['slug'][k]) or './'
        rep.update({'{' + k + '}': v for k, v in SRC.items()})
        for k, v in rep.items():
            s = s.replace(k, v)
        return re.sub(r'\{FIG:([^|}]+)\|([^|}]+)\|([^}]+)\}', lambda m: f'<figure class="shot-hero">{self.img(m.group(1), m.group(2))}<figcaption class="shot-cap">{m.group(3)}</figcaption></figure>', s)

    def img(self, name, alt, cls='shot', lazy=True):
        d = self.R + self.img_dir
        if not os.path.exists(OUT + self.img_dir + name + '-1600.webp'):
            d = self.R + 'img/'
        return f'<img class="{cls}" src="{d}{name}-1600.webp" srcset="{d}{name}-800.webp 800w, {d}{name}-1600.webp 1600w" sizes="(max-width: 860px) 100vw, 860px" width="1600" height="1000" alt="{html.escape(alt)}"{" loading=\"lazy\"" if lazy else ""} decoding="async">'

    def imgpath(self, name):
        if os.path.exists(OUT + self.img_dir + name):
            return self.R + self.img_dir + name
        return self.R + 'img/' + name


def faq_html(items):
    return '<div class="faq-list">' + ''.join(f'<details><summary>{q}</summary><p>{a}</p></details>' for q, a in items) + '</div>'


def faq_ld(items):
    return {'@context': 'https://schema.org', '@type': 'FAQPage', 'mainEntity': [
        {'@type': 'Question', 'name': q, 'acceptedAnswer': {'@type': 'Answer', 'text': strip(a)}} for q, a in items]}


def crumbs_ld(L, page, name):
    return {'@context': 'https://schema.org', '@type': 'BreadcrumbList', 'itemListElement': [
        {'@type': 'ListItem', 'position': 1, 'name': 'MyRoots', 'item': url(L, 'home')},
        {'@type': 'ListItem', 'position': 2, 'name': name, 'item': url(L, page)}]}


def crumbs(c, name):
    return f'<nav class="crumbs" aria-label="{c.L["ui"]["crumbs"]}"><a href="{c.Lr or "./"}">MyRoots</a> › {name}</nav>'


LOGO = '<svg viewBox="0 0 36 34" aria-hidden="true"><path fill="currentColor" d="M29.9,5.3c-0.3,0.1-1.2,0.3-1.6,0.4c0,0,0,0-0.1,0c-0.1,0-0.3,0-0.4,0.1c-1.3,0.2-2.7,0-4-0.1c-0.6,0-1.2-0.1-1.8-0.3c-0.6-0.2-1.2-0.3-1.8-0.4c-0.4-0.1-0.9-0.1-1.4-0.2c0-1.1,0-2.3,0-3.4c-0.3,0-0.6,0-0.9,0s-0.6,0-0.9,0c0,1.1,0,2.2,0,3.4c-0.5,0.1-0.9,0.1-1.4,0.2c-0.6,0.1-1.2,0.2-1.8,0.4c-0.6,0.2-1.2,0.3-1.8,0.3c-1.3,0.1-2.6,0.3-4,0.1c-0.1,0-0.3,0-0.4-0.1c0,0,0,0-0.1,0C7.3,5.6,6.4,5.4,6.1,5.3c0,0.6,0.1,1.2,0.2,1.8c1,0.4,2.2,0.4,3.3,0.4C8.7,8.3,7.8,8.9,7,9.6c0.2,0.6,0.5,1.1,0.8,1.6c0.4-0.3,0.8-0.6,1.1-0.9c1.2-0.9,2.3-1.8,3.6-2.5c1.4-0.7,2.8-1.2,4.4-1.3c0.1,0,0.1,0,0.2,0c0,0.5,0,1,0,1.5c0,0-0.1,0.1-0.2,0.1c-1.6,0.2-3.1,1-4.4,1.9c-1.1,0.7-2.2,1.5-3.4,2c-0.2,0.1-0.4,0.2-0.7,0.2c0.4,0.5,0.8,1,1.2,1.4c1.3-0.5,2.5-1.3,3.7-2.1c-0.5,1.2-0.7,2.6-1.8,3.6c0.5,0.3,1.1,0.6,1.6,0.9c0.3-0.3,0.6-0.7,0.7-1.1c0.4-0.9,0.7-1.8,1-2.7c0.2-0.5,0.4-1.1,0.7-1.6c0.3-0.6,0.6-0.8,1.4-0.8c-0.1,2.5-1.2,4.6-2.2,6.8c0.6,0.2,1.2,0.3,1.8,0.3c0.5-1.1,1.3-2.9,1.3-2.9s0.8,1.9,1.3,2.9c0.6-0.1,1.2-0.2,1.8-0.3c-0.9-2.2-2.1-4.3-2.2-6.8c0.8,0,1.1,0.2,1.4,0.8c0.3,0.5,0.5,1,0.7,1.6c0.4,0.9,0.7,1.8,1,2.7c0.2,0.4,0.4,0.8,0.7,1.1c0.6-0.2,1.1-0.5,1.6-0.9c-1.1-1-1.3-2.4-1.8-3.6c1.2,0.7,2.4,1.6,3.7,2.1c0.4-0.4,0.9-0.9,1.2-1.4c-0.2-0.1-0.5-0.2-0.7-0.2c-1.2-0.5-2.3-1.3-3.4-2c-1.4-0.9-2.8-1.6-4.4-1.9C19,8.1,18.9,8,18.9,8c0-0.5,0-1,0-1.5c0.1,0,0.2,0,0.2,0c1.6,0.1,3,0.6,4.4,1.3c1.3,0.7,2.4,1.6,3.6,2.5c0.4,0.3,0.7,0.6,1.1,0.9c0.3-0.5,0.6-1,0.8-1.6c-0.9-0.7-1.7-1.3-2.6-2c1.2,0,2.3,0,3.3-0.4C29.8,6.5,29.9,5.9,29.9,5.3z"/></svg>'


def page(c, title, desc, body, lds, og_img=None, script='', file=None, index=True):
    L, R, Lr = c.L, c.R, c.Lr
    pg = c.page
    nav = ''
    for k in ['home', 'tpl', 'guide', 'wp']:
        cur = ' aria-current="page"' if k == pg and index else ''
        nav += f'<a href="{(Lr + L["slug"][k]) or "./"}"{cur}>{L["nav"][k]}</a>'
    nav += f'<a href="{support(L)}">{L["nav"]["support"]}</a>'
    # language switcher: the same page in every language
    langs = ''.join(f'<li><a href="{(R + o["prefix"] + o["slug"][pg]) or "./"}" hreflang="{o["code"]}" lang="{o["code"]}"{" aria-current=\"true\"" if o is L else ""}>{o["name"]}</a></li>' for o in ALL.values())
    switch = f'<details class="langs"><summary aria-label="{L["ui"]["lang"]}: {L["name"]}">{L["code"].upper()}</summary><ul>{langs}</ul></details>'
    alts = ''
    if index:
        alts = ''.join(f'<link rel="alternate" hreflang="{o["code"]}" href="{url(o, pg)}">\n' for o in ALL.values())
        alts += f'<link rel="alternate" hreflang="x-default" href="{url(ALL["pl"], pg)}">\n'
    canon = f'<link rel="canonical" href="{url(L, pg)}">\n' if index else '<meta name="robots" content="noindex">\n'
    og = og_img or (L['img'] + HERO_IMG + '-1600.webp')
    if not os.path.exists(OUT + og):
        og = 'img/' + HERO_IMG + '-1600.webp'
    f = L['foot']
    T, G = (Lr + L['slug']['tpl']), (Lr + L['slug']['guide'])
    tpl_links = ''.join(f'<li><a href="{T}#{a}">{n}</a></li>' for a, n in f['tpl_links'])
    guide_links = ''.join(f'<li><a href="{G}{"#" + a if a else ""}">{n}</a></li>' for a, n in f['guide_links'])
    home = Lr or './'
    utm = '?utm_source=myroots&amp;utm_medium=footer&amp;utm_campaign=brand'
    umami = f'<script defer src="{UMAMI_SRC}" data-website-id="{UMAMI_ID}"></script>\n' if UMAMI_ID else ''
    doc = f'''<!doctype html>
<html lang="{L['code']}" dir="{L['dir']}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover">
<title>{html.escape(title)}</title>
<meta name="description" content="{html.escape(desc)}">
{canon}{alts}<meta name="theme-color" content="#1f4a37">
<meta name="author" content="Jarek Hirsch">
<meta property="og:type" content="website">
<meta property="og:locale" content="{L['locale']}">
<meta property="og:site_name" content="MyRoots">
<meta property="og:title" content="{html.escape(title)}">
<meta property="og:description" content="{html.escape(desc)}">
<meta property="og:url" content="{url(L, pg)}">
<meta property="og:image" content="{SITE}/{og}">
<meta property="og:image:width" content="1600">
<meta property="og:image:height" content="1000">
<meta name="twitter:card" content="summary_large_image">
<link rel="icon" type="image/svg+xml" href="{R}favicon.svg">
<link rel="preload" href="{R}fonts/onest-latin-wght-normal.woff2" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="{R}style.css">
{''.join(ld(x) for x in lds)}
{umami}</head>
<body>
<a class="skip" href="#main">{L['ui']['skip']}</a>
<header class="top">
<div class="wrap">
<a class="logo" href="{home}" aria-label="{L['ui']['logo']}">{LOGO}MyRoots</a>
<nav class="nav" aria-label="{L['ui']['menu']}">{nav}</nav>
{switch}
<a class="btn btn-primary" href="{home}#pobierz">{L['ui']['cta']}</a>
</div>
<nav class="mnav" aria-label="{L['ui']['menu']}">{nav}</nav>
</header>
<main id="main">
{body}
</main>
<footer><div class="wrap foot">
<div class="foot-links">
<div><h2>{f['app']}</h2><ul><li><a href="{home}#pobierz">{f['dl']}</a></li><li><a href="{Lr}{L['slug']['news']}">{L['news']['link']}</a></li><li><a href="{DEMO}">{f['demo']}</a></li><li><a href="{Lr}{L['slug']['wp']}">{f['wp']}</a></li><li><a href="{home}#porownanie">{f['cmp']}</a></li></ul></div>
<div><h2>{f['tpl']}</h2><ul>{tpl_links}</ul></div>
<div><h2>{f['guide']}</h2><ul>{guide_links}</ul></div>
<div><h2>{f['author']}</h2><ul><li><a href="{home}#autor">{f['who']}</a></li><li><a href="{support(L)}">{f['support']}</a></li><li><a href="{AUTHOR_URL}{utm}">hirsch.lt</a></li></ul></div>
</div>
<div class="foot-line">
<a href="{Lr}{L['slug']['news']}">MyRoots {VERSION}</a><a href="https://www.gnu.org/licenses/gpl-3.0.html">{f['license']}</a><a href="{Lr}{L['slug']['priv']}">{f['priv']}</a><a href="{Lr}{L['slug']['sign']}">{f['sign']}</a><span class="sp">© 2026 <a href="{AUTHOR_URL}{utm}">Jarek Hirsch</a></span>
</div>
</div></footer>
{script}
</body>
</html>
'''
    target = OUT + (file if file else L['prefix'] + L['slug'][pg] + 'index.html')
    os.makedirs(os.path.dirname(target), exist_ok=True)
    with open(target, 'w') as fh:
        fh.write(doc)


TAB_JS = """<script>
(function(){var tabs=[].slice.call(document.querySelectorAll('.tab'));function sel(t,f){tabs.forEach(function(x){var on=x===t;x.setAttribute('aria-selected',on);x.tabIndex=on?0:-1;var p=document.getElementById(x.getAttribute('aria-controls'));if(p)p.hidden=!on;});if(f)t.focus();}
tabs.forEach(function(t,i){t.addEventListener('click',function(){sel(t)});t.addEventListener('keydown',function(e){var d=e.key==='ArrowDown'||e.key==='ArrowRight'?1:e.key==='ArrowUp'||e.key==='ArrowLeft'?-1:0;if(document.dir==='rtl'&&(e.key==='ArrowRight'||e.key==='ArrowLeft'))d=-d;if(d){e.preventDefault();sel(tabs[(i+d+tabs.length)%tabs.length],true)}})});
var top=document.querySelector('.top');addEventListener('scroll',function(){top.classList.toggle('scrolled',scrollY>8)},{passive:true});
document.addEventListener('click',function(e){document.querySelectorAll('details.langs[open]').forEach(function(d){if(!d.contains(e.target))d.open=false})});})();
</script>"""
LANG_JS = """<script>document.addEventListener('click',function(e){document.querySelectorAll('details.langs[open]').forEach(function(d){if(!d.contains(e.target))d.open=false})});</script>"""


# ===================================================================== home
def build_home(L):
    c = Ctx(L, 'home')
    H = L['home']
    F = c.fill
    tabs = ''.join(f'<button class="tab" role="tab" id="t-{k}" aria-controls="p-{k}" aria-selected="{"true" if i == 0 else "false"}"{"" if i == 0 else " tabindex=\"-1\""}>{H["show"][i][0]}</button>' for i, k in enumerate(SHOW_KEY))
    panes = ''
    for i, k in enumerate(SHOW_KEY):
        name = SHOW_IMG[i] + ('-780.webp' if k == 'telefon' else '-1600.webp')
        src = c.imgpath(name)
        cap = H['show'][i][1]
        panes += f'<div class="panel-pane" id="p-{k}" role="tabpanel" aria-labelledby="t-{k}"{"" if i == 0 else " hidden"}><div class="panel-frame"><img src="{src}" alt="{html.escape(cap)}" loading="lazy"{" class=\"contain\"" if k == "telefon" else ""}></div><div class="panel-meta"><span>{cap}</span><a href="{src}">{L["ui"]["full"]}</a></div></div>'
    feats = ''
    for i, (alt, h, p, aside) in enumerate(H['feats']):
        feats += f'<div class="feat{" flip" if i % 2 else ""}"><figure class="shot">{c.img(FEAT_IMG[i], alt)}</figure><div class="feat-text"><h2>{h}</h2><p>{p}</p><p class="aside">{aside}</p></div></div>\n'
    spec = ''.join(f'<div><dt>{a}</dt><dd>{b}</dd></div>' for a, b in H['spec'])
    more = ''.join(f'<li><h3>{icon(MORE_IC[i % len(MORE_IC)])}{a}</h3><p>{b}</p></li>' for i, (a, b) in enumerate(H['more']))
    st_h, st_p, st_items = STACK[L['code']]
    st_note = re.sub(r'\[([^\]]+)\]', lambda m: f'<a href="{GITHUB}">{m.group(1)}</a>', st_p)
    # a strip of the greyed logos; the description shows on hover and focus
    desc = {a.split(' ')[0]: b for a, b in st_items}
    desc.update(STACK_MORE[L['code']])
    stack = ''
    for n, path in LOGOS.items():
        d = html.escape(desc.get(n, ''))
        stack += (f'<li tabindex="0" aria-describedby="lg-{n.lower()}"><svg viewBox="0 0 24 24" role="img" aria-label="{n}"><path d="{path}"/></svg>'
                  f'<span class="lg-tip" id="lg-{n.lower()}" role="tooltip"><b lang="en" dir="ltr">{n}</b> {d}</span></li>')

    def row(k):
        name, req, how, label = H['dl'][k]
        s = size(DL[k])
        meta = f'{DL_EXT[k]} · {s}' if s else DL_EXT[k]
        href, dl = f'{c.R}{DL[k]}', ' download'
        if k == 'win':  # Windows: Microsoft Store (signed by Microsoft); the .exe stays as a link in `how`
            href, dl, meta = STORE_URL, '', 'Microsoft Store'
        return f'<li><div class="dl-name"><h4>{icon(DL_IC[k])}{name}</h4><p>{req}</p></div><p class="dl-how">{how}</p><div class="dl-get"><a class="btn btn-ghost btn-sm" href="{href}"{dl} data-umami-event="download-{k}">{label}</a><span>{meta}</span></div></li>'
    apps = ''.join(row(k) for k in ('mac', 'win', 'lin', 'apk'))
    web = ''.join(row(k) for k in ('wp', 'srv'))
    head = ''.join(f'<th{" class=\"us\"" if i == 0 else ""}>{a}<small>{b}</small></th>' for i, (a, b) in enumerate(H['cmp_head']))

    def td(v, us):
        cls = (['us'] if us else []) + (['no'] if v.startswith('~') else [])
        return f'<td{" class=\"" + " ".join(cls) + "\"" if cls else ""}>{v.lstrip("~")}</td>'
    rows = ''.join(f'<tr><th>{r[0]}</th>{td(r[1], True)}{td(r[2], False)}{td(r[3], False)}</tr>' for r in H['cmp_rows'])
    plat = ''.join(f'<li>{x}</li>' for x in H['plat_list'])
    body = f'''
<section class="hero" aria-labelledby="hero-h"><div class="wrap">
<div class="hero-top">
<div><h1 id="hero-h" class="long">{H['h1']}</h1></div>
<div>
<p class="lead">{H['lead']}</p>
<div class="ctas"><a class="btn btn-primary" href="#pobierz">{L['ui']['cta']}</a><a class="btn btn-ghost" href="{DEMO}">{H['demo']}</a></div>
<p class="facts">{H['facts']}</p>
</div>
</div>
<figure class="shot-hero">{c.img(HERO_IMG, H['hero_alt'], cls='', lazy=False)}<figcaption class="shot-cap">{H['hero_cap']}</figcaption></figure>
</div></section>

<section class="sec pitch" aria-label="{H['pitch_label']}"><div class="wrap pitch-grid">
<div><h2>{H['free_h']}</h2><p class="big">{H['free_p']}</p></div>
<div class="plat"><h2>{H['plat_h']}</h2><p>{H['plat_p']}</p><ul class="plat-list">{plat}</ul></div>
</div></section>

<section class="show" id="zobacz" aria-labelledby="show-h"><div class="wrap"><div class="show-grid">
<div>
<h2 id="show-h">{H['show_h']}</h2>
<p class="sub">{H['show_sub']}</p>
<div class="tabs" role="tablist" aria-label="{L['ui']['views']}">{tabs}</div>
</div>
<div class="panel">{panes}</div>
</div></div></section>

<section class="sec feats" aria-label="{H['feats_label']}"><div class="wrap">
{feats}</div></section>

<section class="sec gedcom" id="gedcom" aria-labelledby="ged-h"><div class="wrap gedcom-grid">
<div><h2 id="ged-h">{H['ged_h']}</h2>{''.join(f'<p>{p}</p>' for p in H['ged_p'])}</div>
<dl class="spec">{spec}</dl>
</div></section>

<section class="sec more" aria-labelledby="more-h"><div class="wrap">
<h2 id="more-h">{H['more_h']}</h2>
<ul class="more-list">{more}</ul>
</div></section>

<section class="logos" id="technologia" aria-labelledby="stack-h"><div class="wrap">
<h2 id="stack-h">{st_h}</h2>
<ul class="logo-strip">{stack}</ul>
<p class="logo-note">{st_note}</p>
</div></section>

<section class="sec dl-sec" id="pobierz" aria-labelledby="dl-h"><div class="wrap">
<h2 id="dl-h">{H['dl_h']}</h2>
<p class="sub">{H['dl_sub']}</p>
<h3 class="dl-group">{H['dl_apps']}</h3>
<ul class="dl-list">{apps}</ul>
<h3 class="dl-group">{H['dl_web']}</h3>
<ul class="dl-list">{web}</ul>
<p class="note">{H['dl_note']}</p>
</div></section>

<section class="compare" id="porownanie" aria-labelledby="cmp-h"><div class="wrap">
<h2 id="cmp-h">{H['cmp_h']}</h2>
<p class="sub">{H['cmp_sub']}</p>
<div class="table-scroll"><table>
<thead><tr><th></th>{head}</tr></thead>
<tbody>{rows}</tbody></table></div>
<p class="src">{H['src']}</p>
</div></section>

<section class="author" id="autor" aria-labelledby="auth-h"><div class="wrap author-grid">
<div><h2 id="auth-h">{H['auth_h']}</h2></div>
<div>{''.join(f'<p>{p}</p>' for p in H['auth_p'])}
<div class="links"><a class="btn btn-primary" href="{support(L)}">{H['auth_btn']}</a><a class="btn btn-ghost" href="{AUTHOR_URL}?utm_source=myroots&amp;utm_medium=author&amp;utm_campaign=brand">{H['auth_more']}</a></div></div>
</div></section>

<section class="faq" id="faq" aria-labelledby="faq-h"><div class="wrap">
<h2 id="faq-h">{H['faq_h']}</h2>
{faq_html(H['faq'])}
</div></section>

<section class="final" aria-labelledby="fin-h"><div class="wrap">
<h2 id="fin-h">{H['fin_h']}</h2>
<p>{H['fin_p']}</p>
<div class="ctas"><a class="btn btn-primary" href="#pobierz">{L['ui']['cta']}</a><a class="btn btn-ghost" href="{DEMO}">{H['demo']}</a></div>
</div></section>
'''
    body = F(body)
    lds = [
        {'@context': 'https://schema.org', '@type': 'SoftwareApplication', 'name': 'MyRoots', 'applicationCategory': 'LifestyleApplication', 'operatingSystem': 'macOS, Windows, Linux, Android, WordPress, PHP',
         'downloadUrl': [STORE_URL] + [SITE + '/' + DL[k] for k in DL if os.path.exists(OUT + DL[k])], 'description': H['ld_desc'], 'featureList': H['ld_features'],
         'url': url(L, 'home'), 'softwareVersion': VERSION, 'inLanguage': L['code'], 'offers': {'@type': 'Offer', 'price': '0', 'priceCurrency': 'EUR'},
         'author': {'@type': 'Person', 'name': 'Jarek Hirsch', 'url': AUTHOR_URL}, 'screenshot': SITE + '/' + L['img'] + HERO_IMG + '-1600.webp', 'license': 'https://www.gnu.org/licenses/gpl-3.0.html'},
        {'@context': 'https://schema.org', '@type': 'WebSite', 'name': 'MyRoots', 'url': url(L, 'home'), 'inLanguage': L['code']},
        faq_ld([(q, c.fill(a)) for q, a in H['faq']]),
    ]
    page(c, H['title'], H['desc'], body, lds, script=TAB_JS)


# ===================================================================== templates
def build_tpl(L):
    c = Ctx(L, 'tpl')
    T = L['tpl']
    base = c.Lr + L['slug']['tpl']
    cards = ''
    for a, f, t, d, m in T['items']:
        pv = c.R + L['tpl_pv'].format(f=f)
        cards += f'<article class="tpl" id="{a}"><div class="pv"><img src="{pv}" alt="{html.escape(T["pv_alt"].format(t=t))}" loading="lazy"></div><div class="bd"><h3>{t}</h3><p>{d}</p><span class="meta">{m}</span><div><a class="btn btn-ghost btn-sm" href="{base}{f}.pdf" download>{T["btn"]}</a></div></div></article>'
    xl_img = c.R + L['tpl_pv'].format(f=T['xl_file'])
    body = f'''
<section class="hero page"><div class="wrap">
{crumbs(c, T['crumb'])}
<div class="intro">
<h1>{T['h1']}</h1>
<p class="lead">{T['lead']}</p>
<div class="ctas"><a class="btn btn-primary" href="#{T['items'][0][0]}">{T['cta1']}</a><a class="btn btn-ghost" href="#excel">{T['cta2']}</a></div>
</div>
</div></section>

<section class="sec"><div class="wrap">
<h2>{T['pdf_h']}</h2>
<div class="tpls">{cards}</div>
<p class="note" style="margin-top:18px">{T['note']}</p>
</div></section>

<section class="sec" id="excel"><div class="wrap">
<div class="split"><div class="stack">
<h2>{T['xl_h']}</h2>
<p>{T['xl_p']}</p>
<p class="note">{T['xl_note']}</p>
<div><a class="btn btn-primary" href="{base}{T['xl_file']}.xlsx" download>{T['xl_btn']}</a></div>
</div>
<figure class="shot"><img src="{xl_img}" alt="{html.escape(T['xl_alt'])}" loading="lazy"></figure></div>
</div></section>

<section class="sec" id="schemat"><div class="wrap prose">
{T['schema']}
</div></section>

<section class="faq"><div class="wrap">
<h2>{T['faq_h']}</h2>
{faq_html(T['faq'])}
</div></section>
'''
    body = c.fill(body)
    lds = [crumbs_ld(L, 'tpl', T['crumb']),
           {'@context': 'https://schema.org', '@type': 'ItemList', 'name': T['ld_name'], 'itemListElement': [
               {'@type': 'ListItem', 'position': i + 1, 'item': {'@type': 'DigitalDocument', 'name': t, 'description': d, 'encodingFormat': 'application/pdf', 'url': url(L, 'tpl') + f + '.pdf', 'isAccessibleForFree': True, 'inLanguage': L['code']}}
               for i, (a, f, t, d, m) in enumerate(T['items'])] + [
               {'@type': 'ListItem', 'position': len(T['items']) + 1, 'item': {'@type': 'DigitalDocument', 'name': T['xl_h'], 'encodingFormat': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', 'url': url(L, 'tpl') + T['xl_file'] + '.xlsx', 'isAccessibleForFree': True, 'inLanguage': L['code']}}]},
           faq_ld(T['faq'])]
    og = L['img'] + 'plakat-wachlarz-przodkow-1600.webp'
    page(c, T['title'], T['desc'], body, lds, og_img=og, script=LANG_JS)


# ===================================================================== guide
def build_guide(L):
    c = Ctx(L, 'guide')
    G = L['guide']
    steps = ''.join(f'<li><div><h3>{h}</h3><p>{p}</p></div></li>' for h, p in G['steps'])
    body = f'''
<section class="hero page"><div class="wrap">
{crumbs(c, G['crumb'])}
<div class="intro">
<h1>{G['h1']}</h1>
<p class="lead">{G['lead']}</p>
<p class="note" style="margin-top:14px">{G['meta']}</p>
</div>
</div></section>

<section class="sec" id="jak-zaczac"><div class="wrap">
<h2>{G['steps_h']}</h2>
<ol class="steps">{steps}</ol>
</div></section>

<section class="sec" id="jak-sprawdzic"><div class="wrap prose">
{G['body']}
</div></section>

<section class="faq"><div class="wrap">
<h2>{G['faq_h']}</h2>
{faq_html(G['faq'])}
</div></section>
'''
    body = c.fill(body)
    lds = [crumbs_ld(L, 'guide', G['crumb']),
           {'@context': 'https://schema.org', '@type': 'Article', 'headline': G['title'], 'inLanguage': L['code'], 'datePublished': '2026-10-21', 'dateModified': UPDATED,
            'author': {'@type': 'Person', 'name': 'Jarek Hirsch', 'url': AUTHOR_URL}, 'publisher': {'@type': 'Organization', 'name': 'MyRoots', 'url': url(L, 'home')},
            'image': SITE + '/' + L['img'] + HERO_IMG + '-1600.webp', 'mainEntityOfPage': url(L, 'guide')},
           faq_ld([(q, c.fill(a)) for q, a in G['faq']])]
    page(c, G['title'], G['desc'], body, lds, script=LANG_JS)


# ===================================================================== wordpress
def build_wp(L):
    c = Ctx(L, 'wp')
    W = L['wp']
    more = ''.join(f'<li><h3>{icon(WP_IC[i % len(WP_IC)])}{a}</h3><p>{b}</p></li>' for i, (a, b) in enumerate(W['more']))
    steps = ''.join(f'<li><div><h3>{h}</h3><p>{p}</p></div></li>' for h, p in W['steps'])
    body = f'''
<section class="hero page"><div class="wrap">
{crumbs(c, W['crumb'])}
<div class="intro">
<h1>{W['h1']}</h1>
<p class="lead">{W['lead']}</p>
<div class="ctas"><a class="btn btn-primary" href="{c.R}{DL['wp']}" download>{W['btn']}</a><a class="btn btn-ghost" href="{DEMO}">{W['demo']}</a></div>
</div>
<figure class="shot-hero">{c.img('zwierzeta-w-drzewie-genealogicznym', W['img_alt'])}<figcaption class="shot-cap">{W['img_cap']}</figcaption></figure>
</div></section>

<section class="sec more"><div class="wrap">
<h2>{W['more_h']}</h2>
<ul class="more-list">{more}</ul>
</div></section>

<section class="sec"><div class="wrap">
<h2>{W['inst_h']}</h2>
<ol class="steps">{steps}</ol>
</div></section>

<section class="faq"><div class="wrap">
<h2>{W['faq_h']}</h2>
{faq_html(W['faq'])}
</div></section>
'''
    body = c.fill(body)
    lds = [crumbs_ld(L, 'wp', W['crumb']),
           {'@context': 'https://schema.org', '@type': 'SoftwareApplication', 'name': 'MyRoots for WordPress', 'applicationCategory': 'WebApplication', 'operatingSystem': 'WordPress 6.5+',
            'offers': {'@type': 'Offer', 'price': '0', 'priceCurrency': 'EUR'}, 'softwareVersion': '3.0.0', 'url': url(L, 'wp'), 'author': {'@type': 'Person', 'name': 'Jarek Hirsch', 'url': AUTHOR_URL}},
           faq_ld([(q, c.fill(a)) for q, a in W['faq']])]
    page(c, W['title'], W['desc'], body, lds, og_img=L['img'] + 'zwierzeta-w-drzewie-genealogicznym-1600.webp', script=LANG_JS)


# ===================================================================== privacy, code signing
def build_prose(L, key):
    c = Ctx(L, key)
    P = L[key]
    body = f'''
<section class="hero page"><div class="wrap">
{crumbs(c, P['crumb'])}
<div class="intro">
<h1>{P['h1']}</h1>
<p class="lead">{P['lead']}</p>
</div>
</div></section>
<section class="sec"><div class="wrap prose">
{P['body']}
</div></section>
'''
    page(c, P['title'], P['desc'], c.fill(body), [crumbs_ld(L, key, P['crumb'])], script=LANG_JS)


def build_404(L):
    c = Ctx(L, 'home')
    c.R = '/'
    c.Lr = '/' + L['prefix']
    N = L['nf']
    en = ALL.get('en')
    alt = f'<p class="note" style="margin-top:22px" lang="en"><a href="/en/">MyRoots in English</a></p>' if en and L is not en else ''
    body = f'''
<section class="hero page nf"><div class="wrap">
<div class="intro">
<h1>{N['h1']}</h1>
<p class="lead">{N['lead']}</p>
<div class="ctas"><a class="btn btn-primary" href="{c.Lr}">{N['home']}</a><a class="btn btn-ghost" href="{c.Lr}#pobierz">{N['dl']}</a><a class="btn btn-ghost" href="{c.Lr}{L['slug']['tpl']}">{N['tpl']}</a></div>
{alt}
</div>
</div></section>
'''
    page(c, N['title'], N['h1'], c.fill(body), [], file='404.html', index=False, script=LANG_JS)


# ===================================================================== what's new (from the changelog)
def md_inline(t):
    t = html.escape(t, quote=False)
    t = re.sub(r'\*\*(.+?)\*\*', r'<strong>\1</strong>', t)
    t = re.sub(r'`([^`]+)`', r'<code>\1</code>', t)
    t = re.sub(r'\[([^\]]+)\]\((https?://[^)\s]+)\)', r'<a href="\2">\1</a>', t)
    return t


def md_block(lines, level=3):
    """The small Markdown of the changelog: paragraphs, "**Heading**" lines and "- " lists."""
    out, para, items = [], [], []

    def flush():
        if para:
            out.append('<p>' + md_inline(' '.join(para)) + '</p>')
            para.clear()
        if items:
            out.append('<ul>' + ''.join(f'<li>{md_inline(i)}</li>' for i in items) + '</ul>')
            items.clear()
    for raw in lines:
        line = raw.rstrip()
        m = re.match(r'^\*\*(.+)\*\*$', line.strip())
        if not line.strip():
            flush()
        elif m:
            flush()
            out.append(f'<h{level}>{md_inline(m.group(1))}</h{level}>')
        elif line.lstrip().startswith('- '):
            if para:
                flush()
            items.append(line.lstrip()[2:].strip())
        elif items and raw.startswith('  '):
            items[-1] += ' ' + line.strip()
        else:
            if items:
                flush()
            para.append(line.strip())
    flush()
    return '\n'.join(out)


def fmt_date(N, iso):
    y, m, d = iso.split('-')
    return N['date'].format(d=int(d), m=N['months'][int(m) - 1], y=y)


def build_news(L):
    c = Ctx(L, 'news')
    N = L['news']
    rels = CHANGES['pl'] if L['code'] == 'pl' else CHANGES['en']
    arts = ''
    for r in rels:
        rid = 'v' + r['version'].replace('.', '-')
        arts += f'<article class="release" id="{rid}"><h2>{N["version"].format(v=r["version"])} <small><time datetime="{r["date"]}">{fmt_date(N, r["date"])}</time></small></h2>\n{md_block(r["lines"])}</article>\n'
    note = f'<p class="note">{N["note"]}</p>' if N['note'] else ''
    feed = f'{c.Lr}{L["slug"]["news"]}feed.xml'
    body = f'''
<section class="hero page"><div class="wrap">
{crumbs(c, N['crumb'])}
<div class="intro">
<h1>{N['h1']}</h1>
<p class="lead">{N['lead']}</p>
<p class="note"><a href="{feed}">{N['feed']}</a> · <a href="{GITHUB}/blob/main/{'CHANGELOG.md' if L['code'] == 'pl' else 'CHANGELOG.en.md'}">{N['source']}</a></p>
</div>
</div></section>
<section class="sec"><div class="wrap prose news">
{note}
<div{'' if L['code'] in ('pl', 'en') else ' lang="en" dir="ltr"'}>
{arts}</div>
</div></section>
'''
    page(c, N['title'], N['desc'], c.fill(body), [crumbs_ld(L, 'news', N['crumb'])], script=LANG_JS)
    # Atom feed of the releases
    page_url = url(L, 'news')
    upd = rels[0]['date'] if rels else SITEMAP_DATE
    entries = ''.join(
        f'''<entry><title>{html.escape(N["version"].format(v=r["version"]))}</title><id>{page_url}#v{r["version"].replace(".", "-")}</id>
<link href="{page_url}#v{r["version"].replace(".", "-")}"/><updated>{r["date"]}T12:00:00Z</updated>
<content type="html">{html.escape(md_block(r["lines"]))}</content></entry>
''' for r in rels[:20])
    xml = f'''<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xml:lang="{L['code'] if L['code'] == 'pl' else 'en'}">
<title>{html.escape(N['h1'])}</title><id>{page_url}</id><link href="{page_url}"/><link rel="self" href="{page_url}feed.xml"/>
<updated>{upd}T12:00:00Z</updated><author><name>Jarek Hirsch</name></author>
{entries}</feed>
'''
    with open(OUT + L['prefix'] + L['slug']['news'] + 'feed.xml', 'w') as fh:
        fh.write(xml)


for code, L in ALL.items():
    build_news(L)
    build_home(L)
    build_tpl(L)
    build_guide(L)
    build_wp(L)
    build_prose(L, 'priv')
    build_prose(L, 'sign')
build_404(ALL['pl'])

# ===================================================================== robots, sitemap
xml = ['<?xml version="1.0" encoding="UTF-8"?>', '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">']
for L in ALL.values():
    for pg in PAGES:
        xml.append(f'  <url><loc>{url(L, pg)}</loc><lastmod>{SITEMAP_DATE}</lastmod>')
        for o in ALL.values():
            xml.append(f'    <xhtml:link rel="alternate" hreflang="{o["code"]}" href="{url(o, pg)}"/>')
        xml.append(f'    <xhtml:link rel="alternate" hreflang="x-default" href="{url(ALL["pl"], pg)}"/>')
        xml.append('  </url>')
xml.append('</urlset>')
with open(OUT + 'sitemap.xml', 'w') as fh:
    fh.write('\n'.join(xml) + '\n')
with open(OUT + 'robots.txt', 'w') as fh:
    fh.write(f'User-agent: *\nAllow: /\n\nSitemap: {SITE}/sitemap.xml\n')
# ===================================================================== llms.txt (a plain summary for AI assistants)
E = ALL['en']
llm = ['# MyRoots', '',
       '> Free and open source (GNU GPL 3) family tree software by Jarek Hirsch. No account, no subscription, no limit on people; '
       'the family data stays on the user\'s own device or server. For macOS, Windows (Microsoft Store), Android, as a WordPress plugin '
       'and as a self-hosted PHP server, in 12 languages.', '',
       '## Key facts', '',
       '- Price: free, no paid version, no ads, no analytics.',
       '- Licence: GNU GPL 3. Author: Jarek Hirsch (' + AUTHOR_URL + ').',
       '- Languages of the app and of this site: ' + ', '.join(o['name'] for o in ALL.values()) + '.',
       '- Online demo: ' + DEMO,
       '- Privacy: the app collects no data; it goes online only for features the user turns on (map, historical weather, import from the user\'s own WordPress site). See ' + url(E, 'priv'),
       '']
feats = E['home'].get('ld_features')
if feats:
    llm += ['## Features', ''] + ['- ' + f.strip() for f in (feats.split(',') if isinstance(feats, str) else feats)] + ['']
llm += ['## Download', '',
        f'- Windows 10 and 11: [Microsoft Store]({STORE_URL}) (signed by Microsoft, automatic updates), or the [.exe installer]({SITE}/{DL["win"]})',
        f'- macOS 11 or newer: [{DL["mac"].split("/")[-1]}]({SITE}/{DL["mac"]})',
        f'- Linux (64-bit): [AppImage]({SITE}/{DL["lin"]}), [.deb]({SITE}/{DL["deb"]}), [.rpm]({SITE}/{DL["rpm"]})',
        f'- Android 8 or newer: [{DL["apk"].split("/")[-1]}]({SITE}/{DL["apk"]})',
        f'- WordPress plugin: [{DL["wp"].split("/")[-1]}]({SITE}/{DL["wp"]})',
        f'- MyRoots Server (PHP 8 + SQLite, no MySQL needed): [{DL["srv"].split("/")[-1]}]({SITE}/{DL["srv"]})',
        '', '## Pages (English)', '']
for pg in PAGES:
    llm.append(f'- [{E[pg]["title"]}]({url(E, pg)}): {strip(E[pg]["desc"])}')
llm += ['', '## Other languages', '']
for o in ALL.values():
    if o is not E:
        llm.append(f'- {o["name"]}: [{o["home"]["title"]}]({url(o, "home")})')
llm += ['', '## Optional', '', f'- Sitemap with all pages in all languages: {SITE}/sitemap.xml', f'- Support the project: {SUPPORT_INTL}', '']
with open(OUT + 'llms.txt', 'w') as fh:
    fh.write('\n'.join(llm))
print('built', ', '.join(ALL))
