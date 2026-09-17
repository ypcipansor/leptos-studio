use crate::builder::breadcrumb::BreadcrumbNavigation;
use crate::builder::canvas::renderer::ComponentRenderer;
use crate::builder::component_library::{
    create_canvas_component, create_canvas_component_from_payload,
};
use crate::builder::context_menu::ContextMenu;
use crate::domain::ComponentId;
use crate::state::app_state::AppState;
use leptos::{ev, html, prelude::*};
use wasm_bindgen::JsCast;

pub mod renderer;

pub fn handle_drag_over(ev: ev::DragEvent) {
    ev.prevent_default();
}

pub fn handle_drop(ev: ev::DragEvent, _target_id: Option<ComponentId>, app_state: AppState) {
    ev.prevent_default();
    ev.stop_propagation();

    let drag_ev = ev.unchecked_into::<web_sys::DragEvent>();

    // Check for "component" (New component from palette)
    if let Some(dt) = drag_ev.data_transfer() {
        if let Ok(component_type_str) = dt.get_data("component")
            && !component_type_str.is_empty()
        {
            if let Some(new_component) = create_canvas_component_from_payload(
                &component_type_str,
                &app_state.ui.component_library.get_untracked(),
            ) {
                if let Some(target) = _target_id {
                    app_state.canvas.add_child_component(&target, new_component);
                } else {
                    app_state.canvas.add_component(new_component);
                }
            }
        }
        // Check for "move-component" (Reordering/Moving existing component)
        else {
            #[allow(clippy::collapsible_if)]
            if let Ok(move_id_str) = dt.get_data("move-component")
                && !move_id_str.is_empty()
            {
                if let Ok(move_id) = uuid::Uuid::parse_str(&move_id_str) {
                    #[allow(clippy::collapsible_if)]
                    if let Some(target) = _target_id {
                        app_state
                            .canvas
                            .move_component_to_parent(move_id.into(), target);
                    } else {
                        // Moved to root (canvas background)
                        app_state.canvas.move_component_to_root(move_id.into());
                    }
                }
            }
        }
    }

    app_state
        .canvas
        .drag_state
        .set(crate::builder::drag_drop::DragState::NotDragging);
}

#[component]
pub fn Canvas() -> impl IntoView {
    let app_state = AppState::expect_context();

    // Track canvas element for dimension measurements
    let canvas_ref = NodeRef::<html::Div>::new();

    // Context Menu State
    let (cm_visible, set_cm_visible) = signal(false);
    let (cm_position, set_cm_position) = signal((0.0, 0.0));
    let (cm_target_id, set_cm_target_id) = signal(Option::<ComponentId>::None);

    // Save Custom Component Logic
    let save_custom_component = move |id: ComponentId| {
        if let Some(comp) = app_state.canvas.get_component(&id) {
            // Convert CanvasComponent to LibraryComponent
            // This is a simplified conversion. Realistically we need a name prompt.
            // For now, we'll use a prompt via window.prompt (not ideal UX but functional for MVP)
            if let Some(window) = web_sys::window()
                && let Ok(Some(name)) =
                    window.prompt_with_message("Enter name for custom component:")
            {
                let lib_comp = crate::builder::component_library::LibraryComponent {
                    id: crate::builder::component_library::new_library_id(),
                    name,
                    kind: comp.component_type().to_string(),
                    category: "Custom".to_string(),
                    description: Some("User saved component".to_string()),
                    template: Some(serde_json::to_string_pretty(&comp).unwrap_or_default()),
                    props_schema: None, // Simplified
                };

                let mut custom = app_state.ui.custom_components.get();
                let mut library = app_state.ui.component_library.get();
                match crate::builder::component_library::ComponentRegistry::add_custom(
                    &mut custom,
                    &mut library,
                    lib_comp,
                ) {
                    Ok(()) => {
                        let saved_name = custom.last().map(|c| c.name.clone()).unwrap_or_default();
                        app_state.ui.custom_components.set(custom);
                        app_state.ui.component_library.set(library);
                        app_state
                            .ui
                            .notify(crate::state::app_state::Notification::success(format!(
                                "Saved '{}' to the Custom category",
                                saved_name
                            )));
                    }
                    // A blank or duplicate name must not silently create an
                    // entry the palette cannot tell apart from another one.
                    Err(err) => {
                        app_state
                            .ui
                            .notify(crate::state::app_state::Notification::warning(
                                err.message(),
                            ));
                    }
                }
            }
        }
    };

    // Handle background click to deselect
    let on_canvas_click = move |ev: ev::MouseEvent| {
        // Only deselect if clicking the canvas background directly
        let target = event_target::<web_sys::HtmlElement>(&ev);
        if target.id() == "main-canvas" {
            app_state.canvas.clear_selection();
        }
    };

    // Ctrl+scroll zooms the canvas
    let on_wheel = move |ev: ev::WheelEvent| {
        if ev.ctrl_key() || ev.meta_key() {
            ev.prevent_default();
            let factor = if ev.delta_y() < 0.0 { 1.1 } else { 1.0 / 1.1 };
            app_state.canvas.zoom_by(factor);
        }
    };

    // Pan by dragging the canvas background when zoomed in
    let pan_state: RwSignal<Option<(f64, f64, f64, f64)>> = RwSignal::new(None);

    let canvas_area_element = || {
        leptos::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector(".canvas-area").ok().flatten())
    };

    let on_pan_start = move |ev: ev::MouseEvent| {
        if app_state.canvas.zoom.get() <= 1.0 {
            return;
        }
        let target = event_target::<web_sys::HtmlElement>(&ev);
        if target.id() != "main-canvas" {
            return;
        }
        if let Some(area) = canvas_area_element() {
            pan_state.set(Some((
                ev.client_x() as f64,
                ev.client_y() as f64,
                area.scroll_left() as f64,
                area.scroll_top() as f64,
            )));
        }
    };

    let on_pan_move = move |ev: ev::MouseEvent| {
        if let Some((start_x, start_y, scroll_x, scroll_y)) = pan_state.get()
            && let Some(area) = canvas_area_element()
        {
            area.set_scroll_left((scroll_x - (ev.client_x() as f64 - start_x)) as i32);
            area.set_scroll_top((scroll_y - (ev.client_y() as f64 - start_y)) as i32);
        }
    };

    let on_pan_end = move |_ev: ev::MouseEvent| {
        pan_state.set(None);
    };

    // Open the context menu at the event's position; targets the closest component
    // if any, otherwise opens the generic canvas menu.
    let open_context_menu = Callback::new(move |ev: ev::MouseEvent| {
        let target = event_target::<web_sys::Element>(&ev);

        // Find closest component ID
        let found_id = target
            .closest("[data-component-id]")
            .ok()
            .flatten()
            .and_then(|closest| closest.get_attribute("data-component-id"))
            .and_then(|id_str| {
                find_component_id_by_string(&app_state.canvas.components.get_untracked(), &id_str)
            });

        set_cm_target_id.set(found_id);
        set_cm_position.set((ev.client_x() as f64, ev.client_y() as f64));
        set_cm_visible.set(true);

        // Also select it when a component was found
        if let Some(id) = found_id {
            app_state.canvas.select_single(id);
        }
    });

    // Handle context menu
    let on_context_menu = move |ev: ev::MouseEvent| {
        ev.prevent_default();
        open_context_menu.run(ev);
    };

    // Double-click opens the context menu (instead of the old select/lock behavior)
    let on_dblclick = move |ev: ev::MouseEvent| {
        // Only intercept when it hits a component, so native dblclick text
        // selection on the empty canvas keeps working
        let target = event_target::<web_sys::Element>(&ev);
        if target
            .closest("[data-component-id]")
            .ok()
            .flatten()
            .is_some()
        {
            ev.prevent_default();
        }
        open_context_menu.run(ev);
    };

    // Calculate width based on responsive mode
    let canvas_width = move || {
        use crate::state::app_state::ResponsiveMode;
        match app_state.ui.responsive_mode.get() {
            ResponsiveMode::Desktop => "100%".to_string(),
            ResponsiveMode::Tablet => "768px".to_string(),
            ResponsiveMode::TabletLandscape => "1024px".to_string(),
            ResponsiveMode::Mobile => "375px".to_string(),
            ResponsiveMode::MobileLandscape => "667px".to_string(),
        }
    };

    view! {
        <div
            class="canvas-wrapper"
            on:contextmenu=on_context_menu
            on:dblclick=on_dblclick
            on:wheel=on_wheel
            on:mousedown=on_pan_start
            on:mousemove=on_pan_move
            on:mouseup=on_pan_end
            on:mouseleave=on_pan_end
        >
            <div
                class="canvas-dropzone"
                on:click=on_canvas_click
                on:dragover=handle_drag_over
                on:drop=move |ev| handle_drop(ev, None, app_state)
            >
                <div
                    id="main-canvas"
                    node_ref=canvas_ref
                    class="canvas-surface"
                    class:preview-active=move || app_state.ui.preview_mode.get()
                    style:width=canvas_width
                    style:transform=move || format!("scale({})", app_state.canvas.zoom.get())
                    style:transform-origin="top center"
                    role="application"
                    aria-label="Canvas editor surface"
                >
                    {move || {
                        let components = app_state.canvas.components.get();

                        if components.is_empty() {
                            view! {
                                <div class="canvas-empty-state">
                                    <div class="empty-state-content">
                                        <h3>"Start from scratch"</h3>
                                        <p>"Drag components from the left sidebar or add a container to get started."</p>
                                        <button
                                            class="btn btn-primary"
                                            on:click=move |_| {
                                                if let Some(comp) = create_canvas_component("Container") {
                                                    app_state.canvas.add_component(comp);
                                                }
                                            }
                                        >
                                            "Add Container"
                                        </button>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <For
                                    each=move || components.clone()
                                    key=|comp| *comp.id()
                                    children=move |comp| {
                                        view! {
                                            <ComponentRenderer
                                                component=comp
                                                canvas_state=app_state.canvas
                                            />
                                        }
                                    }
                                />
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            // Breadcrumbs at the bottom
            <BreadcrumbNavigation />

            // Context Menu
            <ContextMenu
                visible=cm_visible
                position=cm_position
                component_id=cm_target_id
                on_close=Callback::new(move |_| set_cm_visible.set(false))
                on_delete=Callback::new(move |id| {
                    app_state.canvas.remove_component(&id);
                })
                on_duplicate=Callback::new(move |id| {
                    // Use the new duplicate_with_new_id method
                    if let Some(comp) = app_state.canvas.get_component(&id) {
                         let new_comp = comp.duplicate_with_new_id();
                         app_state.canvas.add_component(new_comp);
                    }
                })
                on_select_parent=Callback::new(move |id| {
                     if let Some(parent_id) = find_parent_id(&app_state.canvas.components.get_untracked(), id) {
                         app_state.canvas.select_single(parent_id);
                     }
                })
                on_save_custom=Callback::new(move |id| {
                     save_custom_component(id);
                })
                on_add_container=Callback::new(move |_| {
                    if let Some(comp) = create_canvas_component("Container") {
                        app_state.canvas.add_component(comp);
                    }
                })
            />
        </div>
    }
}

// Helper to find ID from string
fn find_component_id_by_string(
    components: &[crate::domain::CanvasComponent],
    id_str: &str,
) -> Option<ComponentId> {
    for comp in components {
        if comp.id().to_string() == id_str {
            return Some(*comp.id());
        }

        match comp {
            crate::domain::CanvasComponent::Container(c) => {
                if let Some(found) = find_component_id_by_string(&c.children, id_str) {
                    return Some(found);
                }
            }
            crate::domain::CanvasComponent::Card(c) => {
                if let Some(found) = find_component_id_by_string(&c.children, id_str) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

// Helper to find parent ID
fn find_parent_id(
    components: &[crate::domain::CanvasComponent],
    target_id: ComponentId,
) -> Option<ComponentId> {
    for comp in components {
        let is_parent = match comp {
            crate::domain::CanvasComponent::Container(c) => {
                c.children.iter().any(|child| *child.id() == target_id)
            }
            crate::domain::CanvasComponent::Card(c) => {
                c.children.iter().any(|child| *child.id() == target_id)
            }
            _ => false,
        };

        if is_parent {
            return Some(*comp.id());
        }

        // Recurse
        match comp {
            crate::domain::CanvasComponent::Container(c) => {
                if let Some(found) = find_parent_id(&c.children, target_id) {
                    return Some(found);
                }
            }
            crate::domain::CanvasComponent::Card(c) => {
                if let Some(found) = find_parent_id(&c.children, target_id) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}
