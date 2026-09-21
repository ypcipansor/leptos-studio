use axum::{
    Json, Router,
    extract::{Path, State},
    http::{Method, StatusCode},
    routing::{delete, get},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

mod analytics;
mod git;
mod paths;
mod templates;
mod validation;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProjectMetadata {
    id: String,
    name: String,
    last_modified: f64,
    component_count: usize,
}

/// The project store: the in-memory map plus the file it is persisted to.
///
/// Bundling the path with the data keeps persistence testable — a test can point
/// a store at an isolated temp file and exercise the real `save_project`
/// handler without touching the tracked `backend/projects.json`.
#[derive(Clone)]
struct Store {
    projects: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    data_file: std::path::PathBuf,
}

impl Store {
    fn new(projects: HashMap<String, serde_json::Value>, data_file: std::path::PathBuf) -> Self {
        Self {
            projects: Arc::new(RwLock::new(projects)),
            data_file,
        }
    }
}

fn get_data_file() -> std::path::PathBuf {
    paths::data_file("DATA_FILE", "projects.json")
}

// Load store synchronously at startup (acceptable blocking)
fn load_store() -> HashMap<String, serde_json::Value> {
    let path = get_data_file();
    if path.exists() {
        if let Ok(file) = std::fs::File::open(&path) {
            let reader = std::io::BufReader::new(file);
            if let Ok(map) = serde_json::from_reader(reader) {
                tracing::info!("Loaded projects from {}", path.display());
                return map;
            }
        }
        tracing::error!("Failed to load projects from {}", path.display());
    }
    HashMap::new()
}

// Save store asynchronously
async fn save_store(store: &Store) -> std::io::Result<()> {
    let projects = store.projects.read().await;
    let data = serde_json::to_vec_pretty(&*projects)?;
    tokio::fs::write(&store.data_file, data).await
}

/// The project CRUD routes for a given store. Shared by `main` and the
/// integration-style tests so both exercise the exact same handlers.
fn router_for_store(store: Store) -> Router {
    Router::new()
        .route("/api/projects", get(list_projects).post(save_project))
        .route(
            "/api/projects/{id}",
            get(get_project).delete(delete_project),
        )
        .with_state(store)
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let initial_data = load_store();
    let store = Store::new(initial_data, get_data_file());

    let initial_templates = templates::load_templates();
    let template_store = Arc::new(RwLock::new(initial_templates));

    let initial_git = git::load_git_data();
    let git_store = Arc::new(RwLock::new(initial_git));

    let initial_analytics = analytics::load_analytics();
    let analytics_store = Arc::new(RwLock::new(initial_analytics));

    // CORS
    // Use CORS_ORIGIN env var if set, otherwise default to Any (for dev)
    let cors_origin = std::env::var("CORS_ORIGIN").ok();
    let cors = if let Some(origin) = cors_origin {
        tracing::info!("CORS restricted to origin: {}", origin);
        CorsLayer::new()
            .allow_origin(
                origin
                    .parse::<axum::http::HeaderValue>()
                    .expect("Invalid CORS_ORIGIN value"),
            )
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers(Any)
    } else {
        tracing::warn!("CORS allowing ANY origin (development mode)");
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers(Any)
    };

    let project_routes = router_for_store(store);

    let template_routes = Router::new()
        .route(
            "/api/templates",
            get(templates::list_templates).post(templates::save_template),
        )
        .route("/api/templates/{id}", delete(templates::delete_template))
        .with_state(template_store);

    let git_routes = Router::new()
        .route(
            "/api/projects/{id}/commits",
            get(git::get_log)
                .post(git::post_commit)
                .delete(git::delete_history),
        )
        .with_state(git_store);

    let analytics_routes = Router::new()
        .route(
            "/api/analytics",
            axum::routing::post(analytics::post_analytics),
        )
        .with_state(analytics_store);

    // Serve frontend static files, resolved against the repository root so the
    // backend works from any working directory. Fallback to index.html for SPA
    // routing.
    let static_dir = paths::static_dir();
    tracing::info!("Serving static files from {}", static_dir.display());
    let static_files =
        ServeDir::new(&static_dir).fallback(ServeFile::new(static_dir.join("index.html")));

    let app = Router::new()
        .merge(project_routes)
        .merge(template_routes)
        .merge(git_routes)
        .merge(analytics_routes)
        .fallback_service(static_files)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn list_projects(State(store): State<Store>) -> Json<Vec<ProjectMetadata>> {
    let store = store.projects.read().await;
    let mut projects: Vec<ProjectMetadata> = store
        .values()
        .map(|p| {
            let id = p
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let name = p
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled")
                .to_string();
            let last_modified = p
                .get("last_modified")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let component_count = p
                .get("layout")
                .and_then(|l| l.as_array())
                .map(|a| a.len())
                .unwrap_or(0);

            ProjectMetadata {
                id,
                name,
                last_modified,
                component_count,
            }
        })
        .collect();

    // Sort by last modified desc
    projects.sort_by(|a, b| {
        b.last_modified
            .partial_cmp(&a.last_modified)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Json(projects)
}

async fn save_project(
    State(store): State<Store>,
    Json(mut payload): Json<serde_json::Value>,
) -> Result<Json<ProjectMetadata>, StatusCode> {
    // Extract or generate ID
    let id = payload
        .get("id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Ensure ID is in payload
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("id".to_string(), serde_json::Value::String(id.clone()));
        // Ensure last_modified is updated if not present (though frontend should send it)
        if !obj.contains_key("last_modified") {
            obj.insert("last_modified".to_string(), serde_json::Value::from(0.0));
        }
    }

    // Validate serialized components before storing
    if let Some(layout) = payload.get("layout")
        && let Err(e) = validation::validate_layout(layout)
    {
        tracing::warn!("Rejecting project save: {}", e);
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();
    let last_modified = payload
        .get("last_modified")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let component_count = payload
        .get("layout")
        .map(validation::count_components)
        .unwrap_or(0);

    {
        let mut guard = store.projects.write().await;
        // Insert into memory first, capture old value for rollback
        let old_value = guard.insert(id.clone(), payload);
        drop(guard);

        // Try to save to disk
        if let Err(e) = save_store(&store).await {
            tracing::error!("Failed to save store: {}", e);
            let mut guard = store.projects.write().await;
            // Rollback: Restore old value or remove if it was a new insert
            if let Some(v) = old_value {
                guard.insert(id, v);
            } else {
                guard.remove(&id);
            }
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(Json(ProjectMetadata {
        id,
        name,
        last_modified,
        component_count,
    }))
}

async fn get_project(
    Path(id): Path<String>,
    State(store): State<Store>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = store.projects.read().await;
    if let Some(project) = store.get(&id) {
        Ok(Json(project.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_project(Path(id): Path<String>, State(store): State<Store>) -> StatusCode {
    let removed_project = {
        let mut guard = store.projects.write().await;
        guard.remove(&id)
    };

    if let Some(removed_project) = removed_project {
        if let Err(e) = save_store(&store).await {
            tracing::error!("Failed to save store after delete: {}", e);
            // Rollback: put it back
            let mut guard = store.projects.write().await;
            guard.insert(id, removed_project);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    /// A store backed by a unique temp file, so tests never touch the tracked
    /// `backend/projects.json`.
    struct TestStore {
        store: Store,
        path: std::path::PathBuf,
    }

    impl TestStore {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "leptos-studio-test-{}-{}-{}.json",
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

    /// POST a project payload through the real router and return the status.
    async fn post_project(store: Store, payload: serde_json::Value) -> StatusCode {
        let app = router_for_store(store);
        let request = Request::builder()
            .method("POST")
            .uri("/api/projects")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .expect("request must build");

        app.oneshot(request)
            .await
            .expect("router must respond")
            .status()
    }

    fn link_layout_flow() -> serde_json::Value {
        json!({
            "name": "Link Project",
            "last_modified": 1.0,
            "layout": [
                { "Link": {
                    "id": "0e5b0f3a-0000-4000-8000-000000000001",
                    "text": "Docs",
                    "href": "https://example.com",
                    "animation": null,
                    "bindings": {},
                    "style": { "color": "#2563eb" }
                } },
                { "Container": { "children": [ { "Link": {
                    "id": "0e5b0f3a-0000-4000-8000-000000000002",
                    "text": "Nested",
                    "href": "https://example.com/nested",
                    "animation": null,
                    "bindings": {},
                    "style": {}
                } } ] } },
                { "Card": { "children": [ { "Link": {
                    "id": "0e5b0f3a-0000-4000-8000-000000000003",
                    "text": "In a card",
                    "href": "#anchor",
                    "animation": null,
                    "bindings": {},
                    "style": {}
                } } ] } }
            ]
        })
    }

    /// Regression: a project whose layout contains a Link was rejected with a
    /// 422 because `Link` was missing from the backend allowlist. The save must
    /// now succeed, persist, and survive a reopen.
    #[tokio::test]
    async fn saving_a_project_with_links_is_accepted_and_persists() {
        let test = TestStore::new("link-save");

        let status = post_project(test.store.clone(), link_layout_flow()).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "a layout with Links must be saved, not rejected with 422"
        );

        // Persisted to disk, with the layout intact.
        let written: HashMap<String, serde_json::Value> =
            serde_json::from_slice(&std::fs::read(&test.path).expect("data file must exist"))
                .expect("data file must be valid JSON");
        assert_eq!(written.len(), 1, "the project must be persisted");

        let stored = written.values().next().unwrap();
        let layout = stored
            .get("layout")
            .and_then(|l| l.as_array())
            .expect("layout must be an array");
        assert_eq!(layout.len(), 3, "every component must persist");
        assert_eq!(validation::count_components(&json!(layout)), 5);

        // Reopen through the real GET handler.
        let app = router_for_store(test.store.clone());
        let id = stored
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/projects/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("router must respond");
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let reopened: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let reopened_layout = reopened.get("layout").and_then(|l| l.as_array()).unwrap();
        assert_eq!(
            reopened_layout.len(),
            3,
            "the Link layout must reopen intact"
        );
        assert!(
            reopened_layout[0].get("Link").is_some(),
            "the root Link must survive a reopen"
        );
    }

    /// An unknown component type must still be rejected with a 422 — the Link
    /// fix must not turn validation off.
    #[tokio::test]
    async fn saving_a_project_with_an_unknown_type_is_still_rejected() {
        let test = TestStore::new("unknown-type");
        let payload = json!({
            "name": "Bad Project",
            "last_modified": 1.0,
            "layout": [ { "Widget": {} } ]
        });

        let status = post_project(test.store.clone(), payload).await;
        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "an unknown component type must still be rejected"
        );
        assert!(
            !test.path.exists(),
            "a rejected save must not write the data file"
        );
    }
}
