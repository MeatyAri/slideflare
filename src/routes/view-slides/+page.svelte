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

<NavBar />

<main class="flex flex-col items-center justify-center">
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
