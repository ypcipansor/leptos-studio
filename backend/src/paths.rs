//! Path resolution for the backend's data files and static assets.
//!
//! Every default is derived from `CARGO_MANIFEST_DIR`, so the server finds the
//! same files no matter which directory it was launched from. Reading the
//! working directory was the source of a real bug: the documented
//! `cd backend && cargo run` made the defaults resolve inside `backend/`, while
//! running from the repository root resolved them at the root — where
//! `dist/index.html` did not exist and a stray `projects.json` got created.
//!
//! The `*_from` functions are pure so the resolution rules can be tested
//! without touching the environment or starting a server; the thin wrappers
//! below are the only ones that read environment variables, and they never
//! write to them.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// The backend crate's directory, fixed at compile time.
pub fn backend_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The repository root: the backend crate's parent directory.
pub fn repo_root() -> PathBuf {
    backend_dir()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Resolve a data file path.
///
/// An explicit override (typically an environment variable such as `DATA_FILE`)
/// wins; otherwise the file is looked up next to the backend crate, which is
/// where `projects.json`, `templates.json`, `git_data.json` and
/// `analytics.json` live in the repository.
pub fn data_file_from(override_path: Option<OsString>, base: &Path, default_name: &str) -> PathBuf {
    match override_path {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => base.join(default_name),
    }
}

/// Resolve the directory of built frontend assets.
///
/// An explicit override (typically `STATIC_DIR`) wins; otherwise the assets are
/// expected in `dist/` at the repository root, which is where `trunk build`
/// writes them (see `Trunk.toml`).
pub fn static_dir_from(override_path: Option<OsString>, root: &Path) -> PathBuf {
    match override_path {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => root.join("dist"),
    }
}

/// Resolve a data file path from an environment variable.
pub fn data_file(env_var: &str, default_name: &str) -> PathBuf {
    data_file_from(std::env::var_os(env_var), &backend_dir(), default_name)
}

/// Resolve the static asset directory from `STATIC_DIR`.
pub fn static_dir() -> PathBuf {
    static_dir_from(std::env::var_os("STATIC_DIR"), &repo_root())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The project store must be read from `backend/projects.json`, which is
    /// where the repository keeps it — not from the process working directory.
    #[test]
    fn default_project_data_file_is_backend_projects_json() {
        let resolved = data_file_from(None, &backend_dir(), "projects.json");
        assert_eq!(resolved, backend_dir().join("projects.json"));
        assert!(resolved.is_absolute(), "{resolved:?}");
        assert!(
            resolved
                .to_string_lossy()
                .ends_with("backend/projects.json"),
            "{resolved:?}"
        );
    }

    /// All four stores default into the backend crate directory.
    #[test]
    fn every_store_defaults_into_the_backend_directory() {
        for name in [
            "projects.json",
            "templates.json",
            "git_data.json",
            "analytics.json",
        ] {
            assert_eq!(
                data_file_from(None, &backend_dir(), name),
                backend_dir().join(name),
                "{name}"
            );
        }
    }

    /// The static default must be the repository-root `dist/`, matching where
    /// `trunk build` writes (the backend crate's sibling, not its child).
    #[test]
    fn default_static_dir_is_the_root_dist() {
        let resolved = static_dir_from(None, &repo_root());
        assert_eq!(resolved, repo_root().join("dist"));
        assert!(
            resolved.to_string_lossy().ends_with("/dist"),
            "{resolved:?}"
        );
        assert_ne!(
            resolved,
            backend_dir().join("dist"),
            "the backend must not look for its own dist/ directory"
        );
    }

    /// An explicit override replaces the default so deployments and containers
    /// can relocate their data.
    #[test]
    fn overrides_replace_the_default() {
        assert_eq!(
            data_file_from(
                Some("/app/data/projects.json".into()),
                &backend_dir(),
                "projects.json"
            ),
            PathBuf::from("/app/data/projects.json")
        );
        assert_eq!(
            static_dir_from(Some("/srv/www".into()), &repo_root()),
            PathBuf::from("/srv/www")
        );
        // An empty override (e.g. `DATA_FILE=`) must not blank out the path.
        assert_eq!(
            data_file_from(Some(OsString::new()), &backend_dir(), "projects.json"),
            backend_dir().join("projects.json")
        );
    }

    /// The resolution must not consult the working directory, so launching the
    /// backend from the repository root or from `backend/` cannot change it.
    #[test]
    fn resolution_is_an_absolute_build_time_path() {
        let from_env = data_file("LEPTOS_STUDIO_UNSET_DATA_FILE_VAR", "projects.json");
        assert_eq!(from_env, backend_dir().join("projects.json"));
        assert!(from_env.is_absolute());

        let static_from_env = static_dir();
        assert!(
            static_from_env.is_absolute(),
            "{static_from_env:?} must be absolute so the CWD cannot affect it"
        );
        assert!(static_from_env.ends_with("dist"));

        // A relative override stays relative on purpose: that is the operator's
        // explicit choice, and the process CWD then decides its meaning.
        assert_eq!(
            static_dir_from(Some("some/where".into()), &repo_root()),
            PathBuf::from("some/where")
        );
    }
}
