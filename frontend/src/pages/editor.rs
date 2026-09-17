use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::builder::accessibility::{AccessibilityProvider, SkipLink, announce};
use crate::builder::breadcrumb::BreadcrumbNavigation;
use crate::builder::canvas::Canvas;
use crate::builder::code_panel::CodePanel;
use crate::builder::command_palette::CommandPalette;
use crate::builder::component_palette::ComponentPalette;
use crate::builder::debug_panel::DebugPanel;
use crate::builder::design_tokens::DesignTokenProvider;
use crate::builder::drag_drop::DragPreview;
use crate::builder::export_modal::ExportModal;
use crate::builder::git_panel::GitPanel;
use crate::builder::history_panel::HistoryPanel;
use crate::builder::hooks::use_keyboard_actions::use_keyboard_actions;
use crate::builder::hooks::use_resize::use_resizable_sidebar;
use crate::builder::keyboard::{KeyboardHandler, get_default_shortcuts};
use crate::builder::preview::Preview;
use crate::builder::property_editor::PropertyEditor;
use crate::builder::responsive_preview::{CanvasViewport, ResponsivePreviewControls};
use crate::builder::save_template_modal::SaveTemplateModal;
use crate::builder::settings_modal::SettingsModal;
use crate::builder::shortcuts_modal::ShortcutsModal;
use crate::builder::snackbar::Snackbar;
use crate::builder::status_bar::StatusBar;
use crate::builder::template_gallery::TemplateGallery;
use crate::builder::theme_editor::ThemeEditor;
use crate::builder::toolbar::Toolbar;
use crate::builder::tree_view::TreeView;
use crate::constants::{
    DEFAULT_LEFT_SIDEBAR_WIDTH, DEFAULT_RIGHT_SIDEBAR_WIDTH, STORAGE_KEY_LEFT_SIDEBAR_WIDTH,
    STORAGE_KEY_RIGHT_SIDEBAR_WIDTH,
};
use crate::state::app_state::{AppState, Notification};

#[component]
pub fn EditorPage() -> impl IntoView {
    let app_state = AppState::expect_context();
    let params = use_params_map();

    // Load project on mount/param change
    Effect::new(move |_| {
        let params = params.get();
        if let Some(id) = params.get("id") {
            // Only load if it's different or we haven't loaded anything
            if app_state.current_project_id.get().as_deref() != Some(&id) {
                app_state.load_project(&id);
            }
        } else {
            // If no ID (shouldn't happen with correct routing), maybe new project?
            // But EditorPage is bound to /editor/:id
        }
    });

    // Initialize UI signals (similar to App.rs)
    let show_export = RwSignal::new(false);
    let export_code = RwSignal::new(String::new());
    let export_template = RwSignal::new("leptos".to_string());
    let show_template_gallery = RwSignal::new(false);
    let show_save_template = RwSignal::new(false);

    let show_left_sidebar_mobile = RwSignal::new(false);

    let keyboard_action_handler = use_keyboard_actions(
        show_export.write_only(),
        export_code.write_only(),
        show_template_gallery.write_only(),
    );

    let left_sidebar = use_resizable_sidebar(
        DEFAULT_LEFT_SIDEBAR_WIDTH,
        STORAGE_KEY_LEFT_SIDEBAR_WIDTH,
        true,
    );
    let right_sidebar = use_resizable_sidebar(
        DEFAULT_RIGHT_SIDEBAR_WIDTH,
        STORAGE_KEY_RIGHT_SIDEBAR_WIDTH,
        false,
    );

    // Global resize cursor handler
    Effect::new(move |_| {
        if left_sidebar.is_dragging.get() || right_sidebar.is_dragging.get() {
            let _ = document()
                .body()
                .expect("body")
                .style()
                .set_property("cursor", "col-resize");
        } else {
            let _ = document()
                .body()
                .expect("body")
                .style()
                .remove_property("cursor");
        }
    });

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum RightPanelTab {
        Properties,
        Git,
        Code,
        History,
        Debug,
    }

    let active_right_tab = RwSignal::new(RightPanelTab::Properties);

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum LeftPanelTab {
        Add,
        Layers,
        Theme,
        Variables,
    }

    let active_left_tab = RwSignal::new(LeftPanelTab::Add);

    // Any modal/dialog that overlays the editor. While one is open the canvas is
    // not the user's target, so the global shortcuts must stay dormant.
    let modal_open = Signal::derive(move || {
        app_state.ui.show_command_palette.get()
            || app_state.ui.show_export_modal.get()
            || app_state.ui.show_settings_modal.get()
            || app_state.ui.show_shortcuts_modal.get()
            || show_template_gallery.get()
            || show_save_template.get()
    });

    view! {
        <DesignTokenProvider tokens=app_state.ui.design_tokens>
            <AccessibilityProvider>
                <SkipLink target="#main-canvas" label="Skip to canvas" />
                <div class="leptos-studio editor-layout" tabindex="0" role="application" aria-label="Leptos Studio Visual Builder">
                    <KeyboardHandler
                        shortcuts=get_default_shortcuts()
                        modal_open=modal_open
                        on_action=keyboard_action_handler.clone()
                    />

                    <DragPreview drag_state=app_state.canvas.drag_state />

                    <CommandPalette
                        is_open=app_state.ui.show_command_palette.read_only()
                        close=app_state.ui.show_command_palette.write_only()
                        search=RwSignal::new(String::new())
                        on_action=keyboard_action_handler
                    />

                    <Toolbar
                        show_template_gallery=show_template_gallery.write_only()
                        show_export=show_export.write_only()
                        show_save_template=show_save_template.write_only()
                        export_code=export_code.write_only()
                        export_template=export_template.read_only()
                    />

                    <BreadcrumbNavigation />

                    // Mobile Sidebar Toggle
                    <button
                        class="mobile-sidebar-toggle"
                        on:click=move |_| show_left_sidebar_mobile.update(|v| *v = !*v)
                    >
                        {move || if show_left_sidebar_mobile.get() { "✕ Close" } else { "☰ Menu" }}
                    </button>

                    <div class="app-layout">
                        <aside
                            class=move || if show_left_sidebar_mobile.get() { "sidebar-panel mobile-visible" } else { "sidebar-panel" }
                            role="navigation"
                            aria-label="Component library"
                            style=move || format!("width: {}px", left_sidebar.width.get())
                        >
                            <div class="panel-tabs">
                                <button
                                    class=move || if active_left_tab.get() == LeftPanelTab::Add { "tab active" } else { "tab" }
                                    on:click=move |_| active_left_tab.set(LeftPanelTab::Add)
                                >
                                    "Add"
                                </button>
                                <button
                                    class=move || if active_left_tab.get() == LeftPanelTab::Layers { "tab active" } else { "tab" }
                                    on:click=move |_| active_left_tab.set(LeftPanelTab::Layers)
                                >
                                    "Layers"
                                </button>
                                <button
                                    class=move || if active_left_tab.get() == LeftPanelTab::Theme { "tab active" } else { "tab" }
                                    on:click=move |_| active_left_tab.set(LeftPanelTab::Theme)
                                >
                                    "Theme"
                                </button>
                                <button
                                    class=move || if active_left_tab.get() == LeftPanelTab::Variables { "tab active" } else { "tab" }
                                    on:click=move |_| active_left_tab.set(LeftPanelTab::Variables)
                                >
                                    "Vars"
                                </button>
                            </div>
                            <div class="panel-content">
                                {move || match active_left_tab.get() {
                                    LeftPanelTab::Add => view! { <ComponentPalette /> }.into_any(),
                                    LeftPanelTab::Layers => view! { <TreeView /> }.into_any(),
                                    LeftPanelTab::Theme => view! { <ThemeEditor tokens=app_state.ui.design_tokens /> }.into_any(),
                                    LeftPanelTab::Variables => view! { <crate::builder::variable_panel::VariablePanel /> }.into_any(),
                                }}
                            </div>
                        </aside>

                        <div
                            class=move || if left_sidebar.is_dragging.get() { "resize-handle active" } else { "resize-handle" }
                            on:mousedown=move |ev| left_sidebar.start_drag.run(ev)
                        />

                        <main role="main">
                            <nav class="main-nav" aria-label="Main actions">
                                <ResponsivePreviewControls />
                            </nav>
                            <div class="main-content">
                                <section id="main-canvas" class="canvas-area" role="region" aria-label="Design canvas">
                                    <CanvasViewport>
                                        <Canvas />
                                    </CanvasViewport>
                                </section>

                                <div
                                    class=move || if right_sidebar.is_dragging.get() { "resize-handle active" } else { "resize-handle" }
                                    on:mousedown=move |ev| right_sidebar.start_drag.run(ev)
                                />

                                <aside
                                    class="property-panel"
                                    role="complementary"
                                    aria-label="Right Panel"
                                    style=move || format!("width: {}px", right_sidebar.width.get())
                                >
                                    <div class="panel-tabs">
                                        <button
                                            class=move || if active_right_tab.get() == RightPanelTab::Properties { "tab active" } else { "tab" }
                                            on:click=move |_| active_right_tab.set(RightPanelTab::Properties)
                                        >
                                            "Properties"
                                        </button>
                                        <button
                                            class=move || if active_right_tab.get() == RightPanelTab::Code { "tab active" } else { "tab" }
                                            on:click=move |_| active_right_tab.set(RightPanelTab::Code)
                                        >
                                            "Code"
                                        </button>
                                        <button
                                            class=move || if active_right_tab.get() == RightPanelTab::History { "tab active" } else { "tab" }
                                            on:click=move |_| active_right_tab.set(RightPanelTab::History)
                                        >
                                            "History"
                                        </button>
                                        <button
                                            class=move || if active_right_tab.get() == RightPanelTab::Git { "tab active" } else { "tab" }
                                            on:click=move |_| active_right_tab.set(RightPanelTab::Git)
                                        >
                                            "Git"
                                        </button>
                                        <button
                                            class=move || if active_right_tab.get() == RightPanelTab::Debug { "tab active" } else { "tab" }
                                            on:click=move |_| active_right_tab.set(RightPanelTab::Debug)
                                        >
                                            "Debug"
                                        </button>
                                    </div>

                                    <div class="panel-content">
                                        {move || match active_right_tab.get() {
                                            RightPanelTab::Debug => view! { <DebugPanel /> }.into_any(),
                                            RightPanelTab::Git => view! { <GitPanel /> }.into_any(),
                                            RightPanelTab::Code => view! { <CodePanel /> }.into_any(),
                                            RightPanelTab::History => view! { <HistoryPanel /> }.into_any(),
                                            RightPanelTab::Properties => view! {
                                                <div class="property-editor-container">
                                                    <PropertyEditor />
                                                    <div class="preview-section-min">
                                                        <Preview />
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }}
                                    </div>
                                </aside>
                            </div>
                        </main>
                    </div>

                    <StatusBar />

                    {move || if show_template_gallery.get() {
                        view! {
                            <TemplateGallery
                                on_close=move || show_template_gallery.set(false)
                                on_apply=move |template: crate::services::Template| {
                                    app_state.canvas.record_snapshot(&format!("Apply Template: {}", template.name));
                                    let comp_count = template.components.len();
                                    let template_name = template.name.clone();
                                    for comp in template.components {
                                        app_state.canvas.add_component_without_snapshot(comp);
                                    }
                                    show_template_gallery.set(false);
                                    app_state.ui.notification.set(Some(Notification::success(
                                        format!("✨ Template '{}' applied!", template_name)
                                    )));
                                    announce(&format!("Template {} applied with {} components", template_name, comp_count));
                                }
                            />
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }}

                <SettingsModal />
                <ShortcutsModal />

                {move || if show_save_template.get() {
                    view! {
                        <SaveTemplateModal
                            show=show_save_template
                            on_close=Callback::new(move |_| show_save_template.set(false))
                        />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}

                {move || if show_export.get() {
                    view! {
                        <ExportModal
                            show=show_export
                            code=export_code
                            format=export_template
                            on_close=Callback::new(move |_| show_export.set(false))
                            notification_signal=app_state.ui.notification
                        />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}

                // Removed WelcomeModal from here, maybe move to Dashboard or keep it as a global "first time" thing?
                // Keeping it out of editor for now for cleaner UX.

                    <Snackbar notification=app_state.ui.notification />
                </div>
            </AccessibilityProvider>
        </DesignTokenProvider>
    }
}
