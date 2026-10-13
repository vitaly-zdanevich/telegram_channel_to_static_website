const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const { join } = require('node:path');
const test = require('node:test');
const vm = require('node:vm');

const source = readFileSync(join(__dirname, 'post-navigation.js'), 'utf8');

/** Create the small DOM surface needed to exercise keyboard event targets. */
function element(tagName = 'DIV', parentElement = null, attributes = {}) {
	return {
		tagName,
		nodeName: tagName,
		parentElement,
		isContentEditable:
			(attributes.contenteditable !== undefined &&
				attributes.contenteditable !== 'false') ||
			Boolean(parentElement?.isContentEditable),
		getAttribute(name) {
			return attributes[name] ?? null;
		},
		closest(selectors) {
			for (let current = this; current; current = current.parentElement) {
				for (const selector of selectors
					.split(',')
					.map((item) => item.trim())) {
					if (selector.toUpperCase() === current.tagName) return current;
					if (
						selector === 'a[href]' &&
						current.tagName === 'A' &&
						current.getAttribute('href')
					)
						return current;
					if (
						selector.startsWith('[contenteditable') &&
						current.isContentEditable
					)
						return current;
					const role = selector.match(/^\[role="([a-z]+)"\]$/)?.[1];
					if (role && current.getAttribute('role') === role) return current;
				}
			}
			return null;
		},
	};
}

/** Evaluate the production script with deterministic article positions and scrolling. */
function page({
	positions = [100, 600, 1100],
	scrollY = 0,
	maxScroll = Number.POSITIVE_INFINITY,
	modal = false,
	next,
	previous,
	hash = '',
} = {}) {
	const listeners = new Map();
	const scrolls = [];
	const focused = [];
	const navigations = [];
	const body = element('BODY');
	const dialog = modal ? element('DIALOG') : null;
	const document = {
		activeElement: body,
		body,
		querySelectorAll(selector) {
			assert.equal(selector, 'main > article[data-post-id]');
			return posts;
		},
		querySelector(selector) {
			if (selector === 'a[data-post-next]') return next ? { href: next } : null;
			if (selector === 'a[data-post-prev]')
				return previous ? { href: previous } : null;
			if (
				selector ===
				'dialog[open], [role="dialog"][aria-modal="true"]:not([hidden])'
			)
				return dialog;
			throw new Error(`Unexpected document selector: ${selector}`);
		},
		addEventListener(type, listener) {
			listeners.set(type, listener);
		},
	};
	const window = {
		scrollY,
		location: {
			hash,
			assign(url) {
				navigations.push(url);
			},
		},
		scrollTo(options) {
			scrolls.push({ top: options.top, behavior: options.behavior });
			this.scrollY = Math.max(0, Math.min(maxScroll, options.top));
		},
	};
	const posts = positions.map((position, index) => ({
		...element('ARTICLE'),
		id: `post-${index + 1}`,
		dataset: { postId: String(index + 1) },
		getBoundingClientRect() {
			return { top: position - window.scrollY };
		},
		focus(options) {
			assert.equal(options.preventScroll, true);
			focused.push(this.id);
			document.activeElement = this;
		},
	}));
	vm.runInNewContext(
		source,
		{ document, window },
		{ filename: 'post-navigation.js' },
	);

	return {
		document,
		focused,
		navigations,
		hasKeyListener: listeners.has('keydown'),
		scrolls,
		window,
		key(key, properties = {}) {
			const event = {
				key,
				target: document.activeElement,
				altKey: false,
				ctrlKey: false,
				metaKey: false,
				shiftKey: false,
				repeat: false,
				isComposing: false,
				defaultPrevented: false,
				preventDefault() {
					this.defaultPrevented = true;
				},
				...properties,
			};
			listeners.get('keydown')?.(event);
			return event;
		},
	};
}

test('hotkeys activate only on marked full-post listings, including a one-post final page', () => {
	const subject = page({ positions: [] });
	assert.equal(subject.hasKeyListener, false);
	assert.equal(subject.key('j').defaultPrevented, false);
	assert.deepEqual(subject.scrolls, []);
	assert.equal(page().hasKeyListener, true);
	assert.equal(page({ positions: [100] }).hasKeyListener, true);
});

test('j and k scroll through posts in display order and move keyboard focus', () => {
	const subject = page();
	assert.equal(subject.key('j').defaultPrevented, true);
	subject.key('j');
	subject.key('k');
	assert.deepEqual(subject.focused, ['post-1', 'post-2', 'post-1']);
	assert.deepEqual(subject.scrolls, [
		{ top: 92, behavior: 'instant' },
		{ top: 592, behavior: 'instant' },
		{ top: 92, behavior: 'instant' },
	]);
});

test('the first j selects the first heading below the viewport reading edge', () => {
	const subject = page({ scrollY: 91 });
	subject.key('j');
	assert.deepEqual(subject.focused, ['post-1']);
});

test('at a heading, j selects the following post and k selects the preceding post', () => {
	const next = page({ scrollY: 592 });
	next.key('j');
	assert.deepEqual(next.focused, ['post-3']);
	const previous = page({ scrollY: 592 });
	previous.key('k');
	assert.deepEqual(previous.focused, ['post-1']);
});

test('partway through a post, j advances while k returns to its heading', () => {
	const next = page({ scrollY: 350 });
	next.key('j');
	assert.deepEqual(next.focused, ['post-2']);
	const previous = page({ scrollY: 350 });
	previous.key('k');
	assert.deepEqual(previous.focused, ['post-1']);
});

test('navigation does not wrap or consume keys at either end', () => {
	const first = page({ scrollY: 92 });
	assert.equal(first.key('k').defaultPrevented, false);
	assert.deepEqual(first.scrolls, []);
	const last = page({ scrollY: 1092 });
	assert.equal(last.key('j').defaultPrevented, false);
	assert.deepEqual(last.scrolls, []);
});

test('j at the last post opens the next page at its first post', () => {
	const subject = page({
		scrollY: 1092,
		next: 'https://example.test/channel/page/3/',
	});
	assert.equal(subject.key('j').defaultPrevented, true);
	assert.deepEqual(subject.navigations, [
		'https://example.test/channel/page/3/#post-first',
	]);
});

test('k at the first post opens the previous page at its last post', () => {
	const subject = page({
		scrollY: 92,
		previous: 'https://example.test/channel/?lang=en#old-fragment',
	});
	assert.equal(subject.key('k').defaultPrevented, true);
	assert.deepEqual(subject.navigations, [
		'https://example.test/channel/?lang=en#post-last',
	]);
});

test('pagination works on a listing page containing just one post and in offline copies', () => {
	const subject = page({
		positions: [100],
		scrollY: 92,
		previous: 'file:///archive/page/2/index.html',
	});
	assert.equal(subject.key('j').defaultPrevented, false);
	assert.equal(subject.key('k').defaultPrevented, true);
	assert.deepEqual(subject.navigations, [
		'file:///archive/page/2/index.html#post-last',
	]);
});

test('arrival fragments select the correct post and retain position for the next key', () => {
	const first = page({ hash: '#post-first' });
	assert.deepEqual(first.focused, ['post-1']);
	first.key('j');
	assert.deepEqual(first.focused, ['post-1', 'post-2']);
	const last = page({ hash: '#post-last', maxScroll: 700 });
	assert.deepEqual(last.focused, ['post-3']);
	last.key('k');
	assert.deepEqual(last.focused, ['post-3', 'post-2']);
	assert.deepEqual(page({ hash: '#unrelated' }).scrolls, []);
});

test('j follows pagination after selecting a bottom-clamped last post', () => {
	const subject = page({
		maxScroll: 700,
		next: 'https://example.test/page/2/',
	});
	subject.key('j');
	subject.key('j');
	subject.key('j');
	assert.deepEqual(subject.navigations, []);
	subject.key('j');
	assert.deepEqual(subject.navigations, [
		'https://example.test/page/2/#post-first',
	]);
});

test('bottom-clamped scrolling remembers the selected post for the next key', () => {
	const subject = page({ maxScroll: 700 });
	subject.key('j');
	subject.key('j');
	subject.key('j');
	assert.equal(subject.window.scrollY, 700);
	assert.equal(subject.key('j').defaultPrevented, false);
	subject.key('k');
	assert.deepEqual(subject.focused, ['post-1', 'post-2', 'post-3', 'post-2']);
});

test('manual scrolling invalidates the previously selected post', () => {
	const subject = page();
	subject.key('j');
	subject.key('j');
	subject.window.scrollY = 350;
	subject.key('k');
	assert.deepEqual(subject.focused, ['post-1', 'post-2', 'post-1']);
});

test('modified, repeated, composing, handled, and unrelated keys are untouched', () => {
	const subject = page();
	for (const property of [
		'altKey',
		'ctrlKey',
		'metaKey',
		'shiftKey',
		'repeat',
		'isComposing',
		'defaultPrevented',
	]) {
		subject.key('j', { [property]: true });
	}
	subject.key('j', { keyCode: 229 });
	for (const key of ['J', 'K', 'n', 'o', 'ArrowDown', ' ']) {
		assert.equal(subject.key(key).defaultPrevented, false);
	}
	assert.deepEqual(subject.scrolls, []);
});

test('editing fields and interactive controls retain their own keyboard behavior', () => {
	const subject = page();
	const controls = [
		...[
			'INPUT',
			'TEXTAREA',
			'SELECT',
			'BUTTON',
			'SUMMARY',
			'AUDIO',
			'VIDEO',
		].map((tag) => element(tag)),
		element('A', null, { href: '/posts/1/' }),
		element('DIV', null, { contenteditable: 'true' }),
		element('DIV', null, { contenteditable: 'plaintext-only' }),
		...[
			'button',
			'textbox',
			'combobox',
			'listbox',
			'slider',
			'spinbutton',
			'grid',
			'menu',
			'application',
		].map((role) => element('DIV', null, { role })),
		element('DIALOG'),
	];
	for (const control of controls) {
		assert.equal(
			subject.key('j', { target: control }).defaultPrevented,
			false,
			control.tagName,
		);
		assert.equal(
			subject.key('k', { target: element('SPAN', control) }).defaultPrevented,
			false,
			`${control.tagName} descendant`,
		);
	}
	assert.deepEqual(subject.scrolls, []);
});

test('an open dialog suppresses navigation even if the event target is outside it', () => {
	const subject = page({ modal: true });
	assert.equal(subject.key('j').defaultPrevented, false);
	assert.deepEqual(subject.scrolls, []);
});

test('editable targets hidden by shadow DOM event retargeting keep their keys', () => {
	const subject = page();
	const host = element('DIV');
	const input = element('INPUT');
	assert.equal(
		subject.key('j', {
			target: host,
			composedPath: () => [input, host],
		}).defaultPrevented,
		false,
	);
	assert.deepEqual(subject.scrolls, []);
});

test('a focused input retains its keys even when the event target is retargeted', () => {
	const subject = page();
	subject.document.activeElement = element('INPUT');
	assert.equal(
		subject.key('j', { target: element('DIV') }).defaultPrevented,
		false,
	);
	assert.deepEqual(subject.scrolls, []);
});

test('explicitly noneditable prose can still use navigation shortcuts', () => {
	const subject = page();
	subject.key('j', {
		target: element('DIV', null, { contenteditable: 'false' }),
	});
	assert.deepEqual(subject.focused, ['post-1']);
});

test('short pages still change focused posts when all scrolling is clamped', () => {
	const subject = page({ positions: [0, 50, 100], maxScroll: 0 });
	subject.key('j');
	subject.key('j');
	subject.key('k');
	subject.key('k');
	assert.deepEqual(subject.focused, ['post-2', 'post-3', 'post-2', 'post-1']);
	assert.equal(subject.window.scrollY, 0);
	assert.equal(subject.scrolls.at(-1).top, 0);
});
