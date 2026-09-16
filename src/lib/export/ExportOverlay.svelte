<script lang="ts">
  /**
   * Covers the deck while an export runs.
   *
   * This is not only cosmetic. PDF export rescales every slide from the window
   * to the page while the deck is still on screen, which would otherwise show as
   * a jarring jump; the overlay hides that. `data-no-print` keeps it out of the
   * printed output.
   */
  interface Props {
    label: string;
  }

  let { label }: Props = $props();
</script>

<div
  class="fixed inset-0 z-[100] flex items-center justify-center bg-gray-900/80 backdrop-blur-sm"
  data-no-print
  role="status"
  aria-live="polite"
>
  <div class="flex flex-col items-center gap-3">
    <svg
      class="h-8 w-8 animate-spin text-blue-400"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
    >
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" opacity="0.25" />
      <path
        d="M22 12a10 10 0 0 0-10-10"
        stroke="currentColor"
        stroke-width="3"
        stroke-linecap="round"
      />
    </svg>
    <p class="text-sm font-medium text-gray-200">{label}</p>
  </div>
</div>
