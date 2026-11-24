// MyRoots Server and the WordPress plugin as ready ZIPs, with a check of each.
//
//   node scripts/packages.mjs              build the app twice, assemble, zip, check
//   node scripts/packages.mjs --check A.zip B.zip   check ready ZIPs only
//
// Result in target/packages/: myroots-server.zip (folder myroots-server/) and
// myroots.zip (folder myroots/, the plugin for wp-content/plugins). The source of
// each is in server/ and wordpress/myroots/; the app (app/ inside) is the current
// build. In the ZIPs folders are 755 and files 644, so a hosting can read them
// after unpacking and uploading over FTP, whatever the permissions on this computer.
import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import JSZip from 'jszip';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const OUT = path.join(ROOT, 'target', 'packages');
const VERSION = JSON.parse(fs.readFileSync(path.join(ROOT, 'package.json'), 'utf8')).version;
// not in app/: the website template (published separately) and the app's own offline files
const SKIP_APP = new Set(['viewer', 'sw.js', 'manifest.webmanifest']);
// never in a package: data of a running installation, local settings, system files
const SKIP = (rel) => /(^|\/)(\.DS_Store|data|media|config\.php)$/.test(rel) && !rel.startsWith('app/');

function build(noCsp) {
  const env = { ...process.env };
  delete env.VITE_MYROOTS_STORE;
  if (noCsp) env.MYROOTS_NO_CSP = '1';
  else delete env.MYROOTS_NO_CSP;
  execSync('npx vite build', { cwd: ROOT, env, stdio: 'inherit' });
}

function copyTree(src, dest, skip) {
  fs.mkdirSync(dest, { recursive: true });
  for (const e of fs.readdirSync(src, { withFileTypes: true })) {
    if (skip(e.name)) continue;
    const s = path.join(src, e.name);
    const d = path.join(dest, e.name);
    if (e.isDirectory()) copyTree(s, d, () => false);
    else fs.copyFileSync(s, d);
  }
}

/** Source folder + the current dist/ as app/ → target/packages/<name>/ */
function assemble(src, name, { shards }) {
  const dest = path.join(OUT, name);
  fs.rmSync(dest, { recursive: true, force: true });
  copyTree(src, dest, (n) => n === '.DS_Store' || n === 'app' || n === 'data' || n === 'media' || n === 'config.php');
  copyTree(path.join(ROOT, 'dist'), path.join(dest, 'app'), (n) => SKIP_APP.has(n));
  if (shards) {
    // WordPress.org does not accept .bin files: the face models get another extension
    const face = path.join(dest, 'app', 'models', 'face');
    for (const f of fs.readdirSync(face)) {
      if (f.endsWith('.bin')) fs.renameSync(path.join(face, f), path.join(face, f.slice(0, -4) + '.shard'));
    }
    for (const f of fs.readdirSync(face)) {
      if (f.endsWith('weights_manifest.json')) {
        const p = path.join(face, f);
        fs.writeFileSync(p, fs.readFileSync(p, 'utf8').replaceAll('.bin"', '.shard"'));
      }
    }
  }
  return dest;
}

async function zipFolder(dir, file) {
  const zip = new JSZip();
  const top = path.basename(dir);
  const walk = (d, rel) => {
    zip.file(rel + '/', null, { dir: true, unixPermissions: 0o40755 });
    for (const e of fs.readdirSync(d, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const r = rel + '/' + e.name;
      if (SKIP(r.slice(top.length + 1))) continue;
      if (e.isDirectory()) walk(path.join(d, e.name), r);
      else zip.file(r, fs.readFileSync(path.join(d, e.name)), { unixPermissions: 0o100644 });
    }
  };
  walk(dir, top);
  const buf = await zip.generateAsync({ type: 'nodebuffer', platform: 'UNIX', compression: 'DEFLATE', compressionOptions: { level: 9 } });
  fs.writeFileSync(file, buf);
}

/** Problems of one ZIP (empty list = good). */
async function check(file) {
  const bad = [];
  const zip = await JSZip.loadAsync(fs.readFileSync(file));
  const names = Object.keys(zip.files);
  const top = names[0].split('/')[0];
  const server = top === 'myroots-server';
  const app = `${top}/app/`;
  const text = (n) => (zip.files[n] ? zip.files[n].async('string') : Promise.resolve(''));
  for (const n of names) {
    const mode = zip.files[n].unixPermissions;
    const want = n.endsWith('/') ? 0o40755 : 0o100644;
    if (mode !== want) {
      bad.push(`${n}: permissions ${mode?.toString(8)} instead of ${want.toString(8)}`);
      break;
    }
  }
  if (names.some((n) => /\/data\//.test(n) || n.endsWith('/config.php'))) bad.push('data/ or config.php in the package');
  if (names.some((n) => n.endsWith('.DS_Store'))) bad.push('.DS_Store files in the package');
  if (server && !names.includes(`${top}/.htaccess`)) bad.push('no .htaccess');
  if (!names.includes(app + 'index.html')) bad.push('no app/index.html');
  else {
    const html = await text(app + 'index.html');
    for (const [, ref] of html.matchAll(/(?:src|href)="\.\/(assets\/[^"]+)"/g)) {
      if (!names.includes(app + ref)) bad.push(`app/index.html points to a missing file ${ref}`);
    }
    if (!server && html.includes('Content-Security-Policy')) bad.push('the plugin has a built-in CSP (built without MYROOTS_NO_CSP)');
  }
  const js = names.filter((n) => n.startsWith(app + 'assets/') && n.endsWith('.js'));
  if (js.length < 10) bad.push(`too few app files (${js.length})`);
  if (names.some((n) => n.startsWith(app + 'viewer/') || n === app + 'sw.js')) bad.push('files that do not belong in app/ (viewer/, sw.js)');
  let hasVersion = false;
  for (const n of js) if ((await text(n)).includes(VERSION)) hasVersion = true;
  if (!hasVersion) bad.push(`the app in the package is not version ${VERSION}`);
  if (server) {
    if (!(await text(`${top}/lib/site.php`)).includes('function mirror')) bad.push('lib/site.php without the photo mirror');
  } else {
    if (names.some((n) => n.includes('/models/face/') && n.endsWith('.bin'))) bad.push('.bin files in the plugin (WordPress.org refuses them)');
    const php = await text(`${top}/my-roots.php`);
    const readme = await text(`${top}/readme.txt`);
    if (!php.includes(` * Version:           ${VERSION}\n`)) bad.push(`my-roots.php: header Version is not ${VERSION}`);
    if (!php.includes(`define( 'MYROOTS_VERSION', '${VERSION}' );`)) bad.push(`my-roots.php: MYROOTS_VERSION is not ${VERSION}`);
    if (!readme.includes(`Stable tag: ${VERSION}\n`)) bad.push(`readme.txt: Stable tag is not ${VERSION}`);
  }
  return bad;
}

async function report(files) {
  let problems = 0;
  for (const f of files) {
    const b = await check(f);
    console.log((b.length ? 'ERROR ' : 'OK    ') + path.relative(process.cwd(), f));
    for (const x of b) console.log('        - ' + x);
    problems += b.length;
  }
  return problems;
}

const args = process.argv.slice(2);
if (args[0] === '--check') {
  process.exit((await report(args.slice(1))) ? 1 : 0);
}

fs.mkdirSync(OUT, { recursive: true });
// the plugin first (without the built-in CSP: WordPress sends its own headers),
// then the server, so dist/ ends as the normal build
build(true);
const wp = assemble(path.join(ROOT, 'wordpress', 'myroots'), 'myroots', { shards: true });
build(false);
const srv = assemble(path.join(ROOT, 'server'), 'myroots-server', { shards: false });
const zips = [path.join(OUT, 'myroots-server.zip'), path.join(OUT, 'myroots.zip')];
await zipFolder(srv, zips[0]);
await zipFolder(wp, zips[1]);
if (await report(zips)) {
  console.log('\nThe packages have errors: do not upload them.');
  process.exit(1);
}
console.log('\nReady: target/packages/myroots-server.zip and myroots.zip');
