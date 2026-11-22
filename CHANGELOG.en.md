# Changes

## 3.1.0 (2026-11-22)

Big trees run smoothly, sources can be attached to every piece of information, and MyRoots is available for Linux.

**New**
- Sources next to every piece of information, as in MyHeritage: a Source button next to the name, birth, death, occupation, education, religion, biography, grave, every event and every marriage. Each citation can have the page or record number, its reliability (from direct evidence to unreliable) and the text of the record, and a new source can be created without leaving the form.
- Settings, Sources: choose between sources next to every fact and one list of sources per person with ticks of the facts each source confirms. Both keep the same data.
- Profile: every source shows what it confirms, with the page, reliability and text of the record; source numbers also next to the name, the grave and marriages.
- GEDCOM: import and export of citations of facts, events and marriages with PAGE, QUAY and a note (DATA/TEXT, NOTE), including sources of the name, education, religion and burial.
- Map: a Graves layer and a cemetery filter. A chosen cemetery shows the graves of the people buried there and, with the other layers, where they came from.
- Linux: .deb (Ubuntu, Debian, Mint), .rpm (Fedora, openSUSE) and AppImage packages.
- Tree: highlight the direct line (the ancestors and descendants of the chosen person stay big, side lines get smaller and paler) and colors of the four lines of the grandparents, as in MyHeritage. Both are in the Lines list above the tree.
- Tree: clicking a clan on a card highlights only the people of that clan and lists them. The list of clans also opens a clan in the tree and on the map.
- Settings, Appearance: colors of the tree background, the lines between people, the line to the chosen person and the four lines of the grandparents.
- Person: an Address field (place of residence), also in GEDCOM (RESI with ADDR), and Unconfirmed information: family stories and clues to check, seen only by you and never on the family website.
- The people list shows the cause of death and the address, and the search looks in them.
- Map, migrations: Who from this place went further, that is the people born at one place (a house, a village) and where their descendants went. A Where the family lived tile on the home page and a Migrations button in the list of places.
- A progress bar when adding photos and recordings, cutting photos out of an album page, importing GEDCOM and restoring a backup.
- Updates: the app for Mac, Windows and Linux (AppImage) checks once a day for a new version and offers to install it. This can be turned off in Settings.

**Speed of big trees (tens of thousands of people)**
- Possible duplicates are found in a fraction of a second instead of tens of seconds.
- Profiles, the edit form, lifelines and kinship open at once: family relations are computed once, not on every click.
- People lists, search boxes and person pickers stay fast while typing.
- Whole family draws only what is on the screen; from far away the cards are simplified.
- Around a person starts big trees with 6 generations of ancestors and 3 of descendants (can be changed).
- Graves, data consistency and missing data are shown in parts, and the home page no longer builds the whole list of problems only to show their number.

**Fixes**
- Importing a GEDCOM file again with new people no longer duplicates the people already in the tree: they are recognised by the identifiers in the file (_UID, RIN, REFN and others) or by name and dates.
- The More menu shows only what is not in the bar, without repeats.
- Links to the repository point to github.com/jarekhirsch/myroots.

**Libraries**
- Vite 8, Svelte 5.59, Tauri 2.12, TypeScript 6, rusqlite 0.43, wasm-bindgen 0.2.130 and the other dependencies in their latest versions; no known vulnerabilities.

## 3.0.0 (2026-10-20)

The first standalone release of MyRoots, without WordPress. The same app runs on Mac, Windows and Linux, on phones (iOS, Android) and in the browser, and each of them can publish a read-only family website.

**New**
- Tree: Around a person and Whole family (every person and connection on one board), adding people straight from the tree, a panel with the data of the selected person.
- Home: a timeline of generations with zoom and folding.
- Through the years: photos of a person from different years aligned on the eyes (faces found on the device), played with the year and age; export to video.
- Voice recordings: record from the microphone or add files; a player in the profile and on the family website; a recording as the soundtrack of Through the years.
- Map: clusters of places, a toolbar as in the tree, places of one person, a list of places without coordinates with search under another name or a click on the map, and Migrations with lines between places of birth of parents and children, moves during a life, a year slider and playback.
- Back button and shortcuts (Alt+Left, Cmd+[, the mouse back button).
- GEDCOM: ANSEL, Windows-1250/1252, ISO-8859-2, Mac Roman, GEDCOM 7 and zipped files (.gdz, .zip); married and birth surnames, many more events, inline sources, dates from other programs; export compatible with GEDCOM 5.5.1.
- Deceased with an unknown date of death.
- The edit form creates a new parent, partner or child straight from the picker.
- People on group photos: faces found on the device can be assigned to people in the tree, new people or just named; the profile shows Also in photos; the family website respects the privacy settings.
- Historical background: about 160 key events of the history of Poland since 1600 in the lifeline with the age of the person at the time, local events only for people from the region; own events and turning off built-in ones in Settings.
- Graves: cemetery, plot, position (from the map, from the phone at the grave or from the photo of the tombstone), photo and a link to grave registers; the Graves page groups them by cemetery, shows them on a map and plans a visit in Google Maps; exported to GEDCOM as burials.
- Photos: suggestions of who is in a photo (face recognition on the device), the estimated date of a photo, splitting a scanned album page into photos.
- Printable posters: an ancestor fan (up to 8 generations) and a descendant tree, A4 to A0; PDF, SVG and PNG.
- Weather on the day of an event (Open-Meteo.com, from 1940; can be turned off).
- About window with the licence and the list of libraries.

**Fixes**
- Mac app: printing opens the system print dialog; posters save PDF in the exact paper size.
- Map search identifies itself to OpenStreetMap and falls back to Photon; a wrong place name can be corrected in the whole tree.
- Prints, posters and the Through the years video carry a discreet “© MyRoots · Jarek Hirsch”.
- Desktop app: adding photos works, with a clear message when a photo cannot be added.

**Security and privacy**
- The family website contains no hidden photo data (GPS position, camera, comments).
- Links from imported files are checked (http, https, mailto only), HTML is sanitized, and the app has a Content-Security-Policy.
- Robustness tests with random and damaged GEDCOM files, backups and data.

**Licence**
- GNU GPL version 3 or later with additional terms (see `NOTICE.md`).
