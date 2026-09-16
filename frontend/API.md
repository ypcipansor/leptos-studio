# Leptos Studio – API Documentation

Complete API reference for Leptos Studio modules and components.

## Table of Contents

1. [State Management](#state-management)
2. [Domain Types](#domain-types)
3. [Services](#services)
4. [Builder Components](#builder-components)
5. [Styling & Layout](#styling--layout)
6. [Utilities](#utilities)

---

## State Management

### AppState

**Location**: `src/state/app_state.rs`

The root application state containing canvas, UI, and settings state.

```rust
pub struct AppState {
    pub canvas: CanvasState,
    pub ui: UiState,
    pub settings: SettingsState,
}

impl AppState {
    pub fn use_context() -> AppState;
    pub fn provide_context();
    pub fn save() -> Result<(), AppError>;
    pub fn load() -> Result<(), AppError>;
    pub fn apply_project(project: Project);
    pub fn take_snapshot() -> Snapshot;
}
```

### CanvasState

Canvas-specific state for components and editing.

```rust
pub struct CanvasState {
    pub components: RwSignal<Vec<CanvasComponent>>,
    pub selected: RwSignal<Option<ComponentId>>,
    pub history: RwSignal<History>,
    pub drag_state: RwSignal<DragState>,
}

impl CanvasState {
    pub fn add_component(&self, component: CanvasComponent);
    pub fn remove_component(&self, id: &ComponentId);
    pub fn get_component(&self, id: &ComponentId) -> Option<CanvasComponent>;
    pub fn update_component(&self, id: &ComponentId, new_component: CanvasComponent);
    pub fn record_snapshot(&self);
    pub fn apply_snapshot(&self, snapshot: &Snapshot);
}
```

### UiState

UI visibility and settings state.

```rust
pub struct UiState {
    pub show_command_palette: RwSignal<bool>,
    pub show_export_modal: RwSignal<bool>,
    pub show_git_panel: RwSignal<bool>,
    pub show_debug_panel: RwSignal<bool>,
    pub notification: RwSignal<Option<Notification>>,
    pub responsive_mode: RwSignal<ResponsiveMode>,
    pub custom_components: RwSignal<Vec<LibraryComponent>>,
    pub component_library: RwSignal<Vec<LibraryComponent>>,
    pub render_count: RwSignal<u32>,
    pub render_time: RwSignal<f64>,
}
```

### Notification

User notification/alert system.

```rust
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: Option<u32>,  // milliseconds
}

impl Notification {
    pub fn success(message: String) -> Self;
    pub fn error(message: String) -> Self;
    pub fn warning(message: String) -> Self;
    pub fn info(message: String) -> Self;
}

pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}
```

---

## Domain Types

### CanvasComponent

**Location**: `src/domain/component.rs`

Union type for all possible components on the canvas.

```rust
pub enum CanvasComponent {
    Button(ButtonComponent),
    Text(TextComponent),
    Input(InputComponent),
    Container(ContainerComponent),
    Custom(CustomComponent),
}

impl CanvasComponent {
    pub fn id(&self) -> &ComponentId;
    pub fn kind(&self) -> &str;
}
```

### ButtonComponent

```rust
pub struct ButtonComponent {
    pub id: ComponentId,
    pub label: String,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub disabled: bool,
    pub on_click: Option<String>,
}

pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
}

pub enum ButtonSize {
    Small,
    Medium,
    Large,
}
```

### TextComponent

```rust
pub struct TextComponent {
    pub id: ComponentId,
    pub content: String,
    pub style: TextStyle,
    pub tag: TextTag,
}

pub enum TextStyle {
    Normal,
    Bold,
    Italic,
    Code,
}

pub enum TextTag {
    H1, H2, H3, H4, H5, H6,
    Paragraph,
    Span,
}
```

### InputComponent

```rust
pub struct InputComponent {
    pub id: ComponentId,
    pub placeholder: String,
    pub input_type: InputType,
    pub required: bool,
    pub value: String,
}

pub enum InputType {
    Text,
    Email,
    Password,
    Number,
    Textarea,
    Checkbox,
    Radio,
}
```

### ContainerComponent

```rust
pub struct ContainerComponent {
    pub id: ComponentId,
    pub children: Vec<ComponentId>,
    pub layout: LayoutType,
    pub gap: u32,
}

pub enum LayoutType {
    Row,
    Column,
    Grid,
}
```

### CustomComponent

```rust
pub struct CustomComponent {
    pub id: ComponentId,
    pub name: String,
    pub template: String,
    pub properties: HashMap<String, PropValue>,
}

pub enum PropValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}
```

---

## Services

### Export Service

**Location**: `src/services/export_service.rs`

Code generation from canvas components.

```rust
pub trait CodeGenerator {
    fn generate(&self, components: &[CanvasComponent]) -> Result<String, AppError>;
}

pub struct LeptosCodeGenerator {
    preset: ExportPreset,
}

pub struct HtmlCodeGenerator;
pub struct MarkdownCodeGenerator;

pub enum ExportPreset {
    Plain,
    WithStyling,
    WithComments,
}
```

**Example**:

```rust
let generator = LeptosCodeGenerator::new(ExportPreset::Plain);
match generator.generate(&components) {
    Ok(code) => println!("{}", code),
    Err(e) => eprintln!("Error: {}", e.user_message()),
}
```

### Property Service

**Location**: `src/services/property_service.rs`

Property update functions for components.

```rust
pub fn update_button_prop(
    mut component: ButtonComponent,
    prop_name: &str,
    value: PropValue,
) -> Result<ButtonComponent, AppError>;

pub fn update_text_prop(
    mut component: TextComponent,
    prop_name: &str,
    value: PropValue,
) -> Result<TextComponent, AppError>;

pub fn update_input_prop(
    mut component: InputComponent,
    prop_name: &str,
    value: PropValue,
) -> Result<InputComponent, AppError>;
```

### Project Service

**Location**: `src/services/project_service.rs`

Project serialization/deserialization.

```rust
pub fn project_to_json(project: &Project) -> Result<String, AppError>;
pub fn project_from_json(json: &str) -> Result<Project, AppError>;

pub struct Project {
    pub canvas_components: Vec<CanvasComponent>,
    pub metadata: ProjectMetadata,
}
```

### Git Service

**Location**: `src/services/git_service.rs`

Git backend abstraction.

```rust
pub trait GitBackend {
    fn status(&self) -> Result<GitStatus, AppError>;
    fn log(&self) -> Result<Vec<GitCommit>, AppError>;
    fn commit(&self, message: &str) -> Result<(), AppError>;
    fn push(&self) -> Result<(), AppError>;
}

pub struct NoopGitBackend;  // Default no-op implementation
```

### Event Bus Service 🆕

**Location**: `src/services/event_bus.rs`

Centralized event system for decoupled component communication.

```rust
pub enum AppEvent {
    ComponentAdded(ComponentId),
    ComponentRemoved(ComponentId),
    ComponentUpdated(ComponentId),
    SelectionChanged(Option<ComponentId>),
    LayoutChanged,
    UndoPerformed,
    RedoPerformed,
    ProjectSaved,
    ProjectLoaded,
    ExportCompleted(String),
    ThemeChanged(Theme),
    Custom(String, String),
}

pub struct EventBus {
    subscribers: RwSignal<Vec<StoredValue<Box<dyn Fn(&AppEvent)>>>>,
}

impl EventBus {
    pub fn new() -> Self;
    pub fn subscribe(&self, callback: impl Fn(&AppEvent) + 'static);
    pub fn emit(&self, event: AppEvent);
}
```

### Template Service 🆕

**Location**: `src/services/template_service.rs`

Pre-built layout templates for quick-start designs.

```rust
pub enum TemplateCategory {
    LandingPage,
    Dashboard,
    Form,
    Navigation,
    Card,
    Hero,
    Footer,
    Custom,
}

pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub thumbnail: Option<String>,
    pub components: Vec<CanvasComponent>,
    pub tags: Vec<String>,
}

pub struct TemplateService;

impl TemplateService {
    pub fn new() -> Self;
    pub fn builtin_templates() -> Vec<Template>;
    pub fn templates_by_category(category: TemplateCategory) -> Vec<Template>;
    pub fn search_templates(query: &str) -> Vec<Template>;
}
```

**Available Templates**:
- Login Form
- Contact Form
- Hero Section
- Pricing Card
- Navigation Bar
- Footer
- Dashboard Header
- Feature Grid

### Analytics Service 🆕

**Location**: `src/services/analytics_service.rs`

Usage analytics and performance metrics collection.

```rust
pub struct UsageMetrics {
    pub components_added: u64,
    pub components_removed: u64,
    pub components_updated: u64,
    pub exports_count: u64,
    pub saves_count: u64,
    pub loads_count: u64,
    pub undo_count: u64,
    pub redo_count: u64,
}

pub struct PerformanceMetrics {
    pub render_times: Vec<f64>,
    pub average_render_time: f64,
    pub peak_render_time: f64,
}

pub struct AnalyticsService {
    pub session: RwSignal<SessionInfo>,
    pub usage: RwSignal<UsageMetrics>,
    pub performance: RwSignal<PerformanceMetrics>,
}

impl AnalyticsService {
    pub fn new() -> Self;
    pub fn track_event(&self, event: &AppEvent);
    pub fn record_render_time(&self, ms: f64);
    pub fn get_session_duration(&self) -> Duration;
    pub fn export_metrics(&self) -> String;
}
```

### Advanced Export Service 🆕

**Location**: `src/services/export_advanced.rs`

Extended code generation formats.

```rust
/// JSON Schema generator
#[derive(Default)]
pub struct JsonSchemaGenerator;

/// TypeScript types generator
#[derive(Default)]
pub struct TypeScriptGenerator;

/// React/JSX component generator
#[derive(Default)]
pub struct ReactGenerator;

/// HTML with Tailwind CSS classes
#[derive(Default)]
pub struct TailwindHtmlGenerator;

/// Svelte component generator
#[derive(Default)]
pub struct SvelteGenerator;

/// Vue component generator
pub struct VueGenerator;

/// CSS extraction generator
pub struct CssGenerator;
```

**All generators implement the `CodeGenerator` trait**:

```rust
impl CodeGenerator for JsonSchemaGenerator {
    fn generate(&self, components: &[CanvasComponent]) -> AppResult<String>;
    fn file_extension(&self) -> &str; // "json"
}
```

**Example Usage**:

```rust
use crate::services::export_advanced::*;

// Generate TypeScript types
let ts_gen = TypeScriptGenerator::default();
let types = ts_gen.generate(&components)?;

// Generate React component
let react_gen = ReactGenerator::default();
let jsx = react_gen.generate(&components)?;

// Generate Tailwind HTML
let tw_gen = TailwindHtmlGenerator::default();
let html = tw_gen.generate(&components)?;
```

---

## Derived State 🆕

**Location**: `src/state/derived.rs`

Memoized computed values for efficient reactive updates.

```rust
pub struct DerivedState {
    pub component_count: Memo<usize>,
    pub type_counts: Memo<TypeCounts>,
    pub is_empty: Memo<bool>,
    pub has_selection: Memo<bool>,
    pub selected_component: Memo<Option<CanvasComponent>>,
    pub max_nesting_depth: Memo<usize>,
    pub custom_count: Memo<usize>,
    pub can_undo: Memo<bool>,
    pub can_redo: Memo<bool>,
}

pub struct TypeCounts {
    pub buttons: usize,
    pub texts: usize,
    pub inputs: usize,
    pub containers: usize,
    pub customs: usize,
}

impl DerivedState {
    pub fn new(app_state: AppState) -> Self;
    pub fn provide_context(app_state: AppState);
    pub fn use_context() -> Self;
}
```

---

## Builder Components

### Canvas

**Location**: `src/builder/canvas/mod.rs`

Main editing canvas component.

```rust
#[component]
pub fn Canvas() -> impl IntoView;
```

### Sidebar

**Location**: `src/builder/sidebar.rs`

Component library and custom component editor.

```rust
#[component]
pub fn Sidebar() -> impl IntoView;
```

### PropertyEditor

**Location**: `src/builder/property_editor.rs`

Schema-driven property editor for selected components.

```rust
#[component]
pub fn PropertyEditor() -> impl IntoView;
```

### Preview

**Location**: `src/builder/preview.rs`

Live preview of selected component.

```rust
#[component]
pub fn Preview() -> impl IntoView;
```

### StatusBar 🆕

**Location**: `src/builder/status_bar.rs`

Bottom status bar showing application state.

```rust
#[component]
pub fn StatusBar() -> impl IntoView;

#[component]
pub fn StatusIndicator(
    label: String,
    value: Signal<String>,
    icon: Option<&'static str>,
) -> impl IntoView;
```

**Displays**:
- Component count
- Nesting depth
- Selection status
- Undo/Redo buttons
- Responsive mode
- Theme indicator
- Render performance

### TemplateGallery 🆕

**Location**: `src/builder/template_gallery.rs`

Visual template browser and applicator.

```rust
#[component]
pub fn TemplateGallery(
    on_close: Callback<()>,
    on_apply: Callback<Template>,
) -> impl IntoView;

#[component]
pub fn TemplateGalleryToggle() -> impl IntoView;
```

### ComponentPalette 🆕

**Location**: `src/builder/component_palette.rs`

Enhanced component browser with search.

```rust
#[component]
pub fn ComponentPalette() -> impl IntoView;
```

**Features**:
- Fuzzy search
- Category tabs
- Keyboard navigation
- Component favorites

### Accessibility Components 🆕

**Location**: `src/builder/accessibility.rs`

Accessibility utilities and components.

```rust
/// Wraps app with accessibility providers
#[component]
pub fn AccessibilityProvider(children: Children) -> impl IntoView;

/// Skip link for keyboard navigation
#[component]
pub fn SkipLink(
    target: String,
    label: Option<String>,
) -> impl IntoView;

/// Screen reader live region
#[component]
pub fn LiveRegion() -> impl IntoView;

/// Focus trap for modals
#[component]
pub fn FocusTrap(
    active: Signal<bool>,
    children: Children,
) -> impl IntoView;

/// Accessible progress bar
#[component]
pub fn ProgressBar(
    value: Signal<f64>,
    label: String,
    show_value: bool,
) -> impl IntoView;

/// Tooltip with accessible descriptions
#[component]
pub fn Tooltip(
    content: String,
    children: Children,
) -> impl IntoView;

/// Screen reader only text
#[component]
pub fn VisuallyHidden(children: Children) -> impl IntoView;

/// Keyboard shortcut hint
#[component]
pub fn KeyboardHint(shortcut: String) -> impl IntoView;

/// Announce message to screen readers
pub fn announce(message: &str);
pub fn announce_assertive(message: &str);
```

### BreakpointEditor 🆕

**Location**: `src/builder/breakpoint_editor.rs`

Custom responsive breakpoint configuration.

```rust
#[derive(Clone)]
pub struct CustomBreakpoint {
    pub id: String,
    pub name: String,
    pub min_width: u32,
    pub max_width: Option<u32>,
    pub icon: String,
}

#[component]
pub fn BreakpointEditor() -> impl IntoView;

#[component]
pub fn BreakpointPreview(breakpoints: Signal<Vec<CustomBreakpoint>>) -> impl IntoView;

#[component]
pub fn ResponsiveWrapper(
    breakpoints: Signal<Vec<CustomBreakpoint>>,
    children: Children,
) -> impl IntoView;
```

### ResponsivePreviewControls

**Location**: `src/builder/responsive_preview.rs`

Viewport size selector buttons.

```rust
#[component]
pub fn ResponsivePreviewControls() -> impl IntoView;

#[component]
pub fn CanvasViewport(children: Children) -> impl IntoView;

#[component]
pub fn ResponsiveIndicator() -> impl IntoView;
```

### CommandPalette

**Location**: `src/builder/command_palette.rs`

Searchable command palette (VS Code style).

```rust
#[component]
pub fn CommandPalette(
    is_open: Signal<bool>,
    close: WriteSignal<bool>,
    search: RwSignal<String>,
    on_action: Callback<KeyboardAction>,
) -> impl IntoView;
```

### DebugPanel

**Location**: `src/builder/debug_panel.rs`

Debug information display.

```rust
#[component]
pub fn DebugPanel() -> impl IntoView;
```

### GitPanel

**Location**: `src/builder/git_panel.rs`

Git operations interface.

```rust
#[component]
pub fn GitPanel() -> impl IntoView;
```

### Snackbar

**Location**: `src/builder/snackbar.rs`

Notification display component.

```rust
#[component]
pub fn Snackbar(notification: Signal<Option<Notification>>) -> impl IntoView;
```

---

## Styling & Layout

### ComponentStyle

**Location**: `src/builder/styling_system.rs`

Component styling properties.

```rust
pub struct ComponentStyle {
    pub padding: Option<String>,
    pub margin: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: Option<u32>,
    pub border_radius: Option<u32>,
    pub font_size: Option<u32>,
    pub font_weight: Option<String>,
    pub text_align: Option<String>,
    pub display: Option<String>,
    pub flex_direction: Option<String>,
    pub gap: Option<String>,
    pub custom_css: Option<String>,
}

impl ComponentStyle {
    pub fn to_css_string(&self) -> String;
    pub fn to_class_string(&self) -> Option<String>;
}
```

### ThemePreset

Theme presets for quick styling.

```rust
pub enum ThemePreset {
    Light,
    Dark,
    HighContrast,
    Colorful,
    Minimal,
}

impl ThemePreset {
    pub fn get_button_style(&self) -> ComponentStyle;
    pub fn get_container_style(&self) -> ComponentStyle;
}
```

### SizeConstraints

**Location**: `src/builder/component_constraints.rs`

Component size validation.

```rust
pub struct SizeConstraints {
    pub min_width: Option<u32>,
    pub max_width: Option<u32>,
    pub min_height: Option<u32>,
    pub max_height: Option<u32>,
    pub aspect_ratio: Option<f32>,
}

impl SizeConstraints {
    pub fn validate(&self, width: u32, height: u32) -> Result<(), String>;
    pub fn for_button() -> Self;
    pub fn for_input() -> Self;
    pub fn for_container() -> Self;
    pub fn for_text() -> Self;
}
```

### AlignmentOption

Flexbox alignment presets.

```rust
pub enum AlignmentOption {
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl AlignmentOption {
    pub fn as_css(&self) -> &'static str;
    pub fn as_label(&self) -> &'static str;
}
```

---

## Utilities

### Clipboard

**Location**: `src/utils/clipboard.rs`

Clipboard operations.

```rust
pub async fn copy_to_clipboard(text: &str) -> Result<(), AppError>;
pub async fn read_from_clipboard() -> Result<String, AppError>;
```

### Format

**Location**: `src/utils/format.rs`

String and number formatting.

```rust
pub fn format_file_size(bytes: u64) -> String;
pub fn format_timestamp(timestamp: f64) -> String;
pub fn truncate_string(s: &str, max_len: usize) -> String;
```

### DOM

**Location**: `src/utils/dom.rs`

DOM manipulation helpers.

```rust
pub fn get_element_by_id(id: &str) -> Option<HtmlElement>;
pub fn scroll_to_element(element: &HtmlElement);
```

---

## Validation

### Validators

**Location**: `src/domain/validation.rs`

Validation trait and implementations.

```rust
pub trait Validator<T> {
    fn validate(&self, value: &T) -> Result<(), ValidationError>;
}

pub struct ComponentNameValidator;
pub struct HtmlTemplateValidator;

pub enum ValidationError {
    EmptyName,
    InvalidName(String),
    EmptyTemplate,
    InvalidTemplate(String),
    // ...
}
```

---

## Error Handling

### AppError

**Location**: `src/domain/error.rs`

Application error type.

```rust
pub enum AppError {
    NotFound(String),
    InvalidOperation(String),
    ValidationError(ValidationError),
    SerializationError(String),
    ClipboardError(String),
    // ...
}

impl AppError {
    pub fn user_message(&self) -> String;
}
```

---

## Examples

### Add Component to Canvas

```rust
let app_state = AppState::use_context();

let new_button = ButtonComponent {
    id: ComponentId::new(),
    label: "Click me".to_string(),
    variant: ButtonVariant::Primary,
    size: ButtonSize::Medium,
    disabled: false,
    on_click: None,
};

let component = CanvasComponent::Button(new_button);
app_state.canvas.add_component(component);
app_state.canvas.record_snapshot();
```

### Update Component Property

```rust
use crate::services::property_service::update_button_prop;
use crate::domain::component::PropValue;

if let Some(CanvasComponent::Button(mut btn)) = app_state.canvas.get_component(&id) {
    match update_button_prop(btn, "label", PropValue::String("New Text".to_string())) {
        Ok(updated_btn) => {
            app_state.canvas.update_component(&id, CanvasComponent::Button(updated_btn));
        }
        Err(e) => {
            app_state.ui.notification.set(Some(Notification::error(e.user_message())));
        }
    }
}
```

### Export Code

```rust
use crate::services::export_service::LeptosCodeGenerator;

let components = app_state.canvas.components.get();
let generator = LeptosCodeGenerator::new(ExportPreset::Plain);

match generator.generate(&components) {
    Ok(code) => {
        // Copy to clipboard or download
        copy_to_clipboard(&code).await.ok();
    }
    Err(e) => {
        app_state.ui.notification.set(Some(Notification::error(e.user_message())));
    }
}
```

---

## See Also

- **[QUICKSTART.md](./QUICKSTART.md)** - Task-oriented walkthrough
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Architecture overview
- **[DEVELOPMENT.md](./DEVELOPMENT.md)** - Development guide
- **[Leptos Docs](https://leptos.dev)** - Leptos framework documentation
