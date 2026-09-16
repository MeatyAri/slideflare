<script lang="ts">
  import { shared } from './shared.svelte';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  interface Props {
    onExportHtml: () => void;
    onExportPdf: () => void;
    exportDisabled?: boolean;
  }

  let { onExportHtml, onExportPdf, exportDisabled = false }: Props = $props();

  function goBack() {
    goto(resolve('/'));
  }

  let backVisible = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;

  function revealBack() {
    backVisible = true;
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => (backVisible = false), 2500);
  }

  function handleMouseMove(e: MouseEvent) {
    // Reveal when cursor is near the left edge (within 120px).
    if (e.clientX <= 120) revealBack();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') goBack();
  }

  const handleClick = (index: number) => {
    const slide = document.getElementById(String(index));
    slide?.scrollIntoView({ behavior: 'smooth' });
    shared.index = index;
  };

  function handleResize(): void {
    const slide = document.getElementById(String(shared.index));
    slide?.scrollIntoView({ behavior: 'instant' });
  }

  let windowInnerHeight = $state(0);

  let navOffsetTop = $derived.by(() => {
    return windowInnerHeight * 0.5 - shared.index * 48;
  });

  onMount(() => {
    window.addEventListener('resize', handleResize);
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('keydown', handleKeydown);
    handleResize();

    return () => {
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('keydown', handleKeydown);
      clearTimeout(hideTimer);
    };
  });
</script>

<svelte:window bind:innerHeight={windowInnerHeight} />

<nav
  class="fixed top-0 left-0 z-50 flex h-full w-20 flex-col items-center bg-gradient-to-l to-gray-900/30 py-8"
  onmouseenter={revealBack}
>
  <!-- Revealed together with the back button, on the same left-edge hover, so
       nothing overlays the deck while presenting. -->
  <div
    class="absolute top-4 z-50 mr-5 flex flex-col items-center gap-2 transition-all duration-300
      {backVisible ? 'opacity-100' : 'pointer-events-none opacity-0'}"
  >
    <button
      class="flex h-9 w-9 items-center justify-center rounded-full border border-gray-600 bg-gray-800/80 text-gray-200 shadow-lg backdrop-blur transition-colors hover:bg-gray-700 focus:outline-none"
      aria-label="Back to file selection"
      title="Back (Esc)"
      onclick={goBack}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
        <path
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M19 12H5m0 0l6 6m-6-6l6-6"
        />
      </svg>
    </button>

    <button
      class="flex h-9 w-9 items-center justify-center rounded-full border border-gray-600 bg-gray-800/80 text-gray-200 shadow-lg backdrop-blur transition-colors hover:bg-gray-700 focus:outline-none disabled:cursor-not-allowed disabled:opacity-40"
      aria-label="Export deck to PDF"
      title="Export to PDF"
      disabled={exportDisabled}
      onclick={onExportPdf}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
        <g fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M14 3v4a1 1 0 0 0 1 1h4" />
          <path
            stroke-linejoin="round"
            d="M19 10v9a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7l5 5z"
          />
          <path d="M9 14h6m-6 3h4" />
        </g>
      </svg>
    </button>

    <button
      class="flex h-9 w-9 items-center justify-center rounded-full border border-gray-600 bg-gray-800/80 text-gray-200 shadow-lg backdrop-blur transition-colors hover:bg-gray-700 focus:outline-none disabled:cursor-not-allowed disabled:opacity-40"
      aria-label="Export deck to a self-contained HTML file"
      title="Export to HTML"
      disabled={exportDisabled}
      onclick={onExportHtml}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24">
        <g
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="m9 9l-3 3l3 3m6-6l3 3l-3 3" />
          <rect x="3" y="4" width="18" height="16" rx="2" />
        </g>
      </svg>
    </button>
  </div>
  <div
    class="absolute transition-all delay-500 duration-300 ease-out"
    style="top: {navOffsetTop}px;"
  >
    <div class="relative z-10 flex w-full flex-col items-center pr-5">
      {#if shared.slides.length > 0}
        <div
          class="pointer-events-none absolute z-20 transition-[top] duration-300"
          style="top: {shared.index * 3}rem;"
        >
          <span
            class="block h-4 w-4 scale-125 rounded-full border-2 border-blue-500 bg-blue-500 shadow-lg"
          ></span>
        </div>
      {/if}
      {#each shared.slides as slide, index (index)}
        <button
          class="
                        group relative flex flex-col items-center transition-all focus:outline-none
                        {shared.index === index ? 'h-18' : 'h-12'}
                    "
          aria-label={`Go to slide ${index + 1}`}
          onclick={() => handleClick(index)}
        >
          <span
            class="
                            h-4 w-4 rounded-full border-2 border-gray-500 bg-gray-700 transition-all"
          ></span>
          {#if index < shared.slides.length - 1}
            <span
              class="
                            w-1 bg-gray-600 transition-all
                            {shared.index === index ? 'h-14' : 'h-8'}
                        "
            ></span>
          {/if}
          <span
            class="pointer-events-none absolute top-1/2 left-8 -translate-y-1/2 text-xs whitespace-nowrap text-gray-200 opacity-0 transition-opacity group-hover:opacity-100"
          >
            {slide.title}
          </span>
        </button>
      {/each}
    </div>
  </div>
</nav>
