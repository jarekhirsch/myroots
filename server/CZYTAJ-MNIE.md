# MyRoots Server

Ta sama aplikacja MyRoots co na Macu, Windowsie i w WordPressie, ale jako samodzielna strona na Twoim serwerze. Nie potrzebuje WordPressa ani bazy MySQL: wystarczy zwykły hosting z PHP.

- Edytujesz drzewo w przeglądarce, z dowolnego komputera, pod adresem `…/admin`.
- Rodzina ogląda stronę rodzinną pod adresem głównym. Każde kolejne drzewo ma własny adres, np. `…/rodzina-nowakow/`.
- Zdjęcia, baza danych i opublikowane strony zostają w folderze `data/` na Twoim serwerze.
- Działa wszystko z aplikacji: kilka drzew, hasło do całej strony albo wybranych gałęzi, propozycje poprawek od rodziny, instalacja jak aplikacja na telefonie (PWA).
- Konta mają trzy role:
  - **Administrator**: wszystko, łącznie z kontami i ustawieniami serwera.
  - **Redaktor**: edytuje drzewa i publikuje stronę.
  - **Rodzina**: tylko ogląda, gdy strona jest dostępna wyłącznie dla osób z kontem.

## Wymagania

- PHP 8.0 lub nowsze z rozszerzeniami `pdo_sqlite` i `mbstring`. Na polskich hostingach (home.pl, nazwa.pl, cyber_Folks, LH.pl, Zenbox i podobnych) zwykle są włączone.
- Apache albo LiteSpeed z obsługą `.htaccess`. Większość hostingów tak działa. Przykład dla nginx jest w pliku `nginx.conf.example`.
- HTTPS (darmowy Let’s Encrypt w panelu hostingu). Bez niego hasła lecą otwartym tekstem, a strony nie da się zainstalować jak aplikacji.

## Instalacja na hostingu

1. Wgraj cały folder `myroots-server` przez FTP do katalogu strony. Może to być katalog główny domeny albo podfolder, np. `public_html/rodzina/`.
2. Otwórz adres w przeglądarce, np. `https://twojadomena.pl/rodzina/`. Pojawi się instalator.
3. Wpisz nazwę strony, swój login i hasło administratora, wybierz, kto widzi stronę rodzinną, i kliknij **Zainstaluj**.
4. Otworzy się aplikacja. Zaimportuj kopię `.myroots` albo plik GEDCOM (Import i kopia) albo zacznij od pierwszej osoby. Potem w zakładce **Strona rodzinna** kliknij **Opublikuj na stronie**.
5. W **Ustawieniach serwera** (menu aplikacji → Ustawienia serwera) dodaj konta dla rodziny i redaktorów.

Jeśli instalator pokaże błąd o folderze `data`, nadaj mu w FTP prawa zapisu (755 albo 775).

### Bezpieczniej: dane poza folderem publicznym

Folder `data/` jest zablokowany przez `.htaccess`. Lepiej jednak trzymać dane poza katalogiem strony:

1. Przenieś folder `data` jeden poziom wyżej, np. jako `myroots-data`.
2. Skopiuj `config.sample.php` jako `config.php`.
3. Sprawdź w nim ścieżkę do danych.

## Docker (własny serwer, VPS, NAS)

```
docker compose up -d
```

Strona działa pod `http://localhost:8080`, a dane leżą w folderze `data/` obok. Do publikacji w internecie postaw przed kontenerem serwer z HTTPS, np. Caddy albo Nginx Proxy Manager.

## Próba na własnym komputerze

```
php -S localhost:8000 router.php
```

Potem otwórz http://localhost:8000.

## Kopie zapasowe

Wszystko jest w folderze `data/`: baza `myroots.sqlite`, zdjęcia (`files-*`) i opublikowane strony (`site-*`). Regularnie kopiuj ten folder. Możesz też zrobić kopię `.myroots` w aplikacji (Import i kopia) i przenieść ją do aplikacji na Maca, Windowsa, Androida albo do wtyczki WordPress.

## Aktualizacja

Wgraj nowe pliki na stare, z wyjątkiem folderu `data/` i pliku `config.php`.

## Bezpieczeństwo w skrócie

- Hasła są przechowywane jako skróty (bcrypt).
- Logowanie ma limit 8 prób na 15 minut z jednego adresu.
- Zmiany wymagają tokenu sesji, a ciasteczka są `HttpOnly` i `SameSite=Lax`.
- Zdjęcia w edytorze widzą tylko zalogowani redaktorzy.
- Na stronę rodzinną trafia tylko to, co aplikacja opublikuje zgodnie z ustawieniami prywatności. Część chroniona hasłem jest szyfrowana jeszcze w przeglądarce.
