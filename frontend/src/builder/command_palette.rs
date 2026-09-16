//! Command Palette Component
//!
//! VS Code-style command palette for quick access to all application commands.
//! Features fuzzy search, keyboard navigation, and command execution.

use crate::builder::keyboard::KeyboardAction;
use leptos::prelude::*;

/// Represents a single command in the command palette
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub category: String,
    pub action: KeyboardAction,
}

impl Command {
    /// Create a new command
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the command
    /// * `title` - Display title shown in palette
    /// * `category` - Category for grouping (e.g., "Edit", "File")
    /// * `action` - The keyboard action to execute
    pub fn new(id: &str, title: &str, category: &str, action: KeyboardAction) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            category: category.to_string(),
            action,
        }
    }
}

/// Fuzzy match algorithm for searching commands
///
/// Returns a score if the pattern matches the text, with higher scores
/// for consecutive character matches.
///
/// # Arguments
/// * `text` - The text to search in
/// * `pattern` - The search pattern
///
/// # Returns
/// * `Some(score)` if pattern matches, `None` otherwise
fn fuzzy_match(text: &str, pattern: &str) -> Option<i32> {
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    let mut pattern_idx = 0;
    let mut score = 0;
    let mut consecutive_matches = 0;

    for &text_char in text_chars.iter() {
        if pattern_idx < pattern_chars.len() && text_char == pattern_chars[pattern_idx] {
            pattern_idx += 1;
            consecutive_matches += 1;
            score += consecutive_matches * 10;

            if pattern_idx == pattern_chars.len() {
                return Some(score);
            }
        } else {
            consecutive_matches = 0;
        }
    }

    None
}

/// Get all available commands
///
/// Returns the complete list of commands available in the command palette,
/// organized by category (Edit, File, Components, Selection).
fn get_commands() -> Vec<Command> {
    vec![
        // Edit commands
        Command::new("undo", "Undo", "Edit", KeyboardAction::Undo),
        Command::new("redo", "Redo", "Edit", KeyboardAction::Redo),
        // Delete
        Command::new("delete", "Delete Selected", "Edit", KeyboardAction::Delete),
        // Copy/Paste
        Command::new("copy", "Copy", "Edit", KeyboardAction::Copy),
        // Paste
        Command::new("paste", "Paste", "Edit", KeyboardAction::Paste),
        // Selection
        Command::new(
            "select_all",
            "Select All",
            "Selection",
            KeyboardAction::SelectAll,
        ),
        // Deselect
        Command::new(
            "deselect",
            "Deselect All",
            "Selection",
            KeyboardAction::Deselect,
        ),
        // File operations
        Command::new("save", "Save Project", "File", KeyboardAction::Save),
        // Export
        Command::new("export", "Export Code", "File", KeyboardAction::Export),
        // Components
        Command::new(
            "new_component",
            "Add Component",
            "Components",
            KeyboardAction::NewComponent,
        ),
        // Specific components
        Command::new(
            "add_button",
            "Add Button",
            "Components",
            KeyboardAction::AddComponent("Button".to_string()),
        ),
        Command::new(
            "add_text",
            "Add Text",
            "Components",
            KeyboardAction::AddComponent("Text".to_string()),
        ),
        Command::new(
            "add_input",
            "Add Input",
            "Components",
            KeyboardAction::AddComponent("Input".to_string()),
        ),
        Command::new(
            "add_container",
            "Add Container",
            "Components",
            KeyboardAction::AddComponent("Container".to_string()),
        ),
        Command::new(
            "add_card",
            "Add Card",
            "Components",
            KeyboardAction::AddComponent("Card".to_string()),
        ),
        Command::new(
            "add_image",
            "Add Image",
            "Components",
            KeyboardAction::AddComponent("Image".to_string()),
        ),
        // Arrange
        Command::new("move_up", "Move Up", "Arrange", KeyboardAction::MoveUp),
        Command::new(
            "move_down",
            "Move Down",
            "Arrange",
            KeyboardAction::MoveDown,
        ),
    ]
}

/// VS Code-style Command Palette Component
///
/// A modal overlay that provides quick access to all application commands
/// through fuzzy search and keyboard navigation.
///
/// # Features
/// * Fuzzy search across command titles and categories
/// * Keyboard navigation (Arrow Up/Down, Enter, Escape)
/// * Mouse hover selection
/// * Command execution through callback
///
/// # Keyboard Shortcuts
/// * `↑/↓` - Navigate commands
/// * `Enter` - Execute selected command
/// * `Escape` - Close palette
///
/// # Props
/// * `is_open` - Read signal controlling visibility
/// * `close` - Write signal to close the palette
/// * `search` - RwSignal for search input
/// * `on_action` - Callback executed when a command is selected
#[component]
pub fn CommandPalette<F>(
    is_open: ReadSignal<bool>,
    close: WriteSignal<bool>,
    #[prop(into)] search: RwSignal<String>,
    on_action: F,
) -> impl IntoView
where
    F: Fn(KeyboardAction) + 'static + Clone + Send + Sync,
{
    let (filtered_commands, set_filtered_commands) = signal(get_commands());
    let (selected_index, set_selected_index) = signal(0);

    // Update filtered commands when search changes
    Effect::new(move |_| {
        let search_term = search.get().to_lowercase();
        let commands = get_commands();
        let filtered = if search_term.is_empty() {
            commands
        } else {
            commands
                .into_iter()
                .filter_map(|cmd| {
                    let score = fuzzy_match(&cmd.title.to_lowercase(), &search_term)
                        .or_else(|| fuzzy_match(&cmd.category.to_lowercase(), &search_term))?;
                    Some((cmd, score))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(cmd, _)| cmd)
                .collect()
        };
        set_filtered_commands.set(filtered);
        set_selected_index.set(0);
    });

    // Clone on_action for use in multiple closures
    let on_action_clone = on_action.clone();

    view! {
        <Show when=move || is_open.get()>
            <div
                class="command-palette-backdrop"
                on:click=move |_| close.set(false)
            >
                <div
                    class="command-palette"
                    role="dialog"
                    aria-modal="true"
                    aria-label="Command palette"
                    on:click=move |ev| ev.stop_propagation()
                    on:keydown={
                        let on_action_keydown = on_action_clone.clone();
                        move |ev: web_sys::KeyboardEvent| {
                            let key = ev.key();
                            match key.as_str() {
                                "ArrowUp" => {
                                    ev.prevent_default();
                                    set_selected_index.update(|idx| {
                                        let len = filtered_commands.get().len();
                                        *idx = if *idx > 0 { *idx - 1 } else { len.saturating_sub(1) };
                                    });
                                }
                                "ArrowDown" => {
                                    ev.prevent_default();
                                    set_selected_index.update(|idx| {
                                        let len = filtered_commands.get().len();
                                        if len > 0 {
                                            *idx = (*idx + 1) % len;
                                        }
                                    });
                                }
                                "Enter" => {
                                    ev.prevent_default();
                                    let commands = filtered_commands.get();
                                    if let Some(command) = commands.get(selected_index.get()) {
                                        on_action_keydown.clone()(command.action.clone());
                                        close.set(false);
                                    }
                                }
                                "Escape" => {
                                    ev.prevent_default();
                                    close.set(false);
                                }
                                _ => {}
                            }
                        }
                    }
                >
                    <h2 class="command-palette-title">"Command Palette"</h2>
                    <div class="command-palette-search">
                        <input
                            type="text"
                            aria-label="Search commands"
                            placeholder="Search commands..."
                            prop:value=move || search.get()
                            on:input=move |ev| {
                                search.set(event_target_value(&ev));
                            }
                        />
                    </div>

                    <div class="command-palette-results">
                        <For
                            each=move || filtered_commands.get().into_iter().enumerate()
                            key=|(idx, cmd)| format!("{}-{}", idx, cmd.id)
                            children={
                                let on_action_for = on_action_clone.clone();
                                move |(idx, command): (usize, Command)| {
                                let is_selected = move || selected_index.get() == idx;
                                let command_clone = command.clone();
                                let on_action_click = on_action_for.clone();

                                view! {
                                    <div
                                        class="command-item"
                                        class:selected=is_selected
                                        on:click={
                                            let command = command_clone.clone();
                                            move |_| {
                                                on_action_click.clone()(command.action.clone());
                                                close.set(false);
                                            }
                                        }
                                        on:mouseenter=move |_| set_selected_index.set(idx)
                                    >
                                        <div>
                                            <div class="command-item-title">
                                                {command.title}
                                            </div>
                                            <div class="command-item-meta">
                                                {command.category}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }}
                        />

                        <Show when=move || filtered_commands.get().is_empty()>
                            <div class="command-empty">
                                No commands found
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </Show>
    }
}
