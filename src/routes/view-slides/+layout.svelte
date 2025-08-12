<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';
	import { shared } from './shared.svelte';
	import { listen } from '@tauri-apps/api/event';

	let { children }: { children: Snippet } = $props();

	// Watch for changes and save to localStorage
	$effect(() => {
		localStorage.setItem('slides', JSON.stringify(shared.slides));
	});

	onMount(() => {
		setTimeout(() => {
			listen('markdown-updated', (event: any) => {
				shared.slides = JSON.parse(event.payload as string);
			});
		}, 0);

		/**
		 * Prevent default scroll on wheel and touch
		 */
		const preventScroll = (e: Event) => {
			e.preventDefault();
		};

		/**
		 * Handle arrow key navigation
		 */
		const handleArrowKeys = (e: KeyboardEvent) => {
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
