<script>
	import { onDestroy } from 'svelte';

	let { id, bgColor, textColor, title, content } = $props();

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
			<article class="prose lg:prose-xl dark:prose-invert {textColor}">
				{#if typeof content === 'string'}
					<p>{@html content}</p>
				{:else}
					{@html content}
				{/if}
			</article>
		</div>
	</div>
</section>
