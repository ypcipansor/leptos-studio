use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::store::{Mutation, Store};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub session_id: String,
    pub timestamp: f64,
    pub event_type: String,
    pub payload: serde_json::Value,
}

/// Analytics is an append-only log, so the state is a list.
pub type Analytics = Vec<AnalyticsData>;

pub type AnalyticsStore = Store<Analytics>;

fn get_data_file() -> std::path::PathBuf {
    crate::paths::data_file("ANALYTICS_DATA_FILE", "analytics.json")
}

pub fn load_analytics() -> Analytics {
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

pub fn store() -> AnalyticsStore {
    Store::new(load_analytics(), get_data_file())
}

#[derive(Deserialize)]
pub struct AnalyticsBatch {
    pub events: Vec<AnalyticsData>,
}

pub async fn post_analytics(
    State(store): State<AnalyticsStore>,
    Json(batch): Json<AnalyticsBatch>,
) -> StatusCode {
    // An empty batch changes nothing, so it must not attempt a write — and must
    // not fail with a storage error when there is nothing to store.
    if batch.events.is_empty() {
        return StatusCode::OK;
    }

    match store
        .commit(move |events| {
            events.extend(batch.events);
            Mutation::Changed(())
        })
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Failed to save analytics: {}", e);
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
    use axum::routing::post;
    use tower::ServiceExt;

    struct TestStore {
        store: AnalyticsStore,
        path: std::path::PathBuf,
    }

    impl TestStore {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "leptos-studio-analytics-{}-{}-{}.json",
                name,
                std::process::id(),
                uuid::Uuid::new_v4()
            ));
            let store = Store::new(Vec::new(), path.clone());
            Self { store, path }
        }
    }

    impl Drop for TestStore {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    fn router(store: AnalyticsStore) -> Router {
        Router::new()
            .route("/api/analytics", post(post_analytics))
            .with_state(store)
    }

    async fn post_events(store: AnalyticsStore, events: serde_json::Value) -> StatusCode {
        let app = router(store);
        let request = Request::builder()
            .method("POST")
            .uri("/api/analytics")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({ "events": events }).to_string(),
            ))
            .unwrap();
        app.oneshot(request).await.unwrap().status()
    }

    fn event(session: &str) -> serde_json::Value {
        serde_json::json!({
            "session_id": session,
            "timestamp": 1.0,
            "event_type": "click",
            "payload": {}
        })
    }

    /// A batch must be persisted through the shared atomic transaction.
    #[tokio::test]
    async fn post_analytics_persists_events() {
        let test = TestStore::new("post");
        let status = post_events(
            test.store.clone(),
            serde_json::json!([event("s1"), event("s1")]),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let on_disk: Analytics =
            serde_json::from_slice(&std::fs::read(&test.path).unwrap()).unwrap();
        assert_eq!(on_disk.len(), 2);
    }

    /// An empty batch is a no-op: it must not write, so a failing storage layer
    /// cannot turn an empty request into a 500.
    #[tokio::test]
    async fn an_empty_batch_does_not_write() {
        let test = TestStore::new("empty");
        test.store.set_fail_hook(Some(Box::new(|_| true)));

        let status = post_events(test.store.clone(), serde_json::json!([])).await;
        assert_eq!(status, StatusCode::OK);
        assert!(!test.path.exists(), "an empty batch must not write");
    }

    /// A failed write must roll the appended events back.
    #[tokio::test]
    async fn a_failed_analytics_write_rolls_back() {
        let test = TestStore::new("fail");
        test.store.set_fail_hook(Some(Box::new(|_| true)));

        let status = post_events(test.store.clone(), serde_json::json!([event("s1")])).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            test.store.state.read().await.is_empty(),
            "the failed append must be rolled back"
        );
    }

    /// Concurrent batches must all survive, with the file agreeing with memory.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_batches_leave_file_matching_memory() {
        let test = TestStore::new("concurrent");
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(4));
        let mut writers = Vec::new();
        for i in 0..4 {
            let store = test.store.clone();
            let barrier = barrier.clone();
            writers.push(tokio::spawn(async move {
                barrier.wait().await;
                let status = post_events(store, serde_json::json!([event(&format!("s{i}"))])).await;
                assert_eq!(status, StatusCode::OK);
            }));
        }
        for w in writers {
            w.await.expect("writer must not panic");
        }

        let memory = test.store.state.read().await.clone();
        assert_eq!(memory.len(), 4, "every batch must be in memory");
        let on_disk: Analytics =
            serde_json::from_slice(&std::fs::read(&test.path).unwrap()).unwrap();
        assert_eq!(memory, on_disk, "the file must match memory exactly");
    }
}
