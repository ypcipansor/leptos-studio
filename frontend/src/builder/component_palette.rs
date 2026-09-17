//! Component Palette
//!
//! Enhanced component palette with fuzzy search, categorization,
//! and drag-drop support. Provides quick access to all available
//! components with search and filter capabilities.

use leptos::prelude::*;

use crate::builder::component_library::{
    LibraryComponent, create_canvas_component_from_payload, palette_drag_payload,
};
use crate::builder::drag_drop::{DragDropConfig, create_drag_handlers};
use crate::state::AppState;

/// Component categories for filtering.
///
/// Each variant except `All` and `Custom` maps to the `category` string a
/// `LibraryComponent` declares, so a component is always reachable from the
/// tab that names its category.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ComponentCategory {
    All,
    Basic,
    Form,
    Layout,
    Media,
    Typography,
    Navigation,
    Custom,
}

/// Every category tab, in display order.
pub const CATEGORIES: [ComponentCategory; 8] = [
    ComponentCategory::All,
    ComponentCategory::Basic,
    ComponentCategory::Form,
    ComponentCategory::Layout,
    ComponentCategory::Media,
    ComponentCategory::Typography,
    ComponentCategory::Navigation,
    ComponentCategory::Custom,
];

impl ComponentCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ComponentCategory::All => "All",
            ComponentCategory::Basic => "Basic",
            ComponentCategory::Form => "Form",
            ComponentCategory::Layout => "Layout",
            ComponentCategory::Media => "Media",
            ComponentCategory::Typography => "Typography",
            ComponentCategory::Navigation => "Navigation",
            ComponentCategory::Custom => "Custom",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ComponentCategory::All => "📦",
            ComponentCategory::Basic => "🔷",
            ComponentCategory::Form => "📝",
            ComponentCategory::Layout => "📐",
            ComponentCategory::Media => "🖼️",
            ComponentCategory::Typography => "🔤",
            ComponentCategory::Navigation => "🧭",
            ComponentCategory::Custom => "⚙️",
        }
    }

    pub fn matches(&self, component: &LibraryComponent) -> bool {
        match self {
            ComponentCategory::All => true,
            ComponentCategory::Custom => component.category == "Custom",
            other => component.category == other.label(),
        }
    }
}

/// Whether a key press on a palette row should add the component to the canvas.
///
/// The rows are focusable and exposed as `option`s, so Enter and Space must do
/// something; every other key (including arrows, which the grid handles) is left
/// alone. Kept as a named predicate so the accessibility behaviour is testable.
pub fn is_palette_activation_key(key: &str) -> bool {
    matches!(key, "Enter" | " " | "Spacebar")
}

/// Fuzzy search score
fn fuzzy_score(text: &str, query: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    // Exact match gets highest score
    if text_lower == query_lower {
        return Some(1000);
    }

    // Starts with gets high score
    if text_lower.starts_with(&query_lower) {
        return Some(500);
    }

    // Contains gets medium score
    if text_lower.contains(&query_lower) {
        return Some(100);
    }

    // Fuzzy character match
    let text_chars: Vec<char> = text_lower.chars().collect();
    let query_chars: Vec<char> = query_lower.chars().collect();

    let mut query_idx = 0;
    let mut score = 0;
    let mut consecutive = 0;

    for (i, &tc) in text_chars.iter().enumerate() {
        if query_idx < query_chars.len() && tc == query_chars[query_idx] {
            query_idx += 1;
            consecutive += 1;
            // Bonus for consecutive matches
            score += consecutive * 10;
            // Bonus for matching at word start
            if i == 0 || text_chars.get(i.saturating_sub(1)) == Some(&' ') {
                score += 20;
            }
        } else {
            consecutive = 0;
        }
    }

    if query_idx == query_chars.len() {
        Some(score)
    } else {
        None
    }
}

/// Component palette with search and filter
#[component]
pub fn ComponentPalette() -> impl IntoView {
    let app_state = AppState::expect_context();

    // Local state
    let search_query = RwSignal::new(String::new());
    let selected_category = RwSignal::new(ComponentCategory::All);
    let is_expanded = RwSignal::new(true);

    // Filtered and sorted components
    let filtered_components = Memo::new(move |_| {
        let query = search_query.get();
        let category = selected_category.get();
        let library = app_state.ui.component_library.get();

        let mut components: Vec<(LibraryComponent, i32)> = library
            .into_iter()
            .filter(|comp| category.matches(comp))
            .filter_map(|comp| {
                // Score by name and description
                let name_score = fuzzy_score(&comp.name, &query);
                let desc_score = comp
                    .description
                    .as_ref()
                    .and_then(|d| fuzzy_score(d, &query))
                    .unwrap_or(0);

                let total_score = name_score.map(|s| s + desc_score / 2);
                total_score.map(|score| (comp, score))
            })
            .collect();

        // Sort by score (descending)
        components.sort_by_key(|b| std::cmp::Reverse(b.1));

        components.into_iter().map(|(c, _)| c).collect::<Vec<_>>()
    });

    // Category counts. Derived from `matches` so a badge always equals the
    // number of rows the filter will actually show.
    let category_counts = Memo::new(move |_| {
        let library = app_state.ui.component_library.get();
        let mut counts = std::collections::HashMap::new();

        for cat in CATEGORIES {
            let n = library.iter().filter(|comp| cat.matches(comp)).count();
            counts.insert(cat, n);
        }

        counts
    });

    view! {
        <div class="component-palette">
            <div class="palette-header">
                <h3 class="palette-title">
                    <button
                        class="palette-toggle"
                        on:click=move |_| is_expanded.update(|v| *v = !*v)
                        aria-expanded=move || is_expanded.get().to_string()
                    >
                        {move || if is_expanded.get() { "▼" } else { "▶" }}
                    </button>
                    "Components"
                </h3>
                <span class="palette-count">
                    {move || filtered_components.get().len()}
                </span>
            </div>

            {move || {
                if is_expanded.get() {
                    view! {
                        <div class="palette-content">
                            // Search input
                            <div class="palette-search">
                                <span class="search-icon">"🔍"</span>
                                <input
                                    type="text"
                                    class="palette-search-input"
                                    placeholder="Search components..."
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| search_query.set(event_target_value(&ev))
                                    aria-label="Search components"
                                />
                                {move || {
                                    if !search_query.get().is_empty() {
                                        view! {
                                            <button
                                                class="search-clear"
                                                on:click=move |_| search_query.set(String::new())
                                                aria-label="Clear search"
                                            >
                                                "×"
                                            </button>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }
                                }}
                            </div>

                            // Category tabs
                            <div class="palette-categories" role="tablist">
                                {CATEGORIES.into_iter().map(|cat| {
                                    let cat_for_click = cat.clone();
                                    let cat_for_class = cat.clone();
                                    let cat_for_count = cat.clone();
                                    let count = category_counts.get().get(&cat_for_count).copied().unwrap_or(0);

                                    view! {
                                        <button
                                            class=move || {
                                                if selected_category.get() == cat_for_class {
                                                    "category-tab active"
                                                } else {
                                                    "category-tab"
                                                }
                                            }
                                            on:click=move |_| selected_category.set(cat_for_click.clone())
                                            role="tab"
                                            aria-selected=move || (selected_category.get() == cat).to_string()
                                        >
                                            <span class="category-icon">{cat.icon()}</span>
                                            <span class="category-label">{cat.label()}</span>
                                            <span class="category-count">{count}</span>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>

                            // Component grid
                            <div class="palette-grid" role="listbox">
                                <For
                                    each=move || filtered_components.get()
                                    key=|comp| comp.id.clone()
                                    children=move |comp| {
                                        let comp_kind = palette_drag_payload(&comp);
                                        let comp_name = comp.name.clone();
                                        let comp_label = comp_name.clone();
                                        let comp_desc = comp.description.clone().unwrap_or_default();
                                        let comp_category = comp.category.clone();

                                        let (on_drag_start, on_drag, on_drag_end) = create_drag_handlers(
                                            comp_kind.clone(),
                                            app_state.canvas.drag_state,
                                            DragDropConfig::default(),
                                        );

                                        // Keyboard equivalent of dropping the row on the canvas.
                                        // The palette rows are focusable and exposed as options, so
                                        // Enter/Space must actually add the component: it goes to the
                                        // root with a snapshot (undoable) and becomes the selection.
                                        let on_keydown = {
                                            let payload = comp_kind.clone();
                                            move |ev: leptos::ev::KeyboardEvent| {
                                                if !is_palette_activation_key(&ev.key()) {
                                                    return;
                                                }
                                                ev.prevent_default();
                                                let Some(component) =
                                                    create_canvas_component_from_payload(
                                                        &payload,
                                                        &app_state
                                                            .ui
                                                            .component_library
                                                            .get_untracked(),
                                                    )
                                                else {
                                                    return;
                                                };
                                                let id = *component.id();
                                                app_state.canvas.add_component(component);
                                                app_state.canvas.select_single(id);
                                            }
                                        };

                                        view! {
                                            <div
                                                class="palette-item"
                                                draggable="true"
                                                on:dragstart=on_drag_start
                                                on:drag=on_drag
                                                on:dragend=on_drag_end
                                                on:keydown=on_keydown
                                                role="option"
                                                tabindex="0"
                                                aria-label=format!("Add {}", comp_label)
                                            >
                                                <div class="palette-item-icon">
                                                    {component_icon(&comp.kind)}
                                                </div>
                                                <div class="palette-item-content">
                                                    <span class="palette-item-name">{comp_name}</span>
                                                    {if !comp_desc.is_empty() {
                                                        view! {
                                                            <span class="palette-item-desc">{comp_desc}</span>
                                                        }.into_any()
                                                    } else {
                                                        ().into_any()
                                                    }}
                                                </div>
                                                <span class="palette-item-category">{comp_category}</span>
                                            </div>
                                        }
                                    }
                                />

                                {move || {
                                    if filtered_components.get().is_empty() {
                                        view! {
                                            <div class="palette-empty">
                                                <p>"No components found"</p>
                                                <p class="palette-empty-hint">
                                                    {move || if !search_query.get().is_empty() {
                                                        "Try a different search term"
                                                    } else {
                                                        "No components in this category"
                                                    }}
                                                </p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }
                                }}
                            </div>
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}

/// Get icon for component type
fn component_icon(kind: &str) -> &'static str {
    match kind {
        "Button" => "🔘",
        "Text" => "📝",
        "Input" => "📥",
        "Container" => "📦",
        "Custom" => "⚙️",
        "Link" => "🔗",
        _ => "❓",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CanvasComponent, ComponentType};

    #[test]
    fn test_fuzzy_score_exact_match() {
        let score = fuzzy_score("Button", "Button");
        assert!(score.is_some());
        assert_eq!(score.unwrap(), 1000);
    }

    #[test]
    fn test_fuzzy_score_starts_with() {
        let score = fuzzy_score("Button", "But");
        assert!(score.is_some());
        assert_eq!(score.unwrap(), 500);
    }

    #[test]
    fn test_fuzzy_score_contains() {
        let score = fuzzy_score("Submit Button", "Button");
        assert!(score.is_some());
        assert_eq!(score.unwrap(), 100);
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        let score = fuzzy_score("Button", "xyz");
        assert!(score.is_none());
    }

    #[test]
    fn test_fuzzy_score_empty_query() {
        let score = fuzzy_score("Button", "");
        assert_eq!(score, Some(0));
    }

    #[test]
    fn test_category_matches() {
        let basic_comp = LibraryComponent {
            id: "test-basic".to_string(),
            name: "Button".to_string(),
            kind: "Button".to_string(),
            template: None,
            category: "Basic".to_string(),
            props_schema: None,
            description: None,
        };

        assert!(ComponentCategory::All.matches(&basic_comp));
        assert!(ComponentCategory::Basic.matches(&basic_comp));
        assert!(!ComponentCategory::Layout.matches(&basic_comp));
    }

    /// The palette rows are focusable `option`s, so they must be activatable
    /// from the keyboard, not drag-only.
    #[test]
    fn palette_rows_are_keyboard_activatable() {
        assert!(is_palette_activation_key("Enter"));
        assert!(is_palette_activation_key(" "));
        assert!(is_palette_activation_key("Spacebar"));
        assert!(!is_palette_activation_key("ArrowDown"));
        assert!(!is_palette_activation_key("a"));
        assert!(!is_palette_activation_key("Tab"));
    }

    /// The live default library the editor actually installs into
    /// `UiState.component_library`. Tests must exercise this, not
    /// `builtin_library_components()`, which omits the Div/Heading/Link entries
    /// `UiState` adds — otherwise a test could pass while the live library
    /// disagrees with it.
    fn live_library() -> Vec<LibraryComponent> {
        crate::builder::component_library::default_library_components()
    }

    /// Collect every id in a component tree, so a nested container's children are
    /// covered too.
    fn collect_ids(component: &CanvasComponent, out: &mut Vec<String>) {
        out.push(component.id().to_string());
        if let CanvasComponent::Container(c) = component {
            for child in &c.children {
                collect_ids(child, out);
            }
        }
        if let CanvasComponent::Card(c) = component {
            for child in &c.children {
                collect_ids(child, out);
            }
        }
    }

    /// True when two trees have any id in common. Two drops of one palette entry
    /// must never share an id, at any depth.
    fn shares_any_id(a: &CanvasComponent, b: &CanvasComponent) -> bool {
        let (mut ids_a, mut ids_b) = (Vec::new(), Vec::new());
        collect_ids(a, &mut ids_a);
        collect_ids(b, &mut ids_b);
        ids_a.iter().any(|id| ids_b.contains(id))
    }

    /// Drop every `id` field so two trees can be compared on design alone.
    fn strip_ids(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("id");
                for (_, child) in map.iter_mut() {
                    strip_ids(child);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items.iter_mut() {
                    strip_ids(item);
                }
            }
            _ => {}
        }
    }

    /// Every entry in the *live* library must be reachable from exactly one
    /// non-`All` tab, resolvable to a canvas component, and the per-category
    /// counts must partition the library.
    #[test]
    fn test_every_component_is_reachable_by_category() {
        let library = live_library();

        for comp in &library {
            assert!(!comp.id.is_empty(), "{} has no id", comp.name);

            let reachable: Vec<_> = CATEGORIES
                .iter()
                .filter(|c| **c != ComponentCategory::All && c.matches(comp))
                .collect();
            assert_eq!(
                reachable.len(),
                1,
                "{} (category {}) must be reachable from exactly one non-All tab, got {}",
                comp.name,
                comp.category,
                reachable.len()
            );

            let payload = palette_drag_payload(comp);
            let resolved = create_canvas_component_from_payload(&payload, &library);
            let resolved = resolved.unwrap_or_else(|| {
                panic!(
                    "{} ({}) has payload `{}` that resolves to no component",
                    comp.name, comp.category, payload
                )
            });
            assert!(
                !matches!(resolved.component_type(), ComponentType::Custom),
                "{}: a built-in entry must not resolve as a custom component",
                comp.name
            );

            // Dropping the same entry twice must mint fresh ids every time,
            // recursively, or the second drop would share an id with the first.
            let again = create_canvas_component_from_payload(&payload, &library)
                .unwrap_or_else(|| panic!("{}: payload must resolve twice", comp.name));
            assert!(
                !shares_any_id(&resolved, &again),
                "{}: resolving the same payload twice must regenerate every id",
                comp.name
            );

            // An entry carrying a template must hand back exactly that design,
            // not a default built from its kind. Heading/Link depend on this:
            // they share `kind: "Text"` with plain Text.
            if let Some(template) = comp.template.as_deref() {
                let mut expected: serde_json::Value = serde_json::from_str(template)
                    .unwrap_or_else(|e| panic!("{}: template must be JSON: {e}", comp.name));
                let mut actual = serde_json::to_value(&resolved).unwrap();
                strip_ids(&mut expected);
                strip_ids(&mut actual);
                assert_eq!(
                    expected, actual,
                    "{}: dropping the entry must restore its stored design",
                    comp.name
                );
            }
        }

        // Entries sharing a `kind` are the ones a kind-based payload would
        // collapse, so each must carry its own distinct design.
        let mut by_kind: std::collections::HashMap<&str, Vec<&LibraryComponent>> =
            std::collections::HashMap::new();
        for comp in &library {
            by_kind.entry(comp.kind.as_str()).or_default().push(comp);
        }
        for (kind, entries) in &by_kind {
            for (i, a) in entries.iter().enumerate() {
                for b in &entries[i + 1..] {
                    let design = |c: &LibraryComponent| {
                        let mut v: serde_json::Value =
                            serde_json::from_str(c.template.as_deref().unwrap_or("null"))
                                .unwrap_or(serde_json::Value::Null);
                        strip_ids(&mut v);
                        v
                    };
                    assert_ne!(
                        design(a),
                        design(b),
                        "{} and {} share kind `{}`; their designs must differ so the \
                         palette can distinguish them",
                        a.name,
                        b.name,
                        kind
                    );
                }
            }
        }

        // Ids are unique.
        let mut ids: Vec<&str> = library.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        let unique = ids.len();
        ids.dedup();
        assert_eq!(unique, ids.len(), "library ids must be unique");

        // The badge count for a non-`All` category uses the same predicate the
        // row filter uses, and the categories partition the library.
        let per_category: usize = CATEGORIES
            .iter()
            .filter(|c| **c != ComponentCategory::All)
            .map(|cat| library.iter().filter(|c| cat.matches(c)).count())
            .sum();
        assert_eq!(
            per_category,
            library.len(),
            "each component must appear under exactly one category tab"
        );

        let all_count = library
            .iter()
            .filter(|c| ComponentCategory::All.matches(c))
            .count();
        assert_eq!(all_count, library.len(), "All must match the whole library");
    }

    /// Div, Heading and Link are added to the live library *after* the built-in
    /// list, so a reachability test limited to `builtin_library_components()`
    /// would never see them.
    #[test]
    fn live_library_covers_div_heading_and_link() {
        let library = live_library();
        for name in ["Div", "Heading", "Link"] {
            assert!(
                library.iter().any(|c| c.name == name),
                "the live library must contain {name}"
            );
        }
    }

    /// Heading and Link share a `kind` with plain Text but must resolve to
    /// different designs: a heading is a heading, a link is a hyperlink.
    #[test]
    fn heading_and_link_resolve_to_their_own_semantics() {
        let library = live_library();

        let by_name = |name: &str| {
            library
                .iter()
                .find(|c| c.name == name)
                .unwrap_or_else(|| panic!("{name} missing from the live library"))
        };

        let heading_component = create_canvas_component_from_payload(
            &palette_drag_payload(by_name("Heading")),
            &library,
        )
        .expect("Heading must resolve");
        let link_component =
            create_canvas_component_from_payload(&palette_drag_payload(by_name("Link")), &library)
                .expect("Link must resolve");

        let CanvasComponent::Text(heading) = &heading_component else {
            panic!(
                "Heading must not degrade to {:?}",
                heading_component.component_type()
            );
        };
        assert_eq!(
            heading.tag,
            crate::domain::TextTag::H1,
            "Heading must be semantic, not a default paragraph"
        );

        let CanvasComponent::Link(link) = &link_component else {
            panic!(
                "Link must not degrade to {:?}",
                link_component.component_type()
            );
        };
        assert!(
            !link.href.is_empty(),
            "a resolved Link must carry an href so it exports as a hyperlink"
        );

        assert_ne!(
            heading_component.component_type(),
            link_component.component_type(),
            "Heading and Link must resolve to different component types"
        );

        // Plain Text/Button built-ins must not regress.
        let text =
            create_canvas_component_from_payload(&palette_drag_payload(by_name("Text")), &library)
                .expect("Text must resolve");
        let CanvasComponent::Text(text) = &text else {
            panic!("Text must resolve to a Text component");
        };
        assert_eq!(
            text.tag,
            crate::domain::TextTag::P,
            "plain Text must stay a paragraph"
        );

        let button = create_canvas_component_from_payload(
            &palette_drag_payload(by_name("Button")),
            &library,
        )
        .expect("Button must resolve");
        assert!(matches!(button, CanvasComponent::Button(_)));
    }

    /// The keyboard activation path and the drag payload path must share one
    /// resolver, so Enter/Space produces the same component as a drop. The DOM
    /// behaviour is covered by `wasm_tests` below; this only pins that the
    /// payload each path starts from is the same value.
    #[test]
    fn keyboard_and_drag_paths_share_one_payload() {
        let library = live_library();
        for name in ["Heading", "Link", "Div", "Button"] {
            let entry = library
                .iter()
                .find(|c| c.name == name)
                .unwrap_or_else(|| panic!("{name} missing"));
            let payload = palette_drag_payload(entry);

            let resolved = create_canvas_component_from_payload(&payload, &library)
                .unwrap_or_else(|| panic!("{name}: payload `{payload}` must resolve"));
            // Both UI paths hand this one payload to the same resolver, so the
            // payload itself has to identify the design unambiguously.
            assert!(!payload.is_empty(), "{name}: the payload must not be empty");
            // A resolved entry must still be a real component, not a fallback.
            assert!(
                !matches!(resolved.component_type(), ComponentType::Custom),
                "{name}: built-ins must not resolve as custom components"
            );
        }
    }
}

/// Browser-only checks that the palette's *actual DOM* wires Enter/Space and
/// drag-and-drop to the shared resolver. Pure-function tests cannot catch a row
/// whose handler is missing or attached to the wrong event.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::builder::canvas::handle_drop;
    use crate::domain::{CanvasComponent, TextTag};
    use crate::state::{DerivedState, app_state::AppState};
    use wasm_bindgen::JsCast;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    /// Tests share one browser page, so a previous test's DOM and LocalStorage
    /// survive into the next. Stale rows would make a selector match the wrong
    /// palette. Only the hosts these tests create are removed: the page body
    /// also holds the wasm-bindgen-test harness, and clearing it wholesale stops
    /// the runner from reporting results.
    fn reset_environment() {
        if let Some(storage) = web_sys::window().unwrap().local_storage().ok().flatten() {
            storage.clear().ok();
        }
        let document = document();
        if let Ok(hosts) = document.query_selector_all("[data-palette-test-host]") {
            for i in 0..hosts.length() {
                if let Some(node) = hosts.item(i)
                    && let Some(parent) = node.parent_node()
                {
                    parent.remove_child(&node).ok();
                }
            }
        }
    }

    /// Append a host div the next `reset_environment` will clean up.
    fn test_host(document: &web_sys::Document) -> web_sys::HtmlElement {
        let host = document
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        host.set_attribute("data-palette-test-host", "").unwrap();
        document.body().unwrap().append_child(&host).unwrap();
        host
    }

    /// The one component the activation added.
    ///
    /// `AppState::new` loads the most recent project from the backend, so the
    /// canvas is not empty when a test starts and its size is not the test's
    /// business. Comparing against a snapshot taken just before the interaction
    /// keeps the assertion about the palette and nothing else.
    fn newly_added(before: &[CanvasComponent], after: &[CanvasComponent]) -> CanvasComponent {
        let before_ids: Vec<String> = before.iter().map(|c| c.id().to_string()).collect();
        let added: Vec<&CanvasComponent> = after
            .iter()
            .filter(|c| !before_ids.contains(&c.id().to_string()))
            .collect();
        assert_eq!(
            added.len(),
            1,
            "the interaction must add exactly one component, added {}",
            added.len()
        );
        added[0].clone()
    }

    /// A `keydown` that bubbles, so the row's own handler sees it.
    fn keydown(key: &str) -> web_sys::KeyboardEvent {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key(key);
        init.set_bubbles(true);
        web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap()
    }

    async fn settle() {
        for _ in 0..6 {
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    }

    /// Mount the palette with a live `AppState` and return the canvas state plus
    /// the unmount handle. The providers must be installed *inside* `mount_to`,
    /// which is what creates the executor and the `Owner` effects need. The state
    /// travels back through an `Rc<Cell>` because `AppState` is `Copy`: a plain
    /// `let mut state = None` captured by the `move` closure would mutate the
    /// closure's copy and leave the caller's `None`.
    async fn mount_palette() -> (AppState, impl Sized) {
        reset_environment();
        let document = document();
        let host = test_host(&document);

        let slot: std::rc::Rc<std::cell::Cell<Option<AppState>>> =
            std::rc::Rc::new(std::cell::Cell::new(None));
        let writer = slot.clone();
        let unmount = leptos::mount::mount_to(host, move || {
            AppState::provide_context();
            let app_state = AppState::expect_context();
            DerivedState::provide_context(app_state);
            writer.set(Some(app_state));
            view! { <ComponentPalette /> }
        });

        settle().await;
        (slot.get().expect("state must be created"), unmount)
    }

    fn row(document: &web_sys::Document, name: &str) -> web_sys::HtmlElement {
        document
            .query_selector(&format!(r#"[aria-label="Add {name}"]"#))
            .unwrap()
            .unwrap_or_else(|| panic!("palette row for {name} must be rendered"))
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap()
    }

    /// Pressing Enter on the Heading row must add a semantic heading, not the
    /// default paragraph `kind: "Text"` would produce.
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn keyboard_activation_adds_the_real_heading() {
        let (app_state, unmount) = mount_palette().await;
        let document = document();

        let heading_row = row(&document, "Heading");
        let before = app_state.canvas.components.get_untracked();
        heading_row.dispatch_event(&keydown("Enter")).unwrap();
        settle().await;

        let added = newly_added(&before, &app_state.canvas.components.get_untracked());
        match &added {
            CanvasComponent::Text(text) => assert_eq!(
                text.tag,
                TextTag::H1,
                "the keyboard path must add a heading, not a paragraph"
            ),
            other => panic!("expected a heading, got {:?}", other.component_type()),
        }

        drop(unmount);
    }

    /// The same row, activated with Space, must agree with Enter.
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn space_activation_adds_the_real_link() {
        let (app_state, unmount) = mount_palette().await;
        let document = document();

        let link_row = row(&document, "Link");
        let before = app_state.canvas.components.get_untracked();
        link_row.dispatch_event(&keydown(" ")).unwrap();
        settle().await;

        let added = newly_added(&before, &app_state.canvas.components.get_untracked());
        match &added {
            CanvasComponent::Link(link) => assert!(
                !link.href.is_empty(),
                "the keyboard path must add a hyperlink with an href"
            ),
            other => panic!("expected a link, got {:?}", other.component_type()),
        }

        drop(unmount);
    }

    /// Dropping the payload the palette emits onto the canvas must build the
    /// same component the keyboard path does.
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn dropping_the_palette_payload_builds_the_real_link() {
        reset_environment();
        let document = document();
        let host = test_host(&document);

        let slot: std::rc::Rc<std::cell::Cell<Option<AppState>>> =
            std::rc::Rc::new(std::cell::Cell::new(None));
        let writer = slot.clone();
        let unmount = leptos::mount::mount_to(host, move || {
            AppState::provide_context();
            let app_state = AppState::expect_context();
            DerivedState::provide_context(app_state);
            writer.set(Some(app_state));
            view! { <div id="drop-sink" on:drop=move |ev| handle_drop(ev, None, app_state) /> }
        });
        settle().await;
        let app_state = slot.get().expect("state must be created");

        let library = app_state.ui.component_library.get_untracked();
        let entry = library
            .iter()
            .find(|c| c.name == "Link")
            .expect("Link must be in the live library");
        let payload = palette_drag_payload(entry);

        let data_transfer = web_sys::DataTransfer::new().unwrap();
        data_transfer.set_data("component", &payload).unwrap();

        // `web-sys` is built without the `DragEventInit` feature, so build the
        // event through the real JS constructor to get a usable `dataTransfer`.
        let init = js_sys::Object::new();
        let set = |key: &str, value: &wasm_bindgen::JsValue| {
            js_sys::Reflect::set(&init, &wasm_bindgen::JsValue::from_str(key), value).unwrap();
        };
        set("dataTransfer", data_transfer.as_ref());
        set("bubbles", &wasm_bindgen::JsValue::from_bool(true));
        set("cancelable", &wasm_bindgen::JsValue::from_bool(true));

        let ctor: js_sys::Function = js_sys::Reflect::get(
            &js_sys::global(),
            &wasm_bindgen::JsValue::from_str("DragEvent"),
        )
        .unwrap()
        .dyn_into()
        .unwrap();
        let drop_ev: web_sys::DragEvent = js_sys::Reflect::construct(
            &ctor,
            &js_sys::Array::of2(&wasm_bindgen::JsValue::from_str("drop"), &init),
        )
        .unwrap()
        .dyn_into()
        .unwrap();

        let before = app_state.canvas.components.get_untracked();
        document
            .get_element_by_id("drop-sink")
            .unwrap()
            .dispatch_event(&drop_ev)
            .unwrap();
        settle().await;

        let added = newly_added(&before, &app_state.canvas.components.get_untracked());
        match &added {
            CanvasComponent::Link(link) => assert!(
                !link.href.is_empty(),
                "the drop path must add a hyperlink with an href"
            ),
            other => panic!("expected a link, got {:?}", other.component_type()),
        }

        drop(unmount);
    }
}
