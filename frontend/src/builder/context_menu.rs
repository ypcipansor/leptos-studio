use crate::domain::ComponentId;
use leptos::prelude::*;

#[component]
pub fn ContextMenu(
    #[prop(into)] visible: Signal<bool>,
    #[prop(into)] position: Signal<(f64, f64)>,
    #[prop(into)] component_id: Signal<Option<ComponentId>>,
    on_close: Callback<()>,
    on_delete: Callback<ComponentId>,
    on_duplicate: Callback<ComponentId>,
    on_select_parent: Callback<ComponentId>,
    #[prop(optional)] on_save_custom: Option<Callback<ComponentId>>,
    #[prop(optional)] on_add_container: Option<Callback<()>>,
) -> impl IntoView {
    // We attach a click listener to the window to close the menu when clicking outside
    // This is handled by the parent component or via a global listener, but usually
    // a transparent overlay is easier for modals/popups.
    // Here we assume the parent renders an overlay or handles outside clicks.

    let style = move || {
        let (x, y) = position.get();
        if visible.get() {
            format!(
                "display: block; position: fixed; left: {}px; top: {}px; z-index: 9999;",
                x, y
            )
        } else {
            "display: none;".to_string()
        }
    };

    view! {
        <div
            class="context-menu"
            style=style
            on:contextmenu=|ev| ev.prevent_default() // Prevent native menu on our menu
        >
            <div class="context-menu-header">
                {move || {
                    if let Some(id) = component_id.get() {
                        format!("Component {}", id.to_string().chars().take(8).collect::<String>())
                    } else {
                        "Canvas".to_string()
                    }
                }}
            </div>

            // Component-specific actions
            <Show when=move || component_id.get().is_some()>
                <button
                    class="context-menu-item"
                    on:click=move |_| {
                        if let Some(id) = component_id.get() {
                            on_select_parent.run(id);
                        }
                        on_close.run(());
                    }
                >
                    <span>"⬆️"</span> "Select Parent"
                </button>

                <button
                    class="context-menu-item"
                    on:click=move |_| {
                        if let Some(id) = component_id.get() {
                            on_duplicate.run(id);
                        }
                        on_close.run(());
                    }
                >
                    <span>"📋"</span> "Duplicate"
                </button>

                <Show when=move || on_save_custom.is_some()>
                    <button
                        class="context-menu-item"
                        on:click=move |_| {
                             if let Some(id) = component_id.get()
                                && let Some(cb) = on_save_custom
                             {
                                    cb.run(id);
                             }
                            on_close.run(());
                        }
                    >
                        <span>"💾"</span> "Save to Library"
                    </button>
                </Show>

                <div class="context-menu-divider"></div>

                <button
                    class="context-menu-item danger"
                    on:click=move |_| {
                        if let Some(id) = component_id.get() {
                            on_delete.run(id);
                        }
                        on_close.run(());
                    }
                >
                    <span>"🗑️"</span> "Delete"
                </button>
            </Show>

            // Generic canvas actions (shown when no component is targeted)
            <Show when=move || component_id.get().is_none() && on_add_container.is_some()>
                <button
                    class="context-menu-item"
                    on:click=move |_| {
                        if let Some(cb) = on_add_container {
                            cb.run(());
                        }
                        on_close.run(());
                    }
                >
                    <span>"📦"</span> "Add Container"
                </button>
            </Show>
        </div>

        // Invisible overlay to catch clicks outside
        <Show when=move || visible.get()>
            <div
                class="context-menu-overlay"
                on:click=move |_| on_close.run(())
                on:contextmenu=move |ev| {
                    ev.prevent_default();
                    on_close.run(());
                }
            ></div>
        </Show>
    }
}
