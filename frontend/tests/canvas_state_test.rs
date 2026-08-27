use leptos::prelude::*;
use leptos_studio::domain::ComponentId;
use leptos_studio::state::app_state::CanvasState;

#[test]
fn select_single_replaces_multi_select() {
    let owner = Owner::new();
    owner.with(|| {
        let canvas = CanvasState::new();
        let a = ComponentId::new();
        let b = ComponentId::new();

        canvas.toggle_multi_select(a);
        canvas.toggle_multi_select(b);
        assert_eq!(canvas.selected_components.get().len(), 2);

        canvas.select_single(a);
        assert_eq!(canvas.selected_components.get(), vec![a]);
        assert_eq!(canvas.selected.get(), Some(a));
    });
}

#[test]
fn toggle_multi_select_adds_and_removes() {
    let owner = Owner::new();
    owner.with(|| {
        let canvas = CanvasState::new();
        let a = ComponentId::new();

        canvas.toggle_multi_select(a);
        assert!(canvas.is_selected(a));

        canvas.toggle_multi_select(a);
        assert_eq!(canvas.selected_components.get().len(), 0);
        // Primary selection remains the last-clicked component
        assert!(canvas.is_selected(a));
    });
}

#[test]
fn clear_selection_resets_both() {
    let owner = Owner::new();
    owner.with(|| {
        let canvas = CanvasState::new();
        let a = ComponentId::new();
        canvas.select_single(a);

        canvas.clear_selection();
        assert_eq!(canvas.selected.get(), None);
        assert!(canvas.selected_components.get().is_empty());
        assert!(!canvas.is_selected(a));
    });
}

#[test]
fn zoom_is_clamped_and_resettable() {
    let owner = Owner::new();
    owner.with(|| {
        let canvas = CanvasState::new();
        assert_eq!(canvas.zoom.get(), 1.0);

        for _ in 0..50 {
            canvas.zoom_by(1.1);
        }
        assert_eq!(canvas.zoom.get(), CanvasState::MAX_ZOOM);

        for _ in 0..100 {
            canvas.zoom_by(1.0 / 1.1);
        }
        assert_eq!(canvas.zoom.get(), CanvasState::MIN_ZOOM);

        canvas.reset_zoom();
        assert_eq!(canvas.zoom.get(), 1.0);
    });
}
