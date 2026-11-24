# Architektura

## Jeden silnik, jeden interfejs, pięć platform

```
            src/ (Svelte 5, TypeScript)  ── ten sam interfejs wszędzie
                         │  call(method, params) → JSON
         ┌───────────────┼────────────────┬─────────────────┬──────────────────┐
   TauriBackend      WasmBackend      ViewerBackend       WpBackend          WpBackend
  (Mac/Win/Linux,   (przeglądarka,   (strona rodzinna,   (wtyczka WP,       (MyRoots Server,
   Android, iOS)      PWA)             tylko odczyt)       wp-admin)          PHP + SQLite)
         │               │                │                  │                  │
  Rust natywnie     Rust → WASM       Rust → WASM        Rust → WASM        Rust → WASM
  + SQLite           + IndexedDB       + data.js          + REST WP          + REST serwera
  (myroots-store)                      (eksport ZIP)       (tabela rekordów)  (data/myroots.sqlite)
```

- **Silnik** (`crates/myroots-core`): cały model danych i logika (relacje, prywatność, pokrewieństwo, statystyki, GEDCOM, kopia `.myroots`, eksport strony). Jedno wejście: `Engine::call(method, params, ctx)` w `engine.rs`, które zwraca JSON. Szczegóły: [silnik.md](silnik.md).
- **Interfejs** (`src/`): Svelte 5 z runami. Woła silnik przez `call()` z `src/lib/api.svelte.ts`, nigdy bezpośrednio. Szczegóły: [interfejs.md](interfejs.md).
- **Backend** (`src/lib/backend/`): wybiera platformę w `index.ts` i implementuje interfejs `Backend` z `types.ts` (wywołanie silnika, zdjęcia, kopie, zapis plików, drzewa).

## Gdzie są dane

Cała baza drzewa jest w pamięci (`Database` w `db.rs`: mapy `BTreeMap<Id, …>` osób, związków, źródeł, zdjęć, miejsc…). Każda zmiana przechodzi przez `put_*` / `del_*` i trafia do `ChangeSet`. Po każdym wywołaniu platforma zabiera zmiany (`take_changes`) i zapisuje tylko je:

| Platforma | Zapis rekordów | Zdjęcia |
|---|---|---|
| Tauri (`src-tauri/src/lib.rs`, `crates/myroots-store`) | SQLite: wiersze JSON `(kind, id)` w folderze danych aplikacji, osobny folder na każde drzewo | pliki `media/<id>` i `media/<id>_t` (miniatura), protokół `media://` |
| Przeglądarka (`backend/wasm.ts`, `idb.ts`) | IndexedDB, baza na drzewo, kolejka zapisów | IndexedDB `blobs`, adresy `blob:` (`ensureMedia`) |
| Strona rodzinna (`backend/viewer.ts`) | tylko odczyt z `data.js` (publiczna kopia, opcjonalnie szyfrowana: `sitelock.ts`) | pliki w ZIP strony |
| WordPress / Server (`backend/wp.ts`) | REST API wtyczki lub serwera, paczki zmian z ponawianiem | REST, pliki na serwerze |

Uruchomienie: platforma wczytuje wszystkie rekordy i buduje `Database::from_records`. Przy 20 000 osób to ok. 0,3 s natywnie.

## Kilka drzew

Każde drzewo to osobna baza (folder w Tauri, baza IndexedDB w przeglądarce, osobne tabele / foldery w WP i na serwerze). Lista drzew: `TreesApi` w `backend/types.ts`. Zmiana drzewa przeładowuje stronę.

## Strona myroots.cc

Źródła w `strona/`: generator `strona/zrodla/build.py` (12 języków, teksty w `lang/*.py` i `lang_news.py`) i pliki stałe w `strona/static/`. Strona „Co nowego” powstaje z `CHANGELOG.md` i `CHANGELOG.en.md`.
