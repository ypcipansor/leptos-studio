use super::AnimationPropertyEditor;
use crate::builder::property_inputs::{BoolCheckbox, StringInput};
use crate::builder::styling_system::StyleEditor;
use crate::domain::{CheckboxComponent, ComponentId};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn CheckboxPropertyEditor(id: ComponentId, checkbox: CheckboxComponent) -> impl IntoView {
    let app_state = AppState::expect_context();
    let canvas_state = app_state.canvas;

    let update_label = move |new_val: String| {
        canvas_state.record_snapshot("Update Checkbox Label");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Checkbox(cb) = c {
                cb.label = new_val;
            }
        });
    };

    let update_checked = move |new_val: bool| {
        canvas_state.record_snapshot("Update Checkbox Checked");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Checkbox(cb) = c {
                cb.checked = new_val;
            }
        });
    };

    let update_disabled = move |new_val: bool| {
        canvas_state.record_snapshot("Update Checkbox Disabled");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Checkbox(cb) = c {
                cb.disabled = new_val;
            }
        });
    };

    let style = checkbox.style.clone();
    let animation = checkbox.animation.clone();

    view! {
        <div class="property-group">
            <div class="group-title">"Checkbox"</div>
            <StringInput
                value=checkbox.label.clone()
                label="Label".to_string()
                on_change=update_label
            />
            <BoolCheckbox
                checked=checkbox.checked
                label="Checked".to_string()
                on_change=update_checked
            />
            <BoolCheckbox
                checked=checkbox.disabled
                label="Disabled".to_string()
                on_change=update_disabled
            />
        </div>

        <StyleEditor
            style=style
            on_change=move |new_style| {
                canvas_state.record_snapshot("Update Checkbox Style");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Checkbox(cb) = c {
                        cb.style = new_style;
                    }
                });
            }
        />

        <AnimationPropertyEditor
            _id=id
            animation=animation
            on_change=move |new_anim| {
                canvas_state.record_snapshot("Update Checkbox Animation");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Checkbox(cb) = c {
                        cb.animation = new_anim;
                    }
                });
            }
        />
    }
}
