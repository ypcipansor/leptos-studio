use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::store::{Mutation, Store};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GitCommit {
    pub id: String,
    pub message: String,
    pub timestamp: f64, // JS timestamp
    pub snapshot: serde_json::Value,
}

/// Map: ProjectID -> List of Commits
pub type GitMap = HashMap<String, Vec<GitCommit>>;

pub type GitStore = Store<GitMap>;

fn get_data_file() -> std::path::PathBuf {
    crate::paths::data_file("GIT_DATA_FILE", "git_data.json")
}

pub fn load_git_data() -> GitMap {
    let path = get_data_file();
    if path.exists() {
        if let Ok(file) = std::fs::File::open(&path) {
            let reader = std::io::BufReader::new(file);
            if let Ok(map) = serde_json::from_reader(reader) {
                tracing::info!("Loaded git data from {}", path.display());
                return map;
            }
        }
        tracing::error!("Failed to load git data from {}", path.display());
    }
    HashMap::new()
}

pub fn store() -> GitStore {
    Store::new(load_git_data(), get_data_file())
}

pub async fn get_log(
    Path(project_id): Path<String>,
    State(store): State<GitStore>,
) -> Json<Vec<GitCommit>> {
    let store = store.state.read().await;
    Json(store.get(&project_id).cloned().unwrap_or_default())
}

#[derive(Deserialize)]
pub struct CommitPayload {
    pub message: String,
    pub timestamp: f64,
    pub snapshot: serde_json::Value,
}

pub async fn post_commit(
    Path(project_id): Path<String>,
    State(store): State<GitStore>,
    Json(payload): Json<CommitPayload>,
) -> Result<Json<GitCommit>, StatusCode> {
    let commit = GitCommit {
        id: uuid::Uuid::new_v4().to_string(),
        message: payload.message,
        timestamp: payload.timestamp,
        snapshot: payload.snapshot,
    };

    let result = commit.clone();
    match store
        .commit(move |data| {
            data.entry(project_id).or_default().push(commit);
            Mutation::Changed(())
        })
        .await
    {
        Ok(_) => Ok(Json(result)),
        Err(e) => {
            tracing::error!("Failed to save git data: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_history(
    Path(project_id): Path<String>,
    State(store): State<GitStore>,
) -> StatusCode {
    // A project with no history is a no-op, so the delete skips persistence and
    // answers 404 even when the storage layer is failing.
    match store
        .commit(|data| match data.remove(&project_id) {
            Some(_removed) => Mutation::Changed(()),
            None => Mutation::Unchanged(()),
        })
        .await
    {
        Ok(Mutation::Changed(())) => StatusCode::NO_CONTENT,
        Ok(Mutation::Unchanged(())) => StatusCode::NOT_FOUND,
        Err(e) => {
            tracing::error!("Failed to save git data after delete: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    struct TestStore {
        store: GitStore,
        path: std::path::PathBuf,
    }

    impl TestStore {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "leptos-studio-git-{}-{}-{}.json",
                name,
                std::process::id(),
                uuid::Uuid::new_v4()
            ));
            let store = Store::new(HashMap::new(), path.clone());
            Self { store, path }
        }
    }

    impl Drop for TestStore {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    fn router(store: GitStore) -> Router {
        Router::new()
            .route(
                "/api/projects/{id}/commits",
                get(get_log).post(post_commit).delete(delete_history),
            )
            .with_state(store)
    }

    async fn request(
        store: GitStore,
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, Vec<u8>) {
        let app = router(store);
        let mut builder = Request::builder().method(method).uri(uri);
        let body = match body {
            Some(value) => {
                builder = builder.header("content-type", "application/json");
                Body::from(value.to_string())
            }
            None => Body::empty(),
        };
        let response = app
            .oneshot(builder.body(body).unwrap())
            .await
            .expect("router must respond");
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, bytes.to_vec())
    }

    fn commit_payload(message: &str) -> serde_json::Value {
        serde_json::json!({
            "message": message,
            "timestamp": 1.0,
            "snapshot": { "layout": [] }
        })
    }

    /// A commit must be persisted through the shared atomic transaction and be
    /// readable back through the real route.
    #[tokio::test]
    async fn post_commit_persists_and_reads_back() {
        let test = TestStore::new("post");
        let (status, _) = request(
            test.store.clone(),
            "POST",
            "/api/projects/p1/commits",
            Some(commit_payload("first")),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let on_disk: GitMap = serde_json::from_slice(&std::fs::read(&test.path).unwrap()).unwrap();
        assert_eq!(on_disk["p1"].len(), 1);
        assert_eq!(on_disk["p1"][0].message, "first");

        let (status, body) =
            request(test.store.clone(), "GET", "/api/projects/p1/commits", None).await;
        assert_eq!(status, StatusCode::OK);
        let commits: Vec<GitCommit> = serde_json::from_slice(&body).unwrap();
        assert_eq!(commits.len(), 1);
    }

    /// A failed write must roll the commit back, so a reported error cannot still
    /// leave the commit in memory.
    #[tokio::test]
    async fn a_failed_commit_write_rolls_back() {
        let test = TestStore::new("post-fail");
        test.store.set_fail_hook(Some(Box::new(|_| true)));

        let (status, _) = request(
            test.store.clone(),
            "POST",
            "/api/projects/p1/commits",
            Some(commit_payload("doomed")),
        )
        .await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            test.store.state.read().await.is_empty(),
            "the failed commit must be rolled back"
        );
        assert!(!test.path.exists(), "nothing may be published");
    }

    /// Deleting history for a project that has none is a no-op: 404 without a
    /// write, even with a failing storage layer.
    #[tokio::test]
    async fn deleting_absent_history_is_a_404_without_a_write() {
        let test = TestStore::new("delete-missing");
        test.store.set_fail_hook(Some(Box::new(|_| true)));

        let (status, _) = request(
            test.store.clone(),
            "DELETE",
            "/api/projects/nope/commits",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(!test.path.exists());
    }

    /// Concurrent commits to the same project must all survive, with the file
    /// agreeing with memory.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_commits_leave_file_matching_memory() {
        let test = TestStore::new("concurrent");
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(4));
        let mut writers = Vec::new();
        for i in 0..4 {
            let store = test.store.clone();
            let barrier = barrier.clone();
            writers.push(tokio::spawn(async move {
                barrier.wait().await;
                let (status, _) = request(
                    store,
                    "POST",
                    "/api/projects/p1/commits",
                    Some(commit_payload(&format!("c{i}"))),
                )
                .await;
                assert_eq!(status, StatusCode::OK);
            }));
        }
        for w in writers {
            w.await.expect("writer must not panic");
        }

        let memory = test.store.state.read().await.clone();
        assert_eq!(memory["p1"].len(), 4, "every commit must be in memory");
        let on_disk: GitMap = serde_json::from_slice(&std::fs::read(&test.path).unwrap()).unwrap();
        assert_eq!(memory, on_disk, "the file must match memory exactly");
    }
}
