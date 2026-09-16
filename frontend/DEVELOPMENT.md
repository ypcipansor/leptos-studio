# Leptos Studio – Development Guide

A comprehensive guide for contributing to and extending Leptos Studio.

## Table of Contents

1. [Project Structure](#project-structure)
2. [Development Setup](#development-setup)
3. [Architecture](#architecture)
4. [Adding Features](#adding-features)
5. [Testing](#testing)
6. [Code Style](#code-style)
7. [Common Tasks](#common-tasks)

---

## Project Structure

```
frontend/
├── src/
│   ├── app.rs                      # Router and root application component
│   ├── lib.rs                      # Library root
│   │
│   ├── domain/                     # Data models
│   │   ├── component.rs            # Component types and definitions
│   │   ├── error.rs                # Error types
│   │   ├── style.rs                # Styling model
│   │   ├── validation.rs           # Validators
│   │   ├── variable.rs             # Global variable model
│   │   └── mod.rs                  # Domain module exports
│   │
│   ├── state/                      # State management
│   │   ├── app_state.rs            # Main application state
│   │   ├── derived.rs              # Derived/computed signals
│   │   ├── history.rs              # Undo/redo history
│   │   ├── persistence.rs          # Persistence helpers
│   │   ├── project.rs              # Project serialization
│   │   └── mod.rs                  # State module exports
│   │
│   ├── services/                   # Use cases and integrations
│   │   ├── export_service.rs       # Core code generation
│   │   ├── export_advanced.rs      # React/Vue/Svelte/Tailwind generators
│   │   ├── git_service.rs          # Git integration
│   │   ├── git_factory.rs          # Remote vs LocalStorage backend selection
│   │   ├── local_storage_git.rs    # LocalStorage Git backend
│   │   ├── remote_git.rs           # Backend-API Git backend
│   │   ├── project_service.rs      # Project management
│   │   ├── project_manager.rs      # Project listing and CRUD
│   │   ├── property_service.rs     # Property updates
│   │   ├── template_service.rs     # Built-in and custom templates
│   │   ├── event_bus.rs            # Decoupled app events
│   │   ├── analytics_service.rs    # Usage metrics
│   │   └── mod.rs                  # Services module exports
│   │
│   ├── pages/                      # Route-level views
│   │   ├── dashboard.rs            # Project dashboard (/)
│   │   ├── editor.rs               # Builder editor (/editor/:id)
│   │   ├── not_found.rs            # 404 page
│   │   └── mod.rs                  # Pages module exports
│   │
│   ├── builder/                    # Builder UI components
│   │   ├── mod.rs                  # Exports all components
│   │   ├── canvas/                 # Canvas surface and rendering
│   │   ├── component_palette.rs    # Palette with category filters
│   │   ├── component_library.rs    # Library utilities
│   │   ├── component_library_enhanced.rs  # Search & filtering
│   │   ├── component_constraints.rs # Size & alignment rules
│   │   ├── property_editor.rs      # Property editing panel
│   │   ├── property_editors/       # Per-type property editors
│   │   ├── property_inputs.rs      # Shared property inputs
│   │   ├── preview.rs              # Live preview panel
│   │   ├── responsive_preview.rs   # Device viewport presets
│   │   ├── breakpoint_editor.rs    # Custom breakpoints
│   │   ├── theme_editor.rs         # Design-token theme editing
│   │   ├── design_tokens.rs        # Design system tokens
│   │   ├── variable_panel.rs       # Global variable management
│   │   ├── styling_system.rs       # Styling features
│   │   ├── tree_view.rs            # Layers tree
│   │   ├── toolbar.rs              # Editor toolbar
│   │   ├── status_bar.rs           # Canvas status bar
│   │   ├── keyboard.rs             # Keyboard shortcuts
│   │   ├── command_palette.rs      # Command palette
│   │   ├── context_menu.rs         # Right-click menus
│   │   ├── drag_drop.rs            # Drag & drop handling
│   │   ├── export_modal.rs         # Export dialog
│   │   ├── code_panel.rs           # Live code view
│   │   ├── history_panel.rs        # Undo/redo history UI
│   │   ├── git_panel/              # Git integration UI
│   │   ├── project.rs              # Project management UI
│   │   ├── debug_panel.rs          # Debug information
│   │   ├── template_gallery.rs     # Template gallery
│   │   ├── save_template_modal.rs  # Save-as-template dialog
│   │   ├── settings_modal.rs       # Settings dialog
│   │   ├── shortcuts_modal.rs      # Shortcut reference
│   │   ├── welcome_modal.rs        # First-run dialog
│   │   ├── accessibility.rs        # Accessibility helpers
│   │   ├── breadcrumb.rs           # Navigation
│   │   ├── snackbar.rs             # Notifications
│   │   ├── hooks/                  # Shared builder hooks
│   │   └── sidebar.rs              # Legacy standalone sidebar
│   │
│   └── utils/                      # Utility functions
│       ├── async_task.rs           # Async task helpers
│       ├── clipboard.rs            # Clipboard operations
│       ├── dom.rs                  # DOM utilities
│       ├── file.rs                 # File download/upload
│       ├── format.rs               # String formatting
│       ├── sanitize.rs             # HTML sanitisation
│       ├── syntax_highlight.rs     # Code highlighting
│       └── mod.rs                  # Utils module exports
│
├── tests/                          # Integration tests
│   ├── wasm_smoke.rs               # WASM smoke test
│   ├── canvas_state_test.rs        # Canvas state behaviour
│   ├── export_tests.rs             # Export generator coverage
│   ├── git_dirty_test.rs           # Git dirty-state tracking
│   ├── git_flow_test.rs            # Git commit/restore flow
│   └── accessibility_test.rs       # Accessibility assertions
│
├── style.css                       # Global styles
├── index.html                      # Entry point
├── Trunk.toml                      # Trunk configuration
├── Cargo.toml                      # Rust dependencies
├── API.md                          # Module-level API reference
├── ARCHITECTURE.md                 # Architecture overview
├── CONTRIBUTING.md                 # Contribution guidelines
├── QUICKSTART.md                   # Task-oriented walkthrough
├── SECURITY.md                     # Security notes
├── CHANGELOG.md                    # Changelog
└── README.md                       # Frontend README
```

---

## Development Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk (WASM bundler)
cargo install trunk

# Install dependencies for linting (optional)
cargo install cargo-dylint
cargo install leptosfmt
```

### Running Development Server

```bash
# Start dev server (hot reload)
trunk serve

# Open browser
open http://localhost:8899
```

### Building for Production

```bash
# Release build
trunk build --release

# Output in dist/
```

---

## Architecture

### Layer Overview

```
┌─────────────────────────────────────┐
│  UI Layer (builder/)                │  ← Components, pages
│  - Canvas, Sidebar, Panels          │
└────────────────┬────────────────────┘
                 │
┌────────────────▼────────────────────┐
│  State Layer (state/)               │  ← Global state, signals
│  - AppState, CanvasState, UiState   │
└────────────────┬────────────────────┘
                 │
┌────────────────▼────────────────────┐
│  Services Layer (services/)         │  ← Business logic
│  - Export, Project, Git, Property   │
└────────────────┬────────────────────┘
                 │
┌────────────────▼────────────────────┐
│  Domain Layer (domain/)             │  ← Types, validation
│  - Components, Errors, Validators   │
└─────────────────────────────────────┘
```

### Data Flow Example: Adding a Component

```
User (Canvas)
    ↓
Drop event → DragDropHandler
    ↓
Update canvas_state.components (state/)
    ↓
Trigger canvas.record_snapshot() (history)
    ↓
CanvasComponent rendered (builder/canvas/renderer)
    ↓
User sees new component
```

---

## Adding Features

### 1. New UI Component

**Step 1: Create component file**

```rust
// src/builder/my_feature.rs

use leptos::prelude::*;
use crate::state::AppState;

#[component]
pub fn MyFeature() -> impl IntoView {
    let app_state = AppState::use_context();
    
    view! {
        <div class="my-feature">
            {/* Your component */}
        </div>
    }
}
```

**Step 2: Export in builder/mod.rs**

```rust
pub mod my_feature;
pub use my_feature::MyFeature;
```

**Step 3: Add to app.rs**

```rust
use crate::builder::my_feature::MyFeature;

// In App component view:
<MyFeature />
```

**Step 4: Add styles to style.css**

```css
.my-feature {
    /* Your styles */
}
```

### 2. New Validator

**Step 1: Create validator in domain/validation.rs**

```rust
pub struct MyValidator;

impl Validator<String> for MyValidator {
    fn validate(&self, value: &String) -> Result<(), ValidationError> {
        if value.is_empty() {
            return Err(ValidationError::Empty);
        }
        Ok(())
    }
}
```

**Step 2: Use in services**

```rust
use crate::domain::validation::MyValidator;

let validator = MyValidator;
validator.validate(&input)?;
```

### 3. New Service Function

**Step 1: Create in appropriate service file**

```rust
// src/services/my_service.rs

pub fn my_operation(input: &str) -> Result<String, AppError> {
    // Business logic
    Ok(result)
}
```

**Step 2: Export in services/mod.rs**

```rust
pub mod my_service;
pub use my_service::my_operation;
```

**Step 3: Use in components**

```rust
use crate::services::my_operation;

let result = my_operation(&input)?;
```

---

## Testing

### Running Tests

```bash
# Unit and binary tests (this is what CI runs)
cargo test --workspace --lib --bins

# A single test by name
cargo test --lib test_category_matches

# Integration tests under tests/ (native)
cargo test --test export_tests

# With output
cargo test -- --nocapture
```

The `tests/wasm_smoke.rs` suite is compiled for the browser with
`wasm-bindgen-test`. Run it with `wasm-pack test --headless --chrome`, and note
that `cargo test --target wasm32-unknown-unknown` does **not** work — the test
harness and some dependencies do not build for that target.

### Writing Tests

**Unit test in domain module:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        let result = my_function("input");
        assert_eq!(result, "expected");
    }
}
```

**Integration test in tests/ directory:**

```rust
#[test]
fn test_export_service() {
    let components = vec![/* ... */];
    let generator = LeptosCodeGenerator::new(ExportPreset::Plain);
    let result = generator.generate(&components);
    assert!(result.is_ok());
}
```

---

## Code Style

### Naming Conventions

- **Functions**: `snake_case` - `update_component_prop()`
- **Types**: `PascalCase` - `CanvasComponent`
- **Constants**: `SCREAMING_SNAKE_CASE` - `DEFAULT_TIMEOUT`
- **CSS classes**: `kebab-case` - `.component-card`

### Comments

```rust
/// Doc comment for public items
/// Explains what this does, why, and how to use it
pub fn my_function() {
    // Regular comment for implementation details
    let x = 42;
}
```

### Error Handling

```rust
// Good: Descriptive error handling
match operation() {
    Ok(result) => handle_success(result),
    Err(e) => {
        let msg = e.user_message();
        app_state.ui.notification.set(Some(Notification::error(msg)));
    }
}

// Avoid: Silent failures
let _ = operation();
```

### Imports

```rust
// Group imports: standard lib, dependencies, local modules
use std::collections::HashMap;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::domain::Component;
```

---

## Common Tasks

### Adding a New Component Type

1. **Define in domain/component.rs**

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyComponent {
    pub id: ComponentId,
    pub property1: String,
    pub property2: i32,
}

// Add to CanvasComponent enum
pub enum CanvasComponent {
    // ...
    My(MyComponent),
}
```

2. **Add to sidebar library**

```rust
LibraryComponent {
    name: "My Component".to_string(),
    kind: "MyComponent".to_string(),
    category: "Custom".to_string(),
    // ...
}
```

3. **Add property editor support**

```rust
// In property_service.rs
pub fn update_my_component_prop(
    mut component: MyComponent,
    prop_name: &str,
    value: PropValue,
) -> Result<MyComponent, AppError> {
    match prop_name {
        "property1" => component.property1 = value.as_string()?,
        _ => return Err(AppError::InvalidOperation),
    }
    Ok(component)
}
```

4. **Add renderer**

```rust
// In canvas/renderer.rs
CanvasComponent::My(comp) => {
    view! {
        <div class="my-component">
            {&comp.property1}
        </div>
    }
}
```

### Modifying Styles

1. **Add CSS class in style.css**
2. **Use class in component**: `class="my-class"`
3. **Or inline styles**: `style="color: red;"`

### Adding Keyboard Shortcut

1. **Define action in keyboard.rs**

```rust
pub enum KeyboardAction {
    MyAction,
    // ...
}
```

2. **Add shortcut mapping**

```rust
pub fn get_default_shortcuts() -> Vec<KeyboardShortcut> {
    vec![
        KeyboardShortcut::new(
            vec![Modifier::Ctrl, Modifier::Shift],
            "M",
            KeyboardAction::MyAction,
        ),
        // ...
    ]
}
```

3. **Handle in app.rs**

```rust
KeyboardAction::MyAction => {
    // Handle action
}
```

---

## Best Practices

1. **State Management**
   - Use `RwSignal` for reactive state
   - Access via context: `AppState::use_context()`
   - Keep derived signals simple

2. **Error Handling**
   - Return `Result<T, AppError>`
   - Provide user-friendly messages
   - Log for debugging: `log::info!("Message")`

3. **Performance**
   - Use `.into_view()` for conditional rendering
   - Avoid unnecessary clones
   - Memoize computed values

4. **Testing**
   - Test domain logic thoroughly
   - Write integration tests for services
   - Mock external dependencies

5. **Documentation**
   - Document public APIs
   - Add examples for complex functions
   - Keep README and guides updated

---

## Debugging

### Enable Logging

```rust
// In main code
log::info!("Debug message: {:?}", value);
log::warn!("Warning: {}", message);
log::error!("Error: {}", error);

// In Cargo.toml
[dependencies]
log = "0.4"
```

### Browser Console

```rust
// Print to browser console
web_sys::console::log_1(&"Debug message".into());
```

### WASM Debugging

```bash
# Debug symbols
trunk build  # Default includes debug symbols in dev

# Source maps
# Chrome DevTools shows original Rust code
```

---

## Resources

- **Leptos**: https://leptos.dev
- **Rust Book**: https://doc.rust-lang.org/book/
- **WASM**: https://webassembly.org/
- **Trunk**: https://trunkrs.dev/

---

## Contributing

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/my-feature`
3. **Make changes** following code style guidelines
4. **Write tests** for new functionality
5. **Update documentation** (README, ARCHITECTURE)
6. **Commit with descriptive messages**: `git commit -m "Add my feature"`
7. **Push to your fork**: `git push origin feature/my-feature`
8. **Open a Pull Request**

---

## Support

For questions or issues:

1. Check existing issues
2. Review documentation (README, ARCHITECTURE)
3. Check code examples in tests/
4. Ask in discussions

---

Happy developing! 🚀
