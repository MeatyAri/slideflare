<script lang="ts">
	// import { renderMermaidDiagrams } from '$lib/render_mermaid';

	interface Props {
		id: string;
		bgColor: string;
		textColor: string;
		title: string;
		content: string;
	}

	let { id, bgColor, textColor, title, content }: Props = $props();

	// $effect(() => {
	// 	renderMermaidDiagrams();
	// });
</script>

<section {id} class="h-screen w-full {bgColor} overflow-hidden">
	<div class="flex h-screen w-full flex-col items-center justify-center px-24 py-10">
		<article
			class="prose lg:prose-xl prose-invert flex h-full w-full flex-col justify-center {textColor}"
		>
			{@html content}
		</article>
	</div>
</section>
