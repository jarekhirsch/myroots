# MyRoots dla WordPressa

## 3.1.0 (2026-11-23)

Wtyczka zawiera aplikację MyRoots 3.1 (zmiany: CHANGELOG.md aplikacji).

- Źródła przy każdej informacji albo jedna lista na osobę (do wyboru w Ustawieniach).
- Publikacja strony i przywracanie kopii wysyłają kilka zdjęć naraz, ponawiają kawałek pliku po zgubionej odpowiedzi serwera i ostrzegają przed zamknięciem karty w trakcie.
- Mapy z OpenFreeMap, z nazwami w języku strony.
- Drzewo: linia główna, kolory linii dziadków, filtr rodu, kolory drzewa w Wyglądzie.
- Pola „Adres” i „Niepotwierdzone informacje”, przyczyna śmierci na liście osób, migracje z jednego miejsca na mapie.
- Pasek postępu przy wgrywaniu zdjęć, imporcie GEDCOM i przywracaniu kopii.
- Ponowny import GEDCOM nie dubluje osób; duże drzewa działają szybko.

## 3.0.0 (2026-10-20)

Wtyczka przebudowana na aplikację MyRoots 3: ten sam interfejs (Svelte) i ten sam silnik w Rust (WebAssembly) co w aplikacji na Maca, Windowsa, Linuksa, telefony i przeglądarkę.

- Edytor w panelu WordPressa otwiera się na pełnym ekranie i wygląda jak aplikacja na Maca. Powrót do WordPressa jest w menu aplikacji („Więcej”), tam też widać stan zapisu.
- Dane: tabela `wp_myroots_records` (rekordy silnika 1:1), zdjęcia i nagrania w `wp-content/uploads/myroots/files-…` (dostępne tylko dla redaktorów).
- Strona rodzinna: przeglądarka tylko do odczytu z aplikacji, publikowana przyciskiem albo automatycznie po zmianach. Prywatność żyjących liczy silnik; wysyłane są tylko publiczne dane i brakujące zdjęcia.
- Adres strony: `/drzewo/` (polski WordPress) lub `/family-tree/`, do zmiany w MyRoots → Ustawienia. Widoczność: wszyscy, zalogowani albo nikt.
- Blok „MyRoots” i shortcode `[myroots view="/tree" height="85vh"]`. Shortcody 2.x (`[myrootstree]`, `[myrootsmap]` itd.) dalej działają.
- Propozycje poprawek z rodzinnej strony trafiają do WordPressa i same pojawiają się w aplikacji (Propozycje), opcjonalnie z e-mailem.
- Przenosiny z 2.x jednym przyciskiem (Import i kopia → Z WordPressa → „Importuj z tej strony”), razem ze zdjęciami z biblioteki mediów. Dane 2.x zostają nietknięte.
- Kopia `.myroots` jest zgodna z aplikacją desktopową: można przenosić drzewo między WordPressem a Makiem w obie strony.
- Rola „Redaktor genealogii” i uprawnienie `myroots_manage` z 2.x działają dalej.
- Kilka drzew na jednej stronie (Drzewa genealogiczne w menu aplikacji). Każde drzewo ma własną tabelę (`wp_myroots_records_<id>`), własne foldery zdjęć i strony, własne opcje oraz adres `/drzewo/<nazwa>/` i shortcode `[myroots tree="<nazwa>"]`. Pierwsze drzewo zostaje pod dotychczasowymi nazwami, więc istniejące dane działają bez zmian.
- Hasło do strony rodzinnej: na całą stronę albo na wybrane osoby i gałęzie (z potomkami i partnerami albo z przodkami). Chroniona część i jej zdjęcia są szyfrowane (AES-GCM, klucz z PBKDF2-SHA256, 250 000 iteracji) w przeglądarce redaktora przed wysłaniem. Hasło nie jest nigdzie zapisywane, a klucz nie trafia na stronę.
- PWA: strona rodzinna każdego drzewa ma manifest i service worker (instalacja na telefonie i komputerze, działanie offline po pierwszym obejrzeniu); panel edycji ma własny manifest i też da się go zainstalować jako osobne okno.
