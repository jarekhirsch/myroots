# Dokumentacja robocza MyRoots

Dla osoby (albo asystenta), która zmienia kod. Opisuje stan na wersję 3.1.0. Każda zmiana w kodzie aktualizuje te pliki w tym samym commicie (zasady w [zasady.md](zasady.md)).

| Plik | Co zawiera |
|---|---|
| [zasady.md](zasady.md) | Zasady pracy: dokumentacja z kodem, autor, lista zmian, tłumaczenia, wydajność, testy, komendy. |
| [architektura.md](architektura.md) | Warstwy, platformy, przepływ danych, zapis danych na każdej platformie, powiązane foldery (serwer, wtyczka, strona). |
| [silnik.md](silnik.md) | Silnik w Rust: moduły, indeks relacji, prywatność, lista wszystkich metod `Engine::call` (generowana), jak dodać metodę. |
| [model.md](model.md) | Rekordy (osoba, związek, źródło, cytowanie, zdjęcie, miejsce…), ustawienia, zgodność wstecz. |
| [interfejs.md](interfejs.md) | Svelte 5: widoki, komponenty, stan, ładowanie danych, wyszukiwanie, tłumaczenia. |
| [budowanie.md](budowanie.md) | Budowanie, testy, wydania (GitHub Actions), podpisy, aktualizacje, strona myroots.cc. |
| [zaleznosci.md](zaleznosci.md) | Biblioteki npm i Rust z wersjami (generowane) i po co są. |
| [pliki.md](pliki.md) | Wszystkie pliki źródłowe z opisem (generowane z pierwszego komentarza w pliku). |
| [wydajnosc.md](wydajnosc.md) | Pomiary na drzewie 20 000 osób, narzędzia do pomiarów, zasady szybkiego kodu. |
| [pulapki.md](pulapki.md) | Rzeczy, które łatwo zepsuć. Przeczytaj przed większą zmianą. |
| [../GEDCOM.md](../GEDCOM.md) | Zakres obsługi standardu GEDCOM (import i eksport). |

Części między `<!-- auto:… -->` a `<!-- /auto:… -->` są generowane przez `npm run docs`. Nie edytuj ich ręcznie.
