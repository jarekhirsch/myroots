# Generates the printable templates (6 PDFs + 1 Excel) and their preview images in every language.
# Usage: python3 make_templates.py [lang ...]
import os, sys, subprocess, tempfile, importlib.util
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, 'out') + '/'
sys.path.insert(0, HERE)
import templates, xlsx_tpl
from tpl_i18n import P, X


def load(code):
    spec = importlib.util.spec_from_file_location('lang_' + code, os.path.join(HERE, 'lang', code + '.py'))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m.L


def preview(pdf, dst, page=1):
    with tempfile.TemporaryDirectory() as t:
        subprocess.run(['pdftoppm', '-png', '-r', '110', '-f', str(page), '-l', str(page), '-singlefile', pdf, t + '/p'], check=True)
        im = Image.open(t + '/p.png').convert('RGB')
    w = 720
    im = im.resize((w, round(im.height * w / im.width)), Image.LANCZOS)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    im.save(dst, quality=86)


def xlsx_preview(xlsx, dst):
    """Renders the tree sheet with LibreOffice and crops it to the content."""
    with tempfile.TemporaryDirectory() as t:
        subprocess.run(['soffice', '--headless', '--convert-to', 'pdf', '--outdir', t, xlsx], check=True, capture_output=True)
        pdf = os.path.join(t, os.path.basename(xlsx)[:-5] + '.pdf')
        n = int([l for l in subprocess.run(['pdfinfo', pdf], capture_output=True, text=True).stdout.splitlines() if l.startswith('Pages:')][0].split()[1])
        subprocess.run(['pdftoppm', '-png', '-r', '130', '-f', str(n), '-l', str(n), '-singlefile', pdf, t + '/p'], check=True)
        im = Image.open(t + '/p.png').convert('RGB')
    g = im.convert('L').point(lambda v: 255 if v > 245 else 0)
    inv = Image.eval(g, lambda v: 255 - v)
    box = inv.getbbox()
    if box:
        pad = 14
        im = im.crop((max(0, box[0] - pad), max(0, box[1] - pad), min(im.width, box[2] + pad), min(im.height, box[3] + pad)))
    w = 720
    im = im.resize((w, round(im.height * w / im.width)), Image.LANCZOS)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    im.save(dst, quality=88)


def make(code):
    L = load(code)
    T = L['tpl']
    outdir = OUT + L['prefix'] + L['slug']['tpl']
    files = [it[1] for it in T['items']]
    templates.build(code, P[code], outdir, files)
    for f in files:
        preview(outdir + f + '.pdf', OUT + L['tpl_pv'].format(f=f))
    xl = outdir + T['xl_file'] + '.xlsx'
    xlsx_tpl.build(X[code], xl)
    xlsx_preview(xl, OUT + L['tpl_pv'].format(f=T['xl_file']))
    print('templates', code)


if __name__ == '__main__':
    codes = sys.argv[1:] or [f[:-3] for f in sorted(os.listdir(os.path.join(HERE, 'lang'))) if f.endswith('.py')]
    for c in codes:
        make(c)
