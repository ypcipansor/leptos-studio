use super::AnimationPropertyEditor;
use crate::builder::property_inputs::{EnumSelect, StringInput};
use crate::builder::styling_system::StyleEditor;
use crate::domain::{BadgeComponent, BadgeVariant, ComponentId};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn BadgePropertyEditor(id: ComponentId, badge: BadgeComponent) -> impl IntoView {
    let app_state = AppState::expect_context();
    let canvas_state = app_state.canvas;

    let update_text = move |new_val: String| {
        canvas_state.record_snapshot("Update Badge Text");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Badge(b) = c {
                b.text = new_val;
            }
        });
    };

    let update_variant = move |new_val: String| {
        canvas_state.record_snapshot("Update Badge Variant");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Badge(b) = c {
                b.variant = match new_val.as_str() {
                    "Primary" => BadgeVariant::Primary,
                    "Success" => BadgeVariant::Success,
                    "Warning" => BadgeVariant::Warning,
                    "Error" => BadgeVariant::Error,
                    _ => BadgeVariant::Default,
                };
            }
        });
    };

    let current_variant = match badge.variant {
        BadgeVariant::Default => "Default",
        BadgeVariant::Primary => "Primary",
        BadgeVariant::Success => "Success",
        BadgeVariant::Warning => "Warning",
        BadgeVariant::Error => "Error",
    }
    .to_string();

    let style = badge.style.clone();
    let animation = badge.animation.clone();

    view! {
        <div class="property-group">
            <div class="group-title">"Badge"</div>
            <StringInput
                value=badge.text.clone()
                label="Text".to_string()
                on_change=update_text
            />
            <EnumSelect
                value=current_variant
                label="Variant".to_string()
                options=vec![
                    "Default".to_string(),
                    "Primary".to_string(),
                    "Success".to_string(),
                    "Warning".to_string(),
                    "Error".to_string(),
                ]
                on_change=update_variant
            />
        </div>

        <StyleEditor
            style=style
            on_change=move |new_style| {
                canvas_state.record_snapshot("Update Badge Style");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Badge(b) = c {
                        b.style = new_style;
                    }
                });
            }
        />

        <AnimationPropertyEditor
            _id=id
            animation=animation
            on_change=move |new_anim| {
                canvas_state.record_snapshot("Update Badge Animation");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Badge(b) = c {
                        b.animation = new_anim;
                    }
                });
            }
        />
    }
}
