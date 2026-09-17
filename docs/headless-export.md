# Headless export — what is possible, and what landed

Status: **Linux/BSD only.** Implemented and verified against `main` (7c4255c).
**Windows and macOS are untested and unchanged** — they still render into a real
window, and nothing here has been run on either. Testing and porting them is a
CI job, not a local one; it is tracked in `TODO.md`.

Everything under "Evidence" and "Verification" was measured on one machine: Arch,
Wayland session, WebKitGTK 2.52.5, GTK 3.24.52, GTK 4.22.4.

## The question

`slideflare export pdf` used to boot the full app, create a real toplevel window,
move it to `(-10000, -10000)`, and print the live webview. Could it instead
render with no window at all, the way `chromium --headless` does?

Two different things get called "headless", and they have different answers:

1. **No window** — nothing is ever mapped on the compositor, nothing flashes on
   screen, no offscreen-coordinate trick, no Wayland caveat.
2. **No display server** — the binary runs over SSH or in a bare container with
   no `DISPLAY`, no `WAYLAND_DISPLAY`, no `xvfb`.

## Verdict

| | Linux/BSD | Windows | macOS |
| --- | --- | --- | --- |
| No window | **Done — output identical to the windowed build** | Likely, untested | Uncertain, three ranked experiments |
| No display server | **Impossible** with WebKitGTK | Needs a window station (interactive session) | Needs an Aqua session |

The short version: **(1) is achievable and has landed on GTK; (2) is not
achievable with system webviews and should be dropped as a goal.** The system
webview *is* the renderer, and on every platform it is a UI-toolkit widget whose
existence requires a display connection. Chromium can go display-free because it
ships its own rasteriser and a headless platform layer; WebKitGTK, WKWebView and
WebView2 do not expose one.

## Evidence

### Linux: a display connection is mandatory

```
$ env -u DISPLAY -u WAYLAND_DISPLAY ./hlprobe offscreen
PROBE gtk_init: FAIL Failed to initialize GTK
```

Bypassing the Rust binding's `assert_initialized_main_thread!` and calling the C
API directly does not help — it crashes rather than degrading:

```
$ env -u DISPLAY -u WAYLAND_DISPLAY ./raw
RAW gtk_init_check -> 0
RAW creating webview...
(raw): Gtk-CRITICAL: _gtk_style_provider_private_get_settings: assertion failed
Segmentation fault (core dumped)
```

GTK 3 has no headless GDK backend. `GDK_BACKEND=broadway` is compiled into
`libgtk-3.so.0` here but needs a running `broadwayd`, so it is not display-free
either. GTK 4.22's `libgtk-4.so.1` exports only `gdk_x11_display_open` and
`_gdk_wayland_display_open` — no headless backend there either, so moving to
`webkitgtk-6.0` would not change the answer.

### Linux: a *window* is not needed at all

A `WebKitWebView` that was **never added to any container, never realized, never
mapped** prints a correct PDF:

```
$ ./hlprobe orphan
PROBE gtk_init: OK
PROBE load: finished
PROBE print: FINISHED
$ pdfinfo probe.pdf
Pages: 2        Page size: 960 x 540 pts
$ pdftotext probe.pdf -
HEADLESS PROBE √ page one
two
```

Two pages from a `page-break-before`, exact 960x540 paper, selectable text — real
vector output through the normal GTK print path, with no window. The producer
string is `Skia/PDF m145`, so WebKitGTK 2.52 paginates through Skia and does not
consult the compositor at all.

This contradicts the note that stood in `docs/testing-platform-exports.md`: an
unrealized widget prints fine. What is unsafe is something else.

### Linux: the *container* decides what the web content sees

Same probe, measuring what JavaScript observes, across four container shapes:

| container | `innerWidth x innerHeight` | `visibilityState` | rAF in 3s | layout reads |
| --- | --- | --- | --- | --- |
| `GtkOffscreenWindow`, shown | `1280x720` | `visible` | **3** | ok |
| toplevel, realized, never mapped | `1280x720` | `hidden` | **0** | ok |
| orphan widget + `set_size_request` | `0x0` | `hidden` | **0** | ok |
| orphan widget | `0x0` | `hidden` | **0** | ok |

`document.fonts.ready` resolved and `offsetHeight` was correct in every case, so
fonts and synchronous layout are safe everywhere. Animation frames and the
viewport size are not.

`GtkOffscreenWindow` is the outlier that matters: the only shape where the web
content believes it is a normal visible 1280x720 page, while nothing reaches the
compositor.

### Why `visible: false` on its own is a trap

Hiding the toplevel and changing nothing else exits `0` and produces a
plausible-looking 7-page PDF that is silently wrong: slides that should have been
shrunk to fit are rendered at full size and clipped by the page.

![left: baseline; right: hidden window, title clipped and image off the page](./images/headless-hidden-window-regression.png)

_Left: the windowed build. Right: the same slide with the window hidden — the
deck failed to shrink it, so the title is cut off and the image runs off the
page._

Root cause is the deck's self-measurement. `Slide.svelte` reports its natural
content height with `bind:clientHeight`; `+page.svelte` derives one `fitScale`
for the whole deck from those heights. Two follow-up experiments narrowed it:

- Polling `article.clientHeight` every 50ms instead of relying on the
  `ResizeObserver` behind `bind:clientHeight` — **no change**. Observer delivery
  is not the bottleneck.
- Additionally awaiting `HTMLImageElement.decode()` on every image before
  declaring readiness — **six of seven pages became pixel-identical**. So the
  under-measurement is images not having settled, not observers not firing.
- The one page still wrong was the `<video>` slide (`examples/example.md:103`),
  which has no equivalent of `decode()` in the readiness path.

This is the class of failure `docs/testing-platform-exports.md` calls the
dangerous one — success exit code, wrong file — and it is why the implementation
uses a container the engine keeps rendering rather than one that merely hides.

## What landed (Linux/BSD)

`cli::gui::render_offscreen`, reached from `configure_export_window` behind a
`gtk_platform` cfg that `build.rs` sets from the same target list Cargo.toml
gates `webkit2gtk`/`gtk`/`glib` on:

1. `window.hide()` — the toplevel stays, unmapped and empty, because Tauri's
   window bookkeeping and the `RunEvent::Exit` guard in `run_app` key off it.
2. Through `with_webview`, remove the `WebKitWebView` from its GTK parent.
3. Add it to a `GtkOffscreenWindow` sized `EXPORT_WIDTH`x`EXPORT_HEIGHT` and
   `show_all()` it — that realizes and allocates it, which is what gives the page
   its viewport.
4. Park the offscreen window in a `thread_local`, since it now owns the webview.

`OFFSCREEN`, `set_position` and `set_skip_taskbar` are now Windows-only; the
GTK path has no toplevel to move. No frontend change was needed.

## Verification

Reference is a binary built from `main` at 7c4255c on the same machine, with the
same frontend build. `pdftoppm` at the stated resolution, then
`magick compare -metric AE` per page; `AE` is the count of differing pixels, so
`0` is exact.

| check | result |
| --- | --- |
| `main` vs `main`, `example.md` (determinism of the baseline itself) | 7/7 pages `AE=0` @100dpi |
| `main` vs offscreen, `example.md` | **7/7 pages `AE=0` @150dpi** |
| `main` vs offscreen, `example.md` | **7/7 pages `AE=0` @300dpi** |
| `main` vs offscreen, `intro-to-slideflare.md` | **12/12 pages `AE=0` @150dpi** |
| `main` vs offscreen, second offscreen run (reproducibility) | 7/7 pages `AE=0` @150dpi |
| `main` vs offscreen, `export html` | **byte-identical** (`cmp` clean, same md5) |

Contracts and checks that also still hold: `cargo fmt --check`, `cargo clippy
--all-targets -- -D warnings` (with and without `custom-protocol`), `cargo test`
(55 passed, 1 ignored), exit `1` on a missing deck, exit `4` on `--timeout 1`,
and presentation mode still opening a normal window.

`examples/example.md` is the deck that matters most here: it is the one with a
raster image and a `<video>`, the two elements the hidden-window failure showed
up in.

## Still to do

### Phase 0 — make readiness frame-independent (frontend)

Not required for GTK, but it is what would make any hidden-window fallback safe
on the other platforms, and it fixes a real latent bug in the GUI path:

- `src/lib/export/export.svelte.ts` — `nextFrames()` there has **no timer cap**,
  unlike the one in `src/routes/view-slides/+page.svelte`. In any webview that
  stops delivering animation frames, PDF export hangs before it ever invokes
  `export_pdf`. Give it the same 1s cap. Worth doing on its own merits.
- `waitForDeckReady` gains a media settle step after `document.fonts.ready`:
  `await Promise.allSettled(Array.from(document.images).map((i) => i.decode()))`,
  plus a `loadedmetadata`/`readyState >= 1` wait with a short per-element timeout
  for `<video>` and `<iframe>`, then one more measurement pass.
- Add the settled-media state to `readinessState()` so a timeout names it.

### Phase 2 — Windows: keep the controller visible, hide the HWND

WebView2's rendering is gated on `ICoreWebView2Controller::IsVisible`, which is a
property of the controller, **not** of the parent HWND being shown. So the
analogue of the GTK trick is: build the export window with `visible: false`, then
`put_IsVisible(TRUE)` on the controller through `with_webview` so the renderer
keeps running. `PrintToPdf` needs no change.

Unknowns to settle in CI, not locally: whether WebView2 really keeps producing
rendering updates for a controller marked visible inside a never-shown HWND, and
whether the viewport comes out `1280x720` or `0x0`. Phase 0 is the fallback that
makes the output correct either way.

### Phase 3 — macOS: three ranked experiments

Do **not** switch to `WKWebView.createPDF`. The module comment in
`src-tauri/src/export/pdf_macos.rs` records why: `WKPDFConfiguration` carries only
a capture rect, so it snapshots one continuous page and ignores the print
stylesheet entirely. Recovering per-slide pagination would mean a macOS-only rect
loop plus a PDF merge — a second, divergent renderer for one platform.

Keeping the current print operation, try in order:

1. **Disable occlusion detection.** `-[NSApplication _setWindowOcclusionDetectionEnabled:]`
   with `NO`, then park the window offscreen as on Windows. Private API,
   long-lived, used by Chromium and Electron for the same reason. Flag it if the
   app is ever submitted to the App Store.
2. **Zero-alpha window.** `alphaValue = 0`, `ignoresMouseEvents = true`, left at
   its normal frame. Public API. Needs verifying that AppKit does not then call
   it occluded — if it does, dead end, and the check is cheap.
3. **Status quo.** Keep the window visible, accessory activation policy. No
   regression, no win.

### Phase 4 — CI

`xvfb-run` **stays** on Linux: this removes the window, not the display
requirement. Add a fidelity assertion to `export-smoke` so a silent mis-scale
cannot pass: render the example deck, `pdftoppm` it, and `compare -metric AE`
against committed reference PNGs. That is the check that would have caught the
hidden-window regression; nothing currently in CI would.

## Rejected alternatives

- **Bare wry / bare WebKitGTK for export, skipping Tauri.** wry can build a
  webview into any GTK container (`build_gtk`), so the rendering half is easy.
  The rest is not: the deck reaches its content through Tauri's invoke and asset
  protocols, so this means reimplementing IPC and ending up with the parallel,
  drifting code path the CLI was designed to avoid.
- **Drive a headless Chromium over CDP.** Genuinely display-free and
  cross-platform, but it adds an external runtime dependency and a *second
  renderer*, so PDF output would no longer match what the user sees in the app.
  Worth revisiting only if a server-side render service becomes a product goal,
  and then as an explicitly separate `--renderer` backend.
- **WPE WebKit.** Designed for exactly this (embedded/headless, no X or Wayland)
  and the same engine, so fidelity would hold. There is no maintained Rust
  binding and no wry support; it would mean owning the bindings. Research note,
  not a plan.
- **Auto-spawning `Xvfb`/`cage`/`weston --backend=headless` when `DISPLAY` is
  unset.** Cheap, and it does make `slideflare export` work over SSH. It is not
  headless rendering, it is a virtual display, and it adds a dependency-detection
  path. Reasonable as a separate convenience; the honest alternative is a clear
  error naming `xvfb-run`.

## Reproducing the comparison

```
bun run build
cd src-tauri && cargo build --features custom-protocol
./src-tauri/target/debug/slideflare export pdf examples/example.md -o cand.pdf
# then against a binary built from main:
pdftoppm -png -r 150 ref.pdf /tmp/a && pdftoppm -png -r 150 cand.pdf /tmp/b
magick compare -metric AE /tmp/a-1.png /tmp/b-1.png null:
```

Note the ordering: `frontendDist` is embedded at *Rust* compile time, so a
frontend-only rebuild changes nothing until `cargo build` runs again. That cost
one round of confusing results during this spike.
