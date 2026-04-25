# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

This crate is a **Makepad 2.0 `script_mod!` app**, not a standard `live_design!` widget crate. It implements a Raycast-style launcher with three runtime modes:

- launcher/search UI for apps and commands
- a dynamically loaded Splash app surface (currently used by the Todo app)
- a chat panel that can ask an OpenAI-compatible LLM to generate Splash apps and save them as JSON descriptors

The workspace root already has a broader `CLAUDE.md`; this file is the crate-specific supplement for `crates/raycast-launcher`.

## Common Commands

Run from the workspace root unless noted otherwise.

```bash
# Fast compile check for this crate
cargo check -p raycast-launcher

# Build this crate
cargo build -p raycast-launcher

# Run the launcher
cargo run -p raycast-launcher

# Run tests for this crate
cargo test -p raycast-launcher

# Run a single test by substring
cargo test -p raycast-launcher -- test_name_substring

# Lint this crate strictly
cargo clippy -p raycast-launcher -- -D warnings

# Format the workspace
cargo fmt --all
```

Notes:

- `raycast-launcher` is a workspace member in `Cargo.toml`, so `-p raycast-launcher` is the safest way to target it.
- This crate currently has little or no direct test coverage, so `cargo test -p raycast-launcher` is often just a compile/integration smoke check.

## High-Level Architecture

### Main app shell: `src/main.rs`

`src/main.rs` is the center of the crate. It contains:

- the top-level `script_mod!` UI definitions
- widget registration for `TodoList`, `ChatList`, and `LauncherPanel`
- `LauncherPanel`, which owns most app state and event handling
- launcher item discovery, filtering, selection, and launching
- mode switching between launcher, todo/splash surface, and chat
- runtime loading of JSON-defined Splash apps into the main content area

Important reference points:

- `src/main.rs:17` defines the script UI and widget registration
- `src/main.rs:758` defines `LauncherPanel`
- `src/main.rs:808` initializes launcher state in `on_after_new`
- `src/main.rs:1058` loads launcher items from applications, built-ins, and local Splash app descriptors
- `src/main.rs:1250` handles search filtering and built-in result prioritization
- `src/main.rs:1434` launches the selected item
- `src/main.rs:1495` loads a JSON-defined Splash app into the todo/splash surface

### Runtime Splash app loading: `src/app_loader.rs`

This file is the bridge between JSON descriptors and the Makepad script VM.

It is responsible for:

- parsing app descriptor JSON (`AppDescriptor`)
- wrapping raw Splash code so it can be evaluated with `vm.eval()`
- converting descriptor `state` JSON into `ScriptValue`
- injecting state into `mod.state.app`
- reading VM state back out to JSON for persistence

Key references:

- `src/app_loader.rs:21` `AppDescriptor`
- `src/app_loader.rs:84` injects descriptor state into `mod.state.app`
- `src/app_loader.rs:168` reads current app state back from the VM
- `src/app_loader.rs:179` persists updated state back into the descriptor JSON

### Built-in Todo path: `src/todo.rs`

The current Todo implementation is a **hybrid**:

- layout/templates come from `todo-app.json` Splash code
- list rendering and interaction still rely on a custom Rust widget (`TodoList`)
- todo state is read from and written back into VM state, then persisted to the JSON descriptor file

That means the current Todo app is not yet fully JSON/Splash-defined end to end.

Key references:

- `src/todo.rs:54` reads todos from VM state
- `src/todo.rs:96` writes todos back into VM state
- `src/todo.rs:149` saves todo state back to disk
- `src/todo.rs:311` defines the custom `TodoList` widget

### Chat-generated app flow: `src/chat.rs` + `src/a2ui_bridge_embed.rs`

The chat mode is an app generator and runtime preview flow.

`src/chat.rs` handles:

- persisted chat history (`.chat-history.json`)
- chat mode UI synchronization
- extracting `**App Name:** ...`
- extracting ```runsplash fenced blocks
- saving generated app descriptors to JSON
- reopening the newly saved app from chat state

`src/a2ui_bridge_embed.rs` handles:

- building an OpenAI-compatible chat-completions request
- deciding when to inject Splash generation guidance
- parsing the LLM response body

Key references:

- `src/chat.rs:18` chat history file path
- `src/chat.rs:91` extracts `runsplash` blocks
- `src/chat.rs:189` saves a generated app descriptor
- `src/chat.rs:269` sends the LLM request
- `src/a2ui_bridge_embed.rs:21` UI-generation keyword detection
- `src/a2ui_bridge_embed.rs:33` embedded Splash generation reference
- `src/a2ui_bridge_embed.rs:100` request-body builder
- `src/a2ui_bridge_embed.rs:132` response parsing

### Chat list rendering: `src/chat_list.rs`

`ChatList` is a small custom widget that reads shared `CHAT_DATA` and renders messages through a `PortalList` with separate templates for user and assistant messages.

Reference:

- `src/chat_list.rs:5`

## Runtime Mental Model

There are three important layers:

1. **Descriptor file** — JSON file such as `todo-app.json` containing:

   - `app` metadata
   - `splash_code`
   - `state`

2. **Script VM state** — app state is injected into the Makepad VM under `mod.state.app`

3. **Rendered UI** — `LauncherPanel` swaps the visible surface between launcher, chat, and the Splash/todo view

For dynamic apps, the typical flow is:

- discover `*-app.json` / `*_app.json`
- load descriptor JSON
- eval `splash_code`
- inject `state` into `mod.state.app`
- render/update UI
- optionally persist state back into the descriptor JSON

## Important Constraints and Gotchas

### `mod.state` lives on `heap.modules`

When working with Makepad script VM state here, do not assume `vm.module(id!(mod))` contains the runtime state. This crate explicitly reads and writes `mod.state` through `heap.modules`.

Reference:

- `src/app_loader.rs:88`

### Runtime Splash eval expects inline templates

For runtime-generated Splash code, inline templates work reliably, but `mod.widgets.*` references do not. The generated/evaluated code should return inline templates and the final UI tree directly.

References:

- `src/app_loader.rs:6`
- `src/a2ui_bridge_embed.rs:73`

### Saved chat apps and launcher discovery are currently inconsistent

Chat-generated apps are saved as `"<name>.json"`, but launcher discovery only scans `*-app.json` and `*_app.json`.

References:

- `src/chat.rs:239`
- `src/main.rs:1033`

If a saved chat app does not appear in launcher search after refresh, check the filename pattern first.

### The Todo app is still partly Rust-driven

Even though the UI is loaded from `todo-app.json`, the current todo list display and interactions are not fully defined in Splash JSON yet; they still depend on `src/todo.rs`.

### macOS-specific behavior is real

Application discovery and icon extraction assume macOS:

- scans `/Applications`, `/System/Applications`, `/System/Applications/Utilities`
- uses `open` to launch apps
- uses `sips` to convert `.icns` icons into cached PNGs

References:

- `src/main.rs:934`
- `src/main.rs:1164`
- `src/main.rs:1442`

On non-macOS platforms, the launcher falls back to demo items.

### This crate writes local state files

Runtime behavior modifies files in the crate directory:

- `.chat-history.json` for chat persistence
- `todo-app.json` or another descriptor file when app state is saved
- generated app descriptor JSON files saved from chat

References:

- `src/chat.rs:18`
- `src/todo.rs:149`
- `src/chat.rs:239`

## Files Worth Reading First

When changing behavior in this crate, start here:

- `src/main.rs` — app shell, launcher flow, mode switching, runtime app loading
- `src/app_loader.rs` — JSON descriptor and VM state bridge
- `src/chat.rs` — save/open generated apps, history, request flow
- `src/a2ui_bridge_embed.rs` — Splash prompting constraints for generated apps
- `todo-app.json` — current example descriptor for a runtime-loaded Splash app

## Relationship to Workspace Docs

Use the workspace-level docs for general Makepad/A2UI conventions:

- `../../CLAUDE.md`
- `../../AGENTS.md`

Use this file when the task is specific to `raycast-launcher`, especially around `script_mod!`, runtime Splash evaluation, descriptor JSON loading, and chat-generated apps.
