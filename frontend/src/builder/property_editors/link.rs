use super::AnimationPropertyEditor;
use crate::builder::property_inputs::StringInput;
use crate::builder::styling_system::StyleEditor;
use crate::domain::{ComponentId, LinkComponent};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn LinkPropertyEditor(id: ComponentId, link: LinkComponent) -> impl IntoView {
    let app_state = AppState::expect_context();
    let canvas_state = app_state.canvas;

    let update_text = move |new_val: String| {
        canvas_state.record_snapshot("Update Link Text");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Link(l) = c {
                l.text = new_val;
            }
        });
    };

    let update_href = move |new_val: String| {
        canvas_state.record_snapshot("Update Link Href");
        canvas_state.update_component(&id, |c| {
            if let crate::domain::CanvasComponent::Link(l) = c {
                l.href = new_val;
            }
        });
    };

    let style = link.style.clone();
    let animation = link.animation.clone();

    view! {
        <div class="property-group">
            <div class="group-title">"Link"</div>
            <StringInput
                value=link.text.clone()
                label="Text".to_string()
                on_change=update_text
            />
            <StringInput
                value=link.href.clone()
                label="Href".to_string()
                on_change=update_href
            />
        </div>

        <StyleEditor
            style=style
            on_change=move |new_style| {
                canvas_state.record_snapshot("Update Link Style");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Link(l) = c {
                        l.style = new_style;
                    }
                });
            }
        />

        <AnimationPropertyEditor
            _id=id
            animation=animation
            on_change=move |new_anim| {
                canvas_state.record_snapshot("Update Link Animation");
                canvas_state.update_component(&id, |c| {
                    if let crate::domain::CanvasComponent::Link(l) = c {
                        l.animation = new_anim;
                    }
                });
            }
        />
    }
}
