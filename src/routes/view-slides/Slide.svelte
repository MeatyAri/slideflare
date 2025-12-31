<script lang="ts">
  interface Props {
    id: string;
    bgColor: string;
    textColor: string;
    title: string;
    content: string;
  }

  let { id, bgColor, textColor, title, content }: Props = $props();
</script>

<section {id} class="h-screen w-full {bgColor}" data-title={title}>
  <div class="h-screen w-full px-24">
    <div class="flex h-screen flex-col items-center justify-center">
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      <article class="prose prose-invert lg:prose-xl {textColor}">{@html content}</article>
    </div>
  </div>
</section>
