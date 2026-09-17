//! Component Palette
//!
//! Enhanced component palette with fuzzy search, categorization,
//! and drag-drop support. Provides quick access to all available
//! components with search and filter capabilities.

use leptos::prelude::*;

use crate::builder::component_library::{LibraryComponent, palette_drag_payload};
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
                                    key=|comp| format!("{}:{}", comp.kind, comp.name)
                                    children=move |comp| {
                                        let comp_kind = palette_drag_payload(&comp);
                                        let comp_name = comp.name.clone();
                                        let comp_desc = comp.description.clone().unwrap_or_default();
                                        let comp_category = comp.category.clone();

                                        let (on_drag_start, on_drag, on_drag_end) = create_drag_handlers(
                                            comp_kind.clone(),
                                            app_state.canvas.drag_state,
                                            DragDropConfig::default(),
                                        );

                                        view! {
                                            <div
                                                class="palette-item"
                                                draggable="true"
                                                on:dragstart=on_drag_start
                                                on:drag=on_drag
                                                on:dragend=on_drag_end
                                                role="option"
                                                tabindex="0"
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
        _ => "❓",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Every component in the built-in library must be reachable from at least
    /// one non-`All` tab, and the per-category counts must partition the
    /// library (each component counted under exactly one category tab).
    #[test]
    fn test_every_component_is_reachable_by_category() {
        let library = crate::builder::component_library::builtin_library_components();

        for comp in &library {
            assert!(
                CATEGORIES
                    .iter()
                    .any(|c| *c != ComponentCategory::All && c.matches(comp)),
                "{} (category {}) is not reachable from any category tab",
                comp.name,
                comp.category
            );
        }

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
    }
}
