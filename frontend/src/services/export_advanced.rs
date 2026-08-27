//! Advanced Export Formats
//!
//! Additional export generators for JSON Schema, TypeScript types,
//! React components, and other formats.

use crate::domain::{AppError, AppResult, CanvasComponent, Variable, VariableType};

use super::CodeGenerator;

/// JSON Schema generator for component validation
pub struct JsonSchemaGenerator;

impl CodeGenerator for JsonSchemaGenerator {
    fn generate(
        &self,
        components: &[CanvasComponent],
        _variables: &[Variable],
    ) -> AppResult<String> {
        let schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Leptos Studio Layout",
            "description": "JSON Schema for Leptos Studio component layouts",
            "type": "array",
            "items": {
                "$ref": "#/definitions/CanvasComponent"
            },
            "definitions": {
                "CanvasComponent": {
                    "oneOf": [
                        { "$ref": "#/definitions/ButtonComponent" },
                        { "$ref": "#/definitions/TextComponent" },
                        { "$ref": "#/definitions/InputComponent" },
                        { "$ref": "#/definitions/ContainerComponent" },
                        { "$ref": "#/definitions/ImageComponent" },
                        { "$ref": "#/definitions/CardComponent" },
                        { "$ref": "#/definitions/SelectComponent" },
                        { "$ref": "#/definitions/CustomComponent" }
                    ]
                },
                "SelectComponent": {
                    "type": "object",
                    "required": ["Select"],
                    "properties": {
                        "Select": {
                            "type": "object",
                            "required": ["id", "options", "placeholder", "disabled"],
                            "properties": {
                                "id": { "type": "string", "format": "uuid" },
                                "options": { "type": "string" },
                                "placeholder": { "type": "string" },
                                "disabled": { "type": "boolean" }
                            }
                        }
                    }
                },
                "ButtonComponent": {
                    "type": "object",
                    "required": ["Button"],
                    "properties": {
                        "Button": {
                            "type": "object",
                            "required": ["id", "label", "variant", "size", "disabled"],
                            "properties": {
                                "id": { "type": "string", "format": "uuid" },
                                "label": { "type": "string" },
                                "variant": {
                                    "type": "string",
                                    "enum": ["Primary", "Secondary", "Outline", "Ghost"]
                                },
                                "size": {
                                    "type": "string",
                                    "enum": ["Small", "Medium", "Large"]
                                },
                                "disabled": { "type": "boolean" },
                                "on_click": { "type": ["string", "null"] }
                            }
                        }
                    }
                },
                "TextComponent": {
                    "type": "object",
                    "required": ["Text"],
                    "properties": {
                        "Text": {
                            "type": "object",
                            "required": ["id", "content", "style", "tag"],
                            "properties": {
                                "id": { "type": "string", "format": "uuid" },
                                "content": { "type": "string" },
                                "style": {
                                    "type": "string",
                                    "enum": ["Heading1", "Heading2", "Heading3", "Body", "Caption"]
                                },
                                "tag": {
                                    "type": "string",
                                    "enum": ["H1", "H2", "H3", "P", "Span"]
                                }
                            }
                        }
                    }
                },
                "InputComponent": {
                    "type": "object",
                    "required": ["Input"],
                    "properties": {
                        "Input": {
                            "type": "object",
                            "required": ["id", "placeholder", "input_type", "required", "disabled"],
                            "properties": {
                                "id": { "type": "string", "format": "uuid" },
                                "placeholder": { "type": "string" },
                                "input_type": {
                                    "type": "string",
                                    "enum": ["Text", "Password", "Email", "Number", "Tel"]
                                },
                                "required": { "type": "boolean" },
                                "disabled": { "type": "boolean" }
                            }
                        }
                    }
                },
                "ContainerComponent": {
                    "type": "object",
                    "required": ["Container"],
                    "properties": {
                        "Container": {
                            "type": "object",
                            "required": ["id", "children", "layout", "gap", "padding"],
                            "properties": {
                                "id": { "type": "string", "format": "uuid" },
                                "children": {
                                    "type": "array",
                                    "items": { "$ref": "#/definitions/CanvasComponent" }
                                },
                                "layout": { "$ref": "#/definitions/LayoutType" },
                                "gap": { "type": "integer", "minimum": 0 },
                                "padding": { "$ref": "#/definitions/Spacing" }
                            }
                        }
                    }
                },
                "ImageComponent": {
                    "type": "object",
                    "required": ["Image"],
                    "properties": {
                        "Image": {
                            "type": "object",
                            "required": ["id", "src", "alt"],
                            "properties": {
                                "id": { "type": "string", "format": "uuid" },
                                "src": { "type": "string" },
                                "alt": { "type": "string" },
                                "width": { "type": ["string", "null"] },
                                "height": { "type": ["string", "null"] }
                            }
                        }
                    }
                },
                "CardComponent": {
                    "type": "object",
                    "required": ["Card"],
                    "properties": {
                        "Card": {
                            "type": "object",
                            "required": ["id", "children", "padding", "shadow", "border", "border_radius"],
                            "properties": {
                                "id": { "type": "string", "format": "uuid" },
                                "children": {
                                    "type": "array",
                                    "items": { "$ref": "#/definitions/CanvasComponent" }
                                },
                                "padding": { "type": "integer", "minimum": 0 },
                                "shadow": { "type": "boolean" },
                                "border": { "type": "boolean" },
                                "border_radius": { "type": "integer", "minimum": 0 }
                            }
                        }
                    }
                },
                "CustomComponent": {
                    "type": "object",
                    "required": ["Custom"],
                    "properties": {
                        "Custom": {
                            "type": "object",
                            "required": ["id", "name", "template", "props"],
                            "properties": {
                                "id": { "type": "string", "format": "uuid" },
                                "name": { "type": "string" },
                                "template": { "type": "string" },
                                "props": { "type": "object" }
                            }
                        }
                    }
                },
                "LayoutType": {
                    "oneOf": [
                        {
                            "type": "object",
                            "properties": {
                                "Flex": {
                                    "type": "object",
                                    "properties": {
                                        "direction": { "enum": ["Row", "Column"] },
                                        "wrap": { "type": "boolean" }
                                    }
                                }
                            }
                        },
                        {
                            "type": "object",
                            "properties": {
                                "Grid": {
                                    "type": "object",
                                    "properties": {
                                        "columns": { "type": "integer" },
                                        "rows": { "type": "integer" }
                                    }
                                }
                            }
                        },
                        { "const": "Stack" }
                    ]
                },
                "Spacing": {
                    "type": "object",
                    "required": ["top", "right", "bottom", "left"],
                    "properties": {
                        "top": { "type": "integer", "minimum": 0 },
                        "right": { "type": "integer", "minimum": 0 },
                        "bottom": { "type": "integer", "minimum": 0 },
                        "left": { "type": "integer", "minimum": 0 }
                    }
                }
            },
            "examples": [serde_json::to_value(components).unwrap_or_default()]
        });

        serde_json::to_string_pretty(&schema)
            .map_err(|e| AppError::Export(format!("Failed to generate JSON Schema: {}", e)))
    }

    fn file_extension(&self) -> &str {
        "schema.json"
    }
}

/// TypeScript types generator
pub struct TypeScriptGenerator;

impl CodeGenerator for TypeScriptGenerator {
    fn generate(
        &self,
        _components: &[CanvasComponent],
        _variables: &[Variable],
    ) -> AppResult<String> {
        let types = r#"/**
 * Leptos Studio - TypeScript Type Definitions
 * Auto-generated from component layout
 */

// Component ID type
export type ComponentId = string;

// Button variants
export type ButtonVariant = 'Primary' | 'Secondary' | 'Outline' | 'Ghost';

// Button sizes
export type ButtonSize = 'Small' | 'Medium' | 'Large';

// Text styles
export type TextStyle = 'Heading1' | 'Heading2' | 'Heading3' | 'Body' | 'Caption';

// HTML tags for text
export type TextTag = 'H1' | 'H2' | 'H3' | 'P' | 'Span';

// Input types
export type InputType = 'Text' | 'Password' | 'Email' | 'Number' | 'Tel';

// Flex direction
export type FlexDirection = 'Row' | 'Column';

// Layout types
export type LayoutType = 
  | { Flex: { direction: FlexDirection; wrap: boolean } }
  | { Grid: { columns: number; rows: number } }
  | 'Stack';

// Spacing
export interface Spacing {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

// Button component
export interface ButtonComponent {
  id: ComponentId;
  label: string;
  variant: ButtonVariant;
  size: ButtonSize;
  disabled: boolean;
  on_click?: string | null;
}

// Text component
export interface TextComponent {
  id: ComponentId;
  content: string;
  style: TextStyle;
  tag: TextTag;
}

// Input component
export interface InputComponent {
  id: ComponentId;
  placeholder: string;
  input_type: InputType;
  required: boolean;
  disabled: boolean;
}

// Select component
export interface SelectComponent {
  id: ComponentId;
  options: string;
  placeholder: string;
  disabled: boolean;
}

// Container component
export interface ContainerComponent {
  id: ComponentId;
  children: CanvasComponent[];
  layout: LayoutType;
  gap: number;
  padding: Spacing;
}

// Custom component
export interface CustomComponent {
  id: ComponentId;
  name: string;
  template: string;
  props: Record<string, PropValue>;
}

// Property value types
export type PropValue = 
  | { String: string }
  | { Number: number }
  | { Boolean: boolean }
  | 'Null';

// Image component
export interface ImageComponent {
  id: ComponentId;
  src: string;
  alt: string;
  width?: string | null;
  height?: string | null;
}

// Card component
export interface CardComponent {
  id: ComponentId;
  children: CanvasComponent[];
  padding: number;
  shadow: boolean;
  border: boolean;
  border_radius: number;
}

// Canvas component union type
export type CanvasComponent = 
  | { Button: ButtonComponent }
  | { Text: TextComponent }
  | { Input: InputComponent }
  | { Container: ContainerComponent }
  | { Image: ImageComponent }
  | { Card: CardComponent }
  | { Select: SelectComponent }
  | { Custom: CustomComponent };

// Layout type (array of components)
export type Layout = CanvasComponent[];

// Project export type
export interface Project {
  name: string;
  description?: string;
  layout: Layout;
  theme: string;
  created_at: string;
  updated_at: string;
}

// Helper type guards
export function isButton(c: CanvasComponent): c is { Button: ButtonComponent } {
  return 'Button' in c;
}

export function isText(c: CanvasComponent): c is { Text: TextComponent } {
  return 'Text' in c;
}

export function isInput(c: CanvasComponent): c is { Input: InputComponent } {
  return 'Input' in c;
}

export function isContainer(c: CanvasComponent): c is { Container: ContainerComponent } {
  return 'Container' in c;
}

export function isImage(c: CanvasComponent): c is { Image: ImageComponent } {
  return 'Image' in c;
}

export function isCard(c: CanvasComponent): c is { Card: CardComponent } {
  return 'Card' in c;
}

export function isSelect(c: CanvasComponent): c is { Select: SelectComponent } {
  return 'Select' in c;
}

export function isCustom(c: CanvasComponent): c is { Custom: CustomComponent } {
  return 'Custom' in c;
}
"#;

        Ok(types.to_string())
    }

    fn file_extension(&self) -> &str {
        "d.ts"
    }
}

/// React component generator
pub struct ReactGenerator;

impl CodeGenerator for ReactGenerator {
    fn generate(
        &self,
        components: &[CanvasComponent],
        variables: &[Variable],
    ) -> AppResult<String> {
        let mut output = String::new();

        // Imports
        output.push_str("import React from 'react';\n\n");

        // Generate component
        output.push_str("export function GeneratedLayout() {\n");
        output.push_str("  // Signals / State\n");

        let vars_init = if variables.is_empty() {
            "{}".to_string()
        } else {
            let entries: Vec<String> = variables
                .iter()
                .map(|v| {
                    let val = match v.data_type {
                        VariableType::String => format!("'{}'", v.default_value),
                        _ => v.default_value.clone(),
                    };
                    format!("{}: {}", v.name, val)
                })
                .collect();
            format!("{{ {} }}", entries.join(", "))
        };

        output.push_str(&format!(
            "  const [vars, setVars] = React.useState({});\n\n",
            vars_init
        ));
        output.push_str("  return (\n");
        output.push_str("    <>\n");

        for component in components {
            Self::generate_react(component, &mut output, 3)?;
        }

        output.push_str("    </>\n");
        output.push_str("  );\n");
        output.push_str("}\n\n");

        output.push_str("export default GeneratedLayout;\n");

        Ok(output)
    }

    fn file_extension(&self) -> &str {
        "tsx"
    }
}

impl ReactGenerator {
    fn generate_react(
        component: &CanvasComponent,
        output: &mut String,
        indent_level: usize,
    ) -> AppResult<()> {
        let indent = "  ".repeat(indent_level);

        match component {
            CanvasComponent::Button(btn) => {
                let id_attr = if let Some(bind) = btn.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = btn.bindings.get("custom_css_classes") {
                    format!(" className={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let variant_class = match btn.variant {
                    crate::domain::ButtonVariant::Primary => "btn-primary",
                    crate::domain::ButtonVariant::Secondary => "btn-secondary",
                    crate::domain::ButtonVariant::Outline => "btn-outline",
                    crate::domain::ButtonVariant::Ghost => "btn-ghost",
                };

                let size_class = match btn.size {
                    crate::domain::ButtonSize::Small => "btn-sm",
                    crate::domain::ButtonSize::Medium => "btn-md",
                    crate::domain::ButtonSize::Large => "btn-lg",
                };

                let label_expr = if let Some(bind) = btn.bindings.get("label") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    btn.label.clone()
                };

                let disabled_expr = if let Some(bind) = btn.bindings.get("disabled") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    btn.disabled.to_string()
                };

                output.push_str(&format!(
                    "{}<button{} {} className=\"{} {}\" disabled={{{}}}>{}</button>\n",
                    indent,
                    id_attr,
                    class_attr,
                    variant_class,
                    size_class,
                    disabled_expr,
                    label_expr
                ));
            }
            CanvasComponent::Text(txt) => {
                let id_attr = if let Some(bind) = txt.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = txt.bindings.get("custom_css_classes") {
                    format!(" className={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let tag = match txt.tag {
                    crate::domain::TextTag::H1 => "h1",
                    crate::domain::TextTag::H2 => "h2",
                    crate::domain::TextTag::H3 => "h3",
                    crate::domain::TextTag::P => "p",
                    crate::domain::TextTag::Span => "span",
                };

                let content_expr = if let Some(bind) = txt.bindings.get("content") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    txt.content.clone()
                };

                output.push_str(&format!(
                    "{}<{} {}{}>{}</{}>\n",
                    indent, tag, id_attr, class_attr, content_expr, tag
                ));
            }
            CanvasComponent::Input(inp) => {
                let id_attr = if let Some(bind) = inp.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = inp.bindings.get("custom_css_classes") {
                    format!(" className={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let input_type = match inp.input_type {
                    crate::domain::InputType::Text => "text",
                    crate::domain::InputType::Password => "password",
                    crate::domain::InputType::Email => "email",
                    crate::domain::InputType::Number => "number",
                    crate::domain::InputType::Tel => "tel",
                };

                let placeholder_expr = if let Some(bind) = inp.bindings.get("placeholder") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    format!("\"{}\"", inp.placeholder)
                };

                let disabled_expr = if let Some(bind) = inp.bindings.get("disabled") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    inp.disabled.to_string()
                };

                output.push_str(&format!(
                    "{}<input{} {} type=\"{}\" placeholder={} required={{{}}} disabled={{{}}} />\n",
                    indent,
                    id_attr,
                    class_attr,
                    input_type,
                    placeholder_expr,
                    inp.required,
                    disabled_expr
                ));
            }
            CanvasComponent::Select(sel) => {
                let id_attr = if let Some(bind) = sel.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = sel.bindings.get("custom_css_classes") {
                    format!(" className={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let disabled_expr = if let Some(bind) = sel.bindings.get("disabled") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    sel.disabled.to_string()
                };

                output.push_str(&format!(
                    "{}<select{} {} disabled={{{}}}>\n",
                    indent, id_attr, class_attr, disabled_expr
                ));
                if !sel.placeholder.is_empty() {
                    let placeholder_expr = if let Some(bind) = sel.bindings.get("placeholder") {
                        format!("{{vars['{}']}}", bind)
                    } else {
                        format!("\" {} \"", sel.placeholder)
                    };
                    output.push_str(&format!(
                        "{}  <option value=\"\" disabled selected>{{{}}}</option>\n",
                        indent, placeholder_expr
                    ));
                }

                if let Some(bind) = sel.bindings.get("options") {
                    output.push_str(&format!(
                        "{}  {{vars['{}']?.split(',').map(opt => <option key={{opt}} value={{opt.trim()}}>{{opt.trim()}}</option>)}}\n",
                        indent, bind
                    ));
                } else {
                    for option in sel.options.split(',') {
                        let opt = option.trim();
                        output.push_str(&format!(
                            "{}  <option value=\"{}\">{{\" {} \"}}</option>\n",
                            indent, opt, opt
                        ));
                    }
                }
                output.push_str(&format!("{}</select>\n", indent));
            }
            CanvasComponent::Container(container) => {
                let layout_style = match &container.layout {
                    crate::domain::LayoutType::Flex {
                        direction,
                        align_items,
                        justify_content,
                        ..
                    } => {
                        let dir = match direction {
                            crate::domain::FlexDirection::Row => "row",
                            crate::domain::FlexDirection::Column => "column",
                        };
                        let align = match align_items {
                            crate::domain::FlexAlign::Start => "flex-start",
                            crate::domain::FlexAlign::Center => "center",
                            crate::domain::FlexAlign::End => "flex-end",
                            crate::domain::FlexAlign::Stretch => "stretch",
                            crate::domain::FlexAlign::Baseline => "baseline",
                        };
                        let justify = match justify_content {
                            crate::domain::FlexJustify::Start => "flex-start",
                            crate::domain::FlexJustify::Center => "center",
                            crate::domain::FlexJustify::End => "flex-end",
                            crate::domain::FlexJustify::Between => "space-between",
                            crate::domain::FlexJustify::Around => "space-around",
                            crate::domain::FlexJustify::Evenly => "space-evenly",
                        };
                        format!(
                            "display: 'flex', flexDirection: '{}', alignItems: '{}', justifyContent: '{}'",
                            dir, align, justify
                        )
                    }
                    crate::domain::LayoutType::Grid { columns, rows } => {
                        format!(
                            "display: 'grid', gridTemplateColumns: 'repeat({}, 1fr)', gridTemplateRows: 'repeat({}, auto)'",
                            columns, rows
                        )
                    }
                    crate::domain::LayoutType::Stack => {
                        "display: 'flex', flexDirection: 'column'".to_string()
                    }
                };

                let style = format!(
                    "{{ {}, gap: '{}px', padding: '{}px {}px {}px {}px' }}",
                    layout_style,
                    container.gap,
                    container.padding.top,
                    container.padding.right,
                    container.padding.bottom,
                    container.padding.left
                );

                let id_attr = if let Some(bind) = container.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = container.bindings.get("custom_css_classes") {
                    format!(" className={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "{}<div{} {} style={{{}}}>\n",
                    indent, id_attr, class_attr, style
                ));

                for child in &container.children {
                    Self::generate_react(child, output, indent_level + 1)?;
                }

                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Image(img) => {
                let id_attr = if let Some(bind) = img.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = img.bindings.get("custom_css_classes") {
                    format!(" className={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let width_attr = img
                    .width
                    .as_ref()
                    .map_or(String::new(), |w| format!(" width=\"{}\"", w));
                let height_attr = img
                    .height
                    .as_ref()
                    .map_or(String::new(), |h| format!(" height=\"{}\"", h));
                let src_val = if let Some(bind) = img.bindings.get("src") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    format!("\"{}\"", img.src)
                };

                let alt_val = if let Some(bind) = img.bindings.get("alt") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    format!("\"{}\"", img.alt)
                };

                output.push_str(&format!(
                    "{}<img{} {} src={} alt={}{}{} />\n",
                    indent, id_attr, class_attr, src_val, alt_val, width_attr, height_attr
                ));
            }
            CanvasComponent::Card(card) => {
                let mut style_parts = vec![
                    format!("padding: '{}px'", card.padding),
                    format!("borderRadius: '{}px'", card.border_radius),
                ];
                if card.shadow {
                    style_parts.push("boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)'".to_string());
                }
                if card.border {
                    style_parts.push("border: '1px solid #e5e7eb'".to_string());
                }
                let style_str = style_parts.join(", ");

                let id_attr = if let Some(bind) = card.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = card.bindings.get("custom_css_classes") {
                    format!(" className={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "{}<div{} {} style={{{{ {} }}}}>\n",
                    indent, id_attr, class_attr, style_str
                ));
                for child in &card.children {
                    Self::generate_react(child, output, indent_level + 1)?;
                }
                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Custom(custom) => {
                let id_attr = if let Some(bind) = custom.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = custom.bindings.get("custom_css_classes") {
                    format!(" className={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                output.push_str(&format!("{}<!-- Custom: {} -->\n", indent, custom.name));
                output.push_str(&format!(
                    "{}<div{}{} dangerouslySetInnerHTML={{{{ __html: `{}` }}}} />\n",
                    indent, id_attr, class_attr, custom.template
                ));
            }
            CanvasComponent::Divider(divider) => {
                let style = match divider.orientation {
                    crate::domain::DividerOrientation::Horizontal => format!(
                        "border: none; border-top: {}px solid #e5e7eb; margin: 8px 0;",
                        divider.thickness
                    ),
                    crate::domain::DividerOrientation::Vertical => format!(
                        "border: none; border-left: {}px solid #e5e7eb; margin: 0 8px; align-self: stretch;",
                        divider.thickness
                    ),
                };
                output.push_str(&format!("{}<hr style={{{{ `{}` }}}} />\n", indent, style));
            }
            CanvasComponent::Checkbox(checkbox) => {
                output.push_str(&format!(
                    "{}<label className=\"checkbox\">\n{}  <input type=\"checkbox\" defaultChecked={{{}}} disabled={{{}}} />\n{}  {}\n{}</label>\n",
                    indent, indent, checkbox.checked, checkbox.disabled, indent, checkbox.label, indent
                ));
            }
            CanvasComponent::RadioGroup(radio) => {
                let name = format!("radio-{}", radio.id);
                output.push_str(&format!("{}<div className=\"radio-group\">\n", indent));
                for opt in radio
                    .options
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    let checked = !radio.selected.is_empty() && radio.selected == opt;
                    output.push_str(&format!(
                        "{}  <label><input type=\"radio\" name=\"{}\" defaultChecked={{{}}} disabled={{{}}} /> {}</label>\n",
                        indent, name, checked, radio.disabled, opt
                    ));
                }
                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Switch(switch) => {
                output.push_str(&format!(
                    "{}<label className=\"switch\">\n{}  <input type=\"checkbox\" defaultChecked={{{}}} disabled={{{}}} />\n{}  {}\n{}</label>\n",
                    indent, indent, switch.checked, switch.disabled, indent, switch.label, indent
                ));
            }
            CanvasComponent::Badge(badge) => {
                let variant = match badge.variant {
                    crate::domain::BadgeVariant::Default => "default",
                    crate::domain::BadgeVariant::Primary => "primary",
                    crate::domain::BadgeVariant::Success => "success",
                    crate::domain::BadgeVariant::Warning => "warning",
                    crate::domain::BadgeVariant::Error => "error",
                };
                output.push_str(&format!(
                    "{}<span className=\"badge badge-{}\">{}</span>\n",
                    indent, variant, badge.text
                ));
            }
            CanvasComponent::Progress(progress) => {
                output.push_str(&format!(
                    "{}<progress value={{{}}} max={{{}}} style={{{{ width: \"100%\" }}}} />\n",
                    indent, progress.value, progress.max
                ));
            }
        }

        Ok(())
    }
}

/// Vue component generator
pub struct VueGenerator;

impl CodeGenerator for VueGenerator {
    fn generate(
        &self,
        components: &[CanvasComponent],
        variables: &[Variable],
    ) -> AppResult<String> {
        let mut template = String::from("<template>\n  <div class=\"generated-layout\">\n");

        for component in components {
            Self::generate_vue(component, &mut template, 2)?;
        }

        template.push_str("  </div>\n</template>\n\n");

        // Script section
        template.push_str("<script setup lang=\"ts\">\n");
        template.push_str("import { ref } from 'vue';\n");
        template.push_str("// Generated by Leptos Studio\n");

        let vars_init = if variables.is_empty() {
            "{}".to_string()
        } else {
            let entries: Vec<String> = variables
                .iter()
                .map(|v| {
                    let val = match v.data_type {
                        VariableType::String => format!("'{}'", v.default_value),
                        _ => v.default_value.clone(),
                    };
                    format!("{}: {}", v.name, val)
                })
                .collect();
            format!("{{ {} }}", entries.join(", "))
        };

        template.push_str(&format!(
            "const vars = ref<Record<string, any>>({});\n",
            vars_init
        ));
        template.push_str("</script>\n\n");

        // Style section
        template.push_str("<style scoped>\n");
        template.push_str(".generated-layout {\n");
        template.push_str("  /* Add your styles here */\n");
        template.push_str("}\n");
        template.push_str("</style>\n");

        Ok(template)
    }

    fn file_extension(&self) -> &str {
        "vue"
    }
}

impl VueGenerator {
    fn generate_vue(
        component: &CanvasComponent,
        output: &mut String,
        indent_level: usize,
    ) -> AppResult<()> {
        let indent = "  ".repeat(indent_level);

        match component {
            CanvasComponent::Button(btn) => {
                let id_attr = if let Some(bind) = btn.bindings.get("id") {
                    format!(" :id=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = btn.bindings.get("custom_css_classes") {
                    format!(" :class=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let label_expr = if let Some(bind) = btn.bindings.get("label") {
                    format!("{{{{ vars['{}'] }}}}", bind)
                } else {
                    btn.label.clone()
                };

                let disabled_attr = if let Some(bind) = btn.bindings.get("disabled") {
                    format!(":disabled=\"vars['{}']\"", bind)
                } else {
                    format!(":disabled=\"{}\"", btn.disabled)
                };

                output.push_str(&format!(
                    "{}<button{}{} {}>{}</button>\n",
                    indent, id_attr, class_attr, disabled_attr, label_expr
                ));
            }
            CanvasComponent::Text(txt) => {
                let id_attr = if let Some(bind) = txt.bindings.get("id") {
                    format!(" :id=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = txt.bindings.get("custom_css_classes") {
                    format!(" :class=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let tag = match txt.tag {
                    crate::domain::TextTag::H1 => "h1",
                    crate::domain::TextTag::H2 => "h2",
                    crate::domain::TextTag::H3 => "h3",
                    crate::domain::TextTag::P => "p",
                    crate::domain::TextTag::Span => "span",
                };

                let content_expr = if let Some(bind) = txt.bindings.get("content") {
                    format!("{{{{ vars['{}'] }}}}", bind)
                } else {
                    txt.content.clone()
                };

                output.push_str(&format!(
                    "{}<{} {}{}>{}</{}>\n",
                    indent, tag, id_attr, class_attr, content_expr, tag
                ));
            }
            CanvasComponent::Input(inp) => {
                let id_attr = if let Some(bind) = inp.bindings.get("id") {
                    format!(" :id=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = inp.bindings.get("custom_css_classes") {
                    format!(" :class=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let input_type = match inp.input_type {
                    crate::domain::InputType::Text => "text",
                    crate::domain::InputType::Password => "password",
                    crate::domain::InputType::Email => "email",
                    crate::domain::InputType::Number => "number",
                    crate::domain::InputType::Tel => "tel",
                };

                let placeholder_attr = if let Some(bind) = inp.bindings.get("placeholder") {
                    format!(":placeholder=\"vars['{}']\"", bind)
                } else {
                    format!("placeholder=\"{}\"", inp.placeholder)
                };

                let disabled_attr = if let Some(bind) = inp.bindings.get("disabled") {
                    format!(":disabled=\"vars['{}']\"", bind)
                } else {
                    format!(":disabled=\"{}\"", inp.disabled)
                };

                output.push_str(&format!(
                    "{}<input{}{} type=\"{}\" {} :required=\"{}\" {} />\n",
                    indent,
                    id_attr,
                    class_attr,
                    input_type,
                    placeholder_attr,
                    inp.required,
                    disabled_attr
                ));
            }
            CanvasComponent::Select(sel) => {
                let id_attr = if let Some(bind) = sel.bindings.get("id") {
                    format!(" :id=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = sel.bindings.get("custom_css_classes") {
                    format!(" :class=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let disabled_attr = if let Some(bind) = sel.bindings.get("disabled") {
                    format!(":disabled=\"vars['{}']\"", bind)
                } else {
                    format!(":disabled=\"{}\"", sel.disabled)
                };

                output.push_str(&format!(
                    "{}<select{}{} {}>\n",
                    indent, id_attr, class_attr, disabled_attr
                ));
                if !sel.placeholder.is_empty() {
                    let placeholder_expr = if let Some(bind) = sel.bindings.get("placeholder") {
                        format!("{{{{ vars['{}'] }}}}", bind)
                    } else {
                        sel.placeholder.clone()
                    };
                    output.push_str(&format!(
                        "{}  <option value=\"\" disabled selected>{}</option>\n",
                        indent, placeholder_expr
                    ));
                }

                if let Some(bind) = sel.bindings.get("options") {
                    output.push_str(&format!(
                        "{}  <option v-for=\"opt in vars['{}']?.split(',')\" :key=\"opt\" :value=\"opt.trim()\">{{{{ opt.trim() }}}}</option>\n",
                        indent, bind
                    ));
                } else {
                    for option in sel.options.split(',') {
                        let opt = option.trim();
                        output.push_str(&format!(
                            "{}  <option value=\"{}\">{}</option>\n",
                            indent, opt, opt
                        ));
                    }
                }
                output.push_str(&format!("{}</select>\n", indent));
            }
            CanvasComponent::Container(container) => {
                let id_attr = if let Some(bind) = container.bindings.get("id") {
                    format!(" :id=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = container.bindings.get("custom_css_classes") {
                    format!(" :class=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                output.push_str(&format!("{}<div{}{}>\n", indent, id_attr, class_attr));
                for child in &container.children {
                    Self::generate_vue(child, output, indent_level + 1)?;
                }
                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Image(img) => {
                let id_attr = if let Some(bind) = img.bindings.get("id") {
                    format!(" :id=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = img.bindings.get("custom_css_classes") {
                    format!(" :class=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let width_attr = img
                    .width
                    .as_ref()
                    .map_or(String::new(), |w| format!(" width=\"{}\"", w));
                let height_attr = img
                    .height
                    .as_ref()
                    .map_or(String::new(), |h| format!(" height=\"{}\"", h));
                let src_val = if let Some(bind) = img.bindings.get("src") {
                    format!(":src=\"vars['{}']\"", bind)
                } else {
                    format!("src=\"{}\"", img.src)
                };

                let alt_val = if let Some(bind) = img.bindings.get("alt") {
                    format!(":alt=\"vars['{}']\"", bind)
                } else {
                    format!("alt=\"{}\"", img.alt)
                };

                output.push_str(&format!(
                    "{}<img{}{} {} {}{}{} />\n",
                    indent, id_attr, class_attr, src_val, alt_val, width_attr, height_attr
                ));
            }
            CanvasComponent::Card(card) => {
                let shadow_class = if card.shadow { "shadow-md" } else { "" };
                let border_class = if card.border { "border" } else { "" };

                let id_attr = if let Some(bind) = card.bindings.get("id") {
                    format!(" :id=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = card.bindings.get("custom_css_classes") {
                    format!(" :class=\"vars['{}']\"", bind)
                } else {
                    format!(" class=\"card {} {}\"", shadow_class, border_class)
                };

                output.push_str(&format!(
                    "{}<div{}{} style=\"padding: {}px; border-radius: {}px;\">\n",
                    indent, id_attr, class_attr, card.padding, card.border_radius
                ));
                for child in &card.children {
                    Self::generate_vue(child, output, indent_level + 1)?;
                }
                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Custom(custom) => {
                let id_attr = if let Some(bind) = custom.bindings.get("id") {
                    format!(" :id=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = custom.bindings.get("custom_css_classes") {
                    format!(" :class=\"vars['{}']\"", bind)
                } else {
                    String::new()
                };

                output.push_str(&format!("{}<!-- {} -->\n", indent, custom.name));
                output.push_str(&format!(
                    "{}<div{}{} v-html=\"`{}`\"></div>\n",
                    indent, id_attr, class_attr, custom.template
                ));
            }
            CanvasComponent::Divider(divider) => {
                let style = match divider.orientation {
                    crate::domain::DividerOrientation::Horizontal => format!(
                        "border: none; border-top: {}px solid #e5e7eb; margin: 8px 0;",
                        divider.thickness
                    ),
                    crate::domain::DividerOrientation::Vertical => format!(
                        "border: none; border-left: {}px solid #e5e7eb; margin: 0 8px; align-self: stretch;",
                        divider.thickness
                    ),
                };
                output.push_str(&format!("{}<hr style=\"{}\" />\n", indent, style));
            }
            CanvasComponent::Checkbox(checkbox) => {
                output.push_str(&format!(
                    "{}<label>\n{}  <input type=\"checkbox\" :checked=\"{}\" :disabled=\"{}\" />\n{}  {}\n{}</label>\n",
                    indent, indent, checkbox.checked, checkbox.disabled, indent, checkbox.label, indent
                ));
            }
            CanvasComponent::RadioGroup(radio) => {
                let name = format!("radio-{}", radio.id);
                output.push_str(&format!("{}<div class=\"radio-group\">\n", indent));
                for opt in radio
                    .options
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    let checked = !radio.selected.is_empty() && radio.selected == opt;
                    output.push_str(&format!(
                        "{}  <label><input type=\"radio\" name=\"{}\" :checked=\"{}\" :disabled=\"{}\" /> {}</label>\n",
                        indent, name, checked, radio.disabled, opt
                    ));
                }
                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Switch(switch) => {
                output.push_str(&format!(
                    "{}<label class=\"switch\">\n{}  <input type=\"checkbox\" :checked=\"{}\" :disabled=\"{}\" />\n{}  {}\n{}</label>\n",
                    indent, indent, switch.checked, switch.disabled, indent, switch.label, indent
                ));
            }
            CanvasComponent::Badge(badge) => {
                let variant = match badge.variant {
                    crate::domain::BadgeVariant::Default => "default",
                    crate::domain::BadgeVariant::Primary => "primary",
                    crate::domain::BadgeVariant::Success => "success",
                    crate::domain::BadgeVariant::Warning => "warning",
                    crate::domain::BadgeVariant::Error => "error",
                };
                output.push_str(&format!(
                    "{}<span class=\"badge badge-{}\">{}</span>\n",
                    indent, variant, badge.text
                ));
            }
            CanvasComponent::Progress(progress) => {
                output.push_str(&format!(
                    "{}<progress :value=\"{}\" :max=\"{}\" style=\"width: 100%;\"></progress>\n",
                    indent, progress.value, progress.max
                ));
            }
        }

        Ok(())
    }
}

/// CSS generator (extracts styles)
pub struct CssGenerator;

impl CodeGenerator for CssGenerator {
    fn generate(
        &self,
        components: &[CanvasComponent],
        _variables: &[Variable],
    ) -> AppResult<String> {
        let mut css = String::from("/* Generated by Leptos Studio */\n\n");

        // Basic button styles
        css.push_str(".btn-primary {\n");
        css.push_str("  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);\n");
        css.push_str("  color: white;\n");
        css.push_str("  border: none;\n");
        css.push_str("  padding: 8px 16px;\n");
        css.push_str("  border-radius: 6px;\n");
        css.push_str("  cursor: pointer;\n");
        css.push_str("}\n\n");

        css.push_str(".btn-secondary {\n");
        css.push_str("  background: white;\n");
        css.push_str("  color: #475569;\n");
        css.push_str("  border: 1px solid #cbd5e1;\n");
        css.push_str("  padding: 8px 16px;\n");
        css.push_str("  border-radius: 6px;\n");
        css.push_str("  cursor: pointer;\n");
        css.push_str("}\n\n");

        css.push_str(".btn-outline {\n");
        css.push_str("  background: transparent;\n");
        css.push_str("  color: #3b82f6;\n");
        css.push_str("  border: 1px solid #3b82f6;\n");
        css.push_str("  padding: 8px 16px;\n");
        css.push_str("  border-radius: 6px;\n");
        css.push_str("  cursor: pointer;\n");
        css.push_str("}\n\n");

        css.push_str(".btn-ghost {\n");
        css.push_str("  background: transparent;\n");
        css.push_str("  color: #6b7280;\n");
        css.push_str("  border: none;\n");
        css.push_str("  padding: 8px 16px;\n");
        css.push_str("  cursor: pointer;\n");
        css.push_str("}\n\n");

        // Size modifiers
        css.push_str(".btn-sm { padding: 4px 12px; font-size: 12px; }\n");
        css.push_str(".btn-md { padding: 8px 16px; font-size: 14px; }\n");
        css.push_str(".btn-lg { padding: 12px 24px; font-size: 16px; }\n\n");

        // Container styles
        Self::extract_container_styles(components, &mut css);

        Ok(css)
    }

    fn file_extension(&self) -> &str {
        "css"
    }
}

impl CssGenerator {
    fn extract_container_styles(components: &[CanvasComponent], css: &mut String) {
        for component in components {
            if let CanvasComponent::Container(container) = component {
                let id = container.id.as_string();
                let class_name = format!("container-{}", &id[..8]);

                css.push_str(&format!(".{} {{\n", class_name));

                match &container.layout {
                    crate::domain::LayoutType::Flex {
                        direction,
                        wrap,
                        align_items,
                        justify_content,
                    } => {
                        css.push_str("  display: flex;\n");
                        css.push_str(&format!(
                            "  flex-direction: {};\n",
                            match direction {
                                crate::domain::FlexDirection::Row => "row",
                                crate::domain::FlexDirection::Column => "column",
                            }
                        ));
                        if *wrap {
                            css.push_str("  flex-wrap: wrap;\n");
                        }

                        let align_css = match align_items {
                            crate::domain::FlexAlign::Start => "flex-start",
                            crate::domain::FlexAlign::Center => "center",
                            crate::domain::FlexAlign::End => "flex-end",
                            crate::domain::FlexAlign::Stretch => "stretch",
                            crate::domain::FlexAlign::Baseline => "baseline",
                        };

                        let justify_css = match justify_content {
                            crate::domain::FlexJustify::Start => "flex-start",
                            crate::domain::FlexJustify::Center => "center",
                            crate::domain::FlexJustify::End => "flex-end",
                            crate::domain::FlexJustify::Between => "space-between",
                            crate::domain::FlexJustify::Around => "space-around",
                            crate::domain::FlexJustify::Evenly => "space-evenly",
                        };

                        css.push_str(&format!("  align-items: {};\n", align_css));
                        css.push_str(&format!("  justify-content: {};\n", justify_css));
                    }
                    crate::domain::LayoutType::Grid { columns, rows } => {
                        css.push_str("  display: grid;\n");
                        css.push_str(&format!(
                            "  grid-template-columns: repeat({}, 1fr);\n",
                            columns
                        ));
                        css.push_str(&format!("  grid-template-rows: repeat({}, auto);\n", rows));
                    }
                    crate::domain::LayoutType::Stack => {
                        css.push_str("  display: flex;\n");
                        css.push_str("  flex-direction: column;\n");
                    }
                }

                css.push_str(&format!("  gap: {}px;\n", container.gap));
                css.push_str(&format!(
                    "  padding: {}px {}px {}px {}px;\n",
                    container.padding.top,
                    container.padding.right,
                    container.padding.bottom,
                    container.padding.left
                ));

                css.push_str("}\n\n");

                // Recurse into children
                Self::extract_container_styles(&container.children, css);
            } else if let CanvasComponent::Card(card) = component {
                let id = card.id.as_string();
                let class_name = format!("card-{}", &id[..8]);

                css.push_str(&format!(".{} {{\n", class_name));
                css.push_str(&format!("  padding: {}px;\n", card.padding));
                css.push_str(&format!("  border-radius: {}px;\n", card.border_radius));
                if card.shadow {
                    css.push_str("  box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);\n");
                }
                if card.border {
                    css.push_str("  border: 1px solid #e5e7eb;\n");
                }
                css.push_str("}\n\n");

                Self::extract_container_styles(&card.children, css);
            }
        }
    }
}

// Additional Generators
// ============================================================================

/// HTML with Tailwind CSS classes generator
pub struct TailwindHtmlGenerator;

impl CodeGenerator for TailwindHtmlGenerator {
    fn generate(
        &self,
        components: &[CanvasComponent],
        _variables: &[Variable],
    ) -> AppResult<String> {
        let mut output = String::new();

        output.push_str("<!-- Generated by Leptos Studio with Tailwind CSS -->\n");
        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html lang=\"en\">\n");
        output.push_str("<head>\n");
        output.push_str("  <meta charset=\"UTF-8\">\n");
        output.push_str(
            "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
        );
        output.push_str("  <script src=\"https://cdn.tailwindcss.com\"></script>\n");
        output.push_str("  <title>Generated Layout</title>\n");
        output.push_str("</head>\n");
        output.push_str("<body class=\"min-h-screen bg-gray-50\">\n");
        output.push_str("  <main class=\"container mx-auto p-4\">\n");

        for component in components {
            Self::generate_tailwind(component, &mut output, 2)?;
        }

        output.push_str("  </main>\n");
        output.push_str("</body>\n");
        output.push_str("</html>\n");

        Ok(output)
    }

    fn file_extension(&self) -> &str {
        "html"
    }
}

impl TailwindHtmlGenerator {
    fn generate_tailwind(
        component: &CanvasComponent,
        output: &mut String,
        indent_level: usize,
    ) -> AppResult<()> {
        let indent = "  ".repeat(indent_level);

        match component {
            CanvasComponent::Button(btn) => {
                let variant_classes = match btn.variant {
                    crate::domain::ButtonVariant::Primary => {
                        "bg-blue-600 hover:bg-blue-700 text-white"
                    }
                    crate::domain::ButtonVariant::Secondary => {
                        "bg-gray-200 hover:bg-gray-300 text-gray-800"
                    }
                    crate::domain::ButtonVariant::Outline => {
                        "bg-transparent border-2 border-blue-600 text-blue-600 hover:bg-blue-50"
                    }
                    crate::domain::ButtonVariant::Ghost => {
                        "bg-transparent text-gray-600 hover:bg-gray-100"
                    }
                };

                let size_classes = match btn.size {
                    crate::domain::ButtonSize::Small => "px-3 py-1 text-sm",
                    crate::domain::ButtonSize::Medium => "px-4 py-2 text-base",
                    crate::domain::ButtonSize::Large => "px-6 py-3 text-lg",
                };

                let disabled_classes = if btn.disabled {
                    "opacity-50 cursor-not-allowed"
                } else {
                    "cursor-pointer"
                };

                output.push_str(&format!(
                    "{}<button class=\"rounded-md font-medium transition-colors {} {} {}\"{}>{}</button>\n",
                    indent,
                    variant_classes,
                    size_classes,
                    disabled_classes,
                    if btn.disabled { " disabled" } else { "" },
                    btn.label
                ));
            }
            CanvasComponent::Text(txt) => {
                let (tag, classes) = match txt.tag {
                    crate::domain::TextTag::H1 => ("h1", "text-4xl font-bold text-gray-900"),
                    crate::domain::TextTag::H2 => ("h2", "text-3xl font-semibold text-gray-800"),
                    crate::domain::TextTag::H3 => ("h3", "text-2xl font-medium text-gray-700"),
                    crate::domain::TextTag::P => ("p", "text-base text-gray-600"),
                    crate::domain::TextTag::Span => ("span", "text-base text-gray-600"),
                };

                output.push_str(&format!(
                    "{}<{} class=\"{}\">{}</{}>\n",
                    indent, tag, classes, txt.content, tag
                ));
            }
            CanvasComponent::Input(inp) => {
                let input_type = match inp.input_type {
                    crate::domain::InputType::Text => "text",
                    crate::domain::InputType::Password => "password",
                    crate::domain::InputType::Email => "email",
                    crate::domain::InputType::Number => "number",
                    crate::domain::InputType::Tel => "tel",
                };

                output.push_str(&format!(
                    "{}<input type=\"{}\" placeholder=\"{}\" class=\"w-full px-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent\"{}{}>\n",
                    indent,
                    input_type,
                    inp.placeholder,
                    if inp.required { " required" } else { "" },
                    if inp.disabled { " disabled" } else { "" }
                ));
            }
            CanvasComponent::Select(sel) => {
                output.push_str(&format!("{}<select class=\"block w-full px-4 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500\" {}>\n", indent, if sel.disabled { "disabled" } else { "" }));
                if !sel.placeholder.is_empty() {
                    output.push_str(&format!(
                        "{}  <option value=\"\" disabled selected>{}</option>\n",
                        indent, sel.placeholder
                    ));
                }
                for option in sel.options.split(',') {
                    let opt = option.trim();
                    output.push_str(&format!(
                        "{}  <option value=\"{}\">{}</option>\n",
                        indent, opt, opt
                    ));
                }
                output.push_str(&format!("{}</select>\n", indent));
            }
            CanvasComponent::Container(container) => {
                let layout_classes = match &container.layout {
                    crate::domain::LayoutType::Flex {
                        direction,
                        wrap,
                        align_items,
                        justify_content,
                    } => {
                        let dir = match direction {
                            crate::domain::FlexDirection::Row => "flex-row",
                            crate::domain::FlexDirection::Column => "flex-col",
                        };
                        let wrap_cls = if *wrap { "flex-wrap" } else { "" };

                        let align_cls = match align_items {
                            crate::domain::FlexAlign::Start => "items-start",
                            crate::domain::FlexAlign::Center => "items-center",
                            crate::domain::FlexAlign::End => "items-end",
                            crate::domain::FlexAlign::Stretch => "items-stretch",
                            crate::domain::FlexAlign::Baseline => "items-baseline",
                        };

                        let justify_cls = match justify_content {
                            crate::domain::FlexJustify::Start => "justify-start",
                            crate::domain::FlexJustify::Center => "justify-center",
                            crate::domain::FlexJustify::End => "justify-end",
                            crate::domain::FlexJustify::Between => "justify-between",
                            crate::domain::FlexJustify::Around => "justify-around",
                            crate::domain::FlexJustify::Evenly => "justify-evenly",
                        };

                        format!("flex {} {} {} {}", dir, wrap_cls, align_cls, justify_cls)
                    }
                    crate::domain::LayoutType::Grid { columns, .. } => {
                        format!("grid grid-cols-{}", columns.min(&12))
                    }
                    crate::domain::LayoutType::Stack => "flex flex-col".to_string(),
                };

                let gap_class = format!("gap-{}", (container.gap / 4).clamp(1, 16));
                let padding_class = format!(
                    "pt-{} pr-{} pb-{} pl-{}",
                    (container.padding.top / 4).min(16),
                    (container.padding.right / 4).min(16),
                    (container.padding.bottom / 4).min(16),
                    (container.padding.left / 4).min(16)
                );

                output.push_str(&format!(
                    "{}<div class=\"{} {} {}\">\n",
                    indent, layout_classes, gap_class, padding_class
                ));

                for child in &container.children {
                    Self::generate_tailwind(child, output, indent_level + 1)?;
                }

                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Image(img) => {
                output.push_str(&format!(
                    "{}<img src=\"{}\" alt=\"{}\" class=\"max-w-full h-auto\" />\n",
                    indent, img.src, img.alt
                ));
            }
            CanvasComponent::Card(card) => {
                let shadow_class = if card.shadow { "shadow-md" } else { "" };
                let border_class = if card.border {
                    "border border-gray-200"
                } else {
                    ""
                };
                let padding_class = format!("p-{}", (card.padding / 4).max(1));
                let rounded_class = match card.border_radius {
                    0 => "rounded-none",
                    1..=4 => "rounded-sm",
                    5..=8 => "rounded",
                    9..=12 => "rounded-md",
                    _ => "rounded-lg",
                };

                output.push_str(&format!(
                    "{}<div class=\"bg-white {} {} {} {}\">\n",
                    indent, shadow_class, border_class, padding_class, rounded_class
                ));

                for child in &card.children {
                    Self::generate_tailwind(child, output, indent_level + 1)?;
                }

                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Custom(custom) => {
                output.push_str(&format!("{}<!-- Custom: {} -->\n", indent, custom.name));
                output.push_str(&format!("{}<div class=\"custom-component\">\n", indent));
                output.push_str(&format!("{}  {}\n", indent, custom.template));
                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Divider(divider) => {
                let class = match divider.orientation {
                    crate::domain::DividerOrientation::Horizontal => {
                        "border-t border-gray-200 my-2"
                    }
                    crate::domain::DividerOrientation::Vertical => {
                        "border-l border-gray-200 mx-2 self-stretch"
                    }
                };
                output.push_str(&format!("{}<hr class=\"{}\" />\n", indent, class));
            }
            CanvasComponent::Checkbox(checkbox) => {
                output.push_str(&format!(
                    "{}<label class=\"flex items-center gap-2\">\n{}  <input type=\"checkbox\" class=\"rounded border-gray-300\"{}{} />\n{}  <span>{}</span>\n{}</label>\n",
                    indent,
                    indent,
                    if checkbox.checked { " checked" } else { "" },
                    if checkbox.disabled { " disabled" } else { "" },
                    indent,
                    checkbox.label,
                    indent
                ));
            }
            CanvasComponent::RadioGroup(radio) => {
                let name = format!("radio-{}", radio.id);
                output.push_str(&format!("{}<div class=\"flex flex-col gap-1\">\n", indent));
                for opt in radio
                    .options
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    let checked = !radio.selected.is_empty() && radio.selected == opt;
                    output.push_str(&format!(
                        "{}  <label class=\"flex items-center gap-2\"><input type=\"radio\" name=\"{}\"{}{} /> {}</label>\n",
                        indent,
                        name,
                        if checked { " checked" } else { "" },
                        if radio.disabled { " disabled" } else { "" },
                        opt
                    ));
                }
                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Switch(switch) => {
                output.push_str(&format!(
                    "{}<label class=\"flex items-center gap-2\">\n{}  <input type=\"checkbox\" class=\"toggle\"{}{} />\n{}  <span>{}</span>\n{}</label>\n",
                    indent,
                    indent,
                    if switch.checked { " checked" } else { "" },
                    if switch.disabled { " disabled" } else { "" },
                    indent,
                    switch.label,
                    indent
                ));
            }
            CanvasComponent::Badge(badge) => {
                let color = match badge.variant {
                    crate::domain::BadgeVariant::Default => "bg-gray-100 text-gray-800",
                    crate::domain::BadgeVariant::Primary => "bg-blue-100 text-blue-800",
                    crate::domain::BadgeVariant::Success => "bg-green-100 text-green-800",
                    crate::domain::BadgeVariant::Warning => "bg-amber-100 text-amber-800",
                    crate::domain::BadgeVariant::Error => "bg-red-100 text-red-800",
                };
                output.push_str(&format!(
                    "{}<span class=\"inline-block px-2 py-0.5 rounded-full text-xs font-medium {}\">{}</span>\n",
                    indent, color, badge.text
                ));
            }
            CanvasComponent::Progress(progress) => {
                let percent = if progress.max > 0.0 {
                    (progress.value / progress.max * 100.0).clamp(0.0, 100.0)
                } else {
                    0.0
                };
                output.push_str(&format!(
                    "{}<div class=\"w-full bg-gray-200 rounded-full h-2\">\n{}  <div class=\"bg-blue-500 h-2 rounded-full\" style=\"width: {:.0}%\"></div>\n{}</div>\n",
                    indent, indent, percent, indent
                ));
            }
        }

        Ok(())
    }
}

/// Svelte component generator
pub struct SvelteGenerator;

impl CodeGenerator for SvelteGenerator {
    fn generate(
        &self,
        components: &[CanvasComponent],
        variables: &[Variable],
    ) -> AppResult<String> {
        let mut output = String::new();

        // Script section
        output.push_str("<script lang=\"ts\">\n");
        output.push_str("  // Generated by Leptos Studio\n");

        let vars_init = if variables.is_empty() {
            "{}".to_string()
        } else {
            let entries: Vec<String> = variables
                .iter()
                .map(|v| {
                    let val = match v.data_type {
                        VariableType::String => format!("'{}'", v.default_value),
                        _ => v.default_value.clone(),
                    };
                    format!("{}: {}", v.name, val)
                })
                .collect();
            format!("{{ {} }}", entries.join(", "))
        };

        output.push_str(&format!("  let vars = {};\n", vars_init));
        output.push_str("</script>\n\n");

        // Template section
        output.push_str("<div class=\"generated-layout\">\n");

        for component in components {
            Self::generate_svelte(component, &mut output, 1)?;
        }

        output.push_str("</div>\n\n");

        // Style section
        output.push_str("<style>\n");
        output.push_str("  .generated-layout {\n");
        output.push_str("    /* Add your styles here */\n");
        output.push_str("  }\n");
        output.push_str("  \n");
        output.push_str("  .btn {\n");
        output.push_str("    padding: 8px 16px;\n");
        output.push_str("    border-radius: 6px;\n");
        output.push_str("    cursor: pointer;\n");
        output.push_str("    transition: all 0.2s;\n");
        output.push_str("  }\n");
        output.push_str("  \n");
        output.push_str("  .btn-primary {\n");
        output.push_str("    background: #3b82f6;\n");
        output.push_str("    color: white;\n");
        output.push_str("    border: none;\n");
        output.push_str("  }\n");
        output.push_str("  \n");
        output.push_str("  .btn-secondary {\n");
        output.push_str("    background: #e5e7eb;\n");
        output.push_str("    color: #374151;\n");
        output.push_str("    border: none;\n");
        output.push_str("  }\n");
        output.push_str("  \n");
        output.push_str("  input {\n");
        output.push_str("    width: 100%;\n");
        output.push_str("    padding: 8px 12px;\n");
        output.push_str("    border: 1px solid #d1d5db;\n");
        output.push_str("    border-radius: 6px;\n");
        output.push_str("  }\n");
        output.push_str("</style>\n");

        Ok(output)
    }

    fn file_extension(&self) -> &str {
        "svelte"
    }
}

impl SvelteGenerator {
    fn generate_svelte(
        component: &CanvasComponent,
        output: &mut String,
        indent_level: usize,
    ) -> AppResult<()> {
        let indent = "  ".repeat(indent_level);

        match component {
            CanvasComponent::Button(btn) => {
                let id_attr = if let Some(bind) = btn.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = btn.bindings.get("custom_css_classes") {
                    format!(" class={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let variant_class = match btn.variant {
                    crate::domain::ButtonVariant::Primary => "btn btn-primary",
                    crate::domain::ButtonVariant::Secondary => "btn btn-secondary",
                    crate::domain::ButtonVariant::Outline => "btn btn-outline",
                    crate::domain::ButtonVariant::Ghost => "btn btn-ghost",
                };

                let label_expr = if let Some(bind) = btn.bindings.get("label") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    btn.label.clone()
                };

                let disabled_expr = if let Some(bind) = btn.bindings.get("disabled") {
                    format!("disabled={{vars['{}']}}", bind)
                } else {
                    if btn.disabled { " disabled" } else { "" }.to_string()
                };

                output.push_str(&format!(
                    "{}<button{}{} class=\"{}\" {}>{}</button>\n",
                    indent, id_attr, class_attr, variant_class, disabled_expr, label_expr
                ));
            }
            CanvasComponent::Text(txt) => {
                let id_attr = if let Some(bind) = txt.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = txt.bindings.get("custom_css_classes") {
                    format!(" class={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let tag = match txt.tag {
                    crate::domain::TextTag::H1 => "h1",
                    crate::domain::TextTag::H2 => "h2",
                    crate::domain::TextTag::H3 => "h3",
                    crate::domain::TextTag::P => "p",
                    crate::domain::TextTag::Span => "span",
                };

                let content_expr = if let Some(bind) = txt.bindings.get("content") {
                    format!("{{vars['{}']}}", bind)
                } else {
                    txt.content.clone()
                };

                output.push_str(&format!(
                    "{}<{} {}{}>{}</{}>\n",
                    indent, tag, id_attr, class_attr, content_expr, tag
                ));
            }
            CanvasComponent::Input(inp) => {
                let id_attr = if let Some(bind) = inp.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = inp.bindings.get("custom_css_classes") {
                    format!(" class={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let input_type = match inp.input_type {
                    crate::domain::InputType::Text => "text",
                    crate::domain::InputType::Password => "password",
                    crate::domain::InputType::Email => "email",
                    crate::domain::InputType::Number => "number",
                    crate::domain::InputType::Tel => "tel",
                };

                let placeholder_attr = if let Some(bind) = inp.bindings.get("placeholder") {
                    format!("placeholder={{vars['{}']}}", bind)
                } else {
                    format!("placeholder=\"{}\"", inp.placeholder)
                };

                let disabled_attr = if let Some(bind) = inp.bindings.get("disabled") {
                    format!("disabled={{vars['{}']}}", bind)
                } else {
                    if inp.disabled { " disabled" } else { "" }.to_string()
                };

                output.push_str(&format!(
                    "{}<input{}{} type=\"{}\" {} {} {} />\n",
                    indent,
                    id_attr,
                    class_attr,
                    input_type,
                    placeholder_attr,
                    if inp.required { " required" } else { "" },
                    disabled_attr
                ));
            }
            CanvasComponent::Select(sel) => {
                let id_attr = if let Some(bind) = sel.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = sel.bindings.get("custom_css_classes") {
                    format!(" class={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let disabled_attr = if let Some(bind) = sel.bindings.get("disabled") {
                    format!("disabled={{vars['{}']}}", bind)
                } else {
                    if sel.disabled { " disabled" } else { "" }.to_string()
                };

                output.push_str(&format!(
                    "{}<select{}{} {}>\n",
                    indent, id_attr, class_attr, disabled_attr
                ));
                if !sel.placeholder.is_empty() {
                    let placeholder_expr = if let Some(bind) = sel.bindings.get("placeholder") {
                        format!("{{vars['{}']}}", bind)
                    } else {
                        sel.placeholder.clone()
                    };
                    output.push_str(&format!(
                        "{}  <option value=\"\" disabled selected>{}</option>\n",
                        indent, placeholder_expr
                    ));
                }

                if let Some(bind) = sel.bindings.get("options") {
                    output.push_str(&format!(
                        "{}  {{#each (vars['{}']?.split(',') || []) as opt}}\n",
                        indent, bind
                    ));
                    output.push_str(&format!(
                        "{}    <option value={{opt.trim()}}>{{opt.trim()}}</option>\n",
                        indent
                    ));
                    output.push_str(&format!("{}  {{/each}}\n", indent));
                } else {
                    for option in sel.options.split(',') {
                        let opt = option.trim();
                        output.push_str(&format!(
                            "{}  <option value=\"{}\">{}</option>\n",
                            indent, opt, opt
                        ));
                    }
                }
                output.push_str(&format!("{}</select>\n", indent));
            }
            CanvasComponent::Container(container) => {
                let style = format!(
                    "display: flex; flex-direction: {}; gap: {}px; padding: {}px {}px {}px {}px;",
                    match &container.layout {
                        crate::domain::LayoutType::Flex { direction, .. } => {
                            match direction {
                                crate::domain::FlexDirection::Row => "row",
                                crate::domain::FlexDirection::Column => "column",
                            }
                        }
                        _ => "column",
                    },
                    container.gap,
                    container.padding.top,
                    container.padding.right,
                    container.padding.bottom,
                    container.padding.left
                );

                let id_attr = if let Some(bind) = container.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = container.bindings.get("custom_css_classes") {
                    format!(" class={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "{}<div{}{} style=\"{}\">\n",
                    indent, id_attr, class_attr, style
                ));

                for child in &container.children {
                    Self::generate_svelte(child, output, indent_level + 1)?;
                }

                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Image(img) => {
                let id_attr = if let Some(bind) = img.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = img.bindings.get("custom_css_classes") {
                    format!(" class={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let src_val = if let Some(bind) = img.bindings.get("src") {
                    format!("src={{vars['{}']}}", bind)
                } else {
                    format!("src=\"{}\"", img.src)
                };

                let alt_val = if let Some(bind) = img.bindings.get("alt") {
                    format!("alt={{vars['{}']}}", bind)
                } else {
                    format!("alt=\"{}\"", img.alt)
                };

                output.push_str(&format!(
                    "{}<img{}{} {} {} />\n",
                    indent, id_attr, class_attr, src_val, alt_val
                ));
            }
            CanvasComponent::Card(card) => {
                let style = format!(
                    "padding: {}px; border-radius: {}px; {}; {}",
                    card.padding,
                    card.border_radius,
                    if card.shadow {
                        "box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1)"
                    } else {
                        ""
                    },
                    if card.border {
                        "border: 1px solid #e5e7eb"
                    } else {
                        ""
                    }
                );

                let id_attr = if let Some(bind) = card.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = card.bindings.get("custom_css_classes") {
                    format!(" class={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                output.push_str(&format!(
                    "{}<div{}{} style=\"{}\">\n",
                    indent, id_attr, class_attr, style
                ));

                for child in &card.children {
                    Self::generate_svelte(child, output, indent_level + 1)?;
                }

                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Custom(custom) => {
                let id_attr = if let Some(bind) = custom.bindings.get("id") {
                    format!(" id={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                let class_attr = if let Some(bind) = custom.bindings.get("custom_css_classes") {
                    format!(" class={{vars['{}']}}", bind)
                } else {
                    String::new()
                };

                output.push_str(&format!("{}<!-- Custom: {} -->\n", indent, custom.name));
                output.push_str(&format!(
                    "{}<div{}{}>{{@html `{}`}}</div>\n",
                    indent, id_attr, class_attr, custom.template
                ));
            }
            CanvasComponent::Divider(divider) => {
                let style = match divider.orientation {
                    crate::domain::DividerOrientation::Horizontal => format!(
                        "border: none; border-top: {}px solid #e5e7eb; margin: 8px 0;",
                        divider.thickness
                    ),
                    crate::domain::DividerOrientation::Vertical => format!(
                        "border: none; border-left: {}px solid #e5e7eb; margin: 0 8px; align-self: stretch;",
                        divider.thickness
                    ),
                };
                output.push_str(&format!("{}<hr style=\"{}\" />\n", indent, style));
            }
            CanvasComponent::Checkbox(checkbox) => {
                output.push_str(&format!(
                    "{}<label>\n{}  <input type=\"checkbox\"{}{} />\n{}  {}\n{}</label>\n",
                    indent,
                    indent,
                    if checkbox.checked { " checked" } else { "" },
                    if checkbox.disabled { " disabled" } else { "" },
                    indent,
                    checkbox.label,
                    indent
                ));
            }
            CanvasComponent::RadioGroup(radio) => {
                let name = format!("radio-{}", radio.id);
                output.push_str(&format!("{}<div class=\"radio-group\">\n", indent));
                for opt in radio
                    .options
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    let checked = !radio.selected.is_empty() && radio.selected == opt;
                    output.push_str(&format!(
                        "{}  <label><input type=\"radio\" name=\"{}\"{}{} /> {}</label>\n",
                        indent,
                        name,
                        if checked { " checked" } else { "" },
                        if radio.disabled { " disabled" } else { "" },
                        opt
                    ));
                }
                output.push_str(&format!("{}</div>\n", indent));
            }
            CanvasComponent::Switch(switch) => {
                output.push_str(&format!(
                    "{}<label class=\"switch\">\n{}  <input type=\"checkbox\"{}{} />\n{}  {}\n{}</label>\n",
                    indent,
                    indent,
                    if switch.checked { " checked" } else { "" },
                    if switch.disabled { " disabled" } else { "" },
                    indent,
                    switch.label,
                    indent
                ));
            }
            CanvasComponent::Badge(badge) => {
                let variant = match badge.variant {
                    crate::domain::BadgeVariant::Default => "default",
                    crate::domain::BadgeVariant::Primary => "primary",
                    crate::domain::BadgeVariant::Success => "success",
                    crate::domain::BadgeVariant::Warning => "warning",
                    crate::domain::BadgeVariant::Error => "error",
                };
                output.push_str(&format!(
                    "{}<span class=\"badge badge-{}\">{}</span>\n",
                    indent, variant, badge.text
                ));
            }
            CanvasComponent::Progress(progress) => {
                output.push_str(&format!(
                    "{}<progress value={{{}}} max={{{}}} style=\"width: 100%;\"></progress>\n",
                    indent, progress.value, progress.max
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ButtonComponent, TextComponent};

    #[test]
    fn test_json_schema_generator() {
        let generator = JsonSchemaGenerator;
        let button = CanvasComponent::Button(ButtonComponent::new("Test".to_string()));
        let schema = generator.generate(&[button], &[]).unwrap();

        assert!(schema.contains("$schema"));
        assert!(schema.contains("definitions"));
        assert!(schema.contains("ButtonComponent"));
    }

    #[test]
    fn test_typescript_generator() {
        let generator = TypeScriptGenerator;
        let types = generator.generate(&[], &[]).unwrap();

        assert!(types.contains("ButtonVariant"));
        assert!(types.contains("CanvasComponent"));
        assert!(types.contains("export interface"));
    }

    #[test]
    fn test_react_generator() {
        let generator = ReactGenerator;
        let text = CanvasComponent::Text(TextComponent::new("Hello".to_string()));
        let code = generator.generate(&[text], &[]).unwrap();

        assert!(code.contains("import React"));
        assert!(code.contains("Hello"));
        assert!(code.contains("export function"));
    }

    #[test]
    fn test_vue_generator() {
        let generator = VueGenerator;
        let text = CanvasComponent::Text(TextComponent::new("Hello".to_string()));
        let code = generator.generate(&[text], &[]).unwrap();

        assert!(code.contains("<template>"));
        assert!(code.contains("<script setup"));
        assert!(code.contains("Hello"));
    }

    #[test]
    fn test_css_generator() {
        let generator = CssGenerator;
        let button = CanvasComponent::Button(ButtonComponent::new("Test".to_string()));
        let css = generator.generate(&[button], &[]).unwrap();

        assert!(css.contains(".btn-primary"));
        assert!(css.contains(".btn-secondary"));
    }

    #[test]
    fn test_tailwind_generator() {
        let generator = TailwindHtmlGenerator;
        let button = CanvasComponent::Button(ButtonComponent::new("Test".to_string()));
        let html = generator.generate(&[button], &[]).unwrap();

        assert!(html.contains("class="));
        assert!(html.contains("Test"));
    }

    #[test]
    fn test_svelte_generator() {
        let generator = SvelteGenerator;
        let text = CanvasComponent::Text(TextComponent::new("Hello".to_string()));
        let code = generator.generate(&[text], &[]).unwrap();

        assert!(code.contains("<script"));
        assert!(code.contains("Hello"));
    }
}
