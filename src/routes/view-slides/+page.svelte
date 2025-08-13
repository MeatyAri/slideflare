<script lang="ts">
	import NavBar from './NavBar.svelte';
	import Slide from './Slide.svelte';
	import { shared } from './shared.svelte';
	import { onDestroy } from 'svelte';

	onDestroy(() => {
		// refresh the page when the component is destroyed
		window.location.reload();
	});
</script>

<svelte:head>
	<script src="../tailwind.min.js"></script>
	<!-- Include the Stylesheet for math -->
	<!-- <link
		rel="stylesheet"
		href="https://cdn.jsdelivr.net/gh/carloskiki/pulldown-latex@latest/styles.min.css"
	/> -->
	<!-- Include the Fonts for math -->
	<!-- <link
		rel="preload"
		href="https://cdn.jsdelivr.net/gh/carloskiki/pulldown-latex@latest/font/"
		as="font"
		crossorigin="anonymous"
	/> -->
</svelte:head>

<NavBar />

<main class="flex flex-col items-center justify-center select-none">
	{#each shared.slides as slide, index}
		<Slide
			id={String(index)}
			bgColor={slide.bg_color}
			textColor={slide.text_color}
			title={slide.title}
			content={slide.content}
		/>
	{/each}
</main>
