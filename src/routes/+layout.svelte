<script lang="ts">
  import '../app.css';
  import type { Snippet } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { shared, applySlideChange } from './view-slides/shared.svelte';

  let { children }: { children: Snippet } = $props();

  // Legacy event for backward compatibility
  listen('markdown-updated', (event: { payload: string }) => {
    shared.slides = JSON.parse(event.payload as string);
    localStorage.setItem('slides', JSON.stringify(shared.slides));
  });

  // New incremental event
  listen('slide-changed', (event: { payload: string }) => {
    const slideChangeEvent = JSON.parse(event.payload as string);
    applySlideChange(slideChangeEvent);
  });
</script>

{@render children()}
