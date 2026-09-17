use crate::domain::{
    ButtonComponent, CanvasComponent, ContainerComponent, CustomComponent, FlexDirection,
    InputComponent, LayoutType, SelectComponent, TextComponent,
};
use serde::{Deserialize, Serialize};

// Re-export types from state module to avoid duplication
pub use crate::state::app_state::{ResponsiveMode, Theme};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropType {
    String,
    Number,
    Bool,
    Enum { options: Vec<String> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropSchema {
    pub name: String,
    pub prop_type: PropType, // e.g. "string", "number", "bool"
    pub required: bool,
    pub description: Option<String>,
}

// Shared definition for LibraryComponent used in component library management

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibraryComponent {
    pub name: String,
    pub kind: String, // e.g. "Button", "Text", "Input", "Container", "Custom"
    pub template: Option<String>, // for custom
    pub category: String, // e.g. "Basic", "Custom"
    pub props_schema: Option<Vec<PropSchema>>, // daftar props dan validasi
    pub description: Option<String>,
}

/// Drag payload prefix for a saved library entry.
///
/// Two entries can share a component `kind` while holding different designs, so
/// a saved entry is identified by its name rather than by its kind.
pub const SAVED_COMPONENT_PREFIX: &str = "Saved::";

/// Drag payload prefix for the built-in `Custom` placeholder component.
pub const CUSTOM_COMPONENT_PREFIX: &str = "Custom::";

/// Simple registry helper for working with LibraryComponent collections.
pub struct ComponentRegistry;

impl ComponentRegistry {
    /// Return all components with category "Custom" from a library.
    pub fn custom_from_library(library: &[LibraryComponent]) -> Vec<LibraryComponent> {
        library
            .iter()
            .filter(|c| c.category == "Custom")
            .cloned()
            .collect()
    }

    /// Check whether a library already contains a component with the given name.
    pub fn exists_by_name(library: &[LibraryComponent], name: &str) -> bool {
        library.iter().any(|c| c.name == name)
    }

    /// Add a custom component to both the custom_components list and the
    /// component_library collection.
    pub fn add_custom(
        custom_components: &mut Vec<LibraryComponent>,
        component_library: &mut Vec<LibraryComponent>,
        component: LibraryComponent,
    ) {
        custom_components.push(component.clone());
        component_library.push(component);
    }

    /// Delete a custom component by index from custom_components and remove the
    /// corresponding entry from component_library by name.
    pub fn delete_custom_by_index(
        custom_components: &mut Vec<LibraryComponent>,
        component_library: &mut Vec<LibraryComponent>,
        idx: usize,
    ) {
        if idx >= custom_components.len() {
            return;
        }

        let name = custom_components[idx].name.clone();
        custom_components.remove(idx);

        if let Some(pos) = component_library.iter().position(|c| c.name == name) {
            component_library.remove(pos);
        }
    }

    #[allow(clippy::ptr_arg)]
    pub fn update_custom_by_index(
        custom_components: &mut [LibraryComponent],
        component_library: &mut Vec<LibraryComponent>,
        idx: usize,
        new_name: String,
        new_template: String,
    ) {
        if idx >= custom_components.len() {
            return;
        }

        let old_name = custom_components[idx].name.clone();

        if let Some(item) = custom_components.get_mut(idx) {
            item.name = new_name.clone();
            item.template = Some(new_template.clone());
        }

        if let Some(item) = component_library.iter_mut().find(|c| c.name == old_name) {
            item.name = new_name;
            item.template = Some(new_template);
        }
    }
}

pub fn builtin_library_components() -> Vec<LibraryComponent> {
    vec![
        LibraryComponent {
            name: "Button".to_string(),
            kind: "Button".to_string(),
            template: None,
            category: "Basic".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "label".to_string(),
                    prop_type: PropType::String,
                    required: true,
                    description: Some("Button label".to_string()),
                },
                PropSchema {
                    name: "variant".to_string(),
                    prop_type: PropType::Enum {
                        options: vec![
                            "Primary".to_string(),
                            "Secondary".to_string(),
                            "Outline".to_string(),
                            "Ghost".to_string(),
                        ],
                    },
                    required: true,
                    description: Some("Visual style variant".to_string()),
                },
                PropSchema {
                    name: "size".to_string(),
                    prop_type: PropType::Enum {
                        options: vec![
                            "Small".to_string(),
                            "Medium".to_string(),
                            "Large".to_string(),
                        ],
                    },
                    required: true,
                    description: Some("Button size".to_string()),
                },
                PropSchema {
                    name: "disabled".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Disable interaction".to_string()),
                },
            ]),
            description: Some("Interactive button component".to_string()),
        },
        LibraryComponent {
            name: "Text".to_string(),
            kind: "Text".to_string(),
            template: None,
            category: "Basic".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "content".to_string(),
                    prop_type: PropType::String,
                    required: false,
                    description: Some("Text content".to_string()),
                },
                PropSchema {
                    name: "style".to_string(),
                    prop_type: PropType::Enum {
                        options: vec![
                            "Heading1".to_string(),
                            "Heading2".to_string(),
                            "Heading3".to_string(),
                            "Body".to_string(),
                            "Caption".to_string(),
                        ],
                    },
                    required: true,
                    description: Some("Typographic style".to_string()),
                },
                PropSchema {
                    name: "tag".to_string(),
                    prop_type: PropType::Enum {
                        options: vec![
                            "H1".to_string(),
                            "H2".to_string(),
                            "H3".to_string(),
                            "P".to_string(),
                            "Span".to_string(),
                        ],
                    },
                    required: true,
                    description: Some("HTML tag".to_string()),
                },
            ]),
            description: Some("Text label or paragraph".to_string()),
        },
        LibraryComponent {
            name: "Input".to_string(),
            kind: "Input".to_string(),
            template: None,
            category: "Basic".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "placeholder".to_string(),
                    prop_type: PropType::String,
                    required: false,
                    description: Some("Placeholder text".to_string()),
                },
                PropSchema {
                    name: "input_type".to_string(),
                    prop_type: PropType::Enum {
                        options: vec![
                            "Text".to_string(),
                            "Password".to_string(),
                            "Email".to_string(),
                            "Number".to_string(),
                            "Tel".to_string(),
                        ],
                    },
                    required: true,
                    description: Some("Input type".to_string()),
                },
                PropSchema {
                    name: "required".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Field is required".to_string()),
                },
                PropSchema {
                    name: "disabled".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Disable input".to_string()),
                },
            ]),
            description: Some("Text input field".to_string()),
        },
        LibraryComponent {
            name: "Select".to_string(),
            kind: "Select".to_string(),
            template: None,
            category: "Basic".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "options".to_string(),
                    prop_type: PropType::String,
                    required: true,
                    description: Some("Comma separated options (e.g., A, B, C)".to_string()),
                },
                PropSchema {
                    name: "placeholder".to_string(),
                    prop_type: PropType::String,
                    required: false,
                    description: Some("Placeholder text".to_string()),
                },
                PropSchema {
                    name: "disabled".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Disable select".to_string()),
                },
            ]),
            description: Some("Dropdown selection component".to_string()),
        },
        LibraryComponent {
            name: "Container".to_string(),
            kind: "Container".to_string(),
            template: None,
            category: "Layout".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "layout".to_string(),
                    prop_type: PropType::Enum {
                        options: vec![
                            "FlexRow".to_string(),
                            "FlexColumn".to_string(),
                            "Grid".to_string(),
                            "Stack".to_string(),
                        ],
                    },
                    required: true,
                    description: Some("Layout type".to_string()),
                },
                PropSchema {
                    name: "align_items".to_string(),
                    prop_type: PropType::Enum {
                        options: vec![
                            "Start".to_string(),
                            "Center".to_string(),
                            "End".to_string(),
                            "Stretch".to_string(),
                            "Baseline".to_string(),
                        ],
                    },
                    required: false,
                    description: Some("Align items (cross axis)".to_string()),
                },
                PropSchema {
                    name: "justify_content".to_string(),
                    prop_type: PropType::Enum {
                        options: vec![
                            "Start".to_string(),
                            "Center".to_string(),
                            "End".to_string(),
                            "Between".to_string(),
                            "Around".to_string(),
                            "Evenly".to_string(),
                        ],
                    },
                    required: false,
                    description: Some("Justify content (main axis)".to_string()),
                },
                PropSchema {
                    name: "gap".to_string(),
                    prop_type: PropType::Number,
                    required: false,
                    description: Some("Gap between children (px)".to_string()),
                },
                PropSchema {
                    name: "padding_top".to_string(),
                    prop_type: PropType::Number,
                    required: false,
                    description: Some("Padding top (px)".to_string()),
                },
                PropSchema {
                    name: "padding_right".to_string(),
                    prop_type: PropType::Number,
                    required: false,
                    description: Some("Padding right (px)".to_string()),
                },
                PropSchema {
                    name: "padding_bottom".to_string(),
                    prop_type: PropType::Number,
                    required: false,
                    description: Some("Padding bottom (px)".to_string()),
                },
                PropSchema {
                    name: "padding_left".to_string(),
                    prop_type: PropType::Number,
                    required: false,
                    description: Some("Padding left (px)".to_string()),
                },
            ]),
            description: Some("Container for other components".to_string()),
        },
        LibraryComponent {
            name: "Row".to_string(),
            kind: "Row".to_string(),
            template: None,
            category: "Layout".to_string(),
            props_schema: None, // Inherits from Container
            description: Some("Horizontal layout container".to_string()),
        },
        LibraryComponent {
            name: "Column".to_string(),
            kind: "Column".to_string(),
            template: None,
            category: "Layout".to_string(),
            props_schema: None, // Inherits from Container
            description: Some("Vertical layout container".to_string()),
        },
        LibraryComponent {
            name: "Image".to_string(),
            kind: "Image".to_string(),
            template: None,
            category: "Media".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "src".to_string(),
                    prop_type: PropType::String,
                    required: true,
                    description: Some("Image source URL".to_string()),
                },
                PropSchema {
                    name: "alt".to_string(),
                    prop_type: PropType::String,
                    required: true,
                    description: Some("Alt text for accessibility".to_string()),
                },
                PropSchema {
                    name: "width".to_string(),
                    prop_type: PropType::String,
                    required: false,
                    description: Some("Width (e.g. 100%, 200px)".to_string()),
                },
                PropSchema {
                    name: "height".to_string(),
                    prop_type: PropType::String,
                    required: false,
                    description: Some("Height (e.g. auto, 150px)".to_string()),
                },
            ]),
            description: Some("Display an image".to_string()),
        },
        LibraryComponent {
            name: "Card".to_string(),
            kind: "Card".to_string(),
            template: None,
            category: "Layout".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "padding".to_string(),
                    prop_type: PropType::Number,
                    required: false,
                    description: Some("Internal padding (px)".to_string()),
                },
                PropSchema {
                    name: "border_radius".to_string(),
                    prop_type: PropType::Number,
                    required: false,
                    description: Some("Border radius (px)".to_string()),
                },
                PropSchema {
                    name: "shadow".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Show shadow".to_string()),
                },
                PropSchema {
                    name: "border".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Show border".to_string()),
                },
            ]),
            description: Some("Card container with shadow and rounded corners".to_string()),
        },
        LibraryComponent {
            name: "Divider".to_string(),
            kind: "Divider".to_string(),
            template: None,
            category: "Layout".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "orientation".to_string(),
                    prop_type: PropType::Enum {
                        options: vec!["Horizontal".to_string(), "Vertical".to_string()],
                    },
                    required: true,
                    description: Some("Divider orientation".to_string()),
                },
                PropSchema {
                    name: "thickness".to_string(),
                    prop_type: PropType::Number,
                    required: false,
                    description: Some("Line thickness in pixels".to_string()),
                },
            ]),
            description: Some("Visual separator between content".to_string()),
        },
        LibraryComponent {
            name: "Checkbox".to_string(),
            kind: "Checkbox".to_string(),
            template: None,
            category: "Form".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "label".to_string(),
                    prop_type: PropType::String,
                    required: false,
                    description: Some("Checkbox label".to_string()),
                },
                PropSchema {
                    name: "checked".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Checked state".to_string()),
                },
                PropSchema {
                    name: "disabled".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Disable interaction".to_string()),
                },
            ]),
            description: Some("Checkbox input with label".to_string()),
        },
        LibraryComponent {
            name: "RadioGroup".to_string(),
            kind: "RadioGroup".to_string(),
            template: None,
            category: "Form".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "options".to_string(),
                    prop_type: PropType::String,
                    required: true,
                    description: Some("Comma separated options".to_string()),
                },
                PropSchema {
                    name: "selected".to_string(),
                    prop_type: PropType::String,
                    required: false,
                    description: Some("Selected option".to_string()),
                },
                PropSchema {
                    name: "disabled".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Disable interaction".to_string()),
                },
            ]),
            description: Some("Radio button group".to_string()),
        },
        LibraryComponent {
            name: "Switch".to_string(),
            kind: "Switch".to_string(),
            template: None,
            category: "Form".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "label".to_string(),
                    prop_type: PropType::String,
                    required: false,
                    description: Some("Switch label".to_string()),
                },
                PropSchema {
                    name: "checked".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("On/Off state".to_string()),
                },
                PropSchema {
                    name: "disabled".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Disable interaction".to_string()),
                },
            ]),
            description: Some("Toggle switch".to_string()),
        },
        LibraryComponent {
            name: "Badge".to_string(),
            kind: "Badge".to_string(),
            template: None,
            category: "Basic".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "text".to_string(),
                    prop_type: PropType::String,
                    required: true,
                    description: Some("Badge text".to_string()),
                },
                PropSchema {
                    name: "variant".to_string(),
                    prop_type: PropType::Enum {
                        options: vec![
                            "Default".to_string(),
                            "Primary".to_string(),
                            "Success".to_string(),
                            "Warning".to_string(),
                            "Error".to_string(),
                        ],
                    },
                    required: true,
                    description: Some("Color variant".to_string()),
                },
            ]),
            description: Some("Small status badge".to_string()),
        },
        LibraryComponent {
            name: "Progress".to_string(),
            kind: "Progress".to_string(),
            template: None,
            category: "Basic".to_string(),
            props_schema: Some(vec![
                PropSchema {
                    name: "value".to_string(),
                    prop_type: PropType::Number,
                    required: true,
                    description: Some("Current value".to_string()),
                },
                PropSchema {
                    name: "max".to_string(),
                    prop_type: PropType::Number,
                    required: true,
                    description: Some("Maximum value".to_string()),
                },
                PropSchema {
                    name: "show_label".to_string(),
                    prop_type: PropType::Bool,
                    required: false,
                    description: Some("Show percentage label".to_string()),
                },
            ]),
            description: Some("Progress bar".to_string()),
        },
    ]
}

/// Build the drag payload a palette entry carries.
///
/// Entries with a stored `template` keep their design, so the payload has to
/// identify the entry itself instead of its component kind.
pub fn palette_drag_payload(component: &LibraryComponent) -> String {
    if component.template.is_some() {
        return format!("{}{}", SAVED_COMPONENT_PREFIX, component.name);
    }
    if component.kind == "Custom" {
        return format!("{}{}", CUSTOM_COMPONENT_PREFIX, component.name);
    }
    component.kind.clone()
}

/// Resolve a palette drag payload into a fresh canvas component.
///
/// Payloads naming a saved library entry are rebuilt by deserializing the entry's
/// `template`, so the saved properties, style and children come back. `library` is
/// the palette's current contents, which is where the template lives. The result
/// always gets fresh ids (recursively for containers and cards) so the same entry
/// can be dropped repeatedly. Payloads naming a component kind fall back to a new
/// default component.
pub fn create_canvas_component_from_payload(
    payload: &str,
    library: &[LibraryComponent],
) -> Option<CanvasComponent> {
    if let Some(name) = payload.strip_prefix(SAVED_COMPONENT_PREFIX) {
        let entry = library.iter().find(|entry| entry.name == name)?;
        let restored = entry
            .template
            .as_deref()
            .and_then(|template| serde_json::from_str::<CanvasComponent>(template).ok());
        return match restored {
            Some(component) => Some(component.duplicate_with_new_id()),
            // An unreadable template must not lose the component entirely; fall
            // back to the default shape its kind produces.
            None => create_canvas_component(&entry.kind),
        };
    }
    create_canvas_component(payload)
}

pub fn create_canvas_component(component_type: &str) -> Option<CanvasComponent> {
    match component_type {
        "Button" => {
            let button = ButtonComponent::new("Button".to_string());
            Some(CanvasComponent::Button(button))
        }
        "Text" => {
            let text = TextComponent::new("Text".to_string());
            Some(CanvasComponent::Text(text))
        }
        "Input" => {
            let input = InputComponent::new();
            Some(CanvasComponent::Input(input))
        }
        "Select" => {
            let select = SelectComponent::new();
            Some(CanvasComponent::Select(select))
        }
        "Container" => {
            let container = ContainerComponent::new();
            Some(CanvasComponent::Container(container))
        }
        "Row" => {
            let mut container = ContainerComponent::new();
            container.layout = LayoutType::Flex {
                direction: FlexDirection::Row,
                wrap: false,
                align_items: Default::default(),
                justify_content: Default::default(),
            };
            Some(CanvasComponent::Container(container))
        }
        "Column" => {
            let mut container = ContainerComponent::new();
            container.layout = LayoutType::Flex {
                direction: FlexDirection::Column,
                wrap: false,
                align_items: Default::default(),
                justify_content: Default::default(),
            };
            Some(CanvasComponent::Container(container))
        }
        "Image" => {
            let image = crate::domain::ImageComponent::new(
                "https://via.placeholder.com/150".to_string(),
                "Placeholder Image".to_string(),
            );
            Some(CanvasComponent::Image(image))
        }
        "Card" => {
            let card = crate::domain::CardComponent::new();
            Some(CanvasComponent::Card(card))
        }
        "Divider" => {
            let divider = crate::domain::DividerComponent::new();
            Some(CanvasComponent::Divider(divider))
        }
        "Checkbox" => {
            let checkbox = crate::domain::CheckboxComponent::new("Checkbox".to_string());
            Some(CanvasComponent::Checkbox(checkbox))
        }
        "RadioGroup" => {
            let radio = crate::domain::RadioGroupComponent::new();
            Some(CanvasComponent::RadioGroup(radio))
        }
        "Switch" => {
            let switch = crate::domain::SwitchComponent::new("Switch".to_string());
            Some(CanvasComponent::Switch(switch))
        }
        "Badge" => {
            let badge = crate::domain::BadgeComponent::new("Badge".to_string());
            Some(CanvasComponent::Badge(badge))
        }
        "Progress" => {
            let progress = crate::domain::ProgressComponent::new();
            Some(CanvasComponent::Progress(progress))
        }
        data if data.starts_with(CUSTOM_COMPONENT_PREFIX) => {
            let name = data
                .strip_prefix(CUSTOM_COMPONENT_PREFIX)
                .unwrap_or("Custom");
            let custom =
                CustomComponent::new(name.to_string(), "<div>Custom Component</div>".to_string());
            Some(CanvasComponent::Custom(custom))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ButtonComponent, ButtonSize, ButtonVariant, ComponentStyle, ComponentType,
    };

    fn saved_button_entry(label: &str, background_color: &str) -> LibraryComponent {
        let mut button = ButtonComponent::new(label.to_string());
        button.variant = ButtonVariant::Outline;
        button.size = ButtonSize::Large;
        button.style = ComponentStyle {
            background_color: Some(background_color.to_string()),
            ..ComponentStyle::default()
        };

        LibraryComponent {
            name: format!("{label} preset"),
            kind: ComponentType::Button.to_string(),
            category: "Custom".to_string(),
            description: Some("User saved component".to_string()),
            template: Some(
                serde_json::to_string_pretty(&CanvasComponent::Button(button))
                    .expect("button should serialize"),
            ),
            props_schema: None,
        }
    }

    #[test]
    fn test_saved_component_keeps_its_design_when_recreated() {
        let entry = saved_button_entry("Checkout", "#2563eb");
        let payload = palette_drag_payload(&entry);

        let recreated =
            create_canvas_component_from_payload(&payload, std::slice::from_ref(&entry))
                .expect("saved entry should be recreated");

        let CanvasComponent::Button(button) = &recreated else {
            panic!("expected a Button, got {:?}", recreated.component_type());
        };

        assert_eq!(button.label, "Checkout");
        assert_eq!(button.variant, ButtonVariant::Outline);
        assert_eq!(button.size, ButtonSize::Large);
        assert_eq!(button.style.background_color.as_deref(), Some("#2563eb"));

        // The saved design must come back under a fresh id so the same entry can
        // be dropped repeatedly without the canvas keyed components colliding.
        let original: CanvasComponent = serde_json::from_str(entry.template.as_deref().unwrap())
            .expect("template should deserialize");
        assert_ne!(recreated.id(), original.id());
    }

    #[test]
    fn test_container_saved_to_library_regenerates_child_ids() {
        let mut container = ContainerComponent::new();
        container
            .children
            .push(CanvasComponent::Button(ButtonComponent::new(
                "Inner".to_string(),
            )));

        let entry = LibraryComponent {
            template: Some(serde_json::to_string(&CanvasComponent::Container(container)).unwrap()),
            ..saved_button_entry("Card shell", "#ffffff")
        };

        let recreated = create_canvas_component_from_payload(
            &palette_drag_payload(&entry),
            std::slice::from_ref(&entry),
        )
        .expect("saved container should be recreated");

        let CanvasComponent::Container(container) = &recreated else {
            panic!("expected a Container");
        };
        assert_eq!(container.children.len(), 1);
        assert_ne!(container.children[0].id(), recreated.id());
    }

    #[test]
    fn test_builtin_entries_still_drag_by_kind() {
        let library = builtin_library_components();
        let button = library
            .iter()
            .find(|c| c.kind == "Button")
            .expect("built-in Button missing");

        assert_eq!(palette_drag_payload(button), "Button");

        let recreated = create_canvas_component_from_payload("Button", &library)
            .expect("built-in kind should still create a component");
        assert!(matches!(recreated, CanvasComponent::Button(_)));
        assert!(create_canvas_component_from_payload("Saved::Missing", &library).is_none());
    }

    /// Mirrors the real "Save to Library" flow: the canvas component is stored on
    /// the entry's `template` and registered via `add_custom`, then the palette
    /// payload for that entry must resolve back to the saved design.
    #[test]
    fn test_saved_entry_round_trips_through_add_custom() {
        let mut button = ButtonComponent::new("Purchase".to_string());
        button.variant = ButtonVariant::Ghost;
        button.style = ComponentStyle {
            color: Some("#111827".to_string()),
            ..ComponentStyle::default()
        };
        let on_canvas = CanvasComponent::Button(button);

        let entry = LibraryComponent {
            name: "Purchase action".to_string(),
            kind: on_canvas.component_type().to_string(),
            category: "Custom".to_string(),
            description: Some("User saved component".to_string()),
            template: Some(serde_json::to_string_pretty(&on_canvas).unwrap()),
            props_schema: None,
        };

        let mut custom = Vec::new();
        let mut library = builtin_library_components();
        ComponentRegistry::add_custom(&mut custom, &mut library, entry);

        assert_eq!(
            ComponentRegistry::custom_from_library(&library).len(),
            1,
            "saved entry must be visible in the palette"
        );

        let saved = library
            .iter()
            .find(|c| c.name == "Purchase action")
            .expect("saved entry missing from library");
        let recreated =
            create_canvas_component_from_payload(&palette_drag_payload(saved), &library)
                .expect("saved entry should be recreated");

        let CanvasComponent::Button(button) = &recreated else {
            panic!("expected a Button");
        };
        assert_eq!(button.label, "Purchase");
        assert_eq!(button.variant, ButtonVariant::Ghost);
        assert_eq!(button.style.color.as_deref(), Some("#111827"));
        assert_ne!(recreated.id(), on_canvas.id());

        // Dropping the same entry twice must not reuse an id.
        let second =
            create_canvas_component_from_payload(&palette_drag_payload(saved), &library).unwrap();
        assert_ne!(second.id(), recreated.id());
    }
}
