//! Records which build of the native code this is, for `phytkit.build_info()`:
//! - PHYTKIT_GIT_COMMIT: the commit, from that environment variable (release CI sets it),
//!   else `git rev-parse HEAD`, else "unknown" (an sdist built without git);
//! - PHYTKIT_GIT_DIRTY: "true" when the working tree had uncommitted changes;
//! - PHYTKIT_BUILD_KIND: "release" when release CI sets it, else "local".

use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    for v in ["PHYTKIT_GIT_COMMIT", "PHYTKIT_BUILD_KIND"] {
        println!("cargo:rerun-if-env-changed={v}");
    }
    // Rebuild when HEAD moves. Only watch files that exist: a missing one would make
    // cargo rerun this script on every build.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for f in [".git/HEAD", ".git/index"] {
        if root.join(f).exists() {
            println!("cargo:rerun-if-changed={}", root.join(f).display());
        }
    }
    let from_env = std::env::var("PHYTKIT_GIT_COMMIT")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let (commit, dirty) = match from_env {
        Some(c) => (c.trim().to_string(), "false".to_string()),
        None => match git(&["rev-parse", "HEAD"]) {
            Some(c) if !c.is_empty() => {
                let dirty = git(&["status", "--porcelain", "--untracked-files=no"])
                    .map(|s| !s.is_empty())
                    .unwrap_or(false);
                (c, dirty.to_string())
            }
            _ => ("unknown".to_string(), "false".to_string()),
        },
    };
    let kind = match std::env::var("PHYTKIT_BUILD_KIND").ok().as_deref() {
        Some("release") => "release",
        _ => "local",
    };
    println!("cargo:rustc-env=PHYTKIT_GIT_COMMIT={commit}");
    println!("cargo:rustc-env=PHYTKIT_GIT_DIRTY={dirty}");
    println!("cargo:rustc-env=PHYTKIT_BUILD_KIND={kind}");
}
