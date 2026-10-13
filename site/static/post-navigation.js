/** Navigate full posts with j/k without interfering with editing controls. */
(function () {
	const posts = Array.from(
		document.querySelectorAll('main > article[data-post-id]'),
	);
	if (!posts.length) return;

	const topGap = 8;
	const tolerance = 0.5;
	const controls =
		'input, textarea, select, button, a, summary, video, audio, dialog, ' +
		'[contenteditable]:not([contenteditable="false"]), [role="button"], ' +
		'[role="textbox"], [role="combobox"], [role="listbox"], [role="slider"], ' +
		'[role="spinbutton"], [role="grid"], [role="menu"], [role="application"]';
	let selected = -1;
	let selectedScrollY = null;

	/** Preserve text entry and keyboard interactions, including shadow editors. */
	function isControl(target) {
		return Boolean(
			target &&
				(target.isContentEditable ||
					(typeof target.closest === 'function' && target.closest(controls))),
		);
	}

	/** Locate the next heading after manual scrolling or before the first jump. */
	function headingInDirection(direction) {
		for (
			let i = direction > 0 ? 0 : posts.length - 1;
			i >= 0 && i < posts.length;
			i += direction
		) {
			const distance = posts[i].getBoundingClientRect().top - topGap;
			if (distance * direction > tolerance) return i;
		}
		return -1;
	}

	/** Scroll instantly and focus the destination without a second focus scroll. */
	function selectPost(index) {
		window.scrollTo({
			top: Math.max(
				0,
				posts[index].getBoundingClientRect().top + window.scrollY - topGap,
			),
			behavior: 'instant',
		});
		posts[index].focus({ preventScroll: true });
		selected = index;
		selectedScrollY = window.scrollY;
	}

	// Cross-page navigation carries its landing position in the URL, not storage.
	if (window.location.hash === '#post-first') selectPost(0);
	else if (window.location.hash === '#post-last') selectPost(posts.length - 1);

	document.addEventListener('keydown', function (event) {
		if (
			(event.key !== 'j' && event.key !== 'k') ||
			event.defaultPrevented ||
			event.repeat ||
			event.isComposing ||
			event.keyCode === 229 ||
			event.altKey ||
			event.ctrlKey ||
			event.metaKey ||
			event.shiftKey
		)
			return;
		const path = event.composedPath ? event.composedPath() : [event.target];
		if (
			path.some(isControl) ||
			isControl(document.activeElement) ||
			document.querySelector(
				'dialog[open], [role="dialog"][aria-modal="true"]:not([hidden])',
			)
		)
			return;

		const direction = event.key === 'j' ? 1 : -1;
		// Remember the actual scroll offset: the last heading may be too close
		// to the page bottom to align with the reading edge of the viewport.
		const index =
			selected >= 0 && window.scrollY === selectedScrollY
				? selected + direction
				: headingInDirection(direction);
		if (index < 0 || index >= posts.length) {
			const link = document.querySelector(
				direction > 0 ? 'a[data-post-next]' : 'a[data-post-prev]',
			);
			if (!link) return;
			event.preventDefault();
			window.location.assign(
				link.href.split('#')[0] +
					(direction > 0 ? '#post-first' : '#post-last'),
			);
			return;
		}

		event.preventDefault();
		selectPost(index);
	});
})();
