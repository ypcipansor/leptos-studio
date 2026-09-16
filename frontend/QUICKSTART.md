# Leptos Studio — Quick Start Guide

Go from an empty canvas to exported Leptos code in a few minutes.

## What is Leptos Studio?

Leptos Studio is a visual UI builder for the [Leptos](https://leptos.dev) framework (Rust + WebAssembly). It gives you:

- **Drag & drop** component building on a zoomable canvas
- **Live preview** as you edit
- **Code export** in nine formats (Leptos, React, Svelte, HTML, Tailwind HTML, JSON, JSON Schema, TypeScript, Markdown)
- **Responsive preview** at mobile, tablet, and desktop widths
- **Custom components** from your own HTML templates
- **Undo/redo and history**, plus Git-style commits
- **Project management** with save, load, rename, and delete
- **Templates** — start from one of eight built-in layouts

## Installation & Setup

### 1. Prerequisites

Ensure you have:

- Rust toolchain (stable): https://rustup.rs/
- WASM target: `rustup target add wasm32-unknown-unknown`
- Trunk (WASM bundler): `cargo install --locked trunk`

### 2. Clone the Repository

```bash
git clone https://github.com/analisaperlengkapan/leptos-studio.git
cd leptos-studio
```

### 3. Start the Development Server

```bash
trunk serve
```

The application will be available at `http://localhost:8899`.

`trunk serve` does not run the API, so projects and templates will not persist. For the full experience, run the backend in a second terminal from the repository root:

```bash
cd backend && cargo run
```

## Basic Usage

### 1. Add Components

1. Open the **Add** tab in the sidebar.
2. Filter by category if you like — **All**, **Basic**, **Form**, **Layout**, **Media**, **Typography**, **Navigation**, or **Custom**. Each tab shows its component count.
3. Use the search box to filter by name or description.
4. **Drag** a component onto the canvas, or click it to place it.

### 2. Edit Component Properties

1. Click a component on the canvas to select it.
2. Open the **Properties** tab in the right-hand panel.
3. Adjust content, layout, spacing, typography, border, effects, and animation.

To work with several components at once, `Shift`/`Ctrl`+click to toggle them into the selection, or press `Ctrl+A` to select everything.

### 3. Save a Component to the Library

Turn any component you have already styled into a reusable custom component:

1. Right-click the component on the canvas.
2. Choose **"💾 Save to Library"**.
3. Enter a name when prompted.

The component is added to the **Custom** category in the palette, and you can drop copies onto the canvas from there. The **All** count increases to match.

Note that saved components live in the current browser session, so save the project (or export it) if you want to keep them.

### 4. Preview on Different Devices

Use the responsive controls to check the layout at different widths:

- **Mobile** — 375px wide
- **Mobile Landscape** — 667px wide
- **Tablet** — 768px wide
- **Tablet Landscape** — 1024px wide
- **Desktop** — full width

For a clean look at the finished design, toggle **Preview Mode** to hide the editing chrome.

### 5. Zoom and Pan

- **Zoom** with the toolbar `−` / `+` controls, `Ctrl+scroll`, or `Ctrl+=` / `Ctrl+-`.
- **Reset** to 100% by clicking the percentage readout or pressing `Ctrl+0`.
- **Pan** by dragging the canvas background while zoomed in.

### 6. Export Code

1. Click **Export** in the toolbar, or press `Ctrl+E`.
2. Choose a format:
   - **Framework code** — Leptos Component, React/JSX Component, Svelte Component
   - **Web output** — Plain HTML, HTML + Tailwind CSS
   - **Data** — Raw JSON, JSON Schema, TypeScript Types
   - **Documentation** — Markdown
3. **Copy** to the clipboard or **Download** as a file.

The **Code** tab in the right-hand panel always shows the live Leptos source for the current design.

### 7. Save & Load Projects

- **Save** — press `Ctrl+S`, or use **Save** in the toolbar.
- **Dashboard** — the home page lists all projects with component counts and last-modified times. Open, rename, or delete from there.
- **Import / Export** — use **Import** to load a project from JSON, and the export options to save one out.

Projects are stored by the backend in `backend/projects.json`.

### 8. Track Changes

- **History** — every edit is recorded; undo with `Ctrl+Z`, redo with `Ctrl+Y`, or restore any earlier state from the list.
- **Git** — commit the current layout and browse or restore previous revisions from the **Git** tab.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
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

On macOS, use `Cmd` in place of `Ctrl`.

## Advanced Features

### Search and Filter Components

Type in the sidebar search box to filter the palette by component name or description. Combine it with a category tab to narrow the results further.

### Apply Themes

Open the **Theme** tab to edit global design tokens — colour scales, typography, spacing, and border radius. Changes apply to the canvas immediately.

### Manage Variables

Open the **Vars** tab to define typed global variables (string, number, boolean) and bind them to component properties, so a design can be driven by shared values instead of repeated literals.

### Start From a Template

Open the **Template Gallery** to browse eight built-in layouts — Login Form, Contact Form, Hero Section, Pricing Card, Navigation Bar, Footer, Dashboard Header, and Feature Grid. Filter by category or search by name, then insert one onto the canvas.

### Save a Template

Use **Save as Template** in the toolbar to turn the current design into a reusable template with a name, description, and tags.

### Inspect the App

The **Debug** tab shows internal application state and render-time metrics, which is useful when investigating performance or state issues.

## Troubleshooting

### Components Not Appearing

- Confirm you dropped the component onto the canvas surface rather than the surrounding chrome.
- Check the **Layers** tab — the component may be nested inside a container.

### Export Issues

- Make sure at least one component is on the canvas.
- Try a different format — some formats include more detail than others.

### Projects Not Saving

- Check that the backend is running (`cd backend && cargo run`), since `trunk serve` alone does not provide the API.
- Look for an error notification in the app, then check the backend log.

## Next Steps

- Read the [frontend README](README.md) for the full feature list.
- See [ARCHITECTURE.md](ARCHITECTURE.md) for how the layers fit together.
- Check the [root README](../README.md) for screenshots and architecture diagrams.