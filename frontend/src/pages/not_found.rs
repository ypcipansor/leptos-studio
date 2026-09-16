use leptos::prelude::*;

/// Fallback page for unknown routes.
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="not-found-page">
            <div class="not-found-card">
                <div class="not-found-code">"404"</div>
                <h1 class="not-found-title">"Page not found"</h1>
                <p class="not-found-text">
                    "The page you are looking for doesn't exist or has been moved."
                </p>
                <div class="not-found-actions">
                    <a href="/" class="btn btn-primary">"Back to projects"</a>
                </div>
            </div>
        </div>
    }
}
