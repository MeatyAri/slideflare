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
