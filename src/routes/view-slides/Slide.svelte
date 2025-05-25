<script>
	let { id, bgColor, textColor, title, content } = $props();
</script>

<section {id} class="h-screen w-full {bgColor}">
	<div class="h-screen w-full px-24">
		<div class="flex h-screen flex-col items-center justify-center">
			<article class="prose lg:prose-xl dark:prose-invert {textColor}">{@html content}</article>
		</div>
	</div>
</section>
