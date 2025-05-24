<script>
	// @ts-nocheck

	import NavBar from './NavBar.svelte';
	import Slide from './Slide.svelte';
	import { shared } from './shared.svelte.js';
</script>

<NavBar />

<main class="flex flex-col items-center justify-center">
	{#each shared.slides as slide, index}
		<Slide
			id={index}
			bgColor={slide.bgColor}
			textColor={slide.textColor}
			title={slide.title}
			content={slide.content}
		/>
	{/each}
</main>
