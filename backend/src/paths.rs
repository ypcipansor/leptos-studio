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
//! write to them. [`resolve_data_file`] is the one function here that touches
//! the filesystem, because canonicalising a directory requires it to exist.

use std::{
    ffi::OsString,
    io,
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

/// Resolve the single file a store persists to, confined to the directory that
/// names it.
///
/// The nomination comes from an environment variable (`DATA_FILE`,
/// `TEMPLATES_FILE`, …). That is operator configuration rather than request
/// data, but it still decides which file the process overwrites, so it is
/// resolved rather than trusted:
///
/// * a `..` component is refused in both the directory and the file name, so the
///   configured path cannot walk upwards;
/// * the directory is created if needed and then canonicalised, so symlinks and
///   `.` segments in it are resolved *before* anything is written, and the
///   resolved directory is the one the file is taken to live in;
/// * the file name must be a single ordinary component and must not already be a
///   directory, so the result is a file directly inside that directory.
///
/// Every caller must derive the paths it writes from the returned value — the
/// parent for the temporary file, the value itself for the rename. Deriving them
/// from the same resolved path is what makes the confinement hold: a caller that
/// keeps using the uncanonicalised nomination for one of those writes can still
/// escape it.
pub fn resolve_data_file(nominated: &Path) -> io::Result<PathBuf> {
    // The checks below run on the text form, before any part of the configured
    // path is handed to the filesystem, so nothing is created or written for a
    // path that is about to be refused.
    let dir_text = nominated
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(|dir| dir.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());
    if dir_text.contains("..") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "data file directory must not contain `..`",
        ));
    }

    let file_name = nominated.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "data file must name a file, not a directory",
        )
    })?;
    // `Path::file_name` never yields a path separator, so this cannot contain a
    // component boundary; the check rules out a literal `..` name.
    let name_text = file_name.to_string_lossy();
    if name_text.contains("..") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "data file name must not contain `..`",
        ));
    }

    // Rebuilt from the checked text, so the path used from here on is the one
    // that was validated rather than the nomination it came from.
    let dir = PathBuf::from(&dir_text);
    std::fs::create_dir_all(&dir)?;

    // Resolves `.` and symlinks, so the directory decided on here is the one the
    // write actually lands in.
    let resolved_dir = dir.canonicalize()?;
    let resolved = resolved_dir.join(&*name_text);
    // `resolved_dir` is canonical and the name is a single component, so the file
    // is a direct child by construction; stating the containment is what pins the
    // guarantee for readers and analysers alike.
    if !resolved.starts_with(&resolved_dir) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "data file escapes its directory",
        ));
    }
    // A nominated directory would otherwise be written to as a file *inside
    // itself*, which is never what the configuration meant.
    if resolved.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "data file must name a file, not an existing directory",
        ));
    }

    Ok(resolved)
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

    /// A unique scratch directory, removed on drop so a test leaves nothing
    /// behind.
    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "leptos-studio-paths-{}-{}",
                std::process::id(),
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir_all(&path).expect("scratch directory must be creatable");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// The ordinary case: the file keeps its name and gains a resolved absolute
    /// directory, so a store configured with a relative path still lands where
    /// its parent says.
    #[test]
    fn resolve_data_file_confines_the_file_to_its_directory() {
        let scratch = TempDir::new();
        let nominated = scratch.0.join("nested").join("projects.json");

        let resolved = resolve_data_file(&nominated).expect("a plain file name must resolve");

        assert_eq!(resolved.file_name().unwrap(), "projects.json");
        assert_eq!(
            resolved.parent().unwrap(),
            scratch.0.join("nested").canonicalize().unwrap(),
            "the directory must be resolved and created"
        );
        assert!(scratch.0.join("nested").is_dir());
    }

    /// `..` is refused before anything is written, so a configured path cannot
    /// walk upwards out of the directory it names.
    #[test]
    fn resolve_data_file_rejects_dot_dot_in_the_directory() {
        let scratch = TempDir::new();
        let inner = scratch.0.join("inner");
        std::fs::create_dir_all(&inner).unwrap();

        assert!(resolve_data_file(&inner.join("..").join("projects.json")).is_err());
    }

    /// A trailing directory reference is not a file name: accepting it would let
    /// the resolved path escape the directory that was canonicalised.
    #[test]
    fn resolve_data_file_rejects_a_path_without_a_file_name() {
        let scratch = TempDir::new();
        assert!(resolve_data_file(&scratch.0).is_err());
        assert!(resolve_data_file(&scratch.0.join("..")).is_err());
    }
}
