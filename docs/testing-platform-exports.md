# Testing the platform-dependent export code

Status as of 2026-09-17. Tiers 1 and 2 are implemented, along with the cheap
half of tier 3; the rest of tier 3 and tier 4 are planned.

**Both exports now pass on all three platforms.** That is new: before this, only
the Linux PDF backend had ever been executed. Getting there took four CI rounds
and turned up two genuine macOS bugs, one of them in code that had shipped
untested — which is the entire argument for tier 2.

## The problem

PDF export is implemented three times — once per platform — against three
unrelated native APIs:

| Platform    | Backend                          | Entry point                           |
| ----------- | -------------------------------- | ------------------------------------- |
| Linux / BSD | WebKitGTK + GTK print            | `src-tauri/src/export/pdf_linux.rs`   |
| Windows     | WebView2 `PrintToPdf`            | `src-tauri/src/export/pdf_windows.rs` |
| macOS       | `WKWebView` + `NSPrintOperation` | `src-tauri/src/export/pdf_macos.rs`   |

Until Tier 2 landed, only the Linux backend had ever been executed; Windows and
macOS type-checked against the real APIs but were runtime-unverified. All three
are now exercised on every push — and the macOS one turned out to be broken.

Worse, until Tier 1 landed, the only workflow that compiled the Rust side at all
(`.github/workflows/tauri-action.yml`) triggered on `push: tags: ['app-v*']`.
A Windows- or macOS-only compile break was therefore discovered _at release
time, on a tag_ — the most expensive possible moment.

This is not hypothetical. During implementation, `printOperationWithPrintInfo:`
turned out to live inside an `#[cfg(feature = "objc2-app-kit")] impl WKWebView`
block, so `objc2-web-kit` needed `features = ["WKWebView", "objc2-app-kit"]`.
That bug was caught only because a throwaway scratch crate was built by hand to
type-check the macOS path. Nothing in the repo would have caught it.

## Tier 1 — compile coverage on every push (implemented)

`.github/workflows/ci.yml` runs on push to `main`, on pull requests, and on
manual dispatch.

Two jobs:

- **frontend** (ubuntu only) — `bun run lint`, `bun run check`, `bun run build`.
- **rust** (matrix: `ubuntu-22.04`, `windows-latest`, `macos-latest`) —
  `cargo fmt --check` (Linux only; formatting is platform-independent),
  `cargo clippy --all-targets -- -D warnings`, `cargo test`.

This is the highest value per unit of effort by a wide margin. It needs no
webview, no display server, and no printer, yet it catches the entire class of
bug that actually occurred: wrong feature flag, wrong signature, missing `cfg`
arm. It also compiles `pdf_unsupported.rs`, which no other job ever reaches.

Two implementation notes, both verified empirically rather than assumed:

- The Rust jobs do **not** need the frontend built. `tauri.conf.json` sets
  `frontendDist: "../build"` and `/build` is gitignored, so the obvious worry is
  that `generate_context!` would fail on a fresh checkout. It does not — this
  was tested by moving `build/` aside and forcing both `lib.rs` and `build.rs`
  to recompile. So the Rust job skips bun entirely.
- The Linux runner needs `libwebkit2gtk-4.1-dev` and friends installed before
  `cargo check`, because the `webkit2gtk`, `gtk`, and `glib` crates resolve
  system libraries through `pkg-config` at build time. Windows and macOS need no
  extra system packages.

### Known limit

The matrix checks one native target per OS. Release builds both
`aarch64-apple-darwin` and `x86_64-apple-darwin`, so an arch-specific break on
macOS could still escape. Judged acceptable: the platform-conditional code is
gated on `target_os`, never on architecture, and adding a second Apple target
would recompile every dependency for marginal coverage.

## Tier 2 — runtime smoke test, via the CLI (implemented)

Compile coverage proves the code builds, not that it prints. For that, a real
webview has to run on each OS.

`.github/workflows/ci.yml` gained an `export-smoke` job, matrixed over the same
three runners, which builds the frontend and the binary and then runs:

```
slideflare export html examples/example.md -o out.html
slideflare export pdf  examples/example.md -o out.pdf
```

under `xvfb-run` on Linux and directly elsewhere, asserting the exit code and the
output. Both exports are also kept as build artifacts, so a failure can be looked
at rather than only read about.

They are separate steps, HTML first, deliberately. They exercise different halves
of the feature — HTML harvests the runtime stylesheet, PDF drives the platform
print pipeline — and while they shared one step a PDF failure hid whether HTML
had worked at all. That is not hypothetical either: it is what obscured, for two
rounds, that macOS was fine apart from the print call.

### Why the CLI was the right vehicle

A CLI was planned for the project, and it is a strictly better vehicle for this
test than a dedicated test harness, so Tier 2 was deliberately deferred until it
landed rather than building a harness that would immediately rot.

What the CLI **cannot** remove: the webview _is_ the PDF renderer.
`print_to_pdf` takes a `&WebviewWindow` and calls `with_webview` to reach the
real `WKWebView` / `ICoreWebView2` / `webkit2gtk::WebView`. No webview, no PDF.
A CLI changes who pushes the button, not what renders.

What the CLI **does** remove, which is the expensive half:

- No `tauri-driver` / WebDriver dependency — which had no macOS support anyway,
  meaning it would have covered the two platforms that need it least.
- No test-only binary to drift out of sync with the shipping code path.
- CI exercises the code users actually run.
- Fixture path, output path, and exit code become plain argv and process exit,
  trivially assertable from a shell script.

`cargo test` still cannot do this — it needs a real webview and a display — but
`slideflare export pdf fixture.md -o out.pdf` as a CI step is far cleaner than a
bespoke harness.

### Display requirements stay, per platform

Unchanged by the CLI:

- **Linux** — GTK needs `DISPLAY` or `WAYLAND_DISPLAY`; `gtk_init` fails
  outright without one. Requires `xvfb-run`.
- **Windows** — WebView2 needs an `HWND` to create a controller. A hidden window
  is fine. **Confirmed working**: the first `export-smoke` run passed on
  `windows-latest`, so the WebView2 runtime is present on the image and
  `PrintToPdf` completes without an interactive desktop. This was the open
  question; it is now closed.
- **macOS** — `NSPrintOperation` needs AppKit with a window server session, which
  the runners do provide. The activation policy is set to accessory to avoid a
  dock icon. `runOperation()` was called the least-trusted piece of the whole
  feature and the first runs bore that out: it hangs, and the backend now uses
  `runOperationModalForWindow:` instead. **Confirmed working** — see below.

### What the first runs found on macOS — and the fix

macOS was the only platform that failed, and it failed twice over. Both are
fixed; all three platforms now pass both exports. The path there is worth
recording, because almost every cheap explanation turned out to be wrong.

**Run 1 — a build break.** `tauri::generate_context!` had been expanded twice,
once per CLI mode. Each expansion emits an `_EMBED_INFO_PLIST` symbol, so the
link failed with ``symbol `_EMBED_INFO_PLIST` is already defined``. It is
macOS-only and completely invisible elsewhere: `cargo clippy --all-targets -D
warnings` and `cargo test` both pass on Linux with two expansions present. Tier 1
caught it. `cli::gui::run_app` now holds the single expansion.

**Runs 1–3 — the print hang.** `export-smoke` exited 4, the watchdog's timeout,
with nothing else logged. Diagnosis took three rounds because the obvious causes
were all innocent:

- _Not readiness._ A readiness failure reports itself and exits 1. Markers later
  showed the deck ready in 9s.
- _Not the deck._ HTML export on the same runner succeeded, producing 5.3MB —
  so parsing, layout, fonts and the runtime stylesheet harvest all work there.
- _Not occlusion._ The window was being parked offscreen, and AppKit does treat
  a fully offscreen window as occluded, which suspends WebKit rendering. Fixing
  that was correct and is kept — `configure_export_window` no longer moves the
  window offscreen on macOS — but it did not fix the hang.
- _Not a missing printer._ The suspicion was the macOS analogue of the `lpr`
  trap below: a runner with no printer configured. Dumping the whole print
  dictionary disproved it — the runner resolves a real `NSPrinter`, and
  `NSJobDisposition`, `NSJobSavingURL` and `NSSavePath` are all set correctly.
- _Not the thread._ `with_webview` does dispatch to the main thread, and
  `setCanSpawnSeparateThread(false)` changed nothing.

What remained was the call itself. `printOperationWithPrintInfo:` returned in
20ms; `runOperation()` then never returned at all. It blocks the calling thread
in a nested run loop while WebKit asks its web content process for a page count,
and on this runner that reply is never serviced.

The fix is `runOperationModalForWindow:delegate:didRunSelector:contextInfo:` —
exactly what the earlier draft of this document guessed at. It schedules the job
and returns immediately, so the main run loop stays free to deliver the reply,
and reports the outcome through a delegate declared with `define_class!` in
`pdf_macos.rs`. Success is still only claimed once the print genuinely finished,
now from the callback rather than a return value, so the silent-success hazard
stays closed.

The print went from hanging for 120s to completing in **0.84s**.

`pdf_macos.rs` keeps its stderr markers around the operation. This backend cannot
be stepped through on the machines that usually build it, and a hang is otherwise
indistinguishable from a deck that never became ready — those two lines are what
turned round three from a guess into a diagnosis.

### A macOS-only build trap: `generate_context!`

`tauri::generate_context!` must be expanded **exactly once per crate**. Each
expansion emits an `_EMBED_INFO_PLIST` symbol, so a second one fails the link
with ``symbol `_EMBED_INFO_PLIST` is already defined``.

This is macOS-only, and it is invisible locally on any other platform: a full
`cargo clippy --all-targets -D warnings` and `cargo test` pass on Linux with two
expansions present. Tier 1 compile coverage is what caught it. `cli::gui::run_app`
now funnels both modes through a single expansion.

### The design catch, and how it turned out

As documented in `src-tauri/src/export.rs`, PDF export deliberately prints the
**live, realized** webview, because layout, font resolution, and MathML
measurement are already settled there. An unrealized widget carries no such
guarantee from either GTK or WebKit.

So a naive `visible: false` window is the risky choice — on GTK an unmapped
window may never realize. The shape built first, in `cli::gui::run_export`, was
a window that stays realized but is undecorated, kept out of the taskbar, and
moved to `(-10000, -10000)`. Two caveats found while building it:

- Positioning is **best-effort**. Wayland has no global window coordinates, so
  compositors there ignore the move and the window is simply visible while the
  render runs. Cosmetic, and irrelevant to CI, which runs under `xvfb` — an X
  server, where the move works.
- An offscreen window **is not guaranteed to be painted**, and animation frames
  may stop being delivered to it entirely. Anything waiting on
  `requestAnimationFrame` therefore needs a timer to fall back on; the first
  working version of this hung indefinitely on exactly that.

**Half of that turned out to be wrong, and GTK no longer works this way.** See
`docs/headless-export.md` for the measurements; the corrections are:

- Printing does **not** need a realized widget. On WebKitGTK 2.52 a
  `WebKitWebView` that was never added to a container, never realized and never
  mapped still prints correct, paginated, selectable PDF.
- What actually breaks in a hidden window is not the print, it is the
  **measurement** the deck does before it. `visibilityState` follows the
  container; in a webview the engine calls hidden, images never settle their
  layout box, so the deck measures short, `fitScale` stays 1, and every slide
  prints over-sized and clipped — at exit code 0.
- `cli::gui::render_offscreen` therefore renders into a `GtkOffscreenWindow`,
  which the engine treats as a normal visible 1280x720 page (animation frames
  included) while nothing is ever mapped on the compositor. The toplevel is
  hidden and emptied. Both caveats above disappear with it: there is no position
  for Wayland to ignore, and frames keep arriving.
- This does **not** remove the display-server requirement. `gtk_init` still fails
  outright with no `DISPLAY`/`WAYLAND_DISPLAY`, so CI keeps `xvfb-run`.

Windows and macOS are unchanged and still use the realized-window shape.

The readiness signal — the one genuine piece of new design work — lives in
`src/routes/view-slides/+page.svelte` as `waitForDeckReady`, emitting `deck-ready`
or `deck-failed`. It waits for four things in order:

1. a parse delivered by **this** process (`shared.generation > 0`), not merely a
   non-empty deck,
2. every slide having reported its height, so each has been laid out and measured
   and `fitScale` is derived from a complete set,
3. `document.fonts.ready`, since text measured against a fallback face reflows
   when the real one loads, and
4. two animation frames, so the scale from (2) has been applied and not just
   computed.

Condition (1) needed a change beyond the CLI: `shared.slides` used to be seeded
from `localStorage`, which made "there are slides" meaningless at startup — those
slides belonged to whichever deck was open last, so a readiness check could pass
on the **wrong document** and export it. The cache is gone; Rust re-parses and
re-sends on every launch regardless, so nothing was lost.

It is polled rather than watched reactively. `waitForDeckReady` is awaited from an
async `onMount`, and effects created in a detached `$effect.root` from there are
not reliably re-run — which presented as an export patiently waiting out its whole
timeout while the finished deck sat rendered behind it.

### The other trap: `custom-protocol`

Tauri chooses between `devUrl` and the bundled frontend from the
`custom-protocol` feature **alone** — `dev: cfg!(not(feature = "custom-protocol"))`
in `tauri-macros` — and not from `debug_assertions`. A plain `cargo build
--release` therefore produces a binary that loads `http://localhost:1420`, and
every export fails with `Connection refused` inside an empty window.

`src-tauri/Cargo.toml` now declares the standard passthrough feature, CI builds
with `--features custom-protocol`, and `run_export` prints a warning naming the
missing feature when it is absent, so the next person meets a sentence rather
than a blank window.

### Exit codes are part of the contract

`cli::exit` fixes them: `0` success, `1` failure, `2` usage, `3` the deck did not
parse, `4` the export timed out. Two failure modes are worth calling out because
both would otherwise be silent successes:

- Tauri exits the process with `0` when the last window closes. A window closed
  by hand, or a webview that dies on startup, would look exactly like a
  successful export. `run_export` intercepts `RunEvent::Exit` and fails unless
  the export has genuinely reported back.
- An export that never reports at all is caught by a watchdog thread, rather than
  leaving an invisible window running until the CI job's own timeout kills it
  with nothing to diagnose.

### HTML export has the same shape

`harvestCss()` in `src/lib/export/standalone.ts` walks `document.styleSheets` to
capture the stylesheet Tailwind's browser build generates **at runtime**. There
is no stylesheet on disk to read instead. So CLI HTML export also needs a live
webview — either evaluating `buildStandaloneHtml()` inside it, or a Rust-side
reimplementation that would immediately drift. Evaluate it in the webview.

Good news: same window, same readiness signal. One mechanism serves both
exports.

## Tier 3 — what to assert (partly implemented)

This matters more than the harness does. The dangerous failure mode is not a
crash; it is a PDF that is produced but blank. That is silent, and a page-count
check will not catch it.

Assertions, in order of importance:

1. File exists and is non-trivial in size.
2. Page count equals slide count.
3. Every `MediaBox` equals `[0 0 960 540]`.
4. The dominant colour of page N matches that slide's frontmatter `bg_color`.

Number 4 is the real regression guard: it is what catches `print-color-adjust`
breaking, or a white/empty page. Items 2 and 3 are cheap regex over the PDF
bytes and work on all three runners. Item 4 needs a rasterizer (`pdftoppm` is
trivial on Linux, awkward elsewhere), so run structure checks everywhere and the
colour check on Linux only.

Avoid pixel-golden diffs. Font rasterization differs per platform and they will
flap.

**Built so far:** item 1 on all three runners (both exports), and items 2 and 3
on Linux via `pdfinfo`, with the expected slide count taken from `slideflare
validate` rather than hardcoded so the fixture can grow.

**Item 4 is still open in CI**, though it has now been run by hand on both
platforms that produce a PDF differently. Page 1 of `examples/example.md`
rasterizes to a dominant `rgb(25, 60, 184)` against the `bg-blue-800`
(`#1e40af`) the frontmatter asks for — and it is the _same_ value from the Linux
GTK backend and from the macOS Quartz one, which says `print-color-adjust` is
working and neither is emitting a blank page. Worth automating on Linux, since
it is the only assertion that catches the failure mode this document calls the
dangerous one.

For the record, the macOS artifact also satisfies items 2 and 3 by hand: 7 pages
for 7 slides, `960 x 540 pts`. Only the Linux job asserts those automatically,
because that is the runner with poppler available without fuss.

## Tier 4 — coverage automation cannot reach

Short pre-release manual pass:

- A Linux box **with** CUPS configured. The development machine had no `lpr` at
  all, so the happy path with a real default printer is unexercised.
- Wayland and X11 separately.
- An old WebView2 runtime, to confirm the version-cast error message is what
  users actually see.
- A deck with embedded video.
- A non-English locale (see below), if not automated.

## The locale risk

`print_to_file_printer()` in `pdf_linux.rs` resolves GTK's virtual printer name
through `glib::dgettext(Some("gtk30"), "Print to File")`, because GTK registers
that printer under a _translated_ name. Setting `output-uri` alone is not
enough — without naming the printer, GTK falls through to the default backend
and tries to spawn `lpr`, which is exactly the failure this guards against.

In an English locale `dgettext` trivially returns its input, so the lookup is
currently untested in the one situation it exists for. Running the Linux smoke
test a second time under `LC_ALL=de_DE.UTF-8`, with GTK's message catalogues
installed, would actually prove it.

## Testing the HTML export

Mostly platform-independent and much easier. `buildStandaloneHtml` is nearly
pure, so a unit test could cover escaping, slide markup, and structure with a
stubbed `document.styleSheets`.

Note honestly what that would _not_ have caught: the single-page bug, where
`#sf-deck` inherited `flex flex-col` while also being given `height: 100%`, so
every `h-screen` child was flex-shrunk into one viewport. That was a layout
failure visible only when rendered. The check that would have caught it is
loading the export in WebKitGTK and asserting page count — and that one runs
fine on Linux CI.
