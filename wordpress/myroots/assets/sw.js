/* MyRoots: service worker of a family website published by WordPress.
   C (cache name, start page, app and media paths) is printed above by the plugin. */
self.addEventListener('install', (e) => {
	self.skipWaiting();
	e.waitUntil(caches.open(C.cache).then((c) => c.add(C.start)).catch(() => {}));
});

self.addEventListener('activate', (e) => {
	e.waitUntil(
		caches
			.keys()
			.then((keys) => Promise.all(keys.filter((k) => k.startsWith(C.prefix) && k !== C.cache).map((k) => caches.delete(k))))
			.then(() => self.clients.claim())
	);
});

function fromNetwork(req) {
	return fetch(req).then((r) => {
		if (r.ok && r.type === 'basic') {
			const copy = r.clone();
			caches.open(C.cache).then((c) => c.put(req, copy));
		}
		return r;
	});
}

self.addEventListener('fetch', (e) => {
	const req = e.request;
	if (req.method !== 'GET') return;
	const url = new URL(req.url);
	if (url.origin !== location.origin) return;
	const full = url.pathname + url.search;
	// pages of the editor and the server (login, settings, API) are never kept
	if ((C.skip || []).some((p) => full.startsWith(p))) return;
	// files of the app (names with a hash) and photos: from the cache first
	if (url.pathname.startsWith(C.app) || full.startsWith(C.media)) {
		e.respondWith(caches.match(req).then((hit) => hit || fromNetwork(req)));
		return;
	}
	// the page and its data: the newest version, the saved one offline
	if (req.mode === 'navigate' || /data\.js/.test(full)) {
		e.respondWith(
			fromNetwork(req).catch(() =>
				caches.match(req, { ignoreSearch: req.mode !== 'navigate' }).then((hit) => hit || caches.match(C.start))
			)
		);
	}
});
