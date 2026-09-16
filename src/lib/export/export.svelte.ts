/**
 * Deck export orchestration: save dialogs, print-mode handshake, feedback.
 *
 * HTML export is synchronous from the frontend's point of view — build the
 * document, hand it to Rust, done.
 *
 * PDF export is a handshake. The Rust side prints the *live* deck webview
 * through the platform's own print pipeline, so before invoking it the deck has
 * to be put into print mode (slides scaled to the page instead of the window)
 * and given a moment to lay out. Rust then reports the outcome as an event,
 * because on two of the three platforms printing is asynchronous and the command
 * returns as soon as the job has *started*.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { save } from '@tauri-apps/plugin-dialog';

import { notify, shared } from '../../routes/view-slides/shared.svelte';
import { buildStandaloneHtml } from './standalone';

const EVENT_PDF_DONE = 'pdf-export-done';
const EVENT_PDF_FAILED = 'pdf-export-failed';

/**
 * How long to wait for a print to report back before giving up and taking the
 * deck out of print mode. Without this a backend that never fires its
 * completion signal would strand the deck in a half-scaled state forever.
 */
const PDF_TIMEOUT_MS = 120_000;

/** Wait for layout to actually settle, not merely for Svelte to have updated. */
function nextFrames(): Promise<void> {
  return new Promise((resolve) =>
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
  );
}

function baseName(filePath: string): string {
  const name = filePath.split(/[\\/]/).pop() ?? '';
  return name.replace(/\.md$/i, '');
}

export function createExport() {
  let busy = $state(false);
  let busyLabel = $state('');

  async function deckName(): Promise<string> {
    try {
      const filePath = await invoke<string | null>('current_file_path');
      const name = filePath ? baseName(filePath) : '';
      return name || 'slideflare-deck';
    } catch {
      return 'slideflare-deck';
    }
  }

  /** Common guard: nothing to export, or an export already running. */
  function canStart(): boolean {
    if (busy) return false;
    if (shared.slides.length === 0) {
      notify('Nothing to export', 'yellow');
      return false;
    }
    return true;
  }

  async function exportHtml(): Promise<void> {
    if (!canStart()) return;

    const name = await deckName();
    const path = await save({
      defaultPath: `${name}.html`,
      filters: [{ name: 'HTML', extensions: ['html'] }]
    });
    if (!path) return;

    busy = true;
    busyLabel = 'Building HTML…';
    try {
      // Built while the deck is on screen: the stylesheet harvest reads the
      // live styles, which only exist because the slides are rendered.
      const contents = buildStandaloneHtml({ slides: shared.slides, title: name });
      await invoke('write_export', { path, contents });
      notify('Exported to HTML', 'blue');
    } catch (error) {
      notify(`HTML export failed: ${error}`, 'red');
    } finally {
      busy = false;
      busyLabel = '';
    }
  }

  async function exportPdf(): Promise<void> {
    if (!canStart()) return;

    const name = await deckName();
    const path = await save({
      defaultPath: `${name}.pdf`,
      filters: [{ name: 'PDF', extensions: ['pdf'] }]
    });
    if (!path) return;

    busy = true;
    busyLabel = 'Preparing PDF…';

    const listeners: UnlistenFn[] = [];
    let settled = false;
    let timer: ReturnType<typeof setTimeout> | undefined;

    const finish = (message: string, color: 'blue' | 'red') => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      listeners.forEach((stop) => stop());
      shared.printMode = false;
      busy = false;
      busyLabel = '';
      notify(message, color);
    };

    try {
      listeners.push(await listen(EVENT_PDF_DONE, () => finish('Exported to PDF', 'blue')));
      listeners.push(
        await listen<string>(EVENT_PDF_FAILED, (event) =>
          finish(`PDF export failed: ${event.payload}`, 'red')
        )
      );
      timer = setTimeout(() => finish('PDF export timed out', 'red'), PDF_TIMEOUT_MS);

      // Lay the deck out for paper, then let the browser actually do it before
      // handing the webview to the print pipeline.
      shared.printMode = true;
      await nextFrames();

      await invoke('export_pdf', { path });
    } catch (error) {
      finish(`PDF export failed: ${error}`, 'red');
    }
  }

  return {
    get busy() {
      return busy;
    },
    get busyLabel() {
      return busyLabel;
    },
    exportHtml,
    exportPdf
  };
}
