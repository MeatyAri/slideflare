<script lang="ts">
  import NavBar from './NavBar.svelte';
  import Slide from './Slide.svelte';
  import ErrorScreen from './ErrorScreen.svelte';
  import { webview } from '@tauri-apps/api';
  import { listen } from '@tauri-apps/api/event';
  import { shared, type ParseError } from './shared.svelte';
  import { onDestroy } from 'svelte';

  onDestroy(() => {
    webview.getCurrentWebview().emit('terminate-event');
    window.location.reload();
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

<NavBar />

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
      />
    {/each}
  </main>
{/if}
