# Leptos Studio — Frontend

The Leptos Studio frontend is a [Leptos](https://github.com/leptos-rs/leptos) 0.8 application compiled to WebAssembly. It renders the entire visual builder — canvas, palette, inspector, and all editor panels — in the browser. It talks to the [backend](../backend/) over HTTP for project and template persistence.

For the project overview, screenshots, and architecture, see the [root README](../README.md).

## 🚀 Features

### Visual Editor
*   **Drag-and-drop canvas** — drag components from the palette onto the surface.
*   **Zoom (25%–400%)** — toolbar controls, `Ctrl+scroll`, or `Ctrl+=` / `Ctrl+-` / `Ctrl+0`; drag the canvas background to pan when zoomed in.
*   **Multi-select** — `Shift`/`Ctrl`+click to toggle components, `Ctrl+A` to select all, `Esc` to clear.
*   **Context menus** — right-click a component for component actions, or empty canvas space for canvas actions.
*   **Preview mode** — hides the editing chrome so the design renders as users will see it.

### Component Library
18 built-in components, grouped into filterable categories:

*   **Basic** — Button, Text, Input, Select, Badge, Progress.
*   **Form** — Checkbox, RadioGroup, Switch.
*   **Layout** — Container, Row, Column, Card, Divider, Div.
*   **Media** — Image.
*   **Typography** — Heading.
*   **Navigation** — Link.
*   **Custom** — components you have saved from the canvas via the context menu's **Save to Library** action.

The palette supports fuzzy search across names and descriptions, and each category tab shows how many components it contains. Components saved with **Save to Library** are added to the **Custom** category immediately; dragging one back out restores the saved design under a fresh id. Saved names are validated — trimmed, non-blank, and unique — and library entries are session-only state that a saved project does not carry.

### Design & Customization
*   **Property Editor** — content, layout, spacing, typography, border, effects, and animation per component.
*   **Variable Management** — define typed global variables (string, number, boolean) and bind them to component properties.
*   **Theme Editor** — global design tokens: colour scales, typography, spacing, and border radius, applied live.
*   **Responsive Preview** — Mobile (375px), Mobile Landscape (667px), Tablet (768px), Tablet Landscape (1024px), and Desktop (full width).
*   **Styling system** — design tokens exposed as CSS variables, with Flexbox-based layouts.

### Developer Tools
*   **Code Export** — nine formats:
    *   *Framework code* — Leptos component, React/JSX, Svelte.
    *   *Web output* — plain HTML, HTML + Tailwind CSS.
    *   *Data* — raw JSON, JSON Schema, TypeScript types.
    *   *Documentation* — Markdown.
*   **History** — undo/redo with a full snapshot stack, plus restore-to-any-point time travel.
*   **Git Panel** — commit layouts and browse or restore revision history. Uses the backend API when a project is server-backed, and falls back to browser LocalStorage for offline and new projects.
*   **Command Palette** — run any action by name with `Ctrl+K`.
*   **Layers Tree** — hierarchical component tree for selecting nested nodes.
*   **Debug Panel** — inspect internal state and render-time metrics.
*   **Template Gallery** — 8 built-in templates across 8 categories (Form, Hero, Navigation, Card, Dashboard, Footer, LandingPage, Custom), with search and filtering.

## 🛠️ Tech Stack

*   **Language**: [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **Framework**: [Leptos](https://github.com/leptos-rs/leptos) 0.8 (CSR) with `leptos_router`
*   **Build Tool**: [Trunk](https://trunkrs.dev/)
*   **WASM Target**: `wasm32-unknown-unknown`
*   **Persistence**: backend HTTP API; browser LocalStorage for Git history on local projects

## 🏁 Getting Started

For a task-oriented walkthrough, see [QUICKSTART.md](QUICKSTART.md).

### Prerequisites

*   [Rust](https://rustup.rs/) (stable)
*   WASM target: `rustup target add wasm32-unknown-unknown`
*   [Trunk](https://trunkrs.dev/): `cargo install --locked trunk`

### Installation & Running

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/analisaperlengkapan/leptos-studio.git
    cd leptos-studio
    ```

2.  **Run the development server**:
    ```bash
    trunk serve
    ```

3.  **Open the application**:
    Navigate to `http://localhost:8899` in your browser.

`trunk serve` rebuilds on change but does not serve the API. Run the backend in a second terminal for project and template persistence:

```bash
cargo run -p backend
```

To serve everything from one origin instead, build with `trunk build` and run that same backend command — it serves the root `dist/` on `http://localhost:3000`.

The backend resolves its paths from `CARGO_MANIFEST_DIR`, never from the working directory: data
defaults to `backend/projects.json` and assets to the root `dist/`. `cd backend && cargo run` and
`cargo run -p backend` from the root are equivalent. `STATIC_DIR`, `DATA_FILE`,
`TEMPLATES_FILE`, `GIT_DATA_FILE` and `ANALYTICS_DATA_FILE` override them.

### Running the checks

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
```

##  Project Structure

For an in-depth architecture overview, see [ARCHITECTURE.md](ARCHITECTURE.md); for module-level API details, see [API.md](API.md).

*   **`src/app.rs`**: Application entry point, routing, and top-level layout.
*   **`src/builder/`**: Builder UI — canvas, palette, property editor, panels, and modals.
*   **`src/domain/`**: Data models and validation rules.
*   **`src/services/`**: Use cases — export, project, Git, and templates.
*   **`src/state/`**: Global state built on Leptos signals.
*   **`src/utils/`**: Helpers for clipboard, sanitisation, syntax highlighting, and formatting.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to get started, run tests, and submit pull requests.

## 📄 License

This project is licensed under the Apache-2.0 License — see the [LICENSE](LICENSE) file for details.