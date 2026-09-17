use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

// Match frontend TemplateCategory enum
#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub thumbnail: Option<String>,
    pub components: Vec<serde_json::Value>,
    pub tags: Vec<String>,
}

pub type TemplateStore = Arc<RwLock<HashMap<String, Template>>>;

fn get_data_file() -> std::path::PathBuf {
    crate::paths::data_file("TEMPLATES_FILE", "templates.json")
}

pub fn load_templates() -> HashMap<String, Template> {
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

async fn save_store(store: &HashMap<String, Template>) -> std::io::Result<()> {
    let path = get_data_file();
    let data = serde_json::to_vec_pretty(store)?;
    tokio::fs::write(&path, data).await
}

pub async fn list_templates(State(store): State<TemplateStore>) -> Json<Vec<Template>> {
    let store = store.read().await;
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

    let mut guard = store.write().await;
    let old_value = guard.insert(payload.id.clone(), payload.clone());

    if let Err(e) = save_store(&guard).await {
        tracing::error!("Failed to save templates: {}", e);
        if let Some(v) = old_value {
            guard.insert(payload.id.clone(), v);
        } else {
            guard.remove(&payload.id);
        }
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(payload))
}

pub async fn delete_template(
    Path(id): Path<String>,
    State(store): State<TemplateStore>,
) -> StatusCode {
    let mut guard = store.write().await;

    if let Some(removed) = guard.remove(&id) {
        if let Err(e) = save_store(&guard).await {
            tracing::error!("Failed to save templates after delete: {}", e);
            guard.insert(id, removed);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
