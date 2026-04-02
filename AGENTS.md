# Repository Guidelines for AI Agents

This repository is a Rust workspace focused on Makepad widgets, the A2UI (Agent-to-UI) protocol renderer, and various demo applications.

## 1. Project Structure & Module Organization

```
makepad-component/
├── crates/
│   ├── ui/                      # Core library (makepad-component)
│   │   └── src/
│   │       ├── lib.rs           # Public API exports
│   │       ├── widgets/         # Native Makepad widget implementations
│   │       ├── a2ui/            # A2UI protocol renderer
│   │       │   ├── message.rs   # Protocol types (serde JSON)
│   │       │   ├── processor.rs # Message → widget tree conversion
│   │       │   ├── host.rs      # SSE client for A2A servers
│   │       │   ├── data_model.rs
│   │       │   ├── registry.rs
│   │       │   └── surface/     # Component tree rendering
│   │       └── theme/           # Color palettes, typography
│   ├── makepad-plot/            # Charting library (29 chart types)
│   ├── component-zoo/           # Widget showcase demo
│   ├── a2ui-demo/               # A2UI demo + bridge servers
│   ├── raycast-launcher/         # Raycast-style launcher app
│   └── makepad-clipboard/        # Clipboard utilities
```

## 2. Build, Test, and Lint Commands

All commands execute from repository root.

### Build & Run
```bash
cargo build --workspace                    # Full workspace build
cargo check -p makepad-component           # Fast compile check for core
cargo run -p component-zoo --bin component-zoo    # Widget showcase
cargo run -p a2ui-demo --bin a2ui-demo            # A2UI demo app
cargo run -p a2ui-demo --bin a2ui-bridge          # LLM bridge server
```

### Testing
```bash
cargo test --workspace                     # Run all tests
cargo test -p makepad-component           # Tests for specific crate
cargo test -p makepad-component -- test_process_surface_update  # Single test (substring)
cargo test -p makepad-component -- test_name --exact            # Exact match
```

### Linting & Formatting
```bash
cargo clippy --workspace -- -D warnings   # Strict lint (fails on warnings)
cargo fmt --all                          # Format all files
cargo fmt --all -- --check               # Check without changes
```

## 3. Coding Style & Conventions

### General Rust Style
- **Edition**: Rust 2021 | **Indentation**: 4 spaces | **Line Length**: Default

### Naming Conventions
| Type | Convention | Example |
|------|------------|---------|
| Variables, functions, modules | `snake_case` | `draw_bg`, `handle_event` |
| Structs, Enums, Traits | `UpperCamelCase` | `MpButton`, `MpButtonAction` |
| Constants, statics | `SCREAMING_SNAKE_CASE` | `MAX_WIDTH` |
| Widget types (public) | `Mp` prefix | `MpButton`, `MpSlider`, `MpCheckbox` |
| Internal fields | `snake_case` | `draw_bg`, `animator` |

### Import Organization
```rust
use makepad_widgets::*;           // 1. External crate glob (makepad-widgets)
use std::collections::HashMap;    // 2. Standard library
use serde::{Deserialize, Serialize}; // 3. External crates
use crate::a2ui::message::*;      // 4. Local modules (crate::)
use super::data_model::*;         // 5. Parent module (super::)
```

### Public API Patterns (lib.rs / mod.rs)
```rust
// lib.rs - Export public modules
pub mod a2ui;
pub mod theme;
pub mod widgets;

// widgets/mod.rs - Re-export with glob + individual overrides
pub mod button;
pub mod checkbox;
// ... other modules
pub use button::*;
pub use checkbox::*;
// dropdown, list only define live_design styles (no pub use)

pub fn live_design(cx: &mut Cx) {
    crate::widgets::button::live_design(cx);
    // ... register all widgets
}
```

## 4. Makepad DSL (`live_design!`)

### Widget Definition Pattern
```rust
live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::theme::colors::*;

    // Base component - {{WidgetName}} creates the struct
    pub MpButton = {{MpButton}} {
        width: Fit,
        height: Fit,
        padding: { left: 16, right: 16, top: 8, bottom: 8 }

        draw_bg: {
            instance radius: 6.0
            instance color: (PRIMARY)
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(/* ... */);
                sdf.fill(self.color);
                return sdf.result;
            }
        }

        animator: {
            hover = {
                default: off
                off = { from: { all: Forward { duration: 0.15 } }
                        apply: { draw_bg: { hover: 0.0 } } }
                on = { from: { all: Forward { duration: 0.15 } }
                       apply: { draw_bg: { hover: 1.0 } } }
            }
        }
    }

    // Variant inherits from base
    pub MpButtonPrimary = <MpButton> {
        draw_bg: { color: (PRIMARY) }
    }
}
```

### Struct with Live Derive
```rust
#[derive(Live, LiveHook, Widget)]
pub struct MpButton {
    #[redraw]           // Redraw on change
    #[live]
    draw_bg: DrawQuad,
    #[live]
    draw_text: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[live]
    text: ArcStringMut,
    #[live]
    disabled: bool,
    #[animator]
    animator: Animator,
    #[rust]             // Runtime-only field
    area: Area,
}
```

### ID Stability
- **CRITICAL**: Keep DSL IDs stable and descriptive
- Use `ids!()` macro for animator states: `ids!(hover.on)`
- ID examples: `save_button`, `user_input`, `modal_dialog`

## 5. Error Handling

```rust
// Prefer Result/Option for expected failures
fn parse_message(json: &str) -> Result<A2uiMessage, ParseError> {
    serde_json::from_str(json).map_err(ParseError::InvalidJson)
}

// Use ? for propagation
fn process_surface(json: &str) -> Result<Surface, ProcessorError> {
    let msg = parse_message(json)?;
    validate_surface(&msg)?;
    Ok(msg)
}

// Use expect() only for impossible states (library code)
let widget = registry.get(id).expect("Widget must be registered in live_design");

// Runtime errors in GUI paths
use makepad_widgets::log;
log!("Failed to render component: {}", error);
```

## 6. Widget Implementation Pattern

```rust
// Action enum (custom actions)
#[derive(Clone, Debug, DefaultNone)]
pub enum MpButtonAction {
    Clicked,
    Pressed,
    Released,
    None,
}

impl Widget for MpButton {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let uid = self.widget_uid();
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        if self.disabled { return; }

        match event.hits(cx, self.area) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerDown(_) => {
                self.animator_play(cx, ids!(pressed.on));
                cx.widget_action(uid, &scope.path, MpButtonAction::Pressed);
            }
            Hit::FingerUp(fe) => {
                self.animator_play(cx, ids!(pressed.off));
                if fe.is_over {
                    cx.widget_action(uid, &scope.path, MpButtonAction::Clicked);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_text.draw_walk(cx, Walk::fit(), Align::default(), self.text.as_ref());
        self.draw_bg.end(cx);
        self.area = self.draw_bg.area();
        DrawStep::done()
    }
}

// Helper methods
impl MpButton {
    pub fn clicked(&self, actions: &Actions) -> bool {
        actions.find_widget_action(self.widget_uid())
            .map(|a| matches!(a.cast::<MpButtonAction>(), MpButtonAction::Clicked))
            .unwrap_or(false)
    }
}

impl MpButtonRef {
    pub fn clicked(&self, actions: &Actions) -> bool {
        self.borrow().map(|inner| inner.clicked(actions)).unwrap_or(false)
    }
}
```

## 7. A2UI Protocol

When adding new A2UI components:
1. Add to `ComponentType` enum in `message.rs`
2. Handle in `processor.rs` (parse component definition)
3. Add render impl in `surface/render_impl.rs`
4. Register in `ComponentRegistry`

```rust
// message.rs - Add component type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "component")]
pub enum ComponentType {
    Text { text, usage_hint },
    Button { child, primary, action },
    // ... add new variants
}
```

## 8. Testing Guidelines

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_update_preserves_state() {
        let mut processor = A2uiMessageProcessor::new(registry);
        let events = processor.process_message(surface_update.clone());
        assert!(matches!(events[0], ProcessorEvent::SurfaceUpdated(_)));
    }
}
```

- Place unit tests in `mod tests` at bottom of implementation file
- Use descriptive names: `test_should_fail_on_invalid_json`
- Prioritize `Processor` and `DataModel` testing (A2UI correctness)
- For UI changes, verify visually: `cargo run -p component-zoo`

## 9. Commit & Pull Request Guidelines

### Commit Messages
```
feat: add MpSlider with range mode support
fix: correct tooltip positioning at screen edges
docs: add A2UI protocol documentation
refactor: extract chart_bridge module
test: add processor surface update tests
chore: update makepad-widgets dependency
```

### PR Content
- Summary of changes and rationale
- Evidence: `cargo test` and `cargo clippy` output
- Visual changes: screenshots from `component-zoo`

## 10. Proactive Agent Behavior

### Before Modifying Widgets
1. Read corresponding `live_design!` block to understand properties
2. Check existing variants for inheritance patterns
3. Run `cargo check -p makepad-component` after changes

### A2UI Development
1. New component type → Add to `ComponentType` enum + handler
2. New chart type → Add to `chart_bridge/` module
3. Always validate against A2UI protocol spec

### Dependency Changes
- `makepad-widgets`: External, pin to specific commit/branch
- Run `cargo check --workspace` to catch downstream breakages

### Code Review Triggers
- Architecture changes → Consult Oracle first
- New widget patterns → Document in AGENTS.md
- 2+ failed fix attempts → Consult Oracle
