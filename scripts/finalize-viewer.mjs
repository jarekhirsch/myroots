// Makes the viewer template work from a local disk (file://): classic deferred
// script instead of a module script, file name index.html.
import fs from 'node:fs';
import path from 'node:path';
const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const dir = path.join(root, 'dist-viewer');
let html = fs.readFileSync(path.join(dir, 'viewer.html'), 'utf8');
html = html.replace(/<script type="module" crossorigin src="([^"]+)"><\/script>/, '<script defer src="$1"></script>');
html = html.replace(/<link rel="stylesheet" crossorigin href=/, '<link rel="stylesheet" href=');
fs.writeFileSync(path.join(dir, 'index.html'), html);
fs.rmSync(path.join(dir, 'viewer.html'));
fs.copyFileSync(path.join(root, 'public/icon.svg'), path.join(dir, 'icon.svg'));
// manifest of files, used by the exporters
const files = [];
const walk = (d, rel = '') => {
  for (const f of fs.readdirSync(d)) {
    const p = path.join(d, f);
    const r = rel ? `${rel}/${f}` : f;
    if (fs.statSync(p).isDirectory()) walk(p, r);
    else if (f !== 'files.json') files.push(r);
  }
};
walk(dir);
fs.writeFileSync(path.join(dir, 'files.json'), JSON.stringify(files));
// The browser app serves the template under /viewer/ (site export from the PWA).
const pub = path.join(root, 'public/viewer');
fs.rmSync(pub, { recursive: true, force: true });
fs.cpSync(dir, pub, { recursive: true });
console.log('viewer files:', files.join(', '));
