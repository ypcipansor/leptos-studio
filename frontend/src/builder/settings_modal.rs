use crate::state::app_state::{AppState, Theme};
use leptos::prelude::*;

#[component]
pub fn SettingsModal() -> impl IntoView {
    use crate::builder::hooks::use_escape_key::use_escape_key;

    let app_state = AppState::expect_context();
    let show = app_state.ui.show_settings_modal;
    let settings = app_state.settings;

    use_escape_key(show, move || show.set(false));

    view! {
        <Show when=move || show.get()>
            <div
                class="modal-backdrop"
                role="presentation"
                on:click=move |_| show.set(false)
            >
                <div
                    class="modal-content settings-modal"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="settings-modal-title"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <div class="modal-header">
                        <h3 id="settings-modal-title">"Settings"</h3>
                        <button
                            class="close-btn"
                            aria-label="Close settings"
                            on:click=move |_| show.set(false)
                        >
                            "×"
                        </button>
                    </div>
                    <div class="modal-body">
                        <div class="setting-item">
                            <label>"Theme"</label>
                            <select
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    settings.update(|s| s.theme = match val.as_str() {
                                        "Dark" => Theme::Dark,
                                        "Custom" => Theme::Custom,
                                        _ => Theme::Light,
                                    });
                                    app_state.save_settings();
                                }
                                prop:value=move || match settings.get().theme {
                                    Theme::Dark => "Dark",
                                    Theme::Custom => "Custom",
                                    _ => "Light",
                                }
                            >
                                <option value="Light">"Light"</option>
                                <option value="Dark">"Dark"</option>
                                <option value="Custom">"Custom"</option>
                            </select>
                        </div>

                        <div class="setting-item">
                            <label>"Auto-Save"</label>
                            <div class="toggle-switch">
                                <input
                                    type="checkbox"
                                    id="autosave-toggle"
                                    prop:checked=move || settings.get().auto_save
                                    on:change=move |ev| {
                                        let checked = event_target_checked(&ev);
                                        settings.update(|s| s.auto_save = checked);
                                        app_state.save_settings();
                                    }
                                />
                                <label for="autosave-toggle" class="toggle-slider"></label>
                            </div>
                            <p class="setting-hint">
                                "Automatically save project changes every 2 seconds."
                            </p>
                        </div>

                        <div class="setting-item">
                            <label>"Export Preset"</label>
                            <select
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    use crate::state::app_state::ExportPreset;
                                    settings.update(|s| s.export_preset = match val.as_str() {
                                        "ThawUi" => ExportPreset::ThawUi,
                                        "LeptosMaterial" => ExportPreset::LeptosMaterial,
                                        "LeptosUse" => ExportPreset::LeptosUse,
                                        _ => ExportPreset::Plain,
                                    });
                                    app_state.save_settings();
                                }
                                prop:value=move || match settings.get().export_preset {
                                    crate::state::app_state::ExportPreset::ThawUi => "ThawUi",
                                    crate::state::app_state::ExportPreset::LeptosMaterial => "LeptosMaterial",
                                    crate::state::app_state::ExportPreset::LeptosUse => "LeptosUse",
                                    _ => "Plain",
                                }
                            >
                                <option value="Plain">"Plain Leptos"</option>
                                <option value="ThawUi">"Thaw UI"</option>
                                <option value="LeptosMaterial">"Leptos Material"</option>
                                <option value="LeptosUse">"Leptos Use"</option>
                            </select>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
