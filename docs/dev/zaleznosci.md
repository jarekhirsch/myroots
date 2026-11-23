# Zależności

Zasada: najnowsze stabilne wersje. Sprawdzenie: `npm outdated`, `npm audit`, `cargo update --dry-run --verbose`. Po aktualizacji: testy, `npm run check`, `npm run build`, aplikacja Tauri.

## Po co są

| Biblioteka | Do czego |
|---|---|
| `svelte`, `@sveltejs/vite-plugin-svelte`, `vite` | Interfejs i budowanie. Vite 8 używa Rolldown; konfiguracja w `vite.config.ts` (`build.rolldownOptions` dla szablonu strony rodzinnej, format IIFE). |
| `typescript`, `svelte-check`, `@tsconfig/svelte` | Typy. svelte-check obsługuje TypeScript 5 i 6 (nie 7), dlatego `typescript ~6.0`. |
| `@tauri-apps/*` (api, cli, plugin-dialog, fs, http, notification, opener, updater, process) | Aplikacja natywna. Wersje pluginów JS i Rust muszą się zgadzać w wersji minor (`Cargo.lock` i `package-lock.json`). |
| `family-chart` (z `d3`) | Drzewo „Wokół osoby”. `d3` zostaje, bo family-chart go wymaga; własny kod używa `d3-selection`, `d3-zoom`, `d3-transition`, `d3-hierarchy`. |
| `leaflet`, `leaflet.markercluster` | Mapy (główna, groby, mini mapa). Leaflet 2 jest jeszcze w becie, zostajemy przy 1.9. |
| `maplibre-gl`, `@maplibre/maplibre-gl-leaflet` | Wektorowy podkład map z OpenFreeMap rysowany wewnątrz Leafleta (`src/lib/basemap.ts`, `src/lib/maplibre.ts`). Ładowany dopiero przy otwarciu mapy. MapLibre 5 ma krytyczną lukę XSS (do 6.4.0), więc wersja 6. |
| `@vladmandic/face-api` | Twarze na zdjęciach (na urządzeniu), modele w `public/models/face/`. |
| `dompurify` | Czyszczenie HTML w podglądzie. |
| `fflate` | ZIP w przeglądarce (strona rodzinna, kopie). |
| `@lucide/svelte` | Ikony. |
| `@fontsource-variable/onest`, `literata` | Fonty. |
| `binaryen` | `wasm-opt` przy budowaniu silnika. |
| Rust: `serde`, `serde_json`, `chrono`, `zip`, `base64` | Silnik. |
| Rust: `rusqlite` (`bundled`) | Zapis natywny. |
| Rust: `wasm-bindgen` (dokładna wersja), `js-sys` | Most WASM. Wersja `wasm-bindgen` musi być równa wersji `wasm-bindgen-cli`. |
| Rust: `tauri` 2 i pluginy | Aplikacja natywna. Tauri 3 jest w wersji alpha. |

## Wersje (generowane)

<!-- auto:deps -->
### npm (package.json)

| Pakiet | Wersja | Rodzaj |
|---|---|---|
| `@fontsource-variable/literata` | ^5.3.0 | aplikacja |
| `@fontsource-variable/onest` | ^5.3.1 | aplikacja |
| `@lucide/svelte` | ^1.77.0 | aplikacja |
| `@tauri-apps/api` | ^2.12.5 | aplikacja |
| `@tauri-apps/plugin-dialog` | ^2.8.4 | aplikacja |
| `@tauri-apps/plugin-fs` | ^2.7.1 | aplikacja |
| `@tauri-apps/plugin-http` | ^2.9.0 | aplikacja |
| `@tauri-apps/plugin-notification` | ^2.8.1 | aplikacja |
| `@tauri-apps/plugin-opener` | ^2.7.3 | aplikacja |
| `@tauri-apps/plugin-process` | ^2.4.0 | aplikacja |
| `@tauri-apps/plugin-updater` | ^2.14.1 | aplikacja |
| `@vladmandic/face-api` | ^1.7.15 | aplikacja |
| `d3` | ^7.9.2 | aplikacja |
| `dompurify` | ^3.5.0 | aplikacja |
| `family-chart` | ^0.9.0 | aplikacja |
| `fflate` | ^0.8.3 | aplikacja |
| `leaflet` | ^1.9.4 | aplikacja |
| `leaflet.markercluster` | ^1.5.3 | aplikacja |
| `maplibre-gl` | ^6.19.0 | aplikacja |
| `@maplibre/maplibre-gl-leaflet` | ^0.1.4 | aplikacja |
| `@sveltejs/vite-plugin-svelte` | ^7.3.1 | budowanie |
| `@tauri-apps/cli` | ^2.12.6 | budowanie |
| `@tsconfig/svelte` | ^5.0.9 | budowanie |
| `@types/d3` | ^7.4.3 | budowanie |
| `@types/leaflet` | ^1.9.22 | budowanie |
| `@types/leaflet.markercluster` | ^1.5.6 | budowanie |
| `binaryen` | ^132.0.0 | budowanie |
| `svelte` | ^5.59.1 | budowanie |
| `svelte-check` | ^4.7.8 | budowanie |
| `typescript` | ~6.0.3 | budowanie |
| `vite` | ^8.3.8 | budowanie |

### Rust (Cargo.toml)

| Crate | Zależność | Wersja |
|---|---|---|
| myroots-core | `chrono` | 0.4 |
| myroots-core | `zip` | 9 |
| myroots-core | `base64` | 0.23 |
| myroots-store | `myroots-core` | lokalny |
| myroots-store | `rusqlite` | 0.43 |
| myroots-wasm | `myroots-core` | lokalny |
| myroots-wasm | `wasm-bindgen` | =0.2.130 |
| myroots-wasm | `js-sys` | 0.3 |
| myroots-wasm | `chrono` | 0.4 |
| src-tauri | `tauri-build` | 2 |
| src-tauri | `myroots-core` | lokalny |
| src-tauri | `myroots-store` | lokalny |
| src-tauri | `chrono` | 0.4 |
| src-tauri | `tauri` | 2 |
| src-tauri | `tauri-plugin-dialog` | 2 |
| src-tauri | `tauri-plugin-fs` | 2 |
| src-tauri | `tauri-plugin-http` | 2 |
| src-tauri | `tauri-plugin-notification` | 2 |
| src-tauri | `tauri-plugin-opener` | 2 |
| src-tauri | `tauri-plugin-updater` | 2 |
| src-tauri | `tauri-plugin-process` | 2 |
<!-- /auto:deps -->
