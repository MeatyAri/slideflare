<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import type { TutorialSlide } from './tutorial.svelte';

  interface Props {
    mode: 'welcome' | 'whatsnew';
    version: string;
    slides: TutorialSlide[];
    onDismiss: () => void;
  }

  let { mode, version, slides, onDismiss }: Props = $props();

  let page = $state(0);

  const total = $derived(slides.length);
  const current = $derived(slides[page]);
  const isLast = $derived(page >= total - 1);
  const isFirst = $derived(page <= 0);

  const heading = $derived(
    mode === 'welcome' ? 'Welcome to SlideFlare' : `What's new in v${version}`
  );

  function next() {
    if (isLast) {
      onDismiss();
    } else {
      page += 1;
    }
  }

  function back() {
    if (!isFirst) page -= 1;
  }

  function onKeydown(e: KeyboardEvent) {
    switch (e.key) {
      case 'ArrowRight':
        e.preventDefault();
        next();
        break;
      case 'ArrowLeft':
        e.preventDefault();
        back();
        break;
      case 'Escape':
        e.preventDefault();
        onDismiss();
        break;
      case 'Enter':
        e.preventDefault();
        next();
        break;
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<!-- eslint-disable-next-line svelte/no-static-element-interactions -->
<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-gray-950/80 p-6 backdrop-blur-sm select-none"
  transition:fade={{ duration: 150 }}
  role="dialog"
  aria-modal="true"
  aria-label={heading}
>
  <div
    class="flex w-full max-w-xl flex-col overflow-hidden rounded-2xl bg-gray-800 shadow-2xl ring-1 ring-white/10"
    transition:scale={{ duration: 200, start: 0.96, easing: cubicOut }}
  >
    <!-- Header -->
    <div class="flex items-center justify-between border-b border-white/10 px-6 py-4">
      <h2 class="text-lg font-semibold text-gray-100">{heading}</h2>
      <button
        class="rounded-md px-2 py-1 text-sm text-gray-400 transition-colors hover:bg-white/5 hover:text-gray-200"
        onclick={onDismiss}
      >
        Skip
      </button>
    </div>

    <!-- Body -->
    <div class="flex min-h-[18rem] flex-col px-6 py-6">
      {#key page}
        <div class="flex flex-1 flex-col" in:fade={{ duration: 150 }}>
          {#if current.grouped}
            <!-- Grouped short features: list several one-liners together -->
            <ul class="flex flex-1 flex-col justify-center gap-5">
              {#each current.features as feature (feature.id)}
                <li class="flex flex-col gap-1">
                  <span class="text-base font-semibold text-blue-300">{feature.title}</span>
                  <span class="text-sm leading-relaxed text-gray-300">{feature.body}</span>
                </li>
              {/each}
            </ul>
          {:else}
            <!-- Single feature slide (may carry media) -->
            {@const feature = current.features[0]}
            <div class="flex flex-1 flex-col items-center justify-center gap-4 text-center">
              {#if feature.media}
                <img
                  src={feature.media}
                  alt={feature.title}
                  class="max-h-48 w-auto rounded-lg ring-1 ring-white/10"
                />
              {/if}
              <h3 class="text-xl font-semibold text-gray-100">{feature.title}</h3>
              <p class="max-w-md text-sm leading-relaxed text-gray-300">{feature.body}</p>
            </div>
          {/if}
        </div>
      {/key}
    </div>

    <!-- Footer / controls -->
    <div class="flex items-center justify-between border-t border-white/10 px-6 py-4">
      <button
        class="rounded-md px-3 py-1.5 text-sm font-medium text-gray-300 transition-colors hover:bg-white/5 disabled:cursor-not-allowed disabled:opacity-30"
        onclick={back}
        disabled={isFirst}
      >
        Back
      </button>

      {#if total > 1}
        <div class="flex items-center gap-2">
          {#each slides as _, i (i)}
            <span
              class="h-2 w-2 rounded-full transition-colors {i === page
                ? 'bg-blue-400'
                : 'bg-gray-600'}"
            ></span>
          {/each}
        </div>
      {/if}

      <button
        class="rounded-md bg-blue-600 px-4 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-blue-500"
        onclick={next}
      >
        {isLast ? 'Get started' : 'Next'}
      </button>
    </div>
  </div>
</div>
