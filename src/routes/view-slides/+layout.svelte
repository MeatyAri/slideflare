<script>
	import { onMount } from 'svelte';
	import { activeIndex } from './shared.svelte.js';
	import slides from './slides.json';

	let { children } = $props();

	onMount(() => {
        /**
         * Prevent default scroll on wheel and touch
         * @param {WheelEvent | TouchEvent} e
         */
		const preventScroll = (e) => {
			e.preventDefault();
		};

        /**
         * Handle arrow key navigation
         * @param {KeyboardEvent} e
         */
        const handleArrowKeys = (e) => {
			e.preventDefault();
			if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
				activeIndex.index -= 1;
				if (activeIndex.index < 0) {
					activeIndex.index = 0;
				}
			}
			if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
				activeIndex.index += 1;
				if (activeIndex.index >= slides.length) {
					activeIndex.index = slides.length - 1;
				}
			}

			const slide = document.getElementById(String(activeIndex.index));
			slide?.scrollIntoView({ behavior: 'smooth' });
		};

		window.addEventListener('wheel', preventScroll, { passive: false });
		window.addEventListener('touchmove', preventScroll, { passive: false });
		window.addEventListener('keydown', handleArrowKeys, { passive: false });

		return () => {
			window.removeEventListener('wheel', preventScroll);
			window.removeEventListener('touchmove', preventScroll);
			window.removeEventListener('keydown', handleArrowKeys);
		};
	});
</script>

{@render children()}
