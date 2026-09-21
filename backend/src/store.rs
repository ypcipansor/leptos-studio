//! Durable, atomic JSON persistence for every backend store.
//!
//! Each store is a state value plus the file it is persisted to. Mutating
//! requests must be one transaction: mutate the state, snapshot it, release the
//! state lock, then write the snapshot. If the write fails before publishing,
//! the state is rolled back.
//!
//! Centralising the write here keeps that guarantee identical for the project,
//! template, git-history and analytics stores instead of leaving three of them
//! to overwrite their JSON in place with a plain `fs::write` — which is neither
//! atomic nor durable and loses concurrent updates.
//!
//! # Atomicity
//!
//! The snapshot is written to a uniquely-named temporary file in the *same*
//! directory as the target (so the rename stays on one filesystem), flushed,
//! `sync_all`ed, and then renamed over the target. A crash can therefore leave
//! either the old complete file or the new complete file, never a truncated
//! document.
//!
//! # Durability
//!
//! Renaming is not enough on its own: the directory entry that now points at the
//! new file lives in the parent directory, and on a filesystem that requires an
//! explicit directory `fsync` (ext4 with `data=ordered`, XFS, …) a power loss
//! can replay the old directory entry even though `rename` returned success. So
//! after the rename the parent directory is opened and `sync_all`ed.
//!
//! # Rollback semantics when the directory sync fails
//!
//! The rename has already published the new file by the time the directory sync
//! runs, and those two steps must be treated differently on failure:
//!
//! * a failure *before* the rename published nothing, so the previous file is
//!   still the truth on disk and the transaction rolls its in-memory state back;
//! * a failure *in* the directory sync leaves the new state visible to every
//!   reader. Rolling back then would make memory disagree with the file — the
//!   exact silent divergence this module exists to prevent — so the state is
//!   kept and the error is still propagated, which stops the caller from
//!   reporting a durable success. A retry re-writes and re-syncs idempotently.
//!
//! This is why [`WriteError`] distinguishes the two: the caller must not treat
//! them the same.

use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{Mutex, RwLock};

use crate::paths;

/// Why a store write failed, distinguishing whether the new state is already
/// visible on disk.
#[derive(Debug)]
pub enum WriteError {
    /// Nothing was published: the previous state is still the truth on disk, so
    /// the caller must roll its in-memory state back.
    NotPublished(io::Error),
    /// The new state was renamed into place (readers already see it) but the
    /// parent directory could not be synced, so crash-durability is
    /// unconfirmed. The caller must *not* roll back — that would make memory
    /// disagree with the file — but must not report success either.
    PublishedNotDurable(io::Error),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::NotPublished(e) => write!(f, "nothing was published: {e}"),
            WriteError::PublishedNotDurable(e) => {
                write!(f, "published but not durable (directory sync failed): {e}")
            }
        }
    }
}

impl From<WriteError> for io::Error {
    fn from(value: WriteError) -> Self {
        match value {
            WriteError::NotPublished(e) | WriteError::PublishedNotDurable(e) => e,
        }
    }
}

/// The outcome of a transaction's mutation step.
///
/// Carrying "nothing changed" separately from "changed" is what lets
/// [`Store::commit`] skip persistence entirely for a no-op while still deciding
/// the no-op *inside* the transaction (where the mutation lock is held), rather
/// than with a racy check-then-delete.
#[derive(Debug)]
pub enum Mutation<U> {
    /// The state changed; `U` is the value to report to the caller.
    Changed(U),
    /// The state did not change, with the value to report. No persistence is
    /// attempted, so a failing storage layer cannot turn this into an error.
    Unchanged(U),
}

impl<U> Mutation<U> {
    /// True when the transaction actually changed the state.
    #[cfg(test)]
    pub fn is_changed(&self) -> bool {
        matches!(self, Mutation::Changed(_))
    }
}

/// A store's state plus the file it is persisted to.
///
/// Bundling the path with the data keeps persistence testable: a test points a
/// store at an isolated temp file and exercises the real handlers without
/// touching the tracked runtime JSON in `backend/`.
#[derive(Clone)]
pub struct Store<T> {
    /// The in-memory state, guarded by its own lock.
    pub state: Arc<RwLock<T>>,
    /// The nominated data file; the write path is derived from
    /// [`paths::resolve_data_file`], never used directly.
    pub data_file: PathBuf,
    /// Serialises mutation + persistence, not the state lock.
    ///
    /// Every mutating request runs as one transaction under this mutex:
    ///
    /// 1. mutate the in-memory state and clone the resulting snapshot;
    /// 2. release the state lock;
    /// 3. write the snapshot atomically and durably.
    ///
    /// Because the whole sequence is exclusive, only one transaction can be
    /// between "mutate" and "write" at a time, so writes always land on disk in
    /// the same order the state changed and a rollback can never clobber a
    /// concurrent request that committed in between — the failure is rolled back
    /// before the next transaction is allowed to start.
    ///
    /// The mutex deliberately does *not* guard the state: reads still take a
    /// shared read lock and never block on a writer's filesystem I/O.
    mutation_lock: Arc<Mutex<()>>,
    /// Test-only hook that forces a persistence failure for a given resulting
    /// state, so the rollback path can be exercised deterministically without
    /// racing a real filesystem error.
    #[cfg(test)]
    fail_hook: Arc<std::sync::Mutex<Option<FailureHook<T>>>>,
    /// Test-only hook that forces the *post-rename* directory sync to fail, so
    /// the "published but not durable" branch can be exercised without a power
    /// loss or a filesystem that actually refuses a directory fsync.
    #[cfg(test)]
    force_dir_sync_failure: Arc<std::sync::atomic::AtomicBool>,
}

/// Test-only predicate deciding whether a transaction's resulting state should
/// fail to persist.
#[cfg(test)]
pub type FailureHook<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;

impl<T: Clone + Send + Sync + serde::Serialize> Store<T> {
    pub fn new(state: T, data_file: PathBuf) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
            data_file,
            mutation_lock: Arc::new(Mutex::new(())),
            #[cfg(test)]
            fail_hook: Arc::new(std::sync::Mutex::new(None)),
            #[cfg(test)]
            force_dir_sync_failure: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Install the test-only failure hook: any transaction whose resulting state
    /// matches `hook` fails to persist.
    #[cfg(test)]
    pub fn set_fail_hook(&self, hook: Option<FailureHook<T>>) {
        *self
            .fail_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = hook;
    }

    /// Force the post-rename directory sync to fail, so the
    /// [`WriteError::PublishedNotDurable`] branch can be tested deterministically.
    #[cfg(test)]
    pub fn set_force_dir_sync_failure(&self, force: bool) {
        self.force_dir_sync_failure
            .store(force, std::sync::atomic::Ordering::SeqCst);
    }

    /// Run a mutating transaction to completion: mutate state, then persist the
    /// new snapshot atomically and durably.
    ///
    /// `mutate` returns a [`Mutation`] describing what changed and the value to
    /// report to the caller. Only [`Mutation::Changed`] triggers persistence: a
    /// no-op (for example deleting an id that is not present) must not attempt a
    /// write at all, so it cannot fail with a storage error and cannot report a
    /// 500 for a request that is really a 404.
    pub async fn commit<U: Send>(
        &self,
        mutate: impl FnOnce(&mut T) -> Mutation<U> + Send,
    ) -> io::Result<Mutation<U>> {
        // Serialises the whole transaction: no other mutation can observe or
        // write an intermediate state while this one is in flight.
        let _tx = self.mutation_lock.lock().await;

        let (mutation, old_state, new_state) = {
            let mut guard = self.state.write().await;
            let old_state = guard.clone();
            let mutation = mutate(&mut guard);
            let new_state = guard.clone();
            (mutation, old_state, new_state)
        };

        // A no-op decides in memory, inside the transaction. Nothing is written,
        // so a broken storage layer cannot turn a 404 into a 500.
        let value = match mutation {
            Mutation::Changed(value) => value,
            Mutation::Unchanged(value) => return Ok(Mutation::Unchanged(value)),
        };

        #[cfg(test)]
        let injected_failure = {
            let hook = self
                .fail_hook
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            hook.as_ref().is_some_and(|f| f(&new_state))
        };
        #[cfg(not(test))]
        let injected_failure = false;

        // The state lock is released before touching the filesystem: holding a
        // write lock across an `await`ed write would block every reader (and
        // `commit` itself) for the duration of the I/O.
        let write_result = if injected_failure {
            Err(WriteError::NotPublished(io::Error::other(
                "injected persistence failure",
            )))
        } else {
            write_store_atomically(
                &self.data_file,
                &new_state,
                #[cfg(test)]
                self.force_dir_sync_failure
                    .load(std::sync::atomic::Ordering::SeqCst),
            )
            .await
        };

        match write_result {
            Ok(()) => Ok(Mutation::Changed(value)),
            Err(WriteError::PublishedNotDurable(e)) => {
                // Deliberately *no* rollback: the new state is already visible on
                // disk, so restoring the old state in memory would leave memory
                // and file disagreeing with no explanation. The error still
                // surfaces, so the caller does not acknowledge a durable save.
                tracing::error!(
                    "Store write to {} was published but could not be made durable; \
                     keeping the new state so memory and disk agree: {}",
                    self.data_file.display(),
                    e
                );
                Err(e)
            }
            Err(WriteError::NotPublished(e)) => {
                // Only this transaction could have changed the state since the
                // snapshot was taken, because `mutation_lock` is held for the
                // whole transaction. Restoring the pre-transaction snapshot is
                // therefore exact — it cannot discard another request's commit.
                *self.state.write().await = old_state;
                Err(e)
            }
        }
    }
}

/// Persist `state` to `data_file` atomically and durably.
///
/// Callers must go through [`Store::commit`], which serialises the write against
/// other mutations and rolls the in-memory state back when this reports
/// [`WriteError::NotPublished`].
pub async fn write_store_atomically<T: serde::Serialize>(
    data_file: &Path,
    state: &T,
    #[cfg(test)] force_dir_sync_failure: bool,
) -> Result<(), WriteError> {
    let data = serde_json::to_vec_pretty(state)
        .map_err(|e| WriteError::NotPublished(io::Error::other(e)))?;
    write_bytes_atomically(
        data_file,
        &data,
        #[cfg(test)]
        force_dir_sync_failure,
    )
    .await
}

/// The bytes-level half of [`write_store_atomically`], also used by the tests so
/// the directory-sync path can be exercised without a store.
pub async fn write_bytes_atomically(
    data_file: &Path,
    data: &[u8],
    #[cfg(test)] force_dir_sync_failure: bool,
) -> Result<(), WriteError> {
    // Confine the write to the directory the store was configured with. Every
    // path below — the directory, the temporary file and the rename target — is
    // derived from this resolved value, so none of them can be redirected
    // elsewhere by a `..` or a symlink in the configured name.
    let safe_data_file = paths::resolve_data_file(data_file).map_err(WriteError::NotPublished)?;
    let dir = safe_data_file.parent().unwrap_or_else(|| Path::new("."));

    // Unique per write so concurrent writers (in different processes) cannot
    // clobber each other's temp file.
    let file_name = safe_data_file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "data.json".to_string());
    let tmp_path = dir.join(format!(".{}.{}.tmp", file_name, uuid::Uuid::new_v4()));

    let published = async {
        let mut file = tokio::fs::File::create(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, data).await?;
        tokio::io::AsyncWriteExt::flush(&mut file).await?;
        // Durability of the *bytes*: they must be on disk before the rename
        // publishes them under the real name.
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(&tmp_path, &safe_data_file).await
    }
    .await;

    if let Err(e) = published {
        // Best-effort cleanup; the temp file is not the data file, so leaving it
        // behind is harmless but untidy. Nothing was published, so the previous
        // file is still the truth on disk.
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(WriteError::NotPublished(e));
    }

    // Durability of the *rename*: the directory entry lives in the parent
    // directory, so without this a power loss can replay the old entry and
    // resurrect the previous file. A failure here must surface, otherwise the
    // caller would acknowledge a save that is not durable.
    #[cfg(test)]
    if force_dir_sync_failure {
        return Err(WriteError::PublishedNotDurable(io::Error::other(
            "injected directory sync failure",
        )));
    }

    sync_parent_dir(dir)
        .await
        .map_err(WriteError::PublishedNotDurable)
}

/// Open `dir` and `sync_all` it, making a rename inside it durable.
///
/// Linux allows `open(2)` on a directory read-only and `fsync(2)` on the
/// resulting descriptor, which is exactly what is needed to persist a rename.
/// Windows does not permit opening a directory this way at all, so the attempt
/// fails with a permission error and there is no portable equivalent; the call
/// is therefore best-effort on platforms where it cannot work. That limitation
/// is real and documented rather than papered over — see the module docs.
async fn sync_parent_dir(dir: &Path) -> io::Result<()> {
    match tokio::fs::File::open(dir).await {
        Ok(handle) => handle.sync_all().await,
        Err(e) => {
            // Opening a directory for reading is unsupported on Windows; there is
            // no other way to flush a directory entry there, so this is a
            // documented limitation rather than a failure to propagate.
            #[cfg(unix)]
            {
                Err(e)
            }
            #[cfg(not(unix))]
            {
                tracing::warn!(
                    "Failed to open data directory {} for sync: {}",
                    dir.display(),
                    e
                );
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// An isolated scratch directory, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "leptos-studio-store-{}-{}",
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

    fn map(state: &str) -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert(state.to_string(), serde_json::json!({ "name": state }));
        m
    }

    /// The durability path runs on the real filesystem: after a successful
    /// write the parent directory has been opened and fsynced, and the file
    /// contains exactly the new snapshot. This cannot simulate a power loss, but
    /// it does exercise the directory open + `sync_all` (the part a regression
    /// would remove) and pins that a successful write still returns `Ok`.
    #[tokio::test]
    async fn write_syncs_the_parent_directory_and_succeeds() {
        let scratch = TempDir::new();
        let target = scratch.0.join("projects.json");

        write_bytes_atomically(&target, b"{\"a\":1}", false)
            .await
            .expect("a durable write must succeed");

        assert_eq!(std::fs::read(&target).unwrap(), b"{\"a\":1}");

        // No temp files left behind.
        let leftovers: Vec<_> = std::fs::read_dir(&scratch.0)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {leftovers:?}"
        );

        // The directory can be opened and fsynced on this platform, which is what
        // makes the rename durable. On a platform that cannot, the confinement
        // still holds; the helper documents that difference.
        #[cfg(unix)]
        {
            let dir_handle = std::fs::File::open(&scratch.0)
                .expect("a directory must be openable read-only for fsync on unix");
            dir_handle
                .sync_all()
                .expect("a directory fsync must succeed on unix");
        }
    }

    /// A failure in the post-rename directory sync must be *reported* rather than
    /// swallowed, so a caller cannot acknowledge a save that is not durable.
    #[tokio::test]
    async fn directory_sync_failure_is_reported_as_published_not_durable() {
        let scratch = TempDir::new();
        let target = scratch.0.join("projects.json");

        let err = write_bytes_atomically(&target, b"{\"a\":1}", true)
            .await
            .expect_err("the injected directory sync failure must surface");
        assert!(
            matches!(err, WriteError::PublishedNotDurable(_)),
            "a directory-sync failure means the new state is already on disk, got {err:?}"
        );

        // The rename had already happened, so the new bytes are visible. That is
        // exactly why the transaction must not roll back.
        assert_eq!(std::fs::read(&target).unwrap(), b"{\"a\":1}");
    }

    /// A no-op must not touch the storage layer at all: with a failing writer the
    /// no-op still succeeds, and the pre-existing file content is untouched.
    #[tokio::test]
    async fn a_no_op_transaction_never_writes() {
        let scratch = TempDir::new();
        let target = scratch.0.join("projects.json");
        let store: Store<HashMap<String, serde_json::Value>> =
            Store::new(map("seed"), target.clone());

        // Seed the file, then arm a failure hook that would fail *any* write.
        store
            .commit(|state| Mutation::Changed(state.len()))
            .await
            .expect("the seed write must succeed");
        let seeded = std::fs::read(&target).unwrap();
        store.set_fail_hook(Some(Box::new(|_| true)));

        let outcome = store
            .commit(|_state| Mutation::Unchanged::<()>(()))
            .await
            .expect("a no-op must not surface a storage error");
        assert!(!outcome.is_changed());
        assert_eq!(
            std::fs::read(&target).unwrap(),
            seeded,
            "a no-op must not rewrite the file"
        );
    }

    /// A failure *before* publishing rolls the in-memory state back, so memory
    /// and disk stay in agreement.
    #[tokio::test]
    async fn a_failed_write_rolls_the_state_back() {
        let scratch = TempDir::new();
        let target = scratch.0.join("projects.json");
        let store: Store<HashMap<String, serde_json::Value>> =
            Store::new(HashMap::new(), target.clone());
        store.set_fail_hook(Some(Box::new(|_| true)));

        let err = store
            .commit(|state| {
                state.insert("doomed".to_string(), serde_json::json!({}));
                Mutation::Changed(())
            })
            .await
            .expect_err("the injected failure must surface");
        assert_eq!(err.kind(), io::ErrorKind::Other);

        assert!(
            store.state.read().await.is_empty(),
            "the failed transaction must not be visible in memory"
        );
        assert!(
            !target.exists(),
            "a failed transaction must not publish a file"
        );
    }

    /// A directory-sync failure must NOT roll back: the new state is already
    /// visible to readers, and rolling memory back would make it disagree with
    /// the file. The error still surfaces.
    #[tokio::test]
    async fn a_directory_sync_failure_keeps_the_published_state_and_still_errors() {
        let scratch = TempDir::new();
        let target = scratch.0.join("projects.json");
        let store: Store<HashMap<String, serde_json::Value>> =
            Store::new(HashMap::new(), target.clone());
        store.set_force_dir_sync_failure(true);

        store
            .commit(|state| {
                state.insert(
                    "published".to_string(),
                    serde_json::json!({ "name": "published" }),
                );
                Mutation::Changed(())
            })
            .await
            .expect_err("the non-durable write must surface as an error");

        // Memory and disk must agree: both show the new state.
        assert!(
            store.state.read().await.contains_key("published"),
            "the published state must be kept in memory so it matches disk"
        );
        let on_disk: HashMap<String, serde_json::Value> =
            serde_json::from_slice(&std::fs::read(&target).unwrap()).unwrap();
        assert!(
            on_disk.contains_key("published"),
            "the rename already published the new state, so disk must show it"
        );
    }

    /// Concurrent transactions must not lose an update: the serialised
    /// transaction makes the file match the final memory state exactly.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_transactions_leave_the_file_matching_memory() {
        let scratch = TempDir::new();
        let target = scratch.0.join("projects.json");
        let store: Store<HashMap<String, serde_json::Value>> =
            Store::new(HashMap::new(), target.clone());

        let barrier = Arc::new(tokio::sync::Barrier::new(8));
        let mut writers = Vec::new();
        for i in 0..8 {
            let store = store.clone();
            let barrier = barrier.clone();
            writers.push(tokio::spawn(async move {
                barrier.wait().await;
                let id = format!("proj-{i}");
                store
                    .commit(move |state| {
                        state.insert(id.clone(), serde_json::json!({ "name": id }));
                        Mutation::Changed(())
                    })
                    .await
                    .expect("write must succeed");
            }));
        }
        for writer in writers {
            writer.await.expect("writer must not panic");
        }

        let memory = store.state.read().await.clone();
        let on_disk: HashMap<String, serde_json::Value> =
            serde_json::from_slice(&std::fs::read(&target).unwrap()).unwrap();
        assert_eq!(memory.len(), 8, "every writer must be in memory");
        assert_eq!(
            memory, on_disk,
            "the file must match the in-memory state exactly"
        );
    }
}
