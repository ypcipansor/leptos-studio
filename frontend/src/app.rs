use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::pages::dashboard::DashboardPage;
use crate::pages::editor::EditorPage;
use crate::pages::not_found::NotFoundPage;
use crate::services::analytics_service::AnalyticsService;
use crate::services::event_bus::EventBus;
use crate::services::template_service::TemplateService;
use crate::state::app_state::AppState;
use crate::state::derived::DerivedState;

#[component]
pub fn App() -> impl IntoView {
    // Initialize global AppState context
    AppState::provide_context();
    let app_state = AppState::expect_context();

    // Initialize services
    let _event_bus = StoredValue::new(EventBus::new());
    let _template_service = StoredValue::new(TemplateService::new());

    // Provide AnalyticsService to context so it can be used by child components
    AnalyticsService::provide_context();
    let _analytics_service = StoredValue::new(AnalyticsService::use_context());

    // Create and provide derived state for memoized computations
    DerivedState::provide_context(app_state);

    view! {
        <Router>
            <Routes fallback=|| view! { <NotFoundPage /> }>
                <Route path=path!("/") view=DashboardPage />
                <Route path=path!("/editor/:id") view=EditorPage />
            </Routes>
        </Router>
    }
}
