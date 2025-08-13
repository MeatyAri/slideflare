<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import { shared } from './view-slides/shared.svelte';

	let { children }: { children: Snippet } = $props();

	listen('markdown-updated', (event: any) => {
		shared.slides = JSON.parse(event.payload as string);
		localStorage.setItem('slides', JSON.stringify(shared.slides));
	});
</script>

{@render children()}
