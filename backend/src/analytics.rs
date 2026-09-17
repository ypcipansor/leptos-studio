use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub session_id: String,
    pub timestamp: f64,
    pub event_type: String,
    pub payload: serde_json::Value,
}

// We append to a list, simpler store than map for logs
pub type AnalyticsStore = Arc<RwLock<Vec<AnalyticsData>>>;

fn get_data_file() -> std::path::PathBuf {
    crate::paths::data_file("ANALYTICS_DATA_FILE", "analytics.json")
}

pub fn load_analytics() -> Vec<AnalyticsData> {
    let path = get_data_file();
    if path.exists() {
        if let Ok(file) = std::fs::File::open(&path) {
            let reader = std::io::BufReader::new(file);
            if let Ok(list) = serde_json::from_reader(reader) {
                tracing::info!("Loaded analytics from {}", path.display());
                return list;
            }
        }
        tracing::error!("Failed to load analytics from {}", path.display());
    }
    Vec::new()
}

async fn save_store(store: &Vec<AnalyticsData>) -> std::io::Result<()> {
    let path = get_data_file();
    let data = serde_json::to_vec_pretty(store)?;
    tokio::fs::write(&path, data).await
}

#[derive(Deserialize)]
pub struct AnalyticsBatch {
    pub events: Vec<AnalyticsData>,
}

pub async fn post_analytics(
    State(store): State<AnalyticsStore>,
    Json(batch): Json<AnalyticsBatch>,
) -> StatusCode {
    let mut guard = store.write().await;
    guard.extend(batch.events);

    if let Err(e) = save_store(&guard).await {
        tracing::error!("Failed to save analytics: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
