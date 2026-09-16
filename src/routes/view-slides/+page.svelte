<script lang="ts">
  import NavBar from './NavBar.svelte';
  import Slide from './Slide.svelte';
  import ErrorScreen from './ErrorScreen.svelte';
  import { webview } from '@tauri-apps/api';
  import { listen } from '@tauri-apps/api/event';
  import { shared, type ParseError, DESIGN_W, DESIGN_H, VIEWPORT_PADDING } from './shared.svelte';
  import { onDestroy, onMount } from 'svelte';
  import { createExport } from '$lib/export/export.svelte';
  import ExportOverlay from '$lib/export/ExportOverlay.svelte';

  const exporter = createExport();

  // Track real window size. Scaling is WIDTH-driven: the fixed design width
  // (DESIGN_W) is scaled to fill the window width (minus a tiny padding), so
  // text size is identical at any resolution. maxHeight is passed to each slide
  // so its overflow guard only shrinks content that would exceed the viewport.
  let winW = $state(typeof window !== 'undefined' ? window.innerWidth : DESIGN_W);
  let winH = $state(typeof window !== 'undefined' ? window.innerHeight : DESIGN_H);
  // While exporting to PDF the slide is measured against the page rather than
  // the window: the print stylesheet sizes each section to exactly DESIGN_W by
  // DESIGN_H, so the width scale is 1 and only the overflow guard below can
  // still shrink content. Using the window size here instead would scale every
  // slide by the ratio between the window and the paper.
  let scale = $derived(shared.printMode ? 1 : (winW - 2 * VIEWPORT_PADDING) / DESIGN_W);
  let maxHeight = $derived(shared.printMode ? DESIGN_H : winH - 2 * VIEWPORT_PADDING);

  // Natural (design-size) content height of each slide, reported by the slides.
  // We derive ONE shrink factor from the tallest slide and apply it to every
  // slide, so text size stays identical across the deck AND no slide overflows.
  let heights = $state<Record<number, number>>({});
  const reportHeight = (index: number, h: number) => {
    if (heights[index] !== h) heights[index] = h;
  };
  // Drop stale measurements when the deck changes (e.g. navigating back and
  // opening a different file) so fitScale recalculates for the new slides
  // instead of reusing the previous deck's scaling factor.
  $effect(() => {
    const count = shared.slides.length;
    for (const key of Object.keys(heights)) {
      if (Number(key) >= count) delete heights[Number(key)];
    }
  });
  let fitScale = $derived.by(() => {
    let min = 1;
    for (const h of Object.values(heights)) {
      const scaledHeight = h * scale;
      if (scaledHeight > maxHeight && scaledHeight > 0) {
        min = Math.min(min, maxHeight / scaledHeight);
      }
    }
    return min;
  });

  onDestroy(() => {
    webview.getCurrentWebview().emit('terminate-event');
    window.location.reload();
  });

  onMount(() => {
    // Ctrl/Cmd+E exports to PDF — the export people reach for most. HTML stays
    // on the hover buttons.
    const handleExportShortcut = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'e') {
        event.preventDefault();
        exporter.exportPdf();
      }
    };

    window.addEventListener('keydown', handleExportShortcut);
    return () => window.removeEventListener('keydown', handleExportShortcut);
  });

  listen('parse-error', (event) => {
    const error: ParseError = JSON.parse(event.payload as string);
    shared.error = error;
    shared.slides = [];
  });

  listen('markdown-updated', () => {
    shared.error = null;
  });

  listen('slide-changed', () => {
    shared.error = null;
  });
</script>

<svelte:window bind:innerWidth={winW} bind:innerHeight={winH} />

<svelte:head>
  <script src="../tailwind.min.js"></script>
  <!-- Include the Stylesheet for math -->
  <!-- <link
		rel="stylesheet"
		href="https://cdn.jsdelivr.net/gh/carloskiki/pulldown-latex@latest/styles.min.css"
	/> -->
  <!-- Include the Fonts for math -->
  <!-- <link
		rel="preload"
		href="https://cdn.jsdelivr.net/gh/carloskiki/pulldown-latex@latest/font/"
		as="font"
		crossorigin="anonymous"
	/> -->
</svelte:head>

<NavBar
  onExportHtml={() => exporter.exportHtml()}
  onExportPdf={() => exporter.exportPdf()}
  exportDisabled={exporter.busy}
/>

{#if exporter.busy}
  <ExportOverlay label={exporter.busyLabel} />
{/if}

{#if shared.error}
  <ErrorScreen message={shared.error.message} line={shared.error.line} />
{:else}
  <main class="flex flex-col items-center justify-center select-none">
    {#each shared.slides as slide, index (index)}
      <Slide
        id={String(index)}
        bgColor={slide.bg_color}
        textColor={slide.text_color}
        title={slide.title}
        content={slide.content}
        scale={scale * fitScale}
        onMeasure={(h) => reportHeight(index, h)}
      />
    {/each}
  </main>
{/if}
