# Repository Guidelines for AI Agents

This repository is a Rust workspace focused on Makepad widgets, the A2UI (Agent-to-UI) protocol renderer, and various demo applications.

## 1. Project Structure & Module Organization

- `crates/ui`: Core library (`makepad-component`). Contains widget primitives and A2UI runtime.
  - `src/widgets/`: Native Makepad widget implementations.
  - `src/a2ui/`: Protocol parsing, message processing, and UI surface rendering.
- `crates/makepad-plot`: Charting and data visualization library (2D/3D).
- `crates/raycast-launcher`: A Raycast-style launcher app using A2UI.
- `crates/a2ui-demo`: General demo app for A2UI capabilities and bridge servers.
- `crates/component-zoo`: A showcase gallery for visual verification of widgets.

## 2. Build, Test, and Lint Commands

All commands should be executed from the repository root.

### Build & Run
- `cargo build --workspace`: Build all crates in the workspace.
- `cargo check -p makepad-component`: Fast compile check for core logic.
- `cargo run -p component-zoo --bin component-zoo`: Launch the widget showcase.
- `cargo run -p a2ui-demo --bin a2ui-demo`: Launch the main A2UI demo.

### Testing
- `cargo test`: Run all tests in the workspace.
- `cargo test -p <crate_name>`: Run tests for a specific crate (e.g., `makepad-component`).
- **Run a single test**: `cargo test -p <crate_name> -- <test_name_substring>`
  - *Example*: `cargo test -p makepad-component -- test_process_surface_update`

### Linting & Formatting
- `cargo clippy --workspace -- -D warnings`: Strict lint pass across all crates.
- `cargo fmt --all`: Format all files according to project style.

## 3. Coding Style & Conventions

### General Rust Style
- **Edition**: Rust 2021.
- **Naming**:
  - `snake_case`: Variables, functions, modules, and files.
  - `UpperCamelCase`: Structs, Enums, and Traits.
  - `SCREAMING_SNAKE_CASE`: Constants and statics.
- **Formatting**: 4-space indentation (standard `rustfmt`).

### Imports & Module Usage
- Group imports: Standard library first, then external crates, then local modules.
- Use `pub use` in `lib.rs` or `mod.rs` to expose clean public APIs.
- Avoid deeply nested `use` blocks; prefer clarity.

### Makepad Specifics (`live_design!`)
- **Stable IDs**: Keep IDs in DSL (`live_design!`) stable and descriptive (e.g., `save_button`, `user_input`).
- **Registration**: All widgets and themes must be registered via `live_design(cx)` calls in their respective modules.
- **SDF Drawing**: When modifying widget shaders, maintain the SDF (Signed Distance Field) patterns used in existing widgets.

### Error Handling
- Use `Option` and `Result` for expected failure paths.
- Prefer `?` for error propagation in fallible operations (e.g., JSON parsing).
- Use `makepad_widgets::log!` for runtime debugging and error reporting in GUI paths.
- Avoid `unwrap()` in production-ready library code (`crates/ui`); use `expect("description")` if a failure truly represents an impossible state.

## 4. Testing Guidelines

- **Inline Tests**: Place unit tests in a `mod tests` block at the bottom of the implementation file, guarded by `#[cfg(test)]`.
- **Naming**: Use descriptive names like `test_should_fail_on_invalid_json` or `test_surface_update_preserves_state`.
- **A2UI Verification**: Prioritize testing the `Processor` and `DataModel` logic, as these are critical for AI-generated UI correctness.
- **Manual Check**: For UI changes, verify visually using `cargo run -p component-zoo`.

## 5. Commit & Pull Request Guidelines

- **Conventional Commits**: Use `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, or `chore:`.
- **Scoping**: Keep changes focused. Do not mix refactoring with feature additions in a single commit.
- **PR Content**:
  - Summary of changes and rationale.
  - Evidence of successful `cargo test` and `cargo clippy`.
  - For visual changes, attach screenshots or GIFs from the `component-zoo` or `raycast-launcher`.

## 6. Proactive Agent Behavior

- Before modifying a widget, check its corresponding `live_design!` block to understand its properties.
- When adding a new A2UI component, ensure it is added to the `ComponentType` enum and handled in `processor.rs`.
- Always run `cargo check` after modifying shared crates like `makepad-component` to catch downstream breakages in demo apps.
