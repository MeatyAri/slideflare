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
</svelte:head>

<section {id} class="h-screen w-full {bgColor}">
	<div class="h-screen w-full px-24">
		<div class="flex h-screen flex-col items-center justify-center">
			<article class="prose lg:prose-xl dark:prose-invert {textColor}">{@html content}</article>
		</div>
	</div>
</section>
