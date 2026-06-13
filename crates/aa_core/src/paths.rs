use std::path::{Path, PathBuf};

/// Environment variable holding the active game project root.
pub const PROJECT_ROOT_ENV: &str = "AA_PROJECT_ROOT";

/// Sets the active project root for all `aa_*` crates in this process.
pub fn set_project_root(root: impl AsRef<Path>) {
    // SAFETY: AA_PROJECT_ROOT is process-local engine state set once at boot.
    unsafe {
        std::env::set_var(PROJECT_ROOT_ENV, root.as_ref().as_os_str());
    }
}

/// Returns the active project root (`AA_PROJECT_ROOT` or `"."`).
pub fn project_root() -> PathBuf {
    std::env::var(PROJECT_ROOT_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Joins a path relative to the active project root.
pub fn project_path(relative: impl AsRef<Path>) -> PathBuf {
    project_root().join(relative.as_ref())
}
