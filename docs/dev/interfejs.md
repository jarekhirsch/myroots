# Interfejs (`src/`)

Svelte 5 (runy), TypeScript, Vite 8. Bez frameworka routingu: własny router w `src/lib/router.svelte.ts` (adresy `#/...`), widoki wybierane w `src/App.svelte`.

## Struktura

| Ścieżka | Co tam jest |
|---|---|
| `src/main.ts`, `Boot.svelte`, `App.svelte` | Start: backend, język, motyw, układ (menu, pasek, dok na telefonie), lista widoków, okna (potwierdzenie, lightbox), pasek aktualizacji. |
| `src/views/` | Strony: `People`, `PersonView`, `PersonEdit`, `TreeView`, `MapView`, `Graves`, `Events`, `Health`, `Duplicates`, `SettingsView`, `DataView` (import i kopia), `Website` (strona rodzinna) i inne. |
| `src/lib/components/` | Części wielokrotnego użytku: `PersonPicker`, `CitationButton`, `GraveEditor`, `FamilyGraph` (cała rodzina), `FamilyTimeline` (oś pokoleń), `Lightbox`, `RichText`, `UpdateBanner`… |
| `src/lib/api.svelte.ts` | `call()`, `app` (wersja danych, info, ustawienia), `summaries()` z pamięcią podręczną, lista metod tylko do odczytu `READ`. |
| `src/lib/load.svelte.ts` | `useLoad(fn)`: ładuje dane i powtarza po każdej zmianie (`app.version`). |
| `src/lib/text.ts` | `norm()`, `haystack()`, `search()`, `words()`: wyszukiwanie po znormalizowanym tekście. |
| `src/lib/citations.ts` | Źródła przy faktach: `FACTS`, `personCites`, `withPersonCites`, `QUALITY`, `perFact()`. |
| `src/lib/updates.svelte.ts` | Aktualizacje programu na komputer (Tauri updater). |
| `src/lib/ui.svelte.ts` | `toast`, okna potwierdzenia, `progress` i `withProgress(etykieta, (report) => …)`: wspólny pasek postępu (`ProgressBar.svelte`) dla każdego wgrywania i importu. Nowe wgrywanie plików: zawsze przez `withProgress`. |
| `src/lib/gedmedia.ts` | Zdjęcia z importu GEDCOM: `imagesFromZip` (fflate), `imagesFromFiles` (folder), `attachGedcomMedia` (dopasowanie ścieżek, `origin: gedcom:<ścieżka>` chroni przed duplikatami, potem `media.link_imported`). |
| `src/lib/assoc.ts` | Role powiązań (chrzestny, świadek…): `ROLES`, `roleLabel(role, reverse)`. |
| `src/lib/cemeteries.ts` | Cmentarze z OpenStreetMap w widocznym fragmencie mapy (Overpass API z zapasowymi serwerami, pamięć podręczna na siatce 0,05°). |
| `src/lib/migrations.ts` | Migracje na mapie: `buildMigrations(węzły, miejsca, osoba, miejsce)`; `homePlaces()` (miejsca urodzeń do trybu „kto z tego miejsca poszedł dalej”). |
| `src/lib/types.ts` | Typy danych silnika (trzymaj w zgodzie z `model.rs` i `query.rs`). |
| `src/lib/i18n/` | Tłumaczenia (niżej). |

## Stan i ładowanie danych

- Po każdym wywołaniu metody, która nie jest w `READ`, `app.version` rośnie, a wszystkie `useLoad` w otwartych widokach ładują dane od nowa.
- `useLoad` trzyma dane jako `$state.raw`: bez głębokiego proxy. Dane z silnika tylko się czyta; zmiana to nowe wywołanie silnika. **Nie modyfikuj `x.data` w miejscu.**
- Duże listy z silnika (osoby, węzły drzewa, dane mapy) zawsze w `$state.raw`. Głębokie `$state` na 20 000 obiektów spowalniało listy kilkukrotnie.
- `summaries()` zwraca jedną, wspólną tablicę skrótów osób dla danej wersji danych (równoległe wywołania czekają na to samo zapytanie).
- Formularze (np. `person` w `PersonEdit`) są w głębokim `$state`, bo używają `bind:value`.
- Adresy z parametrami: `#/people?clan=X`, `#/tree?clan=X` (wyróżniony ród), `#/map?clan=X`, `#/map?mig=1`, `#/map?from=Miejsce` (migracje z jednego miejsca).

## Drzewo „Wokół osoby” (`TreeView.svelte`)

Biblioteka `family-chart` rysuje karty (`cardHtml`); po każdym `updateTree` wywołuj `update()` (nie `chart.updateTree` wprost), bo `decorate()` dokłada klasy: `mr-side` (linia boczna, mniejsza), `mr-br` z `--b1`/`--b2` (kolor linii dziadków), `mr-dim`/`mr-lit` (wybrany ród), a na liniach `mr-ldirect`, `mr-lside`, `mr-lbr`. `lines(id)` liczy linię główną (przodkowie, potomkowie, partnerzy) i numer linii dziadka (0–3) każdego przodka. Wybór „Linie” jest zapamiętany w `localStorage` (`myroots-tree-hl`).

## Wyszukiwanie

`search(list, klucz, tekst, zapytanie, limit, filtr)` z `text.ts` normalizuje tekst każdej pozycji raz na listę (pamięć w `WeakMap` po tablicy) i kończy po `limit` wynikach. Używają go lista osób, `PersonPicker`, `QuickSearch`, zdarzenia, groby. Nowa wyszukiwarka: używaj `search`/`haystack`, nie `norm()` w pętli przy każdym klawiszu.

## Tłumaczenia

- Teksty w kodzie po angielsku: `t('…')`, `tn('%d osoba', '%d osoby', n)`, `tx('kontekst', '…')`.
- Stare tłumaczenia z wtyczki: `src/lib/i18n/po/*.po`. Nowe: `src/lib/i18n/app-strings/*.tsv`, wiersz `"tekst angielski" ‖ pl ‖ de ‖ fr ‖ es ‖ it ‖ pt ‖ nl ‖ sv ‖ fi ‖ da ‖ he` (formy mnogie przez ` ¦ `, polski ma 3).
- `npm run i18n` buduje `locales/*.json`; `node scripts/check-i18n.mjs pl` pokazuje brakujące.
- Hebrajski czyta się od prawej: w CSS używaj właściwości logicznych (`inset-inline-end`, `margin-inline-start`).

## Wygląd

Zmienne CSS w `src/app.css` (`--ink`, `--surface`, `--line`, `--radius`, kolory zdarzeń `--ev-*` z `--ev-burial` dla grobów), motyw jasny i ciemny. Kolory drzewa: `--tree-bg`, `--tree-line`, `--tree-line-main`, `--tree-br1`…`4` (z ustawień Wyglądu, `theme.ts`); widoki drzewa używają ich z domyślnymi wartościami. Ikony: `@lucide/svelte` (import pojedynczych ikon: `@lucide/svelte/icons/<nazwa>`).
