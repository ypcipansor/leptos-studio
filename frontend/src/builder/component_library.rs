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
    /// Stable identity of this library entry.
    ///
    /// Display names are not a safe identity: two saved entries can carry the
    /// same name, and a saved name can collide with a built-in one. Everything
    /// that has to point back at one specific entry (drag payloads, delete,
    /// rename) uses this id instead. `#[serde(default)]` mints an id for data
    /// written before the field existed, so old JSON still deserializes.
    #[serde(default = "new_library_id")]
    pub id: String,
    pub name: String,
    pub kind: String, // e.g. "Button", "Text", "Input", "Container", "Custom"
    pub template: Option<String>, // for custom
    pub category: String, // e.g. "Basic", "Custom"
    pub props_schema: Option<Vec<PropSchema>>, // daftar props dan validasi
    pub description: Option<String>,
}

/// Mint a fresh identity for a library entry.
pub fn new_library_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Drag payload prefix for a saved library entry.
///
/// Two entries can share a component `kind` while holding different designs, so
/// a saved entry is identified by its id rather than by its kind or its name.
pub const SAVED_COMPONENT_PREFIX: &str = "Saved::";

/// Drag payload prefix for the built-in `Custom` placeholder component.
pub const CUSTOM_COMPONENT_PREFIX: &str = "Custom::";

/// Why a saved entry could not be added or renamed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LibraryNameError {
    /// The name was empty or only whitespace.
    EmptyName,
    /// Another entry already uses that name (case-insensitively).
    DuplicateName,
}

impl LibraryNameError {
    pub fn message(&self) -> String {
        match self {
            LibraryNameError::EmptyName => "⚠️ Component name cannot be empty.".to_string(),
            LibraryNameError::DuplicateName => {
                "⚠️ A component with that name already exists. Choose another name.".to_string()
            }
        }
    }
}

/// Normalize and validate a library entry name against a library.
///
/// Names are the user-facing identity shown in the palette, so duplicates are
/// rejected rather than silently allowed: the palette would otherwise show two
/// rows that look identical. `ignore_id` lets a rename skip the entry being
/// renamed.
pub fn validate_library_name(
    library: &[LibraryComponent],
    name: &str,
    ignore_id: Option<&str>,
) -> Result<String, LibraryNameError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(LibraryNameError::EmptyName);
    }

    let taken = library.iter().any(|entry| {
        Some(entry.id.as_str()) != ignore_id && entry.name.eq_ignore_ascii_case(trimmed)
    });
    if taken {
        return Err(LibraryNameError::DuplicateName);
    }

    Ok(trimmed.to_string())
}

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
        let name = name.trim();
        library.iter().any(|c| c.name.eq_ignore_ascii_case(name))
    }

    /// Add a custom component to both the custom_components list and the
    /// component_library collection.
    ///
    /// The name is trimmed and validated first, so a blank or duplicate name is
    /// reported back to the caller instead of creating an ambiguous entry.
    pub fn add_custom(
        custom_components: &mut Vec<LibraryComponent>,
        component_library: &mut Vec<LibraryComponent>,
        mut component: LibraryComponent,
    ) -> Result<(), LibraryNameError> {
        let name = validate_library_name(component_library, &component.name, None)?;
        component.name = name;

        custom_components.push(component.clone());
        component_library.push(component);
        Ok(())
    }

    /// Delete a custom component by index, removing the matching library entry
    /// by its id.
    pub fn delete_custom_by_index(
        custom_components: &mut Vec<LibraryComponent>,
        component_library: &mut Vec<LibraryComponent>,
        idx: usize,
    ) {
        if idx >= custom_components.len() {
            return;
        }

        let id = custom_components[idx].id.clone();
        custom_components.remove(idx);

        if let Some(pos) = component_library.iter().position(|c| c.id == id) {
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
    ) -> Result<(), LibraryNameError> {
        if idx >= custom_components.len() {
            return Err(LibraryNameError::EmptyName);
        }

        let id = custom_components[idx].id.clone();
        let name = validate_library_name(component_library, &new_name, Some(&id))?;

        if let Some(item) = custom_components.get_mut(idx) {
            item.name = name.clone();
            item.template = Some(new_template.clone());
        }

        if let Some(item) = component_library.iter_mut().find(|c| c.id == id) {
            item.name = name;
            item.template = Some(new_template);
        }

        Ok(())
    }
}

pub fn builtin_library_components() -> Vec<LibraryComponent> {
    vec![
        LibraryComponent {
            id: "builtin-button".to_string(),
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
            id: "builtin-text".to_string(),
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
            id: "builtin-input".to_string(),
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
            id: "builtin-select".to_string(),
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
            id: "builtin-container".to_string(),
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
            id: "builtin-row".to_string(),
            name: "Row".to_string(),
            kind: "Row".to_string(),
            template: None,
            category: "Layout".to_string(),
            props_schema: None, // Inherits from Container
            description: Some("Horizontal layout container".to_string()),
        },
        LibraryComponent {
            id: "builtin-column".to_string(),
            name: "Column".to_string(),
            kind: "Column".to_string(),
            template: None,
            category: "Layout".to_string(),
            props_schema: None, // Inherits from Container
            description: Some("Vertical layout container".to_string()),
        },
        LibraryComponent {
            id: "builtin-image".to_string(),
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
            id: "builtin-card".to_string(),
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
            id: "builtin-divider".to_string(),
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
            id: "builtin-checkbox".to_string(),
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
            id: "builtin-radiogroup".to_string(),
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
            id: "builtin-switch".to_string(),
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
            id: "builtin-badge".to_string(),
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
            id: "builtin-progress".to_string(),
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
        return format!("{}{}", SAVED_COMPONENT_PREFIX, component.id);
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
/// the palette's current contents, which is where the template lives. The entry is
/// found by its stable id, so two entries sharing a display name — or a saved name
/// that matches a built-in — can never resolve to the wrong design. The result
/// always gets fresh ids (recursively for containers and cards) so the same entry
/// can be dropped repeatedly. Payloads naming a component kind fall back to a new
/// default component.
pub fn create_canvas_component_from_payload(
    payload: &str,
    library: &[LibraryComponent],
) -> Option<CanvasComponent> {
    if let Some(id) = payload.strip_prefix(SAVED_COMPONENT_PREFIX) {
        let entry = library.iter().find(|entry| entry.id == id)?;
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
        ButtonComponent, ButtonSize, ButtonVariant, CardComponent, Component, ComponentStyle,
        ComponentType, ContainerComponent,
    };

    fn saved_button_entry(id: &str, label: &str, background_color: &str) -> LibraryComponent {
        let mut button = ButtonComponent::new(label.to_string());
        button.variant = ButtonVariant::Outline;
        button.size = ButtonSize::Large;
        button.style = ComponentStyle {
            background_color: Some(background_color.to_string()),
            ..ComponentStyle::default()
        };

        LibraryComponent {
            id: id.to_string(),
            name: label.to_string(),
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

    /// Build a saved entry from an arbitrary canvas component, the way the real
    /// "Save to Library" flow stores it.
    fn saved_entry(name: &str, component: CanvasComponent) -> LibraryComponent {
        LibraryComponent {
            id: new_library_id(),
            name: name.to_string(),
            kind: component.component_type().to_string(),
            category: "Custom".to_string(),
            description: Some("User saved component".to_string()),
            template: Some(serde_json::to_string_pretty(&component).unwrap()),
            props_schema: None,
        }
    }

    fn register(
        library: &mut Vec<LibraryComponent>,
        custom: &mut Vec<LibraryComponent>,
        entry: LibraryComponent,
    ) {
        ComponentRegistry::add_custom(custom, library, entry).expect("entry should be accepted");
    }

    #[test]
    fn ids_are_minted_for_entries_written_before_the_field_existed() {
        // Old project JSON has no `id`; deserializing must still work and give
        // the entry a usable identity.
        let legacy = r#"{
            "name": "Legacy",
            "kind": "Button",
            "template": null,
            "category": "Custom",
            "props_schema": null,
            "description": null
        }"#;

        let entry: LibraryComponent = serde_json::from_str(legacy).expect("legacy entry loads");
        assert!(!entry.id.is_empty(), "a legacy entry must gain an id");

        let other: LibraryComponent = serde_json::from_str(legacy).expect("legacy entry loads");
        assert_ne!(entry.id, other.id, "each migrated entry must be distinct");
    }

    #[test]
    fn test_saved_component_keeps_its_design_when_recreated() {
        let entry = saved_button_entry("entry-1", "Checkout", "#2563eb");
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

    /// Regression for the id-based identity: two saved entries with the *same*
    /// display name but different designs must each resolve to their own design.
    #[test]
    fn test_same_named_entries_resolve_to_their_own_design() {
        let first = saved_button_entry("entry-red", "Primary", "#dc2626");
        let second = saved_button_entry("entry-blue", "Primary", "#2563eb");
        let library = vec![first.clone(), second.clone()];

        for entry in [&first, &second] {
            let payload = palette_drag_payload(entry);
            assert!(
                payload.starts_with(SAVED_COMPONENT_PREFIX),
                "a saved entry must be addressed by id, got {payload}"
            );
            assert!(
                payload.ends_with(&entry.id),
                "the payload must carry the entry id, got {payload}"
            );

            let recreated = create_canvas_component_from_payload(&payload, &library)
                .expect("saved entry should be recreated");
            let CanvasComponent::Button(button) = &recreated else {
                panic!("expected a Button");
            };

            let expected = if entry.id == first.id {
                "#dc2626"
            } else {
                "#2563eb"
            };
            assert_eq!(
                button.style.background_color.as_deref(),
                Some(expected),
                "entry {} resolved to the wrong design",
                entry.id
            );
        }

        // Same name, different identity: they must not collide.
        assert_eq!(first.name, second.name);
        assert_ne!(first.id, second.id);
    }

    /// A saved entry may share a name with a built-in component — imported or
    /// legacy library data can contain one — and must still resolve to the saved
    /// design, not to the built-in default.
    #[test]
    fn test_saved_entry_shadowing_a_builtin_keeps_its_own_design() {
        let mut library = builtin_library_components();
        let saved = saved_button_entry("saved-button", "Button", "#f97316");

        // Built by hand rather than through `add_custom`, which rejects the
        // collision: this is the state old or imported library data can be in.
        library.push(saved.clone());

        let builtin = library
            .iter()
            .find(|c| c.id == "builtin-button")
            .expect("built-in Button missing");
        assert_eq!(builtin.name, saved.name, "the names intentionally collide");

        // The built-in still drags by kind and produces a default component.
        let from_builtin =
            create_canvas_component_from_payload(&palette_drag_payload(builtin), &library)
                .expect("built-in Button should still create a component");
        let CanvasComponent::Button(default_button) = &from_builtin else {
            panic!("expected a Button");
        };
        assert_ne!(
            default_button.style.background_color.as_deref(),
            Some("#f97316"),
            "the built-in must not inherit the saved design"
        );

        // The saved entry keeps its own design.
        let from_saved =
            create_canvas_component_from_payload(&palette_drag_payload(&saved), &library)
                .expect("saved entry should be recreated");
        let CanvasComponent::Button(saved_button) = &from_saved else {
            panic!("expected a Button");
        };
        assert_eq!(
            saved_button.style.background_color.as_deref(),
            Some("#f97316")
        );
    }

    #[test]
    fn test_container_saved_to_library_regenerates_child_ids() {
        let inner = CanvasComponent::Button(ButtonComponent::new("Inner".to_string()));
        let original_parent_id = ContainerComponent::new().id;
        let original_child_id = *inner.id();

        let mut container = ContainerComponent::new();
        container.children.push(inner);
        let original_container = CanvasComponent::Container(container);
        let original_container_id = *original_container.id();

        let entry = saved_entry("Card shell", original_container);

        let recreated = create_canvas_component_from_payload(
            &palette_drag_payload(&entry),
            std::slice::from_ref(&entry),
        )
        .expect("saved container should be recreated");

        let CanvasComponent::Container(container) = &recreated else {
            panic!("expected a Container");
        };

        assert_ne!(
            *recreated.id(),
            original_container_id,
            "the recreated parent must get a new id"
        );
        assert_ne!(
            *container.id(),
            original_parent_id,
            "the recreated parent must not reuse its original id"
        );
        assert_eq!(container.children.len(), 1);
        assert_ne!(
            container.children[0].id(),
            &original_child_id,
            "the child must get a new id, not keep its original one"
        );
        assert_ne!(
            container.children[0].id(),
            recreated.id(),
            "the child must not reuse the recreated parent id"
        );

        let CanvasComponent::Button(button) = &container.children[0] else {
            panic!("the child must keep its type");
        };
        assert_eq!(button.label, "Inner", "the child must keep its properties");
    }

    /// Container -> Card -> child: recursion must reach the deepest level.
    #[test]
    fn test_nested_container_card_children_all_get_new_ids() {
        let mut card = CardComponent::new();
        card.children
            .push(CanvasComponent::Button(ButtonComponent::new(
                "Leaf".to_string(),
            )));

        let mut container = ContainerComponent::new();
        container.children.push(CanvasComponent::Card(card));

        let original = CanvasComponent::Container(container);
        let original_container_id = *original.id();
        let CanvasComponent::Container(original_container) = &original else {
            unreachable!()
        };
        let original_card_id = *original_container.children[0].id();
        let CanvasComponent::Card(original_card) = &original_container.children[0] else {
            unreachable!()
        };
        let original_leaf_id = *original_card.children[0].id();

        let entry = saved_entry("Nested shell", original);

        let recreated = create_canvas_component_from_payload(
            &palette_drag_payload(&entry),
            std::slice::from_ref(&entry),
        )
        .expect("nested entry should be recreated");

        let CanvasComponent::Container(outer) = &recreated else {
            panic!("expected a Container");
        };
        assert_ne!(*outer.id(), original_container_id);

        let CanvasComponent::Card(card) = &outer.children[0] else {
            panic!("expected a nested Card");
        };
        assert_ne!(
            *card.id(),
            original_card_id,
            "the nested card needs a new id"
        );

        let CanvasComponent::Button(leaf) = &card.children[0] else {
            panic!("expected a nested Button");
        };
        assert_ne!(
            *leaf.id(),
            original_leaf_id,
            "the deepest child needs a new id"
        );
        assert_ne!(*leaf.id(), *card.id());
        assert_ne!(*leaf.id(), *outer.id());
        assert_eq!(leaf.label, "Leaf", "the deepest child keeps its properties");
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

        let entry = saved_entry("Purchase action", on_canvas.clone());

        let mut custom = Vec::new();
        let mut library = builtin_library_components();
        register(&mut library, &mut custom, entry);

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

    #[test]
    fn test_blank_and_duplicate_names_are_rejected() {
        let mut custom = Vec::new();
        let mut library = builtin_library_components();

        let blank = saved_entry(
            "   ",
            CanvasComponent::Button(ButtonComponent::new("X".into())),
        );
        assert_eq!(
            ComponentRegistry::add_custom(&mut custom, &mut library, blank),
            Err(LibraryNameError::EmptyName)
        );
        assert!(custom.is_empty(), "a rejected entry must not be stored");
        assert_eq!(library.len(), builtin_library_components().len());

        let first = saved_entry(
            "Shared name",
            CanvasComponent::Button(ButtonComponent::new("A".into())),
        );
        register(&mut library, &mut custom, first);

        let duplicate = saved_entry(
            "shared NAME",
            CanvasComponent::Button(ButtonComponent::new("B".into())),
        );
        assert_eq!(
            ComponentRegistry::add_custom(&mut custom, &mut library, duplicate),
            Err(LibraryNameError::DuplicateName)
        );
        assert_eq!(custom.len(), 1, "the duplicate must not be stored");

        // A name that matches a built-in is also ambiguous and rejected.
        let builtin_name = saved_entry(
            "Button",
            CanvasComponent::Button(ButtonComponent::new("C".into())),
        );
        assert_eq!(
            ComponentRegistry::add_custom(&mut custom, &mut library, builtin_name),
            Err(LibraryNameError::DuplicateName)
        );
    }

    #[test]
    fn test_accepted_name_is_trimmed() {
        let mut custom = Vec::new();
        let mut library = builtin_library_components();

        let padded = saved_entry(
            "  Spaced  ",
            CanvasComponent::Button(ButtonComponent::new("A".into())),
        );
        register(&mut library, &mut custom, padded);

        assert_eq!(custom[0].name, "Spaced");
        let stored = library
            .iter()
            .find(|c| c.id == custom[0].id)
            .expect("entry must be in the library");
        assert_eq!(stored.name, "Spaced");
    }

    /// Renaming or deleting one entry must not touch a different entry that
    /// happens to share its display name.
    #[test]
    fn test_rename_and_delete_target_the_right_entry() {
        let first = saved_button_entry("entry-keep", "Twin", "#dc2626");
        let second = saved_button_entry("entry-change", "Twin", "#2563eb");

        let mut library = vec![first.clone(), second.clone()];
        let mut custom = library.clone();

        // Rename the second entry. Names collide, so this exercises the id path.
        ComponentRegistry::update_custom_by_index(
            &mut custom,
            &mut library,
            1,
            "Renamed twin".to_string(),
            second.template.clone().unwrap(),
        )
        .expect("rename should succeed");

        assert_eq!(custom[0].name, "Twin");
        assert_eq!(custom[1].name, "Renamed twin");
        assert_eq!(custom[1].id, "entry-change");

        let renamed = library
            .iter()
            .find(|c| c.id == "entry-change")
            .expect("renamed entry must be present");
        assert_eq!(renamed.name, "Renamed twin");
        let untouched = library
            .iter()
            .find(|c| c.id == "entry-keep")
            .expect("the other entry must survive");
        assert_eq!(untouched.name, "Twin");
        assert_eq!(untouched.template, first.template);

        // Delete the first entry; the renamed one must survive with its design.
        ComponentRegistry::delete_custom_by_index(&mut custom, &mut library, 0);
        assert!(custom.iter().all(|c| c.id != "entry-keep"));
        assert!(library.iter().all(|c| c.id != "entry-keep"));
        let survivor = library
            .iter()
            .find(|c| c.id == "entry-change")
            .expect("the renamed entry must not be deleted");
        assert_eq!(survivor.template, second.template);
    }

    #[test]
    fn test_rename_rejects_blank_and_colliding_names() {
        let first = saved_button_entry("entry-a", "Alpha", "#dc2626");
        let second = saved_button_entry("entry-b", "Beta", "#2563eb");

        let mut library = vec![first.clone(), second.clone()];
        let mut custom = library.clone();

        assert_eq!(
            ComponentRegistry::update_custom_by_index(
                &mut custom,
                &mut library,
                1,
                "   ".to_string(),
                second.template.clone().unwrap(),
            ),
            Err(LibraryNameError::EmptyName)
        );
        assert_eq!(
            ComponentRegistry::update_custom_by_index(
                &mut custom,
                &mut library,
                1,
                "Alpha".to_string(),
                second.template.clone().unwrap(),
            ),
            Err(LibraryNameError::DuplicateName)
        );

        assert_eq!(custom[1].name, "Beta", "a rejected rename must not apply");
        assert_eq!(library[1].name, "Beta");

        // Renaming an entry to its own name is a no-op, not a collision.
        ComponentRegistry::update_custom_by_index(
            &mut custom,
            &mut library,
            1,
            "Beta".to_string(),
            second.template.clone().unwrap(),
        )
        .expect("keeping the same name must be allowed");
        assert_eq!(custom[1].name, "Beta");
    }

    /// The payload must keep working after the entry it names has been renamed.
    #[test]
    fn test_payload_survives_a_rename() {
        let mut library = builtin_library_components();
        let mut custom = Vec::new();
        let mut entry = saved_button_entry("entry-stable", "Before", "#0ea5e9");
        entry.template = Some(
            serde_json::to_string(&CanvasComponent::Button(ButtonComponent::new(
                "Before".to_string(),
            )))
            .unwrap(),
        );
        register(&mut library, &mut custom, entry);

        let payload = palette_drag_payload(&custom[0]);
        let template = custom[0].template.clone().unwrap();
        ComponentRegistry::update_custom_by_index(
            &mut custom,
            &mut library,
            0,
            "After".to_string(),
            template,
        )
        .expect("rename should succeed");

        let recreated = create_canvas_component_from_payload(&payload, &library)
            .expect("a renamed entry must still be draggable by its payload");
        let CanvasComponent::Button(button) = &recreated else {
            panic!("expected a Button");
        };
        assert_eq!(button.label, "Before");
    }
}
