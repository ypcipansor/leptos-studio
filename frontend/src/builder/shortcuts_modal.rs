use crate::builder::keyboard::get_default_shortcuts;
use crate::state::app_state::AppState;
use leptos::prelude::*;

#[component]
pub fn ShortcutsModal() -> impl IntoView {
    use crate::builder::hooks::use_escape_key::use_escape_key;

    let app_state = AppState::expect_context();
    let show = app_state.ui.show_shortcuts_modal;

    use_escape_key(show, move || show.set(false));

    view! {
        <Show when=move || show.get()>
            <div
                class="modal-backdrop"
                role="presentation"
                on:click=move |_| show.set(false)
            >
                <div
                    class="modal-content shortcuts-modal"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="shortcuts-modal-title"
                    on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    <div class="modal-header">
                        <h3 id="shortcuts-modal-title">"Keyboard Shortcuts"</h3>
                        <button
                            class="close-btn"
                            aria-label="Close keyboard shortcuts"
                            on:click=move |_| show.set(false)
                        >
                            "×"
                        </button>
                    </div>
                    <div class="modal-body">
                        <div class="shortcuts-grid">
                            <For
                                each=move || get_default_shortcuts()
                                key=|s| s.key.clone() + &s.action.clone().get_debug_label()
                                children=move |shortcut| {
                                    view! {
                                        <div class="shortcut-row">
                                            <div class="shortcut-desc">{shortcut.description.clone()}</div>
                                            <div class="shortcut-keys">
                                                <kbd>{shortcut.display_string()}</kbd>
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

// Helper trait to get debug label for unique key
trait DebugLabel {
    fn get_debug_label(&self) -> String;
}

impl DebugLabel for crate::builder::keyboard::KeyboardAction {
    fn get_debug_label(&self) -> String {
        format!("{:?}", self)
    }
}
