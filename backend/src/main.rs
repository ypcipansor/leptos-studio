use axum::{
    Json, Router,
    extract::{Path, State},
    http::{Method, StatusCode},
    routing::{delete, get},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, RwLock};
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
    /// Serialises mutation + persistence, not the state lock.
    ///
    /// Every mutating request runs as one transaction under this mutex:
    ///
    /// 1. mutate the in-memory map and clone the resulting snapshot;
    /// 2. release the state lock;
    /// 3. write the snapshot atomically.
    ///
    /// Because the whole sequence is exclusive, only one transaction can be
    /// between "mutate" and "write" at a time, so writes always land on disk in
    /// the same order the state changed and a rollback can never clobber a
    /// concurrent request that committed in between — the failure is rolled back
    /// before the next transaction is allowed to start.
    ///
    /// The mutex deliberately does *not* guard the state: reads (`list_projects`,
    /// `get_project`) still take a shared read lock and never block on a writer's
    /// filesystem I/O. Only the write transaction is serialised, which is what
    /// makes the ordering guarantee hold. Mutation callers must not hold the
    /// projects read/write lock across the filesystem write (see
    /// `Store::commit`).
    mutation_lock: Arc<Mutex<()>>,

    /// Test-only hook that forces a persistence failure for a given resulting
    /// state, so the rollback path can be exercised deterministically without
    /// racing a real filesystem error.
    #[cfg(test)]
    fail_hook: Arc<std::sync::Mutex<Option<FailHook>>>,
}

#[cfg(test)]
type FailHook = Box<dyn Fn(&HashMap<String, serde_json::Value>) -> bool + Send + Sync>;

impl Store {
    fn new(projects: HashMap<String, serde_json::Value>, data_file: std::path::PathBuf) -> Self {
        Self {
            projects: Arc::new(RwLock::new(projects)),
            data_file,
            mutation_lock: Arc::new(Mutex::new(())),
            #[cfg(test)]
            fail_hook: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Run a mutating transaction to completion: mutate state, then persist the
    /// new snapshot atomically. `mutate` returns the value to report to the
    /// caller; if persistence fails the mutation is rolled back and the caller
    /// observes the error.
    async fn commit<T: Send>(
        &self,
        mutate: impl FnOnce(&mut HashMap<String, serde_json::Value>) -> T + Send,
    ) -> std::io::Result<T> {
        // Serialises the whole transaction: no other mutation can observe or
        // write an intermediate state while this one is in flight.
        let _tx = self.mutation_lock.lock().await;

        let (result, old_state, new_state) = {
            let mut guard = self.projects.write().await;
            let old_state = guard.clone();
            let result = mutate(&mut guard);
            let new_state = guard.clone();
            (result, old_state, new_state)
        };

        #[cfg(test)]
        let injected_failure = {
            let hook = self.fail_hook.lock().expect("fail hook mutex poisoned");
            hook.as_ref().is_some_and(|f| f(&new_state))
        };
        #[cfg(not(test))]
        let injected_failure = false;

        // The state lock is released before touching the filesystem: holding a
        // write lock across an `await`ed write would block every reader (and
        // `commit` itself) for the duration of the I/O.
        let write_result = if injected_failure {
            Err(std::io::Error::other("injected persistence failure"))
        } else {
            write_store_atomically(&self.data_file, &new_state).await
        };

        match write_result {
            Ok(()) => Ok(result),
            Err(e) => {
                // Only this transaction could have changed the state since the
                // snapshot was taken, because `mutation_lock` is held for the
                // whole transaction. Restoring the pre-transaction snapshot is
                // therefore exact — it cannot discard another request's commit.
                *self.projects.write().await = old_state;
                Err(e)
            }
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

/// Persist the store to disk atomically.
///
/// Serialises the map, writes it to a temporary file in the *same* directory
/// (so the rename stays on one filesystem and is therefore atomic), syncs it,
/// then renames it over `store.data_file`. A crash or a partial write can
/// therefore only ever leave the old, complete file or the new, complete file —
/// never a truncated or half-written JSON document.
///
/// Callers must go through [`Store::commit`], which serialises the write against
/// other mutations and rolls the in-memory state back if this returns `Err`.
async fn write_store_atomically(
    data_file: &std::path::Path,
    projects: &HashMap<String, serde_json::Value>,
) -> std::io::Result<()> {
    let data = serde_json::to_vec_pretty(projects)?;

    let dir = data_file
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    // Ensure writes stay inside the process working directory tree.
    let safe_base = std::env::current_dir()?.canonicalize()?;
    let candidate_dir = if dir.is_absolute() {
        dir.to_path_buf()
    } else {
        safe_base.join(dir)
    };
    let normalized_dir = if candidate_dir.exists() {
        candidate_dir.canonicalize()?
    } else if let Some(parent) = candidate_dir.parent() {
        let normalized_parent = if parent.exists() {
            parent.canonicalize()?
        } else {
            safe_base.clone()
        };
        let leaf = candidate_dir
            .file_name()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid data directory"))?;
        normalized_parent.join(leaf)
    } else {
        safe_base.clone()
    };

    if !normalized_dir.starts_with(&safe_base) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "data file path escapes allowed base directory",
        ));
    }

    if !dir.as_os_str().is_empty() {
        tokio::fs::create_dir_all(dir).await?;
    }

    // Unique per write so concurrent writers (in different processes) cannot
    // clobber each other's temp file.
    let file_name = data_file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "projects.json".to_string());
    let tmp_path = dir.join(format!(".{}.{}.tmp", file_name, uuid::Uuid::new_v4()));

    let write_result = async {
        let mut file = tokio::fs::File::create(&tmp_path).await?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &data).await?;
        tokio::io::AsyncWriteExt::flush(&mut file).await?;
        // Durability: the bytes must be on disk before the rename publishes them.
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(&tmp_path, data_file).await
    }
    .await;

    if write_result.is_err() {
        // Best-effort cleanup; the temp file is not the data file, so leaving it
        // behind is harmless but untidy.
        let _ = tokio::fs::remove_file(&tmp_path).await;
    }

    write_result
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
                .map(validation::count_components)
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

    // The insert and the resulting file write are one serialised transaction, so
    // a concurrent save/delete can neither lose this project nor persist a stale
    // snapshot after it. On a write failure the whole insert is rolled back —
    // and because no other transaction can run in between, that rollback cannot
    // discard another request's successful commit.
    let insert_id = id.clone();
    match store
        .commit(move |projects| {
            projects.insert(insert_id, payload);
        })
        .await
    {
        Ok(()) => {}
        Err(e) => {
            tracing::error!("Failed to save store: {}", e);
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
    // Same transaction path as `save_project`: the removal and its file write are
    // serialised, and a write failure restores the project exactly.
    match store.commit(|projects| projects.remove(&id)).await {
        Ok(Some(_removed)) => StatusCode::NO_CONTENT,
        Ok(None) => StatusCode::NOT_FOUND,
        Err(e) => {
            tracing::error!("Failed to save store after delete: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
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

    /// Drive the real router and return the status plus the response body.
    async fn request(
        store: Store,
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, Vec<u8>) {
        let app = router_for_store(store);
        let mut builder = Request::builder().method(method).uri(uri);
        let body = match body {
            Some(value) => {
                builder = builder.header("content-type", "application/json");
                Body::from(value.to_string())
            }
            None => Body::empty(),
        };

        let response = app
            .oneshot(builder.body(body).expect("request must build"))
            .await
            .expect("router must respond");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body must be readable");
        (status, bytes.to_vec())
    }

    fn project_payload(id: &str, name: &str) -> serde_json::Value {
        json!({
            "id": id,
            "name": name,
            "last_modified": 1.0,
            "layout": [ { "Link": {
                "id": id,
                "text": name,
                "href": "https://example.com",
                "animation": null,
                "bindings": {},
                "style": {}
            } } ]
        })
    }

    /// Read the on-disk store and parse it, asserting it is a complete document.
    fn read_data_file(path: &std::path::Path) -> HashMap<String, serde_json::Value> {
        let bytes = std::fs::read(path).expect("data file must exist");
        serde_json::from_slice(&bytes).expect("data file must always be valid JSON")
    }

    fn stored_ids(path: &std::path::Path) -> Vec<String> {
        let mut ids: Vec<String> = read_data_file(path).keys().cloned().collect();
        ids.sort();
        ids
    }

    /// The in-memory ids, through the real `GET /api/projects` handler.
    async fn listed_ids(store: Store) -> Vec<String> {
        let (status, body) = request(store, "GET", "/api/projects", None).await;
        assert_eq!(status, StatusCode::OK);
        let mut ids: Vec<String> = serde_json::from_slice::<Vec<ProjectMetadata>>(&body)
            .expect("project list must deserialize")
            .into_iter()
            .map(|p| p.id)
            .collect();
        ids.sort();
        ids
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

    // ---------------------------------------------------------------------
    // Concurrency: mutation + persistence must be one serialised transaction.
    // ---------------------------------------------------------------------

    /// Run one request through the *real* router — the same `save_project` /
    /// `delete_project` handlers production uses — after a barrier so two of
    /// them overlap. Returns the id and the response status.
    async fn transact(
        store: Store,
        barrier: Arc<tokio::sync::Barrier>,
        id: &str,
        delete: bool,
    ) -> (String, StatusCode) {
        barrier.wait().await;
        let uri = if delete {
            format!("/api/projects/{id}")
        } else {
            "/api/projects".to_string()
        };
        let payload = (!delete).then(|| project_payload(id, id));
        let (status, _) =
            request(store, if delete { "DELETE" } else { "POST" }, &uri, payload).await;
        (id.to_string(), status)
    }

    /// Spawn two requests on the multi-threaded runtime so they run on real OS
    /// threads and genuinely interleave, and return their results.
    async fn race(
        store: Store,
        a: (&'static str, bool),
        b: (&'static str, bool),
    ) -> ((String, StatusCode), (String, StatusCode)) {
        let barrier = Arc::new(tokio::sync::Barrier::new(2));
        let first = tokio::spawn(transact(store.clone(), barrier.clone(), a.0, a.1));
        let second = tokio::spawn(transact(store, barrier, b.0, b.1));
        (
            first.await.expect("task must not panic"),
            second.await.expect("task must not panic"),
        )
    }

    /// Two concurrent POSTs must both survive — in memory *and* after the file
    /// is read back. With a read-modify-write that dropped the state lock before
    /// writing, the second writer could persist a snapshot that lacked the
    /// first project (a lost update), or the two writes could land out of order.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_saves_do_not_lose_a_project() {
        let test = TestStore::new("concurrent-saves");

        // Repeated so a genuine race has many chances to interleave.
        for _ in 0..20 {
            test.store.projects.write().await.clear();
            let ((a_id, a_status), (b_id, b_status)) =
                race(test.store.clone(), ("proj-a", false), ("proj-b", false)).await;
            assert_eq!(
                (a_status, b_status),
                (StatusCode::OK, StatusCode::OK),
                "both saves must be accepted"
            );
            assert_ne!(a_id, b_id);

            let expected = vec!["proj-a".to_string(), "proj-b".to_string()];
            assert_eq!(
                listed_ids(test.store.clone()).await,
                expected,
                "both projects must be visible in memory"
            );
            assert_eq!(
                stored_ids(&test.path),
                expected,
                "both projects must be persisted — neither save may be lost"
            );
        }
    }

    /// A concurrent POST and DELETE must leave the file agreeing exactly with
    /// the in-memory state, whichever order the transactions actually ran in.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_save_and_delete_leave_file_matching_memory() {
        for _ in 0..25 {
            let test = TestStore::new("save-delete");
            // Seed the project the delete targets, then remove it — the end
            // state must be the same as if the seed had never existed.
            test.store
                .commit(|projects| {
                    projects.insert("proj-x".to_string(), project_payload("proj-x", "X"));
                })
                .await
                .expect("seed must persist");

            let ((_, save_status), (_, delete_status)) =
                race(test.store.clone(), ("proj-y", false), ("proj-x", true)).await;
            assert_eq!(
                save_status,
                StatusCode::OK,
                "the concurrent save must succeed"
            );
            assert_eq!(
                delete_status,
                StatusCode::NO_CONTENT,
                "the concurrent delete must succeed"
            );

            // Both effects must be present in memory and on disk, in any order.
            let memory = listed_ids(test.store.clone()).await;
            let disk = stored_ids(&test.path);
            assert_eq!(
                memory, disk,
                "the persisted file must match the in-memory state exactly"
            );
            assert_eq!(
                memory,
                vec!["proj-y".to_string()],
                "the save must land and the delete must not be undone"
            );

            // The file is a complete document and contains no leftover temp files.
            let leftovers: Vec<_> = std::fs::read_dir(test.path.parent().unwrap())
                .unwrap()
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.starts_with(".save-delete") || n.starts_with(".proj"))
                .collect();
            assert!(
                leftovers.is_empty(),
                "temp files left behind: {leftovers:?}"
            );
        }
    }

    /// A failed persistence must roll the mutation back *without* discarding a
    /// transaction that committed concurrently. The failing transaction is
    /// forced deterministically through the fail hook (keyed on the resulting
    /// state, so only the intended transaction fails) rather than relying on
    /// scheduler timing.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn failed_persistence_rolls_back_without_clobbering_a_concurrent_commit() {
        let test = TestStore::new("rollback");
        // Seed an existing project so the rollback restores a non-empty state.
        test.store
            .commit(|projects| {
                projects.insert("seed".to_string(), project_payload("seed", "Seed"));
            })
            .await
            .expect("seed must persist");

        // Fail any transaction whose resulting state contains "doomed". That is
        // exactly the doomed transaction — the keeper transaction's resulting
        // state never contains it, whichever order the two run in, so the keeper
        // always commits and the doomed one always rolls back.
        let hook: FailHook =
            Box::new(|state: &HashMap<String, serde_json::Value>| state.contains_key("doomed"));
        *test.store.fail_hook.lock().unwrap() = Some(hook);

        let ((_, doomed_status), (_, keeper_status)) =
            race(test.store.clone(), ("doomed", false), ("keeper", false)).await;

        assert_eq!(
            doomed_status,
            StatusCode::INTERNAL_SERVER_ERROR,
            "the injected persistence failure must surface as an error status"
        );
        assert_eq!(
            keeper_status,
            StatusCode::OK,
            "the concurrent save must still commit"
        );

        // The rollback must have restored exactly the pre-doomed state, and must
        // not have discarded the keeper commit even when the keeper ran first.
        let memory = listed_ids(test.store.clone()).await;
        let disk = stored_ids(&test.path);
        assert_eq!(
            memory,
            vec!["keeper".to_string(), "seed".to_string()],
            "only the committed transaction may be visible, got {memory:?}"
        );
        assert_eq!(
            memory, disk,
            "the file must match the rolled-back memory state"
        );
        assert!(
            !memory.contains(&"doomed".to_string()),
            "the failed transaction must not be visible, got {memory:?}"
        );
    }

    /// The data file must never be observable half-written: every read while
    /// concurrent transactions are in flight is a complete JSON document, and
    /// the file is replaced by rename rather than truncated in place.
    ///
    /// A store is seeded first, so once the file exists it must *always* be a
    /// parseable, non-empty document — a writer that truncated in place would be
    /// caught observing the emptied window.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn data_file_is_always_complete_json_during_concurrent_writes() {
        let test = TestStore::new("atomic-writes");
        test.store
            .commit(|projects| {
                projects.insert("seed".to_string(), project_payload("seed", "Seed"));
            })
            .await
            .expect("seed must persist");

        let barrier = Arc::new(tokio::sync::Barrier::new(9));

        // Spawn (not join sequentially) so the writers genuinely overlap.
        let mut writers = Vec::new();
        for i in 0..8 {
            let store = test.store.clone();
            let barrier = barrier.clone();
            writers.push(tokio::spawn(async move {
                barrier.wait().await;
                let id = format!("proj-{i}");
                store
                    .commit(move |projects| {
                        projects.insert(id.clone(), project_payload(&id, &id));
                    })
                    .await
                    .expect("write must succeed");
            }));
        }

        let reader_path = test.path.clone();
        let reader = tokio::spawn(async move {
            barrier.wait().await;
            // Sample the file continuously while the writers run. The file must
            // be complete at every instant: never missing, never empty, never
            // truncated, never partially written.
            for _ in 0..500 {
                let bytes =
                    std::fs::read(&reader_path).expect("the data file must exist once seeded");
                assert!(
                    !bytes.is_empty(),
                    "the data file must never be observed empty (truncated in place)"
                );
                serde_json::from_slice::<HashMap<String, serde_json::Value>>(&bytes)
                    .expect("a partially written file must never be observable");
                tokio::task::yield_now().await;
            }
        });

        for writer in writers {
            writer.await.expect("writer must not panic");
        }
        reader.await.expect("reader must not panic");

        let ids = stored_ids(&test.path);
        assert_eq!(ids.len(), 9, "every writer must be persisted, got {ids:?}");

        // No temp files left behind.
        let leftovers: Vec<_> = std::fs::read_dir(test.path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains("atomic-writes") && n.ends_with(".tmp"))
            .collect();
        assert!(
            leftovers.is_empty(),
            "temp files left behind: {leftovers:?}"
        );
    }

    // ---------------------------------------------------------------------
    // Dashboard counts must use the same recursive definition as save.
    // ---------------------------------------------------------------------

    /// Regression: `list_projects` counted only root components, so a project
    /// with children inside a Container/Card showed a smaller count after a
    /// dashboard refresh than the count reported when it was saved.
    #[tokio::test]
    async fn dashboard_component_count_is_recursive_and_matches_save() {
        let test = TestStore::new("nested-count");
        let layout = link_layout_flow();

        let (status, body) = request(
            test.store.clone(),
            "POST",
            "/api/projects",
            Some(layout.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let saved: ProjectMetadata =
            serde_json::from_slice(&body).expect("save response must deserialize");

        let expected = 5; // root Link + Container→Link + Card→Link
        assert_eq!(
            saved.component_count, expected,
            "the save response must report the recursive count"
        );

        // The dashboard's list must agree with the save response.
        let (status, body) = request(test.store.clone(), "GET", "/api/projects", None).await;
        assert_eq!(status, StatusCode::OK);
        let listed: Vec<ProjectMetadata> = serde_json::from_slice(&body).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(
            listed[0].component_count, expected,
            "GET /api/projects must report the same recursive count as save, \
             not just the number of root components"
        );

        // And it must still hold after the store is rebuilt from the file.
        let reloaded = Store::new(read_data_file(&test.path), test.path.clone());
        let (status, body) = request(reloaded, "GET", "/api/projects", None).await;
        assert_eq!(status, StatusCode::OK);
        let after_reload: Vec<ProjectMetadata> = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            after_reload[0].component_count, expected,
            "the count must survive a reload from disk"
        );
    }
}
