#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

use leptos_studio::domain::{ButtonComponent, CanvasComponent};
use leptos_studio::services::export_service::{CodeGenerator, JsonCodeGenerator};

wasm_bindgen_test_configure!(run_in_browser);

// Basic smoke test to ensure the crate works under wasm32 and
// export services can run without panicking in a browser environment.
#[wasm_bindgen_test]
fn export_service_works_in_wasm() {
    let generator = JsonCodeGenerator;
    let button = CanvasComponent::Button(ButtonComponent::new("From WASM".to_string()));
    let code = generator
        .generate(&[button], &[])
        .expect("JSON generation should succeed");

    assert!(code.contains("From WASM"));
}
