# Silnik (`crates/myroots-core`)

Wspólny dla wszystkich platform. Nie zna interfejsu ani sposobu zapisu: dostaje JSON, zwraca JSON i zbiera zmiany.

## Moduły

| Moduł | Rola |
|---|---|
| `engine.rs` | `Engine::call(method, params, ctx)`: rozdziela metody, sprawdza tryb tylko do odczytu (strona rodzinna), stosuje politykę prywatności. |
| `db.rs` | `Database` w pamięci, `put_*`/`del_*` ze śledzeniem zmian (`ChangeSet`), `Index` (dzieci, związki, zwierzęta i źródła osoby) **z pamięcią podręczną**, `places_by_name()`. |
| `model.rs` | Typy rekordów i ustawień, `CITABLE_FIELDS`, `norm()` (porównywanie nazw), `strip_tags`, bezpieczne adresy i typy plików. Opis: [model.md](model.md). |
| `ops.rs` | Wszystkie zmiany: zapis osoby z czyszczeniem danych (`sanitize_person`), relacje (z ochroną przed pętlą przodków), związki, źródła, zdjęcia, miejsca, historia zmian. |
| `query.rs` | Widoki dla interfejsu: skróty osób (`Summary`), profil, model edycji, źródła, groby i cmentarze, wydarzenia, rocznice, miejsca, galeria. |
| `privacy.rs` | `Policy`: właściciel widzi wszystko, strona rodzinna według ustawień (ukrycie lub cenzura żyjących, zablokowane sekcje). `view()` zwraca osobę bez kopiowania, gdy nic nie jest cenzurowane. |
| `tree.rs` | Węzły drzewa (format family-chart). |
| `map.rs` | Mapa: wydarzenia według miejsc, groby (z własnym położeniem albo położeniem cmentarza), lista cmentarzy z osobami. |
| `lifeline.rs`, `kinship.rs`, `stats.rs`, `book.rs` | Linia życia, pokrewieństwo, statystyki, księga rodu. |
| `health.rs` | Spójność danych i kompletność. |
| `merge.rs` | Możliwe duplikaty (grupowanie po nazwisku i pierwszej literze imienia), porównanie i scalanie dwóch osób. |
| `gedcom.rs`, `date.rs`, `charsets.rs` | GEDCOM, daty z kwalifikatorami (`~`, `<`, `>`, przedziały), kodowania. Zakres: [../GEDCOM.md](../GEDCOM.md). |
| `archive.rs` | Kopia `.myroots` (ZIP z `myroots.json` i zdjęciami), limity przeciw „zip bombom”. |
| `site.rs` | Strona rodzinna: publiczna kopia bazy (prywatność, tylko opublikowane osoby, źródła użyte przez widoczne osoby) i ZIP. |
| `wp_import.rs` | Import kopii JSON wtyczki WordPress 2.x. |
| `metadata.rs` | Czyszczenie metadanych zdjęć (EXIF, GPS) przed publikacją. |
| `fuzz.rs` | Testy odporności (losowe dane i pliki). |

Pozostałe crate'y: `myroots-store` (SQLite + pliki zdjęć), `myroots-wasm` (most `wasm-bindgen`), `src-tauri` (komendy Tauri, protokół `media://`, drzewa, aktualizacje).

## Indeks relacji

`db.index()` zwraca `Arc<Index>` i buduje go tylko wtedy, gdy dane się zmieniły (licznik `generation` zwiększany w `track()` przy każdym `put_*`/`del_*`, plus liczba rekordów). Wcześniej indeks był budowany przy każdym wywołaniu, przez co profil osoby w drzewie 20 000 osób otwierał się 40–60 ms, teraz 0,1 ms.

Zasada: **wszystkie zmiany przez `put_*`/`del_*`**. Kod, który musi zmienić mapę bezpośrednio (`db.persons.insert`), woła potem `db.touch()`.

## Jak dodać metodę silnika

1. Gałąź w `match method` w `engine.rs` (funkcja w `query.rs` dla odczytu albo `ops.rs` dla zmiany).
2. Jeśli metoda tylko czyta: dopisz ją do `READ` w `src/lib/api.svelte.ts`. Inaczej każde jej wywołanie zwiększy `app.version`, a otwarty widok przeładuje się w pętli.
3. Jeśli ma działać na stronie rodzinnej: dopisz do `READ_ONLY` w `engine.rs` i sprawdź prywatność (`pol`).
4. Test w module (`#[cfg(test)]`), `cargo test -p myroots-core`.
5. `npm run wasm` (przeglądarka, strona rodzinna, WordPress i serwer używają WASM).
6. `npm run docs`, żeby tabela niżej się zaktualizowała.

## Wszystkie metody

<!-- auto:engine-methods -->
Metod: 77. „odczyt” = pasuje do `READ` w `src/lib/api.svelte.ts` (nie odświeża widoków). „strona” = dozwolona na stronie rodzinnej (`READ_ONLY` w `engine.rs`).

| Metoda | Rodzaj | Strona |
|---|---|---|
| `app.info` | odczyt | tak |
| `overview` | odczyt | tak |
| `settings.get` | odczyt | tak |
| `settings.save` | **zapis** |  |
| `data.reset` | **zapis** |  |
| `people.summaries` | odczyt | tak |
| `person.profile` | odczyt | tak |
| `person.edit` | odczyt |  |
| `person.create` | **zapis** |  |
| `person.update` | **zapis** |  |
| `person.delete` | **zapis** |  |
| `person.add_relative` | **zapis** |  |
| `person.link` | **zapis** |  |
| `person.set_parent` | **zapis** |  |
| `person.history` | odczyt | tak |
| `person.restore` | **zapis** |  |
| `person.set_media` | **zapis** |  |
| `person.set_sources` | **zapis** |  |
| `union.save` | **zapis** |  |
| `union.delete` | **zapis** |  |
| `pets.list` | odczyt | tak |
| `pet.get` | odczyt | tak |
| `pet.save` | **zapis** |  |
| `pet.delete` | **zapis** |  |
| `sources.list` | odczyt | tak |
| `sources.brief` | odczyt |  |
| `source.get` | odczyt | tak |
| `source.save` | **zapis** |  |
| `source.delete` | **zapis** |  |
| `media.reserve` | **zapis** |  |
| `media.register` | **zapis** |  |
| `media.update` | **zapis** |  |
| `media.get` | odczyt | tak |
| `media.tags` | **zapis** |  |
| `media.delete` | **zapis** |  |
| `media.list` | odczyt | tak |
| `media.faces` | odczyt |  |
| `media.unused` | odczyt |  |
| `clans.list` | odczyt | tak |
| `clans.rename` | **zapis** |  |
| `clans.delete` | **zapis** |  |
| `clans.assign` | **zapis** |  |
| `graves.list` | odczyt | tak |
| `graves.cemeteries` | odczyt |  |
| `places.list` | odczyt | tak |
| `places.missing` | odczyt |  |
| `places.set` | **zapis** |  |
| `places.weather` | odczyt |  |
| `places.rename` | **zapis** |  |
| `tree.data` | odczyt | tak |
| `lifeline` | odczyt | tak |
| `map.data` | odczyt | tak |
| `stats` | odczyt | tak |
| `anniversaries` | odczyt | tak |
| `events.list` | odczyt | tak |
| `gallery` | odczyt | tak |
| `kinship` | odczyt | tak |
| `health.issues` | odczyt |  |
| `health.count` | odczyt |  |
| `health.completeness` | odczyt |  |
| `merge.suggestions` | odczyt |  |
| `merge.dismiss` | **zapis** |  |
| `merge.compare` | odczyt |  |
| `merge.execute` | **zapis** |  |
| `book.build` | odczyt | tak |
| `suggestions.list` | odczyt |  |
| `suggestion.encode` | odczyt | tak |
| `suggestions.import` | **zapis** |  |
| `suggestions.add` | **zapis** |  |
| `suggestions.set_status` | **zapis** |  |
| `suggestions.delete` | **zapis** |  |
| `gedcom.import` | **zapis** |  |
| `gedcom.export` | odczyt |  |
| `wp.import` | **zapis** |  |
| `wp.pending` | odczyt |  |
| `wp.attach` | **zapis** |  |
| `wp.clear` | **zapis** |  |
<!-- /auto:engine-methods -->
