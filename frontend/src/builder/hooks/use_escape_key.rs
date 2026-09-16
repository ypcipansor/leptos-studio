//! Global modal dismissal hooks (Escape key + backdrop click).

use leptos::prelude::*;

/// Register a `keydown` listener that invokes `on_escape` when Escape is pressed.
///
/// The listener is attached only while `active` is true, and is removed on
/// cleanup so open/close cycles do not leak handlers.
pub fn use_escape_key(active: RwSignal<bool>, on_escape: impl Fn() + 'static + Clone) {
    Effect::new(move |_| {
        if !active.get() {
            return;
        }
        let cb = on_escape.clone();
        let handle =
            window_event_listener(leptos::ev::keydown, move |ev: leptos::ev::KeyboardEvent| {
                if ev.key() == "Escape" {
                    cb();
                }
            });
        on_cleanup(move || handle.remove());
    });
}
