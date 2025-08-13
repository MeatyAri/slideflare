<script lang="ts">
	import { onDestroy } from 'svelte';

	interface Props {
		id: string;
		bgColor: string;
		textColor: string;
		title: string;
		content: string;
	}

	let { id, bgColor, textColor, title, content }: Props = $props();

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

<section {id} class="h-screen w-full {bgColor}">
	<div class="h-screen w-full px-24">
		<div class="flex h-screen flex-col items-center justify-center">
			<article class="prose lg:prose-xl dark:prose-invert {textColor}">{@html content}</article>
		</div>
	</div>
</section>
