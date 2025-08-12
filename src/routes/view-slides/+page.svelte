<script lang="ts">
	import katex from 'katex';
	import NavBar from './NavBar.svelte';
	import Slide from './Slide.svelte';
	import { shared } from './shared.svelte';

	/**
	 * Renders KaTeX only on math blocks with the 'math' class
	 */
	function renderMathBlocks(content: string): string {
		// Use a regular expression to find math blocks with the 'math' class
		const mathBlockRegex = /<[^>]*class="[^"]*\bmath\b[^"]*"[^>]*>([\s\S]*?)<\/[^>]*>/g;
		return content.replace(mathBlockRegex, (match: string, p1: string) => {
			// Render the math content with KaTeX
			return `<span class="math">${katex.renderToString(p1.trim())}</span>`;
		});
	}
</script>

<NavBar />

<main class="flex flex-col items-center justify-center">
	{#each shared.slides as slide, index}
		<Slide
			id={String(index)}
			bgColor={slide.bgColor}
			textColor={slide.textColor}
			title={slide.title}
			content={renderMathBlocks(slide.content)}
		/>
	{/each}
</main>
