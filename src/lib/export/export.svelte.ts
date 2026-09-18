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
 *
 * Both exports take an optional target path. Supplying one skips the save
 * dialog, which is what lets the CLI drive these exact functions rather than a
 * parallel implementation that would drift — see `docs/testing-platform-exports.md`.
 * Both also resolve only once the export has genuinely finished, so a caller can
 * await the real outcome instead of merely the print having started.
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

/**
 * Cap on how long `nextFrames` will wait for frames that may never come.
 *
 * Mirrors the cap on the readiness check's own copy in
 * `src/routes/view-slides/+page.svelte`, and for the same reason.
 */
const FRAME_WAIT_CAP_MS = 1000;

/**
 * Wait for layout to actually settle, not merely for Svelte to have updated.
 *
 * Capped with a timer because CLI export renders into a surface no compositor
 * is showing — an offscreen GTK window, a hidden HWND, a window parked past the
 * edge of every display — and an engine is free to stop delivering animation
 * frames to any of them. Two frames is what we want, but never at the cost of
 * hanging until the watchdog fires.
 */
function nextFrames(): Promise<void> {
  return new Promise((resolve) => {
    const done = () => {
      clearTimeout(cap);
      resolve();
    };
    const cap = setTimeout(done, FRAME_WAIT_CAP_MS);
    requestAnimationFrame(() => requestAnimationFrame(done));
  });
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

  async function exportHtml(targetPath?: string): Promise<void> {
    if (!canStart()) return;

    const name = await deckName();
    const path =
      targetPath ??
      (await save({
        defaultPath: `${name}.html`,
        filters: [{ name: 'HTML', extensions: ['html'] }]
      }));
    // Only reachable interactively; the CLI always supplies a path.
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
      throw error;
    } finally {
      busy = false;
      busyLabel = '';
    }
  }

  async function exportPdf(targetPath?: string): Promise<void> {
    if (!canStart()) return;

    const name = await deckName();
    const path =
      targetPath ??
      (await save({
        defaultPath: `${name}.pdf`,
        filters: [{ name: 'PDF', extensions: ['pdf'] }]
      }));
    // Only reachable interactively; the CLI always supplies a path.
    if (!path) return;

    busy = true;
    busyLabel = 'Preparing PDF…';

    const listeners: UnlistenFn[] = [];
    let settled = false;
    let timer: ReturnType<typeof setTimeout> | undefined;

    // The returned promise settles where `finish` does, so awaiting this
    // function means awaiting the print itself rather than its kick-off.
    return new Promise<void>((resolve, reject) => {
      const finish = (message: string, color: 'blue' | 'red') => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        listeners.forEach((stop) => stop());
        shared.printMode = false;
        busy = false;
        busyLabel = '';
        notify(message, color);

        if (color === 'red') reject(new Error(message));
        else resolve();
      };

      (async () => {
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
      })();
    });
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
