<script>
	let { id, bgColor, textColor, title, content } = $props();
</script>

<section {id} class="h-screen w-full {bgColor} {textColor}">
	<div class="h-screen w-full px-24">
		<div class="flex h-screen flex-col items-center justify-center">
			<h1 class="text-4xl font-bold">{title}</h1>
			<p class="mt-4 text-lg">{content}</p>
		</div>
	</div>
</section>
