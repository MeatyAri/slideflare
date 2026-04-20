<script lang="ts">
  interface Props {
    message: string;
    line?: number | null;
  }

  let { message, line = null }: Props = $props();
</script>

<div class="flex min-h-screen flex-col items-center justify-center bg-gray-900 p-8">
  <div class="max-w-lg rounded-lg border-2 border-red-500 bg-gray-800 p-8">
    <div class="flex flex-col items-center space-y-4">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="64"
        height="64"
        viewBox="0 0 24 24"
        class="text-red-500"
      >
        <g fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" />
          <path stroke-linecap="round" d="M12 8v4M12 16h.01" />
        </g>
      </svg>

      <h1 class="text-2xl font-bold text-red-400">Parse Error</h1>

      <p class="text-center text-gray-300">{message}</p>

      {#if line !== null}
        <p class="text-sm text-gray-500">Line: {line}</p>
      {/if}

      <p class="text-sm text-gray-500">
        Fix the syntax in your markdown file and the slides will update automatically.
      </p>
    </div>
  </div>
</div>
