# Zmiany

## 3.1.0 (2026-11-22)

Duże drzewa działają płynnie, źródła można przypisać do każdej informacji, a MyRoots jest dostępny na Linuksie.

**Nowe**
- Źródła przy każdej informacji, tak jak w MyHeritage: przycisk „Źródło” przy imieniu, urodzeniu, śmierci, zawodzie, wykształceniu, wyznaniu, biografii, grobie, każdym wydarzeniu i każdym małżeństwie. Przy źródle można wpisać stronę lub numer aktu, wiarygodność (od dowodu bezpośredniego do niewiarygodnego) i treść zapisu, a nowe źródło utworzyć bez wychodzenia z formularza.
- Ustawienia, karta „Źródła”: wybór między źródłami przy każdej informacji a jedną listą źródeł dla osoby z zaznaczaniem potwierdzanych informacji. Oba sposoby zapisują te same dane.
- Profil osoby: przy każdym źródle widać, co potwierdza, z numerem strony, wiarygodnością i treścią zapisu; numery źródeł także przy imieniu, grobie i małżeństwach.
- GEDCOM: import i eksport cytowań przy faktach, zdarzeniach i małżeństwach razem z PAGE, QUAY i notatką (DATA/TEXT, NOTE), także źródeł imienia, wykształcenia, wyznania i pochówku.
- Mapa: warstwa „Groby” i filtr cmentarzy. Wybrany cmentarz pokazuje groby pochowanych tam osób, a z pozostałymi warstwami także to, skąd pochodzili.
- Linux: paczki .deb (Ubuntu, Debian, Mint), .rpm (Fedora, openSUSE) i AppImage.
- Drzewo: wyróżnienie linii głównej (przodkowie i potomkowie wybranej osoby zostają duzi, linie boczne są mniejsze i bledsze) oraz kolory czterech linii dziadków, jak w MyHeritage. Oba włącza lista „Linie” nad drzewem.
- Drzewo: kliknięcie rodu na karcie osoby wyróżnia w drzewie tylko osoby z tego rodu i pokazuje ich listę. Z listy rodów można też otworzyć ród w drzewie i na mapie.
- Ustawienia, Wygląd: kolor tła drzewa, linii między osobami, linii do wybranej osoby i czterech linii dziadków.
- Osoba: pole „Adres” (miejsce zamieszkania), także w GEDCOM (RESI z ADDR), oraz „Niepotwierdzone informacje”: rodzinne opowieści i tropy do sprawdzenia, widoczne tylko dla Ciebie, nigdy na stronie rodzinnej.
- Lista osób pokazuje przyczynę śmierci i adres, a wyszukiwarka je przeszukuje.
- Mapa, migracje: „Kto z tego miejsca poszedł dalej”, czyli osoby urodzone w jednym miejscu (domu, wsi) i dokąd rozeszli się ich potomkowie. Kafelek „Gdzie mieszkała rodzina” na stronie startowej i przycisk „Migracje” na liście miejsc.
- GEDCOM: zdjęcia i skany z innych programów (z archiwum .gdz/.zip albo z folderu wskazanego po imporcie), archiwa i parafie ze sygnaturami (REPO), chrzestni i świadkowie (ASSO), przydomki (NICK, ALIA), obrzędy LDS oraz daty kalendarza juliańskiego, hebrajskiego i republikańskiego przeliczane na gregoriański.
- Osoba: przydomek oraz chrzestni, świadkowie, opiekunowie i inne powiązania, widoczne na obu profilach. Źródło: archiwum, parafia lub biblioteka i sygnatura.
- Mapa: warstwa „Cmentarze” z cmentarzami rodziny, cmentarzami z OpenStreetMap w okolicy i własnymi; cmentarz dodasz, wpisując adres albo klikając na mapie.
- Pasek postępu przy dodawaniu zdjęć i nagrań, wycinaniu zdjęć z albumu, imporcie GEDCOM i przywracaniu kopii.
- Aktualizacje: aplikacja na Maca, Windowsa i Linuksa (AppImage) raz dziennie sprawdza, czy jest nowa wersja, i proponuje jej instalację. Można to wyłączyć w Ustawieniach.

**Szybkość dużych drzew (kilkanaście tysięcy osób i więcej)**
- „Możliwe duplikaty” liczą się w ułamku sekundy zamiast kilkudziesięciu sekund.
- Profil, edycja osoby, linia życia i pokrewieństwo otwierają się od razu: powiązania rodzinne są liczone raz, a nie przy każdym kliknięciu.
- Listy osób, wyszukiwarki i pola wyboru osoby nie zwalniają przy wpisywaniu.
- „Cała rodzina” rysuje tylko to, co widać na ekranie; z daleka karty są uproszczone.
- Drzewo „Wokół osoby” w dużych drzewach zaczyna od 6 pokoleń przodków i 3 pokoleń potomków (można zmienić).
- Groby, spójność danych i brakujące dane pokazują się w porcjach, a strona główna nie liczy już całej listy problemów tylko po to, żeby pokazać ich liczbę.

**Poprawki**
- MyRoots Server: zdjęcia ładują się równolegle. Wcześniej sesja logowania blokowała kolejne zapytania i zdjęcia w galerii i drzewie przychodziły jedno po drugim, a publiczna strona rodzinna otwierała sesję przy każdym zdjęciu.
- MyRoots Server i WordPress: publikacja strony i przywracanie kopii wysyłają kilka zdjęć naraz (w teście o ok. 35% szybciej), ponawiają kawałek pliku, gdy hosting zgubi odpowiedź, i ostrzegają przed zamknięciem karty w trakcie. Strona bez pełnych zdjęć nie pobiera już do publikacji oryginałów.
- Mapy: podkład z OpenFreeMap (bez limitów i kluczy) zamiast kafelków openstreetmap.org, które czasem odpowiadały błędem 403, zwłaszcza w aplikacji na komputer. Nazwy miejscowości i krajów są w języku aplikacji (Wiedeń, Lwów, Nowy Jork). Style w ustawieniach: Standardowa, Jasna, Ciemna i Teren.
- Ponowny import GEDCOM z nowymi osobami nie dubluje już osób, które są w drzewie: rozpoznaje je po identyfikatorach z pliku (_UID, RIN, REFN i in.) albo po imieniu, nazwisku i datach.
- Menu „Więcej” pokazuje tylko to, czego nie ma na pasku, bez powtórzeń.
- Linki do repozytorium wskazują na github.com/jarekhirsch/myroots.

**Biblioteki**
- Vite 8, Svelte 5.59, Tauri 2.12, TypeScript 6, rusqlite 0.43, wasm-bindgen 0.2.130 i pozostałe zależności w najnowszych wersjach; audyt bez znanych podatności.

## 3.0.0 (2026-10-20)

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
- Tło historyczne: w linii życia ok. 160 najważniejszych wydarzeń z historii Polski od 1600 r. (wojny, rozbiory, powstania, reformy, epidemie dżumy i cholery, emigracja) z wiekiem osoby w tamtej chwili. Wydarzenia lokalne (np. w jednym zaborze, regionie albo mieście) pokazują się tylko osobom, których miejsca leżą w danym regionie. W Ustawieniach (zakładka „Tło historyczne”) można wyłączyć dowolne wbudowane wydarzenie i dodać własne, także przypisane do miejsca (pokazywane osobom z tego miejsca lub mieszkającym do 50 km od niego).
- Groby: cmentarz, miejsce (kwatera, rząd, numer), położenie (z mapy, z telefonu przy grobie albo samo ze zdjęcia nagrobka), zdjęcie i link do wyszukiwarki grobów. Strona „Groby” grupuje je według cmentarzy, pokazuje na mapie i układa trasę odwiedzin (np. na 1 listopada) w Mapach Google. Grób jest eksportowany do GEDCOM jako pochówek.
- Zdjęcia: podpowiedzi, kto jest na zdjęciu (rozpoznawanie twarzy na urządzeniu na podstawie wcześniejszych oznaczeń; dane twarzy nie trafiają na stronę rodzinną), „Przyjmij wszystkie podpowiedzi”.
- Zdjęcia: „Kiedy zrobiono?”: data zdjęcia, szacowana z wieku oznaczonych twarzy i dat ich urodzenia oraz śmierci.
- Galeria: „Skan strony albumu” dzieli zeskanowaną stronę na osobne, wyprostowane zdjęcia.
- Mapa: tryb „Migracje” z liniami między miejscami urodzenia rodziców i dzieci oraz przeprowadzkami w ciągu życia, z suwakiem lat i odtwarzaniem.
- Plakat do druku: wachlarz przodków (do 8 pokoleń) i drzewo potomków (w wyglądzie drzewa z aplikacji) w formatach A4–A0; druk do PDF, zapis SVG (z czcionką) i PNG.
- Pogoda z dnia wydarzenia (z Open-Meteo.com, dane od 1940 r.; można wyłączyć w Ustawieniach): w linii życia przy wydarzeniach z dokładną datą i miejscem na mapie. Pobrana pogoda zostaje zapisana przy miejscu.
- Okno „O aplikacji” z licencją i listą użytych bibliotek.

**Poprawki**
- Aplikacja na Maca: drukowanie (profil, księga, lista grobów, plakat) otwiera systemowe okno druku.
- Plakat: przycisk PDF zapisuje plik dokładnie w wybranym formacie papieru.
- Mapa: wyszukiwanie miejsc przedstawia się serwisowi OpenStreetMap (wcześniej odrzucał zapytania z aplikacji) i w razie braku wyników pyta drugi serwis (Photon); przy poprawianiu można od razu zmienić błędną nazwę miejsca w całym drzewie.
- Wydruki, plakaty i film „Przez lata” mają dyskretny dopisek „© MyRoots · Jarek Hirsch”.
- Aplikacja na komputer: dodawanie zdjęć (druga polityka bezpieczeństwa z wersji przeglądarkowej blokowała pliki); czytelny komunikat, gdy zdjęcia nie da się dodać (np. HEIC w przeglądarce).

**Bezpieczeństwo i prywatność**
- Strona rodzinna nie zawiera ukrytych danych zdjęć (położenie GPS, aparat, komentarze).
- Linki z importowanych plików są sprawdzane (tylko http, https, mailto), treść HTML jest oczyszczana, aplikacja ma politykę Content-Security-Policy.
- Testy odporności: losowe i uszkodzone pliki GEDCOM, kopie zapasowe i dane nie zawieszają aplikacji.

**Licencja**
- GNU GPL w wersji 3 lub późniejszej z warunkami dodatkowymi (patrz `NOTICE.md`).
