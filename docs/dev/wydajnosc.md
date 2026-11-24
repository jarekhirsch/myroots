# Wydajność dużych drzew

Cel: płynna praca przy kilkudziesięciu tysiącach osób.

## Pomiary (drzewo testowe 20 000 osób, 3 782 rodziny)

Silnik natywnie (`cargo run --release -p myroots-core --example bench -- duze.ged`), przed i po zmianach w 3.1:

| Metoda | 3.0 | 3.1 |
|---|---|---|
| `merge.suggestions` (duplikaty) | 5 100 ms | 170 ms |
| `person.profile`, `person.edit` (kolejne) | 40–66 ms | 0,1 ms |
| `lifeline` | 45 ms | 8 ms |
| `kinship` | 38 ms | 1,4 ms |
| `people.summaries` | 190 ms | 160 ms |

Wersja w przeglądarce (WASM, Chrome, `scripts/perf/perf.py`): czas od kliknięcia do bezczynności strony.

| Widok | 3.0 | 3.1 |
|---|---|---|
| Możliwe duplikaty | 51,1 s | 0,4 s |
| Spójność danych | 3,8 s | 0,3 s |
| Groby | 2,5 s | 0,4 s |
| Drzewo | 1,7 s | 0,7 s |
| Lista osób | 1,1 s | 0,5 s |
| Wpisanie 5 liter w wyszukiwarce | 1,1 s | 0,6 s |

Drugi pomiar 3.1 (to samo drzewo, 3 000 miejsc, 2 000 zdjęć): po poprawkach z listopada.

| Co | Przed | Po |
|---|---|---|
| Strona startowa (komputer) | 3,3 s | 0,3 s (oś pokoleń rysowana po stronie) |
| Wydarzenia (komputer) | 1,6 s | 0,65 s (`sort_by_cached_key`, puste pola pomijane: 16 → 9 MB) |
| Strona startowa (telefon, CPU ×4 wolniejsze) | 8 s | treść po 0,9 s, całość 5,2 s (zwinięte pokolenia na `<canvas>`) |
| Miejsca (telefon) | 3,9 s | porcjami po 200 wierszy |
| Import GEDCOM + ZIP 540 MB ze zdjęciami | niemożliwy (base64 > limit tekstu) | 42 s, 32 MB pamięci (`zipindex.ts`) |
| Kopia zapasowa 20 000 osób + 2 000 zdjęć (550 MB) | ok. 2 GB pamięci | zapis 19 s, przywrócenie 8 s, ok. 11 MB (`archivejs.ts`) |
| MyRoots Server: edytor po odświeżeniu, 20 000 osób, `memory_limit` 128 MB | błąd PHP (brak pamięci) | 4,7 s przy 20 Mbit (rekordy strumieniowo, gzip) |
| MyRoots Server: pierwsze otwarcie strony rodzinnej | 14,3 s (`data.js` 20,6 MB) | 5,8 s (`data.js.gz` 1,3 MB) |

Długa praca: 8 przejść przez 13 widoków nie zwiększa pamięci (25–45 MB, stała liczba elementów strony i nasłuchów).

## Narzędzia

```
python3 scripts/perf/gen_big_ged.py 20000 > duze.ged       # drzewo testowe
cargo run --release -p myroots-core --example bench -- duze.ged
npm run build && (cd dist && python3 -m http.server 4173) &
python3 scripts/perf/perf.py http://localhost:4173/ duze.ged   # Playwright + Chrome
python3 scripts/perf/gen_media.py duze.ged 2000 duze-zdjecia.zip  # to samo drzewo ze zdjęciami (ZIP do importu)
```

Czasy zapytań do silnika w działającej aplikacji: w konsoli przeglądarki `localStorage.setItem('myroots-debug', '1')` i odświeżenie strony.

## Zasady

1. **Silnik:** nie buduj indeksu ani map wyszukiwania w pętli. `db.index()` jest w pamięci podręcznej; `db.place_by_name()` przeszukuje wszystkie miejsca, więc w pętli użyj `db.places_by_name()`. Sortowanie po kluczu liczonym z tekstu: `sort_by_cached_key`. Porównania „każdy z każdym” zawsze w małych grupach (jak `merge::suggestions`).
2. **Prywatność bez kopii:** w pętlach po wszystkich osobach `pol.view(p)` zamiast `pol.redact(p)` (kopiuje tylko, gdy coś jest cenzurowane).
3. **Mniej danych do interfejsu:** do pola wyboru wystarczy lekka metoda (`sources.brief`, `graves.cemeteries`, `health.count`), nie pełna lista.
4. **Interfejs:** dane z silnika w `$state.raw`; wyszukiwanie przez `search()`/`haystack()`; długie listy w porcjach („Pokaż więcej”) albo zwinięte grupy; jeden nasłuch zdarzeń na kontener zamiast kilku na element; tablice SVG rysują tylko widoczny fragment (`FamilyGraph`).
5. **Pliki:** duże pliki (ZIP z GEDCOM-em, kopia `.myroots`) nigdy w całości w pamięci: `zipindex.ts` czyta spis archiwum i wyjmuje pliki po jednym, `archivejs.ts` składa kopię z kawałków (`Blob`). Na serwerze PHP rekordy idą strumieniowo (`stream_all`), a `data.js` jako gotowy `.gz`.
6. **Drzewo „Wokół osoby”:** w drzewach powyżej 1 500 osób domyślnie 6 pokoleń przodków i 3 potomków, dopóki użytkownik sam nie wybierze.
