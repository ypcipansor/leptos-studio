use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::error::ValidationError;
use super::style::ComponentStyle;
use super::validation::Validator;

/// Component ID for unique identification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(Uuid);

impl From<Uuid> for ComponentId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Animation types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnimationType {
    #[default]
    None,
    FadeIn,
    SlideInUp,
    SlideInDown,
    SlideInLeft,
    SlideInRight,
    Bounce,
    ZoomIn,
    Pulse,
}

/// Animation configuration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Animation {
    pub animation_type: AnimationType,
    pub duration: f32, // in seconds
    pub delay: f32,    // in seconds
    pub infinite: bool,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            animation_type: AnimationType::None,
            duration: 0.3,
            delay: 0.0,
            infinite: false,
        }
    }
}

impl Animation {
    pub fn to_css_string(&self) -> String {
        if self.animation_type == AnimationType::None {
            return String::new();
        }

        let anim_name = match self.animation_type {
            AnimationType::None => "",
            AnimationType::FadeIn => "fadeIn",
            AnimationType::SlideInUp => "slideInUp",
            AnimationType::SlideInDown => "slideInDown",
            AnimationType::SlideInLeft => "slideInLeft",
            AnimationType::SlideInRight => "slideInRight",
            AnimationType::Bounce => "bounce",
            AnimationType::ZoomIn => "zoomIn",
            AnimationType::Pulse => "pulse",
        };

        let iteration = if self.infinite { "infinite" } else { "1" };

        format!(
            "animation: {} {}s ease-in-out {}s {} both;",
            anim_name, self.duration, self.delay, iteration
        )
    }
}

impl ComponentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Component type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    Button,
    Text,
    Input,
    Container,
    Image,
    Card,
    Select,
    Custom,
    Divider,
    Checkbox,
    RadioGroup,
    Switch,
    Badge,
    Progress,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Core component trait for all UI components
pub trait Component: Clone {
    fn component_type(&self) -> ComponentType;
    fn id(&self) -> &ComponentId;
    fn validate(&self) -> Result<(), ValidationError>;
}

/// Button variants
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

/// Button sizes
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

/// Button component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ButtonComponent {
    pub id: ComponentId,
    pub label: String,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub disabled: bool,
    pub on_click: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl ButtonComponent {
    pub fn new(label: String) -> Self {
        Self {
            id: ComponentId::new(),
            label,
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            disabled: false,
            on_click: None,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Component for ButtonComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Button
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        if self.label.trim().is_empty() {
            return Err(ValidationError::InvalidPropertyValue(
                "label".to_string(),
                "Button label cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

/// Text styles
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextStyle {
    Heading1,
    Heading2,
    Heading3,
    Body,
    Caption,
}

/// Text HTML tags
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextTag {
    H1,
    H2,
    H3,
    P,
    Span,
}

/// Text component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextComponent {
    pub id: ComponentId,
    pub content: String,
    pub style: TextStyle,
    pub tag: TextTag,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub custom_style: ComponentStyle, // renamed to avoid conflict with existing 'style' field
}

impl TextComponent {
    pub fn new(content: String) -> Self {
        Self {
            id: ComponentId::new(),
            content,
            style: TextStyle::Body,
            tag: TextTag::P,
            animation: None,
            bindings: HashMap::new(),
            custom_style: ComponentStyle::default(),
        }
    }
}

impl Component for TextComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Text
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        // Text content can be empty (for placeholder text)
        Ok(())
    }
}

/// Input types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Tel,
}

/// Input component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputComponent {
    pub id: ComponentId,
    pub placeholder: String,
    pub input_type: InputType,
    pub required: bool,
    pub disabled: bool,
    pub on_change: Option<String>,
    pub on_input: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl InputComponent {
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(),
            placeholder: String::new(),
            input_type: InputType::Text,
            required: false,
            disabled: false,
            on_change: None,
            on_input: None,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Default for InputComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for InputComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Input
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// Select component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectComponent {
    pub id: ComponentId,
    pub options: String, // Comma separated values
    pub placeholder: String,
    pub disabled: bool,
    pub on_change: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl SelectComponent {
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(),
            options: "Option 1, Option 2, Option 3".to_string(),
            placeholder: "Select an option".to_string(),
            disabled: false,
            on_change: None,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Default for SelectComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SelectComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Select
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// Layout types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LayoutType {
    Flex {
        direction: FlexDirection,
        wrap: bool,
        #[serde(default)]
        align_items: FlexAlign,
        #[serde(default)]
        justify_content: FlexJustify,
    },
    Grid {
        columns: u32,
        rows: u32,
    },
    Stack,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FlexAlign {
    #[default]
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FlexJustify {
    #[default]
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}

/// Spacing
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Spacing {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

/// Container component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContainerComponent {
    pub id: ComponentId,
    pub children: Vec<CanvasComponent>,
    pub layout: LayoutType,
    pub gap: u32,
    pub padding: Spacing,
    pub on_click: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl ContainerComponent {
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(),
            children: Vec::new(),
            layout: LayoutType::Flex {
                direction: FlexDirection::Column,
                wrap: false,
                align_items: FlexAlign::default(),
                justify_content: FlexJustify::default(),
            },
            gap: 8,
            padding: Spacing::default(),
            on_click: None,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Default for ContainerComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ContainerComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Container
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        // Validate all children
        for child in &self.children {
            child.validate()?;
        }
        Ok(())
    }
}

/// Image component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageComponent {
    pub id: ComponentId,
    pub src: String,
    pub alt: String,
    pub width: Option<String>,
    pub height: Option<String>,
    pub on_click: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl ImageComponent {
    pub fn new(src: String, alt: String) -> Self {
        Self {
            id: ComponentId::new(),
            src,
            alt,
            width: None,
            height: None,
            on_click: None,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Component for ImageComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Image
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        if self.src.trim().is_empty() {
            return Err(ValidationError::InvalidPropertyValue(
                "src".to_string(),
                "Image source URL cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

/// Card component - A pre-styled container
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CardComponent {
    pub id: ComponentId,
    pub children: Vec<CanvasComponent>,
    pub padding: u32,
    pub shadow: bool,
    pub border: bool,
    pub border_radius: u32,
    pub on_click: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl CardComponent {
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(),
            children: Vec::new(),
            padding: 16,
            shadow: true,
            border: true,
            border_radius: 8,
            on_click: None,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Default for CardComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for CardComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Card
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        for child in &self.children {
            child.validate()?;
        }
        Ok(())
    }
}

/// Property value types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

/// Custom component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomComponent {
    pub id: ComponentId,
    pub name: String,
    pub template: String,
    pub props: HashMap<String, PropValue>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl CustomComponent {
    pub fn new(name: String, template: String) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            template,
            props: HashMap::new(),
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Component for CustomComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Custom
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        use super::validation::{ComponentNameValidator, HtmlTemplateValidator};

        // Validate name
        let name_validator = ComponentNameValidator;
        name_validator.validate(&self.name)?;

        // Validate template
        let template_validator = HtmlTemplateValidator;
        template_validator.validate(&self.template)?;

        Ok(())
    }
}

/// Divider orientation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DividerOrientation {
    Horizontal,
    Vertical,
}

/// Divider component - a visual separator
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DividerComponent {
    pub id: ComponentId,
    pub orientation: DividerOrientation,
    pub thickness: u32,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl DividerComponent {
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(),
            orientation: DividerOrientation::Horizontal,
            thickness: 1,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Default for DividerComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DividerComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Divider
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        if self.thickness == 0 {
            return Err(ValidationError::InvalidPropertyValue(
                "thickness".to_string(),
                "Divider thickness must be at least 1".to_string(),
            ));
        }
        Ok(())
    }
}

/// Checkbox component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckboxComponent {
    pub id: ComponentId,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,
    pub on_change: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl CheckboxComponent {
    pub fn new(label: String) -> Self {
        Self {
            id: ComponentId::new(),
            label,
            checked: false,
            disabled: false,
            on_change: None,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Component for CheckboxComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Checkbox
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// Radio group component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RadioGroupComponent {
    pub id: ComponentId,
    pub options: String, // Comma separated values
    pub selected: String,
    pub disabled: bool,
    pub on_change: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl RadioGroupComponent {
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(),
            options: "Option 1, Option 2, Option 3".to_string(),
            selected: String::new(),
            disabled: false,
            on_change: None,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Default for RadioGroupComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for RadioGroupComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::RadioGroup
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// Switch (toggle) component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SwitchComponent {
    pub id: ComponentId,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,
    pub on_change: Option<String>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl SwitchComponent {
    pub fn new(label: String) -> Self {
        Self {
            id: ComponentId::new(),
            label,
            checked: false,
            disabled: false,
            on_change: None,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Component for SwitchComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Switch
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// Badge variants
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeVariant {
    Default,
    Primary,
    Success,
    Warning,
    Error,
}

/// Badge component - a small status label
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BadgeComponent {
    pub id: ComponentId,
    pub text: String,
    pub variant: BadgeVariant,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl BadgeComponent {
    pub fn new(text: String) -> Self {
        Self {
            id: ComponentId::new(),
            text,
            variant: BadgeVariant::Default,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Component for BadgeComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Badge
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// Progress bar component
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgressComponent {
    pub id: ComponentId,
    pub value: f64,
    pub max: f64,
    pub show_label: bool,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub style: ComponentStyle,
}

impl ProgressComponent {
    pub fn new() -> Self {
        Self {
            id: ComponentId::new(),
            value: 50.0,
            max: 100.0,
            show_label: true,
            animation: None,
            bindings: HashMap::new(),
            style: ComponentStyle::default(),
        }
    }
}

impl Default for ProgressComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ProgressComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Progress
    }

    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn validate(&self) -> Result<(), ValidationError> {
        if self.max <= 0.0 {
            return Err(ValidationError::InvalidPropertyValue(
                "max".to_string(),
                "Progress max must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Main component enum with all variants
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CanvasComponent {
    Button(ButtonComponent),
    Text(TextComponent),
    Input(InputComponent),
    Container(ContainerComponent),
    Image(ImageComponent),
    Card(CardComponent),
    Select(SelectComponent),
    Custom(CustomComponent),
    Divider(DividerComponent),
    Checkbox(CheckboxComponent),
    RadioGroup(RadioGroupComponent),
    Switch(SwitchComponent),
    Badge(BadgeComponent),
    Progress(ProgressComponent),
}

impl CanvasComponent {
    pub fn id(&self) -> &ComponentId {
        match self {
            CanvasComponent::Button(c) => c.id(),
            CanvasComponent::Text(c) => c.id(),
            CanvasComponent::Input(c) => c.id(),
            CanvasComponent::Container(c) => c.id(),
            CanvasComponent::Image(c) => c.id(),
            CanvasComponent::Card(c) => c.id(),
            CanvasComponent::Select(c) => c.id(),
            CanvasComponent::Custom(c) => c.id(),
            CanvasComponent::Divider(c) => c.id(),
            CanvasComponent::Checkbox(c) => c.id(),
            CanvasComponent::RadioGroup(c) => c.id(),
            CanvasComponent::Switch(c) => c.id(),
            CanvasComponent::Badge(c) => c.id(),
            CanvasComponent::Progress(c) => c.id(),
        }
    }

    pub fn component_type(&self) -> ComponentType {
        match self {
            CanvasComponent::Button(c) => c.component_type(),
            CanvasComponent::Text(c) => c.component_type(),
            CanvasComponent::Input(c) => c.component_type(),
            CanvasComponent::Container(c) => c.component_type(),
            CanvasComponent::Image(c) => c.component_type(),
            CanvasComponent::Card(c) => c.component_type(),
            CanvasComponent::Select(c) => c.component_type(),
            CanvasComponent::Custom(c) => c.component_type(),
            CanvasComponent::Divider(c) => c.component_type(),
            CanvasComponent::Checkbox(c) => c.component_type(),
            CanvasComponent::RadioGroup(c) => c.component_type(),
            CanvasComponent::Switch(c) => c.component_type(),
            CanvasComponent::Badge(c) => c.component_type(),
            CanvasComponent::Progress(c) => c.component_type(),
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            CanvasComponent::Button(c) => c.validate(),
            CanvasComponent::Text(c) => c.validate(),
            CanvasComponent::Input(c) => c.validate(),
            CanvasComponent::Container(c) => c.validate(),
            CanvasComponent::Image(c) => c.validate(),
            CanvasComponent::Card(c) => c.validate(),
            CanvasComponent::Select(c) => c.validate(),
            CanvasComponent::Custom(c) => c.validate(),
            CanvasComponent::Divider(c) => c.validate(),
            CanvasComponent::Checkbox(c) => c.validate(),
            CanvasComponent::RadioGroup(c) => c.validate(),
            CanvasComponent::Switch(c) => c.validate(),
            CanvasComponent::Badge(c) => c.validate(),
            CanvasComponent::Progress(c) => c.validate(),
        }
    }

    pub fn duplicate_with_new_id(&self) -> Self {
        match self {
            CanvasComponent::Button(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Button(new_c)
            }
            CanvasComponent::Text(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Text(new_c)
            }
            CanvasComponent::Input(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Input(new_c)
            }
            CanvasComponent::Select(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Select(new_c)
            }
            CanvasComponent::Image(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Image(new_c)
            }
            CanvasComponent::Container(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                new_c.children = c
                    .children
                    .iter()
                    .map(|child| child.duplicate_with_new_id())
                    .collect();
                CanvasComponent::Container(new_c)
            }
            CanvasComponent::Card(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                new_c.children = c
                    .children
                    .iter()
                    .map(|child| child.duplicate_with_new_id())
                    .collect();
                CanvasComponent::Card(new_c)
            }
            CanvasComponent::Custom(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Custom(new_c)
            }
            CanvasComponent::Divider(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Divider(new_c)
            }
            CanvasComponent::Checkbox(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Checkbox(new_c)
            }
            CanvasComponent::RadioGroup(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::RadioGroup(new_c)
            }
            CanvasComponent::Switch(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Switch(new_c)
            }
            CanvasComponent::Badge(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Badge(new_c)
            }
            CanvasComponent::Progress(c) => {
                let mut new_c = c.clone();
                new_c.id = ComponentId::new();
                CanvasComponent::Progress(new_c)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_id_creation() {
        let id1 = ComponentId::new();
        let id2 = ComponentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_button_component_validation() {
        let button = ButtonComponent::new("Click me".to_string());
        assert!(button.validate().is_ok());

        let empty_button = ButtonComponent::new("".to_string());
        assert!(empty_button.validate().is_err());
    }

    #[test]
    fn test_custom_component_validation() {
        let valid_custom =
            CustomComponent::new("MyComponent".to_string(), "<div>Hello</div>".to_string());
        assert!(valid_custom.validate().is_ok());

        let invalid_name =
            CustomComponent::new("123Invalid".to_string(), "<div>Hello</div>".to_string());
        assert!(invalid_name.validate().is_err());

        let invalid_template =
            CustomComponent::new("ValidName".to_string(), "No tags here".to_string());
        assert!(invalid_template.validate().is_err());
    }

    #[test]
    fn test_divider_component_validation() {
        let divider = DividerComponent::new();
        assert!(divider.validate().is_ok());

        let zero_thickness = DividerComponent {
            thickness: 0,
            ..DividerComponent::new()
        };
        assert!(zero_thickness.validate().is_err());
    }

    #[test]
    fn test_progress_component_validation() {
        let progress = ProgressComponent::new();
        assert!(progress.validate().is_ok());

        let invalid_max = ProgressComponent {
            max: 0.0,
            ..ProgressComponent::new()
        };
        assert!(invalid_max.validate().is_err());
    }

    #[test]
    fn test_new_components_duplicate_with_new_id() {
        let components = vec![
            CanvasComponent::Divider(DividerComponent::new()),
            CanvasComponent::Checkbox(CheckboxComponent::new("Check".to_string())),
            CanvasComponent::RadioGroup(RadioGroupComponent::new()),
            CanvasComponent::Switch(SwitchComponent::new("Switch".to_string())),
            CanvasComponent::Badge(BadgeComponent::new("Badge".to_string())),
            CanvasComponent::Progress(ProgressComponent::new()),
        ];

        for comp in &components {
            let dup = comp.duplicate_with_new_id();
            assert_ne!(dup.id(), comp.id());
            assert_eq!(dup.component_type(), comp.component_type());
            assert!(dup.validate().is_ok());
        }
    }

    #[test]
    fn test_badge_variants() {
        let badge = BadgeComponent::new("Status".to_string());
        assert_eq!(badge.variant, BadgeVariant::Default);
        assert!(badge.validate().is_ok());
    }

    #[test]
    fn test_container_component_validation() {
        let mut container = ContainerComponent::new();
        assert!(container.validate().is_ok());

        // Add valid child
        container
            .children
            .push(CanvasComponent::Button(ButtonComponent::new(
                "Button".to_string(),
            )));
        assert!(container.validate().is_ok());

        // Add invalid child
        container
            .children
            .push(CanvasComponent::Button(ButtonComponent::new(
                "".to_string(),
            )));
        assert!(container.validate().is_err());
    }

    #[test]
    fn test_duplicate_with_new_id() {
        let mut container = ContainerComponent::new();
        let button = ButtonComponent::new("Button".to_string());
        container.children.push(CanvasComponent::Button(button));

        let original = CanvasComponent::Container(container);
        let duplicated = original.duplicate_with_new_id();

        assert_ne!(original.id(), duplicated.id());

        if let CanvasComponent::Container(orig_c) = &original
            && let CanvasComponent::Container(dup_c) = &duplicated
        {
            assert_eq!(orig_c.children.len(), dup_c.children.len());
            assert_ne!(orig_c.children[0].id(), dup_c.children[0].id());
        }
    }
}
