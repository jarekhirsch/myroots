# Pliki źródłowe

Lista generowana przez `npm run docs` z pierwszego komentarza w każdym pliku. Nowy plik: zacznij go jednym zdaniem komentarza o tym, co robi.

<!-- auto:files -->
### crates/myroots-core/src

| Plik | Opis (pierwszy komentarz w pliku) |
|---|---|
| `archive.rs` | `.myroots` archive: a ZIP with the full database and all media. |
| `book.rs` | Printable genealogy book: selects people and splits them into chapters. |
| `charsets.rs` | Single-byte code pages and ANSEL used by older GEDCOM files (generated tables). |
| `date.rs` | Dates with an uncertainty qualifier, stored in a single string field |
| `db.rs` | In-memory database with change tracking. |
| `engine.rs` | JSON command dispatcher shared by every platform: |
| `error.rs` | / Error returned to the UI: a stable `code` (translated by the UI) plus a |
| `fuzz.rs` | Robustness tests: random and mangled input must never crash the engine |
| `gedcom.rs` | GEDCOM 5.5.1 import and export. |
| `health.rs` | Data health: consistency checks and completeness report. |
| `kinship.rs` | Kinship calculator. |
| `lib.rs` | MyRoots core — the genealogy engine shared by the desktop, mobile and web |
| `life.rs` | Life helpers: age, living / presumed death. |
| `lifeline.rs` | Personal lifeline: a timeline of the events of a person's life, |
| `map.rs` | Genealogical map: events grouped by place. |
| `merge.rs` | Duplicate detection and merging of two people. |
| `metadata.rs` | Removes hidden metadata from photos published on the family website: |
| `model.rs` | Data model of a MyRoots family tree. |
| `ops.rs` | Mutating operations (people, relations, unions, pets, sources, media …). |
| `privacy.rs` | Privacy for the published family website. |
| `query.rs` | Read-only views assembled for the UI. |
| `site.rs` | Export of the family website: a static, read-only copy of the app |
| `stats.rs` | Family statistics. |
| `tree.rs` | Data for the interactive family tree (family-chart format) and the |
| `wp_import.rs` | Import of a backup made by the MyRoots WordPress plugin |

### crates/myroots-store/src

| Plik | Opis (pierwszy komentarz w pliku) |
|---|---|
| `lib.rs` | Native storage of a MyRoots tree. |

### crates/myroots-wasm/src

| Plik | Opis (pierwszy komentarz w pliku) |
|---|---|
| `lib.rs` | WebAssembly bindings: the browser app (PWA) and the exported family website |

### src-tauri/src

| Plik | Opis (pierwszy komentarz w pliku) |
|---|---|
| `lib.rs` | MyRoots desktop / mobile shell. |
| `main.rs` | Prevents an additional console window on Windows in release. |

### src/views

| Plik | Opis (pierwszy komentarz w pliku) |
|---|---|
| `AgingView.svelte` | "Through the years" of one person: the animation, and (in the editor) |
| `BookView.svelte` | Printable family book: cover, table of contents, one page per person, index of surnames. |
| `Clans.svelte` |  |
| `DataView.svelte` |  |
| `Duplicates.svelte` |  |
| `Events.svelte` |  |
| `Gallery.svelte` |  |
| `Graves.svelte` | Graves of the family: by cemetery, on a map, and a route for a visit |
| `Health.svelte` |  |
| `Home.svelte` |  |
| `Kinship.svelte` |  |
| `LifelineView.svelte` |  |
| `MapView.svelte` | Map of family places (births, marriages, deaths, events). OpenStreetMap is opt-in. |
| `NotFound.svelte` |  |
| `People.svelte` |  |
| `PersonEdit.svelte` |  |
| `PersonHistory.svelte` |  |
| `PersonView.svelte` |  |
| `PetEdit.svelte` |  |
| `PetView.svelte` |  |
| `Pets.svelte` |  |
| `Places.svelte` |  |
| `Poster.svelte` | A poster to print in a large format: a fan of ancestors or a tree of |
| `SettingsView.svelte` |  |
| `SourceEdit.svelte` |  |
| `SourceView.svelte` |  |
| `Sources.svelte` |  |
| `Stats.svelte` |  |
| `Suggestions.svelte` |  |
| `TreeView.svelte` | Interactive family tree (family-chart + d3). Click a card to center it, |
| `TreesView.svelte` | Several separate family trees: the list, a new tree, switching, renaming, deleting. |
| `Website.svelte` |  |

### src/lib/components

| Plik | Opis (pierwszy komentarz w pliku) |
|---|---|
| `AboutDialog.svelte` | About MyRoots: author, licence, the components it is built with. |
| `AddRelative.svelte` | Adds a relative to a person: a brand-new person or a link to someone already in the tree. |
| `AgingPlayer.svelte` | Plays "Through the years": the photos aligned on the eyes, cross-fading |
| `AlbumSplitter.svelte` | A scanned album page → separate, straightened photos. |
| `AnniversaryList.svelte` |  |
| `AudioPlayer.svelte` | A small player for one recording: play / pause, a scrubbable bar and the time. |
| `AudioRecorder.svelte` | Recording from the microphone: record, listen, keep or try again. |
| `Avatar.svelte` | Portrait of a person: a soft rounded square. Without a photo, the initials |
| `BackButton.svelte` | "Back" to the previous view (or, after opening a link directly, to the |
| `BarList.svelte` | Horizontal bars (ranked lists) or columns (distributions). One hue, values as text. |
| `CitationButton.svelte` | Sources of one fact (birth, an event, a marriage …): a small button next |
| `ClanInput.svelte` | Clans (lineages) of a person: chips + input with suggestions. |
| `DateInput.svelte` | Qualified date: exact / about / before / after / between (stored as 1920, ~1920, <1920, >1920, 1900..1910). |
| `EyeAligner.svelte` | Setting the eyes on one photo: two markers to drag onto the centres of |
| `FamilyGraph.svelte` | The whole family on one board: every person and every connection. |
| `FamilyTimeline.svelte` | The whole family on one time axis: a bar from birth to death for every |
| `FramePreview.svelte` | How one photo will look in "Through the years" (aligned on the eyes). |
| `GalleryEditor.svelte` | Ordered list of images with captions. |
| `GraveEditor.svelte` | The grave of a person: cemetery, place on it, position, tombstone photo. |
| `Html.svelte` | links open outside the app and never get access to it |
| `Img.svelte` |  |
| `LifelineList.svelte` | A life on an age scale: own facts and events of close relatives, spaced |
| `Lightbox.svelte` | A photo at full size. People marked on it are listed under the photo and |
| `MapPicker.svelte` | Pick a point on the map (e.g. a grave): search a place, then click the exact spot. |
| `MediaLibrary.svelte` | Picks images already stored in the app (shared between people, pets and sources). |
| `MiniMap.svelte` | Small map of one person's places in the profile (only when map services |
| `Modal.svelte` |  |
| `PersonLink.svelte` |  |
| `PersonPicker.svelte` |  |
| `PhotoField.svelte` | A single image (profile photo, cover, event picture). |
| `PlaceInput.svelte` |  |
| `ProgressBar.svelte` | Bar of long file work (see `progress` / `withProgress` in ui.svelte.ts). |
| `QuickSearch.svelte` | Search box in the menu: jumps to a person's profile. |
| `RecordingsEditor.svelte` | Voice recordings of a person: upload files or record from the microphone. |
| `RichText.svelte` | Small rich-text editor (bold, italic, headings, lists, quote, link). |
| `SuggestDialog.svelte` | "Suggest a correction" on the family website. There is no server: the |
| `TreeDetails.svelte` | Data panel beside the tree: everything important about the selected |
| `Unlock.svelte` | Password of a protected family website (the whole website or chosen people). |
| `UpdateBanner.svelte` | A quiet bar at the bottom: a new version of the desktop app is ready. |
| `Weather.svelte` | The weather on the day of an event (icon and temperatures; details in the tooltip). |

### src/lib

| Plik | Opis (pierwszy komentarz w pliku) |
|---|---|
| `aging.ts` | "Through the years": photos of one person aligned on the eyes and played |
| `albumsplit.ts` | Finds the photos on a scanned album page: everything that differs from the |
| `api.svelte.ts` | Shared reactive state. `version` changes after every write → views reload. |
| `audio.ts` | Sound recordings: storing files and recordings from the microphone, and |
| `citations.ts` | Citations of sources for single facts (see `Person.citations` and |
| `credits.ts` | Author, licence and the components MyRoots is built with (shown in "About"). |
| `errors.ts` | Human message for an engine error code. |
| `exif.ts` | GPS position from the EXIF data of a JPEG photo (taken with a phone at the grave). |
| `faces.ts` | Finding the eyes on a photo, on the device (no network): a small face |
| `familygraph.ts` | Layout of the whole family on one board (the plugin's "whole graph"): |
| `format.ts` |  |
| `generations.ts` | Family on a time axis: generations from the relations, a life span for |
| `geocode.ts` | Geocoding of place names with OpenStreetMap Nominatim (opt-in, 1 request per second). |
| `history.ts` | The most important events in the history of Poland since 1600 (for the whole |
| `kinship.ts` | Kinship terms, ported from the WordPress plugin (MyRoots_Kinship) so that |
| `load.svelte.ts` | Reactive loader: runs `fn` on mount and again after every data change |
| `media.ts` | Image processing before storing: downscale very large photos and make a thumbnail. |
| `migrations.ts` | Moves of the family between places: from a parent's place of birth to the |
| `nav.ts` | Navigation of the app and of the read-only family website. |
| `pdf.ts` | A one-page PDF of an SVG at an exact paper size. The picture is drawn in |
| `photodate.ts` | When was a photo taken? From the people marked on it: everyone was already |
| `poster.ts` | Geometry of the printable posters: a fan of ancestors and a tree of |
| `print.ts` | Printing: the desktop app opens the system print dialog through Tauri |
| `regions.ts` | Approximate historical regions, to show local events of history only to |
| `router.svelte.ts` | Tiny hash router (works in Tauri, the PWA and the static family website). |
| `sitelock.ts` | Password protection of the family website. |
| `text.ts` | Lowercase, Latin accents folded (search and sorting), same idea as `model::norm` in Rust. |
| `theme.ts` | Applies the "Appearance" settings (colors, font, size, corners, theme) as CSS variables. |
| `trees.svelte.ts` | Several family trees: the list, switching and keeping the names in step. |
| `types.ts` | TypeScript mirrors of the Rust model (crates/myroots-core/src/model.rs). |
| `ui.svelte.ts` | Toasts and confirmation dialogs. |
| `updates.svelte.ts` | Updates of the desktop app (macOS, Windows, Linux AppImage) from myroots.cc. |
| `weather.ts` | Historical weather of a day at a place: Open-Meteo, ERA5 reanalysis |
| `wpmedia.ts` | Brings the photos and scans of a WordPress MyRoots site into the app. |

### src/lib/backend

| Plik | Opis (pierwszy komentarz w pliku) |
|---|---|
| `assets.ts` | Where the static files of the app live. Normally next to the page ("./"), |
| `idb.ts` | Minimal promise wrapper around IndexedDB. |
| `index.ts` | The editor inside WordPress (the MyRoots plugin prints its configuration). |
| `tauri.ts` | Desktop / mobile backend: Tauri commands backed by the native Rust engine + SQLite. |
| `types.ts` | Platform abstraction: the same UI runs on Tauri (desktop / mobile), |
| `util.ts` |  |
| `viewer.ts` | Read-only family website: the engine runs on the exported public data. |
| `wasm.ts` | Browser backend: the Rust engine compiled to WebAssembly + IndexedDB storage. |
| `wp.ts` | WordPress backend: the same Rust engine (WebAssembly) as the browser app, |

### scripts

| Plik | Opis (pierwszy komentarz w pliku) |
|---|---|
| `build-app-strings.mjs` | Builds src/lib/i18n/extra/{lang}.json from the hand-written translation |
| `build-wasm.mjs` | Builds the WebAssembly engine used by the browser app and the family website. |
| `check-i18n.mjs` | Lists UI strings that have no translation in the catalogs (plugin .po + extra/*.json). |
| `docs.mjs` | Keeps the generated parts of docs/dev/*.md in step with the code. |
| `finalize-viewer.mjs` | Makes the viewer template work from a local disk (file://): classic deferred |
| `po2json.mjs` | Converts the WordPress plugin translations (.po) + extra strings of the app |
| `updater-json.mjs` | Prepares the update files for myroots.cc/updates/ from the assets of a |
<!-- /auto:files -->
