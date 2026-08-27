# 🎨 Leptos Studio

<div align="center">

**Build Leptos UIs visually. Export real Rust code.**

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Built with Leptos](https://img.shields.io/badge/built%20with-Leptos-orange.svg)](https://github.com/leptos-rs/leptos)
[![Rust](https://img.shields.io/badge/rust-stable-%23dea584.svg)](https://www.rust-lang.org/)

[Features](#-features) • [Architecture](#-architecture) • [Getting Started](#-getting-started) • [Shortcuts](#-keyboard-shortcuts) • [Contributing](#-contributing)

</div>

---

**Leptos Studio** is a visual UI builder for the [Leptos](https://github.com/leptos-rs/leptos) web framework. Drag and drop components onto a canvas, fine-tune every detail, and export clean, idiomatic Rust code — all from your browser.

## ✨ Features

### 🖱️ Visual Editor
- Drag-and-drop canvas with zoom (25%–400%) via toolbar, `Ctrl+scroll`, or `Ctrl+=/-/0`
- Pan by dragging the canvas background when zoomed
- Multi-select with `Shift`/`Ctrl`+click or `Ctrl+A`
- Context menus: double-click (or right-click) any component; right-click the empty canvas for quick actions

### 🧩 Component Library
- Built-in components: Buttons, Text, Inputs, Selects, Images, Containers, Cards, Dividers, Checkboxes, Radio Groups, Switches, Badges, and Progress bars
- Support for custom templates

### 🎛️ Design & Customization
- **Variable Management** — define global variables and bind them to component properties for dynamic UIs
- **Theme Editor** — customize design tokens (colors, typography, spacing, border radius) visually
- **Responsive Preview** — test designs on Mobile, Tablet, and Desktop viewports

### 🛠️ Developer Tools
- **Code Export** — generate production-ready Leptos Rust code, HTML, JSON, or Markdown (plain exports embed a baseline CSS bundle)
- **History** — robust Undo/Redo with "Time Travel" to restore any previous state
- **Command Palette** — quick access to all actions via `Ctrl+K` / `Cmd+K`
- **Project Management** — create, save, and manage multiple projects
- **Auto-Save** — never lose your work with configurable auto-save

## 🏗️ Architecture

Leptos Studio is a Cargo workspace with two crates:

| Crate | Description |
| --- | --- |
| [`frontend/`](frontend/) | Leptos WebAssembly app built with [Trunk](https://trunkrs.dev/) — see the [frontend README](frontend/README.md) for details |
| [`backend/`](backend/) | [Axum](https://github.com/tokio-rs/axum)-based API server |

- The frontend uses `leptos_router` for navigation (`/`, `/editor/:id`) and `async_trait` for pluggable Git backends (remote vs `LocalStorage`)
- The backend handles persistence for projects, templates, Git history, and analytics, storing data in simple JSON files for portability

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable)
- [Trunk](https://trunkrs.dev/) for frontend builds:
  ```bash
  cargo install trunk
  ```

### Development

1. **Start the backend:**
   ```bash
   cd backend
   cargo run
   ```

2. **Start the frontend:**
   ```bash
   cd frontend
   trunk serve
   ```

3. Open [http://localhost:8899](http://localhost:8899) in your browser.

### 🐳 Docker Deployment

Run the full stack in one container:

```bash
docker-compose up --build
```

Access the application at [http://localhost:3000](http://localhost:3000).

## ⌨️ Keyboard Shortcuts

| Group | Keys | Action |
| --- | --- | --- |
| General | `Ctrl + K` | Open Command Palette |
| General | `Ctrl + S` | Save Project |
| General | `Ctrl + E` | Export Code |
| General | `?` | Show Shortcuts Help |
| Editing | `Ctrl + Z` | Undo |
| Editing | `Ctrl + Y` | Redo |
| Editing | `Ctrl + C` / `Ctrl + V` | Copy / Paste |
| Editing | `Delete` | Delete Selected |
| Selection | `Ctrl + A` | Select All |
| Selection | `Esc` | Deselect |

## 🤝 Contributing

Contributions are welcome! To get started:

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Open a Pull Request

See the [CHANGELOG](CHANGELOG.md) for notable changes.

## 📄 License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)
