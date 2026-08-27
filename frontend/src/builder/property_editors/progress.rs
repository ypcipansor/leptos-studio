use super::AnimationPropertyEditor;
use crate::builder::property_inputs::{BoolCheckbox, NumberInput};
use crate::builder::styling_system::StyleEditor;
use crate::domain::{ComponentId, ProgressComponent};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn ProgressPropertyEditor(id: ComponentId, progress: ProgressComponent) -> impl IntoView {
    let app_state = AppState::expect_context();
    let canvas_state = app_state.canvas;

    let update_value = move |new_val: f64| {
        canvas_state.record_snapshot("Update Progress Value");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Progress(p) = c {
                p.value = new_val;
            }
        });
    };

    let update_max = move |new_val: f64| {
        canvas_state.record_snapshot("Update Progress Max");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Progress(p) = c {
                p.max = new_val.max(1.0);
            }
        });
    };

    let update_show_label = move |new_val: bool| {
        canvas_state.record_snapshot("Update Progress Label");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Progress(p) = c {
                p.show_label = new_val;
            }
        });
    };

    let style = progress.style.clone();
    let animation = progress.animation.clone();

    view! {
        <div class="property-group">
            <div class="group-title">"Progress"</div>
            <NumberInput
                value=progress.value
                label="Value".to_string()
                min_value=0.0
                on_change=update_value
            />
            <NumberInput
                value=progress.max
                label="Max".to_string()
                min_value=1.0
                on_change=update_max
            />
            <BoolCheckbox
                checked=progress.show_label
                label="Show label".to_string()
                on_change=update_show_label
            />
        </div>

        <StyleEditor
            style=style
            on_change=move |new_style| {
                canvas_state.record_snapshot("Update Progress Style");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Progress(p) = c {
                        p.style = new_style;
                    }
                });
            }
        />

        <AnimationPropertyEditor
            _id=id
            animation=animation
            on_change=move |new_anim| {
                canvas_state.record_snapshot("Update Progress Animation");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Progress(p) = c {
                        p.animation = new_anim;
                    }
                });
            }
        />
    }
}
