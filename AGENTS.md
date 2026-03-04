# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace for Makepad widgets, A2UI rendering, and launcher/demo apps.

- `crates/ui`: core library (`makepad-component`) for widgets, themes, and A2UI protocol/runtime.
- `crates/raycast-launcher`: Raycast-style launcher app (live_design UI, built-in Todo/Chat).
- `crates/a2ui-demo`: A2UI demo app plus bridge-related binaries and examples.
- `crates/component-zoo`: widget gallery for visual/manual verification.
- `crates/makepad-plot`: charting primitives consumed by A2UI and demos.
- `docs/`, `assets/`, `asserts/`: specs, screenshots, and static assets.

## Build, Test, and Development Commands
Run commands from repository root.

- `cargo build --workspace`: build all workspace crates.
- `cargo test`: run all tests.
- `cargo check -p raycast-launcher`: fast compile check for launcher iteration.
- `cargo test -p makepad-component`: focused tests for core widget/A2UI logic.
- `cargo clippy -p makepad-component -- -D warnings`: strict lint pass on core crate.
- `cargo run -p component-zoo --bin component-zoo`: launch widget showcase.
- `cargo run -p a2ui-demo --bin a2ui-demo`: launch A2UI demo app.
- `cargo run -p raycast-launcher --bin raycast-launcher`: run launcher locally.

## Coding Style & Naming Conventions
- Rust edition: `2021`; format with `rustfmt` (4-space indentation).
- Keep module boundaries clear: widget primitives in `crates/ui/src/widgets/`, protocol/runtime in `crates/ui/src/a2ui/`, app-specific logic in each app crate.
- Naming: `snake_case` (functions/files/modules), `UpperCamelCase` (types/traits), `SCREAMING_SNAKE_CASE` (constants).
- For Makepad `live_design!`, keep IDs stable and descriptive (`chat_send_btn`, `todo_list`, etc.) to avoid selector drift.

## Testing Guidelines
- Use inline unit tests with `#[cfg(test)] mod tests` near implementation.
- Name tests by behavior, e.g., `test_parse_surface_update`, `test_data_model_merge`.
- Prioritize A2UI parsing/builder paths, data model updates, and UI action handling regressions.
- For UI changes, include a runnable verification path (typically `cargo run -p raycast-launcher` or `component-zoo`).

## Commit & Pull Request Guidelines
- Use Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, etc.).
- Keep commits scoped to one concern; avoid mixing refactor and behavior changes.
- PRs should include:
  - concise behavior summary and rationale,
  - linked issue/context,
  - test/verification evidence (`cargo test`, `cargo check -p ...`),
  - screenshots/GIFs for visible UI updates (launcher/demo/zoo).
