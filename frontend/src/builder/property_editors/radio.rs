use super::AnimationPropertyEditor;
use crate::builder::property_inputs::{BoolCheckbox, StringInput};
use crate::builder::styling_system::StyleEditor;
use crate::domain::{ComponentId, RadioGroupComponent};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn RadioGroupPropertyEditor(id: ComponentId, radio: RadioGroupComponent) -> impl IntoView {
    let app_state = AppState::expect_context();
    let canvas_state = app_state.canvas;

    let update_options = move |new_val: String| {
        canvas_state.record_snapshot("Update Radio Options");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::RadioGroup(r) = c {
                r.options = new_val;
            }
        });
    };

    let update_selected = move |new_val: String| {
        canvas_state.record_snapshot("Update Radio Selected");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::RadioGroup(r) = c {
                r.selected = new_val;
            }
        });
    };

    let update_disabled = move |new_val: bool| {
        canvas_state.record_snapshot("Update Radio Disabled");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::RadioGroup(r) = c {
                r.disabled = new_val;
            }
        });
    };

    let style = radio.style.clone();
    let animation = radio.animation.clone();

    view! {
        <div class="property-group">
            <div class="group-title">"Radio Group"</div>
            <StringInput
                value=radio.options.clone()
                label="Options (comma separated)".to_string()
                on_change=update_options
            />
            <StringInput
                value=radio.selected.clone()
                label="Selected option".to_string()
                on_change=update_selected
            />
            <BoolCheckbox
                checked=radio.disabled
                label="Disabled".to_string()
                on_change=update_disabled
            />
        </div>

        <StyleEditor
            style=style
            on_change=move |new_style| {
                canvas_state.record_snapshot("Update Radio Style");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::RadioGroup(r) = c {
                        r.style = new_style;
                    }
                });
            }
        />

        <AnimationPropertyEditor
            _id=id
            animation=animation
            on_change=move |new_anim| {
                canvas_state.record_snapshot("Update Radio Animation");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::RadioGroup(r) = c {
                        r.animation = new_anim;
                    }
                });
            }
        />
    }
}
