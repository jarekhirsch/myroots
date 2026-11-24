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
| `npm run build:app && npm run test:e2e` | Testy całej aplikacji w przeglądarce (`tests/e2e/`, Playwright): każdy ekran na komputerze i telefonie, budowanie rodziny, import/eksport, kopie, 12 języków, komunikaty. Własny Chrome: `PW_CHROME=/ścieżka/chrome`. Z MyRoots Server: `MYROOTS_SERVER_DIR=$PWD/server` (potrzebne PHP 8 z `pdo_sqlite`). Z WordPressem: `MYROOTS_WP_URL=http://adres/` (WordPress z włączoną wtyczką, patrz niżej). Raport: `target/e2e-report/`. |
| `node scripts/packages.mjs` | MyRoots Server (`server/`) i wtyczka WordPress (`wordpress/myroots/`) z aktualną aplikacją w `app/`: `target/packages/myroots-server.zip` i `myroots.zip` (foldery 755, pliki 644) i ich kontrola (uprawnienia, brak danych, kompletna aplikacja, wersja, wtyczka bez CSP i plików `.bin`). Sama kontrola gotowych: `--check plik.zip`. |
| `cargo run -p myroots-core --example check_archive -- plik.myroots` | Czyta kopię tak jak aplikacja na komputer (sprawdzenie zgodności). |
| `npm run i18n` / `node scripts/check-i18n.mjs pl` | Tłumaczenia. |
| `npm run docs` / `npm run docs:check` | Części generowane `docs/dev/`. |

## Wersja

Numer wersji w trzech miejscach: `package.json`, `Cargo.toml` (`[workspace.package]`), `src-tauri/tauri.conf.json`. Wtyczka WordPress: `Version` i `MYROOTS_VERSION` w `wordpress/myroots/my-roots.php` oraz `Stable tag` w `readme.txt` (pilnuje tego `scripts/packages.mjs`). Do tego wpis `## X.Y.Z (data)` w `CHANGELOG.md` i `CHANGELOG.en.md`.

## Sprawdzenia

- `tests.yml` przy każdej zmianie: testy silnika, tłumaczenia, dokumentacja, typy, budowanie, audyt, testy w przeglądarce (komputer i telefon) oraz budowanie na Windowsie i macOS (`platforms`: skrypty, aplikacja, część natywna, wersja ze Sklepu).
- MyRoots Server (`e2e`): instalacja, logowanie, przywrócenie kopii ze zdjęciami, zapis, publikacja, strona dla gościa i zdjęcia jako zwykłe pliki, na najstarszym obsługiwanym PHP (8.0).
- Wtyczka WordPress (`wordpress`): prawdziwy WordPress w Dockerze (Apache + MariaDB), najstarsza obsługiwana wersja (6.5) i najnowsza. Paczki z `scripts/packages.mjs`, składnia PHP na minimalnych wersjach (wtyczka 7.4, serwer 8.0), włączenie wtyczki z `WP_DEBUG`, potem `tests/e2e/wordpress.spec.ts`: edytor w panelu, kopia ze zdjęciami, zapis, publikacja, strona dla gościa, shortcode `[myroots]` na zwykłej stronie, REST i edytor zamknięte dla gościa i subskrybenta. Każdy komunikat PHP z wtyczki w `debug.log` to błąd.
- `release.yml` uruchamia najpierw całe `tests.yml`; przy błędzie nic się nie buduje. Pakiet MSIX jest rozpakowywany i sprawdzany (wersja, plik aplikacji). Paczki serwera i wtyczki (`myroots-server.zip`, `myroots.zip`) są budowane, sprawdzane i dołączane do wydania.
- `scripts/check-live.mjs`: myroots.cc (strony, styl, zrzuty, pliki do pobrania, `updates/latest.json` i pliki aktualizacji) i demo.myroots.cc (strona, pliki aplikacji, dane, logowanie). Uruchamiany po publikacji (`strona.yml`) i codziennie (`monitor.yml`); ręcznie: `node scripts/check-live.mjs [wersja]`.

Wydawanie wersji (sklepy, podpisy, strona) jest opisane poza repozytorium.
