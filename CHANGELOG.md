# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Six new components: **Divider**, **Checkbox**, **RadioGroup**, **Switch**, **Badge**, **Progress**
  with property editors, canvas rendering, unit tests, and export codegen for every generator
  (Leptos, HTML, JSON, Markdown, and advanced React/Vue/Svelte/Tailwind-HTML exporters).
- Component baseline CSS bundle embedded into Plain-preset Leptos exports.
- Preview-mode **split button with dropdown** in the toolbar (choose Edit/Preview explicitly),
  with keyboard support (Escape closes, Arrow keys cycle options) and ARIA menu roles.
- Context-menu behavior unified: **double-click = context menu**; right-click on empty canvas opens
  a generic menu with *Add Container*.
- Canvas ARIA semantics: surface is `role="application"`, rendered components are `role="listitem"`.
- **Canvas zoom (25%–400%)**: toolbar controls (`−` / percentage / `+`, click percentage to reset),
  `Ctrl+scroll` wheel zoom, `Ctrl+=` / `Ctrl+-` / `Ctrl+0` shortcuts, and drag-to-pan on the canvas
  background when zoomed in.
- **Multi-select**: `Shift`/`Ctrl`+click toggles components in the selection set, `Ctrl+A` selects
  all, `Escape` clears, Delete removes every selected component, and the status bar shows the
  selection count. Selection state is pruned automatically on undo/redo and history restore.
- **Backend validation**: `POST /api/projects` rejects serialized layouts whose component type tags
  are unknown (HTTP 422), and component counts now include nested container children.

### Fixed

- **Component palette category filters were unreachable.** Category matching used hardcoded
  component-kind lists that disagreed with the `category` field each component declares, so
  Media, Typography, and Navigation components appeared only under *All* and had no tab of their
  own. Matching now derives from the declared category, and tabs exist for all of them.
- **Palette badges disagreed with the filtered list.** Each tab's count is now computed with the
  same predicate that filters the list, so the badge always matches the number of rows shown.
- **"Save to Library" silently did nothing.** Saving a canvas component as a custom component
  showed a success notification but only appended to `custom_components`, never to
  `component_library`, so it never appeared in the palette. It now registers through
  `ComponentRegistry::add_custom` and shows up under *Custom* immediately.
- **Saved components lost their design when reused.** A saved entry kept the full component as
  JSON in its `template`, but dragging it back into the canvas rebuilt a default component from
  its `kind`, discarding every property, style, and child. The drag payload now identifies the
  saved entry (`Saved::<name>`) and the canvas deserializes its template with fresh ids, for every
  component type.
- **Keyboard shortcuts mutated the canvas behind an open modal.** The global keydown listener only
  ignored events from text inputs, so Delete, `Ctrl+Z`, and `Ctrl+A` still edited the hidden canvas
  while Export, Settings, Shortcuts, Template Gallery, Command Palette, or the save-template dialog
  was open. `KeyboardHandler` now takes a `modal_open` signal and stays dormant while any dialog is
  visible.
- Removed the duplicate "Image" entry from the default library.

### Changed

- Canvas stylesheet is now complete for all rendered components (`canvas-*` classes).
- Leptos export no longer uses `RefCell` to track reactive signals (thread-local correctness fixed);
  imports updated to `leptos::prelude::*`.

### Tests

- Frontend: 102 unit tests green, `wasm_smoke` browser test added, export regression test covering
  all components across generators.
- Backend: new validation module with 5 unit tests.
- CI runs `cargo fmt`, `cargo clippy --all-targets` and unit tests; WASM artifacts uploaded.
