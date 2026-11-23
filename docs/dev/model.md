# Model danych (`crates/myroots-core/src/model.rs`)

Wszystkie rekordy mają `#[serde(default)]`: brakujące pola wczytują się z wartością domyślną, nieznane są pomijane. Dzięki temu kopie ze starszych i nowszych wersji wczytują się bez błędu. Nowe pole: zawsze z sensowną wartością domyślną, a gdy bywa puste, z `skip_serializing_if`, żeby nie powiększać kopii.

## Rekordy

| Rekord | Najważniejsze pola | Uwagi |
|---|---|---|
| `Person` | imię, nazwisko, nazwisko rodowe, płeć, daty i miejsca urodzenia i śmierci (daty w formacie `date.rs`), `deceased`, biografia, notatki i `research` (niepotwierdzone informacje; HTML z białej listy), zawód, wykształcenie, wyznanie, `residence` (adres), `ext_ids` (identyfikatory z innych programów, z GEDCOM), `nickname` (przydomek), `associations` (chrzestni, świadkowie i in.: `{person, role, event}`; role znane aplikacji: godparent, witness, guardian, clergy, officiator, friend, neighbor, inne jako wolny tekst; widok odwrotny liczy `query::profile`), `mother`/`father` z rodzajem relacji, `events`, `clans`, zdjęcie, galeria, nagrania, „Przez lata”, `grave`, `custom`, `citations`, `citation_info` | Dzieci nie są zapisane: wynikają z `mother`/`father` (indeks). `research` nigdy nie wychodzi na stronę rodzinną (`site::public_db` czyści), a prywatność żyjących czyści `residence` (miejsca) i `research` (biografia). |
| `Union` | `a`, `b` (albo imię partnera spoza drzewa), data i miejsce, koniec związku, `informal`, `citations` | Jeden rekord na parę. |
| `LifeEvent` (w osobie) | nazwa, data, miejsce, opis, zdjęcie, `citations` | Standardowe nazwy GEDCOM tłumaczone przy imporcie. |
| `Grave` (w osobie) | cmentarz, kwatera, `lat`/`lng`, zdjęcie nagrobka, link, notatki | Eksport GEDCOM jako `BURI`. |
| `Source` | tytuł, data, adres, notatki, nagranie, okładka, skany, `persons` (ogólne powiązania) | |
| `Media` | plik, typ, wymiary, miniatura, podpis, oznaczenia osób (`tags`, z opisem twarzy), data | Bajty zdjęć trzyma platforma, nie silnik. |
| `Place` | nazwa, `lat`/`lng`, pochodzenie współrzędnych, pogoda | Dopasowanie po nazwie bez wielkości liter. |
| `Pet`, `Revision`, `Suggestion` | zwierzę, wersja osoby (historia), propozycja poprawki od rodziny | |
| `Settings` | patrz niżej | Jeden rekord. |
| `Meta` | `next_id`, `version` (licznik zmian) | |

## Źródła przy faktach (od 3.1)

- `Person.citations: BTreeMap<String, Vec<Id>>`: pole z `CITABLE_FIELDS` (`name`, `birth_date`, `birth_place`, `death_date`, `death_place`, `death_cause`, `burial`, `occupation`, `faith`, `education`, `residence`, `biography`) → źródła.
- `Person.citation_info: BTreeMap<String, CitationInfo>`: szczegóły cytowania pod kluczem `citation_key(pole, źródło)` = `"pole#id"`: `page`, `quality` (0–3 jak GEDCOM `QUAY`), `note`. Brak wpisu = cytowanie bez szczegółów.
- `LifeEvent.citations` i `Union.citations`: `Vec<Citation>` (`source`, `page`, `quality`, `note`).
- Interfejs łączy pola w fakty (`src/lib/citations.ts`, `FACTS`): urodzenie = `birth_date` + `birth_place`, śmierć = data, miejsce i przyczyna. Tak samo GEDCOM (`BIRT.SOUR`).
- Ustawienie `sources_mode`: `""`/`"facts"` (przyciski przy faktach) albo `"general"` (lista z zaznaczaniem). Dane są te same.
- Pilnują spójności: `ops::sanitize_person` (tylko istniejące źródła, szczegóły tylko dla istniejących cytowań), `ops::clean_citations`, `ops::delete_source` (usuwa cytowania wszędzie), `merge.rs`, `site.rs` (tylko źródła eksportowane na stronę).

## Ustawienia (`Settings`)

`tree_name`, `max_age` (domniemana śmierć), `privacy`, `map_external` (mapa, domyślnie wyłączona), `weather_off`, `updates_off` (aktualizacje programu na komputer), `appearance`, `custom_schema` (pola własne), `clans`, `sort_order`, `people_per_page`, `tree_start_mode`/`tree_start_person`, `merge_dismissed`, `notifications`, `suggestions`, `site`, `history`, `language`, `sources_mode`. Typ w TypeScript: `Settings` w `src/lib/types.ts` (trzymaj oba w zgodzie).

## Wygląd (`Appearance`)

Kolory aplikacji (`primary`, `accent`, `bg`, `text`, `border`), czcionka, rozmiar, zaokrąglenia, cienie, motyw, styl mapy oraz kolory drzewa: `tree_bg`, `tree_line`, `tree_line_main`, `tree_branches` (4 kolory linii dziadków: ojciec ojca, matka ojca, ojciec matki, matka matki). `src/lib/theme.ts` zamienia je na zmienne CSS (`--tree-bg`, `--tree-line`, `--tree-line-main`, `--tree-br1`…`4`); kolory drzewa działają w obu motywach.

## Daty

Tekst w formacie `date.rs`: `1900-03-05`, `1900-03`, `1900`, z kwalifikatorem `~` (około), `<` (przed), `>` (po), przedział `1900-01-01/1901`. `date::sort_key` daje klucz do sortowania (kosztowny: w pętlach używaj `sort_by_cached_key`).

## Źródła i miejsca: pola z 3.1

- `Source.repository`, `Source.call_number`: archiwum lub parafia i sygnatura (GEDCOM `REPO`, `CALN`).
- `Place.kind = "cemetery"` i `Place.address`: cmentarz zapisany przez użytkownika na mapie (`cemeteries.save` / `cemeteries.remove`, `ops::save_cemetery`). Zmiana nazwy cmentarza zmienia ją w grobach. Mapa (`map::build`) pokazuje cmentarze z grobów rodziny i te zapisane, nawet bez pochówków.
