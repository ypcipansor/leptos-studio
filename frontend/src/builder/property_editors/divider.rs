use super::AnimationPropertyEditor;
use crate::builder::property_inputs::{EnumSelect, NumberInput};
use crate::builder::styling_system::StyleEditor;
use crate::domain::{ComponentId, DividerComponent, DividerOrientation};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn DividerPropertyEditor(id: ComponentId, divider: DividerComponent) -> impl IntoView {
    let app_state = AppState::expect_context();
    let canvas_state = app_state.canvas;

    let update_orientation = move |new_val: String| {
        canvas_state.record_snapshot("Update Divider Orientation");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Divider(d) = c {
                d.orientation = match new_val.as_str() {
                    "Vertical" => DividerOrientation::Vertical,
                    _ => DividerOrientation::Horizontal,
                };
            }
        });
    };

    let update_thickness = move |new_val: f64| {
        canvas_state.record_snapshot("Update Divider Thickness");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Divider(d) = c {
                d.thickness = new_val.max(1.0) as u32;
            }
        });
    };

    let current_orientation = match divider.orientation {
        DividerOrientation::Horizontal => "Horizontal",
        DividerOrientation::Vertical => "Vertical",
    }
    .to_string();

    let style = divider.style.clone();
    let animation = divider.animation.clone();

    view! {
        <div class="property-group">
            <div class="group-title">"Divider"</div>
            <EnumSelect
                value=current_orientation
                label="Orientation".to_string()
                options=vec!["Horizontal".to_string(), "Vertical".to_string()]
                on_change=update_orientation
            />
            <NumberInput
                value=divider.thickness as f64
                label="Thickness (px)".to_string()
                min_value=1.0
                step_value=1.0
                on_change=update_thickness
            />
        </div>

        <StyleEditor
            style=style
            on_change=move |new_style| {
                canvas_state.record_snapshot("Update Divider Style");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Divider(d) = c {
                        d.style = new_style;
                    }
                });
            }
        />

        <AnimationPropertyEditor
            _id=id
            animation=animation
            on_change=move |new_anim| {
                canvas_state.record_snapshot("Update Divider Animation");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Divider(d) = c {
                        d.animation = new_anim;
                    }
                });
            }
        />
    }
}
