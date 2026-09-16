use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use super::history::{History, Snapshot};
use super::persistence::Persistable;
use super::project::Project;
use crate::builder::component_library::{LibraryComponent, builtin_library_components};
use crate::builder::design_tokens::DesignTokens;
use crate::builder::drag_drop::DragState;
use crate::domain::{CanvasComponent, ComponentId, Variable};

/// Canvas-specific state
#[derive(Clone, Copy)]
pub struct CanvasState {
    pub components: RwSignal<Vec<CanvasComponent>>,
    pub selected: RwSignal<Option<ComponentId>>,
    /// Multi-select set (Shift/Ctrl+click). The primary selection (`selected`)
    /// is always the last-clicked component.
    pub selected_components: RwSignal<Vec<ComponentId>>,
    /// Canvas zoom factor (0.25 – 4.0). 1.0 = 100%.
    pub zoom: RwSignal<f64>,
    pub history: RwSignal<History>,
    pub drag_state: RwSignal<DragState>,
}

impl CanvasState {
    pub const MIN_ZOOM: f64 = 0.25;
    pub const MAX_ZOOM: f64 = 4.0;

    pub fn new() -> Self {
        Self {
            components: RwSignal::new(Vec::new()),
            selected: RwSignal::new(None),
            selected_components: RwSignal::new(Vec::new()),
            zoom: RwSignal::new(1.0),
            history: RwSignal::new(History::new()),
            drag_state: RwSignal::new(DragState::NotDragging),
        }
    }

    /// Clear both the primary selection and the multi-select set.
    pub fn clear_selection(&self) {
        self.selected.set(None);
        self.selected_components.set(Vec::new());
    }

    /// Select a single component, replacing any multi-select.
    pub fn select_single(&self, id: ComponentId) {
        self.selected.set(Some(id));
        self.selected_components.set(vec![id]);
    }

    /// Toggle a component in the multi-select set (Shift/Ctrl+click).
    pub fn toggle_multi_select(&self, id: ComponentId) {
        self.selected_components.update(|list| {
            if let Some(pos) = list.iter().position(|existing| *existing == id) {
                list.remove(pos);
            } else {
                list.push(id);
            }
        });
        self.selected.set(Some(id));
    }

    /// Check whether a component is part of the current selection.
    pub fn is_selected(&self, id: ComponentId) -> bool {
        self.selected_components.get().contains(&id) || self.selected.get().is_some_and(|s| s == id)
    }

    /// Adjust zoom by a multiplicative factor, clamped to the allowed range.
    pub fn zoom_by(&self, factor: f64) {
        self.zoom.update(|z| {
            *z = (*z * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        });
    }

    /// Reset zoom to 100%.
    pub fn reset_zoom(&self) {
        self.zoom.set(1.0);
    }

    /// Add a component to the canvas (internal helper without snapshot)
    pub fn add_component_without_snapshot(&self, component: CanvasComponent) {
        self.components.update(|components| {
            components.push(component);
        });
    }

    /// Add a component to the canvas
    pub fn add_component(&self, component: CanvasComponent) {
        self.record_snapshot("Add Component");
        self.add_component_without_snapshot(component);
    }

    /// Add a child component to a specific parent. Returns true if successful.
    pub fn add_child_component(&self, parent_id: &ComponentId, component: CanvasComponent) -> bool {
        // Check if we can add child first (simple check: does parent exist and support children?)
        let can_add = self.components.with(|comps| {
            Self::get_recursive(comps, parent_id).is_some_and(|p| {
                matches!(p, CanvasComponent::Container(_) | CanvasComponent::Card(_))
            })
        });

        if can_add {
            self.record_snapshot("Add Child Component");
            self.add_child_component_without_snapshot(parent_id, component);
            return true;
        }
        false
    }

    pub fn add_child_component_without_snapshot(
        &self,
        parent_id: &ComponentId,
        component: CanvasComponent,
    ) -> bool {
        let mut result = false;
        self.components.update(|components| {
            result = Self::add_child_recursive(&mut components[..], parent_id, component);
        });
        result
    }

    #[allow(clippy::collapsible_match)]
    fn add_child_recursive(
        components: &mut [CanvasComponent],
        parent_id: &ComponentId,
        child: CanvasComponent,
    ) -> bool {
        for comp in components.iter_mut() {
            if comp.id() == parent_id {
                return match comp {
                    CanvasComponent::Container(container) => {
                        container.children.push(child);
                        true
                    }
                    CanvasComponent::Card(card) => {
                        card.children.push(child);
                        true
                    }
                    _ => false,
                };
            }

            // Recurse into children
            match comp {
                CanvasComponent::Container(container) => {
                    if Self::add_child_recursive(
                        &mut container.children[..],
                        parent_id,
                        child.clone(),
                    ) {
                        return true;
                    }
                }
                CanvasComponent::Card(card) => {
                    if Self::add_child_recursive(&mut card.children[..], parent_id, child.clone()) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Remove a component by ID
    pub fn remove_component(&self, id: &ComponentId) {
        self.record_snapshot("Remove Component");
        self.components.update(|components| {
            Self::remove_recursive(components, id);
        });
    }

    fn remove_recursive(components: &mut Vec<CanvasComponent>, id: &ComponentId) {
        components.retain(|c| c.id() != id);
        for comp in components.iter_mut() {
            match comp {
                CanvasComponent::Container(container) => {
                    Self::remove_recursive(&mut container.children, id);
                }
                CanvasComponent::Card(card) => {
                    Self::remove_recursive(&mut card.children, id);
                }
                _ => {}
            }
        }
    }

    /// Get a component by ID
    pub fn get_component(&self, id: &ComponentId) -> Option<CanvasComponent> {
        self.components
            .with(|components| Self::get_recursive(components, id))
    }

    fn get_recursive(components: &[CanvasComponent], id: &ComponentId) -> Option<CanvasComponent> {
        for comp in components {
            if comp.id() == id {
                return Some(comp.clone());
            }
            match comp {
                CanvasComponent::Container(container) => {
                    if let Some(found) = Self::get_recursive(&container.children, id) {
                        return Some(found);
                    }
                }
                CanvasComponent::Card(card) => {
                    if let Some(found) = Self::get_recursive(&card.children, id) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Update a component
    pub fn update_component(&self, id: &ComponentId, f: impl FnOnce(&mut CanvasComponent)) {
        self.components.update(|components| {
            Self::update_recursive(components, id, f);
        });
    }

    fn update_recursive(
        components: &mut [CanvasComponent],
        id: &ComponentId,
        f: impl FnOnce(&mut CanvasComponent),
    ) -> bool {
        #[allow(clippy::collapsible_match)]
        fn recurse(
            components: &mut [CanvasComponent],
            id: &ComponentId,
            f: &mut Option<impl FnOnce(&mut CanvasComponent)>,
        ) -> bool {
            for comp in components.iter_mut() {
                if comp.id() == id {
                    if let Some(func) = f.take() {
                        func(comp);
                    }
                    return true;
                }

                // Recurse into children
                match comp {
                    CanvasComponent::Container(c) => {
                        if recurse(&mut c.children, id, f) {
                            return true;
                        }
                    }
                    CanvasComponent::Card(c) => {
                        if recurse(&mut c.children, id, f) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }

        let mut f_opt = Some(f);
        recurse(components, id, &mut f_opt)
    }

    /// Update a component and record a snapshot
    pub fn update_component_with_snapshot(
        &self,
        id: &ComponentId,
        new_component: CanvasComponent,
        description: &str,
    ) {
        self.record_snapshot(description);
        self.update_component(id, |c| *c = new_component);
    }

    /// Move a component up within its parent container
    pub fn move_component_up(&self, id: &ComponentId) {
        self.move_component(id, -1);
    }

    /// Move a component down within its parent container
    pub fn move_component_down(&self, id: &ComponentId) {
        self.move_component(id, 1);
    }

    fn move_component(&self, id: &ComponentId, offset: i32) {
        let mut components = self.components.get();
        if Self::move_recursive(&mut components, id, offset) {
            self.record_snapshot(if offset < 0 {
                "Move Component Up"
            } else {
                "Move Component Down"
            });
            self.components.set(components);
        }
    }

    fn move_recursive(components: &mut [CanvasComponent], id: &ComponentId, offset: i32) -> bool {
        if let Some(index) = components.iter().position(|c| c.id() == id) {
            let new_index = index as i32 + offset;
            if new_index >= 0 && new_index < components.len() as i32 {
                components.swap(index, new_index as usize);
                return true;
            }
            return false;
        }

        for comp in components.iter_mut() {
            if let CanvasComponent::Container(container) = comp {
                if Self::move_recursive(&mut container.children, id, offset) {
                    return true;
                }
            } else if let CanvasComponent::Card(card) = comp
                && Self::move_recursive(&mut card.children, id, offset)
            {
                return true;
            }
        }
        false
    }

    // --- New Move Logic for Tree View ---

    pub fn move_component_relative(&self, id: ComponentId, target_id: ComponentId) {
        if id == target_id {
            return;
        }

        let mut components = self.components.get();

        // Check for descendant move to prevent infinite recursion/data loss
        // We use the cloned components for check as well
        if Self::get_recursive(&components, &id)
            .is_some_and(|parent| Self::is_descendant(&parent, &target_id))
        {
            web_sys::console::warn_1(&"Cannot move a component into its own descendant".into());
            return;
        }

        if let Some(comp) = Self::extract_recursive(&mut components, &id) {
            let mut comp_opt = Some(comp);
            if Self::insert_after_recursive(&mut components, &target_id, &mut comp_opt) {
                // Success: record snapshot of OLD state (current signal) then set new state
                self.record_snapshot("Reorder Component");
                self.components.set(components);
            } else if let Some(_c) = comp_opt {
                // Failed to insert: logic might suggest putting it back or logging warning.
                // Since we operated on a clone, the signal is untouched. We don't need to restore 'c'.
                // But we should probably warn.
                web_sys::console::warn_1(
                    &"Failed to move component to target (insert failed)".into(),
                );
                // No mutation happened, no snapshot needed.
            }
        }
    }

    // New: Move component into a parent (for drag and drop)
    pub fn move_component_to_parent(&self, id: ComponentId, parent_id: ComponentId) {
        if id == parent_id {
            return;
        }

        let mut components = self.components.get();

        if Self::get_recursive(&components, &id)
            .is_some_and(|parent| Self::is_descendant(&parent, &parent_id))
        {
            web_sys::console::warn_1(&"Cannot move a component into its own descendant".into());
            return;
        }

        if let Some(comp) = Self::extract_recursive(&mut components, &id) {
            if Self::add_child_recursive(&mut components, &parent_id, comp) {
                self.record_snapshot("Move Component Into Parent");
                self.components.set(components);
            } else {
                // Failed to add child (e.g. parent not found or not container)
                // Since it's a clone, no harm done to original state.
                web_sys::console::warn_1(&"Failed to move component into parent".into());
            }
        }
    }

    // New: Move component to root
    pub fn move_component_to_root(&self, id: ComponentId) {
        let mut components = self.components.get();

        // Optimization: Check if already at root
        if components.iter().any(|comp| *comp.id() == id) {
            return;
        }

        #[allow(clippy::collapsible_if)]
        if let Some(comp) = Self::extract_recursive(&mut components, &id) {
            components.push(comp);
            self.record_snapshot("Move Component to Root");
            self.components.set(components);
        }
    }

    fn is_descendant(parent: &CanvasComponent, target_id: &ComponentId) -> bool {
        match parent {
            CanvasComponent::Container(c) => {
                for child in &c.children {
                    if child.id() == target_id {
                        return true;
                    }
                    if Self::is_descendant(child, target_id) {
                        return true;
                    }
                }
            }
            CanvasComponent::Card(c) => {
                for child in &c.children {
                    if child.id() == target_id {
                        return true;
                    }
                    if Self::is_descendant(child, target_id) {
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn extract_recursive(
        components: &mut Vec<CanvasComponent>,
        id: &ComponentId,
    ) -> Option<CanvasComponent> {
        if let Some(pos) = components.iter().position(|c| c.id() == id) {
            return Some(components.remove(pos));
        }
        for comp in components.iter_mut() {
            match comp {
                CanvasComponent::Container(c) => {
                    if let Some(found) = Self::extract_recursive(&mut c.children, id) {
                        return Some(found);
                    }
                }
                CanvasComponent::Card(c) => {
                    if let Some(found) = Self::extract_recursive(&mut c.children, id) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    #[allow(clippy::collapsible_if, clippy::collapsible_match)]
    fn insert_after_recursive(
        components: &mut Vec<CanvasComponent>,
        target_id: &ComponentId,
        item: &mut Option<CanvasComponent>,
    ) -> bool {
        if let Some(pos) = components.iter().position(|c| c.id() == target_id) {
            if let Some(i) = item.take() {
                components.insert(pos + 1, i);
                return true;
            }
        }

        for comp in components.iter_mut() {
            match comp {
                CanvasComponent::Container(c) => {
                    if Self::insert_after_recursive(&mut c.children, target_id, item) {
                        return true;
                    }
                }
                CanvasComponent::Card(c) => {
                    if Self::insert_after_recursive(&mut c.children, target_id, item) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Record a snapshot for undo/redo
    pub fn record_snapshot(&self, description: &str) {
        let snapshot = Snapshot::new(
            self.components.get(),
            self.selected.get(),
            description.to_string(),
        );
        self.history.update(|h| h.push(snapshot));
    }

    /// Apply a snapshot to the canvas
    pub fn apply_snapshot(&self, snapshot: &Snapshot) {
        self.components.set(snapshot.components.clone());
        self.selected.set(snapshot.selected);
        // Prune multi-select entries that no longer exist
        let existing: Vec<ComponentId> = snapshot.components.iter().map(|c| *c.id()).collect();
        self.selected_components
            .update(|list| list.retain(|id| existing.contains(id)));
    }
}

impl Default for CanvasState {
    fn default() -> Self {
        Self::new()
    }
}

/// Notification types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

/// Notification message
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: Option<u32>, // milliseconds
}

impl Notification {
    pub fn success(message: String) -> Self {
        Self {
            message,
            notification_type: NotificationType::Success,
            duration: Some(3000),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            message,
            notification_type: NotificationType::Error,
            duration: Some(5000),
        }
    }

    pub fn warning(message: String) -> Self {
        Self {
            message,
            notification_type: NotificationType::Warning,
            duration: Some(4000),
        }
    }

    pub fn info(message: String) -> Self {
        Self {
            message,
            notification_type: NotificationType::Info,
            duration: Some(3000),
        }
    }
}

/// Theme options
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
    Custom,
}

/// Responsive preview modes for the canvas
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResponsiveMode {
    #[default]
    Desktop,
    Tablet,
    TabletLandscape,
    Mobile,
    MobileLandscape,
}

/// UI state (modals, panels, etc)
#[derive(Clone, Copy)]
pub struct UiState {
    pub show_command_palette: RwSignal<bool>,
    pub show_export_modal: RwSignal<bool>,
    pub show_settings_modal: RwSignal<bool>,
    pub show_shortcuts_modal: RwSignal<bool>,
    pub show_git_panel: RwSignal<bool>,
    pub show_debug_panel: RwSignal<bool>,
    pub preview_mode: RwSignal<bool>,
    pub notification: RwSignal<Option<Notification>>,
    pub responsive_mode: RwSignal<ResponsiveMode>,
    pub canvas_zoom: RwSignal<f64>,
    pub custom_components: RwSignal<Vec<LibraryComponent>>,
    pub component_library: RwSignal<Vec<LibraryComponent>>,
    pub design_tokens: RwSignal<DesignTokens>,
    pub render_count: RwSignal<u32>,
    pub render_time: RwSignal<f64>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            show_command_palette: RwSignal::new(false),
            show_export_modal: RwSignal::new(false),
            show_settings_modal: RwSignal::new(false),
            show_shortcuts_modal: RwSignal::new(false),
            show_git_panel: RwSignal::new(false),
            show_debug_panel: RwSignal::new(false),
            preview_mode: RwSignal::new(false),
            notification: RwSignal::new(None),
            responsive_mode: RwSignal::new(ResponsiveMode::default()),
            canvas_zoom: RwSignal::new(1.0),
            custom_components: RwSignal::new(Vec::new()),
            component_library: RwSignal::new(Self::default_components()),
            design_tokens: RwSignal::new(DesignTokens::default()),
            render_count: RwSignal::new(0),
            render_time: RwSignal::new(0.0),
        }
    }

    /// Get default component library
    fn default_components() -> Vec<LibraryComponent> {
        let mut components = builtin_library_components();
        components.extend_from_slice(&[
            LibraryComponent {
                name: "Div".to_string(),
                kind: "Container".to_string(),
                template: None,
                category: "Layout".to_string(),
                props_schema: None,
                description: Some("Generic div container".to_string()),
            },
            LibraryComponent {
                name: "Heading".to_string(),
                kind: "Text".to_string(),
                template: None,
                category: "Typography".to_string(),
                props_schema: None,
                description: Some("Heading text (H1-H6)".to_string()),
            },
            LibraryComponent {
                name: "Link".to_string(),
                kind: "Text".to_string(),
                template: None,
                category: "Navigation".to_string(),
                props_schema: None,
                description: Some("Hyperlink component".to_string()),
            },
        ]);
        components
    }

    /// Show a notification
    pub fn notify(&self, notification: Notification) {
        self.notification.set(Some(notification));
    }

    /// Clear notification
    pub fn clear_notification(&self) {
        self.notification.set(None);
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Export preset options
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExportPreset {
    #[default]
    Plain,
    ThawUi,
    LeptosMaterial,
    LeptosUse,
}

/// Settings state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingsState {
    pub theme: Theme,
    pub auto_save: bool,
    pub export_preset: ExportPreset,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            auto_save: true,
            export_preset: ExportPreset::default(),
        }
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

impl Persistable for SettingsState {
    fn storage_key() -> &'static str {
        "leptos_studio_settings"
    }
}

/// Persistable canvas data (for LocalStorage)
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CanvasData {
    components: Vec<CanvasComponent>,
    selected: Option<ComponentId>,
    #[serde(default)]
    variables: Vec<Variable>,
}

impl Persistable for CanvasData {
    fn storage_key() -> &'static str {
        "leptos_studio_canvas"
    }
}

/// Global application state
use crate::services::project_manager::ProjectManager;

#[derive(Clone, Copy)]
pub struct AppState {
    pub canvas: CanvasState,
    pub ui: UiState,
    pub settings: RwSignal<SettingsState>,
    pub project_name: RwSignal<String>,
    pub current_project_id: RwSignal<Option<String>>,
    pub variables: RwSignal<Vec<Variable>>,
    pub last_modified: RwSignal<f64>,
}

impl AppState {
    pub fn new() -> Self {
        // Try to load settings from LocalStorage
        let settings = SettingsState::load_or_default();

        let state = Self {
            canvas: CanvasState::new(),
            ui: UiState::new(),
            settings: RwSignal::new(settings),
            project_name: RwSignal::new("Untitled Project".to_string()),
            current_project_id: RwSignal::new(None),
            variables: RwSignal::new(Vec::new()),
            last_modified: RwSignal::new(js_sys::Date::now()),
        };

        // Setup reactivity for last_modified
        let components = state.canvas.components;
        let last_modified = state.last_modified;
        Effect::new(move |_| {
            components.track();
            last_modified.set(js_sys::Date::now());
        });

        // Attempt to load the most recent project or legacy data
        state.initialize_project_state();

        // Setup auto-save listener
        state.setup_auto_save();

        state
    }

    fn setup_auto_save(&self) {
        let state = *self;
        // Debounce auto-save
        let mut timer_handle: Option<i32> = None;

        Effect::new(move |_| {
            // Track relevant signals
            let _ = state.last_modified.get();
            let settings = state.settings.get();

            if settings.auto_save {
                // Clear previous timer if exists
                if let Some(handle) = timer_handle {
                    window().clear_timeout_with_handle(handle);
                }

                // Set new timer (2 seconds debounce)
                let handle = window()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &wasm_bindgen::closure::Closure::once_into_js(move || {
                            // Only save if we have a project ID (don't auto-save new untitled projects until first manual save)
                            if state.current_project_id.get().is_some() {
                                state.save();
                            }
                        })
                        .unchecked_into::<js_sys::Function>(),
                        2000,
                    )
                    .unwrap_or(0);
                timer_handle = Some(handle);
            }
        });
    }

    fn initialize_project_state(&self) {
        let state = *self;
        leptos::task::spawn_local(async move {
            // 1. Check legacy LocalStorage first to ensure migration happens even if backend has projects
            if let Ok(legacy) = CanvasData::load() {
                // If we have legacy data, load it as "Recovered Legacy Project"
                state.canvas.components.set(legacy.components);
                state.canvas.selected.set(legacy.selected);
                state.variables.set(legacy.variables);
                state
                    .project_name
                    .set("Recovered Legacy Project".to_string());

                // Manually save to handle success/failure explicitly
                let project = state.to_project();
                let id = ProjectManager::generate_id();
                state.current_project_id.set(Some(id.clone()));

                let ui = state.ui;
                match ProjectManager::save_project(&id, &project).await {
                    Ok(_) => {
                        // Only clear legacy storage if save succeeds to prevent data loss
                        if let Ok(Some(storage)) = window().local_storage() {
                            let _ = storage.remove_item(CanvasData::storage_key());
                        }
                        ui.notify(Notification::success(
                            "Legacy project migrated to backend".to_string(),
                        ));
                    }
                    Err(e) => {
                        ui.notify(Notification::error(format!(
                            "Migration failed: {}. Legacy data preserved locally.",
                            e.user_message()
                        )));
                    }
                }
                return;
            }

            // 2. If no legacy data, check backend projects
            if let Ok(projects) = ProjectManager::list_projects().await
                && let Some(latest) = projects.first()
            {
                // Load the latest project
                if let Ok(project) = ProjectManager::load_project(&latest.id).await {
                    state.apply_project(project);
                    state.current_project_id.set(Some(latest.id.clone()));
                }
            }
        });
    }

    /// Provide AppState as context
    pub fn provide_context() {
        let state = Self::new();
        provide_context(state);
    }

    /// Expect AppState from context (panics if missing)
    pub fn expect_context() -> Self {
        expect_context::<Self>()
    }

    /// Try to get AppState from context
    pub fn try_use_context() -> Option<Self> {
        use_context::<Self>()
    }

    /// Save settings to LocalStorage
    pub fn save_settings(&self) {
        if let Err(e) = self.settings.get().save() {
            web_sys::console::error_1(&format!("Failed to save settings: {}", e).into());
        }
    }

    /// Save project to Backend (creates new if no ID)
    pub fn save(&self) {
        let project = self.to_project();
        // Optimistically set ID if None to prevent duplicate saves (race condition)
        let id = self.current_project_id.get().unwrap_or_else(|| {
            let new_id = ProjectManager::generate_id();
            self.current_project_id.set(Some(new_id.clone()));
            new_id
        });

        let ui = self.ui;
        // We capture id by value for the async block

        leptos::task::spawn_local(async move {
            match ProjectManager::save_project(&id, &project).await {
                Ok(_) => {
                    ui.notify(Notification::success(
                        "Project saved successfully".to_string(),
                    ));
                }
                Err(e) => {
                    ui.notify(Notification::error(e.user_message()));
                }
            }
        });
    }

    /// Load project by ID
    pub fn load_project(&self, id: &str) {
        let id = id.to_string();
        let state = *self;

        leptos::task::spawn_local(async move {
            match ProjectManager::load_project(&id).await {
                Ok(project) => {
                    state.apply_project(project);
                    state.current_project_id.set(Some(id));
                    state
                        .ui
                        .notify(Notification::success("Project loaded".to_string()));
                }
                Err(e) => {
                    state.ui.notify(Notification::error(e.user_message()));
                }
            }
        });
    }

    /// Create a new empty project
    pub fn create_new_project(&self) {
        self.project_name.set("Untitled Project".to_string());
        self.canvas.components.set(Vec::new());
        self.canvas.selected.set(None);
        self.canvas.history.update(|h| h.clear());
        self.variables.set(Vec::new());
        self.current_project_id.set(None);
        self.update_last_modified();
    }

    /// Load canvas data from LocalStorage (Legacy / Import)
    pub fn load(&self) -> Result<(), crate::domain::AppError> {
        // This is now an "Import from Legacy" or generic load.
        // For now, let's keep it wrapper for load_project if we had an active ID,
        // but if not, it tries legacy.
        if let Some(id) = self.current_project_id.get() {
            self.load_project(&id);
            return Ok(());
        }

        let data = CanvasData::load()?;
        self.canvas.components.set(data.components);
        self.canvas.selected.set(data.selected);
        self.variables.set(data.variables);
        Ok(())
    }

    /// Build a Project from current state
    pub fn to_project(&self) -> Project {
        Project::new(
            self.project_name.get(),
            self.canvas.components.get(),
            self.settings.get(),
            self.ui.design_tokens.get(),
            self.variables.get(),
        )
    }

    /// Apply a Project to the current state
    pub fn apply_project(&self, project: Project) {
        self.project_name.set(project.name);
        self.canvas.components.set(project.layout);
        self.canvas.selected.set(None);
        self.canvas.history.update(|h| h.clear());
        self.settings.set(project.settings);
        self.ui.design_tokens.set(project.design_tokens);
        self.variables.set(project.variables);
        self.update_last_modified();
    }

    fn update_last_modified(&self) {
        self.last_modified.set(js_sys::Date::now());
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
