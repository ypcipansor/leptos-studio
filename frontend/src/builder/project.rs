// ...existing code...
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::builder::component_library::LibraryComponent;
use crate::domain::CanvasComponent;
use crate::services::{project_from_json, project_to_json};
use crate::state::app_state::{AppState, Notification};
use crate::utils::copy_to_clipboard;
use js_sys::encode_uri_component;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub layout: Vec<CanvasComponent>,
    pub component_library: Vec<LibraryComponent>,
}

impl ProjectFile {
    pub fn new(layout: Vec<CanvasComponent>, component_library: Vec<LibraryComponent>) -> Self {
        Self {
            layout,
            component_library,
        }
    }
}

#[component]
pub fn ProjectPanel() -> impl IntoView {
    let app_state = AppState::expect_context();

    let import_text = RwSignal::new(String::new());

    let new_project = move |_| {
        app_state.project_name.set("Untitled Project".to_string());
        app_state.canvas.components.set(Vec::new());
        app_state.canvas.clear_selection();
        app_state.canvas.history.update(|h| h.clear());
        app_state.ui.notification.set(Some(Notification::info(
            "🆕 New project created".to_string(),
        )));
    };

    let export_project_copy = move |_| {
        let project = app_state.to_project();
        match project_to_json(&project) {
            Ok(json) => {
                let app_state_clone = app_state;
                let json_string = json.to_string(); // Ensure ownership for async block
                wasm_bindgen_futures::spawn_local(async move {
                    match copy_to_clipboard(&json_string).await {
                        Ok(()) => {
                            app_state_clone
                                .ui
                                .notification
                                .set(Some(Notification::success(
                                    "📋 Project JSON copied to clipboard".to_string(),
                                )));
                        }
                        Err(e) => {
                            app_state_clone
                                .ui
                                .notification
                                .set(Some(Notification::error(format!(
                                    "❌ {}",
                                    e.user_message()
                                ))));
                        }
                    }
                });
            }
            Err(e) => {
                app_state
                    .ui
                    .notification
                    .set(Some(Notification::error(format!(
                        "❌ {}",
                        e.user_message()
                    ))));
            }
        }
    };

    let export_project_download = move |_| {
        let project = app_state.to_project();
        match project_to_json(&project) {
            Ok(json) => {
                let encoded: String = encode_uri_component(&json).into();
                let url = format!("data:application/json;charset=utf-8,{}", encoded);

                if let Some(window) = web_sys::window() {
                    let _ = window.open_with_url_and_target(&url, "_blank");
                } else {
                    app_state.ui.notification.set(Some(Notification::error(
                        "❌ Unable to open download window".to_string(),
                    )));
                }
            }
            Err(e) => {
                app_state
                    .ui
                    .notification
                    .set(Some(Notification::error(format!(
                        "❌ {}",
                        e.user_message()
                    ))));
            }
        }
    };

    let import_project = move |_| {
        let json = import_text.get();
        match project_from_json(&json) {
            Ok(project) => {
                app_state.apply_project(project);
                app_state.ui.notification.set(Some(Notification::success(
                    "📂 Project imported".to_string(),
                )));
            }
            Err(e) => {
                app_state
                    .ui
                    .notification
                    .set(Some(Notification::error(format!(
                        "❌ {}",
                        e.user_message()
                    ))));
            }
        }
    };

    view! {
        <div class="sidebar-section sidebar-section-divider">
            <h3>"Project"</h3>
            <div class="project-panel-layout">
                <div>
                    <label class="project-label">"Name"</label>
                    <input
                        type="text"
                        prop:value=move || app_state.project_name.get()
                        on:input=move |ev| app_state.project_name.set(event_target_value(&ev))
                    />
                </div>
                <div class="project-actions-row">
                    <button on:click=new_project class="btn btn-secondary">"New"</button>
                    <button on:click=export_project_copy class="btn btn-secondary">"Copy JSON"</button>
                    <button on:click=export_project_download class="btn btn-secondary">"Download JSON"</button>
                </div>
                <div>
                    <label class="project-label">"Import JSON"</label>
                    <textarea
                        class="project-import-textarea"
                        prop:value=move || import_text.get()
                        on:input=move |ev| import_text.set(event_target_value(&ev))
                    />
                    <button on:click=import_project class="btn btn-outline project-apply-button">"Apply"</button>
                </div>
            </div>
        </div>
    }
}
