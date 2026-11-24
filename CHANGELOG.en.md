# Changes

## 3.2.0 (2026-11-24)

- MyRoots Server: photos on the public family website load at once. The server keeps them next to the website as plain files that the hosting sends without PHP (before, every photo waited in the PHP queue).
- Microsoft Store: the Store version is updated by the Store and shows no update notices of its own.
- MyRoots Server and the WordPress plugin: ZIP packages with file permissions every hosting can read. After an FTP upload there is no more blank page or 403 error.
- Every release first passes the full tests: the app in a browser on a computer and a phone, builds on Windows and macOS, MyRoots Server on PHP and the plugin on a real WordPress (the oldest supported version and the newest).

## 3.1.0 (2026-11-22)

**New look**
- Lighter and calmer: a white background, hairlines instead of panels everywhere, one filled button per screen and outlined ones for the rest. Tabs as text with an underline.
- New icons (Phosphor), thinner and less generic.
- Quick search: "Search" at the top of the menu and Ctrl K (Cmd K on a Mac) from every screen. People with photo, years and place, pages of the app and "Add a person"; arrows choose, Enter opens.
- Profile: a large photo (full width on the phone) and labelled links: Map, Kinship, Through the years, Poster, History, Print / PDF.
- Timelines: the year big and thin on the lifeline, in the events and in the family timeline; numbers in statistics and the data check in the same style.
- Panel beside the tree: the person's photo on top with the place of birth and a round button to the profile.
- People album: the place of birth on a glass label over the photo.
- Phone: a floating menu bar in the accent color.
- A discreet "Support" link in the menu (Suppi in Polish, Ko-fi in other languages).
- Gentle animations when a screen opens and over photos; off when the system asks for reduced motion.
- The appearance settings (colors, presets, font, text size, corners, shadows, dark theme) work as before.

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
- GEDCOM: photos and scans from other programs (from the .gdz/.zip archive or a folder chosen after the import), archives and parishes with call numbers (REPO), godparents and witnesses (ASSO), nicknames (NICK, ALIA), LDS ordinances, and Julian, Hebrew and French Republican dates converted to the Gregorian calendar.
- Person: nickname, and godparents, witnesses, guardians and other associations, shown on both profiles. Source: archive, parish or library, and call number.
- Map: a Cemeteries layer with the family's cemeteries, the cemeteries of OpenStreetMap around, and your own; add a cemetery by its address or by clicking the map.
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
- Big trees (tens of thousands of people, thousands of photos): GEDCOM import with a ZIP of photos of any size (before, a few hundred MB were impossible), backups built and restored piece by piece, faster start page, events and places, also on phones.
- MyRoots Server and WordPress: the editor opens big trees also on hosting with a 128 MB PHP memory limit (before: an error); tree data and the family website are sent compressed (many times less data).
- When the browser runs out of space, the app says so clearly and stops the import (before, changes could silently fail to save).
- Every unexpected error gives a message instead of silence; a wrong file when restoring a backup says it is not a MyRoots backup.
- A big backup is no longer cut off while saving to disk.
- New: "Report a problem" in the menu (e-mail or GitHub, with technical details, without tree data).
- Phone: the bottom bar no longer shows the content scrolled under it.
- MyRoots Server: photos load in parallel. Before, the login session blocked the next requests, so photos in the gallery and the tree arrived one after another, and the public family website opened a session for every photo.
- MyRoots Server and WordPress: publishing the website and restoring a backup send several photos at once (about 35% faster in the test), send a piece of a file again when the hosting loses an answer, and warn before the tab is closed in the middle. A website without full photos no longer fetches the originals to publish.
- Maps: the background now comes from OpenFreeMap (no limits, no keys) instead of openstreetmap.org tiles, which sometimes answered with error 403, especially in the desktop app. Names of places and countries are in the app language. Styles in the settings: Standard, Pale, Night and Terrain.
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
