# Zasady pracy nad kodem MyRoots

Program do drzewa genealogicznego. Autor: Jarek Hirsch (https://hirsch.lt). Licencja GPL-3.0-or-later z warunkami z `NOTICE.md`.

## Zanim zaczniesz

Przeczytaj `README.md` w tym folderze. Tam jest mapa dokumentacji: architektura, silnik i jego metody, model danych, interfejs, budowanie i wydania, zależności, wydajność, znane pułapki. Zakres GEDCOM: `docs/GEDCOM.md`.

## Zasady

1. **Dokumentacja razem z kodem.** Każda zmiana w kodzie, która zmienia działanie, pliki, metody silnika, model danych, ustawienia, zależności albo sposób budowania, aktualizuje odpowiedni plik w `docs/dev/` w tym samym commicie. Części oznaczone `<!-- auto:… -->` generuje `npm run docs`; test `npm run docs:check` (GitHub Actions) nie przepuści nieaktualnych.
2. **Autor.** Jedynym autorem jest Jarek Hirsch. W commitach, plikach, opisach zmian i pull requestach nie dopisuj współautorów ani informacji o narzędziach, którymi powstał kod (żadnych linii `Co-Authored-By`, „Generated with”).
3. **Lista zmian.** Każda zmiana widoczna dla użytkownika trafia do `CHANGELOG.md` (po polsku) i `CHANGELOG.en.md` (po angielsku), w sekcji najbliższej wersji `## X.Y.Z (RRRR-MM-DD)`. Z tych plików powstaje strona „Co nowego” na myroots.cc i opis aktualizacji w aplikacji.
4. **Teksty.** Nowe teksty interfejsu piszesz po angielsku w kodzie (`t('…')`) i tłumaczysz na 11 języków w `src/lib/i18n/app-strings/*.tsv`, potem `npm run i18n` i `node scripts/check-i18n.mjs pl` (ma pokazać 0 brakujących). W tekstach dla ludzi nie używaj pauzy (—).
5. **Duże drzewa.** Program ma działać płynnie przy kilkudziesięciu tysiącach osób. Przed zmianą list, wyszukiwania albo zapytań silnika przeczytaj `docs/dev/wydajnosc.md`.
6. **Testy przed oddaniem:** `cargo test -p myroots-core -p myroots-store`, `npm run check`, `npm run build`.

## Najczęstsze komendy

```
npm install && npm run dev          # wersja w przeglądarce, http://localhost:1420
npm run tauri dev                   # aplikacja na komputer
npm run wasm                        # po zmianach w crates/ (silnik dla przeglądarki)
npm run i18n                        # po zmianach w tłumaczeniach
npm run docs                        # po zmianach w kodzie: części generowane docs/dev
cargo run --release -p myroots-core --example bench -- duze.ged   # pomiar silnika
```
