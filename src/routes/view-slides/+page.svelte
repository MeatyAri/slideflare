<script lang="ts">
  import NavBar from './NavBar.svelte';
  import Slide from './Slide.svelte';
  import ErrorScreen from './ErrorScreen.svelte';
  import { webview } from '@tauri-apps/api';
  import { invoke } from '@tauri-apps/api/core';
  import { emit, listen } from '@tauri-apps/api/event';
  import { shared, type ParseError, DESIGN_W, DESIGN_H, VIEWPORT_PADDING } from './shared.svelte';
  import { onDestroy, onMount } from 'svelte';
  import { createExport } from '$lib/export/export.svelte';
  import ExportOverlay from '$lib/export/ExportOverlay.svelte';

  const exporter = createExport();

  /** An export asked for on the command line. */
  interface CliExportRequest {
    kind: 'pdf' | 'html';
    outPath: string;
  }

  /**
   * How long to wait for the deck to lay itself out.
   *
   * The CLI gets the longer budget because it runs on cold CI machines under
   * xvfb; its own watchdog in `cli::gui` is longer still, so this one reports
   * the more useful "deck never became ready" rather than a bare timeout.
   */
  const CLI_READY_TIMEOUT_MS = 60_000;
  const GUI_READY_TIMEOUT_MS = 30_000;

  /**
   * The exporters reject on genuine failure, but they have already told the user
   * with a notification. Interactive callers therefore only need to stop the
   * rejection becoming an unhandled one.
   */
  const ignoreExportRejection = () => {};

  // Track real window size. Scaling is WIDTH-driven: the fixed design width
  // (DESIGN_W) is scaled to fill the window width (minus a tiny padding), so
  // text size is identical at any resolution. maxHeight is passed to each slide
  // so its overflow guard only shrinks content that would exceed the viewport.
  let winW = $state(typeof window !== 'undefined' ? window.innerWidth : DESIGN_W);
  let winH = $state(typeof window !== 'undefined' ? window.innerHeight : DESIGN_H);
  // While exporting to PDF the slide is measured against the page rather than
  // the window: the print stylesheet sizes each section to exactly DESIGN_W by
  // DESIGN_H, so the width scale is 1 and only the overflow guard below can
  // still shrink content. Using the window size here instead would scale every
  // slide by the ratio between the window and the paper.
  let scale = $derived(shared.printMode ? 1 : (winW - 2 * VIEWPORT_PADDING) / DESIGN_W);
  let maxHeight = $derived(shared.printMode ? DESIGN_H : winH - 2 * VIEWPORT_PADDING);

  // Natural (design-size) content height of each slide, reported by the slides.
  // We derive ONE shrink factor from the tallest slide and apply it to every
  // slide, so text size stays identical across the deck AND no slide overflows.
  let heights = $state<Record<number, number>>({});
  const reportHeight = (index: number, h: number) => {
    if (heights[index] !== h) heights[index] = h;
  };
  // Drop stale measurements when the deck changes (e.g. navigating back and
  // opening a different file) so fitScale recalculates for the new slides
  // instead of reusing the previous deck's scaling factor.
  $effect(() => {
    const count = shared.slides.length;
    for (const key of Object.keys(heights)) {
      if (Number(key) >= count) delete heights[Number(key)];
    }
  });
  let fitScale = $derived.by(() => {
    let min = 1;
    for (const h of Object.values(heights)) {
      const scaledHeight = h * scale;
      if (scaledHeight > maxHeight && scaledHeight > 0) {
        min = Math.min(min, maxHeight / scaledHeight);
      }
    }
    return min;
  });

  onDestroy(() => {
    webview.getCurrentWebview().emit('terminate-event');
    window.location.reload();
  });

  onMount(() => {
    // Ctrl/Cmd+E exports to PDF — the export people reach for most. HTML stays
    // on the hover buttons.
    const handleExportShortcut = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'e') {
        event.preventDefault();
        void exporter.exportPdf().catch(ignoreExportRejection);
      }
    };

    window.addEventListener('keydown', handleExportShortcut);
    return () => window.removeEventListener('keydown', handleExportShortcut);
  });

  listen('parse-error', (event) => {
    const error: ParseError = JSON.parse(event.payload as string);
    shared.error = error;
    shared.slides = [];
  });

  listen('markdown-updated', () => {
    shared.error = null;
    // A fresh deck invalidates every measurement. The $effect above only prunes
    // indices past the end, so without this a shorter-but-different deck could
    // satisfy the readiness check below using the previous deck's heights.
    heights = {};
  });

  listen('slide-changed', () => {
    shared.error = null;
  });

  /** Emitted once the deck is parsed, measured, and painted. */
  const EVENT_DECK_READY = 'deck-ready';
  /** Emitted when the deck cannot be shown at all. */
  const EVENT_DECK_FAILED = 'deck-failed';

  /** How often the readiness conditions are re-checked. */
  const POLL_INTERVAL_MS = 50;
  /** Cap on how long `nextFrames` will wait for frames that may never come. */
  const FRAME_WAIT_CAP_MS = 1000;
  /**
   * Cap on how long readiness will wait for images to settle.
   *
   * A deck may reference a file that is missing, or a remote image on a machine
   * with no network. Neither should turn an export into a timeout, so a slide
   * that never settles is rendered as it stands rather than waited on forever.
   */
  const MEDIA_WAIT_CAP_MS = 10_000;

  /**
   * Wait for layout to settle, not merely for Svelte to have updated.
   *
   * Capped with a timer because an offscreen or occluded window is not
   * guaranteed to be painted: compositors are free to stop delivering animation
   * frames to one, and CLI export deliberately parks the window offscreen. Two
   * frames is what we want, but never at the cost of hanging forever.
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

  /**
   * Resolve once `condition` holds, or reject when `timeoutMs` elapses.
   *
   * Polls rather than watching the state reactively. An `$effect` would be the
   * idiomatic choice, but these conditions are awaited from an async `onMount`
   * callback, and effects created in a detached `$effect.root` from there are
   * not reliably re-run — which showed up as an export that waited out its full
   * timeout while the deck sat fully rendered behind it. Polling every 50ms for
   * something that takes about a second is cheap and has no such failure mode.
   */
  function until(condition: () => boolean, what: string, timeoutMs: number): Promise<void> {
    return new Promise((resolve, reject) => {
      const deadline = Date.now() + timeoutMs;

      const check = () => {
        let satisfied: boolean;
        try {
          satisfied = condition();
        } catch (error) {
          reject(error);
          return;
        }

        if (satisfied) {
          resolve();
        } else if (Date.now() >= deadline) {
          reject(new Error(`timed out waiting for ${what}`));
        } else {
          setTimeout(check, POLL_INTERVAL_MS);
        }
      };

      check();
    });
  }

  /**
   * Resolve once every image on the page has a final layout box.
   *
   * This is the one thing the deck's own measurement cannot infer. A slide
   * reports its natural height as soon as its elements exist, but an `<img>`
   * with no intrinsic size yet occupies nothing, so a deck measured before its
   * images have decoded measures short — and because one `fitScale` is derived
   * for the whole document from the tallest slide, every slide then prints
   * over-sized and clipped, at exit code 0.
   *
   * It matters most where the render surface is not on any screen. Engines
   * deprioritise or outright suspend image decoding for content they consider
   * hidden, which is exactly what CLI export asks them to render into; each
   * platform has its own arrangement to keep the content nominally visible (see
   * `cli::gui`), and this is the safety net under all three.
   *
   * Video is deliberately *not* waited on. `<video>` has no `decode()`, and the
   * nearest equivalent — `loadedmetadata`, which is what fixes `videoWidth` and
   * so the element's height — does not arrive at all in a WebKitGTK offscreen
   * window, so waiting on it only adds the cap to every export with a video in
   * it. A video that has not reported metadata by print time is laid out
   * collapsed, which is how every platform already behaves today; see the
   * entry in `TODO.md`.
   *
   * Failures resolve rather than reject. A broken image is a deck the user
   * should still get, with a broken image on it.
   */
  function mediaSettled(): Promise<void> {
    // `decode()` resolves only once the bitmap is ready to paint, which is
    // strictly later than `complete` and is the point the layout box is final.
    // Already-decoded images resolve immediately.
    const waits = Array.from(document.images, (image) => image.decode().catch(() => undefined));
    if (waits.length === 0) return Promise.resolve();

    const settled = Promise.all(waits).then(() => undefined);
    const capped = new Promise<void>((resolve) => setTimeout(resolve, MEDIA_WAIT_CAP_MS));
    return Promise.race([settled, capped]);
  }

  /**
   * Wait until the deck is genuinely ready to be rendered to paper.
   *
   * PDF export prints the live, realized webview precisely because layout, font
   * resolution, and MathML measurement are already settled there — so "the file
   * parsed" is far too weak a signal. All four conditions matter:
   *
   *  1. `generation > 0` — a parse from *this* process has arrived, rather than
   *     the deck merely being non-empty.
   *  2. every slide has reported its natural height, meaning each one has been
   *     laid out and measured, and `fitScale` is derived from a complete set.
   *  3. images and video have settled, so nothing is still occupying zero
   *     height when it is measured — see `mediaSettled`.
   *  4. fonts have resolved — text measured against a fallback face reflows once
   *     the real one arrives, which on paper shows up as clipped or shifted text.
   *  5. every slide has reported its height *again*, because (3) and (4) both
   *     reflow the slides they settle, and the measurement taken in (2) is
   *     stale the moment they do.
   *  6. two frames have passed, so the scale derived from (5) has actually been
   *     applied rather than merely computed.
   */
  async function waitForDeckReady(timeoutMs: number): Promise<void> {
    await until(
      () => shared.generation > 0 && shared.slides.length > 0 && shared.error === null,
      'the deck to be parsed',
      timeoutMs
    );

    await until(
      () => Object.keys(heights).length === shared.slides.length,
      'every slide to be measured',
      timeoutMs
    );

    await mediaSettled();

    // Not every engine implements the font loading API; where it is missing,
    // there is nothing to wait for.
    await document.fonts?.ready;

    // Both of the above reflow content, and a ResizeObserver reports the new
    // height on a later task. Wait for the measurements to stop moving rather
    // than assuming they already have.
    await measurementsStable(timeoutMs);

    await nextFrames();
  }

  /**
   * Resolve once two consecutive polls see identical slide heights.
   *
   * Cheaper and more honest than guessing a fixed delay: the thing that must be
   * true before printing is that `fitScale` is derived from measurements that
   * are not about to change, and that is exactly what this tests.
   */
  async function measurementsStable(timeoutMs: number): Promise<void> {
    const snapshot = () =>
      Object.keys(heights)
        .sort()
        .map((key) => `${key}:${heights[Number(key)]}`)
        .join(',');

    let previous: string | null = null;
    await until(
      () => {
        const current = snapshot();
        const stable = previous === current;
        previous = current;
        return stable;
      },
      'slide measurements to stop changing',
      timeoutMs
    );
  }

  /**
   * What the readiness check can currently see.
   *
   * Appended to a timeout so the failure names the condition that did not hold,
   * rather than leaving a CI log with nothing but "timed out" to work from.
   */
  function readinessState(): string {
    return [
      `generation=${shared.generation}`,
      `slides=${shared.slides.length}`,
      `measured=${Object.keys(heights).length}`,
      `error=${shared.error ? JSON.stringify(shared.error.message) : 'none'}`
    ].join(' ');
  }

  /**
   * Announce readiness, and carry out a CLI export if one was requested.
   *
   * In the GUI these events have no listener and nothing else happens — the
   * request is always `null`. In CLI export mode the very same exporter the
   * NavBar buttons call is invoked with the path from argv, which is what makes
   * `slideflare export` a test of the shipping code path rather than of a
   * parallel one.
   */
  onMount(async () => {
    let request: CliExportRequest | null = null;
    try {
      request = await invoke<CliExportRequest | null>('cli_export_request');
    } catch (error) {
      console.error('Failed to read the CLI export request:', error);
    }

    // The CLI supplies its own budget; the GUI just wants the event announced.
    const timeoutMs = request ? CLI_READY_TIMEOUT_MS : GUI_READY_TIMEOUT_MS;

    try {
      await waitForDeckReady(timeoutMs);
    } catch (error) {
      const message = `${shared.error?.message ?? String(error)} [${readinessState()}]`;
      await emit(EVENT_DECK_FAILED, message);
      if (request) await invoke('cli_export_finish', { ok: false, message });
      return;
    }

    await emit(EVENT_DECK_READY, shared.slides.length);
    if (!request) return;

    try {
      if (request.kind === 'pdf') await exporter.exportPdf(request.outPath);
      else await exporter.exportHtml(request.outPath);
      await invoke('cli_export_finish', { ok: true, message: request.outPath });
    } catch (error) {
      await invoke('cli_export_finish', { ok: false, message: String(error) });
    }
  });
</script>

<svelte:window bind:innerWidth={winW} bind:innerHeight={winH} />

<svelte:head>
  <script src="../tailwind.min.js"></script>
  <!-- Include the Stylesheet for math -->
  <!-- <link
		rel="stylesheet"
		href="https://cdn.jsdelivr.net/gh/carloskiki/pulldown-latex@latest/styles.min.css"
	/> -->
  <!-- Include the Fonts for math -->
  <!-- <link
		rel="preload"
		href="https://cdn.jsdelivr.net/gh/carloskiki/pulldown-latex@latest/font/"
		as="font"
		crossorigin="anonymous"
	/> -->
</svelte:head>

<NavBar
  onExportHtml={() => void exporter.exportHtml().catch(ignoreExportRejection)}
  onExportPdf={() => void exporter.exportPdf().catch(ignoreExportRejection)}
  exportDisabled={exporter.busy}
/>

{#if exporter.busy}
  <ExportOverlay label={exporter.busyLabel} />
{/if}

{#if shared.error}
  <ErrorScreen message={shared.error.message} line={shared.error.line} />
{:else}
  <main class="flex flex-col items-center justify-center select-none">
    {#each shared.slides as slide, index (index)}
      <Slide
        id={String(index)}
        bgColor={slide.bg_color}
        textColor={slide.text_color}
        title={slide.title}
        content={slide.content}
        scale={scale * fitScale}
        onMeasure={(h) => reportHeight(index, h)}
      />
    {/each}
  </main>
{/if}
