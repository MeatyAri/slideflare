//! Subcommands that never build a `tauri::Builder`.
//!
//! Everything here runs against the same parsing and updating code the app
//! uses, but with no window, no webview, and no event loop — so they start in
//! milliseconds and work over SSH, in a container, and inside an editor's
//! on-save hook.
//!
//! Export is deliberately *not* here: the webview is the renderer, both for PDF
//! (the platform print pipelines take a live `WebviewWindow`) and for HTML (the
//! stylesheet Tailwind's browser build generates exists only at runtime). See
//! [`super::gui`].

use std::io::Write;
use std::path::Path;

use clap::CommandFactory;
use clap_complete::Shell;

use super::{exit, fail, resolve_deck, Cli};
use crate::parser::{parse_markdown_with_frontmatter, Slide};

/// Read a deck and parse it, reporting failures the way the CLI should.
///
/// The base directory is the deck's own parent, matching what the file watcher
/// passes in `watcher::send_new_file`, so relative image and video paths resolve
/// exactly as they would in the app.
fn read_and_parse(deck: &Path) -> Result<Vec<Slide>, (i32, String)> {
    let deck = resolve_deck(deck, false).map_err(|e| (exit::FAILURE, e))?;

    let content = std::fs::read_to_string(&deck)
        .map_err(|e| (exit::FAILURE, format!("{}: {}", deck.display(), e)))?;

    let base_dir = deck
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    parse_markdown_with_frontmatter(&content, &base_dir)
        .map_err(|e| (exit::PARSE_ERROR, format!("{}: {}", deck.display(), e)))
}

/// Write finished output to stdout.
///
/// A closed pipe is treated as success, not failure. `slideflare parse deck.md |
/// head` is an ordinary way to use these commands, and Rust ignores `SIGPIPE` at
/// startup, so the default behaviour would be a panic and a backtrace where a
/// Unix tool is expected to simply stop.
fn write_stdout(text: &str) -> i32 {
    match std::io::stdout().write_all(text.as_bytes()) {
        Ok(()) => exit::OK,
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => exit::OK,
        Err(e) => fail(&format!("could not write output: {e}")),
    }
}

/// Check that a deck parses. Prints nothing on success unless asked.
pub fn validate(deck: &Path, quiet: bool) -> i32 {
    match read_and_parse(deck) {
        Ok(slides) => {
            if !quiet {
                println!(
                    "ok: {} slide{}",
                    slides.len(),
                    if slides.len() == 1 { "" } else { "s" }
                );
            }
            exit::OK
        }
        Err((code, message)) => {
            eprintln!("slideflare: {message}");
            code
        }
    }
}

/// Parse a deck and print the result.
///
/// `--json` emits the identical payload the `markdown-updated` event carries,
/// so anything written against the app's event stream can be driven from the
/// CLI without a second serialization format to keep in step.
pub fn parse(deck: &Path, json: bool) -> i32 {
    let slides = match read_and_parse(deck) {
        Ok(slides) => slides,
        Err((code, message)) => {
            eprintln!("slideflare: {message}");
            return code;
        }
    };

    if json {
        match serde_json::to_string_pretty(&slides) {
            Ok(payload) => write_stdout(&format!("{payload}\n")),
            Err(e) => fail(&format!("could not serialize slides: {e}")),
        }
    } else {
        write_stdout(&render_summary(&slides))
    }
}

/// Human-readable rundown of a parsed deck.
fn render_summary(slides: &[Slide]) -> String {
    let mut out = String::new();

    for (index, slide) in slides.iter().enumerate() {
        let title = if slide.title.is_empty() {
            "(untitled)"
        } else {
            &slide.title
        };
        out.push_str(&format!("{:>3}  {}\n", index + 1, title));
        if !slide.bg_color.is_empty() {
            out.push_str(&format!("     bg   {}\n", slide.bg_color));
        }
        if !slide.text_color.is_empty() {
            out.push_str(&format!("     text {}\n", slide.text_color));
        }
        out.push_str(&format!("     html {} bytes\n", slide.content.len()));
    }

    out
}

/// Install or update the `slideflare-slides` agent skill.
///
/// `updater::install_skill` is a `#[tauri::command]`, but it is a plain async
/// function underneath and takes no app handle, so it runs here unchanged —
/// the GUI dialog and the CLI drive exactly the same code.
pub fn install_skill(source: Option<String>, quiet: bool) -> i32 {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(e) => return fail(&format!("could not start async runtime: {e}")),
    };

    match runtime.block_on(crate::updater::install_skill(source)) {
        Ok(message) => {
            if !quiet {
                println!("{message}");
            }
            exit::OK
        }
        Err(message) => fail(&message),
    }
}

/// Write a shell completion script.
///
/// Takes the sink rather than assuming stdout so tests can generate into a
/// buffer instead of burying the test output in shell script.
pub fn completions(shell: Shell, out: &mut dyn Write) -> i32 {
    let mut command = Cli::command();
    let name = command.get_name().to_string();
    clap_complete::generate(shell, &mut command, name, out);
    exit::OK
}

/// Write a shell completion script to stdout.
///
/// Generated into a buffer first because `clap_complete::generate` writes to the
/// sink incrementally and panics outright if a write fails — so piping into
/// `head`, which every shell's install instructions encourage people to try,
/// would end in a backtrace. Buffering routes the single write through
/// [`write_stdout`], where a closed pipe is simply the end of the output.
pub fn completions_to_stdout(shell: Shell) -> i32 {
    let mut script = Vec::new();
    completions(shell, &mut script);
    write_stdout(&String::from_utf8_lossy(&script))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Path to a file in the repo's `examples/` directory.
    ///
    /// Tests run with the crate root (`src-tauri/`) as the working directory,
    /// so the examples live one level up.
    fn example(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("src-tauri has a parent")
            .join("examples")
            .join(name)
    }

    #[test]
    fn the_shipped_examples_parse() {
        for name in ["example.md", "intro-to-slideflare.md"] {
            let slides = read_and_parse(&example(name))
                .unwrap_or_else(|(_, e)| panic!("{name} should parse: {e}"));
            assert!(!slides.is_empty(), "{name} produced no slides");
        }
    }

    #[test]
    fn validate_accepts_the_shipped_examples() {
        assert_eq!(validate(&example("example.md"), true), exit::OK);
    }

    #[test]
    fn a_missing_deck_is_a_failure_not_a_parse_error() {
        // The distinction matters to CI: exit 1 means "could not read it",
        // exit 3 means "read it and it is wrong".
        let (code, _) = read_and_parse(Path::new("no-such-deck.md")).unwrap_err();
        assert_eq!(code, exit::FAILURE);
    }

    #[test]
    fn completions_generate_for_every_supported_shell() {
        // Really a guard on the clap definition: a duplicate long flag or an
        // invalid name only surfaces when a generator walks the command tree.
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
            let mut script = Vec::new();
            assert_eq!(completions(shell, &mut script), exit::OK);
            assert!(!script.is_empty(), "{shell} produced an empty script");
        }
    }

    #[test]
    fn the_summary_lists_every_slide() {
        let slides = read_and_parse(&example("example.md")).expect("example.md parses");
        let summary = render_summary(&slides);
        for (index, _) in slides.iter().enumerate() {
            assert!(
                summary.contains(&format!("{:>3}  ", index + 1)),
                "slide {} missing from the summary",
                index + 1
            );
        }
    }

    #[test]
    fn completions_reach_stdout_without_panicking() {
        assert_eq!(completions_to_stdout(Shell::Fish), exit::OK);
    }

    #[test]
    fn the_command_tree_is_internally_consistent() {
        Cli::command().debug_assert();
    }
}
