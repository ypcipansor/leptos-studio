use crate::builder::canvas::handle_drop;
use crate::builder::drag_drop::DropZone;
use crate::domain::{
    Animation, BadgeComponent, ButtonComponent, CanvasComponent, CardComponent, CheckboxComponent,
    ContainerComponent, CustomComponent, DividerComponent, ImageComponent, InputComponent,
    LinkComponent, ProgressComponent, RadioGroupComponent, SelectComponent, SwitchComponent,
    TextComponent,
};
use crate::state::{AppState, CanvasState};
use leptos::prelude::*;

fn get_animation_style(animation: &Option<Animation>) -> String {
    animation
        .as_ref()
        .map(|a| a.to_css_string())
        .unwrap_or_default()
}

/// Component renderer for displaying canvas components
#[component]
pub fn ComponentRenderer(
    /// The component to render
    component: CanvasComponent,
    /// Canvas state for selection tracking
    canvas_state: CanvasState,
) -> impl IntoView {
    let app_state = AppState::expect_context();
    let component_id = *component.id();
    let preview_mode = app_state.ui.preview_mode;

    let is_selected =
        Memo::new(move |_| !preview_mode.get() && canvas_state.is_selected(component_id));

    let on_click = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
        if !preview_mode.get() {
            if ev.shift_key() || ev.ctrl_key() || ev.meta_key() {
                canvas_state.toggle_multi_select(component_id);
            } else {
                canvas_state.select_single(component_id);
            }
        }
    };

    let class = move || {
        if is_selected.get() {
            "canvas-component selected"
        } else {
            "canvas-component"
        }
    };

    let component_type_label = match component.component_type() {
        crate::domain::ComponentType::Button => "Button",
        crate::domain::ComponentType::Text => "Text",
        crate::domain::ComponentType::Input => "Input",
        crate::domain::ComponentType::Container => "Container",
        crate::domain::ComponentType::Image => "Image",
        crate::domain::ComponentType::Card => "Card",
        crate::domain::ComponentType::Select => "Select",
        crate::domain::ComponentType::Custom => "Custom",
        crate::domain::ComponentType::Divider => "Divider",
        crate::domain::ComponentType::Checkbox => "Checkbox",
        crate::domain::ComponentType::RadioGroup => "Radio Group",
        crate::domain::ComponentType::Switch => "Switch",
        crate::domain::ComponentType::Badge => "Badge",
        crate::domain::ComponentType::Progress => "Progress",
        crate::domain::ComponentType::Link => "Link",
    };

    view! {
        <div
            class=class
            on:click=on_click
            data-component-id=component_id.to_string()
            role="listitem"
            aria-label=component_type_label
        >
            {move || if is_selected.get() {
                view! { <div class="selected-label">{component_type_label}</div> }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}
            {match component {
                CanvasComponent::Button(btn) => render_button(btn).into_any(),
                CanvasComponent::Text(txt) => render_text(txt).into_any(),
                CanvasComponent::Input(inp) => render_input(inp).into_any(),
                CanvasComponent::Container(container) => render_container(container, canvas_state).into_any(),
                CanvasComponent::Image(img) => render_image(img).into_any(),
                CanvasComponent::Card(card) => render_card(card, canvas_state).into_any(),
                CanvasComponent::Select(sel) => render_select(sel).into_any(),
                CanvasComponent::Custom(custom) => render_custom(custom).into_any(),
                CanvasComponent::Divider(divider) => render_divider(divider).into_any(),
                CanvasComponent::Checkbox(checkbox) => render_checkbox(checkbox).into_any(),
                CanvasComponent::RadioGroup(radio) => render_radio_group(radio).into_any(),
                CanvasComponent::Switch(switch) => render_switch(switch).into_any(),
                CanvasComponent::Badge(badge) => render_badge(badge).into_any(),
                CanvasComponent::Progress(progress) => render_progress(progress).into_any(),
                CanvasComponent::Link(link) => render_link(link).into_any(),
            }}
        </div>
    }
}

fn render_button(button: ButtonComponent) -> impl IntoView {
    let variant_class = match button.variant {
        crate::domain::ButtonVariant::Primary => "btn-primary",
        crate::domain::ButtonVariant::Secondary => "btn-secondary",
        crate::domain::ButtonVariant::Outline => "btn-outline",
        crate::domain::ButtonVariant::Ghost => "btn-ghost",
    };

    let size_class = match button.size {
        crate::domain::ButtonSize::Small => "btn-sm",
        crate::domain::ButtonSize::Medium => "btn-md",
        crate::domain::ButtonSize::Large => "btn-lg",
    };

    let anim_style = get_animation_style(&button.animation);
    let custom_style = button.style.to_css_string();

    view! {
        <button
            class=format!("canvas-button {} {}", variant_class, size_class)
            disabled=button.disabled
            style=format!("{} {}", anim_style, custom_style)
        >
            {button.label}
        </button>
    }
}

fn render_text(text: TextComponent) -> impl IntoView {
    let tag_class = match text.tag {
        crate::domain::TextTag::H1 => "text-h1",
        crate::domain::TextTag::H2 => "text-h2",
        crate::domain::TextTag::H3 => "text-h3",
        crate::domain::TextTag::P => "text-p",
        crate::domain::TextTag::Span => "text-span",
    };

    let style_class = match text.style {
        crate::domain::TextStyle::Heading1 => "style-heading1",
        crate::domain::TextStyle::Heading2 => "style-heading2",
        crate::domain::TextStyle::Heading3 => "style-heading3",
        crate::domain::TextStyle::Body => "style-body",
        crate::domain::TextStyle::Caption => "style-caption",
    };

    let anim_style = get_animation_style(&text.animation);
    let custom_style = text.custom_style.to_css_string();

    view! {
        <span
            class=format!("canvas-text {} {}", tag_class, style_class)
            style=format!("{} {}", anim_style, custom_style)
        >
            {text.content}
        </span>
    }
}

fn render_input(input: InputComponent) -> impl IntoView {
    let input_type = match input.input_type {
        crate::domain::InputType::Text => "text",
        crate::domain::InputType::Password => "password",
        crate::domain::InputType::Email => "email",
        crate::domain::InputType::Number => "number",
        crate::domain::InputType::Tel => "tel",
    };

    let anim_style = get_animation_style(&input.animation);
    let custom_style = input.style.to_css_string();

    view! {
        <input
            class="canvas-input"
            type=input_type
            placeholder=input.placeholder
            required=input.required
            disabled=input.disabled
            style=format!("{} {}", anim_style, custom_style)
        />
    }
}

fn render_select(select: SelectComponent) -> impl IntoView {
    let options: Vec<String> = select
        .options
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let anim_style = get_animation_style(&select.animation);
    let custom_style = select.style.to_css_string();

    view! {
        <select
            class="canvas-select"
            disabled=select.disabled
            style=format!("{} {}", anim_style, custom_style)
        >
            {if !select.placeholder.is_empty() {
                Some(view! { <option value="" disabled selected>{select.placeholder}</option> })
            } else {
                None
            }}
            {options.into_iter().map(|opt| {
                view! { <option value=opt.clone()>{opt.clone()}</option> }
            }).collect_view()}
        </select>
    }
}

fn render_container(container: ContainerComponent, canvas_state: CanvasState) -> impl IntoView {
    let (layout_class, align_style) = match &container.layout {
        crate::domain::LayoutType::Flex {
            direction,
            wrap,
            align_items,
            justify_content,
        } => {
            let dir_class = match direction {
                crate::domain::FlexDirection::Row => "flex-row",
                crate::domain::FlexDirection::Column => "flex-col",
            };

            let mut classes = dir_class.to_string();
            if *wrap {
                classes.push_str(" flex-wrap");
            }

            let align_css = match align_items {
                crate::domain::FlexAlign::Start => "flex-start",
                crate::domain::FlexAlign::Center => "center",
                crate::domain::FlexAlign::End => "flex-end",
                crate::domain::FlexAlign::Stretch => "stretch",
                crate::domain::FlexAlign::Baseline => "baseline",
            };

            let justify_css = match justify_content {
                crate::domain::FlexJustify::Start => "flex-start",
                crate::domain::FlexJustify::Center => "center",
                crate::domain::FlexJustify::End => "flex-end",
                crate::domain::FlexJustify::Between => "space-between",
                crate::domain::FlexJustify::Around => "space-around",
                crate::domain::FlexJustify::Evenly => "space-evenly",
            };

            (
                classes,
                format!(
                    "align-items: {}; justify-content: {};",
                    align_css, justify_css
                ),
            )
        }
        crate::domain::LayoutType::Grid { columns, rows } => (
            format!("grid grid-cols-{} grid-rows-{}", columns, rows),
            String::new(),
        ),
        crate::domain::LayoutType::Stack => ("stack".to_string(), String::new()),
    };

    let anim_style = get_animation_style(&container.animation);
    let custom_style = container.style.to_css_string();

    // Fix Bug 7: Reorder styles.
    // For Container, specific props (gap, padding) should override generic custom_style
    // because Container has a dedicated, high-fidelity Spacing editor.
    let style = format!(
        "{} {} {} gap: {}px; padding: {}px {}px {}px {}px;",
        align_style,
        anim_style,
        custom_style,
        container.gap,
        container.padding.top,
        container.padding.right,
        container.padding.bottom,
        container.padding.left,
    );

    let container_id = container.id;

    // Handle dropping items into this container
    let on_drop = move |ev: leptos::ev::DragEvent| {
        // Fix: Use app_state for the third argument
        let app_state = AppState::expect_context();
        handle_drop(ev, Some(container_id), app_state);
    };

    let has_children = !container.children.is_empty();
    let preview_mode = AppState::expect_context().ui.preview_mode;

    // We need to clone these for the closures to use
    let children = container.children.clone();
    let container_id = container.id;

    view! {
        {move || {
            let style = style.clone();
            let layout_class = layout_class.clone();
            let children = children.clone();

            if !preview_mode.get() {
                view! {
                    <DropZone
                        zone_name=format!("container-{}", container_id)
                        drag_state=canvas_state.drag_state
                        on_drop=on_drop
                        config=None
                    >
                        <div
                            class=format!("canvas-container {}", layout_class)
                            class:hovered=move || {
                                 // Check if this container is being dragged over
                                 if let crate::builder::drag_drop::DragState::DraggingOver { drop_zone, .. } = canvas_state.drag_state.get() {
                                     drop_zone == format!("container-{}", container_id)
                                 } else {
                                     false
                                 }
                            }
                            style=style
                        >
                            {if has_children {
                                 let children = children.clone();
                                view! {
                                    <For
                                        each=move || children.clone()
                                        key=|comp| *comp.id()
                                        children=move |comp| {
                                            view! {
                                                <ComponentRenderer
                                                    component=comp
                                                    canvas_state=canvas_state
                                                />
                                            }
                                        }
                                    />
                                }.into_any()
                            } else {
                                view! {
                                    <div class="empty-container-placeholder">
                                        "Drop items here"
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    </DropZone>
                }.into_any()
            } else {
                view! {
                    <div class=format!("canvas-container {}", layout_class) style=style>
                         <For
                            each=move || children.clone()
                            key=|comp| *comp.id()
                            children=move |comp| {
                                view! {
                                    <ComponentRenderer
                                        component=comp
                                        canvas_state=canvas_state
                                    />
                                }
                            }
                        />
                    </div>
                }.into_any()
            }
        }}
    }
}

fn render_image(image: ImageComponent) -> impl IntoView {
    let anim_style = get_animation_style(&image.animation);
    let custom_style = image.style.to_css_string();

    view! {
        <img
            src=image.src
            alt=image.alt
            class="canvas-image"
            style:width=image.width
            style:height=image.height
            style:max-width="100%"
            style=format!("{} {}", anim_style, custom_style)
        />
    }
}

fn render_card(card: CardComponent, canvas_state: CanvasState) -> impl IntoView {
    let padding = card.padding;
    let border_radius = card.border_radius;
    let card_id = card.id;
    let children = card.children.clone();
    let has_children = !children.is_empty();
    let preview_mode = AppState::expect_context().ui.preview_mode;

    let anim_style = get_animation_style(&card.animation);
    let custom_style = card.style.to_css_string();

    // Fix Bug 7: Reorder styles
    let shadow_style = if card.shadow {
        "box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);"
    } else {
        ""
    };

    let style = format!(
        "{} {} padding: {}px; border-radius: {}px; {}",
        anim_style, custom_style, padding, border_radius, shadow_style
    );

    let border_class = if card.border {
        "canvas-card-bordered"
    } else {
        ""
    };

    // Handle dropping items into this card
    let on_drop = move |ev: leptos::ev::DragEvent| {
        // Fix: Use app_state for the third argument
        let app_state = AppState::expect_context();
        handle_drop(ev, Some(card_id), app_state);
    };

    view! {
        {move || {
            let style = style.clone();
            let children = children.clone();

            if !preview_mode.get() {
                view! {
                    <DropZone
                        zone_name=format!("container-{}", card_id)
                        drag_state=canvas_state.drag_state
                        on_drop=on_drop
                        config=None
                    >
                        <div
                            class=format!("canvas-card {} {}", border_class, if !has_children { "canvas-card-empty" } else { "" })
                            class:hovered=move || {
                                if let crate::builder::drag_drop::DragState::DraggingOver { drop_zone, .. } = canvas_state.drag_state.get() {
                                    drop_zone == format!("container-{}", card_id)
                                } else {
                                    false
                                }
                            }
                            style=style
                        >
                            {if has_children {
                                view! {
                                    <For
                                        each=move || children.clone()
                                        key=|comp| *comp.id()
                                        children=move |comp| {
                                            view! {
                                                <ComponentRenderer
                                                    component=comp
                                                    canvas_state=canvas_state
                                                />
                                            }
                                        }
                                    />
                                }.into_any()
                            } else {
                                view! {
                                    <div class="empty-container-placeholder">
                                        "Drop content here"
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    </DropZone>
                }.into_any()
            } else {
                view! {
                    <div class=format!("canvas-card {}", border_class) style=style>
                         <For
                            each=move || children.clone()
                            key=|comp| *comp.id()
                            children=move |comp| {
                                view! {
                                    <ComponentRenderer
                                        component=comp
                                        canvas_state=canvas_state
                                    />
                                }
                            }
                        />
                    </div>
                }.into_any()
            }
        }}
    }
}

fn render_custom(custom: CustomComponent) -> impl IntoView {
    let custom_style = custom.style.to_css_string();
    view! {
        <div class="canvas-custom" style=custom_style>
            <div class="custom-header">
                <span class="custom-name">{custom.name.clone()}</span>
            </div>
            <div class="custom-template" inner_html=custom.template></div>
        </div>
    }
}

fn render_divider(divider: DividerComponent) -> impl IntoView {
    let anim_style = get_animation_style(&divider.animation);
    let custom_style = divider.style.to_css_string();

    let orientation_class = match divider.orientation {
        crate::domain::DividerOrientation::Horizontal => "canvas-divider-horizontal",
        crate::domain::DividerOrientation::Vertical => "canvas-divider-vertical",
    };

    let size_style = match divider.orientation {
        crate::domain::DividerOrientation::Horizontal => {
            format!("border-top-width: {}px;", divider.thickness)
        }
        crate::domain::DividerOrientation::Vertical => {
            format!("border-left-width: {}px;", divider.thickness)
        }
    };

    view! {
        <div
            class=format!("canvas-divider {}", orientation_class)
            role="separator"
            style=format!("{} {} {}", size_style, anim_style, custom_style)
        ></div>
    }
}

fn render_checkbox(checkbox: CheckboxComponent) -> impl IntoView {
    let anim_style = get_animation_style(&checkbox.animation);
    let custom_style = checkbox.style.to_css_string();

    view! {
        <label
            class="canvas-checkbox"
            class:disabled=checkbox.disabled
            style=format!("{} {}", anim_style, custom_style)
        >
            <input type="checkbox" checked=checkbox.checked disabled=checkbox.disabled />
            <span class="canvas-checkbox-label">{checkbox.label}</span>
        </label>
    }
}

fn render_radio_group(radio: RadioGroupComponent) -> impl IntoView {
    let anim_style = get_animation_style(&radio.animation);
    let custom_style = radio.style.to_css_string();
    let group_name = format!("radio-group-{}", radio.id);
    let selected = radio.selected.clone();
    let disabled = radio.disabled;

    let options: Vec<String> = radio
        .options
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    view! {
        <div
            class="canvas-radio-group"
            role="radiogroup"
            style=format!("{} {}", anim_style, custom_style)
        >
            {options.into_iter().map(|opt| {
                let is_checked = !selected.is_empty() && selected == opt;
                let name = group_name.clone();
                view! {
                    <label class="canvas-radio-option" class:disabled=disabled>
                        <input type="radio" name=name value=opt.clone() checked=is_checked disabled=disabled />
                        <span class="canvas-radio-label">{opt.clone()}</span>
                    </label>
                }
            }).collect_view()}
        </div>
    }
}

fn render_switch(switch: SwitchComponent) -> impl IntoView {
    let anim_style = get_animation_style(&switch.animation);
    let custom_style = switch.style.to_css_string();

    view! {
        <label
            class="canvas-switch"
            class:disabled=switch.disabled
            style=format!("{} {}", anim_style, custom_style)
        >
            <span class="canvas-switch-track" class:checked=switch.checked>
                <input type="checkbox" checked=switch.checked disabled=switch.disabled />
                <span class="canvas-switch-thumb"></span>
            </span>
            <span class="canvas-switch-label">{switch.label}</span>
        </label>
    }
}

fn render_badge(badge: BadgeComponent) -> impl IntoView {
    let anim_style = get_animation_style(&badge.animation);
    let custom_style = badge.style.to_css_string();

    let variant_class = match badge.variant {
        crate::domain::BadgeVariant::Default => "badge-default",
        crate::domain::BadgeVariant::Primary => "badge-primary",
        crate::domain::BadgeVariant::Success => "badge-success",
        crate::domain::BadgeVariant::Warning => "badge-warning",
        crate::domain::BadgeVariant::Error => "badge-error",
    };

    view! {
        <span
            class=format!("canvas-badge {}", variant_class)
            style=format!("{} {}", anim_style, custom_style)
        >
            {badge.text}
        </span>
    }
}

fn render_progress(progress: ProgressComponent) -> impl IntoView {
    let anim_style = get_animation_style(&progress.animation);
    let custom_style = progress.style.to_css_string();

    let percent = if progress.max > 0.0 {
        (progress.value / progress.max * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    let label = format!("{:.0}%", percent);

    view! {
        <div
            class="canvas-progress"
            role="progressbar"
            style=format!("{} {}", anim_style, custom_style)
        >
            <div class="canvas-progress-track">
                <div class="canvas-progress-fill" style:width=format!("{}%", percent)></div>
            </div>
            {if progress.show_label {
                view! { <span class="canvas-progress-label">{label}</span> }.into_any()
            } else {
                view! { <span class="hidden"></span> }.into_any()
            }}
        </div>
    }
}
fn render_link(link: LinkComponent) -> impl IntoView {
    let anim_style = get_animation_style(&link.animation);
    let custom_style = link.style.to_css_string();

    view! {
        <a
            class="canvas-link"
            href=link.href
            style=format!("{} {}", anim_style, custom_style)
            on:click=|ev| ev.prevent_default()
        >
            {link.text}
        </a>
    }
}
