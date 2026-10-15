# Zmiany

## 3.0.0 (2026-10-15)

Pierwsze samodzielne wydanie MyRoots, bez WordPressa. Ta sama aplikacja działa na Macu, Windowsie i Linuksie, na telefonach (iOS, Android) i w przeglądarce, a z każdej z nich można wydać stronę rodzinną tylko do odczytu.

**Nowe**
- Drzewo: tryb „Wokół osoby” i „Cała rodzina” (wszystkie osoby i powiązania na jednej tablicy), dodawanie osób prosto z drzewa, panel z danymi osoby.
- Start: oś czasu pokoleń z powiększaniem i zwijaniem pokoleń.
- „Przez lata”: zdjęcia osoby z różnych lat, wyrównane na oczach (twarz znajdowana na urządzeniu), odtwarzane z rokiem i wiekiem; zapis do wideo.
- Nagrania głosu: nagrywanie z mikrofonu lub dodawanie plików; odtwarzacz w profilu i na stronie rodzinnej; nagranie jako podkład do „Przez lata”.
- Mapa: skupiska miejsc jako kółka z liczbą, pasek narzędzi jak w drzewie, wybór osoby.
- Przycisk „Powrót” i skróty (Alt+←, Cmd+[, przycisk „wstecz” myszy).
- GEDCOM: obsługa ANSEL, Windows-1250/1252, ISO-8859-2, Mac Roman, GEDCOM 7 i spakowanych plików (.gdz, .zip); nazwiska po mężu i rodowe, wiele nowych zdarzeń, źródła wpisane w treści, daty z innych programów; eksport zgodny z GEDCOM 5.5.1.
- „Nie żyje, data śmierci nieznana” dla osób bez daty śmierci.
- Okno „O aplikacji” z licencją i listą użytych bibliotek.

**Bezpieczeństwo i prywatność**
- Strona rodzinna nie zawiera ukrytych danych zdjęć (położenie GPS, aparat, komentarze).
- Linki z importowanych plików są sprawdzane (tylko http, https, mailto), treść HTML jest oczyszczana, aplikacja ma politykę Content-Security-Policy.
- Testy odporności: losowe i uszkodzone pliki GEDCOM, kopie zapasowe i dane nie zawieszają aplikacji.

**Licencja**
- GNU GPL w wersji 3 lub późniejszej z warunkami dodatkowymi (patrz `NOTICE.md`).
