<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { NotificationColor } from './shared.svelte';
  import { dismissNotification } from './shared.svelte';
  import { cubicOut, cubicIn } from 'svelte/easing';
  import type { TransitionConfig } from 'svelte/transition';

  interface Props {
    id: number;
    message: string;
    color: NotificationColor;
  }

  let { id, message, color }: Props = $props();

  const colorMap: Record<NotificationColor, string> = {
    blue: 'bg-blue-500',
    red: 'bg-red-500',
    yellow: 'bg-yellow-500'
  };

  let timeoutId: ReturnType<typeof setTimeout>;

  onMount(() => {
    timeoutId = setTimeout(() => dismissNotification(id), 2500);
  });

  onDestroy(() => clearTimeout(timeoutId));

  function slidePop(
    _node: HTMLElement,
    { duration, direction }: { duration: number; direction: 'in' | 'out' }
  ): TransitionConfig {
    const easing = direction === 'in' ? cubicOut : cubicIn;
    return {
      duration,
      easing,
      css: (t: number) =>
        `transform: translateX(${(1 - t) * 24}px) scale(${direction === 'in' ? 0.92 + 0.08 * t : 0.95 + 0.05 * t}); opacity: ${t};`
    };
  }
</script>

<div
  class="pointer-events-auto flex items-center gap-2.5 rounded-lg bg-gray-800/95 px-4 py-2.5 shadow-xl ring-1 ring-white/10 backdrop-blur-sm"
  in:slidePop={{ duration: 300, direction: 'in' }}
  out:slidePop={{ duration: 200, direction: 'out' }}
>
  <span class="h-2 w-2 shrink-0 rounded-full {colorMap[color]}"></span>
  <span class="text-sm font-medium whitespace-nowrap text-gray-100">{message}</span>
</div>
