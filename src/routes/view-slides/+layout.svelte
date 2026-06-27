<script lang="ts">
  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { shared, notifications, notify } from './shared.svelte';
  import Notification from './Notification.svelte';

  let { children }: { children: Snippet } = $props();

  onMount(() => {
    /**
     * Prevent default scroll on wheel and touch
     */
    const preventScroll = (e: Event) => {
      e.preventDefault();
    };

    /**
     * Handle arrow key navigation
     */
    const handleArrowKeys = (e: KeyboardEvent) => {
      e.preventDefault();
      let shouldScroll = true;
      if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
        shared.index -= 1;
        if (shared.index < 0) {
          shared.index = 0;
          shouldScroll = false;
        }
      }
      if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
        shared.index += 1;
        if (shared.index >= shared.slides.length) {
          shared.index = shared.slides.length - 1;
          shouldScroll = false;
        }
      }

      if (!shouldScroll) return;
      // Scroll to the current slide
      const slide = document.getElementById(String(shared.index));
      slide?.scrollIntoView({ behavior: 'smooth' });
    };

    /**
     * Handle Ctrl+R (or Cmd+R on macOS) to trigger a full reparse of the
     * document. This bypasses the incremental diffing mechanism and re-parses
     * the document from scratch — equivalent to closing and reopening the app.
     * Useful when the diffing mechanism goes wrong and produces a corrupted
     * document.
     */
    const handleRefresh = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'r') {
        e.preventDefault();
        invoke('reparse_document');
        notify('Document reparsed', 'blue');
      }
    };

    window.addEventListener('wheel', preventScroll, { passive: false });
    window.addEventListener('touchmove', preventScroll, { passive: false });
    window.addEventListener('keydown', handleArrowKeys, { passive: false });
    window.addEventListener('keydown', handleRefresh);

    return () => {
      window.removeEventListener('wheel', preventScroll);
      window.removeEventListener('touchmove', preventScroll);
      window.removeEventListener('keydown', handleArrowKeys);
      window.removeEventListener('keydown', handleRefresh);
    };
  });
</script>

{@render children()}

<div class="pointer-events-none fixed right-6 bottom-6 z-50 flex flex-col items-end gap-2">
  {#each notifications as notification (notification.id)}
    <Notification id={notification.id} message={notification.message} color={notification.color} />
  {/each}
</div>
