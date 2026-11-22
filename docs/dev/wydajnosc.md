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

## Narzędzia

```
python3 scripts/perf/gen_big_ged.py 20000 > duze.ged       # drzewo testowe
cargo run --release -p myroots-core --example bench -- duze.ged
npm run build && (cd dist && python3 -m http.server 4173) &
python3 scripts/perf/perf.py http://localhost:4173/ duze.ged   # Playwright + Chrome
```

## Zasady

1. **Silnik:** nie buduj indeksu ani map wyszukiwania w pętli. `db.index()` jest w pamięci podręcznej; `db.place_by_name()` przeszukuje wszystkie miejsca, więc w pętli użyj `db.places_by_name()`. Sortowanie po kluczu liczonym z tekstu: `sort_by_cached_key`. Porównania „każdy z każdym” zawsze w małych grupach (jak `merge::suggestions`).
2. **Prywatność bez kopii:** w pętlach po wszystkich osobach `pol.view(p)` zamiast `pol.redact(p)` (kopiuje tylko, gdy coś jest cenzurowane).
3. **Mniej danych do interfejsu:** do pola wyboru wystarczy lekka metoda (`sources.brief`, `graves.cemeteries`, `health.count`), nie pełna lista.
4. **Interfejs:** dane z silnika w `$state.raw`; wyszukiwanie przez `search()`/`haystack()`; długie listy w porcjach („Pokaż więcej”) albo zwinięte grupy; jeden nasłuch zdarzeń na kontener zamiast kilku na element; tablice SVG rysują tylko widoczny fragment (`FamilyGraph`).
5. **Drzewo „Wokół osoby”:** w drzewach powyżej 1 500 osób domyślnie 6 pokoleń przodków i 3 potomków, dopóki użytkownik sam nie wybierze.
