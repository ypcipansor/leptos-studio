//! Derived State & Computed Values
//!
//! Provides memoized, derived state computations for efficient
//! reactive updates. These signals only update when their
//! dependencies change.

use leptos::prelude::*;

use crate::domain::{CanvasComponent, ComponentId, ComponentType};
use crate::state::AppState;

/// Derived state computations for the canvas
#[derive(Clone, Copy)]
pub struct DerivedState {
    /// Total component count
    pub component_count: Memo<usize>,
    /// Count by component type
    pub type_counts: Memo<TypeCounts>,
    /// Whether canvas is empty
    pub is_empty: Memo<bool>,
    /// Whether any component is selected
    pub has_selection: Memo<bool>,
    /// Selected component (if any)
    pub selected_component: Memo<Option<CanvasComponent>>,
    /// Nesting depth of components
    pub max_nesting_depth: Memo<usize>,
    /// Custom components count
    pub custom_count: Memo<usize>,
    /// Whether undo is available
    pub can_undo: Memo<bool>,
    /// Whether redo is available  
    pub can_redo: Memo<bool>,
}

/// Component type counts
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TypeCounts {
    pub buttons: usize,
    pub texts: usize,
    pub inputs: usize,
    pub containers: usize,
    pub images: usize,
    pub cards: usize,
    pub selects: usize,
    pub customs: usize,
    pub others: usize,
}

impl TypeCounts {
    fn merge(&mut self, other: &TypeCounts) {
        self.buttons += other.buttons;
        self.texts += other.texts;
        self.inputs += other.inputs;
        self.containers += other.containers;
        self.images += other.images;
        self.cards += other.cards;
        self.selects += other.selects;
        self.customs += other.customs;
        self.others += other.others;
    }

    pub fn total(&self) -> usize {
        self.buttons
            + self.texts
            + self.inputs
            + self.containers
            + self.images
            + self.cards
            + self.selects
            + self.customs
            + self.others
    }
}

impl DerivedState {
    /// Create derived state from app state
    pub fn new(app_state: AppState) -> Self {
        let canvas = app_state.canvas;

        // Component count
        let component_count =
            Memo::new(move |_| count_components_recursive(&canvas.components.get()));

        // Type counts
        let type_counts = Memo::new(move |_| count_types_recursive(&canvas.components.get()));

        // Is empty
        let is_empty = Memo::new(move |_| canvas.components.get().is_empty());

        // Has selection
        let has_selection = Memo::new(move |_| canvas.selected.get().is_some());

        // Selected component
        let selected_component = Memo::new(move |_| {
            canvas
                .selected
                .get()
                .and_then(|id| find_component_by_id(&canvas.components.get(), &id))
        });

        // Max nesting depth
        let max_nesting_depth =
            Memo::new(move |_| calculate_max_depth(&canvas.components.get(), 0));

        // Custom count
        let custom_count = Memo::new(move |_| count_custom_components(&canvas.components.get()));

        // Can undo
        let can_undo = Memo::new(move |_| canvas.history.with(|h| h.can_undo()));

        // Can redo
        let can_redo = Memo::new(move |_| canvas.history.with(|h| h.can_redo()));

        Self {
            component_count,
            type_counts,
            is_empty,
            has_selection,
            selected_component,
            max_nesting_depth,
            custom_count,
            can_undo,
            can_redo,
        }
    }

    /// Provide derived state in Leptos context
    pub fn provide_context(app_state: AppState) {
        provide_context(Self::new(app_state));
    }

    /// Use derived state from Leptos context
    pub fn use_context() -> Self {
        expect_context::<Self>()
    }
}

/// Count all components recursively including nested containers
fn count_components_recursive(components: &[CanvasComponent]) -> usize {
    let mut count = components.len();
    for comp in components {
        match comp {
            CanvasComponent::Container(container) => {
                count += count_components_recursive(&container.children);
            }
            CanvasComponent::Card(card) => {
                count += count_components_recursive(&card.children);
            }
            _ => {}
        }
    }
    count
}

/// Count components by type recursively
fn count_types_recursive(components: &[CanvasComponent]) -> TypeCounts {
    let mut counts = TypeCounts::default();

    for comp in components {
        match comp {
            CanvasComponent::Button(_) => counts.buttons += 1,
            CanvasComponent::Text(_) => counts.texts += 1,
            CanvasComponent::Input(_) => counts.inputs += 1,
            CanvasComponent::Container(container) => {
                counts.containers += 1;
                let child_counts = count_types_recursive(&container.children);
                counts.merge(&child_counts);
            }
            CanvasComponent::Image(_) => counts.images += 1,
            CanvasComponent::Card(card) => {
                counts.cards += 1;
                let child_counts = count_types_recursive(&card.children);
                counts.merge(&child_counts);
            }
            CanvasComponent::Select(_) => counts.selects += 1,
            CanvasComponent::Custom(_) => counts.customs += 1,
            _ => counts.others += 1,
        }
    }

    counts
}

/// Find a component by ID in the tree
fn find_component_by_id(
    components: &[CanvasComponent],
    id: &ComponentId,
) -> Option<CanvasComponent> {
    for comp in components {
        if comp.id() == id {
            return Some(comp.clone());
        }
        match comp {
            CanvasComponent::Container(container) => {
                if let Some(found) = find_component_by_id(&container.children, id) {
                    return Some(found);
                }
            }
            CanvasComponent::Card(card) => {
                if let Some(found) = find_component_by_id(&card.children, id) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

/// Calculate maximum nesting depth
fn calculate_max_depth(components: &[CanvasComponent], current_depth: usize) -> usize {
    let mut max_depth = current_depth;

    for comp in components {
        match comp {
            CanvasComponent::Container(container) => {
                let child_depth = calculate_max_depth(&container.children, current_depth + 1);
                max_depth = max_depth.max(child_depth);
            }
            CanvasComponent::Card(card) => {
                let child_depth = calculate_max_depth(&card.children, current_depth + 1);
                max_depth = max_depth.max(child_depth);
            }
            _ => {}
        }
    }

    max_depth
}

/// Count custom components
fn count_custom_components(components: &[CanvasComponent]) -> usize {
    let mut count = 0;

    for comp in components {
        match comp {
            CanvasComponent::Custom(_) => count += 1,
            CanvasComponent::Container(container) => {
                count += count_custom_components(&container.children);
            }
            CanvasComponent::Card(card) => {
                count += count_custom_components(&card.children);
            }
            _ => {}
        }
    }

    count
}

/// Canvas statistics for debugging/display
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasStats {
    pub total_components: usize,
    pub type_counts: TypeCounts,
    pub max_depth: usize,
    pub has_selection: bool,
    pub selected_type: Option<ComponentType>,
}

impl CanvasStats {
    /// Create stats from derived state
    pub fn from_derived(derived: &DerivedState) -> Self {
        Self {
            total_components: derived.component_count.get(),
            type_counts: derived.type_counts.get(),
            max_depth: derived.max_nesting_depth.get(),
            has_selection: derived.has_selection.get(),
            selected_type: derived.selected_component.get().map(|c| c.component_type()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ButtonComponent, ContainerComponent, TextComponent};

    #[test]
    fn test_count_components() {
        let button = CanvasComponent::Button(ButtonComponent::new("Test".to_string()));
        let text = CanvasComponent::Text(TextComponent::new("Hello".to_string()));
        let components = vec![button, text];

        assert_eq!(count_components_recursive(&components), 2);
    }

    #[test]
    fn test_count_nested_components() {
        let button = CanvasComponent::Button(ButtonComponent::new("Test".to_string()));
        let mut container = ContainerComponent::new();
        container.children = vec![
            CanvasComponent::Text(TextComponent::new("Nested".to_string())),
            CanvasComponent::Button(ButtonComponent::new("Nested Button".to_string())),
        ];

        let components = vec![button, CanvasComponent::Container(container)];
        assert_eq!(count_components_recursive(&components), 4);
    }

    #[test]
    fn test_type_counts() {
        let button = CanvasComponent::Button(ButtonComponent::new("Test".to_string()));
        let text = CanvasComponent::Text(TextComponent::new("Hello".to_string()));
        let components = vec![button, text];

        let counts = count_types_recursive(&components);
        assert_eq!(counts.buttons, 1);
        assert_eq!(counts.texts, 1);
        assert_eq!(counts.total(), 2);
    }

    #[test]
    fn test_max_depth() {
        // Simple flat structure
        let button = CanvasComponent::Button(ButtonComponent::new("Test".to_string()));
        let components = vec![button];
        assert_eq!(calculate_max_depth(&components, 0), 0);

        // One level of nesting
        let mut container = ContainerComponent::new();
        container.children = vec![CanvasComponent::Button(ButtonComponent::new(
            "Nested".to_string(),
        ))];
        let components = vec![CanvasComponent::Container(container)];
        assert_eq!(calculate_max_depth(&components, 0), 1);
    }

    #[test]
    fn test_find_component() {
        let button = ButtonComponent::new("Test".to_string());
        let button_id = button.id;
        let component = CanvasComponent::Button(button);
        let components = vec![component.clone()];

        let found = find_component_by_id(&components, &button_id);
        assert!(found.is_some());
    }
}
