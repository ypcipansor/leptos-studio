# 1. OBJECTIVE

Mengembangkan Leptos Studio menjadi aplikasi yang benar-benar dapat dipakai: memperbaiki bug rendering/visual saat ini, memperkuat kualitas export kode, menambah set komponen baru, dan meningkatkan pengalaman canvas — sekaligus membersihkan fondasi kualitas (test, clippy, CI) agar stabil. Prioritas area: **A (komponen), B (canvas), C (export/codegen), F (kualitas/stabilitas)**.

# 2. CONTEXT SUMMARY

Struktur workspace:

- **Frontend (Leptos 0.8 / WASM)** — `frontend/src`: domain (`domain/component.rs` mendefinisikan 8 tipe: Button, Text, Input, Container, Image, Card, Select, Custom), state (`state/app_state.rs` berisi `CanvasState` dgn seleksi tunggal `Option<ComponentId>`), services export (`services/export_service.rs` + `export_advanced.rs`), UI builder (`builder/canvas/{mod,renderer}.rs`, `property_editors/`, dll). Tes: `frontend/tests/` (export, git, accessibility, wasm smoke).
- **Backend (Axum, file-JSON)** — `backend/src/{main,git,templates,analytics}.rs` persisting ke `projects.json`, `templates.json`, `git_data.json`, `analytics.json`.
- **Styling**: satu file monolitik `frontend/style.css` (~1900 baris) + token desain di `builder/design_tokens.rs`.

Temuan gap utama dari analisis codebase:

1. **Bug visual (F)**: renderer (`builder/canvas/renderer.rs`) mereferensikan puluhan kelas CSS (`canvas-button`, `btn-primary/secondary/outline/ghost`, `btn-sm/md/lg`, `canvas-input/select/image/text/container/card/custom`, `text-h1…h3`, `style-body/caption`, `flex-row/col/wrap`, `selected-label`, `empty-container-placeholder`, `custom-header/name/template`) yang **tidak terdefinisi sama sekali** di `style.css` — komponen di canvas tampil tanpa styling. Renderer juga memakai kelas gaya-Tailwind (`bg-white`, `border`, `min-h-[100px]`) padahal Tailwind tidak ada di build.
2. **Kualitas (F)**: skrip debug `check_braces.py` tertinggal di root; tidak ada CI; backend tanpa validasi input/test.
3. **Codegen (C)**: generator Leptos (`services/export_service.rs`) memakai `RefCell` untuk sinyal, menghasilkan `use leptos::*;` (seharusnya `use leptos::prelude::*;` untuk 0.8), dan mereferensikan kelas CSS yang tidak disertakan dalam hasil export → output belum "bisa dipakai langsung".
4. **Canvas (B)**: tidak ada zoom/pan, tidak ada multi-select (hanya `Option<ComponentId>`), tidak ada snapping/alignment.
5. **Komponen (A)**: set komponen masih minimal (8 tipe) — widget dasar seperti Divider, Checkbox, Radio, Switch, Badge, Progress, Avatar, Spacer belum ada.

Dependensi: Leptos 0.8 (`csr`), `leptos_router`, Axum + tower_http (backend), uuid/serde. Editor build via `trunk`.

# 3. APPROACH OVERVIEW

Rencana dibagi menjadi **5 fase berurutan**, masing-masing vertikal (end-to-end) agar main-branch selalu hijau dan setiap fase menghasilkan fitur yang bisa diverifikasi. Meskipun user memprioritaskan A→B→C, bug CSS pada renderer memblokir "tampak bagus" untuk *semua* komponen — jadi fase pertama adalah **perbaikan fondasi visual/kualitas (F)**, lalu barulah penambahan komponen (A), pengalaman canvas (B), penguatan codegen (C), dan pengerasan backend/CI (F lanjutan).

Prinsip yang dipakai sepanjang implementasi:

- **Vertical slice per fase**: untuk setiap komponen baru, ubah domain → renderer → palette → property editor → codegen → test dalam satu fase.
- **Tidak menambah fitur di luar lingkup**: hanya yang dibutuhkan untuk agar hasil "rapi, modern, bisa dipakai".
- **Backward compatible**: format proyek JSON tetap; `#[serde(default)]` untuk field baru.

Alternatif yang dipertimbangkan tetapi tidak dipilih: menambah Tailwind ke build (ditolak — choose to replace remnant classes dengan CSS biasa agar konsisten dengan style.css vanilla); refactor domain menjadi Dyn/flat component registry (ditolak — enum match tersebar tapi aman dan sederhana; refactor besar tidak perlu).

# 4. IMPLEMENTATION STEPS

## Fase 1 — Perbaikan fondasi visual & kualitas (F)

### 1.1 Lengkapi stylesheet renderer canvas
- **Goal**: semua kelas yang dipakai `renderer.rs`/canvas punya definisi CSS; hasil visual match pada preview komponen.
- **Method**: tambahkan blok CSS terorganisasi di `frontend/style.css` untuk `canvas-component`, `canvas-button/btn-*/canvas-input/canvas-select/canvas-image/canvas-text (+ tag- & style- helpers)`, `canvas-container` (`flex-row/col/wrap`), `canvas-card` (shadow/border opsi), `canvas-custom` (header/template), `selected-label`, `empty-container-placeholder`; mapping warna/ukuran mengikuti token desain di `:root`. Hapus kelas sisa-Tailwind di renderer (`bg-white`, `border`, `min-h-[100px]`, dst) → ganti dengan selector CSS khusus atau token CSS.
- **Reference**: `frontend/style.css`, `frontend/src/builder/canvas/renderer.rs`, `frontend/src/builder/canvas/empty_state.rs`.

### 1.2 Bersihkan artefak debug & pastikan workspace hijau
- **Goal**: repo bersih; `cargo test` (workspace) + `cargo clippy --all-targets` lolos tanpa warning besar.
- **Method**: hapus `check_braces.py`; jalankan seluruh test Rust (non-wasm) & perbaiki jika ada; perbaiki warning clippy; pastikan `wasm-bindgen-test` dikenanport (opsional). 
- **Reference**: seluruh crate `frontend`/`backend`.

## Fase 2 — Komponen baru (A)

Tambahkan set widget dasar yang aman & umum. Set komponen yang disetujui: **Divider, Checkbox, RadioGroup, Switch/Toggle, Badge, Progress** (6 tipe). Untuk setiap tipe:

### 2.1 Domain model
- **Goal**: tipe data baru valid & terserialisasi.
- **Method**: tambah `ComponentType` variant & struct (mis. `DividerComponent { orientation, thickness }`, `CheckboxComponent { label, checked, disabled }`, `RadioGroupComponent { options, selected, disabled }`, `SwitchComponent { label, checked }`, `BadgeComponent { text, variant }`, `ProgressComponent { value, max, show_label }`). Sertakan `Animation`, `ComponentStyle`, bindings map mengikuti pola yang ada; extends `duplicate_with_new_id` & `validate()`.
- **Reference**: `frontend/src/domain/component.rs`.

### 2.2 Renderer & Palette
- **Goal**: komponen baru bisa di-drag dari palette dan render dengan benar di canvas.
- **Method**: tambah `render_*` di `builder/canvas/renderer.rs`, daftarkan di `component_palette.rs`/`component_library.rs` (kind string + create), dan tambahkan kelas CSS-nya di `style.css` pada blok per-komponen (mengikuti 1.1).
- **Reference**: `renderer.rs`, `component_library_enhanced.rs`, `style.css`.

### 2.3 Property editor
- **Goal**: properti komponen baru dapat diedit melalui panel.
- **Method**: buat `property_editors/{divider,checkbox,radio,switch,badge,progress}.rs` mengikuti pola editor lain; daftarkan di `property_editors/mod.rs`; sesuaikan `property_service.rs` untuk update properti.
- **Reference**: `property_editors/`, `services/property_service.rs`.

### 2.4 Codegen & advanced export
- **Generator** tipe baru di `export_service.rs` + lama `export_advanced.rs` (JSON Schema); **test unit** baru per tipe.
- **Reference**: `services/export_service.rs`, `services/export_advanced.rs`, `frontend/tests/export_tests.rs`.

## Fase 3 — Pengalaman canvas (B)

### 3.1 Zoom & pan
- **Goal**: zoom 25%–400% lewat toolbar dan Ctrl+scroll; pan via drag papan saat zoom.
- **Method**: `CanvasState` tambahkan sinyal `zoom: RwSignal<f64>`; komponen toolbar tambahkan kontrol (nilai %, tombol +/-, reset 100%); wrapper canvas menerapkan `transform: scale(zoom)` dengan origin top-center; shortcuts di `builder/keyboard.rs`.
- **Reference**: `state/app_state.rs`, `builder/canvas/mod.rs`, `builder/toolbar.rs`.

### 3.2 Multi-select
- **Goal**: Shift/Ctrl+click memilih beberapa komponen; operasi dasar (delete/group?) minimal bisa delete & drag sbg grup? Scope minimal: seleksi berganda, penghapusan berganda, tampil count di status bar.
- **Method**: ubah `CanvasState.selected: RwSignal<Vec<ComponentId>>`; update renderer (is_selected via contains), context menu, property editor (kosong bila >1 atau enable style bersama), keyboard (Ctrl+A memilih semua), tree view.
- **Reference**: `state/app_state.rs`, `builder/canvas/renderer.rs`, `builder/property_editor.rs`, `builder/keyboard.rs`, `builder/tree_view.rs`.

### 3.3 Selection visual & detail UX
- **Goal**: label seleksi & outline mengikuti desain (tackle kelas `selected-label` yang kini kosong).
- **Method**: pastikan CSS dari 1.1 menghasilkan border+label halus di mode edit; transisi halus.
- **Reference**: `style.css`, `renderer.rs`.

## Fase 4 — Penguatan export/codegen (C)

### 4.1 Generator Leptos: import & sinyal tanpa RefCell
- **Goal**: hasil export Leptos 0.8-idiomatic & dapat compile.
- **Method**: ganti import jadi `use leptos::prelude::*;`; pisahkumpul sinyal `required_signals` sebagai data hasil (bukan `RefCell`); generate handler default yang log via `web_sys` hanya bila memang dipanggil; tambahkan unit test per tipe & per preset.
- **Reference**: `services/export_service.rs`, `frontend/tests/export_tests.rs`.

### 4.2 Bundel CSS & animasi pada export
- **Goal**: kode Leptos yang di-export berisi stylesheet untuk kelas yang digunakan (btn-*, flex-*, canvas-*, keyframes animasi) sehingga visual sama dengan editor.
- **Method**: generator kini mengembalikan struktur { code, css } atau embed `<style>{css}</style>` di komponen Root bila preset Plain; sertakan hanya kelas yang dipakai (semuanya di export bundle). Update modal export untuk memudahunduh satu file `.rs` yang mandiri.
- **Reference**: `services/export_service.rs`, `builder/export_modal.rs`, `builder/code_panel.rs`.

### 4.3 Verifikasi multi-format
- **Goal**: generator lain (HTML/TS/React/Vue/Svelte) masih lolos test dan tidak regresi.
- **Method**: tambah test ke `export_tests.rs` yang men-generate untuk komponen baru & memastikan output memiliki struktur kunci (button/nesting/class/css).
- **Reference**: `frontend/tests/export_tests.rs`.

## Fase 5 — Kualitas lanjutan & backend (F)

### 5.1 Validasi & ketahanan backend
- **Goal**: endpoint backend menolak input rusak, dan tidak crash saat file data kosong/rusak.
- **Method**: gunakan struct ter-serde yang tervalidasi,embalikan 400 pada malformat; tangani error IO dengan log & status 500; sertakan validasi minimal pada `templates`, `git`, `analytics` handlers.
- **Reference**: `backend/src/{main,templates,git,analytics}.rs`.

### 5.2 CI minimal
- **Goal**: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` di GitHub Actions untuk setiap PR.
- **Method**: tambahkan `.github/workflows/ci.yml` (atau platform repo) dengan cache Swatinem.
- **Reference**: file CI baru.

### 5.3 Dokumentasi rapi
- **Goal**: README dan DEVELOPMENT menyebut fitur baru (zoom, multi-select, komponen baru) & cara menjalankan bootstrap yang benar; hapus referensi ke Tailwind (tidak dipakai) dan sebut struktur CSS.
- **Reference**: `README.md`, `frontend/DEVELOPMENT.md`, `frontend/ARCHITECTURE.md`.

# 5. TESTING AND VALIDATION

Kondisi sukses (acceptance):

1. **Visual rendering**: buka editor, drag setiap komponen (lama + baru) ke canvas → semuanya menampilkan styling yang benar (button variant/size, flex container layout, badge/divider/checkbox/radio/switch/progress tampil normal); mode preview & responsive viewport tidak rusak. Validasi manual + screenshot checklist.
2. **Canvas UX**: zoom toolbar & Ctrl+scroll bekerja (25%–400%); Shift/Ctrl+click + Ctrl+A memilih berganda; delete multi berfungsi; seleksi berganda terhapus saat canvas di-click.
3. **Export**: untuk tiap preset, `LeptosCodeGenerator::generate()` menghasilkan `use leptos::prelude::*;` dan kelas CSS yang dipakai ikut disertakan (bila Plain embed style). Test unit meng-assert kehadiran import, struktur komponen, & CSS.
4. **Kualitas**: `cargo test` (workspace kedua crate) hijau; `cargo clippy --all-targets` tanpa warning; `cargo fmt --check` lolos; tidak ada artefak debug di root repo; backend menolak payload rusak dengan 4xx dan tidak crash.
5. **Regresi**: test existing (`export_tests`, `git_*`, `accessibility_test`, `wasm_smoke`) tetap hijau; CHANGELOG diperbarui dengan sem fitur/bug-fix pada fase ini.

# 6. STATUS EKSEKUSI (diperbarui)

Selesai pada sesi ini:

- **Fase 1 (visual/kualitas)**: stylesheet renderer dilengkapi; artefak `check_braces.py` dibersihkan; workspace hijau.
- **Fase 2 (komponen baru)**: enam komponen (Divider, Checkbox, RadioGroup, Switch, Badge, Progress) ditambahkan end-to-end (domain, renderer, palette, property editor, codegen semua format, CSS, tes). 102 unit test frontend + 2 export regression test.
- **Fase 3 (canvas)**: tombol preview menjadi split-button + dropdown aksesibel (ARIA menu, Escape/Arrow); double-click = context menu; right-click kanvas kosong = menu generik (Add Container); ARIA `application`/`listitem`. **Zoom 25%–400%** (kontrol toolbar, Ctrl+scroll, shortcut Ctrl+=/-/0, pan via drag background saat zoom). **Multi-select** (Shift/Ctrl+click toggle, Ctrl+A, Delete berganda, count di status bar, sinkron dengan undo/redo). 4 unit test baru (`canvas_state_test.rs`).
- **Fase 4 (export/codegen)**: `RefCell` dihapus dari generator Leptos (sinyal direpresentasikan via Vec lokal), import dipindah ke `leptos::prelude`, CSS baseline tertanam untuk preset Plain. Test struktur & regresi multi-generator ditambahkan.
- **Fase 5 (backend/kualitas/CI)**: `backend/src/validation.rs` menolak tipe komponen tidak dikenal (422) & menghitung nested children; 5 unit test backend. Workflow CI (fmt/clippy/test sebelumnya ada; dikonfirmasi lengkap). `CHANGELOG.md` baru dibuat.

Sisa pekerjaan (deferred): snapping/alignment — dicatat sebagai fitur di luar vertical slice sesi ini.
