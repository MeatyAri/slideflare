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

# No test framework configured yet (see TODO.md)
```

## Tech Stack

- **Frontend**: SvelteKit 5 + TypeScript + Tailwind CSS 4
- **Backend**: Rust with Tauri 2.0
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
