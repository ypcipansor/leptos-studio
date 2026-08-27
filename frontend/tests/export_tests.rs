use leptos_studio::domain::{
    ButtonComponent, CanvasComponent, ContainerComponent, FlexDirection, LayoutType,
};
use leptos_studio::services::export_service::{CodeGenerator, LeptosCodeGenerator};
use leptos_studio::state::ExportPreset;

#[test]
fn test_leptos_generator_structure() {
    let generator = LeptosCodeGenerator::new(ExportPreset::Plain);

    // Create nested structure: Container -> Button
    let mut container = ContainerComponent::new();
    container.layout = LayoutType::Flex {
        direction: FlexDirection::Row,
        wrap: false,
        align_items: Default::default(),
        justify_content: Default::default(),
    };

    let button = CanvasComponent::Button(ButtonComponent::new("Nested Button".to_string()));
    container.children.push(button);

    let components = vec![CanvasComponent::Container(container)];

    let code = generator
        .generate(&components, &[])
        .expect("Failed to generate code");

    println!("{}", code);

    // Check for essential parts
    assert!(code.contains("use leptos::prelude::*;"));
    assert!(code.contains("#[component]"));
    assert!(code.contains("pub fn App()"));

    // Check container structure
    assert!(code.contains("class=\"container flex-row\""));

    // Check nested button
    assert!(code.contains("Nested Button"));
    assert!(code.contains("<button"));
}

use leptos_studio::domain::{
    BadgeComponent, CheckboxComponent, DividerComponent, ImageComponent, InputComponent,
    ProgressComponent, RadioGroupComponent, SelectComponent, SwitchComponent, TextComponent,
};
use leptos_studio::services::export_service::{HtmlCodeGenerator, MarkdownCodeGenerator};

fn all_components() -> Vec<CanvasComponent> {
    let mut container = ContainerComponent::new();
    container
        .children
        .push(CanvasComponent::Button(ButtonComponent::new(
            "Nested".to_string(),
        )));

    vec![
        CanvasComponent::Button(ButtonComponent::new("Save".to_string())),
        CanvasComponent::Text(TextComponent::new("Hello".to_string())),
        CanvasComponent::Input(InputComponent::new()),
        CanvasComponent::Select(SelectComponent::new()),
        CanvasComponent::Image(ImageComponent::new(
            "img.png".to_string(),
            "Image".to_string(),
        )),
        CanvasComponent::Divider(DividerComponent::new()),
        CanvasComponent::Checkbox(CheckboxComponent::new("Check".to_string())),
        CanvasComponent::RadioGroup(RadioGroupComponent::new()),
        CanvasComponent::Switch(SwitchComponent::new("Toggle".to_string())),
        CanvasComponent::Badge(BadgeComponent::new("Badge".to_string())),
        CanvasComponent::Progress(ProgressComponent::new()),
        CanvasComponent::Container(container),
    ]
}

#[test]
fn test_all_components_entire_codegen() {
    let components = all_components();
    assert_eq!(components.len(), 12);

    let leptos = LeptosCodeGenerator::new(leptos_studio::state::ExportPreset::Plain)
        .generate(&components, &[])
        .unwrap();
    let html = HtmlCodeGenerator.generate(&components, &[]).unwrap();
    let markdown = MarkdownCodeGenerator.generate(&components, &[]).unwrap();

    // Every generator must successfully cover the full component set
    for opt in ["Option 1", "Option 2"] {
        assert!(leptos.contains(opt), "Leptos missing radio option {opt}");
        assert!(html.contains(opt), "HTML missing radio option {opt}");
        assert!(
            markdown.contains(opt),
            "Markdown missing radio option {opt}"
        );
    }
    assert!(leptos.contains("badge"));
    assert!(html.contains("badge"));
    assert!(markdown.contains("Badge"));
}
