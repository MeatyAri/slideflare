use std::process::Command;

fn main() {
    // Install source baked in at compile time (see updater::InstallSource for
    // values). Leave unset -> "unknown", which means the app asks the user to
    // pick their install source and remembers it.
    println!("cargo:rerun-if-env-changed=SLIDEFLARE_INSTALL_SOURCE");
    if let Ok(source) = std::env::var("SLIDEFLARE_INSTALL_SOURCE") {
        println!("cargo:rustc-env=SLIDEFLARE_INSTALL_SOURCE={source}");
    }

    // Commit this binary was built from. Git-channel installs compare this
    // against the upstream default-branch head to detect updates. Left unset
    // when unknown, so the app can tell it has nothing to compare against.
    println!("cargo:rerun-if-env-changed=SLIDEFLARE_GIT_COMMIT");
    let commit = std::env::var("SLIDEFLARE_GIT_COMMIT")
        .ok()
        .filter(|c| !c.trim().is_empty())
        .or_else(git_head_commit);
    if let Some(commit) = commit {
        println!("cargo:rustc-env=SLIDEFLARE_GIT_COMMIT={commit}");
    }

    tauri_build::build()
}

fn git_head_commit() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir("..")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!sha.is_empty()).then_some(sha)
}
