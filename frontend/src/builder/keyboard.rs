//! Keyboard Shortcut System
//!
//! Provides a comprehensive keyboard shortcut system with platform-aware
//! modifier keys (Cmd on Mac, Ctrl on Windows/Linux) and action dispatch.
//!
//! # Features
//! * Platform-aware keyboard shortcuts
//! * Prevents conflicts with input fields
//! * Configurable shortcut definitions
//! * Action-based dispatch system

use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;

/// Actions that can be triggered by keyboard shortcuts
#[derive(Clone, Debug, PartialEq)]
pub enum KeyboardAction {
    Undo,
    Redo,
    Delete,
    Copy,
    Paste,
    SelectAll,
    Deselect,
    OpenCommandPalette,
    Save,
    Export,
    NewComponent,
    AddComponent(String),
    Duplicate,
    Cut,
    ShowShortcuts,
    MoveUp,
    MoveDown,
    ZoomIn,
    ZoomOut,
    ZoomReset,
}

/// Defines a keyboard shortcut with modifiers and action
#[derive(Clone, Debug)]
pub struct KeyboardShortcut {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    pub action: KeyboardAction,
    pub description: String,
}

impl KeyboardShortcut {
    /// Create a new keyboard shortcut
    ///
    /// # Arguments
    /// * `key` - The key code (e.g., "z", "Delete")
    /// * `ctrl` - Whether Ctrl key is required
    /// * `shift` - Whether Shift key is required
    /// * `alt` - Whether Alt key is required
    /// * `meta` - Whether Meta/Cmd key is required
    /// * `action` - The action to trigger
    /// * `description` - Human-readable description
    pub fn new(
        key: &str,
        ctrl: bool,
        shift: bool,
        alt: bool,
        meta: bool,
        action: KeyboardAction,
        description: &str,
    ) -> Self {
        Self {
            key: key.to_string(),
            ctrl,
            shift,
            alt,
            meta,
            action,
            description: description.to_string(),
        }
    }

    /// Check if a keyboard event matches this shortcut
    ///
    /// Compares both key and modifiers to determine if the event
    /// should trigger this shortcut's action.
    pub fn matches(&self, event: &KeyboardEvent) -> bool {
        let key_match = self.key.to_lowercase() == event.key().to_lowercase()
            || self.key.to_lowercase() == event.code().to_lowercase();

        key_match
            && self.ctrl == event.ctrl_key()
            && self.shift == event.shift_key()
            && self.alt == event.alt_key()
            && self.meta == event.meta_key()
    }

    /// Get a human-readable display string for the shortcut
    ///
    /// Returns a string like "⌘ + Z" or "Ctrl + Shift + A"
    pub fn display_string(&self) -> String {
        let mut parts = Vec::new();

        if self.meta {
            parts.push("⌘".to_string());
        }
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.shift {
            parts.push("⇧".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }

        let key_upper = self.key.to_uppercase();
        parts.push(key_upper);
        parts.join(" + ")
    }
}

/// Get the default set of keyboard shortcuts
///
/// Returns a comprehensive list of all keyboard shortcuts used by the application,
/// including editing, navigation, file operations, and component management.
///
/// # Default Shortcuts
/// * **Ctrl+Z**: Undo
/// * **Ctrl+Y / Ctrl+Shift+Z**: Redo
/// * **Delete / Backspace**: Delete selected
/// * **Ctrl+C**: Copy
/// * **Ctrl+V**: Paste
/// * **Ctrl+D**: Duplicate
/// * **Ctrl+A**: Select all
/// * **Escape**: Deselect
/// * **Ctrl+K**: Open command palette
/// * **Ctrl+S**: Save
/// * **Ctrl+E**: Export
/// * **Ctrl+N**: New component
pub fn get_default_shortcuts() -> Vec<KeyboardShortcut> {
    vec![
        KeyboardShortcut::new(
            "z",
            true,
            false,
            false,
            false,
            KeyboardAction::Undo,
            "Undo last action",
        ),
        KeyboardShortcut::new(
            "z",
            true,
            true,
            false,
            false,
            KeyboardAction::Redo,
            "Redo last action",
        ),
        KeyboardShortcut::new(
            "y",
            true,
            false,
            false,
            false,
            KeyboardAction::Redo,
            "Redo last action",
        ),
        KeyboardShortcut::new(
            "Delete",
            false,
            false,
            false,
            false,
            KeyboardAction::Delete,
            "Delete selected component",
        ),
        KeyboardShortcut::new(
            "Backspace",
            false,
            false,
            false,
            false,
            KeyboardAction::Delete,
            "Delete selected component",
        ),
        KeyboardShortcut::new(
            "c",
            true,
            false,
            false,
            false,
            KeyboardAction::Copy,
            "Copy selected component",
        ),
        KeyboardShortcut::new(
            "v",
            true,
            false,
            false,
            false,
            KeyboardAction::Paste,
            "Paste component",
        ),
        KeyboardShortcut::new(
            "a",
            true,
            false,
            false,
            false,
            KeyboardAction::SelectAll,
            "Select all components",
        ),
        KeyboardShortcut::new(
            "=",
            true,
            false,
            false,
            false,
            KeyboardAction::ZoomIn,
            "Zoom in canvas",
        ),
        KeyboardShortcut::new(
            "-",
            true,
            false,
            false,
            false,
            KeyboardAction::ZoomOut,
            "Zoom out canvas",
        ),
        KeyboardShortcut::new(
            "0",
            true,
            false,
            false,
            false,
            KeyboardAction::ZoomReset,
            "Reset canvas zoom to 100%",
        ),
        KeyboardShortcut::new(
            "Escape",
            false,
            false,
            false,
            false,
            KeyboardAction::Deselect,
            "Deselect all",
        ),
        KeyboardShortcut::new(
            "k",
            true,
            false,
            false,
            false,
            KeyboardAction::OpenCommandPalette,
            "Open command palette",
        ),
        KeyboardShortcut::new(
            "s",
            true,
            false,
            false,
            false,
            KeyboardAction::Save,
            "Save project",
        ),
        KeyboardShortcut::new(
            "e",
            true,
            false,
            false,
            false,
            KeyboardAction::Export,
            "Export code",
        ),
        KeyboardShortcut::new(
            "n",
            true,
            false,
            false,
            false,
            KeyboardAction::NewComponent,
            "New component",
        ),
        KeyboardShortcut::new(
            "d",
            true,
            false,
            false,
            false,
            KeyboardAction::Duplicate,
            "Duplicate selected",
        ),
        KeyboardShortcut::new(
            "x",
            true,
            false,
            false,
            false,
            KeyboardAction::Cut,
            "Cut selected component",
        ),
        KeyboardShortcut::new(
            "?",
            false,
            true, // ? is Shift+/ usually, but event.key is "?"
            false,
            false,
            KeyboardAction::ShowShortcuts,
            "Show shortcuts",
        ),
        KeyboardShortcut::new(
            "ArrowUp",
            false,
            false,
            true, // Alt + Up
            false,
            KeyboardAction::MoveUp,
            "Move component up",
        ),
        KeyboardShortcut::new(
            "ArrowDown",
            false,
            false,
            true, // Alt + Down
            false,
            KeyboardAction::MoveDown,
            "Move component down",
        ),
    ]
}

/// Decide whether a keydown event may trigger a canvas shortcut.
///
/// Shortcuts are suppressed for events aimed at text inputs (so typing never
/// edits the canvas) and whenever a modal or dialog is open, since acting on the
/// hidden canvas behind it is never what the user intended.
pub fn should_dispatch_shortcut(modal_open: bool, from_text_input: bool) -> bool {
    !modal_open && !from_text_input
}

/// Whether any dialog that overlays the editor is open.
///
/// Every argument is the *same* signal that gates the corresponding modal's own
/// rendering in [`EditorPage`], so the shortcut gate can never disagree with
/// what is actually on screen. Keeping the derivation in one named function lets
/// the gating be exercised with real signals in tests.
pub fn editor_modal_open(
    command_palette: bool,
    export: bool,
    settings: bool,
    shortcuts: bool,
    template_gallery: bool,
    save_template: bool,
) -> bool {
    command_palette || export || settings || shortcuts || template_gallery || save_template
}

/// Global Keyboard Handler Component
///
/// Listens for keyboard events on `window` and dispatches actions when
/// shortcuts match. Attaching the listener to the window rather than to a
/// rendered element is what makes the shortcuts fire regardless of which
/// element currently holds focus. Automatically ignores events from input
/// fields to prevent conflicts.
///
/// # Features
/// * Global keyboard event listening
/// * Smart input field detection
/// * Suppressed while a modal is open, so shortcuts cannot mutate the canvas
///   hidden behind it
/// * Prevents default browser behavior for handled shortcuts
/// * Event propagation control
///
/// # Props
/// * `shortcuts` - List of keyboard shortcuts to handle
/// * `on_action` - Callback invoked when a shortcut is triggered
/// * `modal_open` - Read signal that is true while a modal/dialog is open.
///   Defaults to `false` when omitted.
///
/// # Example
/// ```rust,ignore
/// <KeyboardHandler
///     shortcuts=get_default_shortcuts()
///     modal_open=modal_open
///     on_action=move |action| {
///         match action {
///             KeyboardAction::Undo => // handle undo
///             KeyboardAction::Redo => // handle redo
///             // ...
///         }
///     }
/// />
/// ```
#[component]
pub fn KeyboardHandler<F>(
    shortcuts: Vec<KeyboardShortcut>,
    #[prop(optional, into)] modal_open: Signal<bool>,
    on_action: F,
) -> impl IntoView
where
    F: Fn(KeyboardAction) + 'static + Clone,
{
    let handler = {
        let shortcuts = shortcuts.clone();
        move |ev: KeyboardEvent| {
            // Don't handle shortcuts when typing in inputs
            let from_text_input = ev
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
                .is_some_and(|element| {
                    let tag_name = element.tag_name().to_lowercase();
                    matches!(tag_name.as_str(), "input" | "textarea" | "select")
                        || element.is_content_editable()
                });

            if !should_dispatch_shortcut(modal_open.get_untracked(), from_text_input) {
                return;
            }

            for shortcut in &shortcuts {
                if shortcut.matches(&ev) {
                    ev.prevent_default();
                    ev.stop_propagation();
                    on_action(shortcut.action.clone());
                    break;
                }
            }
        }
    };

    Effect::new(move |_| {
        let cb = handler.clone();
        let handle = window_event_listener(leptos::ev::keydown, cb);
        on_cleanup(move || handle.remove());
    });

    view! { <div class="keyboard-handler" aria-hidden="true"></div> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcuts_fire_when_no_modal_is_open() {
        assert!(should_dispatch_shortcut(false, false));
    }

    #[test]
    fn test_shortcuts_are_suppressed_while_a_modal_is_open() {
        assert!(!should_dispatch_shortcut(true, false));
    }

    #[test]
    fn test_shortcuts_are_suppressed_for_text_inputs() {
        assert!(!should_dispatch_shortcut(false, true));
        assert!(!should_dispatch_shortcut(true, true));
    }

    #[test]
    fn test_default_shortcuts_cover_the_canvas_editing_actions() {
        let shortcuts = get_default_shortcuts();
        for action in [
            KeyboardAction::Delete,
            KeyboardAction::Undo,
            KeyboardAction::Redo,
            KeyboardAction::SelectAll,
        ] {
            assert!(
                shortcuts.iter().any(|s| s.action == action),
                "{action:?} must keep a shortcut so it can be suppressed by the modal gate"
            );
        }
    }
}
