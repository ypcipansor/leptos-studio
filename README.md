# 🎨 Leptos Studio

<div align="center">

**Build Leptos UIs visually. Export real Rust code.**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Built with Leptos](https://img.shields.io/badge/built%20with-Leptos-orange.svg)](https://github.com/leptos-rs/leptos)
[![Rust](https://img.shields.io/badge/rust-stable-%23dea584.svg)](https://www.rust-lang.org/)

[Features](#-features) • [Screenshots](#-screenshots) • [Architecture](#-architecture) • [Getting Started](#-getting-started) • [Shortcuts](#-keyboard-shortcuts) • [Contributing](#-contributing)

</div>

---

**Leptos Studio** is a visual UI builder for the [Leptos](https://github.com/leptos-rs/leptos) web framework. Drag components onto a canvas, fine-tune every property, and export clean, idiomatic Rust code — all from your browser. The builder itself is a Leptos app compiled to WebAssembly, so it doubles as a working reference for the framework it targets.

## 📸 Screenshots

Every view below is captured from the running app. The full set lives in [`docs/screenshots/`](docs/screenshots/).

### Project dashboard

Manage projects, create new ones, or import an existing layout. Each card shows the component count and last-modified time.

![Project dashboard](docs/screenshots/01-dashboard.png)

### Visual editor

The editor is split into a component palette on the left, the canvas in the middle, and a tabbed inspector on the right.

![Editor canvas](docs/screenshots/02-editor-canvas.png)

### Component palette

18 built-in components grouped into categories — Basic, Form, Layout, Media, Typography, Navigation, and Custom. Search filters the list as you type.

![Component palette](docs/screenshots/03-editor-tab-add.png)

### Layers tree

A hierarchical view of the component tree with type icons and content previews, for selecting nested nodes quickly.

![Layers tree](docs/screenshots/04-editor-tab-layers.png)

### Theme editor

Edit global design tokens — colour scales, typography, spacing, and border radius. Changes apply to the canvas immediately.

![Theme editor](docs/screenshots/05-editor-tab-theme.png)

### Variables

Define typed global variables and bind them to component properties to drive dynamic UIs.

![Variables panel](docs/screenshots/06-editor-tab-vars.png)

### Property inspector

Every property of the selected component: content, layout, spacing, typography, border, effects, and animation.

![Property inspector](docs/screenshots/07-editor-tab-properties.png)

### Component selected

Selecting a component on the canvas outlines it and fills the inspector with its editable properties.

![Component selected](docs/screenshots/17-editor-component-selected.png)

### Code view

Live preview of the generated Leptos source for the current design, syntax highlighted.

![Code view](docs/screenshots/08-editor-tab-code.png)

### History (time travel)

Every edit is recorded. Restore any earlier state, or step through with undo/redo.

![History panel](docs/screenshots/09-editor-tab-history.png)

### Git panel

Commit the current layout, browse the commit log, and restore previous revisions.

![Git panel](docs/screenshots/10-editor-tab-git.png)

### Debug panel

Inspect internal application state and performance metrics while building.

![Debug panel](docs/screenshots/11-editor-tab-debug.png)

### Export

Generate code in nine formats across framework code, web output, data, and documentation, then copy or download it.

![Export modal](docs/screenshots/12-modal-export.png)

### Template gallery

Start from a pre-built layout. Filter by category or search by name, then insert the template onto the canvas.

![Template gallery](docs/screenshots/15-modal-template-gallery.png)

![Template gallery filtered](docs/screenshots/34-modal-gallery-filtered.png)

Searching narrows the grid to matching templates.

![Template gallery search](docs/screenshots/35-modal-gallery-search.png)

### Command palette

Press `Ctrl+K` to run any action by name without leaving the keyboard.

![Command palette](docs/screenshots/16-modal-command-palette.png)

### Settings and shortcuts

Configure editor preferences, and open the shortcut reference with `?`.

![Settings modal](docs/screenshots/13-modal-settings.png)

![Shortcuts modal](docs/screenshots/14-modal-shortcuts.png)

### Canvas context menu

Right-click a component for insert, duplicate, and delete actions; right-click empty canvas space for canvas-level actions.

![Canvas context menu](docs/screenshots/32-canvas-context-menu.png)

### Save as template

Turn the current design into a reusable template with a name, description, and tags.

![Save template modal](docs/screenshots/31-modal-save-template.png)

### Zoom and preview mode

Zoom from 25% to 400% with the toolbar, `Ctrl+scroll`, or `Ctrl+=` / `Ctrl+-` / `Ctrl+0`. Preview mode hides the editing chrome to show the design as users will see it.

![Zoomed out](docs/screenshots/19-editor-zoom-out.png)

![Zoomed in](docs/screenshots/20-editor-zoom-in.png)

![Preview mode](docs/screenshots/33-editor-preview-mode.png)

### Responsive preview

Test the layout at tablet and mobile widths, or switch back to the full desktop canvas.

![Tablet preview](docs/screenshots/27-editor-responsive-tablet.png)

![Mobile preview](docs/screenshots/28-editor-responsive-mobile.png)

![Desktop preview](docs/screenshots/29-editor-responsive-desktop.png)

### Smaller viewports

The editor adapts down to laptop and mobile widths rather than overflowing.

![Dashboard at 1024px](docs/screenshots/21-dashboard-1024.png)

![Editor at 1024px](docs/screenshots/22-editor-1024.png)

![Editor at 768px](docs/screenshots/23-editor-768.png)

With the sidebar open at 768px:

![Editor at 768px with sidebar](docs/screenshots/24-editor-768-sidebar.png)

![Dashboard at 390px](docs/screenshots/25-dashboard-390.png)

![Editor at 390px](docs/screenshots/26-editor-390.png)

### Second project

The same canvas with a different project loaded, showing the builder is not tied to one layout.

![Admin project](docs/screenshots/18-editor-admin-project.png)

### Not found

Unknown routes render a styled fallback rather than a blank page.

![404 page](docs/screenshots/30-not-found.png)

## ✨ Features

### 🖱️ Visual Editor
- Drag-and-drop canvas with zoom (25%–400%) via toolbar, `Ctrl+scroll`, or `Ctrl+=` / `Ctrl+-` / `Ctrl+0`
- Pan by dragging the canvas background when zoomed
- Multi-select with `Shift`/`Ctrl`+click or `Ctrl+A`
- Context menus: right-click any component for component actions, or empty canvas space for canvas actions
- Preview mode that strips the editing chrome to show the finished design

### 🧩 Component Library
- 18 built-in components across Basic, Form, Layout, Media, Typography, and Navigation categories — buttons, text, headings, links, inputs, selects, checkboxes, radio groups, switches, images, containers, rows/columns, cards, dividers, badges, and progress bars
- Search and category filters in the palette, with per-category counts that match the rows shown
- Save any canvas component to the **Custom** category via right-click → *Save to Library*, then reuse it like a built-in
- Save any design as a reusable custom template

### 🎛️ Design & Customization
- **Variable Management** — define typed global variables (string, number, boolean) and bind them to component properties
- **Theme Editor** — customize design tokens (colours, typography, spacing, border radius) with changes applied live
- **Property Inspector** — control content, layout, spacing, typography, border, effects, and animation per component
- **Responsive Preview** — test designs at Mobile, Tablet, and Desktop viewports

### 🛠️ Developer Tools
- **Code Export** — nine formats:
  - *Framework code* — Leptos component, React/JSX, Svelte
  - *Web output* — plain HTML, HTML + Tailwind CSS
  - *Data* — raw JSON, JSON Schema, TypeScript types
  - *Documentation* — Markdown
- **History** — undo/redo plus "time travel" to restore any previous state
- **Command Palette** — run any action via `Ctrl+K` / `Cmd+K`
- **Git Panel** — commit layouts and browse or restore revision history
- **Debug Panel** — inspect internal state and performance metrics
- **Template Gallery** — start from a pre-built layout, with category filters and search
- **Project Management** — create, rename, save, and delete multiple projects
- **Auto-Save** — configurable auto-save so work is not lost

## 🏗️ Architecture

Leptos Studio is a Cargo workspace with two crates:

| Crate | Description |
| --- | --- |
| [`frontend/`](frontend/) | Leptos 0.8 WebAssembly app built with [Trunk](https://trunkrs.dev/) — see the [frontend README](frontend/README.md) for details |
| [`backend/`](backend/) | [Axum](https://github.com/tokio-rs/axum) API server that also serves the built frontend |

- The frontend uses `leptos_router` for navigation (`/` and `/editor/:id`) and `async_trait` for pluggable Git backends (remote API vs `LocalStorage`)
- The backend persists projects, templates, Git history, and analytics as plain JSON files (`backend/projects.json`, `git_data.json`, `analytics.json`), keeping state easy to inspect and portable

### API

| Method | Route | Purpose |
| --- | --- | --- |
| `GET` `POST` | `/api/projects` | List / save projects |
| `GET` `DELETE` | `/api/projects/{id}` | Load / delete a project |
| `GET` `POST` | `/api/templates` | List / save templates |
| `DELETE` | `/api/templates/{id}` | Delete a template |
| `GET` `POST` `DELETE` | `/api/projects/{id}/commits` | Git log, commit, clear history |
| `GET` `POST` | `/api/analytics` | Read / record analytics events |

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (stable; the crates declare `rust-version = "1.95.0"`)
- The WASM target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [Trunk](https://trunkrs.dev/) for frontend builds:
  ```bash
  cargo install --locked trunk
  ```

### Development

Run the backend from the repository root so it can find `dist/` and the JSON data files:

1. **Build the frontend:**
   ```bash
   trunk build
   ```

2. **Start the backend:**
   ```bash
   cd backend && cargo run
   ```

3. Open [http://localhost:3000](http://localhost:3000).

For frontend work, run `trunk serve` instead. It watches sources and rebuilds on change, serving on [http://localhost:8899](http://localhost:8899); run the backend separately alongside it for persistence.

### 🐳 Docker Deployment

Run the full stack in one container:

```bash
docker-compose up --build
```

Access the application at [http://localhost:3000](http://localhost:3000).

## 🧪 Development Checks

These are the same checks CI runs:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib --bins
```

The suite covers 107 unit tests across the domain models, validation, sanitisation, syntax highlighting, and formatting utilities.

## ⌨️ Keyboard Shortcuts

Press `?` in the app to see this list at any time.

| Keys | Action |
| --- | --- |
| `Ctrl + K` | Open command palette |
| `Ctrl + S` | Save project |
| `Ctrl + E` | Export code |
| `Ctrl + N` | New component |
| `Ctrl + Z` | Undo |
| `Ctrl + Shift + Z` / `Ctrl + Y` | Redo |
| `Ctrl + C` / `Ctrl + V` | Copy / paste component |
| `Ctrl + X` | Cut component |
| `Ctrl + D` | Duplicate selection |
| `Ctrl + A` | Select all components |
| `Delete` / `Backspace` | Delete selection |
| `Ctrl + =` / `Ctrl + -` | Zoom in / out |
| `Ctrl + 0` | Reset zoom to 100% |
| `Esc` | Deselect |

## 🤝 Contributing

Contributions are welcome! To get started:

1. Fork the repository
2. Create a feature branch
3. Make your change and run the checks above
4. Commit and push to the branch
5. Open a Pull Request

See [CONTRIBUTING.md](frontend/CONTRIBUTING.md) for conventions and the [CHANGELOG](CHANGELOG.md) for notable changes.

## 📄 License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
