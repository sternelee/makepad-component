# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **Makepad UI component library** with a full **A2UI (Agent-to-UI) protocol renderer**. It's a Rust workspace containing:
- Native Makepad widgets (buttons, sliders, charts, etc.)
- A2UI protocol implementation for AI-generated UIs
- Demo applications and LLM bridge servers

## Build, Test, and Run Commands

```bash
# Build
cargo build --workspace                    # Full workspace build
cargo check -p makepad-component           # Fast check for core library

# Run demos
cargo run -p component-zoo --bin component-zoo    # Widget showcase
cargo run -p a2ui-demo --bin a2ui-demo            # A2UI demo app

# Run LLM bridge server (requires feature flag)
cargo build --bin a2ui-bridge --features a2ui-bridge
LLM_API_URL="https://integrate.api.nvidia.com/v1/chat/completions" \
LLM_MODEL="minimaxai/minimax-m2.1" \
LLM_API_KEY="nvapi-your-key" \
./target/debug/a2ui-bridge

# Testing
cargo test --workspace                     # All tests
cargo test -p makepad-component           # Core library only
cargo test -p makepad-component -- test_name_substring  # Single test

# Lint & Format
cargo clippy --workspace -- -D warnings   # Strict linting
cargo fmt --all                           # Format code
```

## Architecture

### Workspace Structure

```
crates/
├── ui/                    # Core library (makepad-component crate)
│   ├── widgets/          # Native widgets: MpButton, MpSlider, etc.
│   ├── a2ui/             # A2UI protocol renderer
│   │   ├── message.rs    # Protocol types (serde JSON)
│   │   ├── processor.rs  # Message → widget tree
│   │   ├── surface/      # Component rendering
│   │   └── chart_bridge/ # Chart → makepad-plot bridge
│   └── theme/            # Colors, typography
├── makepad-plot/         # Chart library (29 types + 3D)
├── component-zoo/        # Widget demo app
├── a2ui-demo/           # A2UI demo + bridge servers
├── raycast-launcher/     # Raycast-style launcher
└── makepad-clipboard/    # Clipboard utilities
```

### Key Concepts

**Makepad DSL (`live_design!`)**: Declarative UI definition with shader-based rendering. Widgets use `{{StructName}}` pattern in DSL with `#[derive(Live, LiveHook, Widget)]`.

**A2UI Protocol**: JSON-based declarative UI protocol for AI agents. Messages: `beginRendering`, `surfaceUpdate`, `dataModelUpdate`. Components reference each other by ID in a flat adjacency list.

**Widget Pattern**: Action enum + `Widget` trait impl + `handle_event` for interactions + `draw_walk` for rendering. See AGENTS.md for full pattern.

### Dependency Notes

- `makepad-widgets`: External Makepad framework, pinned to git branch in `Cargo.toml`
- Changes to `makepad-widgets` may require workspace-wide updates

## Development Workflow

### Adding New Widgets

1. Create widget file in `crates/ui/src/widgets/`
2. Define `live_design!` block with base widget and variants
3. Implement struct with `#[derive(Live, LiveHook, Widget)]`
4. Implement `Widget` trait (`handle_event`, `draw_walk`)
5. Export in `widgets/mod.rs` and add to `live_design()` function
6. Test visually: `cargo run -p component-zoo`

### Adding A2UI Components

1. Add to `ComponentType` enum in `a2ui/message.rs`
2. Handle in `a2ui/processor.rs`
3. Add render implementation in `a2ui/surface/`
4. Register in `ComponentRegistry`
5. Test with `a2ui-demo` app

### Testing Strategy

- Unit tests: `#[cfg(test)] mod tests` at bottom of impl files
- Visual testing: `component-zoo` for widgets, `a2ui-demo` for A2UI
- Protocol correctness: Test `processor.rs` and `data_model.rs` thoroughly

## Important Patterns

### Widget Implementation

```rust
#[derive(Live, LiveHook, Widget)]
pub struct MpButton {
    #[redraw] #[live] draw_bg: DrawQuad,
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[animator] animator: Animator,
    #[rust] area: Area,
}

impl Widget for MpButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(_) => self.animator_play(cx, ids!(hover.on)),
            Hit::FingerUp(_) => cx.widget_action(uid, &scope.path, MpButtonAction::Clicked),
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        // draw children...
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}
```

### A2UI Message Flow

```
JSON message → A2uiMessageProcessor → ProcessorEvent → A2uiSurface → Makepad widgets
```

Messages are parsed in `message.rs`, processed in `processor.rs`, rendered by `surface/`.

## Coding Conventions

- **Naming**: `Mp` prefix for public widgets (`MpButton`), `snake_case` for functions/fields
- **Imports**: `makepad_widgets::*` first, then std, external, local
- **Errors**: Use `Result`/`Option` for expected failures, `log!` for GUI errors
- **Testing**: Place tests in same file under `#[cfg(test)] mod tests`

## Additional Documentation

- **AGENTS.md**: Detailed coding style, commit guidelines, and implementation patterns
- **README.md**: User-facing documentation, A2UI protocol details, demo instructions
- **crates/ui/src/a2ui/**: A2UI protocol implementation with inline documentation
