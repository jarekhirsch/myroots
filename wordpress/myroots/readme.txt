=== MyRoots ===
Contributors: jarczi69
Tags: genealogy, family tree, gedcom, ancestry, family history
Requires at least: 6.5
Tested up to: 7.1
Requires PHP: 7.4
Stable tag: 3.2.0
License: GPLv3 or later
License URI: https://www.gnu.org/licenses/gpl-3.0.html

Family tree for WordPress with the same interface and engine as the MyRoots 3 app for Mac, Windows, Linux, phones and the browser.

== Description ==

MyRoots 3 brings the MyRoots app into WordPress. The editor in the admin panel is the very same app as on the Mac: the same screens, the same Rust engine (compiled to WebAssembly), the same features. Your family members see a read-only family website, published by you, with living people protected.

* People, relations, pets, sources, places, graves, photos, scans and voice recordings.
* Interactive family tree, timeline of generations, lifeline with historical background, map, gallery, events and anniversaries.
* Statistics, kinship calculator, genealogy book, posters, data consistency check, duplicate finder.
* Marking people on group photos, "years" animation of faces, splitting album pages.
* GEDCOM 5.5.1 import and export, full backup (.myroots) that opens in the desktop and mobile app too.
* Family website: privacy for living people (censor fields, hide, or lock sections), suggestions of corrections from the family, a link anyone can open, or only logged-in users.
* Several separate family trees on one site, each with its own people, photos, family website address and shortcode.
* A password for the whole family website or for chosen people and branches. The protected part, photos included, is encrypted in the browser of the editor; the password itself is never stored.
* Installable app (PWA): the family website and the editor can be added to the home screen, the dock or the Start menu. The family website works offline once it has been seen.
* Interface in 12 languages (English, Polish, German, French, Spanish, Italian, Portuguese, Dutch, Swedish, Finnish, Danish, Hebrew).

= Where things are =

* MyRoots → Family tree: the app, full screen. The way back to WordPress is in its menu.
* Shortcode [myroots view="/tree" height="85vh"] or the "MyRoots" block: the family tree for visitors inside any page (the same app, read-only). With several trees: [myroots tree="smiths"].
* MyRoots → Settings: who can see the tree, e-mails about suggestions.

= Moving from MyRoots 2.x =

Install version 3 over 2.x (or next to it, after deactivating 2.x). Open MyRoots → Family tree → Import and backup → From WordPress → "Import from this site". People, relations, pets, sources, photos and scans are copied; the 2.x data is not changed. Pages with 2.x shortcodes keep working (they show the matching view of the website).

= Source code =

The files in app/ (one build, used by the editor and, read-only, by the shortcode) are built from human-readable sources: the interface (Svelte, TypeScript) and the engine (Rust, compiled to WebAssembly) of the MyRoots 3 app, plus the WordPress backend (src/lib/backend/wp.ts). Build: npm ci, npm run i18n, npm run build:viewer, npm run build:app (see the app README). Face models in app/models/face/ are the data files (weights) of face-api, renamed from .bin to .shard.

== Frequently Asked Questions ==

= Where is my data stored? =

In the WordPress database (table wp_myroots_records) and in wp-content/uploads/myroots/ (photos, website). Nothing is sent anywhere else. Optional map and weather services are used only when you turn them on in the app.

= Is the published website updated automatically? =

Yes, by default it is republished a few seconds after each change. You can turn this off in the app (Family website).

== Changelog ==

= 3.2.0 =
* New: the plugin is tested on a real WordPress (6.5 and the newest) before every release.
* Fix: the ZIP package has file permissions every hosting can read.

= 3.1.0 =
* New: sources next to every fact (page, reliability, record text), or one list per person, chosen in Settings.
* New: tree: highlight the direct line, colors of the four lines of the grandparents, clan filter; tree colors in Appearance.
* New: Address and Unconfirmed information fields; cause of death in the people list; migrations from one place on the map.
* New: progress bar for photo uploads, GEDCOM import and backup restore.
* Fix: importing a GEDCOM file again no longer duplicates people.
* Fix: big trees (tens of thousands of people) are fast.

= 3.0.0 =
* New: the plugin is the MyRoots 3 app inside WordPress (same look and features as the desktop app).
* New: family website published from the editor, with incremental photo upload.
* New: suggestions from the family arrive in the app automatically.
* New: one-click move from MyRoots 2.x on the same site.
* New: several family trees on one site (MyRoots → Family tree → Family trees), each published at its own address.
* New: password protection of the family website, for all of it or for chosen people and branches, with encryption of the protected data and photos.
* New: the family website and the editor are installable web apps (manifest, service worker for the website).
