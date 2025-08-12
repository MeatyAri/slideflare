<script>
	// @ts-nocheck
	import katex from 'katex';
	import NavBar from './NavBar.svelte';
	import Slide from './Slide.svelte';
	import { shared } from './shared.svelte.js';

	/**
	 * Renders KaTeX only on math blocks with the 'math' class
	 * @param {string} content - The content to process
	 * @returns {string} - The processed content with rendered math blocks
	 */
	function renderMathBlocks(content) {
		// Use a regular expression to find math blocks with the 'math' class
		const mathBlockRegex = /<[^>]*class="[^"]*\bmath\b[^"]*"[^>]*>([\s\S]*?)<\/[^>]*>/g;
		return content.replace(mathBlockRegex, (match, p1) => {
			// Render the math content with KaTeX
			return `<span class="math">${katex.renderToString(p1.trim())}</span>`;
		});
	}
</script>

<NavBar />

<main class="flex flex-col items-center justify-center">
	{#each shared.slides as slide, index}
		<Slide
			id={index}
			bgColor={slide.bgColor}
			textColor={slide.textColor}
			title={slide.title}
			content={renderMathBlocks(slide.content)}
		/>
	{/each}
</main>
