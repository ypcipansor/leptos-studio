use crate::builder::hooks::use_export_actions::use_export_actions;
use crate::state::app_state::{AppState, Notification, ResponsiveMode};
use leptos::prelude::*;
use leptos_router::components::A;
use wasm_bindgen::JsCast;

#[component]
pub fn Toolbar(
    show_template_gallery: WriteSignal<bool>,
    show_export: WriteSignal<bool>,
    show_save_template: WriteSignal<bool>,
    export_code: WriteSignal<String>,
    export_template: ReadSignal<String>,
) -> impl IntoView {
    let app_state = AppState::expect_context();

    // Save/Load handlers
    let save_layout = move |_| {
        app_state.save();
    };

    // Export handler
    let do_export = use_export_actions(show_export, export_code, export_template);

    // Undo/Redo handlers
    let do_undo = move |_| {
        if let Some(snapshot) = app_state.canvas.history.write().undo() {
            app_state.canvas.apply_snapshot(&snapshot);
            app_state
                .ui
                .notify(Notification::info("↪️ Undo".to_string()));
        }
    };

    let do_redo = move |_| {
        if let Some(snapshot) = app_state.canvas.history.write().redo() {
            app_state.canvas.apply_snapshot(&snapshot);
            app_state
                .ui
                .notify(Notification::info("↪️ Redo".to_string()));
        }
    };

    // History state tracking
    let can_undo = Memo::new(move |_| app_state.canvas.history.with(|h| h.can_undo()));
    let can_redo = Memo::new(move |_| app_state.canvas.history.with(|h| h.can_redo()));

    // Preview mode
    let is_preview = app_state.ui.preview_mode;
    let toggle_preview = move |_| {
        is_preview.update(|p| *p = !*p);
        if is_preview.get() {
            app_state
                .ui
                .notify(Notification::info("👁️ Preview Mode On".to_string()));
            app_state.canvas.clear_selection();
        } else {
            app_state
                .ui
                .notify(Notification::info("✏️ Edit Mode On".to_string()));
        }
    };

    // Preview mode dropdown (split button)
    let preview_menu_open = RwSignal::new(false);

    let set_preview_mode = move |preview: bool| {
        is_preview.set(preview);
        preview_menu_open.set(false);
        if preview {
            app_state.canvas.clear_selection();
        }
    };

    // Keyboard handling: Escape closes, ArrowUp/Down move focus between items
    let on_menu_keydown = move |ev: leptos::ev::KeyboardEvent| match ev.key().as_str() {
        "Escape" => preview_menu_open.set(false),
        "ArrowDown" | "ArrowUp" => {
            ev.prevent_default();
            let doc = leptos::web_sys::window()
                .and_then(|w| w.document())
                .expect("no document");
            if let Ok(list) = doc.query_selector_all(".preview-dropdown button[role='menuitem']") {
                let len = list.length() as i64;
                if len > 0 {
                    let mut current: i64 = -1;
                    for i in 0..len {
                        if let Some(item) = list.get(i as u32)
                            && let Some(active) = doc.active_element()
                            && let Ok(el) = item.dyn_into::<leptos::web_sys::Element>()
                            && active == el
                        {
                            current = i;
                            break;
                        }
                    }
                    let idx = if current == -1 {
                        if ev.key() == "ArrowDown" { 0 } else { len - 1 }
                    } else {
                        let delta = if ev.key() == "ArrowDown" { 1i64 } else { -1i64 };
                        (current + delta + len) % len
                    };
                    if let Some(item) = list.get(idx as u32) {
                        let el = item.unchecked_into::<leptos::web_sys::HtmlElement>();
                        let _ = el.focus();
                    }
                }
            }
        }
        _ => {}
    };

    view! {
        <header class="app-header">
            <div class="header-left">
                <div class="logo-area">
                    <h1>{"Leptos Studio"}</h1>
                    <span class="version-badge">"Beta"</span>
                </div>

                <div class="divider-vertical"></div>

                <div class="toolbar-group">
                    <A href="/" attr:class="btn btn-ghost btn-sm" attr:title="Manage Projects">
                        <span class="icon">"📁"</span>
                        <span class="label">"Projects"</span>
                    </A>
                </div>

                <div class="divider-vertical"></div>

                <div class="toolbar-group">
                     <span class="project-name-display" style="font-size: 0.9em; font-weight: 500; color: var(--text-color, #333); margin-right: 8px;">
                        {move || app_state.project_name.get()}
                     </span>
                </div>

                <div class="divider-vertical"></div>

                <div class="toolbar-group">
                    <button
                        on:click=save_layout
                        class="btn btn-ghost btn-sm"
                        title="Save layout (Ctrl+S)"
                    >
                        <span class="icon">"💾"</span>
                        <span class="label">"Save"</span>
                    </button>
                    <button
                        on:click=move |_| show_save_template.set(true)
                        class="btn btn-ghost btn-sm"
                        title="Save as Template"
                    >
                        <span class="icon">"💾"</span>
                        <span class="label">"Save Tpl"</span>
                    </button>
                </div>

                <div class="divider-vertical"></div>

                <div class="toolbar-group">
                    <button
                        on:click=do_undo
                        class="btn btn-ghost btn-sm"
                        disabled=move || !can_undo.get()
                        title="Undo (Ctrl+Z)"
                    >
                        <span class="icon">"↩️"</span>
                    </button>
                    <button
                        on:click=do_redo
                        class="btn btn-ghost btn-sm"
                        disabled=move || !can_redo.get()
                        title="Redo (Ctrl+Y)"
                    >
                        <span class="icon">"↪️"</span>
                    </button>
                </div>

                 <div class="divider-vertical"></div>

                 <div class="toolbar-group">
                    <button
                        on:click=do_export
                        class="btn btn-ghost btn-sm"
                        title="Export code"
                    >
                        <span class="icon">"📤"</span>
                        <span class="label">"Export"</span>
                    </button>
                 </div>
            </div>

            <div class="header-right">
                <div class="toolbar-group zoom-controls" role="group" aria-label="Canvas zoom">
                    <button
                        class="btn btn-ghost btn-sm"
                        on:click=move |_| app_state.canvas.zoom_by(1.0 / 1.1)
                        title="Zoom out (Ctrl+-)"
                    >
                        <span class="icon">"−"</span>
                    </button>
                    <button
                        class="btn btn-ghost btn-sm zoom-level"
                        on:click=move |_| app_state.canvas.reset_zoom()
                        title="Reset zoom to 100% (Ctrl+0)"
                    >
                        {move || format!("{:.0}%", app_state.canvas.zoom.get() * 100.0)}
                    </button>
                    <button
                        class="btn btn-ghost btn-sm"
                        on:click=move |_| app_state.canvas.zoom_by(1.1)
                        title="Zoom in (Ctrl+=)"
                    >
                        <span class="icon">"+"</span>
                    </button>
                </div>

                <div class="divider-vertical"></div>

                <div class="btn-group">
                    <button
                        class=move || if app_state.ui.responsive_mode.get() == ResponsiveMode::Desktop { "responsive-btn active" } else { "responsive-btn" }
                        on:click=move |_| app_state.ui.responsive_mode.set(ResponsiveMode::Desktop)
                        title="Desktop View"
                    >
                        <span class="icon">"🖥️"</span>
                    </button>
                    <button
                        class=move || if app_state.ui.responsive_mode.get() == ResponsiveMode::Tablet { "responsive-btn active" } else { "responsive-btn" }
                        on:click=move |_| app_state.ui.responsive_mode.set(ResponsiveMode::Tablet)
                        title="Tablet View"
                    >
                        <span class="icon">"📱"</span>
                    </button>
                    <button
                        class=move || if app_state.ui.responsive_mode.get() == ResponsiveMode::Mobile { "responsive-btn active" } else { "responsive-btn" }
                        on:click=move |_| app_state.ui.responsive_mode.set(ResponsiveMode::Mobile)
                        title="Mobile View"
                    >
                        <span class="icon">"📲"</span>
                    </button>
                </div>

                <div class="divider-vertical"></div>

                <div class="preview-split">
                    <button
                        class=move || if is_preview.get() { "preview-main btn btn-primary btn-sm toggle-active" } else { "preview-main btn btn-outline btn-sm" }
                        on:click=toggle_preview
                        title="Toggle Preview Mode"
                    >
                        <span class="icon">{move || if is_preview.get() { "👁️" } else { "✏️" }}</span>
                        <span class="label">{move || if is_preview.get() { "Preview" } else { "Edit" }}</span>
                    </button>
                    <button
                        class=move || if is_preview.get() { "preview-toggle btn btn-primary btn-sm toggle-active" } else { "preview-toggle btn btn-outline btn-sm" }
                        on:click=move |_| preview_menu_open.update(|o| *o = !*o)
                        aria-haspopup="menu"
                        aria-expanded=move || preview_menu_open.get().to_string()
                        title="Preview mode options"
                    >
                        <span class="icon">"▾"</span>
                    </button>
                    {move || {
                        if preview_menu_open.get() {
                            view! {
                                <div class="preview-menu-overlay" on:click=move |_| preview_menu_open.set(false)></div>
                                <div class="preview-dropdown" role="menu" on:keydown=on_menu_keydown>
                                    <button role="menuitem" class="dropdown-item"
                                        class:active=!is_preview.get()
                                        on:click=move |_| set_preview_mode(false)
                                    >
                                        <span class="icon">"✏️"</span>
                                        <span class="label">"Edit Mode"</span>
                                    </button>
                                    <button role="menuitem" class="dropdown-item"
                                        class:active=is_preview.get()
                                        on:click=move |_| set_preview_mode(true)
                                    >
                                        <span class="icon">"👁️"</span>
                                        <span class="label">"Preview Mode"</span>
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                </div>

                <div class="divider-vertical"></div>

                <button
                    class="btn btn-ghost btn-sm"
                    on:click=move |_| show_template_gallery.set(true)
                    title="Open template gallery"
                >
                    <span class="icon">"📑"</span>
                    <span class="label">"Templates"</span>
                </button>

                <div class="divider-vertical"></div>

                <button
                    class="btn btn-ghost btn-sm"
                    on:click=move |_| app_state.ui.show_settings_modal.set(true)
                    title="Settings"
                >
                    <span class="icon">"⚙️"</span>
                </button>
                <button
                    class="btn btn-ghost btn-sm"
                    on:click=move |_| app_state.ui.show_shortcuts_modal.set(true)
                    title="Shortcuts (?)"
                >
                    <span class="icon">"❓"</span>
                </button>
            </div>
        </header>
    }
}
