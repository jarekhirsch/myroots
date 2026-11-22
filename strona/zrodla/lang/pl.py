# Polish (default language, at the site root).
# Placeholders: {R} site root, {L} root of this language, {S_tpl} {S_guide} {S_wp} {S_priv} {S_sign} page links,
# {DEMO} {SUPPORT} {AUTHOR_URL} {GITHUB} {GITHUB_USER}, {FIG:image-name|alt|caption} a screenshot.
L = {
    'code': 'pl', 'name': 'Polski', 'locale': 'pl_PL', 'dir': 'ltr', 'prefix': '',
    'img': 'img/', 'tpl_pv': 'img/szablon-{f}.webp',
    'slug': {'home': '', 'tpl': 'szablony/', 'guide': 'jak-zrobic-drzewo-genealogiczne/', 'wp': 'wordpress/', 'priv': 'prywatnosc/', 'sign': 'podpisywanie-kodu/'},
    'nav': {'home': 'Program', 'tpl': 'Szablony', 'guide': 'Poradnik', 'wp': 'WordPress', 'support': 'Wesprzyj projekt'},
    'ui': {'skip': 'Przejdź do treści', 'menu': 'Menu', 'logo': 'MyRoots, strona główna', 'cta': 'Pobierz za darmo', 'lang': 'Język', 'crumbs': 'Okruszki', 'full': 'Pełny rozmiar', 'views': 'Widoki'},
    'foot': {
        'app': 'MyRoots', 'dl': 'Pobierz program', 'demo': 'Demo w przeglądarce', 'wp': 'Wtyczka WordPress', 'cmp': 'MyRoots, MyHeritage i Gramps',
        'tpl': 'Szablony', 'tpl_links': [('do-druku', 'Wzór do wydruku (PDF)'), ('do-wypelnienia', 'Do wypełnienia na komputerze'), ('excel', 'Szablon w Excelu'), ('ze-zdjeciami', 'Ze zdjęciami')],
        'guide': 'Poradnik', 'guide_links': [('', 'Jak zrobić drzewo genealogiczne'), ('jak-sprawdzic', 'Jak sprawdzić przodków'), ('pomysly', 'Pomysły i projekt szkolny')],
        'author': 'Autor', 'who': 'Kto robi MyRoots', 'support': 'Wesprzyj rozwój projektu', 'license': 'Licencja GPL', 'priv': 'Prywatność', 'sign': 'Code signing policy',
    },
    # ------------------------------------------------------------------ home
    'home': {
        'title': 'Drzewo genealogiczne online za darmo: program MyRoots',
        'desc': 'Darmowy program do drzewa genealogicznego rodziny, bez limitu osób: animacja zdjęć „Przez lata”, księga rodu, zwierzęta, GEDCOM, plakat do druku. Mac, Windows, Linux, Android, WordPress i własny serwer.',
        'h1': 'Drzewo genealogiczne online za darmo',
        'lead': 'MyRoots to darmowy program do drzewa genealogicznego rodziny. Wpisujesz osoby, dodajesz zdjęcia i daty, a program rysuje drzewo, oś czasu i mapę. Działa na Macu, Windowsie, Linuksie, Androidzie i na Twojej stronie internetowej.',
        'demo': 'Zobacz demo',
        'facts': 'Nie wymaga konta. Wolisz kartkę? <a href="{S_tpl}">Szablony do druku</a>.',
        'hero_alt': 'Drzewo genealogiczne w MyRoots: pięć pokoleń rodziny Zawadzkich ze zdjęciami osób i zwierząt oraz panel z danymi wybranej osoby',
        'hero_cap': 'MyRoots 3.0, widok „Wokół osoby” z panelem danych. Małe kółka przy kartach to zwierzęta.',
        'pitch_label': 'Dla kogo jest MyRoots',
        'free_h': 'Cały program za 0&nbsp;zł, bez limitu osób',
        'free_p': 'W darmowym planie MyHeritage drzewo może mieć najwyżej 250 osób i 500&nbsp;MB zdjęć. MyRoots nie ma limitów ani płatnej wersji. Animacje, księga rodu, plakat i strona rodzinna działają od pierwszego uruchomienia.',
        'plat_h': 'Jedno drzewo na każdym urządzeniu',
        'plat_p': 'Drzewo zaczęte na laptopie otworzysz w telefonie, a potem opublikujesz na stronie dla rodziny. Każda wersja zapisuje ten sam plik kopii .myroots, razem ze zdjęciami i nagraniami.',
        'plat_list': ['Mac', 'Windows', 'Linux', 'Android', 'WordPress', 'własny serwer z PHP', 'przeglądarka'],
        'show_h': 'Jak wygląda drzewo w MyRoots',
        'show_sub': 'Projektuję MyRoots tak, żeby drzewo dało się pokazać przy rodzinnym stole i powiesić na ścianie. Każdy widok ma motyw jasny i ciemny, a po hebrajsku układ czyta się od prawej do lewej.',
        'show': [
            ('Oś czasu', 'Pokolenia rodziny na jednej osi czasu, z rocznicami obok.'),
            ('Profil osoby', 'Biografia, animacja „Przez lata”, dane i grób z dojazdem.'),
            ('Album rodzinny', 'Wszystkie zdjęcia rodziny, z filtrem po rodach.'),
            ('Zwierzęta', 'Psy, koty, konie i kanarki rodziny, każde z opiekunem.'),
            ('Plakat', 'Generator plakatu: wachlarz przodków do druku.'),
            ('Przez lata', 'Cztery zdjęcia z lat 1929 do 1998 złożone w krótki film.'),
            ('Na telefonie', 'Profil osoby w aplikacji na Androida.'),
        ],
        'feats_label': 'Najważniejsze funkcje',
        'feats': [
            ('Animacja Przez lata w MyRoots: cztery zdjęcia Tadeusza Zawadzkiego z lat 1929, 1951, 1973 i 1998 z wyrównanymi twarzami',
             'Zdjęcia z całego życia w jednym filmie',
             'Animacja „Przez lata” bierze zdjęcia jednej osoby z różnych lat, wyrównuje je według oczu i płynnie przechodzi od dziecka do starszego człowieka. W rogu widać rok i wiek. Jeśli masz nagranie głosu tej osoby, zagra razem ze zdjęciami, a całość zapiszesz jako wideo dla rodziny.',
             'Animacja składa się wyłącznie z Twoich zdjęć. Nic nie jest dorabiane przez AI, a twarze rozpoznaje Twoje urządzenie.'),
            ('Księga rodu z MyRoots: okładka Rodzina Zawadzkich z dedykacją i strona Józefa Zawadzkiego z danymi, biografią i linią życia',
             'Księga rodu do wydrukowania',
             'Z drzewa składa się książka. Ma okładkę z tytułem i dedykacją, spis treści i rozdziały według pokoleń, gałęzi albo rodów. Każda osoba dostaje własną stronę ze zdjęciem, danymi, biografią i linią życia, a na końcu jest indeks nazwisk.',
             'Zapisujesz ją jako PDF i drukujesz w domu albo oprawiasz w drukarni. Sprawdza się jako prezent na jubileusz dziadków.'),
            ('Zwierzęta rodziny Zawadzkich w MyRoots: pies Burek, kot Filemon, kanarek Kajtek i koń Kasztan, każde z datami i opiekunem',
             'Pies dziadka też należy do rodziny',
             'Zwierzęta mają w MyRoots własne profile z imieniem, gatunkiem, datami, zdjęciami i opiekunami. Na drzewie pojawiają się przy swoich ludziach. Burek z lat trzydziestych i kot z Twojego dzieciństwa trafią do tej samej historii.',
             'W programach genealogicznych, które sprawdzałem, zwierzę dało się wpisać tylko jako osobę albo notatkę.'),
            ('Oznaczanie osób na zdjęciu grupowym w MyRoots: twarze z podpisami i podpowiedź, kto jest na zdjęciu',
             'Głosy bliskich i podpisane twarze',
             'Nagrasz w aplikacji, jak babcia opowiada o wojnie albo śpiewa kołysankę, albo dodasz gotowy plik MP3, M4A czy WAV. Na zdjęciu grupowym klikasz twarz i wybierasz osobę, a zdjęcie pojawia się w profilu każdej oznaczonej osoby. Gdy kilka twarzy jest już opisanych, MyRoots sam podpowiada, kto jest na kolejnych zdjęciach.',
             'Znasz lata urodzenia osób na zdjęciu? Program oszacuje, kiedy je zrobiono. Filmy z rodzinnych uroczystości podepniesz do osoby jako link.'),
        ],
        'ged_h': 'Przenosisz drzewo z innego programu',
        'ged_p': [
            'Masz drzewo w MyHeritage, Ancestry, FamilySearch albo w starym programie na dysku? Wyeksportuj je jako GEDCOM i otwórz w MyRoots. Program sam rozpoznaje kodowanie znaków, więc polskie litery z plików sprzed lat nie zamieniają się w krzaczki.',
            'W drugą stronę też działa: eksport GEDCOM otworzysz w innym programie genealogicznym, a zwierzęta trafiają do niego jako osobne rekordy.',
        ],
        'spec': [
            ('Import', 'GEDCOM 5.5.1 i 7.0, także spakowany jako GEDZIP (.gdz) lub .zip'),
            ('Kodowanie', 'UTF-8, UTF-16, ANSEL, Windows-1250 i 1252, ISO-8859-2, Mac Roman'),
            ('Sprawdzony na plikach z', 'MyHeritage, Legacy, PAF i przykładach GEDCOM 7'),
            ('Eksport', 'GEDCOM 5.5.1 w UTF-8, zwierzęta jako rekordy _PET'),
        ],
        'more_h': 'Do codziennej pracy nad drzewem',
        'more': [
            ('Niepewne daty', '„ok. 1890”, „przed 1850”, „między 1900 a 1905” albo sam rok. Program liczy z tego wiek i rocznice, a kontrola danych zgłasza na przykład śmierć przed urodzeniem albo rodzica młodszego od dziecka.'),
            ('Plakat do 4A0', 'Wachlarz przodków do 12 pokoleń albo drzewo potomków do 16, jako PDF dla drukarni.'),
            ('Oś czasu', 'Życie kilku pokoleń na jednej osi, a w tle wydarzenia z historii Polski.'),
            ('Mapa i groby', 'Miejsca z życia przodków na mapie. Przy grobie zapiszesz cmentarz, kwaterę i zdjęcie nagrobka, a przed 1 listopada ułożysz trasę odwiedzin.'),
            ('Strona dla rodziny', 'Publikujesz drzewo do oglądania, z ukrytymi danymi żyjących osób. Krewni zgłaszają poprawki przyciskiem przy osobie, a Ty je zatwierdzasz.'),
            ('Hasło do gałęzi', 'Hasło chroni całą stronę albo wybrane osoby z ich przodkami lub potomkami. Chroniona część jest szyfrowana razem ze zdjęciami.'),
            ('Kilka drzew', 'Rodzina taty i rodzina mamy osobno, każda z własnymi zdjęciami, ustawieniami i kopiami.'),
            ('Skany albumów', 'Zeskanuj całą stronę starego albumu, a MyRoots podzieli ją na pojedyncze zdjęcia.'),
            ('Pokrewieństwo', 'Kim jest dla Ciebie kuzyn mamy? Program nazwie pokrewieństwo między dowolnymi dwiema osobami.'),
            ('12 języków', 'Polski, angielski, niemiecki, hebrajski i osiem innych, więc krewni za granicą też poczytają.'),
        ],
        'dl_h': 'Pobierz MyRoots',
        'dl_sub': 'Wszystkie wersje są darmowe i zapisują ten sam plik kopii. Chcesz najpierw zobaczyć? Otwórz <a href="{DEMO}">demo w przeglądarce</a>, nic nie instalujesz.',
        'dl_apps': 'Na komputer i telefon', 'dl_web': 'Na stronę internetową',
        'dl': {
            'mac': ('Mac', 'Komputery z procesorem Apple (M1 i nowsze).', 'Program nie jest podpisany przez Apple, więc przy pierwszym uruchomieniu macOS go zatrzyma. Otwórz wtedy Ustawienia systemowe, Prywatność i ochrona, i kliknij „Otwórz mimo to”.', 'Pobierz na Maca'),
            'win': ('Windows', 'Windows 10 i 11, 64-bit.', 'Instalacja z Microsoft Store: Microsoft podpisuje program, a Sklep sam go aktualizuje. Bez Sklepu: <a href="{DL_WIN}">instalator .exe</a> (Windows może pokazać ostrzeżenie SmartScreen).', 'Pobierz z Microsoft Store'),
            'apk': ('Android', 'Telefony i tablety z Androidem 8 lub nowszym.', 'Telefon zapyta o zgodę na instalację aplikacji z tego źródła. Drzewo, zdjęcia i nagrania zostają w telefonie.', 'Pobierz na Androida'),
            'wp': ('Wtyczka WordPress', 'WordPress 6.5 i PHP 7.4 lub nowsze.', 'W panelu wybierz Wtyczki, Dodaj nową, Wyślij wtyczkę na serwer. Drzewo edytujesz w panelu, a rodzina ogląda je pod Twoim adresem. <a href="{S_wp}">Więcej o wtyczce</a>.', 'Pobierz wtyczkę'),
            'srv': ('MyRoots Server', 'Hosting z PHP 8, bez WordPressa i bez MySQL.', 'Wgraj folder przez FTP i otwórz adres strony: instalator poprowadzi Cię dalej. Krewni dostają własne konta. Instrukcja jest w pliku CZYTAJ-MNIE.md.', 'Pobierz MyRoots Server'),
        },
        'dl_note': 'iPhone i iPad: stronę rodzinną z WordPressa albo MyRoots Server dodasz do ekranu początkowego i otworzysz jak aplikację.',
        'cmp_h': 'MyRoots, MyHeritage i Gramps',
        'cmp_sub': 'MyHeritage ma testy DNA i ogromne zbiory rekordów. Gramps to dojrzały, darmowy program dla zaawansowanych, z rozbudowanymi raportami i wersją na Linuksa. MyRoots stawia na wygląd, zdjęcia i stronę dla rodziny. Stan na listopad 2026.',
        'cmp_head': [('MyRoots', 'wszystkie wersje'), ('MyHeritage', 'plan darmowy i abonamenty'), ('Gramps', 'program na komputer i Gramps Web')],
        # (row, MyRoots, MyHeritage, Gramps); a cell starting with "~" is shown muted (a "no")
        'cmp_rows': [
            ('Koszt', '0 zł, wszystkie funkcje', 'plan darmowy, pełne funkcje w abonamencie', '0 zł, otwarty kod'),
            ('Osoby w drzewie', 'bez limitu', 'do 250 w planie darmowym', 'bez limitu'),
            ('Zdjęcia', 'bez limitu', 'do 500 MB w planie darmowym', 'bez limitu'),
            ('Konto', 'niepotrzebne', 'wymagane', 'niepotrzebne'),
            ('Gdzie są dane', 'na Twoim urządzeniu albo serwerze', 'na serwerach serwisu', 'na Twoim komputerze albo serwerze'),
            ('Urządzenia', 'Mac, Windows, Linux, Android, WordPress, własny serwer', 'przeglądarka, aplikacje na telefon, program na komputer', 'Windows, Mac, Linux, Gramps Web na własnym serwerze'),
            ('Animowane zdjęcia', '„Przez lata” z Twoich zdjęć z różnych lat', 'Deep Nostalgia (AI), kilka za darmo, dalej w abonamencie', '~nie'),
            ('Zwierzęta domowe', 'własne profile i opiekunowie', '~nie', '~nie'),
            ('GEDCOM', 'import 5.5.1 i 7.0, eksport 5.5.1', 'import i eksport', 'import i eksport 5.5.1, import GEDCOM 7 częściowy'),
            ('Strona dla rodziny', 'z hasłem do całości albo gałęzi, z szyfrowaniem', 'drzewo prywatne lub publiczne', 'raport „Narracyjna strona WWW” bez hasła albo Gramps Web z kontami'),
            ('Linux', 'tak', 'w przeglądarce', 'tak'),
            ('Testy DNA, podpowiedzi z cudzych drzew', '~nie', 'tak', '~nie'),
        ],
        'src': 'Źródła: <a href="{SRC_MH_FREE}">Is MyHeritage free?</a> (centrum pomocy MyHeritage), <a href="{SRC_MH_DN}">Techlicious o Deep Nostalgia</a>, <a href="{SRC_GRAMPS}">Gramps</a>, <a href="{SRC_GRAMPS_WEB}">Gramps Web</a>, <a href="{SRC_GRAMPS_G7}">forum Gramps o GEDCOM 7</a>.',
        'auth_h': 'Robi to jedna osoba. Od&nbsp;2023 roku.',
        'auth_p': [
            'Nazywam się Jarek Hirsch. Szukałem narzędzia, w którym historia mojej rodziny wyglądałaby dobrze i należałaby do mnie. Nie znalazłem, więc je napisałem. Każdą linijkę kodu i każde tłumaczenie robię sam, bez zespołu i bez inwestora.',
            'Od pierwszej wersji wydałem ponad 350 aktualizacji i poprawek. MyRoots jest darmowy i taki zostanie. Jeśli Ci się przyda, zachęcam do wsparcia jego rozwoju: na Suppi możesz postawić mi kartonik mleka albo pizzę.',
        ],
        'auth_btn': 'Wesprzyj MyRoots', 'auth_more': 'Więcej o mnie na hirsch.lt',
        'faq_h': 'Najczęstsze pytania',
        'faq': [
            ('Czy MyRoots jest naprawdę za darmo?', 'Tak. Wszystkie wersje są darmowe, na licencji GPL. Nie ma abonamentu, reklam ani płatnej wersji z dodatkowymi funkcjami. Jeśli chcesz mieć drzewo na własnej stronie, płacisz tylko za swój hosting.'),
            ('Czy trzeba zakładać konto?', 'Nie. Program na komputerze i na telefonie zapisuje drzewo na Twoim urządzeniu i nie ma logowania. Na stronie z WordPressem albo MyRoots Server logujesz się do własnej strony. Rodzina ogląda drzewo bez konta albo z kontem, które jej założysz.'),
            ('Jak przenieść drzewo z MyHeritage, Ancestry albo FamilySearch?', 'Wyeksportuj drzewo z tamtego serwisu jako plik GEDCOM i otwórz go w MyRoots (Import i kopia, GEDCOM). MyRoots czyta GEDCOM 5.5.1 i 7.0, także spakowany jako GEDZIP, i sam rozpoznaje kodowanie znaków, więc polskie litery zostają na miejscu. Przenoszą się osoby, relacje, daty, miejsca, wydarzenia i notatki. Geneteka jest indeksem aktów, a nie serwisem z drzewami, więc drzewa z niej nie wyeksportujesz.'),
            ('Jak działa animacja „Przez lata”?', 'Wybierasz osobę, która ma kilka zdjęć z różnych lat. MyRoots wyrównuje twarze według oczu i płynnie przechodzi od najstarszego zdjęcia do najnowszego, z rokiem i wiekiem w rogu. Jeśli masz nagranie głosu tej osoby, zagra razem ze zdjęciami. Film zapiszesz jako wideo. Twarze rozpoznaje Twoje urządzenie, zdjęcia nigdzie nie są wysyłane.'),
            ('Czy mogę dodać zdjęcia i daty?', 'Tak. Każda osoba ma zdjęcie profilowe, galerię, skany dokumentów i nagrania głosu. Daty mogą być dokładne albo przybliżone (ok. 1890, przed 1850, między 1900 a 1905). Na zdjęciach grupowych oznaczysz, kto jest kim.'),
            ('Czy mogę wydrukować drzewo?', 'Tak. MyRoots robi plakat w formie wachlarza przodków (do 12 pokoleń) albo drzewa potomków (do 16 pokoleń) w formacie od A4 do 4A0, jako PDF dla drukarni. Księgę rodu z biografiami zapiszesz jako PDF i wydrukujesz jak książkę. Puste wzory do ręcznego wypełnienia są w <a href="{S_tpl}">szablonach do druku</a>.'),
            ('Czy moje dane są bezpieczne?', 'Drzewo zapisuje się na Twoim urządzeniu albo na Twoim serwerze, a autor programu nie ma do niego dostępu. Program łączy się z internetem tylko po mapę (domyślnie wyłączoną) i historyczną pogodę, którą wyłączysz w Ustawieniach. Żyjące osoby ukryjesz na stronie rodzinnej w całości albo tylko ich daty i miejsca. Szczegóły są w <a href="{S_priv}">polityce prywatności</a>.'),
            ('Czy jest aplikacja na telefon?', 'Na Androida tak: pobierz plik .apk i zainstaluj go (telefon zapyta o zgodę na instalację z tego źródła). Na iPhonie stronę rodzinną z WordPressa albo z MyRoots Server dodasz do ekranu początkowego i otworzysz jak aplikację.'),
            ('Czy mogę mieć drzewo na własnym serwerze bez WordPressa?', 'Tak. MyRoots Server to samodzielna strona na hostingu z PHP 8, bez bazy MySQL. Wgrywasz folder przez FTP, otwierasz adres i zakładasz konto administratora. Drzewo edytujesz z każdego komputera, a krewni dostają własne konta albo oglądają stronę bez logowania.'),
            ('Czy mogę zabezpieczyć drzewo hasłem?', 'Tak. W ustawieniach strony rodzinnej wybierasz, czy hasło chroni całą stronę, czy tylko wybrane osoby i gałęzie (z potomkami albo przodkami). Chroniona część, razem ze zdjęciami, jest szyfrowana, więc bez hasła nie da się jej odczytać nawet z plików strony.'),
            ('Czy mogę prowadzić kilka drzew?', 'Tak. W menu Drzewa genealogiczne dodajesz kolejne, na przykład rodzinę taty i rodzinę mamy. Każde ma własne osoby, zdjęcia, ustawienia i kopie. Na stronie internetowej każde drzewo ma też własny adres.'),
            ('W jakich językach działa MyRoots?', 'W 12: polskim, angielskim, niemieckim, francuskim, hiszpańskim, włoskim, portugalskim, niderlandzkim, szwedzkim, fińskim, duńskim i hebrajskim (z układem od prawej do lewej).'),
        ],
        'fin_h': 'Zacznij od siebie',
        'fin_p': 'Potem dodaj rodziców, dziadków i ich zdjęcia. Jeśli robisz drzewo pierwszy raz, zajrzyj do poradnika <a href="{S_guide}">jak zrobić drzewo genealogiczne</a>.',
        'ld_desc': 'Darmowy program do drzewa genealogicznego rodziny ze zdjęciami, datami, mapą i osią czasu, bez limitu osób.',
        'ld_features': ['Animacja zdjęć „Przez lata”', 'Księga rodu do druku (PDF)', 'Zwierzęta domowe w drzewie', 'Import GEDCOM 5.5.1 i 7.0, eksport GEDCOM 5.5.1', 'Plakat do druku do formatu 4A0', 'Strona rodzinna z hasłem i szyfrowaniem', 'Kilka drzew', '12 języków'],
    },
    # ------------------------------------------------------------------ templates
    'tpl': {
        'title': 'Szablon drzewa genealogicznego do druku: wzór PDF i Excel',
        'desc': 'Darmowy szablon drzewa genealogicznego do druku i do wypełnienia: od pradziadków, ze zdjęciami, wachlarz przodków, karta osoby. PDF i Excel bez rejestracji.',
        'crumb': 'Szablony',
        'h1': 'Szablon drzewa genealogicznego do druku',
        'lead': 'Darmowe wzory drzewa genealogicznego w PDF i Excelu: od pradziadków, ze zdjęciami, wachlarz przodków i karta osoby. Wydrukujesz je albo wypełnisz na komputerze.',
        'cta1': 'Wybierz szablon', 'cta2': 'Szablon w Excelu',
        'pdf_h': 'Wzory drzewa genealogicznego do pobrania',
        # (anchor, file (no extension), title, description, meta); anchors are shared by all languages
        'items': [
            ('do-druku', 'drzewo-genealogiczne-4-pokolenia-do-druku', 'Drzewo genealogiczne od pradziadków (4 pokolenia)', 'Ty, rodzice, dziadkowie i ośmioro pradziadków. Przy każdej osobie jest miejsce na imię i nazwisko oraz daty i miejsca urodzenia i śmierci.', 'PDF · A4 poziomo · do druku'),
            ('do-wypelnienia', 'drzewo-genealogiczne-4-pokolenia-do-wypelnienia', 'Drzewo genealogiczne do wypełnienia na komputerze', 'Ten sam wzór z polami formularza. Wpisujesz dane w Adobe Reader, w Podglądzie na Macu albo w przeglądarce, zapisujesz i drukujesz.', 'PDF z polami · A4'),
            ('ze-zdjeciami', 'drzewo-genealogiczne-ze-zdjeciami', 'Drzewo genealogiczne ze zdjęciami (3 pokolenia)', 'Ramki na zdjęcia dla Ciebie, rodziców i dziadków. Pasuje do projektu szkolnego, albumu i na prezent.', 'PDF · A4 poziomo · do druku'),
            ('wachlarz', 'wachlarz-przodkow-5-pokolen-a3', 'Wachlarz przodków (5 pokoleń, A3)', 'Drzewo w półkolu: strona taty po lewej, mamy po prawej, aż do prapradziadków. Wydrukuj na A3 albo powiększ w punkcie ksero.', 'PDF · A3 poziomo · do druku'),
            ('karta', 'karta-osoby-do-druku', 'Karta osoby', 'Dane jednej osoby, jej rodzice, małżeństwa, dzieci, wspomnienia i źródła (akt, parafia, sygnatura). Wypełniasz osobno dla każdego przodka.', 'PDF · A4 · do druku'),
            ('karta-pola', 'karta-osoby-do-wypelnienia', 'Karta osoby do wypełnienia', 'Ta sama karta z polami formularza do wypełnienia na komputerze.', 'PDF z polami · A4'),
        ],
        'btn': 'Pobierz PDF', 'pv_alt': '{t}, podgląd szablonu',
        'note': 'Pliki są wektorowe, więc dobrze wyglądają także po powiększeniu. Drukuj w skali 100%.',
        'xl_h': 'Szablon drzewa genealogicznego w Excelu',
        'xl_p': 'Lista osób z numerami ID i numerami rodziców. Na osobnym arkuszu drzewo czterech pokoleń układa się samo z formuł, gdy wpiszesz numer osoby w żółtej komórce. W pliku jest przykładowa rodzina, żeby było widać, jak wpisywać dane.',
        'xl_note': 'Format .xlsx · Excel, LibreOffice Calc, Numbers, Arkusze Google',
        'xl_btn': 'Pobierz szablon Excel', 'xl_file': 'drzewo-genealogiczne-szablon',
        'xl_alt': 'Szablon drzewa genealogicznego w Excelu: cztery pokolenia wypełnione formułami z listy osób',
        'schema': '''<h2>Jak wygląda schemat drzewa genealogicznego</h2>
<p>Najczęściej spotkasz trzy układy. Każdy szablon powyżej to jeden z nich.</p>
<h3>Drzewo przodków, czyli piramida</h3>
<p>Ty na dole, nad Tobą rodzice, dziadkowie i pradziadkowie. W każdym pokoleniu przodków jest dwa razy więcej: 2, 4, 8, 16. Tak jest zbudowany szablon „od pradziadków”.</p>
<h3>Wachlarz przodków</h3>
<p>To samo drzewo zwinięte w półkole. Mieści więcej pokoleń na jednej kartce i dobrze wygląda w ramie na ścianie.</p>
<h3>Drzewo potomków</h3>
<p>Na górze para przodków, niżej ich dzieci, wnuki i prawnuki. Przydaje się na zjazd rodzinny, bo każdy znajdzie na nim siebie.</p>
<p>W drzewie zapisuje się zwykle imię i nazwisko (u kobiet także nazwisko rodowe), datę i miejsce urodzenia, datę i miejsce śmierci, a przy parze datę ślubu. Jeśli daty nie znasz dokładnie, wpisz „ok. 1890” albo sam rok. To lepsze niż puste pole.</p>
<h2 id="online">Szablon online, bez drukowania</h2>
<p>Gdy kartki zaczynają brakować, przenieś drzewo do darmowego programu <a href="{L}">MyRoots</a>. Wpisujesz osoby, a drzewo, oś czasu i mapa rysują się same. Do każdej osoby dodasz zdjęcia i dokumenty, a na koniec wygenerujesz plakat do druku w dużym formacie.</p>
{FIG:plakat-wachlarz-przodkow|Generator plakatu w MyRoots: wachlarz przodków rodziny Zawadzkich gotowy do druku|Generator plakatu w MyRoots: wachlarz przodków z danymi wpisanymi raz, gotowy do drukarni.}''',
        'faq_h': 'Pytania o szablony',
        'faq': [
            ('Czy szablony są za darmo?', 'Tak. Wszystkie wzory pobierasz za darmo, bez rejestracji i bez podawania e-maila. Możesz je drukować dla siebie, rodziny i szkoły.'),
            ('Jak wypełnić drzewo genealogiczne na komputerze?', 'Pobierz wersję „do wypełnienia”. Otwórz ją w Adobe Acrobat Reader, w Podglądzie (Mac) albo w przeglądarce Chrome lub Edge, kliknij w pole i pisz. Zapisz plik i wydrukuj.'),
            ('Jak wydrukować wzór drzewa genealogicznego?', 'Drzewo 4 pokoleń i drzewo ze zdjęciami drukuj na A4 w poziomie, w skali 100% (bez dopasowania do strony). Wachlarz przodków jest przygotowany na A3; w drukarni lub punkcie ksero wydrukujesz go też na A2.'),
            ('Jak działa szablon drzewa genealogicznego w Excelu?', 'W arkuszu „Osoby” wpisujesz każdą osobę z numerem ID i numerami ID jej rodziców. W arkuszu „Drzewo 4 pokolenia” podajesz ID osoby, a rodzice, dziadkowie i pradziadkowie ułożą się sami. Działa w Excelu, LibreOffice i Arkuszach Google.'),
            ('Ile pokoleń zmieści się na szablonie?', 'Na kartce A4 wygodnie mieszczą się 4 pokolenia, na A3 w formie wachlarza 5. Jeśli chcesz więcej, użyj darmowego programu MyRoots: tam nie ma limitu pokoleń, a gotowy plakat (do 12 pokoleń przodków) wydrukujesz w dużym formacie.'),
            ('Czy jest szablon drzewa genealogicznego online?', 'Szablonem online jest program MyRoots. Wpisujesz osoby, a drzewo rysuje się samo, z datami i zdjęciami. Możesz je potem wydrukować jako plakat albo udostępnić rodzinie jako stronę.'),
        ],
        'ld_name': 'Szablony drzewa genealogicznego do druku',
    },
    # ------------------------------------------------------------------ guide
    'guide': {
        'title': 'Jak zrobić drzewo genealogiczne? Instrukcja krok po kroku',
        'desc': 'Jak zrobić drzewo genealogiczne: jak zacząć, jak sprawdzić przodków w USC, archiwach i Genetece, jak wygląda drzewo i pomysły na projekt. Instrukcja dla początkujących.',
        'crumb': 'Poradnik',
        'h1': 'Jak zrobić drzewo genealogiczne?',
        'lead': 'Jak zacząć, kogo zapytać, jak sprawdzić przodków w archiwach i jak zapisać drzewo rodziny na papierze albo w darmowym programie. Bez wcześniejszego doświadczenia.',
        'meta': 'Aktualizacja: {UPDATED} · czas czytania: 7 minut',
        'steps_h': 'Siedem kroków do drzewa rodziny',
        'steps': [
            ('Zacznij od siebie', 'Zapisz siebie, rodzeństwo, rodziców i ich rodzeństwo. Przy każdej osobie imię, nazwisko (u kobiet także rodowe), datę i miejsce urodzenia. To pierwsze dwa pokolenia, które zwykle znasz z pamięci.'),
            ('Wybierz, gdzie zapisujesz', 'Na początek wystarczy <a href="{S_tpl}">szablon do wydruku</a> z czterema pokoleniami. Jeśli chcesz dodać zdjęcia, dokumenty i więcej pokoleń, od razu zacznij w <a href="{L}">darmowym programie</a>, żeby później nie przepisywać.'),
            ('Porozmawiaj z najstarszymi', 'Babcie, dziadkowie i ciotki pamiętają imiona pradziadków, miejscowości i historie. Nagraj rozmowę telefonem (za zgodą). Zapytaj o przezwiska, przeprowadzki, wojnę, zawody i groby.'),
            ('Przejrzyj domowe archiwum', 'Akty urodzenia i ślubu, świadectwa, legitymacje, listy, książeczki wojskowe, klepsydry i odwrotne strony zdjęć. Zrób zdjęcia lub skany i od razu podpisz, kto jest na fotografii.'),
            ('Sprawdź w archiwach', 'Szukaj aktów urodzeń, ślubów i zgonów. Gdzie ich szukać, opisuję niżej w części <a href="#jak-sprawdzic">jak sprawdzić przodków</a>.'),
            ('Zapisuj źródła', 'Przy każdej informacji zanotuj, skąd ją masz: akt nr 412 z parafii św. Marii, opowieść cioci Heleny. Za rok nie będziesz pamiętać, a źródło pozwala oddzielić pewne fakty od rodzinnych legend.'),
            ('Pokaż drzewo rodzinie', 'Wydrukuj plakat, zrób księgę rodu albo stronę rodzinną. Krewni często dopowiadają brakujące fakty, gdy zobaczą drzewo.'),
        ],
        'body': '''<h2>Gdzie szukać aktów: w USC, w archiwach i w Genetece</h2>
<p>W Polsce akty urodzeń, małżeństw i zgonów są w dwóch miejscach, a o tym, gdzie szukać, decyduje wiek księgi. Zgodnie z art. 128 ustawy Prawo o aktach stanu cywilnego księgi urodzeń trafiają do archiwów państwowych po 100 latach, a księgi małżeństw i zgonów po 80 latach.</p>
<ul>
<li>Nowsze akty, na przykład urodzenia z ostatnich 100 lat, są w urzędzie stanu cywilnego. Odpis możesz zamówić dla siebie i najbliższej rodziny.</li>
<li>Starsze akty są w archiwach państwowych. Wiele z nich zeskanowano i udostępniono za darmo w serwisie <a href="https://www.szukajwarchiwach.gov.pl/">szukajwarchiwach.gov.pl</a>.</li>
<li>Indeksy pozwalają znaleźć akt po nazwisku bez przeglądania ksiąg strona po stronie. Największy polski indeks to <a href="https://geneteka.genealodzy.pl/">Geneteka</a> Polskiego Towarzystwa Genealogicznego.</li>
<li>Księgi parafialne sprzed XIX wieku bywają w archiwach diecezjalnych lub w parafiach. Warto zadzwonić i zapytać o zasady udostępniania.</li>
</ul>
<p>Akty z terenu dawnego zaboru rosyjskiego po 1868 roku są zwykle po rosyjsku, w zaborze pruskim po niemiecku, a księgi kościelne często po łacinie. Imiona i daty da się odczytać z pomocą prostych słowniczków, a na forach genealogicznych chętnie pomagają w tłumaczeniu.</p>
<div class="callout">Zapisuj sygnaturę każdego aktu (archiwum, zespół, rok, numer aktu). Dzięki niej Ty albo ktoś z rodziny wróci do dokumentu w kilka minut.</div>
<h2 id="jak-wyglada">Jak wygląda drzewo genealogiczne</h2>
<p>Klasyczne drzewo przodków to piramida: Ty na dole, nad Tobą rodzice, dziadkowie, pradziadkowie. W każdym pokoleniu wstecz przodków jest dwa razy więcej. Drzewo od pradziadków ma więc 15 osób, a do prapradziadków już 31. Inne układy, wachlarz i drzewo potomków, opisuję przy <a href="{S_tpl}#schemat">schematach drzewa</a>.</p>
{FIG:drzewo-genealogiczne-online-myroots|Przykład drzewa genealogicznego: rodzina Zawadzkich w pięciu pokoleniach w programie MyRoots|Przykład: rodzina Zawadzkich w programie MyRoots.}
<h2 id="pomysly">Pomysły na drzewo genealogiczne</h2>
<ul>
<li>Projekt szkolny z trzema pokoleniami i zdjęciami, wydrukowany na A4. Gotowy wzór to <a href="{S_tpl}#ze-zdjeciami">drzewo ze zdjęciami</a>.</li>
<li>Plakat na ścianę, czyli wachlarz przodków w ramie, z datami i miejscami.</li>
<li>Album rodzinny z drzewem na pierwszej stronie, a dalej ze zdjęciami i historiami każdej osoby.</li>
<li>Księga rodu na jubileusz dziadków albo zjazd rodzinny.</li>
<li>Drzewo z pupilami. Psy i koty z rodzinnych zdjęć to dobry temat do rozmowy z dziećmi.</li>
<li>Mapa rodzinnych miejsc, która pokazuje, skąd pochodzą przodkowie i gdzie są pochowani. Przydaje się przed wyjazdem na groby.</li>
</ul>
<h2 id="w-programie">Jak zrobić drzewo genealogiczne w programie MyRoots</h2>
<ol>
<li><a href="{L}#pobierz">Pobierz MyRoots</a> na komputer albo telefon z Androidem. Jeśli drzewo ma być od razu online, zainstaluj wtyczkę WordPress albo MyRoots Server na swoim hostingu.</li>
<li>Kliknij „Dodaj pierwszą osobę” i wpisz siebie.</li>
<li>Na swoim profilu wybierz „Dodaj krewnego”: ojca, matkę, małżonka albo dziecko. Program sam połączy osoby.</li>
<li>Dodaj zdjęcia, daty i miejsca. Daty niepewne wpisz jako „ok. 1890” albo „przed 1850”.</li>
<li>Masz już drzewo w innym serwisie? Zaimportuj plik GEDCOM.</li>
<li>Gotowe drzewo wydrukuj jako plakat albo księgę rodu, albo pokaż rodzinie jako stronę.</li>
</ol>''',
        'faq_h': 'Pytania o tworzenie drzewa',
        'faq': [
            ('Od czego zacząć drzewo genealogiczne?', 'Od siebie i najbliższych. Zapisz, co wiesz o rodzicach, dziadkach i pradziadkach, potem porozmawiaj z najstarszymi krewnymi i przejrzyj domowe dokumenty. Do archiwów idź na końcu.'),
            ('Jak sprawdzić swoich przodków za darmo?', 'Zacznij od indeksów Geneteki (geneteka.genealodzy.pl) i skanów w serwisie szukajwarchiwach.gov.pl. Akty urodzeń sprzed ponad 100 lat oraz akty małżeństw i zgonów sprzed ponad 80 lat są w archiwach państwowych, a nowsze w urzędach stanu cywilnego.'),
            ('Ile pokoleń wstecz da się odnaleźć?', 'Zwykle 5 do 8 pokoleń, czyli do początku XIX wieku, gdy wprowadzono akta stanu cywilnego. Na niektórych terenach księgi parafialne pozwalają cofnąć się do XVII wieku.'),
            ('Jak zrobić drzewo genealogiczne do szkoły?', 'Wystarczą 3 pokolenia: uczeń, rodzice i dziadkowie, z imionami, nazwiskami i rokiem urodzenia, najlepiej ze zdjęciami. Gotowy wzór ze zdjęciami jest w szablonach do druku.'),
            ('Jakie dane wpisać do drzewa genealogicznego?', 'Imię i nazwisko (u kobiet także rodowe), daty i miejsca urodzenia, ślubu i śmierci, zawód oraz źródło informacji (akt, parafia, kto to powiedział). Źródło pozwoli później sprawdzić, skąd wiesz, co wiesz.'),
        ],
    },
    # ------------------------------------------------------------------ wordpress
    'wp': {
        'title': 'Drzewo genealogiczne na stronę WordPress: wtyczka MyRoots',
        'desc': 'Darmowa wtyczka WordPress do drzewa genealogicznego online: kilka drzew, hasło do drzewa lub gałęzi, aplikacja PWA, zdjęcia, blok i shortcode. Dane zostają na Twoim serwerze.',
        'crumb': 'WordPress',
        'h1': 'Drzewo genealogiczne na Twojej stronie',
        'lead': 'Darmowa wtyczka MyRoots zamienia stronę WordPress w drzewo genealogiczne online. Edytujesz je w przeglądarce, w panelu swojej strony, a rodzina ogląda drzewo pod Twoim adresem, także w telefonie.',
        'btn': 'Pobierz wtyczkę (.zip)', 'demo': 'Zobacz demo',
        'img_alt': 'Panel MyRoots w WordPressie: lista zwierząt rodziny Zawadzkich z datami i opiekunami',
        'img_cap': 'Zwierzęta rodziny w panelu MyRoots, otwartym w WordPressie na pełnym ekranie.',
        'more_h': 'Co daje wtyczka',
        'more': [
            ('Dane zostają u Ciebie', 'Drzewo i zdjęcia są w bazie i folderze Twojego WordPressa. Nie korzystasz z zewnętrznego serwisu i nie płacisz abonamentu.'),
            ('Blok i shortcode', 'Drzewo pokażesz na osobnej stronie albo w dowolnym wpisie.'),
            ('Prywatność żyjących', 'Ukryj żyjące osoby albo tylko ich daty i miejsca. Możesz też ograniczyć widok do zalogowanych krewnych.'),
            ('Poprawki od rodziny', 'Krewni zgłaszają poprawki z poziomu drzewa. Dostajesz e-mail i zatwierdzasz je w panelu.'),
            ('Redaktor genealogii', 'Osobna rola dla krewnego, który pomaga budować drzewo, bez dostępu do ustawień strony.'),
            ('Kopia zgodna z aplikacjami', 'Plik .myroots otworzysz na Macu, Windowsie, Linuksie i Androidzie, a potem wgrasz z powrotem na stronę.'),
            ('Kilka drzew', 'Każde drzewo ma własne osoby, zdjęcia, adres (np. /drzewo/kowalscy/) i shortcode.'),
            ('Hasło do drzewa lub gałęzi', 'Zamknij hasłem całą stronę rodzinną albo wybrane osoby z potomkami lub przodkami. Chronione dane i zdjęcia są szyfrowane.'),
            ('Aplikacja na telefon', 'Strona rodzinna i panel edycji instalują się jak aplikacja PWA. Raz obejrzane drzewo działa też bez internetu.'),
        ],
        'inst_h': 'Instalacja w trzech krokach',
        'steps': [
            ('Wgraj wtyczkę', 'Pobierz plik <a href="{DL_WP}" download>myroots.zip</a>. W panelu wybierz Wtyczki, Dodaj nową, Wyślij wtyczkę na serwer, wskaż plik i włącz wtyczkę.'),
            ('Dodaj rodzinę', 'Otwórz MyRoots w menu panelu. Zacznij od pierwszej osoby, zaimportuj GEDCOM albo przenieś dane z wersji 2.x.'),
            ('Pokaż drzewo', 'Wstaw blok „MyRoots” lub shortcode [myroots] na stronie. Drzewo publikuje się samo po każdej zmianie.'),
        ],
        'faq_h': 'Pytania o wtyczkę',
        'faq': [
            ('Czy wtyczka jest darmowa?', 'Tak, na licencji GPL, bez wersji premium, reklam i limitu osób.'),
            ('Mam wtyczkę MyRoots 2.x. Co z moimi danymi?', 'Wersja 3.0 przenosi je jednym przyciskiem: Import i kopia, Z WordPressa, Importuj z tej strony. Osoby, relacje, zwierzęta, źródła i zdjęcia trafiają do nowej wersji, a stare dane zostają nietknięte. Strony ze starymi shortcode’ami działają dalej.'),
            ('Jak pokazać drzewo na stronie?', 'Wstaw blok „MyRoots” albo shortcode [myroots]. Możesz wybrać widok startowy, np. [myroots view="/tree"] albo [myroots view="/people"], a przy kilku drzewach także drzewo: [myroots tree="kowalscy"].'),
            ('Czy mogę ustawić hasło do drzewa?', 'Tak. W aplikacji, w ustawieniach strony rodzinnej, wybierasz hasło do całej strony albo tylko do wybranych osób i gałęzi. Hasło nie jest nigdzie zapisywane: przy publikacji chroniona część i jej zdjęcia są szyfrowane, a przeglądarka krewnego odszyfrowuje je po wpisaniu hasła.'),
            ('Czy drzewo można zainstalować na telefonie?', 'Tak. Strona rodzinna jest aplikacją PWA: w Chrome i Edge wybierz „Zainstaluj”, w Safari na iPhonie „Do ekranu początkowego”. Tak samo zainstalujesz panel edycji MyRoots.'),
            ('Kto może oglądać drzewo?', 'Wszyscy, tylko zalogowani użytkownicy strony albo nikt poza redaktorami. Żyjące osoby chroni dodatkowo ustawienie prywatności w aplikacji.'),
            ('Jakie są wymagania?', 'WordPress 6.5 lub nowszy i PHP 7.4 lub nowszy. Edytor działa w każdej współczesnej przeglądarce.'),
            ('Nie mam WordPressa. Czy jest inna droga?', 'Tak. <a href="{L}#pobierz">MyRoots Server</a> to ta sama aplikacja jako samodzielna strona na zwykłym hostingu z PHP 8, bez bazy MySQL.'),
        ],
    },
    # ------------------------------------------------------------------ privacy
    'priv': {
        'title': 'Polityka prywatności MyRoots',
        'desc': 'MyRoots nie zbiera danych: bez kont u autora, analityki i reklam. Kiedy program łączy się z internetem i co wtedy wysyła.',
        'crumb': 'Prywatność',
        'h1': 'MyRoots nie zbiera żadnych danych',
        'lead': 'Program nie ma kont u autora, reklam ani analityki. Wszystko, co wpisujesz, zostaje na Twoim urządzeniu albo na Twoim serwerze, a autor programu nie ma do tego dostępu.',
        'body': '''<h2>Gdzie są Twoje dane</h2>
<ul>
<li>Program na Maca, Windows i Androida: w folderze danych programu na urządzeniu.</li>
<li>Wersja w przeglądarce: w pamięci przeglądarki (IndexedDB) na tym komputerze.</li>
<li>Wtyczka WordPress: w bazie danych i folderze <code>uploads</code> Twojej witryny.</li>
<li>MyRoots Server: w folderze <code>data</code> na Twoim serwerze.</li>
<li>Kopie zapasowe, pliki GEDCOM i strona rodzinna powstają tylko wtedy, gdy je zapiszesz, i trafiają tam, gdzie wskażesz.</li>
</ul>
<h2>Kiedy program łączy się z internetem</h2>
<p>Tylko w tych sytuacjach i tylko z Twojej inicjatywy:</p>
<ul>
<li>Mapa (domyślnie wyłączona). Fragmenty mapy są pobierane z serwerów OpenStreetMap, a nazwy miejsc wysyłane do wyszukiwarki Nominatim, a gdy ta nic nie znajdzie, do Photon (komoot), żeby znaleźć współrzędne. Obowiązuje <a href="https://osmfoundation.org/wiki/Privacy_Policy">polityka prywatności OpenStreetMap</a>.</li>
<li>Pogoda (można ją wyłączyć w Ustawieniach; we wtyczce WordPress domyślnie wyłączona). Dla wydarzeń z dokładną datą program wysyła do Open-Meteo.com współrzędne miejsca i datę. Nazwiska i inne dane nie są wysyłane. <a href="https://open-meteo.com/en/terms">Warunki Open-Meteo</a>.</li>
<li>Aktualizacje (tylko program na komputer; można je wyłączyć w Ustawieniach). Raz dziennie program pobiera z myroots.cc numer najnowszej wersji, a po Twojej zgodzie samą aktualizację, podpisaną przez autora. Nic z Twojego drzewa nie jest wysyłane.</li>
<li>Import z WordPressa. Program pobiera zdjęcia z Twojej witryny.</li>
<li>Linki i trasa odwiedzin grobów otwierają się w przeglądarce. Trasa w Mapach Google zawiera tylko położenie wybranych cmentarzy.</li>
</ul>
<h2>Zdjęcia, twarze, mikrofon, lokalizacja</h2>
<p>Rozpoznawanie twarzy i oczu działa wyłącznie na Twoim urządzeniu. Mikrofon działa tylko podczas nagrywania, które sam rozpoczynasz. Położenie urządzenia jest odczytywane tylko po kliknięciu „Jestem przy grobie” albo „Trasa odwiedzin” i nigdzie nie jest wysyłane.</p>
<h2>Strona rodzinna</h2>
<p>Stronę rodzinną tworzysz i publikujesz sam. Możesz ukryć lub ocenzurować dane żyjących osób oraz zamknąć hasłem całą stronę albo wybrane gałęzie; chroniona część jest szyfrowana. Przy publikacji program usuwa ze zdjęć ukryte dane (m.in. położenie GPS) i opisy twarzy.</p>
<h2>Ta strona internetowa</h2>
<p>Strona myroots.cc nie używa ciasteczek ani zewnętrznych czcionek. Odwiedziny liczymy narzędziem <a href="https://umami.is">Umami</a>, bez ciasteczek i bez danych osobowych: widzimy tylko zbiorcze liczby, na przykład które strony są czytane, skąd przychodzą odwiedzający, z jakiego kraju i z jakiej przeglądarki. Serwer zapisuje też standardowe logi dostępu hostingu.</p>
<h2>Kontakt</h2>
<p>Wydawca: Jarek Hirsch, <a href="{AUTHOR_URL}">hirsch.lt</a>. Obowiązuje od 20 października 2026 r., zaktualizowano {UPDATED} r.</p>''',
    },
    # ------------------------------------------------------------------ code signing
    'sign': {
        'title': 'Code signing policy: podpisywanie kodu MyRoots',
        'desc': 'Jak podpisywany jest MyRoots na Windows: pakiet w Microsoft Store podpisuje Microsoft, osobny instalator .exe nie jest podpisany.',
        'crumb': 'Code signing policy',
        'h1': 'Code signing policy',
        'lead': 'MyRoots na Windows jest dostępny w <a href="{STORE}">Microsoft Store</a>. Microsoft sprawdza i podpisuje pakiet, zanim trafi do Ciebie.',
        'body': '''<p>Zalecany sposób instalacji MyRoots na Windows 10 i 11 to Microsoft Store. Każda wersja przechodzi certyfikację Microsoftu, a pakiet MSIX jest podpisywany certyfikatem Microsoftu. Jako wydawca w Sklepie widnieje Jarek Hirsch. Aktualizacje przychodzą automatycznie przez Sklep.</p>
<h2>Instalator bez Sklepu</h2>
<p>Na komputery bez Microsoft Store jest też klasyczny instalator (<a href="{DL_WIN}">.exe</a>). Nie jest podpisany certyfikatem do podpisywania kodu, więc Windows SmartScreen może pokazać ostrzeżenie. Aby kontynuować, kliknij „Więcej informacji”, a potem „Uruchom mimo to”. Instalator zawiera ten sam program co wersja ze Sklepu.</p>
<h2>Inne systemy</h2>
<ul>
<li>Android: plik APK jest podpisany własnym kluczem autora.</li>
<li>macOS: aplikacja nie jest jeszcze notaryzowana przez Apple. Przy pierwszym uruchomieniu kliknij MyRoots.app prawym przyciskiem i wybierz „Otwórz”.</li>
</ul>
<h2>Prywatność</h2>
<p>Ten program nie przesyła żadnych informacji do innych systemów w sieci, chyba że użytkownik o to poprosi albo sam włączy daną funkcję (mapa, historyczna pogoda, import z własnej witryny WordPress). Szczegóły: <a href="{S_priv}">polityka prywatności</a>.</p>''',
    },
    # ------------------------------------------------------------------ 404
    'nf': {'title': 'Nie znaleziono strony · MyRoots', 'h1': 'Tej strony tu nie ma', 'lead': 'Adres mógł się zmienić albo zawierać literówkę. Poniżej są miejsca, których ludzie zwykle szukają.', 'home': 'Strona główna', 'dl': 'Pobierz MyRoots', 'tpl': 'Szablony do druku'},
    'updated': '13 listopada 2026',
}
