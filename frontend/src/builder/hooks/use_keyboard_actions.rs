use crate::builder::component_library::create_canvas_component;
use crate::builder::keyboard::KeyboardAction;
use crate::domain::CanvasComponent;
use crate::services::export_service::{CodeGenerator, LeptosCodeGenerator};
use crate::state::ExportPreset;
use crate::state::app_state::{AppState, Notification};
use crate::utils::{copy_to_clipboard, read_from_clipboard};
use leptos::prelude::*;

pub fn use_keyboard_actions(
    show_export: WriteSignal<bool>,
    export_code: WriteSignal<String>,
    _show_template_gallery: WriteSignal<bool>,
) -> impl Fn(KeyboardAction) + Clone + 'static {
    let app_state = AppState::expect_context();

    move |action: KeyboardAction| {
        match action {
            KeyboardAction::Undo => {
                if let Some(snapshot) = app_state.canvas.history.write().undo() {
                    app_state.canvas.apply_snapshot(&snapshot);
                    app_state
                        .ui
                        .notification
                        .set(Some(Notification::info("↩️ Undo".to_string())));
                } else {
                    app_state.ui.notification.set(Some(Notification::warning(
                        "⚠️ Nothing to undo".to_string(),
                    )));
                }
            }
            KeyboardAction::Redo => {
                if let Some(snapshot) = app_state.canvas.history.write().redo() {
                    app_state.canvas.apply_snapshot(&snapshot);
                    app_state
                        .ui
                        .notification
                        .set(Some(Notification::info("↪️ Redo".to_string())));
                } else {
                    app_state.ui.notification.set(Some(Notification::warning(
                        "⚠️ Nothing to redo".to_string(),
                    )));
                }
            }
            KeyboardAction::Save => {
                app_state.save();
            }
            KeyboardAction::Delete => {
                let multi = app_state.canvas.selected_components.get();
                if multi.len() > 1 {
                    let count = multi.len();
                    for id in &multi {
                        app_state.canvas.remove_component(id);
                    }
                    app_state.canvas.clear_selection();
                    app_state
                        .ui
                        .notification
                        .set(Some(Notification::success(format!(
                            "🗑️ {count} components deleted"
                        ))));
                } else if let Some(selected_id) = app_state.canvas.selected.get() {
                    // remove_component already records snapshot
                    app_state.canvas.remove_component(&selected_id);
                    app_state.canvas.clear_selection();
                    app_state.ui.notification.set(Some(Notification::success(
                        "🗑️ Component deleted".to_string(),
                    )));
                } else {
                    app_state.ui.notification.set(Some(Notification::warning(
                        "⚠️ No component selected".to_string(),
                    )));
                }
            }
            KeyboardAction::Copy => {
                if let Some(selected_id) = app_state.canvas.selected.get() {
                    if let Some(comp) = app_state.canvas.get_component(&selected_id) {
                        match serde_json::to_string(&comp) {
                            Ok(json) => {
                                let app_state_clone = app_state;
                                wasm_bindgen_futures::spawn_local(async move {
                                    match copy_to_clipboard(&json).await {
                                        Ok(()) => {
                                            app_state_clone.ui.notification.set(Some(
                                                Notification::success(
                                                    "📋 Component copied!".to_string(),
                                                ),
                                            ));
                                        }
                                        Err(e) => {
                                            app_state_clone.ui.notification.set(Some(
                                                Notification::error(format!(
                                                    "❌ {}",
                                                    e.user_message()
                                                )),
                                            ));
                                        }
                                    }
                                });
                            }
                            Err(_) => {
                                app_state.ui.notification.set(Some(Notification::error(
                                    "❌ Failed to serialize component".to_string(),
                                )));
                            }
                        }
                    }
                } else {
                    app_state.ui.notification.set(Some(Notification::warning(
                        "⚠️ No component selected".to_string(),
                    )));
                }
            }
            KeyboardAction::Paste => {
                let app_state_clone = app_state;
                wasm_bindgen_futures::spawn_local(async move {
                    match read_from_clipboard().await {
                        Ok(text) => {
                            match serde_json::from_str::<CanvasComponent>(&text) {
                                Ok(comp) => {
                                    // add_component already records snapshot
                                    app_state_clone.canvas.add_component(comp);
                                    app_state_clone.ui.notification.set(Some(
                                        Notification::success("📋 Component pasted!".to_string()),
                                    ));
                                }
                                Err(_) => {
                                    app_state_clone
                                        .ui
                                        .notification
                                        .set(Some(Notification::error(
                                            "⚠️ Invalid clipboard content".to_string(),
                                        )));
                                }
                            }
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
            KeyboardAction::Duplicate => {
                if let Some(selected_id) = app_state.canvas.selected.get() {
                    if let Some(comp) = app_state.canvas.get_component(&selected_id) {
                        // add_component already records snapshot
                        app_state.canvas.add_component(comp);
                        app_state.ui.notification.set(Some(Notification::success(
                            "🔄 Component duplicated!".to_string(),
                        )));
                    }
                } else {
                    app_state.ui.notification.set(Some(Notification::warning(
                        "⚠️ No component selected".to_string(),
                    )));
                }
            }
            KeyboardAction::NewComponent => {
                app_state.ui.notification.set(Some(Notification::info(
                    "ℹ️ Drag component from sidebar to add".to_string(),
                )));
            }
            KeyboardAction::AddComponent(type_name) => {
                if let Some(comp) = create_canvas_component(&type_name) {
                    if let Some(selected_id) = app_state.canvas.selected.get() {
                        // Try to add as child if selected is container
                        if app_state
                            .canvas
                            .add_child_component(&selected_id, comp.clone())
                        {
                            app_state
                                .ui
                                .notification
                                .set(Some(Notification::success(format!(
                                    "➕ Added {} to selected container",
                                    type_name
                                ))));
                        } else {
                            // If adding as child failed (not a container), notify user
                            app_state.ui.notification.set(Some(Notification::warning(
                                "⚠️ Selected component cannot accept children. Select a Container or Card, or deselect to add to root.".to_string(),
                            )));
                        }
                    } else {
                        // Add to root
                        app_state.canvas.add_component(comp);
                        app_state
                            .ui
                            .notification
                            .set(Some(Notification::success(format!(
                                "➕ Added {}",
                                type_name
                            ))));
                    }
                }
            }
            KeyboardAction::OpenCommandPalette => {
                app_state.ui.show_command_palette.set(true);
            }
            KeyboardAction::Deselect => {
                app_state.canvas.clear_selection();
            }
            KeyboardAction::SelectAll => {
                let ids: Vec<_> = app_state
                    .canvas
                    .components
                    .get()
                    .iter()
                    .map(|c| *c.id())
                    .collect();
                if ids.is_empty() {
                    app_state.ui.notification.set(Some(Notification::warning(
                        "⚠️ No components to select".to_string(),
                    )));
                } else {
                    let count = ids.len();
                    app_state.canvas.selected_components.set(ids.clone());
                    app_state.canvas.selected.set(ids.last().copied());
                    app_state
                        .ui
                        .notification
                        .set(Some(Notification::info(format!(
                            "✓ {count} components selected"
                        ))));
                }
            }
            KeyboardAction::ZoomIn => {
                app_state.canvas.zoom_by(1.1);
            }
            KeyboardAction::ZoomOut => {
                app_state.canvas.zoom_by(1.0 / 1.1);
            }
            KeyboardAction::ZoomReset => {
                app_state.canvas.reset_zoom();
            }
            KeyboardAction::Export => {
                let comps = app_state.canvas.components.get();
                let variables = app_state.variables.get();
                let generator = LeptosCodeGenerator::new(ExportPreset::Plain);

                match generator.generate(&comps, &variables) {
                    Ok(code) => {
                        export_code.set(code);
                        show_export.set(true);
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
            }
            KeyboardAction::Cut => {
                if let Some(selected_id) = app_state.canvas.selected.get() {
                    if let Some(comp) = app_state.canvas.get_component(&selected_id) {
                        match serde_json::to_string(&comp) {
                            Ok(json) => {
                                let app_state_clone = app_state;
                                wasm_bindgen_futures::spawn_local(async move {
                                    match copy_to_clipboard(&json).await {
                                        Ok(()) => {
                                            app_state_clone.canvas.remove_component(&selected_id);
                                            app_state_clone.canvas.clear_selection();
                                            app_state_clone.ui.notification.set(Some(
                                                Notification::success(
                                                    "✂️ Component cut!".to_string(),
                                                ),
                                            ));
                                        }
                                        Err(e) => {
                                            app_state_clone.ui.notification.set(Some(
                                                Notification::error(format!(
                                                    "❌ {}",
                                                    e.user_message()
                                                )),
                                            ));
                                        }
                                    }
                                });
                            }
                            Err(_) => {
                                app_state.ui.notification.set(Some(Notification::error(
                                    "❌ Failed to serialize component".to_string(),
                                )));
                            }
                        }
                    }
                } else {
                    app_state.ui.notification.set(Some(Notification::warning(
                        "⚠️ No component selected".to_string(),
                    )));
                }
            }
            KeyboardAction::ShowShortcuts => {
                app_state.ui.show_shortcuts_modal.set(true);
            }
            KeyboardAction::MoveUp => {
                if let Some(id) = app_state.canvas.selected.get() {
                    app_state.canvas.move_component_up(&id);
                }
            }
            KeyboardAction::MoveDown => {
                if let Some(id) = app_state.canvas.selected.get() {
                    app_state.canvas.move_component_down(&id);
                }
            }
        }
    }
}
