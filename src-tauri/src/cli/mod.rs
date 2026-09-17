//! Command line interface.
//!
//! Arguments are parsed in `main`, *before* `tauri::Builder` is ever
//! constructed. That ordering is the whole design, not an incidental detail:
//!
//! - [`Command::Validate`] and friends need no webview at all, so they run and
//!   exit without paying for one (see [`headless`]).
//! - [`Command::Export`] needs a webview configured quite differently from the
//!   GUI's — offscreen, undecorated, no dock icon — and the decision has to be
//!   made before the app is built.
//!
//! The official `tauri-plugin-cli` cannot express either case: its matches are
//! only readable from inside `setup()`, by which point the window described in
//! `tauri.conf.json` already exists.
//!
//! Two platform quirks are handled here rather than being left to leak into
//! every subcommand: macOS hands bundled apps a process-serial-number argument
//! that is not ours, and Windows release builds start with no console attached.

pub mod gui;
pub mod headless;

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

/// Process exit codes.
///
/// These are a contract, not an implementation detail: `docs/testing-platform-exports.md`
/// plans to assert on them from CI shell scripts, so they must stay stable.
pub mod exit {
    /// Everything worked.
    pub const OK: i32 = 0;
    /// A runtime failure: unreadable file, failed write, export reported an error.
    pub const FAILURE: i32 = 1;
    /// Bad arguments. Chosen by clap itself, and repeated here for documentation.
    pub const USAGE: i32 = 2;
    /// The deck could not be parsed.
    pub const PARSE_ERROR: i32 = 3;
    /// An export was started but never reported back in time.
    pub const TIMEOUT: i32 = 4;
}

/// Default seconds to wait for an export to report back.
///
/// Matches `PDF_TIMEOUT_MS` in `src/lib/export/export.svelte.ts`, so the CLI
/// watchdog and the frontend's own give up at the same point rather than racing.
pub const DEFAULT_TIMEOUT_SECS: u64 = 120;

#[derive(Debug, Parser)]
#[command(
    name = "slideflare",
    version,
    about = "Blazing fast, interactive presentations from Markdown"
)]
// Note the absence of `args_conflicts_with_subcommands`: it would make the
// global `--quiet` conflict with every subcommand, rejecting
// `slideflare --quiet validate deck.md`. The bare `[DECK]` positional and the
// subcommands are disambiguated in `dispatch` instead, which prefers an
// explicit subcommand.
pub struct Cli {
    /// Deck to open. Shorthand for `slideflare open <DECK>`.
    ///
    /// Ignored when a subcommand is given.
    #[arg(value_name = "DECK")]
    pub deck: Option<PathBuf>,

    /// Suppress informational output. Errors are still reported.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Open a deck in the presentation window.
    Open {
        #[arg(value_name = "DECK")]
        deck: PathBuf,
    },

    /// Render a deck to PDF or a self-contained HTML file.
    Export(ExportArgs),

    /// Check that a deck parses, without rendering it.
    Validate {
        #[arg(value_name = "DECK")]
        deck: PathBuf,
    },

    /// Parse a deck and print the resulting slides.
    Parse {
        #[arg(value_name = "DECK")]
        deck: PathBuf,

        /// Emit JSON — the same shape the `markdown-updated` event carries.
        #[arg(long)]
        json: bool,
    },

    /// Manage the `slideflare-slides` agent skill.
    Skill {
        #[command(subcommand)]
        command: SkillCommand,
    },

    /// Print a shell completion script to stdout.
    Completions {
        #[arg(value_name = "SHELL")]
        shell: Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum SkillCommand {
    /// Install or update the skill into `~/.agents/skills/`.
    Install {
        /// Override the install source recorded at build time.
        #[arg(long, value_name = "SOURCE")]
        source: Option<String>,
    },
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Output format.
    #[arg(value_name = "FORMAT")]
    pub format: ExportFormat,

    #[arg(value_name = "DECK")]
    pub deck: PathBuf,

    /// File to write.
    #[arg(short, long, value_name = "FILE")]
    pub output: PathBuf,

    /// Seconds to wait for the export to complete before giving up.
    #[arg(long, value_name = "SECS", default_value_t = DEFAULT_TIMEOUT_SECS)]
    pub timeout: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum ExportFormat {
    Pdf,
    Html,
}

impl ExportFormat {
    /// Wire name handed to the frontend, which picks an exporter from it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Html => "html",
        }
    }
}

/// Parse arguments from the real process environment.
pub fn parse() -> Cli {
    Cli::parse_from(filter_process_serial_number(std::env::args_os()))
}

/// Drop the `-psn_0_123456` argument macOS passes to bundled applications.
///
/// Launching SlideFlare from Finder or `open(1)` appends it. clap has no reason
/// to know about it and would reject the whole invocation as a usage error, so
/// the app would refuse to start when double-clicked — with the message going
/// nowhere, because there is no terminal attached.
fn filter_process_serial_number<I>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    args.into_iter()
        .filter(|arg| !arg.to_string_lossy().starts_with("-psn_"))
        .collect()
}

/// Attach to the terminal that launched us, on Windows.
///
/// `main.rs` sets `windows_subsystem = "windows"` for release builds so the GUI
/// does not drag a console window along with it. The side effect is that a
/// release binary starts with no stdout or stderr whatsoever, which would make
/// `--help`, `--version`, and every error message silently vanish. Attaching to
/// the parent's console restores them when one exists; when there is none (a
/// double-click) the call fails harmlessly and GUI mode carries on.
#[cfg(windows)]
pub fn attach_console() {
    use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

    // Failure is the normal case for a GUI launch, so the result is discarded.
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
pub fn attach_console() {}

/// Turn a user-supplied deck path into an absolute one, checking it is usable.
///
/// `std::path::absolute` is used rather than `canonicalize` deliberately: on
/// Windows `canonicalize` returns a `\\?\`-prefixed UNC path, which several
/// APIs downstream — and the window title — handle poorly. Symlinks are left
/// unresolved, which is the friendlier behaviour anyway.
///
/// `require_markdown` is set for the modes that feed the deck to the presentation
/// window, matching the drag-and-drop handler's own `.md` check. `validate` and
/// `parse` leave it off: editors and scripts routinely point them at temporary
/// files that carry no extension.
pub fn resolve_deck(path: &Path, require_markdown: bool) -> Result<PathBuf, String> {
    let absolute = absolute_path(path)?;

    if !absolute.is_file() {
        return Err(format!("{}: not a file", absolute.display()));
    }

    if require_markdown
        && !absolute
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        return Err(format!(
            "{}: expected a Markdown (.md) file",
            absolute.display()
        ));
    }

    Ok(absolute)
}

/// Resolve the output path for an export, creating nothing yet.
///
/// Only made absolute — the directory is created by `write_export` / `export_pdf`
/// at the point of writing, which already handle it.
pub fn resolve_output(path: &Path) -> Result<PathBuf, String> {
    absolute_path(path)
}

/// Make a path absolute and tidy, without consulting the filesystem.
///
/// `std::path::absolute` is used rather than `canonicalize` deliberately: on
/// Windows `canonicalize` returns a `\\?\`-prefixed UNC path, which several APIs
/// downstream — and the window title — handle poorly. It does not, however,
/// collapse `.` and `..`, which leaves paths like `/home/me/src-tauri/../examples/deck.md`
/// in error messages and window titles, so that is done here.
///
/// The collapse is purely lexical, so it can disagree with the filesystem when a
/// `..` follows a symlinked directory. That is the same trade-off `absolute`
/// already makes, and it only affects how the path is displayed and handed to
/// the frontend, never whether the file is found — the `is_file` check below
/// runs against the result.
fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = std::path::absolute(path).map_err(|e| format!("{}: {}", path.display(), e))?;

    let mut tidy = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                tidy.pop();
            }
            Component::CurDir => {}
            other => tidy.push(other),
        }
    }

    Ok(tidy)
}

/// Run the requested command and return the process exit code.
pub fn dispatch(cli: Cli) -> i32 {
    // A bare `slideflare deck.md` is `open` spelled shorter. An explicit
    // subcommand always wins over the positional.
    let command = match cli.command {
        Some(command) => command,
        None => Command::Open {
            deck: match cli.deck {
                Some(deck) => deck,
                // No arguments at all: the drag-and-drop screen, as always.
                None => return gui::run(None),
            },
        },
    };

    match command {
        Command::Open { deck } => match resolve_deck(&deck, true) {
            Ok(deck) => gui::run(Some(deck)),
            Err(message) => fail(&message),
        },

        Command::Export(args) => gui::run_export(args, cli.quiet),

        Command::Validate { deck } => headless::validate(&deck, cli.quiet),

        Command::Parse { deck, json } => headless::parse(&deck, json),

        Command::Skill {
            command: SkillCommand::Install { source },
        } => headless::install_skill(source, cli.quiet),

        Command::Completions { shell } => headless::completions_to_stdout(shell),
    }
}

/// Report a fatal message on stderr and yield the failure exit code.
pub fn fail(message: &str) -> i32 {
    eprintln!("slideflare: {message}");
    exit::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `try_parse_from`, not `parse_from`: the latter calls `process::exit` on a
    /// usage error, which would kill the whole test binary rather than fail one
    /// test.
    fn try_parse_args(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(filter_process_serial_number(
            args.iter().map(OsString::from).collect::<Vec<_>>(),
        ))
    }

    fn parse_args(args: &[&str]) -> Cli {
        try_parse_args(args).expect("arguments should parse")
    }

    #[test]
    fn bare_deck_is_treated_as_a_positional() {
        let cli = parse_args(&["slideflare", "deck.md"]);
        assert_eq!(cli.deck, Some(PathBuf::from("deck.md")));
        assert!(cli.command.is_none());
    }

    #[test]
    fn no_arguments_selects_neither_deck_nor_subcommand() {
        let cli = parse_args(&["slideflare"]);
        assert!(cli.deck.is_none());
        assert!(cli.command.is_none());
    }

    #[test]
    fn export_parses_format_output_and_default_timeout() {
        let cli = parse_args(&["slideflare", "export", "pdf", "deck.md", "-o", "out.pdf"]);
        match cli.command {
            Some(Command::Export(args)) => {
                assert_eq!(args.format, ExportFormat::Pdf);
                assert_eq!(args.deck, PathBuf::from("deck.md"));
                assert_eq!(args.output, PathBuf::from("out.pdf"));
                assert_eq!(args.timeout, DEFAULT_TIMEOUT_SECS);
            }
            other => panic!("expected an export command, got {other:?}"),
        }
    }

    #[test]
    fn export_accepts_an_explicit_timeout() {
        let cli = parse_args(&[
            "slideflare",
            "export",
            "html",
            "deck.md",
            "-o",
            "out.html",
            "--timeout",
            "5",
        ]);
        match cli.command {
            Some(Command::Export(args)) => {
                assert_eq!(args.format, ExportFormat::Html);
                assert_eq!(args.timeout, 5);
            }
            other => panic!("expected an export command, got {other:?}"),
        }
    }

    #[test]
    fn export_rejects_an_unknown_format() {
        assert!(try_parse_args(&["slideflare", "export", "docx", "d.md", "-o", "o.docx"]).is_err());
    }

    #[test]
    fn export_requires_an_output_path() {
        assert!(try_parse_args(&["slideflare", "export", "pdf", "d.md"]).is_err());
    }

    #[test]
    fn an_explicit_subcommand_beats_the_bare_deck_positional() {
        // `dispatch` reads `command` first, so whatever clap does with the
        // stray positional, the subcommand is what runs.
        let cli = parse_args(&["slideflare", "validate", "other.md"]);
        assert!(matches!(cli.command, Some(Command::Validate { .. })));
    }

    #[test]
    fn quiet_is_accepted_before_and_after_a_subcommand() {
        assert!(parse_args(&["slideflare", "--quiet", "validate", "d.md"]).quiet);
        assert!(parse_args(&["slideflare", "validate", "d.md", "--quiet"]).quiet);
    }

    #[test]
    fn macos_process_serial_number_argument_is_ignored() {
        // Finder appends this; clap would otherwise reject the whole launch.
        let cli = parse_args(&["slideflare", "-psn_0_774175"]);
        assert!(cli.deck.is_none());
        assert!(cli.command.is_none());
    }

    #[test]
    fn resolve_deck_rejects_a_missing_file() {
        let result = resolve_deck(Path::new("definitely-not-here.md"), true);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_deck_enforces_the_extension_only_when_asked() {
        // `Cargo.toml` exists but is not Markdown, which is exactly the contrast
        // being checked: presentation modes insist, `validate`/`parse` do not.
        let path = Path::new("Cargo.toml");
        assert!(resolve_deck(path, true).is_err());
        assert!(resolve_deck(path, false).is_ok());
    }

    #[test]
    fn resolve_deck_returns_an_absolute_path() {
        let resolved = resolve_deck(Path::new("Cargo.toml"), false).expect("Cargo.toml exists");
        assert!(resolved.is_absolute());
    }

    #[test]
    fn resolve_deck_collapses_parent_and_current_directory_segments() {
        // `src-tauri/../examples/x.md` should not survive into an error message
        // or the window title.
        let resolved = resolve_deck(Path::new("../examples/example.md"), true)
            .expect("the shipped example exists");

        assert!(!resolved.to_string_lossy().contains(".."));
        assert!(resolved.ends_with("examples/example.md"));
    }
}
