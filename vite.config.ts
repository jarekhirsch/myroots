import { defineConfig, type Plugin } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Two builds from one source:
//  - "app"    (default): the editor — Tauri desktop/mobile and the browser PWA (dist/)
//  - "viewer": the read-only family website template, a single self-contained
//              script (works on any hosting and from a local disk) (dist-viewer/)
// Content-Security-Policy of the built pages (defence in depth: no inline
// scripts, no plugins, no foreign frames). The desktop app sets its own in
// src-tauri/tauri.conf.json (the app page built for Tauri gets no meta policy).
// MYROOTS_NO_CSP=1 leaves it out (preview hosts that add their own scripts).
const CSP_APP = [
  "default-src 'self'",
  "script-src 'self' 'wasm-unsafe-eval'",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob: https:",
  "media-src 'self' blob:",
  "font-src 'self' data:",
  "connect-src 'self' blob: data: https:",
  "worker-src 'self' blob:",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'none'",
].join('; ');
const CSP_VIEWER = [
  "default-src 'self' file:",
  "script-src 'self' file: 'wasm-unsafe-eval'",
  "style-src 'self' file: 'unsafe-inline'",
  "img-src 'self' file: data: blob: https:",
  "media-src 'self' file: blob:",
  "font-src 'self' file: data:",
  "connect-src 'self' file: data: blob: https:",
  "worker-src 'self' blob: data:",
  "object-src 'none'",
  "base-uri 'self'",
  "form-action 'none'",
].join('; ');

function csp(viewer: boolean): Plugin {
  return {
    name: 'myroots-csp',
    apply: 'build',
    transformIndexHtml(html) {
      // The desktop/mobile app gets its policy from Tauri (which also allows its
      // ipc: and media: protocols); a second policy here would block them.
      if (process.env.MYROOTS_NO_CSP || (!viewer && process.env.TAURI_ENV_PLATFORM)) return html;
      return html.replace('<meta charset="UTF-8" />', `<meta charset="UTF-8" />\n    <meta http-equiv="Content-Security-Policy" content="${viewer ? CSP_VIEWER : CSP_APP}" />`);
    },
  };
}

export default defineConfig(({ mode }) => {
  const viewer = mode === 'viewer';
  return {
    plugins: [svelte(), csp(viewer)],
    // The viewer must not copy public/ (it contains the previous viewer build).
    publicDir: viewer ? false : 'public',
    clearScreen: false,
    base: './',
    define: {
      __VIEWER__: JSON.stringify(viewer),
      __APP_VERSION__: JSON.stringify(process.env.npm_package_version || '3.1.0'),
    },
    server: { port: 1420, strictPort: true, watch: { ignored: ['**/src-tauri/**', '**/target/**'] } },
    build: viewer
      ? {
          outDir: 'dist-viewer',
          emptyOutDir: true,
          assetsInlineLimit: 100_000_000,
          cssCodeSplit: false,
          modulePreload: false,
          rolldownOptions: {
            input: 'viewer.html',
            output: { format: 'iife', inlineDynamicImports: true, entryFileNames: 'assets/viewer.js', assetFileNames: 'assets/viewer[extname]' },
          },
          target: 'es2020',
        }
      : {
          outDir: 'dist',
          emptyOutDir: true,
          target: 'es2021',
          chunkSizeWarningLimit: 3000,
        },
  };
});
