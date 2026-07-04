// tg2zola iframe embed helper (opt-in). A cross-origin iframe can't size
// itself to the host page, so when we're framed we post our height to the
// parent window; the host listens for `tg2zola:height` and resizes the iframe.
// We also tag <html> with an `embedded` class so a fork can hide site chrome.
(function () {
	if (window.parent === window) {
		return;
	}
	const root = document.documentElement;
	root.classList.add('embedded');
	let last = 0;
	function report() {
		const height = Math.ceil(root.getBoundingClientRect().height);
		if (height !== last) {
			last = height;
			window.parent.postMessage({ type: 'tg2zola:height', height }, '*');
		}
	}
	addEventListener('load', report);
	addEventListener('resize', report);
	if (typeof ResizeObserver === 'function') {
		new ResizeObserver(report).observe(document.body);
	}
	report();
})();
