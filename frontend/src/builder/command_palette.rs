//! Command Palette Component
//!
//! VS Code-style command palette for quick access to all application commands.
//! Features fuzzy search, keyboard navigation, and command execution.

use crate::builder::keyboard::KeyboardAction;
use leptos::prelude::*;
use web_sys::wasm_bindgen::JsCast;

/// What a key press inside the palette should do.
///
/// Kept as a pure decision function so the navigation rules can be tested
/// without a DOM, while the component only maps the outcome onto signals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteKeyAction {
    /// The key is not a palette control.
    Ignore,
    /// Move the selection by this many rows, wrapping at both ends.
    MoveSelection(i32),
    /// Run the selected command and close the palette.
    ExecuteSelected,
    /// Close the palette without running anything.
    Close,
}

/// Decide what a key press does in the command palette.
pub fn palette_key_action(key: &str) -> PaletteKeyAction {
    match key {
        "ArrowUp" => PaletteKeyAction::MoveSelection(-1),
        "ArrowDown" => PaletteKeyAction::MoveSelection(1),
        "Enter" => PaletteKeyAction::ExecuteSelected,
        "Escape" => PaletteKeyAction::Close,
        _ => PaletteKeyAction::Ignore,
    }
}

/// Move a list selection by `delta`, wrapping around both ends.
///
/// Returns `0` for an empty list so callers never index past the end.
pub fn step_selection(current: usize, delta: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let len_i = len as i32;
    let current = (current as i32).clamp(0, len_i - 1);
    (current + delta).rem_euclid(len_i) as usize
}

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

/// DOM id shared by the palette's listbox and its search input's
/// `aria-controls`, so assistive technology can associate the two.
const LISTBOX_ID: &str = "command-palette-listbox";

/// DOM id of the option rendered for a given command.
fn option_id(command: &Command) -> String {
    format!("command-palette-option-{}", command.id)
}

/// VS Code-style Command Palette Component
///
/// A modal overlay that provides quick access to all application commands
/// through fuzzy search and keyboard navigation.
///
/// # Focus management
///
/// The palette is opened from anywhere (e.g. `Ctrl+K` while the canvas holds
/// focus), so it moves focus into the search input itself and returns focus to
/// whatever was focused before when it closes. Without that, keydown events
/// would keep targeting the element behind the overlay and none of the palette's
/// own controls would fire. Focus is kept inside the dialog while it is open.
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

    let search_input_ref = NodeRef::<leptos::html::Input>::new();

    // The element that held focus before the palette opened. Focus is handed
    // back to it on close so the user resumes where they were.
    let previously_focused =
        StoredValue::<Option<web_sys::HtmlElement>, LocalStorage>::new_local(None);

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

    // Move focus into the palette as it opens and restore it as it closes.
    // The move happens on the next animation frame so the modal subtree has
    // been rendered and the input's node ref is populated.
    Effect::new(move |was_open: Option<bool>| {
        let open = is_open.get();
        if was_open != Some(open) {
            if open {
                previously_focused.update_value(|slot| {
                    *slot = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.active_element())
                        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
                });
                search.set(String::new());
                let input = search_input_ref;
                request_animation_frame(move || {
                    if let Some(input) = input.get() {
                        let _ = input.focus();
                    }
                });
            } else {
                previously_focused.update_value(|slot| {
                    if let Some(element) = slot.take() {
                        let _ = element.focus();
                    }
                });
            }
        }
        open
    });

    // Keep Tab inside the dialog: the search input is the only tab stop, so
    // focus can never wander onto the canvas behind the overlay.
    let trap_tab = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Tab" {
            ev.prevent_default();
            if let Some(input) = search_input_ref.get() {
                let _ = input.focus();
            }
        }
    };

    // Clone on_action for use in multiple closures
    let on_action_clone = on_action.clone();

    let run_selected = {
        let on_action_run = on_action_clone.clone();
        move || {
            let commands = filtered_commands.get();
            if let Some(command) = commands.get(selected_index.get()) {
                on_action_run.clone()(command.action.clone());
                close.set(false);
            }
        }
    };

    view! {
        <Show when=move || is_open.get()>
            <div
                class="command-palette-backdrop"
                on:click=move |ev: web_sys::MouseEvent| {
                    // Only a click on the backdrop itself dismisses the palette;
                    // clicks that started inside the dialog must not close it.
                    if ev.target() == ev.current_target() {
                        close.set(false);
                    }
                }
            >
                <div
                    class="command-palette"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="command-palette-title"
                    on:click=move |ev| ev.stop_propagation()
                    on:keydown={
                        let run_selected = run_selected.clone();
                        move |ev: web_sys::KeyboardEvent| {
                            match palette_key_action(&ev.key()) {
                                PaletteKeyAction::Ignore => trap_tab(ev),
                                PaletteKeyAction::MoveSelection(delta) => {
                                    ev.prevent_default();
                                    set_selected_index.update(|idx| {
                                        *idx = step_selection(
                                            *idx,
                                            delta,
                                            filtered_commands.get().len(),
                                        );
                                    });
                                }
                                PaletteKeyAction::ExecuteSelected => {
                                    ev.prevent_default();
                                    run_selected();
                                }
                                PaletteKeyAction::Close => {
                                    ev.prevent_default();
                                    close.set(false);
                                }
                            }
                        }
                    }
                >
                    <h2 id="command-palette-title" class="command-palette-title">"Command Palette"</h2>
                    <div class="command-palette-search">
                        <input
                            type="text"
                            node_ref=search_input_ref
                            role="combobox"
                            aria-expanded="true"
                            aria-controls=LISTBOX_ID
                            aria-autocomplete="list"
                            aria-activedescendant=move || {
                                filtered_commands
                                    .get()
                                    .get(selected_index.get())
                                    .map(option_id)
                            }
                            aria-label="Search commands"
                            placeholder="Search commands..."
                            prop:value=move || search.get()
                            on:input=move |ev| {
                                search.set(event_target_value(&ev));
                            }
                        />
                    </div>

                    <div id=LISTBOX_ID class="command-palette-results" role="listbox" aria-label="Commands">
                        <For
                            each=move || filtered_commands.get().into_iter().enumerate()
                            key=|(idx, cmd)| format!("{}-{}", idx, cmd.id)
                            children={
                                let on_action_for = on_action_clone.clone();
                                move |(idx, command): (usize, Command)| {
                                let is_selected = move || selected_index.get() == idx;
                                let command_clone = command.clone();
                                let on_action_click = on_action_for.clone();
                                let item_id = option_id(&command);

                                view! {
                                    <div
                                        class="command-item"
                                        id=item_id
                                        role="option"
                                        aria-selected=move || if is_selected() { "true" } else { "false" }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_keys_map_to_their_actions() {
        assert_eq!(
            palette_key_action("ArrowUp"),
            PaletteKeyAction::MoveSelection(-1)
        );
        assert_eq!(
            palette_key_action("ArrowDown"),
            PaletteKeyAction::MoveSelection(1)
        );
        assert_eq!(
            palette_key_action("Enter"),
            PaletteKeyAction::ExecuteSelected
        );
        assert_eq!(palette_key_action("Escape"), PaletteKeyAction::Close);
        // Typing and Tab are not palette controls; the component keeps handling
        // them itself (typing reaches the input, Tab is trapped).
        for key in ["a", "Tab", "Shift", " ", "ArrowLeft"] {
            assert_eq!(palette_key_action(key), PaletteKeyAction::Ignore, "{key}");
        }
    }

    #[test]
    fn selection_walks_the_list_and_wraps() {
        // Both ends wrap, so ArrowUp from the top lands on the last command.
        assert_eq!(step_selection(0, 1, 3), 1);
        assert_eq!(step_selection(2, 1, 3), 0);
        assert_eq!(step_selection(0, -1, 3), 2);
        // An empty list has nothing to select and must never index past the end.
        assert_eq!(step_selection(0, 1, 0), 0);
        assert_eq!(step_selection(7, -1, 0), 0);
        // A stale index (e.g. after filtering narrowed the list) is clamped to
        // the last row before stepping, so it never indexes past the end.
        assert_eq!(step_selection(9, -1, 3), 1);
        assert_eq!(step_selection(9, 1, 3), 0);
    }
}

/// Browser-only regression tests for the palette's focus and key handling.
///
/// These mount the real component, so they exercise DOM focus and dispatched
/// keyboard events rather than only the boolean helpers above.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::builder::keyboard::{editor_modal_open, should_dispatch_shortcut};
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Wait for pending effects and the palette's focus `requestAnimationFrame`
    /// callback to run.
    async fn settle() {
        for _ in 0..4 {
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    }

    fn dispatch_key(target: &web_sys::HtmlElement, key: &str) {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        init.set_bubbles(true);
        let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
            .expect("keydown event");
        let _ = target.dispatch_event(&event);
    }

    fn type_query(input: &web_sys::HtmlInputElement, value: &str) {
        input.set_value(value);
        let event = web_sys::Event::new("input").expect("input event");
        let _ = input.dispatch_event(&event);
    }

    /// The element that currently holds focus.
    fn active_element() -> Option<web_sys::Element> {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.active_element())
    }

    /// Whether `target` is the element that currently holds focus.
    fn is_focused(target: &web_sys::Element) -> bool {
        active_element().as_ref() == Some(target)
    }

    /// The signals and DOM handles a mounted palette test needs.
    struct Harness {
        host: web_sys::HtmlElement,
        is_open: RwSignal<bool>,
        search: RwSignal<String>,
        actions: RwSignal<Vec<KeyboardAction>>,
    }

    impl Harness {
        fn input(&self) -> web_sys::HtmlInputElement {
            self.host
                .query_selector("input[role='combobox']")
                .unwrap()
                .expect("palette must render a search input while open")
                .dyn_into()
                .unwrap()
        }

        fn dialog(&self) -> web_sys::HtmlElement {
            self.host
                .query_selector("[role='dialog']")
                .unwrap()
                .expect("palette must render a dialog while open")
                .dyn_into()
                .unwrap()
        }

        fn options(&self) -> Vec<web_sys::HtmlElement> {
            let options = self.host.query_selector_all("[role='option']").unwrap();
            (0..options.length())
                .filter_map(|i| options.item(i))
                .filter_map(|n| n.dyn_into().ok())
                .collect()
        }

        /// The command the palette announces as selected.
        fn active_descendant(&self) -> Option<String> {
            let value = self.input().get_attribute("aria-activedescendant")?;
            (!value.is_empty()).then_some(value)
        }

        fn is_selected(&self, id: &str) -> bool {
            self.host
                .query_selector(&format!("[role='option'][id='{id}']"))
                .ok()
                .flatten()
                .and_then(|el| el.get_attribute("aria-selected"))
                .as_deref()
                == Some("true")
        }
    }

    /// Mount the real `CommandPalette` and run `body` against it, unmounting
    /// afterwards so tests cannot leak a focused dialog into each other.
    async fn with_palette<F, Fut>(body: F)
    where
        F: FnOnce(Harness) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let document = web_sys::window().unwrap().document().unwrap();
        let host = document
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        document.body().unwrap().append_child(&host).unwrap();

        // A focusable sentinel outside the palette gives focus restoration
        // something real to return to.
        let sentinel = document
            .create_element("button")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        sentinel.set_id("palette-focus-sentinel");
        host.append_child(&sentinel).unwrap();
        let _ = sentinel.focus();

        let harness = Harness {
            host,
            is_open: RwSignal::new(false),
            search: RwSignal::new(String::new()),
            actions: RwSignal::new(Vec::new()),
        };
        let (is_open, search, actions) = (harness.is_open, harness.search, harness.actions);

        let unmount = leptos::mount::mount_to(harness.host.clone(), move || {
            view! {
                <CommandPalette
                    is_open=is_open.read_only()
                    close=is_open.write_only()
                    search=search
                    on_action=move |action: KeyboardAction| {
                        actions.update(|list| list.push(action));
                    }
                />
            }
        });

        body(harness).await;
        drop(unmount);
    }

    /// Opening the palette must move focus into the search input. Without this,
    /// keydown keeps targeting the element behind the overlay and the palette's
    /// own controls never fire.
    #[wasm_bindgen_test]
    async fn opening_moves_focus_into_the_search_input() {
        with_palette(|h| async move {
            h.is_open.set(true);
            settle().await;

            let input: web_sys::Element = h.input().unchecked_into();
            assert!(
                is_focused(&input),
                "focus must land on the palette's search input"
            );
        })
        .await;
    }

    /// The dialog must carry the standard modal semantics and a label.
    #[wasm_bindgen_test]
    async fn dialog_exposes_modal_semantics() {
        with_palette(|h| async move {
            h.is_open.set(true);
            settle().await;

            let dialog = h.dialog();
            assert_eq!(dialog.get_attribute("role").as_deref(), Some("dialog"));
            assert_eq!(dialog.get_attribute("aria-modal").as_deref(), Some("true"));
            assert_eq!(
                dialog.get_attribute("aria-labelledby").as_deref(),
                Some("command-palette-title")
            );

            let input = h.input();
            assert_eq!(input.get_attribute("role").as_deref(), Some("combobox"));
            assert_eq!(
                input.get_attribute("aria-expanded").as_deref(),
                Some("true")
            );
            assert_eq!(
                input.get_attribute("aria-autocomplete").as_deref(),
                Some("list")
            );
            assert_eq!(
                input.get_attribute("aria-controls").as_deref(),
                Some(LISTBOX_ID)
            );

            // Options are announced as selected through ARIA, not only CSS.
            let id = h
                .active_descendant()
                .expect("a command is selected on open");
            assert!(h.is_selected(&id));
            assert_eq!(
                h.host
                    .query_selector(&format!("#{LISTBOX_ID}"))
                    .unwrap()
                    .expect("the listbox must exist")
                    .get_attribute("role")
                    .as_deref(),
                Some("listbox")
            );
        })
        .await;
    }

    /// ArrowDown/ArrowUp must move the selection.
    #[wasm_bindgen_test]
    async fn arrow_keys_move_the_selection() {
        with_palette(|h| async move {
            h.is_open.set(true);
            settle().await;

            let input = h.input();
            let first = h
                .active_descendant()
                .expect("a command is selected on open");
            assert!(h.is_selected(&first));

            dispatch_key(&input, "ArrowDown");
            settle().await;
            let second = h.active_descendant().expect("selection survives ArrowDown");
            assert_ne!(first, second, "ArrowDown must select a different command");
            assert!(h.is_selected(&second));

            dispatch_key(&input, "ArrowUp");
            settle().await;
            assert_eq!(
                h.active_descendant().as_deref(),
                Some(first.as_str()),
                "ArrowUp must return to the previous command"
            );
        })
        .await;
    }

    /// Enter must run the selected command and close the palette.
    #[wasm_bindgen_test]
    async fn enter_executes_the_selected_command_and_closes() {
        with_palette(|h| async move {
            h.is_open.set(true);
            settle().await;

            let input = h.input();
            dispatch_key(&input, "ArrowDown");
            settle().await;
            let selected = h.active_descendant().expect("a command is selected");

            dispatch_key(&input, "Enter");
            settle().await;

            assert_eq!(
                h.actions.get(),
                vec![KeyboardAction::Redo],
                "Enter must run exactly the selected command"
            );
            assert!(!h.is_open.get(), "Enter must close the palette");
            // ArrowDown from `undo` lands on `redo`, whose id is stable.
            assert_eq!(
                selected,
                option_id(&get_commands()[1]),
                "the announced option must be the command that ran"
            );
        })
        .await;
    }

    /// Escape must close the palette without running anything.
    #[wasm_bindgen_test]
    async fn escape_closes_without_running_a_command() {
        with_palette(|h| async move {
            h.is_open.set(true);
            settle().await;

            let input = h.input();
            dispatch_key(&input, "Escape");
            settle().await;

            assert!(!h.is_open.get(), "Escape must close the palette");
            assert!(h.actions.get().is_empty(), "Escape must not run a command");
        })
        .await;
    }

    /// Typing must narrow the list, and reopening must start from a clean query.
    #[wasm_bindgen_test]
    async fn typing_filters_the_command_list() {
        with_palette(|h| async move {
            h.is_open.set(true);
            settle().await;

            let all = h.options().len();
            assert!(all > 1, "the full command list must render");

            type_query(&h.input(), "undo");
            settle().await;

            let filtered = h.options();
            assert!(
                !filtered.is_empty(),
                "searching 'undo' must match something"
            );
            assert!(filtered.len() < all, "searching must narrow the list");

            h.is_open.set(false);
            settle().await;
            h.is_open.set(true);
            settle().await;
            assert_eq!(h.search.get(), "", "reopening clears the query");
            assert_eq!(h.options().len(), all, "reopening restores the full list");
        })
        .await;
    }

    /// Closing must hand focus back to whatever held it before.
    #[wasm_bindgen_test]
    async fn closing_restores_the_previous_focus() {
        with_palette(|h| async move {
            let sentinel = h
                .host
                .query_selector("#palette-focus-sentinel")
                .unwrap()
                .unwrap();

            h.is_open.set(true);
            settle().await;
            let input: web_sys::Element = h.input().unchecked_into();
            assert!(is_focused(&input), "focus starts inside the palette");

            h.is_open.set(false);
            settle().await;
            assert!(
                is_focused(&sentinel),
                "focus must return to the element focused before opening"
            );
        })
        .await;
    }

    /// Tab must not move focus out of the dialog.
    #[wasm_bindgen_test]
    async fn tab_is_trapped_inside_the_dialog() {
        with_palette(|h| async move {
            h.is_open.set(true);
            settle().await;

            let input: web_sys::Element = h.input().unchecked_into();
            dispatch_key(&h.input(), "Tab");
            settle().await;

            assert!(
                is_focused(&input),
                "Tab must keep focus on the palette's search input"
            );
        })
        .await;
    }

    /// Clicks inside the dialog must not dismiss it, while the backdrop still
    /// does. Clicking an option is a third case: it runs the command.
    #[wasm_bindgen_test]
    async fn backdrop_dismisses_but_dialog_content_does_not() {
        with_palette(|h| async move {
            h.is_open.set(true);
            settle().await;

            let click_on = |target: &web_sys::HtmlElement| {
                let init = web_sys::MouseEventInit::new();
                init.set_bubbles(true);
                let click = web_sys::MouseEvent::new_with_mouse_event_init_dict("click", &init)
                    .expect("click event");
                let _ = target.dispatch_event(&click);
            };

            // The dialog itself is not an option: clicking it must not dismiss.
            click_on(&h.dialog());
            settle().await;
            assert!(
                h.is_open.get(),
                "a click on the dialog must not dismiss the palette"
            );

            // An option click is an activation: it runs and closes.
            let option = h.options().into_iter().next().expect("at least one option");
            click_on(&option);
            settle().await;
            assert_eq!(
                h.actions.get().len(),
                1,
                "clicking an option must run its command"
            );
            assert!(
                !h.is_open.get(),
                "clicking an option must close the palette"
            );

            // The backdrop does dismiss.
            h.is_open.set(true);
            settle().await;
            let backdrop = h
                .host
                .query_selector(".command-palette-backdrop")
                .unwrap()
                .expect("the backdrop must exist")
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap();
            click_on(&backdrop);
            settle().await;
            assert!(
                !h.is_open.get(),
                "clicking the backdrop must dismiss the palette"
            );
        })
        .await;
    }

    /// The canvas shortcuts stay blocked while the palette is open and work
    /// again once it closes. This drives the same derivation `EditorPage` uses.
    #[wasm_bindgen_test]
    async fn canvas_shortcuts_are_blocked_while_open_and_restored_after_close() {
        with_palette(|h| async move {
            let modal_open = Signal::derive(move || {
                editor_modal_open(h.is_open.get(), false, false, false, false, false)
            });

            assert!(
                should_dispatch_shortcut(modal_open.get_untracked(), false),
                "shortcuts start enabled"
            );

            h.is_open.set(true);
            settle().await;
            assert!(
                !should_dispatch_shortcut(modal_open.get_untracked(), false),
                "opening the palette must block Delete/Ctrl+Z/Ctrl+A on the canvas"
            );

            h.is_open.set(false);
            settle().await;
            assert!(
                should_dispatch_shortcut(modal_open.get_untracked(), false),
                "closing the palette must restore the canvas shortcuts"
            );
        })
        .await;
    }
}
