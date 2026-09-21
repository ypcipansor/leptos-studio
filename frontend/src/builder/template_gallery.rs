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
    /// The signal that controls whether the gallery is visible. It is the single
    /// source of truth: the same signal must render the gallery, activate the
    /// Escape listener, and feed the global shortcut gate. Passing a synthetic
    /// always-true signal here would leave the Escape listener armed after the
    /// gallery is hidden.
    show: RwSignal<bool>,
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
        // Escape closes the gallery only while it is actually open. Gating on
        // the caller's visibility signal means the listener disarms on close
        // instead of staying live until the component unmounts, so a hidden
        // gallery can never consume Escape or close another modal.
        let close = on_close;
        use_escape_key(show, move || {
            if show.get_untracked() {
                close.run(());
            }
        });
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
                            show=show_gallery
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

/// Browser-only regression tests for the gallery's Escape lifecycle.
///
/// These mount the real component and dispatch real `keydown` events, so they
/// catch a stale or duplicated global listener that a pure helper test cannot.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::state::DerivedState;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    async fn settle() {
        for _ in 0..4 {
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    }

    /// Mount the real gallery with a caller-owned `show` signal, so a test can
    /// hide it without unmounting.
    async fn with_gallery<F, Fut>(body: F)
    where
        F: FnOnce(RwSignal<bool>, Arc<AtomicUsize>) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let document = web_sys::window().unwrap().document().unwrap();
        let host = document
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        document.body().unwrap().append_child(&host).unwrap();

        let show = RwSignal::new(true);
        let closes = Arc::new(AtomicUsize::new(0));
        let closes_for_close = closes.clone();

        let unmount = leptos::mount::mount_to(host.clone(), move || {
            // `AppState` must be built inside the mount closure: `mount_to`
            // installs the executor and `Owner` that `provide_context` needs.
            AppState::provide_context();
            DerivedState::provide_context(AppState::expect_context());
            view! {
                <TemplateGallery
                    show=show
                    on_close=move || {
                        closes_for_close.fetch_add(1, Ordering::SeqCst);
                    }
                    on_apply=move |_| {}
                />
            }
        });

        body(show, closes).await;
        drop(unmount);
    }

    fn dispatch_escape() {
        let init = web_sys::KeyboardEventInit::new();
        init.set_key("Escape");
        init.set_bubbles(true);
        let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
            .expect("keydown event");
        let _ = web_sys::window().unwrap().dispatch_event(&event.into());
    }

    /// Escape must close the gallery exactly once while it is open, and must not
    /// fire again once the visibility signal is false — even though the
    /// component is still mounted.
    #[wasm_bindgen_test]
    async fn escape_closes_once_and_disarms_when_hidden() {
        with_gallery(|show, closes| async move {
            settle().await;
            assert!(show.get_untracked(), "the gallery starts open");

            dispatch_escape();
            settle().await;
            assert_eq!(
                closes.load(Ordering::SeqCst),
                1,
                "Escape while open must invoke on_close exactly once"
            );

            // Hide it without unmounting; the listener must go dormant.
            show.set(false);
            settle().await;

            dispatch_escape();
            dispatch_escape();
            settle().await;
            assert_eq!(
                closes.load(Ordering::SeqCst),
                1,
                "Escape while hidden must not invoke on_close at all"
            );
        })
        .await;
    }

    /// Repeated open/close cycles must not accumulate listeners: each open
    /// registers one handler, each close removes it, so the count per press
    /// stays at one.
    #[wasm_bindgen_test]
    async fn reopen_does_not_leak_duplicate_listeners() {
        with_gallery(|show, closes| async move {
            for cycle in 0..3 {
                show.set(true);
                settle().await;

                dispatch_escape();
                settle().await;
                assert_eq!(
                    closes.load(Ordering::SeqCst),
                    cycle + 1,
                    "each open cycle must close exactly once (cycle {cycle})"
                );

                show.set(false);
                settle().await;
            }

            assert_eq!(
                closes.load(Ordering::SeqCst),
                3,
                "three open/close cycles must produce exactly three closes, no duplicates"
            );
        })
        .await;
    }

    /// A hidden gallery must not consume Escape, so another modal listening for
    /// Escape still receives it.
    #[wasm_bindgen_test]
    async fn hidden_gallery_does_not_steal_escape_from_other_modals() {
        with_gallery(|show, closes| async move {
            settle().await;
            show.set(false);
            settle().await;

            let other_closes = Arc::new(AtomicUsize::new(0));
            let other_for_cb = other_closes.clone();
            let other_show = RwSignal::new(true);
            let handle =
                window_event_listener(leptos::ev::keydown, move |ev: leptos::ev::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        other_for_cb.fetch_add(1, Ordering::SeqCst);
                        other_show.set(false);
                    }
                });

            dispatch_escape();
            settle().await;

            assert_eq!(
                other_closes.load(Ordering::SeqCst),
                1,
                "a hidden gallery must not intercept Escape meant for another modal"
            );
            assert_eq!(
                closes.load(Ordering::SeqCst),
                0,
                "the hidden gallery must not close"
            );
            handle.remove();
        })
        .await;
    }
}
