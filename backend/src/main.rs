use axum::{
    Json, Router,
    extract::{Path, State},
    http::{Method, StatusCode},
    routing::{delete, get},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, path::Path as FilePath, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

mod analytics;
mod git;
mod templates;
mod validation;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProjectMetadata {
    id: String,
    name: String,
    last_modified: f64,
    component_count: usize,
}

type Store = Arc<RwLock<HashMap<String, serde_json::Value>>>;

fn get_data_file() -> String {
    std::env::var("DATA_FILE").unwrap_or_else(|_| "projects.json".to_string())
}

// Load store synchronously at startup (acceptable blocking)
fn load_store() -> HashMap<String, serde_json::Value> {
    let path = get_data_file();
    if FilePath::new(&path).exists() {
        if let Ok(file) = std::fs::File::open(&path) {
            let reader = std::io::BufReader::new(file);
            if let Ok(map) = serde_json::from_reader(reader) {
                tracing::info!("Loaded projects from {}", path);
                return map;
            }
        }
        tracing::error!("Failed to load projects from {}", path);
    }
    HashMap::new()
}

// Save store asynchronously
async fn save_store(store: &HashMap<String, serde_json::Value>) -> std::io::Result<()> {
    let path = get_data_file();
    let data = serde_json::to_vec_pretty(store)?;
    tokio::fs::write(&path, data).await
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let initial_data = load_store();
    let store = Arc::new(RwLock::new(initial_data));

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

    let project_routes = Router::new()
        .route("/api/projects", get(list_projects).post(save_project))
        .route(
            "/api/projects/{id}",
            get(get_project).delete(delete_project),
        )
        .with_state(store);

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

    // Serve frontend static files
    // Fallback to index.html for SPA routing
    let static_files = ServeDir::new("dist").fallback(ServeFile::new("dist/index.html"));

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
    let store = store.read().await;
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
        let mut guard = store.write().await;
        // Insert into memory first, capture old value for rollback
        let old_value = guard.insert(id.clone(), payload);

        // Try to save to disk
        if let Err(e) = save_store(&guard).await {
            tracing::error!("Failed to save store: {}", e);
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
    let store = store.read().await;
    if let Some(project) = store.get(&id) {
        Ok(Json(project.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_project(Path(id): Path<String>, State(store): State<Store>) -> StatusCode {
    let mut guard = store.write().await;

    if let Some(removed_project) = guard.remove(&id) {
        if let Err(e) = save_store(&guard).await {
            tracing::error!("Failed to save store after delete: {}", e);
            // Rollback: put it back
            guard.insert(id, removed_project);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
