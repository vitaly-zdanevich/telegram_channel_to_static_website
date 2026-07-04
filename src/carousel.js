// tg2zola image carousel (opt-in). The horizontal swipe is pure CSS
// (scroll-snap), so it works on touch even without this script; here we add
// prev/next arrows, dots and keep the active dot in sync.
(function () {
	function index(track, slides) {
		let best = 0;
		let min = Number.POSITIVE_INFINITY;
		for (let i = 0; i < slides.length; i++) {
			const d = Math.abs(
				slides[i].offsetLeft - track.offsetLeft - track.scrollLeft,
			);
			if (d < min) {
				min = d;
				best = i;
			}
		}
		return best;
	}

	function setup(root) {
		const track = root.querySelector('.carousel-track');
		if (!track) return;
		const slides = Array.prototype.slice.call(track.children);
		if (slides.length < 2) return;

		function goto(i) {
			const idx = Math.max(0, Math.min(slides.length - 1, i));
			track.scrollTo({
				left: slides[idx].offsetLeft - track.offsetLeft,
				behavior: 'smooth',
			});
		}

		const prev = root.querySelector('.carousel-prev');
		const next = root.querySelector('.carousel-next');
		if (prev) {
			prev.addEventListener('click', function () {
				goto(index(track, slides) - 1);
			});
		}
		if (next) {
			next.addEventListener('click', function () {
				goto(index(track, slides) + 1);
			});
		}

		const dots = document.createElement('div');
		dots.className = 'carousel-dots';
		slides.forEach(function (_slide, i) {
			const dot = document.createElement('button');
			dot.type = 'button';
			dot.setAttribute('aria-label', 'Image ' + (i + 1));
			dot.addEventListener('click', function () {
				goto(i);
			});
			dots.appendChild(dot);
		});
		root.appendChild(dots);

		function sync() {
			const c = index(track, slides);
			Array.prototype.forEach.call(dots.children, function (dot, i) {
				dot.classList.toggle('active', i === c);
			});
		}
		let ticking = false;
		track.addEventListener('scroll', function () {
			if (ticking) return;
			ticking = true;
			window.requestAnimationFrame(function () {
				sync();
				ticking = false;
			});
		});
		sync();
	}

	Array.prototype.forEach.call(document.querySelectorAll('.carousel'), setup);
})();
