# TODO:

- [ ] add better default styling
  - [ ] to heading tags
- [ ] add the cool intro example (make it prettier)
- [ ] add security implications to prevent a melicious slide deck from doing XSS and other types of attacks
  - [ ] use ammonia
- [ ] run tests on building release
- [ ] add easy to use fonts support
- [ ] add easy to use rtl support
- [ ] add shiki magic move
- [ ] add code syntax highlighting
- [ ] add mermaid diagrams
- [ ] add multipart slides
- [ ] add themes, use JSON to create themes
- [ ] add easy way to convert slides from other platforms to slideflare:
  - [ ] get the pdf output of ther platforms and convert them to slideflare markdown using mistral OCR or other OCR tools that support images (mistral takes screenshots of the things that are not convertable to markdown)

- [ ] make slideflare faster: \
  - [ ] pase everything at once rather when rendering whole file and parse per slide when doing incremental updates \
  - [ ] use tokio instead for asynchronous media processing
  - [ ] only process images/videos of the current slide or current + next slide (maybe remove previously loaded content after moving to the next slide)

- [ ] complete the documentation
- [ ] write more tests and move them into a separate folder
- [ ] Do not open links inside the app, open them in the browser, (ask for confirmation before opening the link)

# Testing:

- [x] centralize the versioning so you don't have to change 10 different numbers manually
  - [x] also add some github actions to update the AUR versions
- [x] a convenience option to install/update the slideflare skill after an update to the app or on first launch (maybe inside the tutorial section where we introduce it)
- [ ] test the examples provided in the readme
- [ ] windowless export on Windows and macOS — written, never run (see `docs/headless-export.md`)
  - [x] Windows: `cli::gui::render_hidden` keeps the HWND hidden and forces
        `ICoreWebView2Controller::SetIsVisible(true)`
  - [x] macOS: `cli::gui::render_unoccluded` goes borderless, switches off
        `-[NSApplication _setWindowOcclusionDetectionEnabled:]` and moves the
        window offscreen, since it has to stay ordered in
  - [x] frontend safety net: `nextFrames()` in `src/lib/export/export.svelte.ts`
        is capped, and `waitForDeckReady` settles images and waits for the slide
        measurements to stop moving before it trusts them
  - [x] fidelity gate in `export-smoke`: render `examples/intro-to-slideflare.md`
        twice on the same runner, once with `SLIDEFLARE_EXPORT_WINDOW=visible`,
        and require identical pixels via `scripts/pdf-pixel-diff.py`
  - [ ] **watch that gate go green on the Windows and macOS runners.** Neither
        path has been built, let alone run, on the platform it targets; this
        machine has no MSVC toolchain and no macOS SDK. Until CI says otherwise,
        both are unproven, and every claim about what WebView2 and AppKit do is
        a citation rather than a measurement.
  - [ ] if macOS fails: `_setWindowOcclusionDetectionEnabled:` is private and
        probed with `respondsToSelector:`, so a missing selector degrades to a
        visible window rather than failing. The next thing to try is a
        zero-alpha window at its normal frame — public API, and the check is
        cheap.
- [ ] `<video>` never renders in a GTK windowless export. A `GtkOffscreenWindow`
      starts no media pipeline, so the element never reports metadata and prints
      collapsed. `main` did the same, but only by racing the print against the
      load and winning. Decide whether a printed deck should show a poster frame,
      then make it deliberate. `examples/example.md:103` is the case; readiness
      deliberately does not wait on `loadedmetadata` because on GTK it never
      comes and the wait would only cost every video deck its cap.
- [x] add to AUR
- [x] the reload button should reread the file and do the whole parsing pipeline assuming that something went wrong
- [x] fix screen resizing issue
- [x] create a logo and use it instead of generic tauri icon

# Later

- [ ] add custom code component

# Done

- [x] fix the lowercase/capital s on the program title
- [x] write a better readme
- [x] fix the licensing
- add [tailwindcss-typography](https://github.com/tailwindlabs/tailwindcss-typography) to style the html parsed from the markdown
- [x] add pulldown-cmark for markdown parsing
  - [x] enable all the extensions (enabled gfm, math, frontmatter(yaml support))
  - [x] add custom slides parsing
- [x] check for bugs in the hashing system
  - there were actually no bugs in the hashing system
  - the problem was with the screen refreshing after any slide change
- [x] add a more permanent fix for the screen flashing and make sure the dark mode is getting handled properly
- fixed file watcher and termination logic
- on resize scroll to the active slide with no animation (to keep it persistent when changing resolution or window size)
- make sure same file won't get processed twice, used a non-cryptographic hash function
- [x] handle image/video paths correctly
  - [x] image
  - [x] video
  - [x] fix handling of absolute paths
  - [x] add proper styling to images/videos
- [x] Update the incremental.rs tests
- [x] replace temp incremental.rs implementations with imara-diff
- [x] Performance Bug (watcher.rs:66 & 94): compute_slide_hashes() called twice, causing unnecessary double parsing
- [x] fix jumping on the first slide after an edit
  - use the hash to know what the correct slide is
- [x] add an error screen for when the syntax is not correct
  - [x] make sure it displays the error when opening new slides
  - [x] make sure it displays an error pop up on the opened slide when editing
  - [x] fix: the new slide validator is detecting an error for the perfectly fine example in the example.md
- [x] markdown next to a html line won't get detected (check if there's a possible fix)
  - not possible, it's a CommonMark spec
- [x] update the readme, verify links work and remove katex from acknowledgments
  - [x] mention the AUR installation
- [x] fix the `---` parsing problem
- [x] find a solution for white or close to white backgrounds that make the text inisible
  - [x] the text color property is not getting applied to the heading tags
- [x] add a back button in UI + esc as shorcut
- [x] add a tutorial that pops op on the first use and on every update when new features are added
- [x] publish the AI skill
  - [x] mention it in the readme
- [x] add a help menu
- [x] add convert to pdf
  - native per-platform webview printing: WebKitGTK print-to-file, WebView2 `PrintToPdf`, WKWebView `NSPrintOperation`
- [x] add export to a self-contained HTML file (single file, offline, keyboard-navigable)
