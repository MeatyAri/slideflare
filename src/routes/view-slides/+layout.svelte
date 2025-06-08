<script>
	import { onMount } from 'svelte';
	import { shared } from './shared.svelte.js';
	import { listen } from '@tauri-apps/api/event';

	let { children } = $props();

	// Watch for changes and save to localStorage
	$effect(() => {
		localStorage.setItem('slides', JSON.stringify(shared.slides));
	});

	onMount(() => {
		setTimeout(() => {
			listen('markdown-updated', (event) => {
				shared.slides = JSON.parse(event.payload);
			});
		}, 0);

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
            let shouldScroll = true;
			if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
				shared.index -= 1;
				if (shared.index < 0) {
					shared.index = 0;
                    shouldScroll = false;
				}
			}
			if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
				shared.index += 1;
				if (shared.index >= shared.slides.length) {
					shared.index = shared.slides.length - 1;
                    shouldScroll = false;
				}
			}

            if (!shouldScroll) return;
            // Scroll to the current slide
			const slide = document.getElementById(String(shared.index));
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
