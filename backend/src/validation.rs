//! Validation for serialized canvas components received from the frontend.

use serde_json::Value;

/// Component types that the frontend canvas can serialize. Keep in sync with
/// `frontend/src/domain/component.rs` (`ComponentType`).
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
}
