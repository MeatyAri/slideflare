/**
 * Builds a self-contained HTML deck: one file, no network, openable in any
 * browser, navigable like the app.
 *
 * This is generated in the frontend rather than in Rust for one reason: the
 * styling a slide needs is only knowable at runtime. Slide colours come from
 * user frontmatter (`bg-blue-500`, `text-white`, anything else), so they are not
 * in the compiled stylesheet — Tailwind's browser build generates them live from
 * whatever classes are actually in the DOM. Reading the live stylesheets is
 * therefore the only way to capture both halves of the styling, and only the
 * webview can do that.
 *
 * Media needs no special handling: `post_process_asset_paths` in the Rust parser
 * has already inlined images and video as base64 data URLs by the time slide
 * HTML reaches the frontend, so the markup is self-contained as-is.
 */

import {
  DESIGN_W,
  DESIGN_H,
  VIEWPORT_PADDING,
  type Slide
} from '../../routes/view-slides/shared.svelte';

/**
 * Serialize every stylesheet currently applied to the document.
 *
 * Covers both the compiled `app.css` (prose/typography, base rules, the print
 * stylesheet) and the utilities Tailwind's browser build generated for the
 * classes this particular deck uses.
 */
export function harvestCss(): string {
  return Array.from(document.styleSheets)
    .map((sheet) => serializeSheet(sheet))
    .filter(Boolean)
    .join('\n');
}

function serializeSheet(sheet: CSSStyleSheet): string {
  let rules: CSSRuleList;
  try {
    rules = sheet.cssRules;
  } catch {
    // Cross-origin sheets refuse `cssRules`. Nothing in the app loads styles
    // from another origin today, so this is a guard rather than a path we
    // expect to take — and an unreadable sheet is skipped rather than fatal.
    return '';
  }

  return Array.from(rules)
    .map((rule) => {
      // `@import` serializes as the import statement alone, so the imported
      // sheet has to be walked separately or its rules are lost.
      if (rule instanceof CSSImportRule && rule.styleSheet) {
        return serializeSheet(rule.styleSheet);
      }
      return rule.cssText;
    })
    .join('\n');
}

/** Chrome for the exported deck: scroll behaviour, nav rail, counter. */
const DECK_CSS = `
  html,
  body {
    margin: 0;
    height: 100%;
    overflow: hidden;
  }

  /*
   * "display: block" is load-bearing, not cosmetic. Each slide is h-screen
   * (100vh) and this is a fixed-height scroll container, so as a flex column it
   * would shrink every slide to make the whole deck fit one viewport — the
   * entire deck squashed onto a single unreadable screen. Block layout lets the
   * slides keep their height and the container scroll instead.
   */
  #sf-deck {
    display: block;
    height: 100%;
    overflow-y: auto;
    scroll-snap-type: y mandatory;
    scroll-behavior: smooth;
  }

  #sf-deck > section {
    scroll-snap-align: start;
  }

  #sf-rail {
    position: fixed;
    top: 0;
    left: 0;
    z-index: 50;
    display: flex;
    height: 100%;
    width: 5rem;
    flex-direction: column;
    align-items: center;
    padding: 2rem 0;
  }

  #sf-rail-inner {
    position: absolute;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-right: 1.25rem;
    transition: top 300ms ease-out;
  }

  .sf-dot {
    position: relative;
    display: flex;
    height: 3rem;
    flex-direction: column;
    align-items: center;
    border: 0;
    background: none;
    padding: 0;
    cursor: pointer;
    transition: height 200ms;
  }

  .sf-dot[aria-current='true'] {
    height: 4.5rem;
  }

  .sf-dot > .sf-dot-mark {
    height: 1rem;
    width: 1rem;
    border-radius: 9999px;
    border: 2px solid #9b9b9b;
    background: #4e4e4e;
    transition: all 200ms;
  }

  .sf-dot[aria-current='true'] > .sf-dot-mark {
    border-color: #3b82f6;
    background: #3b82f6;
    transform: scale(1.25);
  }

  .sf-dot > .sf-dot-line {
    width: 0.25rem;
    height: 2rem;
    background: #676767;
    transition: height 200ms;
  }

  .sf-dot[aria-current='true'] > .sf-dot-line {
    height: 3.5rem;
  }

  .sf-dot > .sf-dot-label {
    pointer-events: none;
    position: absolute;
    top: 50%;
    left: 2rem;
    transform: translateY(-50%);
    white-space: nowrap;
    font: 500 0.75rem/1 ui-sans-serif, system-ui, sans-serif;
    color: #ececec;
    opacity: 0;
    transition: opacity 200ms;
  }

  .sf-dot:hover > .sf-dot-label {
    opacity: 1;
  }

  #sf-counter {
    position: fixed;
    right: 1.5rem;
    bottom: 1.5rem;
    z-index: 50;
    border-radius: 0.5rem;
    background: rgb(51 51 51 / 0.8);
    padding: 0.35rem 0.75rem;
    font: 500 0.8rem/1 ui-sans-serif, system-ui, sans-serif;
    color: #ececec;
    backdrop-filter: blur(4px);
  }

  @media print {
    /* The app's print rules target \`main\`; this deck's scroll container is
       \`#sf-deck\`, so it needs the same treatment to paginate. */
    #sf-deck {
      display: block !important;
      height: auto !important;
      overflow: visible !important;
      scroll-snap-type: none !important;
    }

    #sf-deck > section {
      display: flex !important;
      width: ${DESIGN_W}px !important;
      height: ${DESIGN_H}px !important;
      overflow: hidden !important;
      break-after: page;
      page-break-after: always;
    }

    #sf-deck > section:last-child {
      break-after: auto;
      page-break-after: auto;
    }
  }
`;

/**
 * Runtime for the exported deck. Deliberately dependency-free vanilla JS so the
 * file opens anywhere with no network.
 *
 * The scaling mirrors `view-slides/+page.svelte`: slides are laid out once at a
 * fixed design width and then uniformly scaled, and a single shrink factor is
 * derived from the *tallest* slide and applied to all of them, so text stays the
 * same size across the whole deck and no slide overflows.
 */
function deckScript(): string {
  return `
(function () {
  var DESIGN_W = ${DESIGN_W};
  var DESIGN_H = ${DESIGN_H};
  var PADDING = ${VIEWPORT_PADDING};
  var deck = document.getElementById('sf-deck');
  var sections = Array.prototype.slice.call(deck.querySelectorAll('section'));
  var dots = Array.prototype.slice.call(document.querySelectorAll('.sf-dot'));
  var railInner = document.getElementById('sf-rail-inner');
  var counter = document.getElementById('sf-counter');
  var index = 0;

  // 'screen' scales to the window; 'print' scales to the page, since the print
  // stylesheet sizes every slide to exactly the design resolution. Without the
  // print variant the printed deck would inherit whatever scale the browser
  // window happened to have when it was printed.
  function applyScale(mode) {
    var printing = mode === 'print';
    var scale = printing ? 1 : (window.innerWidth - 2 * PADDING) / DESIGN_W;
    var maxHeight = printing ? DESIGN_H : window.innerHeight - 2 * PADDING;

    // Measure every slide unscaled first, then shrink the whole deck by the
    // worst case. Measuring while a transform is applied would compound.
    var fit = 1;
    sections.forEach(function (section) {
      var stage = section.firstElementChild;
      stage.style.transform = 'none';
      var height = stage.firstElementChild.offsetHeight;
      var scaled = height * scale;
      if (scaled > maxHeight && scaled > 0) {
        fit = Math.min(fit, maxHeight / scaled);
      }
    });

    var finalScale = scale * fit;
    sections.forEach(function (section) {
      section.firstElementChild.style.transform = 'scale(' + finalScale + ')';
    });
  }

  function setIndex(next, scroll) {
    index = Math.max(0, Math.min(sections.length - 1, next));
    dots.forEach(function (dot, i) {
      dot.setAttribute('aria-current', i === index ? 'true' : 'false');
    });
    if (railInner) {
      railInner.style.top = window.innerHeight * 0.5 - index * 48 + 'px';
    }
    if (counter) {
      counter.textContent = index + 1 + ' / ' + sections.length;
    }
    if (scroll !== false) {
      sections[index].scrollIntoView({ behavior: scroll === 'instant' ? 'instant' : 'smooth' });
    }
    if (history.replaceState) {
      history.replaceState(null, '', '#slide-' + (index + 1));
    }
  }

  document.addEventListener('keydown', function (event) {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    var key = event.key;
    if (key === 'ArrowRight' || key === 'ArrowDown' || key === 'PageDown' || key === ' ') {
      event.preventDefault();
      setIndex(index + 1);
    } else if (key === 'ArrowLeft' || key === 'ArrowUp' || key === 'PageUp') {
      event.preventDefault();
      setIndex(index - 1);
    } else if (key === 'Home') {
      event.preventDefault();
      setIndex(0);
    } else if (key === 'End') {
      event.preventDefault();
      setIndex(sections.length - 1);
    } else if (key === 'f' || key === 'F') {
      event.preventDefault();
      if (document.fullscreenElement) {
        document.exitFullscreen();
      } else if (document.documentElement.requestFullscreen) {
        document.documentElement.requestFullscreen();
      }
    }
  });

  dots.forEach(function (dot, i) {
    dot.addEventListener('click', function () {
      setIndex(i);
    });
  });

  // Keep the rail and counter honest when the deck is scrolled directly
  // (trackpad, scrollbar) rather than driven by the keyboard.
  var scrollTimer;
  deck.addEventListener('scroll', function () {
    clearTimeout(scrollTimer);
    scrollTimer = setTimeout(function () {
      var nearest = Math.round(deck.scrollTop / window.innerHeight);
      if (nearest !== index) setIndex(nearest, false);
    }, 80);
  });

  window.addEventListener('resize', function () {
    applyScale('screen');
    setIndex(index, 'instant');
  });

  // Printing this file from a browser: rescale to the page, then back.
  window.addEventListener('beforeprint', function () {
    applyScale('print');
  });
  window.addEventListener('afterprint', function () {
    applyScale('screen');
    setIndex(index, 'instant');
  });

  applyScale('screen');

  // Deep link: #slide-3 opens on the third slide.
  var requested = parseInt((location.hash.match(/^#slide-(\\d+)$/) || [])[1], 10);
  setIndex(isNaN(requested) ? 0 : requested - 1, 'instant');

  // Images and fonts settle after first paint and can change slide heights.
  window.addEventListener('load', function () {
    applyScale('screen');
    setIndex(index, 'instant');
  });
})();
`;
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

/** Guard against a stylesheet or slide body closing the tag it sits inside. */
function escapeClosingTag(value: string, tag: string): string {
  return value.replace(new RegExp(`</${tag}`, 'gi'), `<\\/${tag}`);
}

function renderSlide(slide: Slide, index: number): string {
  // Mirrors `view-slides/Slide.svelte` so the harvested stylesheet applies
  // unchanged. The transform is left to the runtime, which computes one scale
  // for the whole deck once the slides can be measured.
  return `<section id="slide-${index + 1}" class="flex h-screen w-full items-center justify-center overflow-hidden ${escapeHtml(slide.bg_color)}" data-title="${escapeHtml(slide.title)}">
  <div class="flex shrink-0 items-center justify-center px-24" style="width: ${DESIGN_W}px; transform-origin: center center;">
    <article class="prose prose-xl prose-invert ${escapeHtml(slide.text_color)}">${slide.content}</article>
  </div>
</section>`;
}

function renderDot(slide: Slide, index: number, total: number): string {
  const line = index < total - 1 ? '<span class="sf-dot-line"></span>' : '';
  return `<button class="sf-dot" aria-current="false" aria-label="Go to slide ${index + 1}">
  <span class="sf-dot-mark"></span>${line}
  <span class="sf-dot-label">${escapeHtml(slide.title)}</span>
</button>`;
}

export interface StandaloneOptions {
  slides: Slide[];
  /** Document title, normally the deck's file name. */
  title: string;
}

/** Render the whole deck as a single self-contained HTML document. */
export function buildStandaloneHtml({ slides, title }: StandaloneOptions): string {
  const css = escapeClosingTag(harvestCss(), 'style');
  const deckCss = escapeClosingTag(DECK_CSS, 'style');

  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>${escapeHtml(title)}</title>
<style>${css}</style>
<style>${deckCss}</style>
</head>
<body>
<main id="sf-deck" class="select-none">
${slides.map(renderSlide).join('\n')}
</main>
<nav id="sf-rail" data-no-print>
  <div id="sf-rail-inner">
${slides.map((slide, index) => renderDot(slide, index, slides.length)).join('\n')}
  </div>
</nav>
<div id="sf-counter" data-no-print></div>
<script>${escapeClosingTag(deckScript(), 'script')}</script>
</body>
</html>
`;
}
