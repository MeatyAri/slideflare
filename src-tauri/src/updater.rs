//! Update checking and skill installation.
//!
//! Two independent things are checked:
//!   - the app itself, via GitHub. Stable installs compare the latest release
//!     tag against `CARGO_PKG_VERSION`; git installs compare the upstream
//!     default-branch head against the commit baked in at build time.
//!   - the `slideflare-slides` agent skill, installed into `~/.agents/skills/`
//!     either through a package runner (`bunx`/`npx`) or by unpacking the repo
//!     tarball.
//!
//! How the app was installed decides how it can be updated, so the install
//! source is baked in at compile time by `build.rs`. When it is missing the
//! frontend asks the user to pick one and stores the answer.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

const APP_REPO: &str = "MeatyAri/slideflare";
const SKILL_REPO: &str = "MeatyAri/slideflare-slides";
const SKILL_NAME: &str = "slideflare-slides";
const USER_AGENT: &str = concat!("slideflare/", env!("CARGO_PKG_VERSION"));

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Unset at build time when the commit could not be determined, in which case a
/// git-channel install has nothing to compare against.
const BUILD_COMMIT: Option<&str> = option_env!("SLIDEFLARE_GIT_COMMIT");
/// Unset at build time -> empty, which maps to `InstallSource::Unknown`.
const BUILD_INSTALL_SOURCE: Option<&str> = option_env!("SLIDEFLARE_INSTALL_SOURCE");

/// Where this build came from, which determines how it updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallSource {
    /// Release artifact from GitHub Actions.
    Github,
    /// Arch `slideflare` package.
    Aur,
    /// Arch `slideflare-git` package, tracks HEAD.
    AurGit,
    Npm,
    Bun,
    Cargo,
    /// Local build from a checkout.
    Source,
    /// Not recorded at build time; the user picks and we remember.
    Unknown,
}

impl InstallSource {
    fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "github" => Self::Github,
            "aur" => Self::Aur,
            "aur-git" | "aurgit" => Self::AurGit,
            "npm" => Self::Npm,
            "bun" => Self::Bun,
            "cargo" => Self::Cargo,
            "source" => Self::Source,
            _ => Self::Unknown,
        }
    }

    /// True when this install tracks the default branch rather than releases.
    fn tracks_git(self) -> bool {
        matches!(self, Self::AurGit | Self::Source)
    }

    /// Package runner to install the skill with, when one applies.
    fn skill_runner(self) -> Option<&'static str> {
        match self {
            Self::Bun => Some("bunx"),
            Self::Npm => Some("npx"),
            _ => None,
        }
    }

    /// Shell command that updates this install, for display only. `None` when
    /// there is no single correct command to show.
    fn update_command(self) -> Option<&'static str> {
        match self {
            Self::Aur => Some("paru -S slideflare"),
            Self::AurGit => Some("paru -S slideflare-git"),
            Self::Npm => Some("npm install -g slideflare"),
            Self::Bun => Some("bun install -g slideflare"),
            Self::Cargo => Some("cargo install slideflare"),
            Self::Source => Some("git pull && bun run tauri build"),
            Self::Github | Self::Unknown => None,
        }
    }
}

/// Result of an update check, for the frontend.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    /// Install source in effect (build-time value, or the user's override).
    pub install_source: InstallSource,
    /// True when the source was never recorded, so the UI should ask.
    pub install_source_unknown: bool,
    /// Version string of the running build: semver, or short commit on git.
    pub current: String,
    /// Latest upstream version: release tag, or short commit on git.
    pub latest: Option<String>,
    pub app_update_available: bool,
    /// Command that updates this install, when one can be named.
    pub update_command: Option<String>,
    /// Release page / commit URL to open.
    pub release_url: Option<String>,
    /// Set when the check itself failed (offline, rate limited, ...).
    pub error: Option<String>,
    pub skill_installed: bool,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

#[derive(Deserialize)]
struct CommitRef {
    sha: String,
}

fn short(sha: &str) -> String {
    sha.chars().take(7).collect()
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| format!("could not create HTTP client: {e}"))
}

/// Strip the `app-v` / `v` prefix the release tags carry.
fn version_from_tag(tag: &str) -> &str {
    tag.trim_start_matches("app-v").trim_start_matches('v')
}

/// Compare dotted versions numerically. `true` when `candidate` is newer.
fn is_newer(current: &str, candidate: &str) -> bool {
    let parts = |v: &str| -> Vec<u64> {
        v.split(['.', '-'])
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    let (a, b) = (parts(current), parts(candidate));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (
            a.get(i).copied().unwrap_or(0),
            b.get(i).copied().unwrap_or(0),
        );
        if y != x {
            return y > x;
        }
    }
    false
}

/// `~/.agents/skills/`
fn skills_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|h| h.join(".agents").join("skills"))
        .ok_or_else(|| "could not locate home directory".to_string())
}

fn skill_installed() -> bool {
    skills_dir()
        .map(|d| d.join(SKILL_NAME).join("SKILL.md").exists())
        .unwrap_or(false)
}

/// The install source in effect: the build-time value, or the user's stored
/// choice when the build didn't record one.
fn effective_source(user_choice: Option<&str>) -> (InstallSource, bool) {
    let built_in = BUILD_INSTALL_SOURCE
        .map(InstallSource::parse)
        .unwrap_or(InstallSource::Unknown);
    if built_in != InstallSource::Unknown {
        return (built_in, false);
    }
    match user_choice.map(InstallSource::parse) {
        Some(choice) if choice != InstallSource::Unknown => (choice, false),
        _ => (InstallSource::Unknown, true),
    }
}

async fn latest_release() -> Result<Release, String> {
    let url = format!("https://api.github.com/repos/{APP_REPO}/releases/latest");
    let res = client()?
        .get(url)
        .send()
        .await
        .map_err(|e| format!("could not reach GitHub: {e}"))?;
    let res = check_status(res)?;
    res.json::<Release>()
        .await
        .map_err(|e| format!("unexpected release data: {e}"))
}

/// Head commit of the repo's default branch.
async fn latest_commit() -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{APP_REPO}/commits/HEAD");
    let res = client()?
        .get(url)
        .send()
        .await
        .map_err(|e| format!("could not reach GitHub: {e}"))?;
    let res = check_status(res)?;
    Ok(res
        .json::<CommitRef>()
        .await
        .map_err(|e| format!("unexpected commit data: {e}"))?
        .sha)
}

fn check_status(res: reqwest::Response) -> Result<reqwest::Response, String> {
    let status = res.status();
    if status.is_success() {
        Ok(res)
    } else if status.as_u16() == 403 || status.as_u16() == 429 {
        Err("GitHub rate limit reached, try again later".to_string())
    } else {
        Err(format!("GitHub returned {status}"))
    }
}

/// Check for app and skill updates.
///
/// `userInstallSource` is the source the user picked previously, used only when
/// the build didn't record one.
#[tauri::command]
pub async fn check_updates(user_install_source: Option<String>) -> UpdateStatus {
    let (source, unknown) = effective_source(user_install_source.as_deref());
    let installed_skill = skill_installed();

    let mut status = UpdateStatus {
        install_source: source,
        install_source_unknown: unknown,
        current: match (source.tracks_git(), BUILD_COMMIT) {
            (true, Some(commit)) => short(commit),
            _ => CURRENT_VERSION.to_string(),
        },
        latest: None,
        app_update_available: false,
        update_command: source.update_command().map(str::to_string),
        release_url: None,
        error: None,
        skill_installed: installed_skill,
    };

    if source.tracks_git() {
        let Some(build_commit) = BUILD_COMMIT else {
            status.error = Some("this build did not record a commit, cannot compare".into());
            return status;
        };
        match latest_commit().await {
            Ok(sha) => {
                status.app_update_available = sha != build_commit;
                status.latest = Some(short(&sha));
                status.release_url = Some(format!("https://github.com/{APP_REPO}/commits"));
            }
            Err(e) => status.error = Some(e),
        }
    } else {
        match latest_release().await {
            Ok(release) => {
                let latest = version_from_tag(&release.tag_name).to_string();
                status.app_update_available = is_newer(CURRENT_VERSION, &latest);
                status.latest = Some(latest);
                status.release_url = Some(release.html_url);
            }
            Err(e) => status.error = Some(e),
        }
    }

    status
}

/// Install or update the `slideflare-slides` skill.
///
/// Uses `bunx`/`npx` when the install source implies one is present, otherwise
/// downloads the repo tarball and unpacks it into `~/.agents/skills/`.
#[tauri::command]
pub async fn install_skill(user_install_source: Option<String>) -> Result<String, String> {
    let (source, _) = effective_source(user_install_source.as_deref());

    if let Some(runner) = source.skill_runner() {
        if which::which(runner).is_ok() {
            return run_skill_runner(runner);
        }
    }
    install_skill_from_tarball().await
}

fn run_skill_runner(runner: &str) -> Result<String, String> {
    let out = Command::new(runner)
        .args(["skills", "add", SKILL_REPO])
        .output()
        .map_err(|e| format!("could not run {runner}: {e}"))?;
    if out.status.success() {
        Ok(format!("Skill installed with {runner}."))
    } else {
        Err(format!(
            "{runner} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Download the skill repo and install it into `~/.agents/skills/`.
///
/// The skill is just Markdown (`SKILL.md` plus a README today), but it is
/// installed as a directory rather than a single file: that is the layout the
/// skills ecosystem expects, and it keeps working if the repo later grows
/// reference files. Grabbing the repo tarball gets that in one request without
/// hardcoding a file list that would silently miss new files.
async fn install_skill_from_tarball() -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{SKILL_REPO}/tarball/HEAD");
    let res = client()?
        .get(url)
        .send()
        .await
        .map_err(|e| format!("could not download skill: {e}"))?;
    let res = check_status(res)?;
    let bytes = res
        .bytes()
        .await
        .map_err(|e| format!("could not read download: {e}"))?;

    let dest_root = skills_dir()?;
    std::fs::create_dir_all(&dest_root)
        .map_err(|e| format!("could not create {}: {e}", dest_root.display()))?;

    // Unpack next to the target, then swap, so a failed download never leaves a
    // half-written skill behind.
    let staging = dest_root.join(format!(".{SKILL_NAME}.new"));
    if staging.exists() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    std::fs::create_dir_all(&staging).map_err(|e| format!("could not create staging dir: {e}"))?;

    let unpacked = unpack_tarball(&bytes, &staging).inspect_err(|_| {
        let _ = std::fs::remove_dir_all(&staging);
    })?;

    let dest = dest_root.join(SKILL_NAME);
    let backup = dest_root.join(format!(".{SKILL_NAME}.old"));
    if dest.exists() {
        let _ = std::fs::remove_dir_all(&backup);
        std::fs::rename(&dest, &backup)
            .map_err(|e| format!("could not move existing skill aside: {e}"))?;
    }
    match std::fs::rename(&unpacked, &dest) {
        Ok(()) => {
            let _ = std::fs::remove_dir_all(&backup);
            let _ = std::fs::remove_dir_all(&staging);
            Ok(format!("Skill installed to {}.", dest.display()))
        }
        Err(e) => {
            // Put the previous version back rather than leaving nothing.
            if backup.exists() {
                let _ = std::fs::rename(&backup, &dest);
            }
            let _ = std::fs::remove_dir_all(&staging);
            Err(format!("could not install skill: {e}"))
        }
    }
}

/// Unpack a gzipped tar into `into`, returning the single top-level directory
/// GitHub wraps its tarballs in.
fn unpack_tarball(bytes: &[u8], into: &Path) -> Result<PathBuf, String> {
    let decoder = flate2::read::GzDecoder::new(Cursor::new(bytes));
    tar::Archive::new(decoder)
        .unpack(into)
        .map_err(|e| format!("could not unpack skill archive: {e}"))?;

    let mut entries = std::fs::read_dir(into)
        .map_err(|e| format!("could not read unpacked archive: {e}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir());

    let root = entries
        .next()
        .ok_or_else(|| "skill archive was empty".to_string())?;
    if entries.next().is_some() {
        return Err("unexpected skill archive layout".to_string());
    }
    // A download that arrived intact but carries no SKILL.md is not a skill;
    // refuse it rather than replacing a working install with something inert.
    if !root.join("SKILL.md").exists() {
        return Err("downloaded archive contains no SKILL.md".to_string());
    }
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_sources() {
        assert_eq!(InstallSource::parse("github"), InstallSource::Github);
        assert_eq!(InstallSource::parse("AUR"), InstallSource::Aur);
        assert_eq!(InstallSource::parse("aur-git"), InstallSource::AurGit);
        assert_eq!(InstallSource::parse(" bun "), InstallSource::Bun);
        assert_eq!(InstallSource::parse("nonsense"), InstallSource::Unknown);
        assert_eq!(InstallSource::parse(""), InstallSource::Unknown);
    }

    #[test]
    fn only_git_sources_track_head() {
        assert!(InstallSource::AurGit.tracks_git());
        assert!(InstallSource::Source.tracks_git());
        assert!(!InstallSource::Aur.tracks_git());
        assert!(!InstallSource::Github.tracks_git());
        assert!(!InstallSource::Unknown.tracks_git());
    }

    #[test]
    fn strips_release_tag_prefixes() {
        assert_eq!(version_from_tag("app-v0.1.1"), "0.1.1");
        assert_eq!(version_from_tag("v0.1.1"), "0.1.1");
        assert_eq!(version_from_tag("0.1.1"), "0.1.1");
    }

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("0.1.1", "0.1.2"));
        assert!(is_newer("0.1.9", "0.2.0"));
        assert!(is_newer("0.9.0", "1.0.0"));
        // 10 > 9 numerically, where a string compare would disagree.
        assert!(is_newer("0.9.0", "0.10.0"));
        assert!(!is_newer("0.1.1", "0.1.1"));
        assert!(!is_newer("0.2.0", "0.1.9"));
    }

    #[test]
    fn user_choice_only_applies_when_build_is_unknown() {
        // The test binary is built without SLIDEFLARE_INSTALL_SOURCE.
        if BUILD_INSTALL_SOURCE.is_none() {
            assert_eq!(effective_source(None), (InstallSource::Unknown, true));
            assert_eq!(effective_source(Some("aur")), (InstallSource::Aur, false));
            assert_eq!(
                effective_source(Some("garbage")),
                (InstallSource::Unknown, true)
            );
        }
    }

    #[test]
    fn runner_only_for_package_manager_installs() {
        assert_eq!(InstallSource::Bun.skill_runner(), Some("bunx"));
        assert_eq!(InstallSource::Npm.skill_runner(), Some("npx"));
        assert_eq!(InstallSource::Aur.skill_runner(), None);
        assert_eq!(InstallSource::Unknown.skill_runner(), None);
    }
}

#[cfg(test)]
mod install_smoke {
    /// Hits the network; run explicitly with --ignored.
    #[tokio::test]
    #[ignore]
    async fn installs_skill_into_home() {
        let tmp = std::env::temp_dir().join("sf_skill_smoke");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("HOME", &tmp);

        let msg = super::install_skill_from_tarball().await.unwrap();
        println!("MSG: {msg}");

        let dest = tmp.join(".agents/skills/slideflare-slides");
        println!("SKILL.md exists: {}", dest.join("SKILL.md").exists());
        println!("README.md exists: {}", dest.join("README.md").exists());
        let mut leftovers: Vec<String> = std::fs::read_dir(tmp.join(".agents/skills"))
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        leftovers.sort();
        println!("skills dir contents: {leftovers:?}");
    }
}
