# MyRoots: polityka prywatności

Obowiązuje od 20 października 2026 r. Wydawca aplikacji: Jarek Hirsch (https://hirsch.lt).

## Krótko

MyRoots nie ma kont, serwerów ani reklam i nie zbiera żadnych danych. Wszystko, co wpisujesz (osoby, zdjęcia, nagrania, notatki), zostaje na Twoim urządzeniu. Autor aplikacji nie ma do tych danych dostępu.

## Gdzie są Twoje dane

- Aplikacja na komputer i telefon: w folderze danych aplikacji na urządzeniu.
- Wersja w przeglądarce: w pamięci przeglądarki (IndexedDB) na tym komputerze.
- Kopie zapasowe (.myroots), pliki GEDCOM i strona rodzinna powstają tylko wtedy, gdy je zapiszesz, i trafiają tam, gdzie wskażesz.

## Kiedy aplikacja łączy się z internetem

Tylko w tych sytuacjach i tylko z Twojej inicjatywy:

- **Mapa** (domyślnie wyłączona): po jej włączeniu fragmenty mapy są pobierane z serwerów OpenStreetMap, a nazwy miejsc są wysyłane do wyszukiwarki Nominatim (OpenStreetMap), a gdy ta nic nie znajdzie, do wyszukiwarki Photon (komoot), żeby znaleźć współrzędne. Warstwa „Cmentarze” wysyła do usługi Overpass (OpenStreetMap) tylko granice widocznego fragmentu mapy, żeby pokazać leżące tam cmentarze. Obowiązuje polityka prywatności OpenStreetMap: https://osmfoundation.org/wiki/Privacy_Policy
- **Pogoda** (można ją wyłączyć w Ustawieniach): dla wydarzeń z dokładną datą aplikacja wysyła do Open-Meteo.com współrzędne miejsca i datę, żeby pobrać historyczną pogodę tego dnia. Nazwy, osoby i inne dane nie są wysyłane. Warunki: https://open-meteo.com/en/terms
- **Aktualizacje** (tylko program na komputer; można je wyłączyć w Ustawieniach): raz dziennie program pobiera z myroots.cc plik z numerem najnowszej wersji, a po Twojej zgodzie samą aktualizację. Nic z Twojego drzewa nie jest wysyłane; serwer widzi tylko zwykłe zapytanie o plik.
- **Import z WordPressa**: aplikacja pobiera zdjęcia z Twojej witryny WordPress.
- **Linki i trasa odwiedzin grobów**: otwierają się w przeglądarce (trasa w Mapach Google zawiera tylko położenie wybranych cmentarzy).

## Mikrofon, aparat i zdjęcia

- Mikrofon jest używany tylko podczas nagrywania głosu, które sam rozpoczynasz. Nagranie zostaje na urządzeniu.
- Aparat i biblioteka zdjęć są używane tylko wtedy, gdy sam dodajesz zdjęcie.
- Funkcja „Przez lata” znajduje oczy na zdjęciach na Twoim urządzeniu. Nic nie jest wysyłane; zapisywane jest tylko położenie oczu na zdjęciu.
- Oznaczanie osób na zdjęciach, podpowiedzi „kto to jest” i szacowanie, kiedy zrobiono zdjęcie, działają wyłącznie na Twoim urządzeniu. Przy oznaczonej twarzy aplikacja zapisuje jej opis liczbowy (do rozpoznawania tej osoby na innych zdjęciach). Jest on częścią Twojego drzewa i kopii zapasowej, nigdy nie trafia na stronę rodzinną ani do internetu.

## Lokalizacja

Położenie urządzenia jest odczytywane tylko po kliknięciu „Jestem przy grobie” (zapisuje położenie grobu) albo „Trasa odwiedzin” (ustala kolejność cmentarzy). Nie jest nigdzie wysyłane.

## Strona rodzinna

Stronę rodzinną tworzysz i publikujesz sam. Możesz ukryć lub ocenzurować dane żyjących osób. Przy eksporcie aplikacja usuwa ze zdjęć ukryte dane (m.in. położenie GPS) i opisy twarzy; oznaczenia osób, groby i pogoda respektują ustawienia prywatności. Formularz „Zaproponuj poprawkę” na stronie wysyła wiadomość na adres, który sam ustawisz.

## Dzieci

MyRoots nie jest przeznaczony dla dzieci i nie zbiera danych od nikogo.

## Kontakt

Pytania o prywatność: https://hirsch.lt

---

# MyRoots: privacy policy

Effective 20 October 2026. Publisher: Jarek Hirsch (https://hirsch.lt).

MyRoots has no accounts, servers or ads and collects no data. Everything you enter (people, photos, recordings, notes) stays on your device; the author has no access to it.

The app connects to the internet only when you choose to: the **map** (off by default) loads map tiles from OpenStreetMap and sends place names to the Nominatim search service (and, when it finds nothing, to Photon by komoot) to find coordinates, and the Cemeteries layer sends only the bounds of the visible map to the Overpass service (OpenStreetMap) to show the cemeteries there; the **weather** (can be turned off in the settings) sends the coordinates of a place and a date to Open-Meteo.com to get the historical weather of that day; **updates** (desktop app only, can be turned off in the settings) read the number of the newest version from myroots.cc once a day and, after you agree, download the update, without sending anything about your tree; the **WordPress import** downloads photos from your own WordPress site; **links** and the route of a visit to the graves open in your browser (the Google Maps route contains only the positions of the chosen cemeteries).

The **microphone** is used only while you record a voice; the recording stays on your device. The **camera and photo library** are used only when you add a photo. "Through the years" finds the eyes on photos on your device; nothing is sent anywhere. Marking people on photos, suggestions of who is on a photo and the estimate of when it was taken run only on your device; for a marked face a numeric description is saved with your tree and backups (to recognise the person on other photos) and is never put on the family website or sent anywhere. The device's **location** is read only when you tap "I am at the grave" or "Route of the visit", and is not sent anywhere.

The **family website** is created and published by you; you choose how the data of living people is hidden, and hidden photo data (such as GPS location) and face descriptions are removed on export.

MyRoots is not directed at children and collects no data from anyone. Contact: https://hirsch.lt
