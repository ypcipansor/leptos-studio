# AGENTS.md

Repository-specific context for Leptos Studio (Leptos 0.8/WASM visual UI builder + Rust backend).

## Layout

- `frontend/` — Leptos 0.8 `csr` app, edition 2024, `wasm32-unknown-unknown`, built with Trunk.
- `backend/` — Axum server. Serves `dist/` plus the JSON API; data lives in `backend/projects.json`.

## Build, test, lint

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --lib --bins     # what CI runs
cd frontend && trunk build              # produces dist/
```

- `cargo test --target wasm32-unknown-unknown` does **not** work (the harness and `mio` fail to
  build for wasm). The browser-only suite `frontend/tests/wasm_smoke.rs` runs via
  `wasm-pack test --headless --chrome`.
- Clippy emits one benign third-party note about `proc-macro-error2`; there are no first-party
  warnings.

## Running locally

- `trunk serve` on `:8899` serves the UI but **not** the API, so projects and templates will not
  persist.
- `cd backend && cargo run` serves the built `dist/` and the API on `:3000`. The backend resolves
  its paths from `env!("CARGO_MANIFEST_DIR")`, never from the working directory: `DATA_FILE` defaults
  to `backend/projects.json` and `STATIC_DIR` to the **root** `../dist`. So `cargo run -p backend`
  from the root and `cd backend && cargo run` are equivalent — do not document one of them as
  requiring a particular cwd. Path resolution lives in `backend/src/paths.rs` as pure `*_from`
  functions (override + base) so it is testable without touching the environment; never call
  `set_var` in tests, run them in parallel. `backend/src/analytics.rs`, `git.rs` and `templates.rs`
  follow the same rule via `ANALYTICS_DATA_FILE` / `GIT_DATA_FILE` / `TEMPLATES_FILE`.
  `Dockerfile` / `docker-compose.yml` set all five to absolute paths.

## Conventions and gotchas

- `builder/sidebar.rs` is **dead code** — only declared in `builder/mod.rs`, never rendered. The
  live palette is `builder/component_palette.rs`, rendered from `pages/editor.rs`.
- The palette's category tabs must stay derived from each `LibraryComponent.category` string.
  Hardcoding component-kind lists per category previously made Media/Typography/Navigation
  components unreachable outside *All*. Badge counts use the same `ComponentCategory::matches`
  predicate as the row filter; keep them in sync (see
  `test_every_component_is_reachable_by_category`).
- `builder/component_library.rs::default_library_components()` is the **single source of truth**
  for the live library `UiState.component_library` is seeded with. It appends Div, Heading and Link
  to `builtin_library_components()`, so palette tests must call *it*, not
  `builtin_library_components()`, or the three appended entries go untested.
- A palette entry's `kind` is **not** a unique design key: Heading and Link both declare
  `kind: "Text"`. An entry that must produce something other than the kind's default carries a
  `template`, and `palette_drag_payload` then emits `Saved::<id>` rather than the bare kind. Adding
  a named built-in whose design differs from its kind's default *requires* a template — a new kind
  string alone changes nothing unless `create_canvas_component` also handles it. Link needed a real
  `CanvasComponent::Link` because a hyperlink's `href` cannot be expressed by `TextComponent`.
  Every `CanvasComponent` variant must be handled at all seven enumeration sites: the renderer, the
  property editor, the tree view (label + icon), the breadcrumb, the preview, and both exporters
  (`export_service.rs`, `export_advanced.rs`), plus `duplicate_with_new_id`.
- `SkipLink` normalises its `target` prop: it accepts `"main-canvas"` or `"#main-canvas"` and emits
  a single `#`. The canvas region it targets needs `tabindex="-1"` for the click handler's
  `.focus()` to do anything. `#main-canvas` is the `<section>` in `pages/editor.rs`; the inner
  surface that the click/pan handlers compare against is `#canvas-surface` — they must not share an
  id. Pinned by the `wasm_tests` module in `pages/editor.rs`.
- Browser-only tests belong in the **lib** target (`#[cfg(all(test, target_arch = "wasm32"))]`
  modules run by `wasm-pack test --lib`). The `wasm-pack` *integration* targets
  (`frontend/tests/*.rs`) fail with `the name 'main' is exported by multiple crates`, because
  `lib.rs`'s `#[wasm_bindgen(start)] fn main` is compiled into them — `#[cfg(not(test))]` only
  applies to the lib-target compilation. When mounting `EditorPage` in a test, construct
  `AppState`/`DerivedState`/`AnalyticsService` *inside* the `mount_to` closure: `mount_to` installs
  the global executor and the `Owner` that effects and `provide_context` need.
- Saving a canvas component as a custom component must go through
  `ComponentRegistry::add_custom`, which writes to both `custom_components` and
  `component_library`. Writing only to `custom_components` leaves it invisible in the palette.
- A library entry is identified by its `id`, not its `name`: two saved entries can share a
  display name and a saved name can collide with a built-in. `palette_drag_payload` emits
  `Saved::<id>` for entries with a template, and `create_canvas_component_from_payload`
  looks the entry up by id, deserializes the template and regenerates ids (recursively).
  Delete and rename also target by id. `id` has `#[serde(default = "new_library_id")]` so
  project JSON written before the field existed still loads. `add_custom` and
  `update_custom_by_index` return `LibraryNameError` for blank or duplicate (case-insensitive)
  names instead of storing an ambiguous entry.
- The palette rows are focusable `option`s, so they must be keyboard-activatable: `Enter`/`Space`
  adds the component to the canvas root (`is_palette_activation_key`). Drag handlers alone would
  make the ARIA semantics a lie.
- `component_library` / `custom_components` are in-memory signals seeded from
  `default_library_components()`; `Project` and `apply_project` do not carry them, so saved
  library entries do not survive a reload or a project reopen. The docs say so — do not claim
  otherwise without adding real serialization plus a round-trip test.
- Modal visibility gates the global shortcuts via `KeyboardHandler`'s `modal_open` prop, derived in
  `pages/editor.rs` from `editor_modal_open(...)`. Every argument must be the *same* signal that
  renders the modal: the Export modal is `app_state.ui.show_export_modal` (aliased as
  `show_export` in `EditorPage`), which is also what `use_export_actions` opens — there is no
  second local signal. Without the gate, Delete, `Ctrl+Z` and friends would edit the canvas hidden
  behind an open dialog. Any new modal must be added to that derivation and to the
  `every_editor_modal_gates_shortcuts` test — but blocking the canvas shortcuts must not block a
  modal's *own* keyboard controls. The Command Palette is the case to watch: because `modal_open`
  suppresses the global handler, the palette handles ArrowUp/ArrowDown/Enter/Escape itself, which
  only works if focus is inside the dialog. It moves focus to its search input on the open edge
  (`CommandPalette`, driven by an `Effect` + `request_animation_frame` since the node ref is only
  populated after render), restores the previously focused element on close, and traps `Tab`. If a
  future change drops that focus move, `Ctrl+K` silently loses every key that is not typing. The
  per-key decisions live in the pure `palette_key_action` / `step_selection` helpers, and the DOM
  behaviour is pinned by the `wasm_tests` module in `builder/command_palette.rs` (mounts the real
  component under `wasm-pack test --headless --chrome`). The palette's search signal is created in
  `EditorPage`'s body, not inside `view!`, so it survives re-renders.
- `AppState::new` skips `initialize_project_state` / `setup_auto_save` under `cfg(test)`. The
  browser suite mounts the real app, so with a backend reachable those two would seed the canvas
  from the newest project and auto-save over it — rewriting the tracked `backend/projects.json` as
  a side effect of running tests. Tests must not read or write persisted data, so canvas
  assertions compare against a snapshot taken just before the interaction (`newly_added`) instead
  of the absolute component count. Keep the `#[cfg_attr(test, allow(dead_code))]` on both methods.
- `backend/projects.json` is tracked runtime data, not a fixture. Screenshot capture must not
  mutate it; keep demo data out of it (use a separate fixture or document the manual step).
- Use `history_rw.get_untracked()` inside async handlers to avoid reactive-cycle panics.
- `cargo test --workspace` also runs integration tests that hit the network/backend; prefer
  `--lib --bins` for a fast local loop.

## Adding a component variant

A new `CanvasComponent` / `ComponentType` variant serialises to a new type tag, and *every* place
that enumerates the variants must learn about it. Missing one is not a cosmetic gap: the backend
allowlist gate rejects the whole layout, so the project cannot be saved at all.

Required sites, in order:

1. `frontend/src/domain/component.rs` — the `ComponentType` variant and the component struct.
2. `backend/src/validation.rs::KNOWN_COMPONENT_TYPES` — the type tag, or `POST /api/projects`
   answers 422 for any layout containing the new component. `known_component_types_match_the_frontend`
   reads the frontend enum in a test and fails when this list drifts.
3. `frontend/src/builder/canvas/renderer.rs` — the canvas rendering, and the type label match.
4. `frontend/src/builder/breadcrumb.rs` — the label.
5. The property editor (`frontend/src/builder/property_editors/`) and the tree view.
6. Every exporter: `export_service.rs` (Leptos, HTML, JSON, Markdown) and `export_advanced.rs`
   (JSON Schema, TypeScript, React, Vue, Svelte, Tailwind HTML). For an exporter that carries a
   component's visual settings, go through the shared `component_inline_css` /
   `style_js_properties` helpers in `export_service.rs` rather than inventing a second
   representation — a styled component must look the same exported as it does on the canvas.
   `frontend/src/domain/component.rs`'s `StyleTag`/variant matches and `duplicate_with_new_id` are
   the other two enumeration sites.
7. Screenshots / docs, if the component is user-visible.

A component's visual settings (`ComponentStyle` + `Animation`) are part of its semantics, not
decoration: if the canvas renderer applies them, every visual exporter must too.

**Exception — `ComponentStyle.custom_css`.** This field is reserved and *not* part of visual
parity: nothing writes it (the `StyleEditor` has no input for it), nothing reads it, and
`to_css_string` omits it. Both the canvas and every visual exporter therefore ignore it, which is
consistent — do not "fix" one without the other, and do not assume it holds CSS declarations or
class names. It survives losslessly through the JSON exporter and is declared in the TypeScript
output so existing data is not dropped; that is the whole of its supported surface. The
supported way to add classes is the `bindings["custom_css_classes"]` binding. Consequently the
parity claim above is "every `ComponentStyle` field the canvas applies", not "every field".
`frontend/src/services/export_advanced.rs`'s `custom_css_is_not_a_visual_setting` test pins the
ignored-in-visual-output / preserved-in-JSON behaviour, and
`export_service.rs`'s `test_every_applied_style_field_reaches_the_canvas_and_exporters` pins the
positive side — every field `to_css_string` applies reaches all six visual exporters.

## Modals and Escape

A modal's visibility signal is the single source of truth for three things and they must not
diverge: what renders the modal, what the `use_escape_key` listener is gated on, and what feeds
`KeyboardHandler`'s `modal_open`. Pass the caller's real `show` signal into a modal component
(`TemplateGallery`, `ExportModal`, `SaveTemplateModal` all take one); never construct a local
`RwSignal::new(true)` to hand to `use_escape_key`, because that listener stays armed after the
modal is hidden and will consume Escape for whatever modal is actually open. The gallery's
open/close/hidden Escape behaviour is pinned by the `wasm_tests` module in
`builder/template_gallery.rs`.

## Persistence and tests

- The backend store is `Store { projects, data_file }` in `backend/src/main.rs`. The data file is
  bundled with the data so tests can point a store at an isolated temp file; `router_for_store`
  mounts the real project routes against it, and the tests drive `save_project` / `get_project`
  through HTTP (`tower::ServiceExt::oneshot`) rather than calling helpers directly.
- Backend tests must never touch `backend/projects.json`. Anything that needs a store creates one
  over a temp file (`TestStore` in `backend/src/main.rs`).
- **A mutating request is one transaction.** `save_project` / `delete_project` must go through
  `Store::commit`, never mutate `store.projects` directly: `commit` holds `mutation_lock` for the
  whole mutate-then-write sequence, snapshots the state under the write lock, releases it *before*
  the filesystem `await` (so readers are not blocked by I/O), and only then writes. Because the
  transaction is exclusive, the write order matches the state-change order and a failed write rolls
  back to a snapshot no other transaction could have moved past. Do not "optimise" this by dropping
  the lock before the write or by cloning outside it — the `concurrent_saves_*` /
  `concurrent_save_and_delete_*` / `failed_persistence_*` tests fail when the lock is removed.
- **File writes are atomic.** `write_store_atomically` writes a unique temp file in the target
  directory, `flush` + `sync_all`, then `rename`s onto `data_file`, so a crash cannot leave
  truncated JSON. Keep the temp file in the same directory (rename is only atomic within a
  filesystem).
- **`component_count` has one definition.** `validation::count_components` (recursive) must be used
  by both `save_project`'s response and `list_projects`; the dashboard count is not
  `layout.as_array().len()`. `dashboard_component_count_is_recursive_and_matches_save` pins that
  save, list, and a reload-from-disk all agree.

## Documentation

- Root `README.md` embeds all 35 screenshots from `docs/screenshots/`. Keep the reference set and
  the directory in sync — a check that flags missing/unused images is trivial to re-run.
- Screenshots are captured with a Playwright harness (Chromium, 1400x875) and compressed before
  publishing. Capture must not mutate `backend/projects.json`.
