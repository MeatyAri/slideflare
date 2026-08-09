<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { INSTALL_SOURCE_CHOICES, type InstallSource, type UpdateStatus } from './updates.svelte';

  interface Props {
    checking: boolean;
    status: UpdateStatus | null;
    installingSkill: boolean;
    skillMessage: string;
    skillError: string;
    onClose: () => void;
    onRecheck: () => void;
    onChooseInstallSource: (source: InstallSource) => void;
    onInstallSkill: () => void;
  }

  let {
    checking,
    status,
    installingSkill,
    skillMessage,
    skillError,
    onClose,
    onRecheck,
    onChooseInstallSource,
    onInstallSkill
  }: Props = $props();

  let picked = $state<InstallSource | ''>('');

  const needsSource = $derived(status?.installSourceUnknown === true);

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div
  class="fixed inset-0 z-50 flex items-center justify-center bg-gray-950/80 p-6 backdrop-blur-sm select-none"
  transition:fade={{ duration: 150 }}
  role="dialog"
  aria-modal="true"
  aria-label="Updates"
>
  <div
    class="flex w-full max-w-lg flex-col overflow-hidden rounded-2xl bg-gray-800 shadow-2xl ring-1 ring-white/10"
    transition:scale={{ duration: 200, start: 0.96, easing: cubicOut }}
  >
    <div class="flex items-center justify-between border-b border-white/10 px-6 py-4">
      <h2 class="text-lg font-semibold text-gray-100">Updates</h2>
      <button
        class="rounded-md px-2 py-1 text-sm text-gray-400 transition-colors hover:bg-white/5 hover:text-gray-200"
        onclick={onClose}
        aria-label="Close updates"
      >
        Close
      </button>
    </div>

    <div class="flex flex-col gap-5 px-6 py-5">
      {#if checking}
        <p class="text-sm text-gray-400">Checking for updates…</p>
      {:else if needsSource}
        <!-- Build didn't record how it was installed: ask once, then remember. -->
        <div class="flex flex-col gap-3">
          <div class="flex flex-col gap-1">
            <h3 class="text-sm font-semibold text-gray-100">How did you install SlideFlare?</h3>
            <p class="text-xs leading-relaxed text-gray-400">
              This build doesn't record its install source, so we can't tell how to update it. Pick
              one and we'll remember it.
            </p>
          </div>
          <select
            class="rounded-md border border-white/10 bg-gray-900 px-3 py-2 text-sm text-gray-100 focus:border-blue-500 focus:outline-none"
            bind:value={picked}
            aria-label="Install source"
          >
            <option value="" disabled>Select an install source…</option>
            {#each INSTALL_SOURCE_CHOICES as choice (choice.value)}
              <option value={choice.value}>{choice.label} — {choice.hint}</option>
            {/each}
          </select>
          <button
            class="self-start rounded-md bg-blue-600 px-4 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40"
            disabled={picked === ''}
            onclick={() => picked !== '' && onChooseInstallSource(picked)}
          >
            Save and check
          </button>
        </div>
      {:else if status}
        <!-- App -->
        <div class="flex flex-col gap-2">
          <div class="flex items-baseline justify-between gap-3">
            <h3 class="text-sm font-semibold text-gray-100">SlideFlare</h3>
            <span class="font-mono text-xs text-gray-500">{status.current}</span>
          </div>

          {#if status.error}
            <p class="text-sm text-yellow-400">{status.error}</p>
          {:else if status.appUpdateAvailable}
            <p class="text-sm text-blue-300">
              Update available{status.latest ? `: ${status.latest}` : ''}
            </p>
            {#if status.updateCommand}
              <p class="text-xs text-gray-400">Update with:</p>
              <code
                class="rounded-md bg-gray-900 px-3 py-2 font-mono text-xs break-all text-gray-200 select-text"
                >{status.updateCommand}</code
              >
            {/if}
            {#if status.releaseUrl}
              {@const url = status.releaseUrl}
              <button
                class="self-start text-xs text-blue-400 underline transition-colors hover:text-blue-300"
                onclick={() => openUrl(url)}
              >
                View on GitHub
              </button>
            {/if}
          {:else}
            <p class="text-sm text-gray-400">You're up to date.</p>
          {/if}
        </div>

        <div class="border-t border-white/10"></div>

        <!-- Skill -->
        <div class="flex flex-col gap-2">
          <div class="flex items-baseline justify-between gap-3">
            <h3 class="text-sm font-semibold text-gray-100">slideflare-slides skill</h3>
            <span class="text-xs text-gray-500">
              {status.skillInstalled ? 'installed' : 'not installed'}
            </span>
          </div>
          <p class="text-xs leading-relaxed text-gray-400">
            Lets your AI agent generate and edit full decks from a plain-language prompt. Installs
            into <code class="font-mono">~/.agents/skills/</code>.
          </p>
          <button
            class="self-start rounded-md bg-blue-600 px-4 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-40"
            disabled={installingSkill}
            onclick={onInstallSkill}
          >
            {installingSkill
              ? 'Installing…'
              : status.skillInstalled
                ? 'Update skill'
                : 'Install skill'}
          </button>
          {#if skillMessage}
            <p class="text-xs break-all text-green-400">{skillMessage}</p>
          {/if}
          {#if skillError}
            <p class="text-xs break-all text-red-400">{skillError}</p>
          {/if}
        </div>
      {/if}
    </div>

    <div class="flex items-center justify-between border-t border-white/10 px-6 py-4">
      <button
        class="rounded-md px-3 py-1.5 text-sm font-medium text-gray-300 transition-colors hover:bg-white/5 disabled:cursor-not-allowed disabled:opacity-30"
        onclick={onRecheck}
        disabled={checking}
      >
        Check again
      </button>
      <button
        class="rounded-md bg-gray-700 px-4 py-1.5 text-sm font-semibold text-gray-100 transition-colors hover:bg-gray-600"
        onclick={onClose}
      >
        Done
      </button>
    </div>
  </div>
</div>
