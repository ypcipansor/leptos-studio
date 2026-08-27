use super::property_editors::{
    BadgePropertyEditor, ButtonPropertyEditor, CardPropertyEditor, CheckboxPropertyEditor,
    ContainerPropertyEditor, CustomPropertyEditor, DividerPropertyEditor, ImagePropertyEditor,
    InputPropertyEditor, ProgressPropertyEditor, RadioGroupPropertyEditor, SelectPropertyEditor,
    SwitchPropertyEditor, TextPropertyEditor,
};
use crate::domain::CanvasComponent;
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn PropertyEditor() -> impl IntoView {
    // Get app state from context
    let app_state = AppState::expect_context();
    let canvas_state = app_state.canvas;

    let delete_selected = move |_| {
        if let Some(id) = canvas_state.selected.get() {
            canvas_state.remove_component(&id);
            canvas_state.clear_selection();
            app_state.ui.notify(crate::state::Notification::info(
                "Component removed".to_string(),
            ));
        }
    };

    view! {
        <section class="property-editor">
            <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 2px solid #e2e8f0; margin-bottom: 16px; padding-bottom: 12px;">
                <h3 style="margin: 0; border: none; padding: 0;">{"Property Editor"}</h3>
                {move || if canvas_state.selected.get().is_some() {
                    view! {
                        <button
                            class="btn btn-danger btn-sm"
                            on:click=delete_selected
                            title="Remove selected component"
                        >
                            "Delete"
                        </button>
                    }.into_any()
                } else {
                    ().into_any()
                }}
            </div>

            {move || {
                if let Some(selected_id) = canvas_state.selected.get() {
                    if let Some(comp) = canvas_state.get_component(&selected_id) {
                        match comp {
                            CanvasComponent::Button(btn) => {
                                view! {
                                    <ButtonPropertyEditor id=selected_id button=btn />
                                }.into_any()
                            },
                            CanvasComponent::Text(txt) => {
                                view! {
                                    <TextPropertyEditor id=selected_id text=txt />
                                }.into_any()
                            },
                            CanvasComponent::Input(inp) => {
                                view! {
                                    <InputPropertyEditor id=selected_id input=inp />
                                }.into_any()
                            },
                            CanvasComponent::Container(container) => {
                                view! {
                                    <ContainerPropertyEditor id=selected_id container=container />
                                }.into_any()
                            },
                            CanvasComponent::Image(img) => {
                                view! {
                                    <ImagePropertyEditor id=selected_id image=img />
                                }.into_any()
                            },
                            CanvasComponent::Card(card) => {
                                view! {
                                    <CardPropertyEditor id=selected_id card=card />
                                }.into_any()
                            },
                            CanvasComponent::Select(sel) => {
                                view! {
                                    <SelectPropertyEditor id=selected_id select=sel />
                                }.into_any()
                            },
                            CanvasComponent::Custom(custom) => {
                                view! {
                                    <CustomPropertyEditor id=selected_id custom=custom />
                                }.into_any()
                            },
                            CanvasComponent::Divider(divider) => {
                                view! {
                                    <DividerPropertyEditor id=selected_id divider=divider />
                                }.into_any()
                            },
                            CanvasComponent::Checkbox(checkbox) => {
                                view! {
                                    <CheckboxPropertyEditor id=selected_id checkbox=checkbox />
                                }.into_any()
                            },
                            CanvasComponent::RadioGroup(radio) => {
                                view! {
                                    <RadioGroupPropertyEditor id=selected_id radio=radio />
                                }.into_any()
                            },
                            CanvasComponent::Switch(switch) => {
                                view! {
                                    <SwitchPropertyEditor id=selected_id switch=switch />
                                }.into_any()
                            },
                            CanvasComponent::Badge(badge) => {
                                view! {
                                    <BadgePropertyEditor id=selected_id badge=badge />
                                }.into_any()
                            },
                            CanvasComponent::Progress(progress) => {
                                view! {
                                    <ProgressPropertyEditor id=selected_id progress=progress />
                                }.into_any()
                            },
                        }
                    } else {
                        view! { <div><p>{"Component not found"}</p></div> }.into_any()
                    }
                } else {
                    view! { <div><p>{"Select a component to edit properties"}</p></div> }.into_any()
                }
            }}
        </section>
    }
}
