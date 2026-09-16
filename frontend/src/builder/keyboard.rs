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
/// * Prevents default browser behavior for handled shortcuts
/// * Event propagation control
///
/// # Props
/// * `shortcuts` - List of keyboard shortcuts to handle
/// * `on_action` - Callback invoked when a shortcut is triggered
///
/// # Example
/// ```rust,ignore
/// <KeyboardHandler
///     shortcuts=get_default_shortcuts()
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
pub fn KeyboardHandler<F>(shortcuts: Vec<KeyboardShortcut>, on_action: F) -> impl IntoView
where
    F: Fn(KeyboardAction) + 'static + Clone,
{
    let handler = {
        let shortcuts = shortcuts.clone();
        move |ev: KeyboardEvent| {
            // Don't handle shortcuts when typing in inputs
            if let Some(target) = ev.target()
                && let Ok(element) = target.dyn_into::<web_sys::HtmlElement>()
            {
                let tag_name = element.tag_name().to_lowercase();
                if tag_name == "input" || tag_name == "textarea" || tag_name == "select" {
                    return;
                }
                if element.is_content_editable() {
                    return;
                }
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
