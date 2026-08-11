const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const test = require('node:test');
const vm = require('node:vm');

const source = readFileSync(join(__dirname, 'sw.js'), 'utf8');

function response(name) {
	return {
		name,
		ok: true,
		type: 'basic',
		clone() {
			return this;
		},
	};
}

/**
 * Evaluate the service worker with deterministic network and Cache API mocks.
 */
function worker({ cachedResponse, networkResponse, networkError }) {
	const calls = [];
	const listeners = new Map();
	const context = {
		self: {
			navigator: {},
			clients: { claim: () => Promise.resolve() },
			skipWaiting() {},
			addEventListener(type, listener) {
				listeners.set(type, listener);
			},
		},
		caches: {
			match(request) {
				calls.push(`cache:${request.url}`);
				return Promise.resolve(cachedResponse);
			},
			open() {
				return Promise.resolve({
					add: () => Promise.resolve(),
					put: () => Promise.resolve(),
				});
			},
			keys: () => Promise.resolve([]),
			delete: () => Promise.resolve(true),
		},
		fetch(request) {
			calls.push(`fetch:${request.url}`);
			if (networkError) return Promise.reject(networkError);
			return Promise.resolve(networkResponse);
		},
	};

	vm.runInNewContext(source, context, { filename: 'sw.js' });

	return {
		calls,
		request(mode) {
			let result;
			listeners.get('fetch')({
				request: { method: 'GET', mode, url: 'https://example.test/' },
				respondWith(promise) {
					result = promise;
				},
			});
			return result;
		},
	};
}

test('page navigation refreshes stale cached HTML', async () => {
	const cachedResponse = response('stale cache');
	const networkResponse = response('fresh network');
	const subject = worker({ cachedResponse, networkResponse });

	assert.equal(await subject.request('navigate'), networkResponse);
	assert.deepEqual(subject.calls, ['fetch:https://example.test/']);
});

test('page navigation falls back to cached HTML while offline', async () => {
	const cachedResponse = response('offline cache');
	const subject = worker({
		cachedResponse,
		networkResponse: undefined,
		networkError: new Error('offline'),
	});

	assert.equal(await subject.request('navigate'), cachedResponse);
	assert.deepEqual(subject.calls, [
		'fetch:https://example.test/',
		'cache:https://example.test/',
	]);
});

test('non-navigation assets remain cache-first', async () => {
	const cachedResponse = response('cached asset');
	const subject = worker({
		cachedResponse,
		networkResponse: response('network asset'),
	});

	assert.equal(await subject.request('no-cors'), cachedResponse);
	assert.deepEqual(subject.calls, ['cache:https://example.test/']);
});
