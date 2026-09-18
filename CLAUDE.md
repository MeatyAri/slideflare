# SlideFlare - Agent Development Guide

## Build & Development Commands

```bash
# Development
bun run tauri dev              # Start development server with hot reload
bun run dev                    # Frontend only development

# Building
bun run tauri build            # Build production app
bun run build                  # Frontend build only

# Code Quality
bun run lint                   # Run ESLint + Prettier check
bun run format                 # Format code with Prettier
bun run check                  # TypeScript type checking
bun run check:watch            # Type checking with watch mode

# Testing
bun run test                   # Run Rust unit tests (cargo test)
bun run test:watch             # Run tests with watch mode

# CLI (see "Command line interface" below)
cargo build --release --features custom-protocol   # the ONLY way to get a working export binary

# Benchmarks — cd into `src-tauri/` first, then run `cargo bench <filter>`
cargo bench                  # Run all benchmarks
cargo bench split            # Benchmark slide splitting only
cargo bench hash             # Benchmark slide hashing only
cargo bench diff             # Benchmark diff computation only
cargo bench parse            # Benchmark full parsing only
```

## Tech Stack

- **Frontend**: SvelteKit 5 + TypeScript + Tailwind CSS 4
- **Backend**: Rust with Tauri 2.0 — `src-tauri/` (parser, incremental diff, file watcher)
- **Benchmarking**: Criterion framework in `src-tauri/benches/slideflare_benchmarks.rs`
- **Styling**: Tailwind CSS with typography plugin
- **Math**: MathML for LaTeX rendering

## Code Style Guidelines

### TypeScript/Svelte

- Use Svelte 5 `$props()` and `$state()` runes
- Strict TypeScript mode enabled
- Interface definitions for all props and data structures
- Single quotes, 2-space indentation, 100 char line width
- Semicolons required

### Rust

- 2021 edition standard
- Module organization: lib.rs, parser.rs, watcher.rs
- Error handling with `expect()` for critical failures

### File Structure

- Frontend: `src/routes/` for pages, components created next to where they're required
- General reusable components: `src/lib/components/`
- Backend: `src-tauri/src/` for Rust modules
- Static assets in `static/`, examples in `examples/`

### Imports & Dependencies

- Use absolute imports from `@tauri-apps/api` for Tauri functions
- Import Tailwind classes via CSS, not in components
- Use shared state with `.svelte.ts` files for reactive state management

### Component Patterns

- Define Props interfaces explicitly
- Use `@html` directive for rendered markdown content
- Prose classes for markdown styling: `prose prose-invert lg:prose-xl`
- Background/text colors via Tailwind classes from YAML frontmatter

## Key Architecture Notes

- Markdown parsing with pulldown-cmark + LaTeX support via pulldown-latex
- File watching with notify crate for hot reload
- Event-driven communication between Rust backend and Svelte frontend
- Static site generation via @sveltejs/adapter-static for Tauri compatibility

## Command line interface

Lives in `src-tauri/src/cli/`. Arguments are parsed with clap in `main.rs`
**before** `tauri::Builder` exists, which is what lets `validate`/`parse`/`skill`
run with no webview at all and lets `export` configure its own window.

- `cli/mod.rs` — argument definitions, exit codes, path resolution, dispatch.
- `cli/headless.rs` — `validate`, `parse`, `skill install`, `completions`. Never
  constructs a `tauri::Builder`.
- `cli/gui.rs` — boots the app for both presentation and export mode.

Adding a command: add a variant to `Command`, handle it in `dispatch`, and — if
it needs a window — register any new `#[tauri::command]` in `cli::gui::builder`
using its **full path** (`crate::export::foo`). A bare name will not resolve
there; the hidden `__cmd__*` macro is re-exported from the defining module.

Three things that will bite:

- **`--features custom-protocol` is mandatory for export.** Tauri picks `devUrl`
  over the bundled frontend from that feature alone (`dev: cfg!(not(feature =
"custom-protocol"))` in `tauri-macros`), _not_ from `debug_assertions`. Without
  it, even a `--release` build loads `localhost:1420` and every export fails with
  "Connection refused" in an empty window.
- **Exit codes are a contract** (`cli::exit`), asserted by CI: `0` ok, `1`
  failure, `2` usage, `3` parse error, `4` timeout. Tauri exits `0` when the last
  window closes, so `run_export` intercepts `RunEvent::Exit` and fails unless the
  export actually reported back.
- **The export shows no window on any platform**, and the reason it can is not
  the same twice. Printing never needed a window; what does need one is the deck
  measuring itself, because a surface the engine calls hidden never settles image
  layout and every slide then prints over-sized at exit code 0 — a wrong file,
  not an error. So `cli::gui::configure_export_window` gives each engine whatever
  makes it call the content visible: GTK reparents the webview into a
  `GtkOffscreenWindow` (`render_offscreen`, no window at all); Windows keeps the
  HWND hidden and forces `ICoreWebView2Controller::SetIsVisible(true)`
  (`render_hidden`); macOS must keep its window ordered in, so it goes borderless,
  switches off AppKit occlusion detection and moves offscreen
  (`render_unoccluded`). A display server is still required on Linux —
  `gtk_init` fails without one.
  **Only the GTK path has ever been run.** Windows and macOS are unverified; the
  fidelity gate in `.github/workflows/ci.yml` is what has to prove them. It
  renders the deck twice, once with `SLIDEFLARE_EXPORT_WINDOW=visible` (also the
  user-facing escape hatch back to a real window), and requires identical pixels
  — but first it requires each run to _report_ the path it took, because all
  three windowless paths fall back to that same visible window and a silent
  fallback would make both sides of the comparison the same render. Every export
  prints `slideflare: export render mode: <token>`; keep those tokens stable,
  CI matches on them. See `docs/headless-export.md`.
- **`tauri::generate_context!` may be expanded only once in the crate.** Every
  expansion emits an `_EMBED_INFO_PLIST` symbol, and a second one fails the macOS
  link — invisible on Linux and Windows, so CI is what tells you. `run_app` holds
  the single call.

Export mode hands the work to the frontend rather than reimplementing it: the
deck signals `deck-ready` from `waitForDeckReady` in
`src/routes/view-slides/+page.svelte`, and the CLI then calls the same
`exportPdf`/`exportHtml` the NavBar buttons call, with a path instead of a save
dialog. See `docs/testing-platform-exports.md` for why.

## Versioning

`package.json` is the single source of truth for the app version; `tauri.conf.json`, the frontend, and `Cargo.toml` all derive from it. Never edit versions in more than one place.

- Change the version: `npm version patch` (or `minor` / `major` / an explicit `x.y.z`). This bumps `package.json`, syncs everything else via the `version` lifecycle hook, and makes the commit + tag.
- Hand-edited `package.json` instead: run `bun run sync-version` to propagate.
- Recover from drift (versions out of sync): set `package.json` to the desired version, then `bun run sync-version`.

## Tutorial (first-launch + what's-new)

Version-gated onboarding shown on the home screen only. Lives in `src/lib/tutorial/`.

- Add a feature card: append a `TutorialFeature` to `TUTORIAL_FEATURES` in `features.ts`. Keep `id` stable and unique forever — gating is keyed on it. Optional `media` path is relative to `static/`. `version` is cosmetic (used only for the "what's new" header label).
- Gating is automatic and id-based: a card is shown until the user dismisses a tutorial that included it (tracked in `tutorialSeenFeatures`). Fresh installs see the full tour; afterwards only unseen cards appear as "what's new". This works for release and HEAD-tracking git/AUR builds alike — a new card surfaces the moment its entry lands, no version bump required.
- Cards are packed into a carousel (short one-liners grouped, `media`/long bodies get their own slide) — no layout work needed.
