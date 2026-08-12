// tg2zola offline service worker (opt-in). On install it precaches the whole
// archive listed in asset-manifest.json — on any non-cellular connection
// (Wi-Fi or wired), never over mobile data. Thereafter it refreshes page
// navigations from the network with an offline fallback, while assets remain
// cache-first and keep filling the cache (again, not on a cellular link).

const CACHE = 'tg2zola-v1';
const MANIFEST = 'asset-manifest.json';

// Best-effort "don't burn mobile data" check: skip on a cellular link or when
// the user has Data Saver on. We deliberately ignore effectiveType (a speed
// estimate) so a slow *wired*/Wi-Fi link still precaches. navigator.connection
// is Chromium-only in workers; where it's missing we can't tell, so we proceed.
function metered() {
	const c =
		self.navigator.connection ||
		self.navigator.mozConnection ||
		self.navigator.webkitConnection;
	if (!c) return false;
	return c.saveData === true || c.type === 'cellular';
}

self.addEventListener('install', function (e) {
	self.skipWaiting();
	if (metered()) return; // don't precache the archive over mobile data
	e.waitUntil(
		fetch(MANIFEST, { cache: 'no-cache' })
			.then(function (r) {
				return r.json();
			})
			.then(function (urls) {
				return caches.open(CACHE).then(function (cache) {
					// Add one at a time so a single 404/large file can't abort the lot.
					return Promise.all(
						urls.map(function (u) {
							return cache.add(u).catch(function () {});
						}),
					);
				});
			})
			.catch(function () {}),
	);
});

self.addEventListener('activate', function (e) {
	e.waitUntil(
		caches
			.keys()
			.then(function (keys) {
				return Promise.all(
					keys
						.filter(function (k) {
							return k !== CACHE;
						})
						.map(function (k) {
							return caches.delete(k);
						}),
				);
			})
			.then(function () {
				return self.clients.claim();
			}),
	);
});

/** Fetch a request and remember a successful same-origin response. */
function fetchAndCache(req) {
	return fetch(req).then(function (resp) {
		if (resp && resp.ok && resp.type === 'basic' && !metered()) {
			const copy = resp.clone();
			caches.open(CACHE).then(function (cache) {
				cache.put(req, copy);
			});
		}
		return resp;
	});
}

/** Refresh an HTML navigation, retaining the cached page for offline use. */
function networkFirst(req) {
	return fetchAndCache(req).catch(function () {
		return caches.match(req);
	});
}

/** Reuse immutable or content-hashed assets before consulting the network. */
function cacheFirst(req) {
	return caches
		.match(req)
		.then(function (hit) {
			if (hit) return hit;
			return fetchAndCache(req);
		})
		.catch(function () {
			return caches.match(req);
		});
}

self.addEventListener('fetch', function (e) {
	const req = e.request;
	if (req.method !== 'GET') return;
	e.respondWith(req.mode === 'navigate' ? networkFirst(req) : cacheFirst(req));
});
