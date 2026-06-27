<script lang="ts">
  import { DESIGN_W } from './shared.svelte';

  interface Props {
    id: string;
    bgColor: string;
    textColor: string;
    title: string;
    content: string;
    /** Final scale factor (uniform across the whole deck) applied to the slide. */
    scale: number;
    /** Reports the natural content height at design size to the parent. */
    onMeasure?: (height: number) => void;
  }

  let { id, bgColor, textColor, title, content, scale, onMeasure }: Props = $props();

  let contentHeight = $state(0);
  $effect(() => {
    onMeasure?.(contentHeight);
  });
</script>

<section
  {id}
  class="flex h-screen w-full items-center justify-center overflow-hidden {bgColor}"
  data-title={title}
>
  <!-- Fixed design width, laid out once at DESIGN_W, then uniformly scaled to
       fill the window width so text size is identical at any resolution. The
       scale is computed globally so every slide renders at the same size. -->
  <div
    class="flex shrink-0 items-center justify-center px-24"
    style="width: {DESIGN_W}px; transform: scale({scale}); transform-origin: center center;"
  >
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    <article bind:clientHeight={contentHeight} class="prose prose-invert prose-xl {textColor}">
      {@html content}
    </article>
  </div>
</section>
