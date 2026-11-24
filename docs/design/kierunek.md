# Kierunek wyglądu: „lżej, więcej zdjęć, jedna mocna akcja”

Status: **propozycja do akceptacji**. Prototyp ekranów (teraz obok propozycji): kanwa „MyRoots – kierunek wyglądu (prototyp)”. Nic z tego nie jest jeszcze w aplikacji.

Punkt wyjścia: zarzut, że aplikacja wygląda „AI-slopowo”, bo wszystko wygląda tak samo. Dziś każdy ekran ma ten sam rytm: szarozielone tło, białe panele z tym samym promieniem i cieniem, ciemnozielone pigułki, ikony Lucide. Nic nie wybija się na pierwszy plan. Referencja (aplikacja do kompensacji CO₂, listopad 2026) pokazuje ten sam charakter, ale z większą ilością bieli, zdjęciami w roli głównej i szklanymi etykietami.

Zasada nadrzędna: **nie zmieniamy tego, co działa** (układ, nawigacja, czcionka Onest, zieleń MyRoots, kolejność treści). Zmieniamy rytm, hierarchię i kilka elementów.

## 1. Co zostaje

- Czcionka **Onest** (400/500/600/700).
- Zieleń MyRoots jako jedyny akcent (domyślnie `#1F4A37`).
- Układ ekranów, kolejność sekcji, nazwy, nawigacja (dock na komputerze, pasek na telefonie).
- Ciemny motyw i motywy kolorystyczne z Ustawień (zmieniają tylko wartości tokenów poniżej).

## 2. Co się zmienia

| Teraz | Propozycja |
|---|---|
| Szarozielone tło (`--bg`), wszystko w białych panelach | **Białe tło**. Panele tylko tam, gdzie grupują treść; zamiast ramek podział **odstępami** i cienką linią `--line`. |
| Wiele ciemnozielonych pigułek naraz (Edytuj, Dodaj osobę…) | **Jedna mocna akcja na ekran**: okrągły przycisk 52–56 px w kolorze akcentu ze strzałką lub plusem. Reszta: przyciski konturowe (biel + `--line`). |
| Zdjęcie jako mały kwadrat obok tekstu | **Zdjęcie prowadzi kartę** (osoba, wydarzenie, miejsce): duże, zaokrąglone 20–28 px, tekst pod spodem. |
| Etykiety jako szare pigułki | Na zdjęciach **szklane etykiety** (miejsce, rok, relacja): `rgba(255,255,255,.22)`, ramka `rgba(255,255,255,.5)`, `backdrop-filter: blur(14px) saturate(1.4)`, biały tekst. Bez zdjęcia: zwykła pigułka `--soft`. |
| Ikony Lucide (najczęstszy zestaw w aplikacjach generowanych przez AI) | **Phosphor, odmiana „light”** (MIT, `@phosphor-icons/core`). Cieńsza kreska, bardziej rysunkowe kształty. Ikona w kolorze tekstu albo akcentu, nigdy w kolorowym kwadracie. |
| Półprzezroczysty pasek na telefonie nachodzi na treść | **Pływający pasek** (pigułka w kolorze akcentu, 64 px, 22 px od dołu, cień), aktywna pozycja jako białe kółko z ikoną w kolorze akcentu. Treść kończy się nad nim (`padding-bottom` ≥ 110 px). |
| Bardzo duże nagłówki (np. tytuł drzewa ~48 px) | Spokojniej: H1 26–30 px na telefonie, 40–44 px na komputerze, `font-weight: 600`, `letter-spacing: -0.02em`. Nad tytułem mała linia kontekstu (np. „48 osób · 69 zdjęć”). |
| Zakładki jako pigułki w szarym tle | Zakładki jako **tekst z podkreśleniem** w kolorze akcentu (2 px). |

## 3. Tokeny (jasny motyw)

| Token | Wartość | Do czego |
|---|---|---|
| `--bg` | `#FFFFFF` | tło aplikacji |
| `--soft` | `#F4F6F2` | tło przycisków ikonowych, kółek przy wydarzeniach, pigułek bez zdjęcia |
| `--line` | `#E6E9E4` | linie podziału, kontury przycisków |
| `--ink` | `#121613` | tekst |
| `--muted` | `#5B645D` | tekst drugoplanowy (kontrast 6:1 na bieli) |
| `--accent` | `#1F4A37` | jedna mocna akcja, pasek nawigacji, aktywna zakładka, ikony wydarzeń |
| `--accent-soft` | `#E9F0EC` | aktywna pozycja menu na komputerze, odznaki „za 3 dni” |
| promienie | 999 px (przyciski, etykiety), 20–22 px (karty), 26–28 px (karty główne, zdjęcie profilu) |
| odstępy | sekcje 26–36 px, w sekcji 12–16 px, siatka kart 14–18 px |

Ciemny motyw: te same role, wartości z obecnego ciemnego motywu; szklane etykiety bez zmian (leżą na zdjęciach).

## 4. Elementy

- **Karta osoby (album):** zdjęcie 4:5, promień 22 px, szklana etykieta z miejscem urodzenia w lewym dolnym rogu, pod spodem imię i nazwisko (15 px, 600) i lata (13 px, `--muted`). Bez ramki i cienia.
- **Profil osoby (telefon):** zdjęcie na całą szerokość (ok. 470 px), na nim szklany przycisk „Wstecz” i etykiety; biała karta z treścią nachodzi na zdjęcie o 28 px (promień 28 px u góry). Obok nazwiska okrągła akcja „Otwórz w drzewie”.
- **Profil osoby (komputer):** zdjęcie 380×470 po lewej, po prawej kontekst, imię, lata, przyciski (jeden wypełniony „Edytuj”, reszta konturowe) i dane w dwóch kolumnach bez panelu.
- **Wiersz krewnego:** awatar 44 px w kółku, imię (600), relacja i lata (`--muted`), strzałka w prawo; linie podziału zamiast kart.
- **Wydarzenie:** ikona 20 px w kółku 40 px `--soft` w kolorze akcentu, tytuł i data/miejsce.
- **Karta „Dziś” na starcie:** duże zdjęcie (300 px) ze szklanym panelem na dole (biel 72%, rozmycie) i okrągłą strzałką w kolorze akcentu.

## 5. Czego unikać

- Kilku wypełnionych przycisków obok siebie (tylko jeden wypełniony na ekran albo sekcję).
- Ikon w kolorowych kwadratach, gradientów, kart z kolorowym paskiem z boku, emoji.
- Ramek i cieni wokół wszystkiego: grupy tworzy odstęp.
- Szkła na jednolitym tle (bez zdjęcia pod spodem rozmycie nic nie daje i obniża kontrast).
- Półpauz w tekstach interfejsu.

## 6. Dostępność

- Tekst na szkle: biały na zdjęciu, z rozmyciem i przyciemnieniem w ramce; gdy zdjęcie jest bardzo jasne, etykieta przechodzi w ciemną wersję (`rgba(18,22,19,.62)`). Tak samo działa przełącznik „szklane etykiety” w prototypie.
- Cele dotyku co najmniej 44 px (przyciski ikonowe 44–48 px, pozycje paska 48 px).
- `--muted` na bieli spełnia 4,5:1; akcent na bieli i biel na akcencie także.

## 7. Wdrożenie (po akceptacji)

1. Tokeny w `src/lib/theme.ts` i `src/app.css` (tło, `--soft`, `--line`, `--accent-soft`), bez zmian w układzie. Sprawdzenie wszystkich ekranów testem `tests/e2e/smoke.spec.ts` i zrzutami.
2. Zamiana ikon: jeden plik mapujący nazwy (`src/lib/icons.ts`) z Lucide na Phosphor light, potem podmiana importów (111 różnych ikon, 199 importów w 51 plikach; mechanicznie).
3. Przyciski: klasa `.btn.primary` zostaje tylko dla jednej akcji na ekran; pozostałe `.btn` konturowe; nowa klasa `.btn.round` (okrągła akcja).
4. Komponent `GlassTag.svelte` i karty prowadzone zdjęciem: album osób, profil, start, galeria.
5. Pływający pasek nawigacji na telefonie.
6. Strona rodzinna (ten sam kod) dostaje to samo automatycznie.
