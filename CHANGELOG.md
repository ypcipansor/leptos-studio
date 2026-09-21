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
- **Palette entries sharing a `kind` produced the wrong component.** Heading and Link both
  declared `kind: "Text"` and had no template, so `palette_drag_payload` emitted the bare kind and
  `create_canvas_component("Text")` built a default paragraph: the advertised Heading became a
  paragraph and Link became plain text. Link is now a real `CanvasComponent::Link` (with an `href`
  the renderer and every exporter emit as an anchor), Heading carries a template that sets
  `TextTag::H1`, and both resolve through their stable library-entry id.
- **The skip link pointed at `##main-canvas`.** `SkipLink` interpolated `target` into `#{}` while
  the editor passed an already-hashed `"#main-canvas"`, so the link never resolved; its target also
  lacked `tabindex="-1"`, so the click handler's `.focus()` was a no-op. The component now strips a
  leading `#` and the canvas region is programmatically focusable.
- **"Save to Library" silently did nothing.** Saving a canvas component as a custom component
  showed a success notification but only appended to `custom_components`, never to
  `component_library`, so it never appeared in the palette. It now registers through
  `ComponentRegistry::add_custom` and shows up under *Custom* immediately.
- **Saved components lost their design when reused.** A saved entry kept the full component as
  JSON in its `template`, but dragging it back into the canvas rebuilt a default component from
  its `kind`, discarding every property, style, and child. The drag payload now identifies the
  saved entry by its new stable `id` (`Saved::<id>`) and the canvas deserializes its template with
  fresh ids, for every component type. Names are no longer used as identity, so two saved entries
  with the same display name — or a saved name that collides with a built-in — each resolve to
  their own design, and delete/rename target the right entry.
- **Saved-component names were unvalidated.** `add_custom` and `update_custom_by_index` now trim
  the name and reject blank or duplicate (case-insensitive) names with a notification instead of
  storing an entry the palette could not tell apart.
- **Keyboard shortcuts mutated the canvas behind an open modal.** The global keydown listener only
  ignored events from text inputs, so Delete, `Ctrl+Z`, and `Ctrl+A` still edited the hidden canvas
  while Export, Settings, Shortcuts, Template Gallery, Command Palette, or the save-template dialog
  was open. `KeyboardHandler` now takes a `modal_open` signal derived from the *same* signals that
  render each modal and stays dormant while any dialog is visible. The Export modal previously had
  two signals — a local `show_export` and `app_state.ui.show_export_modal` — and the gate watched
  the wrong one; they are consolidated onto `app_state.ui.show_export_modal`.
- **Palette rows claimed keyboard support they did not have.** The rows are focusable and exposed
  as `option`s but only had drag handlers. `Enter`/`Space` now adds the component to the canvas
  root (undoable, and selected), matching the documented behaviour.
- **The browser test suite rewrote `backend/projects.json`.** `AppState::new` runs
  `initialize_project_state` and `setup_auto_save`, so mounting the real app in `wasm-pack test`
  seeded the canvas from the newest project and auto-saved over it whenever a backend was
  reachable, mutating tracked runtime data as a side effect of running tests. Both are now skipped
  under `cfg(test)`, and the canvas assertions compare against a pre-interaction snapshot rather
  than an absolute component count.
- Removed the duplicate "Image" entry from the default library.
- **Concurrent project saves could corrupt persistence.** `save_project` / `delete_project` mutated
  `store.projects`, released the write lock, and then wrote the file in a separate step, so two
  overlapping requests could persist snapshots out of order from the order the state changed, and a
  failed write's rollback could overwrite another request that had already committed. Every
  mutating request now runs as one transaction under a dedicated `mutation_lock` (`Store::commit`):
  the state is snapshotted under the write lock, the lock is released *before* the filesystem
  `await` (so readers are never blocked by I/O), and the write happens inside the exclusive
  transaction, so write order matches state-change order and a rollback can only touch this
  transaction's own change. The file itself is written atomically — a unique temp file in the same
  directory, `sync_all`, then `rename` — so a crash cannot leave truncated JSON. After the rename
  the parent directory is opened and `sync_all`ed, so the new directory entry is durable too: on a
  filesystem that needs an explicit directory fsync a power loss could otherwise replay the old
  entry and resurrect the previous file even though the API had acknowledged the save. A failure in
  that final step is reported as `WriteError::PublishedNotDurable` rather than rolled back — the
  new state is already visible to readers, so restoring memory would make it disagree with the file
  — but it still surfaces as an error instead of a false success.
- **`DELETE` for an absent id reported 500 when storage was failing.** `delete_project` (and the
  template and git-history equivalents) called `Store::commit` unconditionally, so a no-op removal
  still attempted a write and a broken storage layer turned a request that should be a 404 into a
  500. The mutation step now returns `Mutation::Changed`/`Unchanged`, and `commit` persists only a
  change — decided inside the transaction, while the mutation lock is held, so no check-then-delete
  race is introduced.
- **Path validation rejected legal names containing `..`.** `resolve_data_file` used a substring
  test, so a valid name such as `archive..old/projects.json` or `projects..backup.json` was refused.
  It now rejects only a real `..` path *component* (which is what can walk upwards) while still
  canonicalising and rebuilding the path, so confinement is preserved and CodeQL reports no new
  `rust/path-injection` alert.
- **Templates, git history, and analytics persistence were not transaction-serialised.** The three
  non-project stores overwrote their JSON with a plain `fs::write` — neither atomic nor durable, and
  liable to lose a concurrent update. All four stores now share the one `Store` implementation in
  `backend/src/store.rs` and mutate through `Store::commit`, so they get the same atomic-write,
  directory-fsync, exclusive-transaction, and rollback guarantees as projects. Each has a
  concurrency test asserting the file matches memory.
- **Leptos export HTML-escaped data into Rust string literals.** The Leptos generator passed a
  link's `href` and text through `escape_html` before embedding them in a `view!` string literal;
  Leptos sets that literal on the DOM verbatim without decoding entities, so `?a=1&b=2` rendered as
  `?a=1&amp;b=2`. Literals are now escaped for Rust (`rust_string_literal`), while the markup
  exporters (HTML, Vue, Svelte, Tailwind) and Markdown keep entity escaping, where it is correct.
- **The dashboard under-counted nested components.** `save_project` reported a recursive component
  count via `validation::count_components`, but `GET /api/projects` used `layout.as_array().len()`,
  so after a refresh a project with children inside a Container or Card showed only its root count
  and the number disagreed with what saving had reported. `list_projects` now uses the same
  recursive `validation::count_components`, so there is one definition of the count.
- **Projects containing a Link could not be saved.** The frontend's `ComponentType` includes `Link`,
  but the backend's `KNOWN_COMPONENT_TYPES` allowlist did not, so `validate_component` rejected any
  layout containing a hyperlink as an unknown type and `POST /api/projects` answered **422** — the
  whole project, not just the link, was lost. `Link` is now in the allowlist, and
  `known_component_types_match_the_frontend` reads the frontend enum in a test so a future variant
  cannot silently drift out of sync again.
- **Exported Links dropped their style and animation.** Every visual exporter (Leptos, HTML, React,
  Vue, Svelte, Tailwind HTML) emitted a bare `<a href>` while the canvas renderer applied the link's
  `ComponentStyle` and animation, so a styled link looked different once exported. All of them now
  emit the same inline CSS/animation as the canvas via shared helpers
  (`component_inline_css` / `style_js_properties`); Markdown, which cannot express styling, emits a
  real `[text](url)` link instead of a bare label. href, text, and attribute values are escaped.
  This parity covers every `ComponentStyle` field the canvas applies; the reserved
  `custom_css` field is applied by neither, and its semantics are documented as undefined.
- **The Template Gallery's Escape listener outlived the gallery.** `TemplateGallery` created a local
  `RwSignal::new(true)` and handed it to `use_escape_key`, so the global listener stayed armed for
  the component's whole lifetime rather than the gallery's visibility. If the parent ever hid the
  gallery without unmounting it, Escape kept calling the stale close callback and could steal Escape
  from another modal. The component now takes the caller's `show` signal — the same one that renders
  it — and gates the listener on it.
- `backend/projects.json` restored to its base state: the two screenshot-session demo projects
  (`Admin Dashboard Shell`, `Marketing Landing Page`) are no longer in tracked runtime data.

### Documentation

- Corrected the click-to-place claim (`frontend/QUICKSTART.md`) to the actual keyboard activation,
  and stated explicitly that **Save to Library** entries are session-only — a saved project carries
  the canvas layout but not the library — across `README.md`, `frontend/README.md`,
  `frontend/QUICKSTART.md`, and `AGENTS.md`. Removed the stale static unit-test count from the
  root README.

### Changed

- Canvas stylesheet is now complete for all rendered components (`canvas-*` classes).
- Leptos export no longer uses `RefCell` to track reactive signals (thread-local correctness fixed);
  imports updated to `leptos::prelude::*`.

### Tests

- Frontend: unit tests green, `wasm_smoke` browser test added, export regression test covering
  all components across generators. New regression tests cover the saved-component id round trip
  (same-named entries, a saved name shadowing a built-in, nested container/card id regeneration,
  rename/delete targeting), name validation, palette keyboard activation, and modal gating of the
  canvas shortcuts (Export included) driven through the real modal signals.
- Backend: validation module with its own unit tests.
- CI runs `cargo fmt`, `cargo clippy --all-targets` and unit tests; WASM artifacts uploaded.
