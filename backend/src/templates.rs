use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::store::{Mutation, Store};

// Match frontend TemplateCategory enum
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TemplateCategory {
    LandingPage,
    Dashboard,
    Form,
    Navigation,
    Card,
    Hero,
    Footer,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub thumbnail: Option<String>,
    pub components: Vec<serde_json::Value>,
    pub tags: Vec<String>,
}

/// The template state: id → template.
pub type TemplateMap = HashMap<String, Template>;

pub type TemplateStore = Store<TemplateMap>;

fn get_data_file() -> std::path::PathBuf {
    crate::paths::data_file("TEMPLATES_FILE", "templates.json")
}

pub fn load_templates() -> TemplateMap {
    let path = get_data_file();
    if path.exists() {
        if let Ok(file) = std::fs::File::open(&path) {
            let reader = std::io::BufReader::new(file);
            if let Ok(map) = serde_json::from_reader(reader) {
                tracing::info!("Loaded templates from {}", path.display());
                return map;
            }
        }
        tracing::error!("Failed to load templates from {}", path.display());
    }
    HashMap::new()
}

pub fn store() -> TemplateStore {
    Store::new(load_templates(), get_data_file())
}

pub async fn list_templates(State(store): State<TemplateStore>) -> Json<Vec<Template>> {
    let store = store.state.read().await;
    let mut templates: Vec<Template> = store.values().cloned().collect();
    // Sort by name
    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Json(templates)
}

pub async fn save_template(
    State(store): State<TemplateStore>,
    Json(mut payload): Json<Template>,
) -> Result<Json<Template>, StatusCode> {
    if payload.id.is_empty() {
        payload.id = uuid::Uuid::new_v4().to_string();
    }

    let result = payload.clone();
    match store
        .commit(move |templates| {
            templates.insert(payload.id.clone(), payload);
            Mutation::Changed(())
        })
        .await
    {
        Ok(_) => Ok(Json(result)),
        Err(e) => {
            tracing::error!("Failed to save templates: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_template(
    Path(id): Path<String>,
    State(store): State<TemplateStore>,
) -> StatusCode {
    // A missing id is a no-op, so the transaction skips persistence and the
    // answer is 404 even when the storage layer is failing.
    match store
        .commit(|templates| match templates.remove(&id) {
            Some(_removed) => Mutation::Changed(()),
            None => Mutation::Unchanged(()),
        })
        .await
    {
        Ok(Mutation::Changed(())) => StatusCode::NO_CONTENT,
        Ok(Mutation::Unchanged(())) => StatusCode::NOT_FOUND,
        Err(e) => {
            tracing::error!("Failed to save templates after delete: {}", e);
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
    use axum::routing::{delete, get};
    use tower::ServiceExt;

    struct TestStore {
        store: TemplateStore,
        path: std::path::PathBuf,
    }

    impl TestStore {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "leptos-studio-templates-{}-{}-{}.json",
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

    fn router(store: TemplateStore) -> Router {
        Router::new()
            .route("/api/templates", get(list_templates).post(save_template))
            .route("/api/templates/{id}", delete(delete_template))
            .with_state(store)
    }

    fn template(id: &str, name: &str) -> Template {
        Template {
            id: id.to_string(),
            name: name.to_string(),
            description: "desc".to_string(),
            category: TemplateCategory::Custom,
            thumbnail: None,
            components: vec![serde_json::json!({ "Text": { "content": name } })],
            tags: vec![],
        }
    }

    async fn request(
        store: TemplateStore,
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> StatusCode {
        let app = router(store);
        let mut builder = Request::builder().method(method).uri(uri);
        let body = match body {
            Some(value) => {
                builder = builder.header("content-type", "application/json");
                Body::from(value.to_string())
            }
            None => Body::empty(),
        };
        app.oneshot(builder.body(body).unwrap())
            .await
            .expect("router must respond")
            .status()
    }

    fn read_file(path: &std::path::Path) -> TemplateMap {
        serde_json::from_slice(&std::fs::read(path).expect("data file must exist"))
            .expect("data file must be valid JSON")
    }

    /// A saved template must reach disk through the same atomic, serialised
    /// transaction the project store uses.
    #[tokio::test]
    async fn save_template_persists_and_reloads() {
        let test = TestStore::new("save");
        let status = request(
            test.store.clone(),
            "POST",
            "/api/templates",
            Some(serde_json::to_value(template("t1", "Hero")).unwrap()),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let on_disk = read_file(&test.path);
        assert_eq!(on_disk.len(), 1);
        assert_eq!(on_disk["t1"].name, "Hero");
    }

    /// A delete for a missing id is a no-op: it answers 404 without touching the
    /// storage layer, so a failing writer cannot turn it into a 500.
    #[tokio::test]
    async fn deleting_a_missing_template_is_a_404_without_a_write() {
        let test = TestStore::new("delete-missing");
        test.store.set_fail_hook(Some(Box::new(|_| true)));

        let status = request(test.store.clone(), "DELETE", "/api/templates/nope", None).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "a missing template must be 404, not a storage error"
        );
        assert!(
            !test.path.exists(),
            "a no-op delete must not write the data file"
        );
    }

    /// A delete of an existing template with a failing writer must roll back and
    /// report 500, leaving the template in place.
    #[tokio::test]
    async fn deleting_an_existing_template_rolls_back_on_a_failed_write() {
        let test = TestStore::new("delete-existing");
        request(
            test.store.clone(),
            "POST",
            "/api/templates",
            Some(serde_json::to_value(template("t1", "Hero")).unwrap()),
        )
        .await;
        let before = std::fs::read(&test.path).unwrap();

        test.store.set_fail_hook(Some(Box::new(|_| true)));
        let status = request(test.store.clone(), "DELETE", "/api/templates/t1", None).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            test.store.state.read().await.contains_key("t1"),
            "the failed delete must be rolled back in memory"
        );
        assert_eq!(
            std::fs::read(&test.path).unwrap(),
            before,
            "the failed delete must not change the file"
        );
    }

    /// Concurrent template saves must not lose one another: the serialised
    /// transaction keeps the file in agreement with the final memory state.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_template_saves_leave_file_matching_memory() {
        let test = TestStore::new("concurrent");
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(4));
        let mut writers = Vec::new();
        for i in 0..4 {
            let store = test.store.clone();
            let barrier = barrier.clone();
            writers.push(tokio::spawn(async move {
                barrier.wait().await;
                let t = template(&format!("t{i}"), &format!("T{i}"));
                assert_eq!(
                    request(
                        store,
                        "POST",
                        "/api/templates",
                        Some(serde_json::to_value(t).unwrap())
                    )
                    .await,
                    StatusCode::OK
                );
            }));
        }
        for w in writers {
            w.await.expect("writer must not panic");
        }

        let memory = test.store.state.read().await.clone();
        assert_eq!(memory.len(), 4, "every save must be in memory");
        assert_eq!(
            memory,
            read_file(&test.path),
            "the file must match the in-memory state exactly"
        );
    }
}
