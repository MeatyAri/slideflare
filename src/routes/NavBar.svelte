<script>
	import slides from './slides.json';
	import { activeIndex } from './shared.svelte.js';

	/**
	 * @param {number} index
	 */
	const handleClick = (index) => {
		const slide = document.getElementById(String(index));
		slide?.scrollIntoView({ behavior: 'smooth' });
		activeIndex.index = index;
	};
</script>

<nav
	class="fixed top-0 left-0 z-50 flex h-full w-20 flex-col items-center justify-center bg-gradient-to-l to-gray-900/30 py-8"
>
	<div class="relative z-10 flex w-full flex-col items-center pr-5">
		{#if slides.length > 0}
			<div
				class="pointer-events-none absolute z-20 transition-[top] duration-300"
				style="top: {activeIndex.index * 3}rem;"
			>
				<span
					class="block h-4 w-4 scale-125 rounded-full border-2 border-blue-500 bg-blue-500 shadow-lg"
				></span>
			</div>
		{/if}
		{#each slides as slide, index}
			<button
				class="group relative flex h-12 flex-col items-center focus:outline-none"
				aria-label={`Go to slide ${index + 1}`}
				onclick={() => handleClick(index)}
			>
				<span
					class="
                        h-4 w-4 rounded-full border-2 border-gray-500 bg-gray-700 transition-all"
				></span>
				{#if index < slides.length - 1}
					<span class="h-10 w-1 bg-gray-600"></span>
				{/if}
				<span
					class="pointer-events-none absolute top-1/2 left-8 -translate-y-1/2 text-xs whitespace-nowrap text-gray-200 opacity-0 transition-opacity group-hover:opacity-100"
				>
					{slide.title}
				</span>
			</button>
		{/each}
	</div>
</nav>
