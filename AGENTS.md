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
- `cd backend && cargo run` serves the built `dist/` and the API on `:3000`. Set
  `DATA_FILE=backend/projects.json` (CORS is permissive in dev).

## Conventions and gotchas

- `builder/sidebar.rs` is **dead code** — only declared in `builder/mod.rs`, never rendered. The
  live palette is `builder/component_palette.rs`, rendered from `pages/editor.rs`.
- The palette's category tabs must stay derived from each `LibraryComponent.category` string.
  Hardcoding component-kind lists per category previously made Media/Typography/Navigation
  components unreachable outside *All*. Badge counts use the same `ComponentCategory::matches`
  predicate as the row filter; keep them in sync (see
  `test_every_component_is_reachable_by_category`).
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
  `builtin_library_components()`; `Project` and `apply_project` do not carry them, so saved
  library entries do not survive a reload or a project reopen. The docs say so — do not claim
  otherwise without adding real serialization plus a round-trip test.
- Modal visibility gates the global shortcuts via `KeyboardHandler`'s `modal_open` prop, derived in
  `pages/editor.rs` from `editor_modal_open(...)`. Every argument must be the *same* signal that
  renders the modal: the Export modal is `app_state.ui.show_export_modal` (aliased as
  `show_export` in `EditorPage`), which is also what `use_export_actions` opens — there is no
  second local signal. Without the gate, Delete, `Ctrl+Z` and friends would edit the canvas hidden
  behind an open dialog. Any new modal must be added to that derivation and to the
  `every_editor_modal_gates_shortcuts` test.
- `backend/projects.json` is tracked runtime data, not a fixture. Screenshot capture must not
  mutate it; keep demo data out of it (use a separate fixture or document the manual step).
- Use `history_rw.get_untracked()` inside async handlers to avoid reactive-cycle panics.
- `cargo test --workspace` also runs integration tests that hit the network/backend; prefer
  `--lib --bins` for a fast local loop.

## Documentation

- Root `README.md` embeds all 35 screenshots from `docs/screenshots/`. Keep the reference set and
  the directory in sync — a check that flags missing/unused images is trivial to re-run.
- Screenshots are captured with a Playwright harness (Chromium, 1400x875) and compressed before
  publishing. Capture must not mutate `backend/projects.json`.
