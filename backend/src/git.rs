use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitCommit {
    pub id: String,
    pub message: String,
    pub timestamp: f64, // JS timestamp
    pub snapshot: serde_json::Value,
}

// Map: ProjectID -> List of Commits
pub type GitStore = Arc<RwLock<HashMap<String, Vec<GitCommit>>>>;

fn get_data_file() -> std::path::PathBuf {
    crate::paths::data_file("GIT_DATA_FILE", "git_data.json")
}

pub fn load_git_data() -> HashMap<String, Vec<GitCommit>> {
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

async fn save_store(store: &HashMap<String, Vec<GitCommit>>) -> std::io::Result<()> {
    let path = get_data_file();
    let data = serde_json::to_vec_pretty(store)?;
    tokio::fs::write(&path, data).await
}

pub async fn get_log(
    Path(project_id): Path<String>,
    State(store): State<GitStore>,
) -> Json<Vec<GitCommit>> {
    let store = store.read().await;
    let commits = store.get(&project_id).cloned().unwrap_or_default();
    // Assuming stored in append order (oldest first), we might want to return newest first?
    // Frontend usually handles sorting or expects specific order.
    // Let's return as is (chronological usually), frontend `rev()` if needed.
    // Actually `LocalStorageGitBackend` returned `rev()`.
    // Let's return raw list, let frontend decide.
    Json(commits)
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

    let mut guard = store.write().await;
    let project_commits = guard.entry(project_id.clone()).or_insert_with(Vec::new);
    project_commits.push(commit.clone());

    if let Err(e) = save_store(&guard).await {
        tracing::error!("Failed to save git data: {}", e);
        // Rollback: pop the commit we just added
        if let Some(commits) = guard.get_mut(&project_id) {
            commits.pop();
        }
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(commit))
}

pub async fn delete_history(
    Path(project_id): Path<String>,
    State(store): State<GitStore>,
) -> StatusCode {
    let mut guard = store.write().await;
    if let Some(removed) = guard.remove(&project_id) {
        if let Err(e) = save_store(&guard).await {
            tracing::error!("Failed to save git data after delete: {}", e);
            // Rollback: put it back
            guard.insert(project_id, removed);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
