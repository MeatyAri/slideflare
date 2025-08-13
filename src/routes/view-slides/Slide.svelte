<script lang="ts">
	interface Props {
		id: string;
		bgColor: string;
		textColor: string;
		title: string;
		content: string;
	}

	let { id, bgColor, textColor, title, content }: Props = $props();
</script>

<section {id} class="h-screen w-full {bgColor}">
	<div class="h-screen w-full px-24">
		<div class="flex h-screen flex-col items-center justify-center">
			<article class="prose lg:prose-xl prose-invert {textColor}">{@html content}</article>
		</div>
	</div>
</section>
