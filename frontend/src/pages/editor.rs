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
use crate::builder::keyboard::{KeyboardHandler, editor_modal_open, get_default_shortcuts};
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

    // Better naming: template to use for the export content
    let export_code = RwSignal::new(String::new());
    let export_template = RwSignal::new("leptos".to_string());
    let show_template_gallery = RwSignal::new(false);
    let show_save_template = RwSignal::new(false);

    // A single, stable search signal for the command palette: it must survive
    // re-renders so the palette's focus/query state is not rebuilt underneath it.
    let palette_search = RwSignal::new(String::new());

    let show_left_sidebar_mobile = RwSignal::new(false);

    // The Export modal is rendered from a single source of truth:
    // `app_state.ui.show_export_modal` both opens it and gates shortcuts, so the
    // two can never drift apart.
    let show_export = app_state.ui.show_export_modal;

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
    // not the user's target, so the global shortcuts must stay dormant. Each
    // argument is the exact signal that gates the matching modal's rendering.
    let modal_open = Signal::derive(move || {
        editor_modal_open(
            app_state.ui.show_command_palette.get(),
            show_export.get(),
            app_state.ui.show_settings_modal.get(),
            app_state.ui.show_shortcuts_modal.get(),
            show_template_gallery.get(),
            show_save_template.get(),
        )
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
                        search=palette_search
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
                                <section id="main-canvas" class="canvas-area" role="region" aria-label="Design canvas" tabindex="-1">
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::keyboard::should_dispatch_shortcut;
    use crate::builder::keyboard::{KeyboardAction, get_default_shortcuts};

    /// The signal set the editor derives `modal_open` from.
    ///
    /// The modal signals that live on `UiState` are taken from a *real*
    /// `UiState`, not mirrored: this is the same state `EditorPage` reads, so the
    /// gate genuinely follows the signals the modals render from. Only the two
    /// page-local signals (template gallery, save template) are created here.
    struct EditorModals {
        ui: crate::state::app_state::UiState,
        show_template_gallery: RwSignal<bool>,
        show_save_template: RwSignal<bool>,
    }

    impl EditorModals {
        fn new() -> Self {
            Self {
                ui: crate::state::app_state::UiState::new(),
                show_template_gallery: RwSignal::new(false),
                show_save_template: RwSignal::new(false),
            }
        }

        /// The Export signal exactly as `EditorPage` aliases it.
        fn show_export(&self) -> RwSignal<bool> {
            self.ui.show_export_modal
        }

        /// Exactly the derivation used by `EditorPage`, so a drift between the
        /// gate and any modal's render condition fails this test.
        fn modal_open(&self) -> Signal<bool> {
            let ui = self.ui;
            let gallery = self.show_template_gallery;
            let save_template = self.show_save_template;
            Signal::derive(move || {
                editor_modal_open(
                    ui.show_command_palette.get(),
                    ui.show_export_modal.get(),
                    ui.show_settings_modal.get(),
                    ui.show_shortcuts_modal.get(),
                    gallery.get(),
                    save_template.get(),
                )
            })
        }

        /// Whether the editor's shortcuts would run right now.
        fn shortcuts_enabled(&self) -> bool {
            should_dispatch_shortcut(self.modal_open().get_untracked(), false)
        }
    }

    /// Delete/Undo/Redo/SelectAll are the actions that mutate the canvas; they
    /// must all be blocked while Export is open and active again once it closes.
    #[test]
    fn export_modal_gates_every_canvas_editing_shortcut() {
        let modals = EditorModals::new();
        let canvas_editing = [
            KeyboardAction::Delete,
            KeyboardAction::Undo,
            KeyboardAction::Redo,
            KeyboardAction::SelectAll,
        ];

        // Every action has a shortcut to suppress in the first place.
        let shortcuts = get_default_shortcuts();
        for action in &canvas_editing {
            assert!(
                shortcuts.iter().any(|s| &s.action == action),
                "{action:?} must keep a shortcut"
            );
        }

        assert!(modals.shortcuts_enabled(), "shortcuts start enabled");

        // Open and close Export through the signal that renders the modal.
        modals.show_export().set(true);
        assert!(
            !modals.shortcuts_enabled(),
            "opening Export must block canvas-editing shortcuts"
        );

        modals.show_export().set(false);
        assert!(
            modals.shortcuts_enabled(),
            "closing Export must re-enable canvas-editing shortcuts"
        );
    }

    /// A `(name, signal-picker)` pair so the modal list stays readable.
    type ModalCase = (&'static str, fn(&EditorModals) -> RwSignal<bool>);

    /// Every editor modal, driven through its own render signal, must gate the
    /// shortcuts — including the Export modal.
    #[test]
    fn every_editor_modal_gates_shortcuts() {
        let modals = EditorModals::new();

        // Export resolves to `UiState::show_export_modal`, the same signal that
        // renders the modal, not a page-local mirror.
        let cases: [ModalCase; 6] = [
            ("command palette", |m| m.ui.show_command_palette),
            ("export", |m| m.ui.show_export_modal),
            ("settings", |m| m.ui.show_settings_modal),
            ("shortcuts", |m| m.ui.show_shortcuts_modal),
            ("template gallery", |m| m.show_template_gallery),
            ("save template", |m| m.show_save_template),
        ];

        for (name, pick) in cases {
            let signal = pick(&modals);
            signal.set(true);
            assert!(
                !modals.shortcuts_enabled(),
                "the {name} modal must gate canvas shortcuts"
            );
            signal.set(false);
            assert!(
                modals.shortcuts_enabled(),
                "closing the {name} modal must restore canvas shortcuts"
            );
        }
    }

    /// The gate must be derived from the modal-render signals, not from a
    /// separate flag: toggling a render signal is what flips it.
    #[test]
    fn gate_tracks_the_rendered_export_modal_state() {
        let modals = EditorModals::new();

        // This mirrors the UI: `show_export` *is* `UiState::show_export_modal`,
        // the signal that renders the modal, so opening the modal through it is
        // what flips the gate.
        let rendered = modals.show_export();
        assert!(!rendered.get_untracked());
        assert!(!modals.modal_open().get_untracked());

        rendered.set(true);
        assert!(modals.show_export().get_untracked());
        assert!(
            modals.modal_open().get_untracked(),
            "the gate must follow the signal that renders Export"
        );

        rendered.set(false);
        assert!(!modals.modal_open().get_untracked());
    }

    /// Text inputs stay suppressed regardless of modal state.
    #[test]
    fn text_inputs_are_still_suppressed() {
        let modals = EditorModals::new();
        assert!(!should_dispatch_shortcut(
            modals.modal_open().get_untracked(),
            true
        ));

        modals.show_export().set(true);
        assert!(!should_dispatch_shortcut(
            modals.modal_open().get_untracked(),
            true
        ));
    }
}

/// Browser-only DOM checks.
///
/// These live in the lib suite rather than an integration target on purpose: the
/// `wasm-pack` integration targets fail to link with `the name 'main' is exported
/// by multiple crates` because `lib.rs`'s `#[wasm_bindgen(start)] fn main` is
/// compiled into them. `#[cfg(not(test))]` on that function only takes effect for
/// the lib-target compilation.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::services::analytics_service::AnalyticsService;
    use crate::state::DerivedState;
    use leptos_router::components::{Route, Router, Routes};
    use leptos_router::path;
    use wasm_bindgen::JsCast;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    async fn settle() {
        for _ in 0..4 {
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    }

    fn is_focused(target: &web_sys::Element) -> bool {
        let active = document().active_element();
        active.as_ref() == Some(target)
    }

    /// The editor has two nested elements that used to share `id="main-canvas"`:
    /// the `<section>` the skip link targets and the inner surface the click/pan
    /// handlers compare against. Only the section may carry that id, and the skip
    /// link has to resolve to it.
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn editor_renders_exactly_one_main_canvas_target() {
        let document = document();
        let host = document
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        document.body().unwrap().append_child(&host).unwrap();

        let history = web_sys::window().unwrap().history().unwrap();
        history
            .push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some("/editor/dom-test"))
            .unwrap();

        // `mount_to` installs the global executor that effects spawn onto and
        // establishes the Owner that `provide_context` needs, so the state has to
        // be constructed *inside* the mount closure: `AppState::new` starts
        // effects, and providing a context outside an Owner silently goes nowhere.
        let unmount = leptos::mount::mount_to(host, move || {
            AppState::provide_context();
            DerivedState::provide_context(AppState::expect_context());
            AnalyticsService::provide_context();

            view! {
                <Router>
                    <Routes fallback=|| view! { <div /> }>
                        <Route path=path!("/editor/:id") view=EditorPage />
                    </Routes>
                </Router>
            }
        });

        settle().await;

        let matches = document
            .query_selector_all("#main-canvas")
            .expect("selector must be valid");
        assert_eq!(
            matches.length(),
            1,
            "the editor must render exactly one #main-canvas"
        );

        let target = matches
            .item(0)
            .expect("the section must exist")
            .dyn_into::<web_sys::Element>()
            .unwrap();

        let skip_link = document
            .query_selector("a.skip-link")
            .unwrap()
            .expect("the editor must render a skip link");
        let href = skip_link.get_attribute("href").unwrap_or_default();
        assert_eq!(href, "#main-canvas", "the skip link must target the region");

        let resolved = document
            .get_element_by_id(href.strip_prefix('#').unwrap())
            .expect("the skip link target must resolve");
        assert_eq!(
            resolved, target,
            "the skip link must resolve to the main canvas region"
        );

        assert_eq!(
            target.get_attribute("role").as_deref(),
            Some("region"),
            "the skip target must be the labelled region, not an inner surface"
        );

        // The skip link focuses the region itself, so cancelling the default
        // navigation is only correct if the target can actually receive focus.
        assert_eq!(
            target.get_attribute("tabindex").as_deref(),
            Some("-1"),
            "the skip target must be programmatically focusable"
        );

        let click = web_sys::MouseEvent::new("click").unwrap();
        skip_link.dispatch_event(&click).unwrap();
        assert!(
            is_focused(&target),
            "activating the skip link must move focus to the canvas region"
        );

        // The inner surface keeps a distinct id so the click/pan handlers can
        // tell it apart from the surrounding section.
        assert!(
            document.get_element_by_id("canvas-surface").is_some(),
            "the inner surface must keep a distinct id"
        );

        drop(unmount);
    }
}
