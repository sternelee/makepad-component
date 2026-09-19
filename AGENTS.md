# Repository Guidelines for AI Agents

This repository is a Rust workspace focused on Makepad widgets, the A2UI (Agent-to-UI) protocol renderer, and various demo applications. It is read by AI coding agents — start here, then consult `README.md` (user-facing docs, A2UI protocol details) and `CLAUDE.md` (Claude Code-oriented supplement) as needed.

## 1. Project Overview

`makepad-component` is a UI component library for the [Makepad](https://github.com/makepad/makepad) framework (GPU-accelerated, SDF shader-based, cross-platform Rust UI), plus:

- A full **A2UI protocol renderer** that lets AI agents generate native UIs from declarative JSON (messages: `beginRendering`, `surfaceUpdate`, `dataModelUpdate`, `deleteSurface`, `userAction`).
- A **charting library** (`makepad-plot`) with ~25 chart types including interactive 3D (surface, scatter, line).
- Demo apps: widget showcase, A2UI demo, LLM bridge server, Raycast-style launcher, Gemini Live voice companion, cross-platform clipboard utilities.

License: MIT OR Apache-2.0. Rust edition 2021.

## 2. Workspace Structure

```
makepad-component/
├── crates/
│   ├── ui/                      # Core library (crate name: makepad-component)
│   │   └── src/
│   │       ├── lib.rs           # Re-exports: a2ui, theme, widgets (+ makepad_plot, makepad_widgets)
│   │       ├── widgets/         # 26 native Makepad widgets (MpButton, MpSlider, ...)
│   │       ├── a2ui/            # A2UI protocol renderer
│   │       │   ├── message.rs   # Protocol types (serde JSON, ComponentType enum)
│   │       │   ├── processor.rs # Message → widget tree conversion
│   │       │   ├── host.rs      # SSE client for A2A servers
│   │       │   ├── a2a_client.rs# A2A protocol client (JSON-RPC + SSE)
│   │       │   ├── sse.rs       # SSE streaming client
│   │       │   ├── data_model.rs, registry.rs, value.rs
│   │       │   ├── chart_bridge/    # ChartComponent → makepad-plot bridge (basic/advanced/statistical)
│   │       │   └── surface/         # Component tree rendering (widget, render_impl, events_impl,
│   │       │                        #   render_charts_impl, render_calendar_impl, render_shader_stage_impl)
│   │       └── theme/           # Color palettes (colors.rs)
│   ├── makepad-plot/            # Charting library (plot/ has per-type modules: bar, line, pie,
│   │                            #   scatter3d, surface3d, heatmap, treemap, contour, ...)
│   ├── component-zoo/           # Widget showcase demo (src/app.rs)
│   ├── a2ui-demo/               # A2UI demo app + bridge servers (multiple binaries, see §4)
│   │   └── src/
│   │       ├── main.rs, app/    # GUI app (logic, theme, sample_data, audio_player)
│   │       ├── a2ui_bridge.rs + a2ui_bridge_impl/  # LLM → A2UI bridge (server, builder, tools, types, mureka)
│   │       ├── watch_server.rs  # File-watching SSE server (port 8080)
│   │       ├── mock_server.rs, streaming_main.rs, math_charts.rs, fft_demo.rs
│   ├── canvas-terminal/         # Infinite-canvas terminal workspace (CNVS-style): PTY via a
│   │                            #   bundled daemon mode (`canvas-terminal --daemon`, rmux-pty +
│   │                            #   rmux-ipc, no system-installed rmux needed), CEF browsers,
│   │                            #   hand-drawn whiteboard/notes, agent cards. Single binary,
│   │                            #   dual mode. Progress: docs/AGENT_WORKBENCH_PROGRESS_CN.md
│   │                            # Agent cards are chat views over a session's PTY: the daemon
│   │                            #   parses the hosted CLI's JSONL (pi/claude/codex) into
│   │                            #   normalized chat events (src/chat/).
│   ├── raycast-launcher/        # Raycast-style launcher, Makepad 2.0 `script_mod!` API
│   │                            #   (runtime Splash app loading from *-app.json descriptors)
│   ├── gemini-talker/           # Gemini Live voice companion, Makepad 2.0 `script_mod!` API
│   ├── dbpro/                   # TablePro-style database GUI (SQLite/MySQL/PostgreSQL),
│   │                            #   Makepad 2.0 `script_mod!` API. Driver layer + custom
│   │                            #   DbTabBar widget. See crates/dbpro/README.md
│   └── makepad-clipboard/       # Cross-platform clipboard (per-OS modules: macos/windows/linux/ios)
├── docs/                        # A2UI guides (EN/CN), bridge docs, splash pitch
├── skills/                      # Claude Code skills: makepad-screenshot, xor-shader-techniques
├── serve_wasm.py                # HTTP server with COOP/COEP headers for the wasm build
└── ui_live.json, chart_test.json, math_test.json, ...  # Sample A2UI JSON payloads
```

Crate-specific guidance exists in `crates/raycast-launcher/CLAUDE.md` and `crates/gemini-talker/CLAUDE.md` — read those before working in those crates. Note: parts of `crates/raycast-launcher/CLAUDE.md` are stale (it references `todo.rs`/`todo-app.json`; the current files are `app-todo.json`, `app-DarkCalc.json`, `app-weather.json` and there is no `todo.rs`).

## 3. Dependencies & Toolchain

- **Rust stable** (edition 2021), resolver "2".
- `makepad-widgets` (`widgets/`) / `makepad-script` (`platform/script/`) / `makepad-cef` (`libs/cef/`) are **path dependencies on a local makepad clone** at `/Users/sternelee/www/github/makepad` (branch `dev`), declared in the root `Cargo.toml` (`[workspace.dependencies]`) and `crates/canvas-terminal/Cargo.toml`. They track whatever is checked out in that clone — there is no rev pin, so a `git checkout` / `git pull` in the makepad clone changes what this workspace builds against. The clone on 2026-09-15 was 39 commits ahead of `14fe611e` (the previously pinned 2026-09 dev HEAD); the newer tree changed the script-resource API to key on `(heap_key, handle)` (`ScriptHandleRef::heap_key()` + `Cx::get_resource` / `Cx::load_script_resource`), which `crates/ui/src/a2ui/surface/widget.rs` was adapted to. To return to the previously verified upstream pin, point these deps back at `git = "https://github.com/makepad/makepad", rev = "14fe611e66c7defda5cef84cc1bbcae2eee29b2f"` — canvas-terminal's dropped-PDF preview needs a build at or after that commit: older pins ship a `PdfView` with an invisible page-paper shader, a wrong `cm` matrix order and broken text decoding.
- Two API generations used to coexist in this workspace; the Makepad 2.0 migration (July 2026) moved every crate to `script_mod!`. The App entry pattern is: `impl AppMain for App { fn script_mod(vm) -> ScriptValue { ...; self::script_mod(vm) } }` with the `ui: Root{...}` tree inside a `startup() do #(App::script_component(vm)){...}` block as the last expression of the `script_mod!` block.

### ⚠️ Current build status (last fully verified 2026-09-15, agent workbench lands)

The workspace has been migrated to the Makepad 2.0 `script_mod!` API. Everything compiles against the currently locked makepad commit except `gemini-talker`:

- `cargo check -p makepad-component` / `-p makepad-plot` / `-p component-zoo` / `-p a2ui-demo` / `-p makepad-clipboard` / `-p raycast-launcher` — **pass**. The legacy `live_design!` API is gone from these crates; all widget/shader registration now happens through `script_mod!` blocks wired into each crate's `script_mod(vm)` function.
- `cargo check -p canvas-terminal` — **pass** (`--tests` too).
- `cargo test -p canvas-terminal` — **39 passed** (chat adapter/fold units with real probed pi JSONL + daemon chat e2e over real sockets: parse/replay, switch scripts, parser pause).
- `cargo test -p makepad-component` — **pass** (28 unit tests, 4 doc-tests ignored).
- `cargo check -p dbpro` — **pass** (with `PATH="/Library/Developer/CommandLineTools/usr/bin:$PATH"` so the bundled sqlite3 cc step uses CLT clang). `cargo test -p dbpro` — **5 passed** (SQLite round trip incl. write-back UPDATE/NULL/DELETE + FK error, INSERT-copy, SQL builders, config persistence). Runtime verified clean: `grep -c '\[E\]'` on the app log is 0.
- `cargo check -p gemini-talker` — **fails** (pre-existing: unresolved `gemini_live::prelude` import). Unrelated to the script_mod migration.
- ⚠️ Test runs require an accepted Xcode license; after a macOS/Xcode update the linker refuses (`cc` exit 69) until `sudo xcodebuild -license accept`. This also breaks `/usr/bin/git` (it is an Xcode shim) — use
  `/Library/Developer/CommandLineTools/usr/bin/git` as a stopgap.

### Interaction alignment (bezel port, 2026-08-25)

Interactive widgets (`MpButton`, `MpCheckbox`, `MpToggle`, `MpSwitch`, `MpRadio`, `MpSlider`, `MpSelectTrigger`) now claim keyboard focus on click (`cx.set_key_focus`), sync their focus ring from Cx (`cx.has_key_focus`), and respond to keyboard: Enter/Space activates buttons/checkboxes/toggles/switches/radios/selects, and arrow keys step the slider value. The focus ring uses the `CARET` token at 2px, matching the bezel focus spec. Motion timing standardized to bezel `HOVER_FADE` (150ms both directions).

**Tab / Shift-Tab traversal (`crates/ui/src/widgets/focus.rs`):** makepad has a single key-focus `Area` on `Cx` and no traversal, so this module adds one. Focusable controls call `focus::register(cx, uid, area)` from `draw_walk` (paint order = tab order); the registry is a `Cx` global keyed by the widget's stable `WidgetUid`, so each control occupies one slot and re-registers its fresh `Area` each frame. The app root calls `focus::handle_key(cx, event)` on Tab/Shift-Tab. Wire it into a new app's `AppMain::handle_event` (see component-zoo `App` and a2ui-demo `App`).

**Shader gotcha:** theme tokens (e.g. `CARET`, `SOLID`) cannot be referenced as bare identifiers inside a shader `pixel: fn()` body — bind them as an instance field (`focus_color: instance(CARET)`) and read via `self.focus_color`. The `script_mod!` macro does NOT catch this; it only fails at runtime when the shader first compiles. Always verify with `grep -c '\[E\]'` on the app's runtime log.

**Inheritance gotcha:** the old `live_design!` `<Base>{...}` angle-bracket inheritance is NOT valid in `script_mod!`. Use `mod.widgets.Variant = mod.widgets.Base{...}`. The just-landed `MpControlBar` variants used the old syntax and were fixed (parse errors only surfaced at runtime in a2ui-demo).

**Script-derive gotchas (hit while building dbpro, 2026-09-15):** the `#[derive(Script)]` field parser is token-based — (1) doc comments (`///`) on struct fields fail with "Unexpected field form"; (2) commas inside generic field types (e.g. `HashMap<u64, Arc<Mutex<T>>>`) break parsing — use type aliases; (3) in the DSL, 6-digit hex colors (`#xRRGGBB`) evaluate to objects at runtime ("type mismatch ... expected Vec4f, got object") — always write 8-digit `#xRRGGBBAA`; (4) don't wrap module/global color values in `instance(...)`/`uniform(...)` when overriding View/DrawText props — assign them plain (`color: mod.db_theme.panel`), matching the mpc widget library convention; (5) `WidgetRef` path accessors (`mp_table`, `mp_tree`, `text_input`, …) live in generated `*WidgetRefExt` traits — `use makepad_component::widgets::*;` brings them in; (6) `MpTextArea` is **display-only** (paints text, no editing, no `Returned` action) — for a real editor use makepad's `TextInput{is_multiline: true}`; with multiline, ⌘/Ctrl+Enter emits `TextInputAction::Returned` instead of inserting a newline (handle it via `TextInputRef::returned(actions)`); (7) `match`/`if let` on `&self.tabs[...]` while calling `&mut self` methods inside the arms is a borrow error — snapshot the needed data (clone) or use `matches!` first; (8) makepad's `DataGrid` widget hosts per-cell widgets via DSL **templates**: any object-valued key in a `DataGrid{...}` block (e.g. `Editor := TextInput{...}`) becomes a template; the app drives cells in its own draw via `while let Some(step) = view.draw_walk(...).step()` → `step.as_data_grid()` → `next_cell` + `cell_text_styled`/`item(row,col,live_id!(Editor))` + `draw_item`, and reads edits back through `cell_widgets_with_actions` (see dbpro/src/grid.rs and makepad's apps/mpsheets for the reference pattern).

Before assuming a change broke something, check whether the failure pre-exists. When fixing builds, prefer pinning/updating the dependency deliberately over speculative edits, and record what you did.

### ⚠️ CEF (canvas-terminal) runtime: helper architecture mismatch

`canvas-terminal` embeds Chromium via `makepad-cef` on macOS. Its `build.rs`
compiles `helper_main_macos.c` with plain `clang` (**no `-arch` flag**), so the
helper binary inherits whatever the `clang` in `PATH` defaults to. If an
Android NDK x86_64 `clang` (e.g. `~/Library/Android/sdk/ndk/*/toolchains/
llvm/prebuilt/darwin-x86_64/bin`) appears before `/usr/bin` in `PATH`, the
helper is built `x86_64` while the app/framework are `arm64`, and the app dies
at startup with:

```text
makepad-cef-helper: dlopen ... Chromium Embedded Framework failed:
incompatible architecture (have 'arm64', need 'x86_64')
```

**Fix:** rebuild with Apple `clang` first in `PATH`, after clearing the stale
helper outputs and the runtime bundle cache:

```bash
rm -rf target/debug/build/makepad-cef-* "$(getconf DARWIN_USER_TEMP_DIR)makepad-cef"
PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH" cargo +stable build -p canvas-terminal
```

Verify: `file target/debug/build/makepad-cef-*/out/makepad-cef-helper` must
report `arm64`. (`/usr/bin/clang` → arm64; NDK clang → x86_64.)

### ⚠️ Killing a canvas-terminal GUI (and why one instance at a time)

The GUI re-execs itself out of the makepad-cef runtime bundle, so a running
instance's command line is
`.../canvas-terminal.app/Contents/MacOS/canvas-terminal --remote=<port>` —
`pkill -f "canvas-terminal$"` never matches it. Kill test instances with
`pkill -f "MacOS/canvas-terminal"` (or `/quit` through the remote surface, which
saves first), and keep at most **one** GUI alive: every instance writes the same
`~/Library/Application Support/canvas-terminal/workspace.json`, so a stale window
holding a stale canvas silently clobbers the live one on its next save. The
`--daemon` process is a different thing — it hosts the PTYs that survive GUI
restarts, so leave it running.

## 4. Build, Test, and Lint Commands

All commands run from the repository root.

```bash
# Build & check
cargo build --workspace                     # Full workspace build
cargo check -p makepad-component            # Core library (currently broken, see §3)
cargo check -p raycast-launcher             # Known-good crate

# Run demos
cargo run -p component-zoo --bin component-zoo   # Widget showcase
cargo run -p a2ui-demo --bin a2ui-demo           # A2UI demo app
cargo run -p raycast-launcher                    # Raycast-style launcher
cargo run -p gemini-talker                       # Gemini Live app (needs GEMINI_API_KEY)
cargo run -p dbpro                               # Database GUI (auto-creates dbpro-demo.db on first run)

# a2ui-demo auxiliary binaries
cargo run --bin a2ui-streaming
cargo run --bin math-charts                     # Generates math_test.json / ui_live.json
cargo run --bin fft-demo
cargo run --bin watch-server --features mock-server      # Port 8080, watches ui_live.json
cargo run --bin mock-a2a-server --features mock-server
cargo build --bin a2ui-bridge --features a2ui-bridge     # LLM bridge server (see §7)

# Testing
cargo test --workspace
cargo test -p makepad-component
cargo test -p makepad-component -- test_name_substring        # Substring match
cargo test -p makepad-component -- test_name --exact          # Exact match

# Lint & format
cargo clippy --workspace -- -D warnings
cargo fmt --all
cargo fmt --all -- --check
```

### Feature flags (`a2ui-demo` crate)

| Feature | Enables |
| --------- | --------- |
| `mock-server` | `watch-server`, `mock-a2a-server` binaries (tokio/hyper) |
| `a2ui-bridge` | LLM bridge server (adds reqwest, futures-util) |
| `mureka` | AI music generation (extends `a2ui-bridge`, requires `MUREKA_API_KEY`) |

### WebAssembly build

```bash
cargo install --force --git https://github.com/makepad/makepad.git --branch rik cargo-makepad
cargo makepad wasm install-toolchain
cargo makepad wasm build -p gallery --release     # the v3 gallery, not component-zoo
python3 serve_wasm.py 8080 [app]                  # COOP/COEP headers required by Makepad wasm

### wasm-readiness rules (browser preview)

- **`std::time::Instant` and `std::time::SystemTime` panic on `wasm32-unknown-unknown`** —
  the std implementation is `unsupported` and it fails **at the call, not at the build**,
  so a green build says nothing. Neither appears in the library core; anything that needs
  a clock must be target-gated.
- **No `std::fs`, `std::process`, `std::net`, or blocking `std::thread`.** The only
  blocking threads are inside the `net` feature.
- **`crates/ui`'s socket is a feature.** `ureq` (TLS) and `uuid`'s `v4` (`getrandom`) do
  not build for wasm, and only `a2ui::{a2a_client, host, sse}` use them, so they sit
  behind `net` (on by default). The deps are declared
  `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`, so a wasm build drops
  them on its own. A `default-features = false` target-scoped override in the
  consumer does **not** work: Cargo unions features across target-scoped and plain
  deps, so `net` stays on and `getrandom` stays in the tree.
- **wasm-bindgen cannot be used with makepad's web platform.** The bridge
  (`makepad/libs/wasm_bridge/src/wasm_bridge.js`) instantiates the module with its own
  `{ env }` import object, so `web-sys`/`js-sys` `__wbindgen_placeholder__` imports are
  never satisfied and every page dies at startup
  (`Import #13 "__wbindgen_placeholder__": module is not an object or function`,
  reported via `POST /api/crash`). Page facts (`?page=`, the timezone) go through two
  bridge-provided env imports instead — `js_query_param` and
  `js_local_timezone_offset` — declared in `crates/gallery/src/app.rs` with
  `#[link(wasm_import_module = "env")]`. Keep the wasm tree free of wasm-bindgen:
  `cargo tree --target wasm32-unknown-unknown -p gallery -i wasm-bindgen` must print
  nothing.
- **`std::env::var` returns `Err` on the web rather than failing**, so the `GALLERY_*`
  knobs are inert in a browser, not broken. The page is named with `?page=<title|index>`
  there — see `page_from_url` in `crates/gallery/src/app.rs`.
- **`cargo makepad wasm build` generates `index.html`** (with the crash reporter and the
  early error hooks), so there is no hand-written host page to keep in step. It writes
  `target/makepad-wasm-app/<profile>/<app>/` — the wasm is content-hashed
  (`gallery.<hash>.wasm`), which `serve_wasm.py` globs for. Its crash reports are
  `POST /api/crash`; the server decodes and prints them, because a dropped report is how
  a blank "Loading.." page stays unexplainable.
- **`--no-threads` for embedded previews.** The default build needs shared memory
  (`--shared-memory` + atomics), which requires `crossOriginIsolated` — true in a normal
  browser served with COOP/COEP, but false in an embedded webview (ZCode's in-app
  browser), where every worker spawn dies with
  `SharedArrayBuffer transfer requires self.crossOriginIsolated`.
  `cargo makepad wasm build -p gallery --release --no-threads` runs the pool inline and
  works everywhere.
```

## 5. Coding Style & Conventions

### Naming

| Type | Convention | Example |
| ------ | ------------ | --------- |
| Variables, functions, modules | `snake_case` | `draw_bg`, `handle_event` |
| Structs, enums, traits | `UpperCamelCase` | `MpButton`, `MpButtonAction` |
| Constants, statics | `SCREAMING_SNAKE_CASE` | `MAX_WIDTH` |
| Public widget types | `Mp` prefix | `MpButton`, `MpSlider`, `MpCheckbox` |
| Internal fields | `snake_case` | `draw_bg`, `animator` |

### Import order

```rust
use makepad_widgets::*;              // 1. makepad-widgets glob
use std::collections::HashMap;       // 2. std
use serde::{Deserialize, Serialize}; // 3. external crates
use crate::a2ui::message::*;         // 4. crate::
use super::data_model::*;            // 5. super::
```

### Public API pattern (`lib.rs` / `mod.rs`)

```rust
pub mod a2ui;
pub mod theme;
pub mod widgets;

// widgets/mod.rs: `pub mod button;` + `pub use button::*;` per widget,
// plus one `pub fn live_design(cx: &mut Cx)` registering every widget.
```

### Widget implementation pattern (legacy `live_design!` API)

- DSL: base widget `pub MpButton = {{MpButton}} { ... }` with `draw_bg` shader instances, `animator` states (`hover.on/off`, `pressed.on/off`); variants via inheritance: `pub MpButtonPrimary = <MpButton> { ... }`.
- Struct: `#[derive(Live, LiveHook, Widget)]` with `#[live]`/`#[walk]`/`#[layout]`/`#[animator]`/`#[rust]` field attributes; `#[redraw]` on fields that must trigger redraws.
- `impl Widget`: `handle_event` (animator first, then `event.hits(cx, self.area)` match on `Hit::FingerHoverIn/Down/Up`, emit `cx.widget_action(uid, &scope.path, MyAction::...)`) and `draw_walk` (`draw_bg.begin/end`, capture `self.area`, return `DrawStep::done()`).
- Actions: `#[derive(Clone, Debug, DefaultNone)]` enum ending in `None`; add `clicked(&self, actions: &Actions) -> bool` helpers on both the widget and its `*Ref` type.
- Keep DSL IDs stable and descriptive (`ids!(hover.on)`, `save_button`, `user_input`).

### Error handling

- `Result`/`Option` + `?` for expected failures; `expect()` only for impossible states in library code; `log!` (from `makepad_widgets`) for GUI-path runtime errors.

## 6. A2UI Architecture

### Component pipeline

```
JSON message → A2uiMessageProcessor (processor.rs) → ProcessorEvent
             → A2uiSurface (surface/) → Makepad widgets / makepad-plot charts
```

`message.rs` defines the serde types; `processor.rs` turns messages into events; `surface/` renders. `host.rs`/`sse.rs`/`a2a_client.rs` handle server connections.

### `ComponentType` variants (`a2ui/message.rs`)

Layout: `Column`, `Row`, `List`, `Card` · Display: `Text`, `Image`, `Icon`, `Divider` · Interactive: `Button`, `TextField`, `CheckBox`, `Slider`, `MultipleChoice` · Containers: `Modal`, `Tabs` · Visualization: `Chart`, `Calendar`, `AudioPlayer`, `ShaderStage` · Raycast-style: `Detail`, `Form`, `ActionPanel`, `Grid`, `PasswordField`, `TextArea`, `DatePicker`, `Dropdown`, `TagPicker`, `FilePicker`, `ListItem`, `DropdownItem`, `DropdownSection`, `TagPickerItem`.

Children are an explicit ID list or a `{template: {componentId, dataBinding}}` reference (flat adjacency list — components reference each other by ID).

### Adding a new A2UI component

1. Add a variant to `ComponentType` in `a2ui/message.rs` (serde struct for its props).
2. Handle it in `a2ui/processor.rs`.
3. Add a render implementation in `a2ui/surface/` (new `render_*_impl.rs` if it's a new family).
4. Register it in `ComponentRegistry` (`a2ui/registry.rs`).
5. Verify with `cargo run -p a2ui-demo` and/or a JSON sample file.

New chart types go in `makepad-plot/src/plot/` and are bridged in `a2ui/chart_bridge/`.

## 7. Servers, Ports, and LLM Configuration

| Server | Default port | Feature | Purpose |
| -------- | ------------- | --------- | --------- |
| A2UI Bridge | 8082 (`LLM_PORT`) | `a2ui-bridge` | LLM chat → A2UI JSON |
| Watch Server | 8080 | `mock-server` | File watcher → SSE stream |
| Mock A2A Server | 8080 | `mock-server` | Static A2A responses |

The a2ui-demo app connects to `localhost:8082` by default; the watch-server port requires editing the URL in `crates/a2ui-demo/src/app/`.

Bridge env vars: `LLM_API_URL` (default `https://api.moonshot.ai/v1/chat/completions`), `LLM_MODEL` (default `kimi-k2.5`), `LLM_API_KEY` (also reads `MOONSHOT_API_KEY`), `LLM_PORT`. The LLM must support tool/function calling; the bridge exposes ~11 tools (`create_text`, `create_button`, `create_chart`, `set_data`, `render_ui`, ...) and assembles valid A2UI JSON from the tool calls. Bridge endpoints: `POST /chat`, `POST /rpc`, `GET /live` (SSE), `POST /reset`, `GET /status`, `POST /inject`.

## 8. Testing Guidelines

- Unit tests live in `#[cfg(test)] mod tests` at the bottom of the implementation file. Current coverage is concentrated in `crates/ui/src/a2ui/` (`message.rs`, `processor.rs`, `sse.rs`, `registry.rs`, `value.rs`, `data_model.rs`) plus one test in `gemini-talker/src/memory.rs`.
- Use descriptive names: `test_should_fail_on_invalid_json`.
- Prioritize `Processor` and `DataModel` correctness (A2UI protocol behavior).
- UI changes are verified visually: `cargo run -p component-zoo` (widgets) or `cargo run -p a2ui-demo` (A2UI). The `makepad-screenshot` skill in `skills/` automates GUI screenshots.
- `canvas-terminal`'s chat path carries the strongest coverage in the workspace (adapter/fold units with real probed pi JSONL + daemon end-to-end over real sockets); keep new contracts under test there. `dbpro` covers its driver layer (SQLite round trip, paging/search/sort, persistence). Other crates have little or no test coverage; treat `cargo check -p <crate>` as the smoke test there.
- No CI is configured (no `.github/` workflows) — run tests and clippy locally before submitting.

## 9. Security Considerations

- API keys are passed only via environment variables (`LLM_API_KEY`, `MOONSHOT_API_KEY`, `MUREKA_API_KEY`, `GEMINI_API_KEY`) — never hardcode them or commit files containing them.
- A2UI is declarative JSON with no code execution by design, which makes it safe across trust boundaries; keep it that way when extending the protocol (the exception is `raycast-launcher`, which evaluates Splash code from local `*-app.json` descriptors in the Makepad script VM — only load descriptors from trusted local sources).
- `raycast-launcher` writes local state files in its crate directory (`.chat-history.json`, updated app descriptors); `gemini-talker` persists data under the platform data dir. Don't delete these blindly — they may be user data.
- `makepad-clipboard` contains per-OS unsafe FFI glue (objc2, win32, GTK, JNI); review platform-specific changes carefully and prefer the existing per-OS module structure.

## 10. Commit Guidelines

Conventional-commit style, e.g.:

```
feat: add MpSlider with range mode support
fix: correct tooltip positioning at screen edges
docs: add A2UI protocol documentation
refactor: extract chart_bridge module
test: add processor surface update tests
chore: update makepad-widgets dependency
```

PRs should include a summary plus `cargo test` / `cargo clippy` evidence, and screenshots from `component-zoo` for visual changes. Given the current build state (§3), state explicitly which crates you verified compile.
