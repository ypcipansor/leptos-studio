use super::AnimationPropertyEditor;
use crate::builder::property_inputs::{BoolCheckbox, StringInput};
use crate::builder::styling_system::StyleEditor;
use crate::domain::{ComponentId, SwitchComponent};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn SwitchPropertyEditor(id: ComponentId, switch: SwitchComponent) -> impl IntoView {
    let app_state = AppState::expect_context();
    let canvas_state = app_state.canvas;

    let update_label = move |new_val: String| {
        canvas_state.record_snapshot("Update Switch Label");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Switch(s) = c {
                s.label = new_val;
            }
        });
    };

    let update_checked = move |new_val: bool| {
        canvas_state.record_snapshot("Update Switch Checked");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Switch(s) = c {
                s.checked = new_val;
            }
        });
    };

    let update_disabled = move |new_val: bool| {
        canvas_state.record_snapshot("Update Switch Disabled");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Switch(s) = c {
                s.disabled = new_val;
            }
        });
    };

    let style = switch.style.clone();
    let animation = switch.animation.clone();

    view! {
        <div class="property-group">
            <div class="group-title">"Switch"</div>
            <StringInput
                value=switch.label.clone()
                label="Label".to_string()
                on_change=update_label
            />
            <BoolCheckbox
                checked=switch.checked
                label="On".to_string()
                on_change=update_checked
            />
            <BoolCheckbox
                checked=switch.disabled
                label="Disabled".to_string()
                on_change=update_disabled
            />
        </div>

        <StyleEditor
            style=style
            on_change=move |new_style| {
                canvas_state.record_snapshot("Update Switch Style");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Switch(s) = c {
                        s.style = new_style;
                    }
                });
            }
        />

        <AnimationPropertyEditor
            _id=id
            animation=animation
            on_change=move |new_anim| {
                canvas_state.record_snapshot("Update Switch Animation");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Switch(s) = c {
                        s.animation = new_anim;
                    }
                });
            }
        />
    }
}
