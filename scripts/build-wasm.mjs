// Builds the WebAssembly engine used by the browser app and the family website.
import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const out = path.join(root, 'src/lib/wasm');
const run = (cmd) => execSync(cmd, { cwd: root, stdio: 'inherit' });

run('cargo build -p myroots-wasm --profile wasm --target wasm32-unknown-unknown');
run(`wasm-bindgen --target web --omit-default-module-path --out-dir "${out}" target/wasm32-unknown-unknown/wasm/myroots_wasm.wasm`);
// Optional size optimization when binaryen is installed.
try {
  execSync('wasm-opt --version', { stdio: 'ignore' });
  run(`wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int -o "${out}/myroots_wasm_bg.wasm" "${out}/myroots_wasm_bg.wasm"`);
} catch {
  /* binaryen not installed — fine */
}
console.log('wasm:', (fs.statSync(path.join(out, 'myroots_wasm_bg.wasm')).size / 1024).toFixed(0), 'KiB');
