//! Validation for serialized canvas components received from the frontend.

use serde_json::Value;

/// Component types that the frontend canvas can serialize.
///
/// This must stay in sync with `frontend/src/domain/component.rs`
/// (`ComponentType`): every variant of `CanvasComponent` serialises to exactly
/// one of these type tags, so a variant missing here makes the whole project
/// unsaveable — the frontend gets a 422 and the layout is lost. Adding a new
/// `CanvasComponent`/`ComponentType` variant therefore *requires* adding its
/// type tag here, to `frontend/src/builder/canvas/renderer.rs` and to every
/// exporter. `known_component_types_match_the_frontend` pins the list, so it
/// cannot drift silently.
const KNOWN_COMPONENT_TYPES: &[&str] = &[
    "Button",
    "Text",
    "Input",
    "Container",
    "Image",
    "Card",
    "Select",
    "Custom",
    "Divider",
    "Checkbox",
    "RadioGroup",
    "Switch",
    "Badge",
    "Progress",
    "Link",
];

/// Container-like component types whose `children` are validated recursively.
const CONTAINER_TYPES: &[&str] = &["Container", "Card"];

/// Validate a serialized component. Each component must be a single-key object
/// whose key is a known component type; container-like types may have a
/// `children` array which is validated recursively.
pub fn validate_component(value: &Value, path: &str) -> Result<(), String> {
    let obj = value
        .as_object()
        .ok_or_else(|| format!("{path}: component must be a JSON object"))?;

    let mut keys = obj.keys();
    let Some(type_key) = keys.next() else {
        return Err(format!("{path}: component object is empty"));
    };

    if keys.next().is_some() {
        return Err(format!(
            "{path}: component has more than one type tag ({type_key}, ...)"
        ));
    }

    if !KNOWN_COMPONENT_TYPES.contains(&type_key.as_str()) {
        return Err(format!("{path}: unknown component type '{type_key}'"));
    }

    let component = obj.get(type_key).cloned().unwrap_or(Value::Null);

    if CONTAINER_TYPES.contains(&type_key.as_str())
        && let Some(children) = component.get("children")
    {
        let children = children
            .as_array()
            .ok_or_else(|| format!("{path}.{type_key}: children must be an array"))?;
        for (i, child) in children.iter().enumerate() {
            validate_component(child, &format!("{path}.{type_key}.children[{i}]"))?;
        }
    }

    Ok(())
}

/// Validate an entire `layout` array. Every element must be a valid component.
pub fn validate_layout(layout: &Value) -> Result<(), String> {
    let components = layout
        .as_array()
        .ok_or_else(|| "layout must be an array".to_string())?;

    for (i, component) in components.iter().enumerate() {
        validate_component(component, &format!("layout[{i}]"))?;
    }

    Ok(())
}

/// Count components in a layout, recursively including container children.
pub fn count_components(layout: &Value) -> usize {
    count_component_list(layout.as_array().map(Vec::as_slice).unwrap_or(&[]))
}

fn count_component_list(components: &[Value]) -> usize {
    components.iter().map(count_component).sum::<usize>()
}

fn count_component(value: &Value) -> usize {
    let children_total = value
        .as_object()
        .and_then(|obj| {
            obj.iter().find_map(|(_, inner)| {
                inner
                    .get("children")
                    .and_then(|c| c.as_array())
                    .map(|children| children.iter().map(count_component).sum::<usize>())
            })
        })
        .unwrap_or(0);

    1 + children_total
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A link with a real href, as the frontend serialises it.
    fn link_json() -> Value {
        json!({
            "Link": {
                "id": "0e5b0f3a-0000-4000-8000-000000000001",
                "text": "Docs",
                "href": "https://example.com",
                "animation": null,
                "bindings": {},
                "style": {}
            }
        })
    }

    #[test]
    fn valid_layout_accepted() {
        let layout = json!([
            { "Button": { "label": "OK" } },
            { "Container": { "children": [ { "Text": { "content": "hi" } } ] } },
            { "Divider": { "orientation": "Horizontal" } },
        ]);
        assert!(validate_layout(&layout).is_ok());
        assert_eq!(count_components(&layout), 4);
    }

    #[test]
    fn unknown_type_rejected() {
        let layout = json!([ { "Widget": {} } ]);
        let err = validate_layout(&layout).unwrap_err();
        assert!(err.contains("unknown component type"));
    }

    #[test]
    fn nested_unknown_type_rejected() {
        let layout = json!([
            { "Container": { "children": [ { "NotAType": {} } ] } }
        ]);
        assert!(validate_layout(&layout).is_err());
    }

    #[test]
    fn invalid_shape_rejected() {
        assert!(validate_layout(&json!("not-an-array")).is_err());
        assert!(validate_layout(&json!([{}])).is_err());
        assert!(validate_layout(&json!([42])).is_err());
    }

    #[test]
    fn counting_includes_nested_children() {
        let layout = json!([
            { "Container": {
                "children": [
                    { "Card": { "children": [ { "Button": {} }, { "Badge": {} } ] } }
                ]
            } }
        ]);
        assert_eq!(count_components(&layout), 4);
    }

    /// Regression: a layout containing a Link was rejected as an unknown type,
    /// so every project that used a hyperlink failed to save with a 422.
    #[test]
    fn link_at_the_root_is_accepted() {
        let layout = json!([link_json()]);
        assert!(
            validate_layout(&layout).is_ok(),
            "a Link at the root must be accepted"
        );
        assert_eq!(count_components(&layout), 1);
    }

    /// Links are legal container children too; the recursive path must accept
    /// them, not just the top-level list.
    #[test]
    fn link_inside_a_container_is_accepted() {
        let layout = json!([
            { "Container": { "children": [ link_json() ] } }
        ]);
        assert!(
            validate_layout(&layout).is_ok(),
            "a Link inside a Container must be accepted"
        );
        assert_eq!(count_components(&layout), 2);
    }

    /// Same for Cards, the other recursive container type.
    #[test]
    fn link_inside_a_card_is_accepted() {
        let layout = json!([
            { "Card": { "children": [ link_json() ] } }
        ]);
        assert!(
            validate_layout(&layout).is_ok(),
            "a Link inside a Card must be accepted"
        );
        assert_eq!(count_components(&layout), 2);
    }

    #[test]
    fn link_deeply_nested_is_accepted() {
        let layout = json!([
            { "Container": {
                "children": [
                    { "Card": { "children": [ link_json() ] } }
                ]
            } }
        ]);
        assert!(validate_layout(&layout).is_ok());
        assert_eq!(count_components(&layout), 3);
    }

    /// A genuinely unknown type must still be rejected — accepting `Link` must
    /// not weaken the allowlist.
    #[test]
    fn genuinely_unknown_type_is_still_rejected() {
        for tag in ["Widget", "Video", "link", "LINK", "Iframe"] {
            let layout = json!([{ tag: {} }]);
            let err = validate_layout(&layout).unwrap_err();
            assert!(
                err.contains("unknown component type") && err.contains(tag),
                "type '{tag}' must still be rejected, got: {err}"
            );
        }
    }

    /// The allowlist is only correct if it covers the frontend's `ComponentType`
    /// enum exactly. Read that enum from its source and compare, so adding a
    /// frontend variant without a backend tag fails here instead of surfacing as
    /// a 422 in production.
    #[test]
    fn known_component_types_match_the_frontend() {
        let path = crate::paths::repo_root().join("frontend/src/domain/component.rs");
        let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "cannot read {} to cross-check component types: {e}; this guard must not silently pass",
                path.display()
            )
        });

        let start = source
            .find("pub enum ComponentType {")
            .expect("ComponentType enum must exist in the frontend domain")
            + "pub enum ComponentType {".len();
        let rest = &source[start..];
        let end = rest.find('}').expect("ComponentType enum must be closed");
        let enum_body = &rest[..end];

        let frontend: Vec<&str> = enum_body
            .split(',')
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect();

        assert!(
            !frontend.is_empty(),
            "the ComponentType enum parsed as empty; the parser needs updating"
        );

        for variant in &frontend {
            assert!(
                KNOWN_COMPONENT_TYPES.contains(variant),
                "frontend ComponentType::{variant} is missing from the backend \
                 KNOWN_COMPONENT_TYPES allowlist; projects containing it cannot be saved"
            );
        }
        for known in KNOWN_COMPONENT_TYPES {
            assert!(
                frontend.contains(known),
                "backend allowlist entry '{known}' has no frontend ComponentType variant"
            );
        }
    }
}
