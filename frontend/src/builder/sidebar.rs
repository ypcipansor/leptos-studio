use leptos::prelude::*;
use web_sys::window;

use super::component_library::{ComponentRegistry, LibraryComponent, ResponsiveMode, Theme};
use super::drag_drop::{DragDropConfig, create_drag_handlers};
use crate::builder::debug_panel::DebugPanel;
use crate::builder::git_panel::GitPanel;
use crate::builder::project::ProjectPanel;
use crate::builder::variable_panel::VariablePanel;
use crate::domain::error::ValidationError;
use crate::domain::validation::{ComponentNameValidator, HtmlTemplateValidator, Validator};
use crate::services::export_service::{CodeGenerator, LeptosCodeGenerator};
use crate::state::app_state::{AppState, ExportPreset, Notification};

/// Convert ValidationError to user-friendly Indonesian message
fn validation_error_to_message(error: ValidationError) -> String {
    match error {
        ValidationError::EmptyName => "Nama komponen wajib diisi.".to_string(),
        ValidationError::InvalidName(_) => {
            "Nama komponen hanya boleh huruf, angka, dan underscore.".to_string()
        }
        ValidationError::EmptyTemplate => "Template wajib diisi.".to_string(),
        ValidationError::InvalidTemplate(msg) => format!("Template tidak valid: {}", msg),
        _ => "Input tidak valid.".to_string(),
    }
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let app_state = AppState::expect_context();

    let custom_theme_color = RwSignal::new(String::from("#888"));
    let show_add_form = RwSignal::new(false);
    let new_name = RwSignal::new(String::new());
    let new_template = RwSignal::new(String::new());
    let filter_query = RwSignal::new(String::new());
    let error_msg = RwSignal::new(String::new());
    let editing_idx = RwSignal::new(None::<usize>);
    let edit_name = RwSignal::new(String::new());
    let edit_template = RwSignal::new(String::new());

    let add_custom_component = move |_| {
        let name = new_name.get().trim().to_string();
        let template = new_template.get().trim().to_string();

        // Validate component name using domain validator
        let name_validator = ComponentNameValidator;
        if let Err(e) = name_validator.validate(&name) {
            error_msg.set(validation_error_to_message(e));
            return;
        }

        // Check for duplicate names
        if ComponentRegistry::exists_by_name(&app_state.ui.custom_components.get(), &name) {
            error_msg.set("Nama komponen sudah ada.".to_string());
            return;
        }

        // Validate template using domain validator
        let template_validator = HtmlTemplateValidator;
        if let Err(e) = template_validator.validate(&template) {
            error_msg.set(validation_error_to_message(e));
            return;
        }

        let new_component = LibraryComponent {
            id: crate::builder::component_library::new_library_id(),
            name: name.clone(),
            kind: "Custom".to_string(),
            template: Some(template.clone()),
            category: "Custom".to_string(),
            props_schema: None,
            description: None,
        };

        let mut custom = app_state.ui.custom_components.get();
        let mut library = app_state.ui.component_library.get();
        if let Err(err) = ComponentRegistry::add_custom(&mut custom, &mut library, new_component) {
            error_msg.set(err.message());
            return;
        }
        app_state.ui.custom_components.set(custom);
        app_state.ui.component_library.set(library);

        new_name.set(String::new());
        new_template.set(String::new());
        error_msg.set(String::new());
        show_add_form.set(false);

        app_state.ui.notification.set(Some(Notification::success(
            "✅ Komponen berhasil ditambahkan".to_string(),
        )));
    };

    let delete_custom_component = move |idx: usize| {
        let mut custom = app_state.ui.custom_components.get();
        let mut library = app_state.ui.component_library.get();
        ComponentRegistry::delete_custom_by_index(&mut custom, &mut library, idx);
        app_state.ui.custom_components.set(custom);
        app_state.ui.component_library.set(library);

        app_state.ui.notification.set(Some(Notification::success(
            "🗑️ Komponen dihapus".to_string(),
        )));
    };

    let start_edit_custom_component = move |idx: usize| {
        if let Some(c) = app_state.ui.custom_components.get().get(idx) {
            edit_name.set(c.name.clone());
            edit_template.set(c.template.as_deref().unwrap_or("").to_string());
            editing_idx.set(Some(idx));
            error_msg.set(String::new());
        }
    };

    let save_edit_custom_component = move |_| {
        let idx = match editing_idx.get() {
            Some(i) => i,
            None => return,
        };

        let name = edit_name.get().trim().to_string();
        let template = edit_template.get().trim().to_string();

        // Validate component name using domain validator
        let name_validator = ComponentNameValidator;
        if let Err(e) = name_validator.validate(&name) {
            error_msg.set(validation_error_to_message(e));
            return;
        }

        // Check for duplicate names (excluding current component)
        let existing_custom = app_state.ui.custom_components.get();
        let mut others = existing_custom.clone();
        if idx < others.len() {
            others.remove(idx);
        }
        if ComponentRegistry::exists_by_name(&others, &name) {
            error_msg.set("Nama komponen sudah ada.".to_string());
            return;
        }

        // Validate template using domain validator
        let template_validator = HtmlTemplateValidator;
        if let Err(e) = template_validator.validate(&template) {
            error_msg.set(validation_error_to_message(e));
            return;
        }

        let mut custom = existing_custom;
        let mut library = app_state.ui.component_library.get();
        if let Err(e) = ComponentRegistry::update_custom_by_index(
            &mut custom,
            &mut library,
            idx,
            name.clone(),
            template.clone(),
        ) {
            error_msg.set(e.message());
            return;
        }
        app_state.ui.custom_components.set(custom);
        app_state.ui.component_library.set(library);

        editing_idx.set(None);
        edit_name.set(String::new());
        edit_template.set(String::new());
        error_msg.set(String::new());

        app_state.ui.notification.set(Some(Notification::success(
            "✅ Komponen diperbarui".to_string(),
        )));
    };

    let cancel_edit_custom_component = move |_| {
        editing_idx.set(None);
        error_msg.set(String::new());
    };

    let export_code = move |_| {
        let components = app_state.canvas.components.get();
        let preset = app_state.settings.with(|s| s.export_preset.clone());
        let variables = app_state.variables.get();
        let generator = LeptosCodeGenerator::new(preset);

        match generator.generate(&components, &variables) {
            Ok(code) => {
                if let Some(win) = window() {
                    let clipboard = win.navigator().clipboard();
                    let _ = clipboard.write_text(&code);
                }
                app_state
                    .ui
                    .notification
                    .set(Some(Notification::success("✅ Kode diekspor!".to_string())));
            }
            Err(e) => {
                app_state
                    .ui
                    .notification
                    .set(Some(Notification::error(format!("❌ Error: {}", e))));
            }
        }
    };

    view! {
        <aside class="sidebar-content">
            <DebugPanel />

            <div class="sidebar-section git-panel-wrapper">
                <b>"Git Panel"</b>
                <GitPanel />
            </div>
            <ProjectPanel />
            <VariablePanel />

            <h2>"Sidebar"</h2>

            <div class="sidebar-section">
                <b>"Theme:"</b>
                <div class="theme-buttons">
                    <button
                        on:click=move |_| app_state.settings.update(|s| s.theme = Theme::Light)
                        disabled=move || app_state.settings.with(|s| s.theme == Theme::Light)
                    >"Light"</button>
                    <button
                        on:click=move |_| app_state.settings.update(|s| s.theme = Theme::Dark)
                        disabled=move || app_state.settings.with(|s| s.theme == Theme::Dark)
                    >"Dark"</button>
                    <button
                        on:click=move |_| app_state.settings.update(|s| s.theme = Theme::Custom)
                        disabled=move || app_state.settings.with(|s| s.theme == Theme::Custom)
                    >"Custom"</button>
                </div>
                {move || {
                    if app_state.settings.with(|s| s.theme == Theme::Custom) {
                        view! {
                            <input
                                type="color"
                                prop:value=move || custom_theme_color.get()
                                on:input=move |ev| custom_theme_color.set(event_target_value(&ev))
                            />
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </div>

            <div class="sidebar-section">
                <b>"Responsive:"</b>
                <select on:change=move |ev| {
                    let mode = match event_target_value(&ev).as_str() {
                        "Mobile" => ResponsiveMode::Mobile,
                        "Tablet" => ResponsiveMode::Tablet,
                        _ => ResponsiveMode::Desktop,
                    };
                    app_state.ui.responsive_mode.set(mode);
                }>
                    <option selected=move || app_state.ui.responsive_mode.get() == ResponsiveMode::Desktop>"Desktop"</option>
                    <option selected=move || app_state.ui.responsive_mode.get() == ResponsiveMode::Tablet>"Tablet"</option>
                    <option selected=move || app_state.ui.responsive_mode.get() == ResponsiveMode::Mobile>"Mobile"</option>
                </select>
            </div>

            <div class="sidebar-section">
                <label>"Export Preset:"</label>
                <select on:change=move |ev| {
                    let preset = match event_target_value(&ev).as_str() {
                        "ThawUi" => ExportPreset::ThawUi,
                        "LeptosMaterial" => ExportPreset::LeptosMaterial,
                        "LeptosUse" => ExportPreset::LeptosUse,
                        _ => ExportPreset::Plain,
                    };
                    app_state.settings.update(|s| s.export_preset = preset);
                }>
                    <option value="Plain">"Plain"</option>
                    <option value="ThawUi">"thaw-ui"</option>
                    <option value="LeptosMaterial">"leptos-material"</option>
                    <option value="LeptosUse">"leptos-use"</option>
                </select>
            </div>

            <button on:click=export_code>"Export Project"</button>

            {move || {
                app_state.ui.notification.get().as_ref().map(|notif| {
                    view! { <div class="sidebar-notification">{notif.message.clone()}</div> }
                })
            }}

            <div class="sidebar-section-divider">
                <h3>"Component Library"</h3>
                <div class="component-library-grid">
                    {move || {
                        app_state
                            .ui
                            .component_library
                            .get()
                            .into_iter()
                            .map(|comp| {
                                let comp_kind = if comp.kind == "Custom" {
                                    format!("Custom::{}", comp.name)
                                } else {
                                    comp.kind.clone()
                                };

                                let (on_drag_start, on_drag, on_drag_end) = create_drag_handlers(
                                    comp_kind.clone(),
                                    app_state.canvas.drag_state,
                                    DragDropConfig::default(),
                                );

                                view! {
                                    <div
                                        class="component-item"
                                        draggable="true"
                                        on:dragstart=on_drag_start
                                        on:drag=on_drag
                                        on:dragend=on_drag_end
                                    >
                                        <div style="font-weight:bold;">{comp.name.clone()}</div>
                                        <div style="font-size:11px;color:#666;">
                                            {comp.description.clone().unwrap_or_default()}
                                        </div>
                                        <div style="font-size:10px;color:#999;margin-top:4px;">
                                            {comp.category.clone()}
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </div>

            <div style="margin-top:16px;border-top:1px solid #ccc;padding-top:12px;">
                <h3>"Custom Components"</h3>

                {move || {
                    if !error_msg.get().is_empty() {
                        view! { <div class="sidebar-error">{error_msg.get()}</div> }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                <input
                    type="text"
                    placeholder="Filter..."
                    class="sidebar-filter-input"
                    prop:value=move || filter_query.get()
                    on:input=move |ev| filter_query.set(event_target_value(&ev))
                />

                <div>
                    {move || {
                        let query = filter_query.get().to_lowercase();
                        app_state.ui.custom_components.get()
                            .into_iter()
                            .enumerate()
                            .filter(|(_, c)| query.is_empty() || c.name.to_lowercase().contains(&query))
                            .map(|(idx, comp)| {
                                view! {
                                    <div class="custom-component-row">
                                        <span>{comp.name.clone()}</span>
                                        <div>
                                            <button on:click=move |_| start_edit_custom_component(idx)>"Edit"</button>
                                            <button on:click=move |_| delete_custom_component(idx)>"Del"</button>
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </div>

                <button on:click=move |_| show_add_form.update(|v| *v = !*v)>
                    {move || if show_add_form.get() { "Cancel" } else { "+ Add" }}
                </button>

                {move || {
                    if show_add_form.get() {
                        view! {
                            <div class="custom-component-form">
                                <input
                                    type="text"
                                    placeholder="Name"
                                    style="width:100%;"
                                    prop:value=move || new_name.get()
                                    on:input=move |ev| new_name.set(event_target_value(&ev))
                                />
                                <textarea
                                    placeholder="Template"
                                    class="custom-component-textarea"
                                    prop:value=move || new_template.get()
                                    on:input=move |ev| new_template.set(event_target_value(&ev))
                                />
                                <button on:click=add_custom_component>"Add"</button>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}

                {move || {
                    if editing_idx.get().is_some() {
                        view! {
                            <div class="custom-component-edit">
                                <h4>"Edit"</h4>
                                <input
                                    type="text"
                                    style="width:100%;"
                                    prop:value=move || edit_name.get()
                                    on:input=move |ev| edit_name.set(event_target_value(&ev))
                                />
                                <textarea
                                    class="custom-component-textarea"
                                    prop:value=move || edit_template.get()
                                    on:input=move |ev| edit_template.set(event_target_value(&ev))
                                />
                                <button on:click=save_edit_custom_component>"Save"</button>
                                <button on:click=cancel_edit_custom_component>"Cancel"</button>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </div>
        </aside>
    }
}
