//! Template Gallery Panel
//!
//! A visual gallery for browsing and applying pre-built templates
//! to the canvas. Supports search, filtering by category, and preview.

use leptos::prelude::*;

use crate::services::{Template, TemplateCategory, TemplateService};
use crate::state::app_state::{AppState, Notification};

/// Template gallery panel component with callbacks
#[component]
pub fn TemplateGallery(
    /// Callback when gallery is closed
    #[prop(into)]
    on_close: Callback<()>,
    /// Callback when a template is applied
    #[prop(into)]
    on_apply: Callback<Template>,
) -> impl IntoView {
    let app_state = AppState::expect_context();

    // Local state
    let search_query = RwSignal::new(String::new());
    let selected_category = RwSignal::new(None::<TemplateCategory>);
    let preview_template = RwSignal::new(None::<Template>);
    let custom_templates = RwSignal::new(Vec::<Template>::new());
    let loading = RwSignal::new(true);

    let refresh_templates = move || {
        loading.set(true);
        leptos::task::spawn_local(async move {
            if let Ok(templates) = TemplateService::fetch_custom_templates().await {
                custom_templates.set(templates);
            }
            loading.set(false);
        });
    };

    // Load on mount
    Effect::new(move |_| {
        refresh_templates();
    });

    {
        use crate::builder::hooks::use_escape_key::use_escape_key;
        let open = RwSignal::new(true);
        use_escape_key(open, move || on_close.run(()));
    }

    // Filtered templates
    let filtered_templates = Memo::new(move |_| {
        let query = search_query.get().to_lowercase();
        let category = selected_category.get();
        let custom = custom_templates.get();

        let mut all = TemplateService::builtin_templates();
        all.extend(custom);

        all.into_iter()
            .filter(|t| {
                let matches_search = query.is_empty()
                    || t.name.to_lowercase().contains(&query)
                    || t.description.to_lowercase().contains(&query)
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&query));

                let matches_category = category.is_none_or(|c| t.category == c);

                matches_search && matches_category
            })
            .collect::<Vec<_>>()
    });

    let on_delete_template = move |id: String| {
        if !window()
            .confirm_with_message("Are you sure you want to delete this template?")
            .unwrap_or(false)
        {
            return;
        }

        leptos::task::spawn_local(async move {
            if TemplateService::delete_custom_template(&id).await.is_ok() {
                refresh_templates();
                app_state
                    .ui
                    .notify(Notification::success("Template deleted".to_string()));
            } else {
                app_state
                    .ui
                    .notify(Notification::error("Failed to delete template".to_string()));
            }
        });
    };

    // Category button helper
    let category_button = move |cat: Option<TemplateCategory>, label: &'static str| {
        let is_active = Memo::new(move |_| selected_category.get() == cat);

        view! {
            <button
                class=move || if is_active.get() { "category-btn active" } else { "category-btn" }
                on:click=move |_| selected_category.set(cat)
            >
                {label}
            </button>
        }
    };

    view! {
        <div class="template-gallery-overlay" role="dialog" aria-modal="true" aria-labelledby="template-gallery-title">
            <div class="template-gallery-panel">
                <div class="template-gallery-header">
                    <h3 id="template-gallery-title">"Template Gallery"</h3>
                    <button
                        class="close-btn"
                        on:click=move |_| on_close.run(())
                        aria-label="Close gallery"
                    >
                        "×"
                    </button>
                </div>

                <div class="template-search">
                    <input
                        type="text"
                        placeholder="🔍 Search templates..."
                        class="template-search-input"
                        prop:value=move || search_query.get()
                        on:input=move |ev| search_query.set(event_target_value(&ev))
                    />
                </div>

                <div class="template-categories">
                    {category_button(None, "All")}
                    {category_button(Some(TemplateCategory::Custom), "User")}
                    {category_button(Some(TemplateCategory::LandingPage), "Landing")}
                    {category_button(Some(TemplateCategory::Form), "Forms")}
                    {category_button(Some(TemplateCategory::Hero), "Hero")}
                    {category_button(Some(TemplateCategory::Navigation), "Nav")}
                    {category_button(Some(TemplateCategory::Card), "Cards")}
                    {category_button(Some(TemplateCategory::Dashboard), "Dash")}
                    {category_button(Some(TemplateCategory::Footer), "Footer")}
                </div>

                <div class="template-grid">
                    {move || if loading.get() && custom_templates.get().is_empty() {
                         view! { <div class="loading-state">"Loading templates..."</div> }.into_any()
                    } else {
                        view! {
                            <For
                                each=move || filtered_templates.get()
                                key=|template| template.id.clone()
                                children=move |template| {
                                    let template_for_preview = template.clone();
                                    let template_for_apply = template.clone();
                                    let template_name = template.name.clone();
                                    let template_desc = template.description.clone();
                                    let template_tags = template.tags.clone();
                                    let is_custom = template.category == TemplateCategory::Custom;
                                    let id_for_delete = template.id.clone();

                                    view! {
                                        <div
                                            class="template-card"
                                            on:mouseenter=move |_| preview_template.set(Some(template_for_preview.clone()))
                                            on:mouseleave=move |_| preview_template.set(None)
                                        >
                                            <div class="template-card-preview">
                                                <div class="template-icon">
                                                    {category_icon(&template.category)}
                                                </div>
                                                {if is_custom {
                                                    view! {
                                                        <button
                                                            class="btn btn-sm btn-ghost template-delete-btn"
                                                            style="position: absolute; top: 5px; right: 5px; color: #ff4444; background: rgba(255,255,255,0.8); border-radius: 4px; padding: 2px 6px; z-index: 10;"
                                                            on:click={
                                                                let id = id_for_delete.clone();
                                                                move |ev: leptos::web_sys::MouseEvent| {
                                                                    ev.stop_propagation();
                                                                    on_delete_template(id.clone());
                                                                }
                                                            }
                                                            title="Delete Template"
                                                        >
                                                            "🗑️"
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    ().into_any()
                                                }}
                                            </div>
                                            <div class="template-card-content">
                                                <h4 class="template-name">{template_name}</h4>
                                                <p class="template-description">{template_desc}</p>
                                                <div class="template-tags">
                                                    {template_tags.into_iter().take(3).map(|tag| {
                                                        view! { <span class="template-tag">{tag}</span> }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                            <button
                                                class="btn btn-primary template-apply-btn"
                                                on:click={
                                                    let template_clone = template_for_apply.clone();
                                                    move |_| on_apply.run(template_clone.clone())
                                                }
                                            >
                                                "Apply"
                                            </button>
                                        </div>
                                    }
                                }
                            />
                        }.into_any()
                    }}
                </div>

                {move || {
                    if !loading.get() && filtered_templates.get().is_empty() {
                        view! {
                            <div class="template-empty-state">
                                <p>"No templates found"</p>
                                <p class="template-empty-hint">"Try a different search or category"</p>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }
                }}
            </div>
        </div>
    }
}

/// Standalone template gallery toggle button with built-in panel
#[component]
pub fn TemplateGalleryToggle() -> impl IntoView {
    let app_state = AppState::expect_context();
    let show_gallery = RwSignal::new(false);

    view! {
        <div class="template-gallery-wrapper">
            <button
                class="btn btn-secondary template-gallery-toggle"
                on:click=move |_| show_gallery.update(|v| *v = !*v)
            >
                {move || if show_gallery.get() { "📁 Close Templates" } else { "📁 Templates" }}
            </button>

            {move || {
                if show_gallery.get() {
                    view! {
                        <TemplateGallery
                            on_close=move || show_gallery.set(false)
                            on_apply=move |template: Template| {
                                app_state.canvas.record_snapshot(&format!("Apply Template: {}", template.name));
                                for component in template.components {
                                    app_state.canvas.add_component_without_snapshot(component);
                                }
                                app_state.ui.notification.set(Some(Notification::success(
                                    format!("✅ Template '{}' applied!", template.name),
                                )));
                                show_gallery.set(false);
                            }
                        />
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}

/// Get icon for template category
fn category_icon(category: &TemplateCategory) -> &'static str {
    match category {
        TemplateCategory::Form => "📝",
        TemplateCategory::Hero => "🦸",
        TemplateCategory::Navigation => "🧭",
        TemplateCategory::Card => "🃏",
        TemplateCategory::Dashboard => "📊",
        TemplateCategory::Footer => "🦶",
        TemplateCategory::LandingPage => "🏠",
        TemplateCategory::Custom => "⚙️",
    }
}
