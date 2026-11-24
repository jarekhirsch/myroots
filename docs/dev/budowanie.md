# Budowanie, testy i wydania

## Wymagania

Node.js 20+, Rust stable z celem `wasm32-unknown-unknown`, `wasm-bindgen-cli` w wersji z `crates/myroots-wasm/Cargo.toml` (`cargo install wasm-bindgen-cli --version 0.2.130`). Do aplikacji natywnej wymagania Tauri 2 (Linux: `libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf`).

## Komendy

| Komenda | Co robi |
|---|---|
| `npm run dev` | Wersja w przeglądarce, http://localhost:1420. |
| `npm run tauri dev` | Aplikacja na komputer z podglądem na żywo. |
| `npm run wasm` | Buduje silnik do `src/lib/wasm/` (potrzebne po zmianach w `crates/`). |
| `npm run build` | WASM + szablon strony rodzinnej (`dist-viewer/`, kopiowany do `public/viewer/`) + aplikacja webowa (`dist/`). |
| `npm run tauri build` | Instalator dla bieżącego systemu (`target/release/bundle/`). |
| `npx tauri build --bundles deb,rpm,appimage` | Linux: .deb, .rpm, .AppImage. |
| `npm run check` | Sprawdzenie typów (svelte-check). |
| `cargo test -p myroots-core -p myroots-store` | Testy silnika i zapisu. |
| `npm run build:app && npm run test:e2e` | Testy całej aplikacji w przeglądarce (`tests/e2e/`, Playwright): każdy ekran na komputerze i telefonie, budowanie rodziny, import/eksport, kopie, 12 języków, komunikaty. Własny Chrome: `PW_CHROME=/ścieżka/chrome`. Z MyRoots Server: `MYROOTS_SERVER_DIR=/ścieżka/myroots-server` (potrzebne PHP 8). Raport: `target/e2e-report/`. |
| `cargo run -p myroots-core --example check_archive -- plik.myroots` | Czyta kopię tak jak aplikacja na komputer (sprawdzenie zgodności). |
| `npm run i18n` / `node scripts/check-i18n.mjs pl` | Tłumaczenia. |
| `npm run docs` / `npm run docs:check` | Części generowane `docs/dev/`. |

## Wersja

Numer wersji w trzech miejscach: `package.json`, `Cargo.toml` (`[workspace.package]`), `src-tauri/tauri.conf.json`. Do tego wpis `## X.Y.Z (data)` w `CHANGELOG.md` i `CHANGELOG.en.md`.

## Sprawdzenia

- `tests.yml` przy każdej zmianie: testy silnika, tłumaczenia, dokumentacja, typy, budowanie, audyt, testy w przeglądarce (komputer i telefon) oraz budowanie na Windowsie i macOS (`platforms`: skrypty, aplikacja, część natywna, wersja ze Sklepu).
- `release.yml` uruchamia najpierw całe `tests.yml`; przy błędzie nic się nie buduje. Pakiet MSIX jest rozpakowywany i sprawdzany (wersja, plik aplikacji).
- `scripts/check-live.mjs`: myroots.cc (strony, styl, zrzuty, pliki do pobrania, `updates/latest.json` i pliki aktualizacji) i demo.myroots.cc (strona, pliki aplikacji, dane, logowanie). Uruchamiany po publikacji (`strona.yml`) i codziennie (`monitor.yml`); ręcznie: `node scripts/check-live.mjs [wersja]`.

Wydawanie wersji (sklepy, podpisy, strona) jest opisane poza repozytorium.
