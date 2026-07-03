(function () {
	const input = document.getElementById('site-search');
	const results = document.getElementById('search-results');
	if (!input || !results || !window.elasticlunr || !window.searchIndex) return;
	const index = elasticlunr.Index.load(window.searchIndex);
	function esc(s) {
		return String(s).replace(/[&<>"']/g, function (c) {
			return {
				'&': '&amp;',
				'<': '&lt;',
				'>': '&gt;',
				'"': '&quot;',
				"'": '&#39;',
			}[c];
		});
	}
	function snippet(s) {
		const t = String(s).replace(/\s+/g, ' ').trim();
		return t.length > 150 ? t.slice(0, 150) + '…' : t;
	}
	function render(items) {
		if (!items.length) {
			results.innerHTML = '';
			results.hidden = true;
			return;
		}
		results.innerHTML = items
			.map(function (it) {
				const doc = index.documentStore.getDoc(it.ref) || {};
				const body = doc.body && doc.body.trim();
				const text = body
					? snippet(body)
					: doc.title && doc.title.trim()
						? doc.title
						: it.ref;
				return '<li><a href="' + esc(it.ref) + '">' + esc(text) + '</a></li>';
			})
			.join('');
		results.hidden = false;
	}
	let t;
	input.addEventListener('input', function () {
		clearTimeout(t);
		t = setTimeout(function () {
			const term = input.value.trim();
			if (!term) {
				render([]);
				return;
			}
			const found = index
				.search(term, { bool: 'AND', expand: true })
				.slice(0, 10);
			render(found);
		}, 150);
	});
	document.addEventListener('click', function (e) {
		if (e.target !== input && !results.contains(e.target))
			results.hidden = true;
	});
	input.addEventListener('keydown', function (e) {
		if (e.key === 'Escape') results.hidden = true;
	});
})();
