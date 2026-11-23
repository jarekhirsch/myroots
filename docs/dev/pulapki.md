# Pułapki

Rzeczy, które już raz się zepsuły albo łatwo je zepsuć.

1. **Nowa metoda tylko do odczytu bez wpisu w `READ`** (`src/lib/api.svelte.ts`): widok, który jej używa, przeładowuje się w kółko (każde wywołanie podbija `app.version`).
2. **Zmiana mapy rekordów z pominięciem `put_*`/`del_*`**: zmiana nie zapisze się na dysku i indeks relacji zostanie stary. Jeśli musisz, wołaj `db.touch()`.
3. **`$state` zamiast `$state.raw` dla danych z silnika**: wszystko działa, ale przy dużym drzewie kilka razy wolniej. Dane z `useLoad` są tylko do odczytu.
4. **Szablon strony rodzinnej** (`vite build --mode viewer`) jest budowany jako jeden plik IIFE i kopiowany do `public/viewer/`. Rolldown ostrzega przy tym o `import.meta` (EMPTY_IMPORT_META): to oczekiwane, `import.meta.glob` i `import.meta.env` są podmieniane wcześniej.
5. **Wersja `wasm-bindgen`** w `crates/myroots-wasm/Cargo.toml` (z `=`) musi być równa zainstalowanemu `wasm-bindgen-cli`, inaczej `npm run wasm` się nie uda.
6. **Pliki aktualizacji** (`createUpdaterArtifacts`) wymagają klucza. Nie włączaj ich w głównym `tauri.conf.json`, tylko przez `--config src-tauri/tauri.updater.conf.json`.
7. **Windows i podpis aktualizacji**: podpis aktualizacji (`.sig`) powstaje z instalatora już podpisanego przez SignPath (krok „Update signature” w `release.yml`). Podpisanie wcześniej daje nieważny `.sig`.
8. **Uprawnienia Tauri**: `capabilities/default.json` obowiązuje też na telefonach, `capabilities/desktop.json` tylko na komputerach (updater, process). Uprawnienie pluginu, którego nie ma na danej platformie, psuje budowanie.
9. **Tłumaczenia**: tekst bez wpisu w `.tsv` pokazuje się po angielsku. Liczba form mnogich: polski 3, inne języki 2. Zmienne `%s`, `%d`, `%1$s` muszą się zgadzać w każdym języku (skrypt to sprawdza).
10. **Daty**: w pętlach sortowania `date::sort_key` parsuje tekst przy każdym porównaniu. Używaj `sort_by_cached_key`.
11. **Strona rodzinna i prywatność**: nowe dane osoby (np. cytowania) muszą przejść przez `site.rs` (publiczna kopia) i `privacy.rs`, inaczej mogą trafić na stronę mimo ustawień.
12. **GEDCOM**: tagi spoza standardu z podkreślnikiem (`_CLAN`) są bezpieczne dla innych programów; nowe pola zapisuj jako takie rozszerzenia albo zgodnie ze standardem (zobacz `docs/GEDCOM.md`).
13. **Autor**: commity i pliki bez dopisków o współautorach i narzędziach (zasada z `zasady.md`).
14. **Folder roboczy na Macu**: usuwanie plików przez asystenta wymaga zgody; tymczasowe pliki trafiają do `~/Downloads/myroots/_to_delete/`.
15. **Mapy i MapLibre 6**: kafelki z tile.openstreetmap.org odpowiadają 403 bez nagłówka Referer (aplikacja Tauri go nie wysyła), dlatego podkład idzie z OpenFreeMap. MapLibre 6 szuka swojego skryptu roboczego obok siebie (`import.meta.url`), co nie działa w Vite ani w szablonie strony rodzinnej; `src/lib/maplibre.ts` ustawia go przez `setWorkerUrl` z adresu `blob:`. Polityka CSP musi więc mieć `worker-src blob:` (aplikacja w przeglądarce i Tauri) oraz `data:` (strona rodzinna otwierana z dysku), a Tauri także `connect-src https://tiles.openfreemap.org`.
16. **Sesja PHP w MyRoots Server**: PHP pozwala tylko jednemu zapytaniu danej osoby korzystać z sesji naraz. Każda odpowiedź z plikiem albo JSON-em musi przed wysłaniem zwolnić sesję (`mr_release_session()` w `mr_json`, `MR_Store::send_file` i `data.js`), a publiczna strona nie może jej w ogóle otwierać (`MR_Site::visible()` sprawdza najpierw widoczność). Inaczej zdjęcia przychodzą jedno po drugim. Po `mr_release_session()` nic już nie zapisuj do `$_SESSION`.
17. **Wysyłanie plików na serwer** (`WpBackend.upload`): kawałki są ponawiane przy błędzie sieci i 5xx, więc serwer przyjmuje powtórzony kawałek (plik `.part` dłuższy niż `offset` jest przycinany). Publikacja i przywracanie idą po 4 pliki naraz (`pool`), a `busy` włącza ostrzeżenie przed zamknięciem karty.
