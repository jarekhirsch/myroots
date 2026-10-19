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
- Edycja osoby: nowego rodzica, małżonka lub dziecko można utworzyć od razu z pola wyboru („+ Dodaj nową osobę”).
- Mapa: lista miejsc bez współrzędnych (przycisk „Popraw”) z wyszukiwaniem pod inną nazwą albo wskazaniem miejsca kliknięciem na mapie.
- „Cała rodzina”: kliknięta osoba podświetla wszystkich swoich przodków do kliknięcia w tło lub klawisza Esc.
- Osoby na wspólnych zdjęciach: przycisk „Oznacz osoby” w podglądzie zdjęcia wyszukuje twarze na urządzeniu; twarz (albo zaznaczony obszar) można przypisać osobie z drzewa, nowej osobie albo tylko podpisać. Imiona są pod zdjęciem, a w profilu osoby pojawia się sekcja „Także na zdjęciach”. Na stronie rodzinnej oznaczenia respektują ustawienia prywatności.
- Tło historyczne: w linii życia najważniejsze wydarzenia z historii Polski (od Konstytucji 3 maja po 2022 r.) z wiekiem osoby w tamtej chwili; można je ukryć.
- Groby: cmentarz, miejsce (kwatera, rząd, numer), położenie (z mapy, z telefonu przy grobie albo samo ze zdjęcia nagrobka), zdjęcie i link do wyszukiwarki grobów. Strona „Groby” grupuje je według cmentarzy, pokazuje na mapie i układa trasę odwiedzin (np. na 1 listopada) w Mapach Google. Grób jest eksportowany do GEDCOM jako pochówek.
- Zdjęcia: podpowiedzi, kto jest na zdjęciu (rozpoznawanie twarzy na urządzeniu na podstawie wcześniejszych oznaczeń; dane twarzy nie trafiają na stronę rodzinną), „Przyjmij wszystkie podpowiedzi”.
- Zdjęcia: „Kiedy zrobiono?”: data zdjęcia, szacowana z wieku oznaczonych twarzy i dat ich urodzenia oraz śmierci.
- Galeria: „Skan strony albumu” dzieli zeskanowaną stronę na osobne, wyprostowane zdjęcia.
- Mapa: tryb „Migracje” z liniami między miejscami urodzenia rodziców i dzieci oraz przeprowadzkami w ciągu życia, z suwakiem lat i odtwarzaniem.
- Plakat do druku: wachlarz przodków (do 8 pokoleń) i drzewo potomków w formatach A4–A0; druk do PDF, zapis SVG (z czcionką) i PNG.
- Pogoda z dnia wydarzenia (z Open-Meteo.com, dane od 1940 r.; można wyłączyć w Ustawieniach): w linii życia przy wydarzeniach z dokładną datą i miejscem na mapie. Pobrana pogoda zostaje zapisana przy miejscu.
- Okno „O aplikacji” z licencją i listą użytych bibliotek.

**Poprawki**
- Aplikacja na komputer: dodawanie zdjęć (druga polityka bezpieczeństwa z wersji przeglądarkowej blokowała pliki); czytelny komunikat, gdy zdjęcia nie da się dodać (np. HEIC w przeglądarce).

**Bezpieczeństwo i prywatność**
- Strona rodzinna nie zawiera ukrytych danych zdjęć (położenie GPS, aparat, komentarze).
- Linki z importowanych plików są sprawdzane (tylko http, https, mailto), treść HTML jest oczyszczana, aplikacja ma politykę Content-Security-Policy.
- Testy odporności: losowe i uszkodzone pliki GEDCOM, kopie zapasowe i dane nie zawieszają aplikacji.

**Licencja**
- GNU GPL w wersji 3 lub późniejszej z warunkami dodatkowymi (patrz `NOTICE.md`).
