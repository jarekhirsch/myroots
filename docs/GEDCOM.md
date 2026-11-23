# MyRoots a standard GEDCOM

Stan na wersję 3.1.0. Kod: `crates/myroots-core/src/gedcom.rs` (import i eksport), `date.rs` (daty), `charsets.rs` (kodowanie znaków). Testy: `cargo test -p myroots-core gedcom`.

## W skrócie

- **Import:** GEDCOM 5.5.1 i 7.0 (starsze 5.5 i nowsze 5.5.5 też się wczytują), także spakowane `.gdz` i `.zip` z plikiem `.ged`. Testy obejmują pliki MyHeritage, GEDCOM 7 i wtyczki MyRoots 2.x oraz uszkodzone i losowe pliki.
- **Eksport:** GEDCOM 5.5.1 w UTF-8 (`1 GEDC / 2 VERS 5.5.1 / 2 FORM LINEAGE-LINKED`), plus rozszerzenia MyRoots z podkreślnikiem, które inne programy pomijają bez błędu.
- **Czego MyRoots nie przenosi:** dat zmian (CHAN), ograniczeń prywatności (RESN), adresów kontaktowych (EMAIL, PHON, WWW przy osobie) i drugiego i kolejnego zdarzenia tego samego typu dla urodzenia i śmierci. Identyfikatory z innych programów (REFN, RIN, _UID i in.) są zapamiętywane, żeby ponowny import nie dublował osób, ale nie trafiają do eksportu. Szczegóły w tabeli.

Ocena: „tak” to pełna obsługa, „część.” to obsługa z uproszczeniem opisanym w uwagach, „nie” to brak.

## Plik i kodowanie

| Element | Import | Eksport | Uwagi |
|---|---|---|---|
| `HEAD`, `TRLR` | tak | tak | Z nagłówka czytany jest program źródłowy (`SOUR`), żeby rozpoznać pliki MyRoots. |
| `CHAR` UTF-8, UTF-16, BOM | tak | tak | Eksport zawsze w UTF-8. |
| ANSEL, Windows-1250/1252, ISO-8859-1/2, Mac Roman | tak | nie | Rozpoznawane automatycznie, także bez poprawnego `CHAR`. |
| `CONC`, `CONT` | tak | tak | Długie wiersze dzielone po 200 znaków. |
| `@@` w treści | tak | tak | |
| GEDCOM 7: `SNOTE`, kalendarz bez `@#D…@`, `SEX X` | tak | nie | Eksport jest w 5.5.1. |
| GEDZIP (`.gdz`), `.zip` z `.ged` | tak | nie | Zdjęcia z archiwum nie są importowane. |
| `SUBM` | pomijany | tak | Eksport podaje autora drzewa. |

## Osoba (`INDI`)

| Element | Import | Eksport | Uwagi |
|---|---|---|---|
| `NAME` z `/nazwisko/`, `GIVN`, `SURN`, `SPFX` | tak | tak | Przedrostki („van der”) trafiają do nazwiska. |
| `NAME` `TYPE birth` / `married`, `_MARNM` | tak | tak | Nazwisko rodowe i po mężu. Eksport: najpierw nazwisko rodowe, potem `TYPE married`. |
| `NICK`, `ALIA` (tekst), `NAME TYPE aka` | tak | tak | Pole „Przydomek” (kilka oddzielonych przecinkiem). Eksport: `2 NICK` pod pierwszym `NAME`. `ALIA` ze wskaźnikiem na inną osobę jest pomijany. |
| `NPFX`, `NSFX` | część. | nie | `NPFX` tylko w plikach wtyczki MyRoots (nazwisko rodowe). Tytuły można zapisać w zdarzeniu „Tytuł”. |
| `SEX` M, F, U, X | tak | tak | X i U jako „nieznana”. |
| `BIRT` (data, miejsce, źródła) | tak | tak | Tylko pierwsze urodzenie. |
| `DEAT` (data, miejsce, `CAUS`, źródła), `DEAT Y` | tak | tak | „Nie żyje, data nieznana” zapisuje się jako `1 DEAT Y`. |
| `BURI` | tak | tak | Pochówek to grób: cmentarz z `PLAC`, położenie z `MAP`, kwatera jako `NOTE`. |
| `CREM` | tak | tak | Jako wydarzenie „Kremacja”. |
| `OCCU`, `EDUC`, `RELI` | tak | tak | Kilka `OCCU` łączy się w jedno pole. |
| `TITL`, `NATI`, `DSCR`, `PROP` | tak | tak | Jako wydarzenia z treścią; eksport przywraca znacznik. |
| Zdarzenia: `BAPM`, `CHR`, `CHRA`, `CONF`, `FCOM`, `BLES`, `ADOP`, `BARM`, `BASM`, `GRAD`, `RETI`, `EMIG`, `IMMI`, `NATU`, `CENS`, `RESI`, `ORDN`, `PROB`, `WILL`, `_MILT`, `_MILI` | tak | tak | Nazwy przetłumaczone na język aplikacji; przy eksporcie nazwa wraca do znacznika. |
| `EVEN` / `FACT` z `TYPE` | tak | tak | Własne zdarzenia (np. „Służba wojskowa”). |
| `NOTE` (w tekście i wspólne `@N1@`) | tak | tak | Biografia; notatki MyRoots mają `_TYPE notes`, niepotwierdzone informacje `_TYPE research` (inne programy widzą je jako zwykłe notatki). |
| `RESI` bez daty, z `ADDR`/`PLAC` | tak | tak | Pole „Adres” osoby (pierwszy taki `RESI`); `RESI` z datą zostaje zdarzeniem „Zamieszkanie”. Cytowanie jako fakt `residence`. |
| `FAMC` z `PEDI` (adopted, foster, birth) | tak | tak | Relacja rodzica: biologiczna, adopcyjna, zastępcza, przybrana. |
| `FAMS` | tak | tak | |
| `OBJE` (zdjęcia) przy osobie, w tekście i jako rekord `0 @O1@ OBJE` | tak | część. | Import: zdjęcia z archiwum `.gdz`/`.zip` (razem z plikiem `.ged`) albo z folderu wskazanego po imporcie; dopasowanie po ścieżce, końcówce ścieżki albo nazwie pliku (ścieżki Windows i `file://` też). `_PRIM Y` (także `_PRIMARY`, `_PHOTO`) = zdjęcie profilowe, `TITL` = podpis, `DATE` = data zdjęcia. Ponowny import nie dubluje zdjęć. Pliki inne niż zdjęcia (PDF, dźwięk) są pomijane i wymienione jako „nie znaleziono”. Eksport tylko z opcją „ze zdjęciami” w kopii ZIP: `media/<id>.<ext>`. |
| `ASSO` (chrzestni, świadkowie) przy osobie, przy zdarzeniu (`BAPM.ASSO`) i przy rodzinie (`FAM.ASSO`, `MARR.ASSO`) | tak | tak | Rola z `RELA` (5.5.1, wolny tekst; rozpoznawane m.in. Godfather, Witness, Chrzestny, Świadek w kilku językach) albo `ROLE` (7.0: `GODP`, `WITN`, `CLERGY`, `OFFICIATOR`, `FRIEND`, `NGHBR`, `OTHER` + `PHRASE`). Zdarzenie zapisuje się przy powiązaniu (np. „Chrzest”); świadkowie ślubu trafiają do obojga małżonków. Eksport: `1 ASSO @I…@`, `2 RELA Godparent`, `2 _EVENT Chrzest`. |
| `ADDR`, `EMAIL`, `PHON`, `WWW` | część. | nie | `ADDR` służy tylko jako miejsce, gdy zdarzenie nie ma `PLAC`. |
| `RESN` | nie | nie | Prywatność ustawia się w aplikacji (żyjące osoby, sekcje strony). |
| `REFN`, `RIN`, `_UID`, `UID`, `EXID`, `AFN`, `_FSFTID` | tak (zapamiętane) | nie | Zapisywane w `Person.ext_ids` jako „ZNACZNIK:wartość” (do 20). Ponowny import pliku z tego samego programu znajduje osobę po nich; bez nich po imieniu i nazwisku (także rodowym) oraz zgodnych datach (±1 rok; różne dokładne dni wykluczają). Istniejąca osoba jest uzupełniana, a dokładniejsze daty zostają. |
| `CHAN` | nie | nie | Data zmiany jest w MyRoots w historii zmian osoby. |
| `BAPL`, `CONL`, `INIL`, `ENDL`, `SLGC`, `SLGS` (obrzędy LDS) | tak | tak | Zdarzenia „Chrzest (LDS)” itd. z datą, miejscem i świątynią (`TEMP`) i stanem (`STAT`) w opisie. `SLGS` (rodzina) przy obojgu małżonkach. W eksporcie `TEMP` i `STAT` wracają jako notatka zdarzenia. |
| Rozszerzenia MyRoots: `_CLAN`, `_ATTR` (wzrost, oczy, krew), `_PET` | tak | tak | |

## Rodzina (`FAM`)

| Element | Import | Eksport | Uwagi |
|---|---|---|---|
| `HUSB`, `WIFE`, `CHIL` | tak | tak | Pary osób tej samej płci z pliku 5.5.1 (dwa `HUSB` lub dwa `WIFE`) też. |
| `MARR` (data, miejsce, źródła) | tak | tak | |
| `DIV`, `ANUL` | tak | tak | Jako koniec związku „rozwód”; eksport `DIV`. |
| `ENGA`, `MARB`, `MARC`, `MARL`, `MARS`, `DIVF`, `EVEN` | tak | część. | Trafiają jako wydarzenia do obojga partnerów; eksport przy osobach, nie przy rodzinie. |
| `_UST PARTNERS` (związek nieformalny) | tak | tak | |
| `CHIL` z `_FREL` / `_MREL` | tak | tak | |
| `NCHI`, `RESI` rodziny | nie | nie | |

## Źródła i cytowania

| Element | Import | Eksport | Uwagi |
|---|---|---|---|
| Rekord `SOUR` z `TITL` (albo `ABBR`), `AUTH`, `PUBL`, `NOTE` | tak | tak | `AUTH` i `PUBL` trafiają do notatki źródła. |
| `SOUR` `_LINK`, `WWW` | tak | tak | Adres źródła. |
| `SOUR.DATA.EVEN.DATE` | tak | tak | Data źródła. |
| `SOUR.REPO`, rekord `REPO`, `CALN` | tak | tak | Pola źródła „Archiwum, parafia lub biblioteka” (nazwa `REPO` z miastem z `ADDR.CITY`) i „Sygnatura” (`CALN`). Eksport: jeden rekord `0 @R…@ REPO` na archiwum. |
| `SOUR.OBJE` (skany) | tak | nie | Import jak zdjęcia osób (skany źródła). W eksporcie skany zostają w kopii `.myroots`. |
| Źródło wpisane w treści (`1 SOUR Księga parafii…`) | tak | nie | Zamieniane na rekord źródła; eksport zawsze przez `@S…@`. |
| Cytowanie przy imieniu (`NAME.SOUR`) | tak | tak | |
| Cytowanie przy urodzeniu, śmierci (z przyczyną), pochówku | tak | tak | W aplikacji to jedno źródło dla daty i miejsca, tak jak w GEDCOM. |
| Cytowanie przy `OCCU`, `EDUC`, `RELI` | tak | tak | |
| Cytowanie przy wydarzeniach (`EVEN`, `RESI`, `BAPM` …) | tak | tak | |
| Cytowanie przy małżeństwie (`MARR.SOUR`) i rodzinie (`FAM.SOUR`) | tak | tak | |
| `PAGE` (strona, numer aktu) | tak | tak | |
| `QUAY` 0–3 (wiarygodność) | tak | tak | |
| `DATA.TEXT`, `NOTE` cytowania | tak | część. | Obie do jednego pola „treść zapisu albo notatka”; eksport jako `NOTE`. |
| `DATA.DATE` cytowania, `EVEN`/`ROLE` cytowania | nie | nie | |
| Ogólne źródła osoby (`1 SOUR` bez faktu) | tak | tak | |

## Miejsca i daty

| Element | Import | Eksport | Uwagi |
|---|---|---|---|
| `PLAC` | tak | tak | Pełna nazwa w jednym polu; `FORM` (hierarchia) nie jest używana. |
| `MAP` z `LATI`, `LONG` | tak | tak | Współrzędne trafiają do listy miejsc (mapa). |
| `FONE`, `ROMN` (fonetyka, transliteracja) | nie | nie | |
| Daty dokładne, miesiąc i rok, sam rok | tak | tak | |
| `ABT`, `CAL`, `EST`, `INT`, `BEF`, `AFT`, `BET … AND …` | tak | tak | `CAL`, `EST` i `INT` jako „około” (eksport `ABT`). |
| `FROM … TO …` | część. | część. | Zapisywany jako przedział dat (jak `BET … AND …`); eksport `BET … AND …`. |
| Frazy dat `(…)` | część. | nie | Z frazy brana jest data, jeśli jest. |
| Kalendarz juliański (`@#DJULIAN@`, `JULIAN`) | tak | nie | Pełna data przeliczana na gregoriańską (np. 5 X 1582 → 15 X 1582); podwójny rok (`1699/00`) = późniejszy rok. Sam miesiąc albo rok zostaje bez zmian. |
| Kalendarz hebrajski (`@#DHEBREW@`, `HEBREW`), francuski republikański (`@#DFRENCH R@`, `FRENCH_R`) | tak | nie | Przeliczane na gregoriański (dzień dokładnie, miesiąc i rok w przybliżeniu: rok, w którym się zaczyna). Eksport zawsze gregoriański. |
| Daty p.n.e. | nie | nie | Pole zostaje puste, a reszta osoby się importuje. |

## Jak sprawdzić plik przed importem

Import zawsze zaczyna się od podglądu (tryb „na próbę”): liczba osób, rodzin, źródeł i lista ostrzeżeń (osoby bez imienia, pętle przodków, dzieci przypisane gdzie indziej). Dane zapisują się dopiero po kliknięciu „Importuj”. Osoby, które już są w drzewie, można zaktualizować, pominąć albo dodać jako nowe.

## Co warto dodać w kolejnych wersjach

1. Eksport do GEDCOM 7.0 obok 5.5.1.
2. `RESN` przy imporcie jako status „prywatny”.
3. Eksport zdjęć i skanów w `.gdz` (GEDCOM 7 z plikami) zamiast samego ZIP.
