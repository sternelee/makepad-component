# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Commands

```bash
# On macOS, set SDKROOT before building (required for Metal headers)
export SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk

# If build fails with "metal_xpc.o: found architecture 'x86_64', required architecture 'arm64'"
# The fix is already applied in cargo cache. If needed, manually patch:
# File: ~/.cargo/git/checkouts/makepad-*/platform/build.rs
# Find the "clang" command in "macos" match block and add:
#   .args(&["-target", &format!("{}-apple-macos", arch)])
# where arch = "arm64" for aarch64 targets (Apple Silicon)

# Run tests (core UI crate has unit tests in data_model, value, registry, processor, message)
cargo test -p makepad-component   # Core UI crate tests
cargo test                        # All workspace tests

# Run linter
cargo clippy -p makepad-component

# Run demos (use --bin to specify which binary)
cargo run -p component-zoo --bin component-zoo    # Widget showcase
cargo run -p a2ui-demo --bin a2ui-demo           # A2UI demo GUI
cargo run -p raycast-launcher                    # Raycast-style launcher

# Fast iteration checks
cargo check -p raycast-launcher                  # Quick compile check for launcher

# Build bridge server (LLM-powered UI) - requires a2ui-bridge feature
cargo build --bin a2ui-bridge --features a2ui-bridge

# Run bridge with custom LLM (e.g., NVIDIA NIM)
LLM_API_URL="https://integrate.api.nvidia.com/v1/chat/completions" \
LLM_MODEL="minimaxai/minimax-m2.1" \
LLM_API_KEY="nvapi-xxx" \
LLM_PORT=8082 \
./target/debug/a2ui-bridge

# Watch server (live file editing) - requires mock-server feature
cargo run --bin watch-server --features mock-server

# Math charts generator
cargo run --bin math-charts

# FFT demo
cargo run --bin fft-demo

# Mock A2A server (for testing A2A protocol)
cargo run --bin mock-a2a-server --features mock-server

# WebAssembly build
cargo makepad wasm build -p component-zoo --release
```

---

## Project Goal

Build an **A2UI (Agent-to-UI) Renderer for Makepad** that enables AI agents to generate native, interactive UIs via declarative JSON protocol.

### Architecture

```
LLM → A2UI Bridge (tools→JSON) → Makepad App (A2uiHost → A2uiProcessor → A2uiSurface → Widgets)
```

### Server Ports

| Server | Port | Feature | Binary |
|--------|------|---------|--------|
| A2UI Bridge | 8082 | `a2ui-bridge` | a2ui-bridge |
| Watch Server | 8080 | `mock-server` | watch-server |
| Mock A2A Server | 8083 | `mock-server` | mock-a2a-server |
| Raycast Launcher | N/A | embedded | raycast-launcher (embedded bridge) |

### Workspace Crates

| Crate | Purpose |
|-------|---------|
| `crates/ui` | Core: A2UI protocol types, message processor, surface renderer |
| `crates/a2ui-demo` | Demo app with bridge server, watch server, math charts |
| `crates/component-zoo` | Widget showcase (Button, Checkbox, Slider, etc.) |
| `crates/makepad-plot` | Chart library (29 2D/3D chart types) |
| `crates/raycast-launcher` | Raycast-style launcher with embedded A2UI bridge |

### Key Files

- `crates/ui/src/a2ui/message.rs` - Protocol types (serde)
- `crates/ui/src/a2ui/processor.rs` - JSON → widget tree conversion
- `crates/ui/src/a2ui/data_model.rs` - DataModel with JSON Pointer path access (has tests)
- `crates/ui/src/a2ui/value.rs` - Value types for data binding (has tests)
- `crates/ui/src/a2ui/registry.rs` - Component registry (has tests)
- `crates/ui/src/a2ui/surface/` - Component rendering
- `crates/ui/src/a2ui/chart_bridge/` - ChartComponent → makepad-plot bridge
- `crates/a2ui-demo/src/a2ui_bridge.rs` - LLM → A2UI bridge server
- `crates/raycast-launcher/src/main.rs` - Raycast launcher with A2UI integration

---

## A2UI Protocol

Messages: `beginRendering`, `surfaceUpdate`, `dataModelUpdate`, `deleteSurface`, `userAction`

Components map to Makepad widgets: Column→View, Row→View, Text→Label, Button→Button, List→PortalList, Chart→makepad-plot

### LLM Bridge Configuration

Environment variables for the A2UI bridge server:

| Variable | Default | Description |
|----------|---------|-------------|
| `LLM_API_URL` | `https://api.moonshot.ai/v1/chat/completions` | Chat completions endpoint |
| `LLM_MODEL` | `kimi-k2.5` | Model name |
| `LLM_API_KEY` | `not-needed` | API key (also reads `MOONSHOT_API_KEY`) |
| `LLM_PORT` | `8081` | Bridge server port |

**Tested LLM Providers:**

| Provider | LLM_API_URL | LLM_MODEL |
|----------|-------------|-----------|
| NVIDIA NIM | `https://integrate.api.nvidia.com/v1/chat/completions` | `minimaxai/minimax-m2.1` |
| NVIDIA NIM | `https://integrate.api.nvidia.com/v1/chat/completions` | `z-ai/glm4.7` |
| Moonshot | `https://api.moonshot.ai/v1/chat/completions` | `kimi-k2.5` |

> **Note:** The LLM must support **tool/function calling** for the bridge to work.

### Bridge Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/chat` | POST | Send natural language `{"message": "..."}`, receive A2UI JSON |
| `/rpc` | POST | A2A protocol endpoint (returns latest UI as SSE stream) |
| `/live` | GET | Live updates via SSE (real-time push) |
| `/reset` | POST | Clear conversation history |
| `/status` | GET | Server health check |
| `/inject` | POST | Inject raw A2UI JSON directly `{"a2ui": [...]}` |

### Raycast Launcher

The `raycast-launcher` crate provides a Raycast-style app launcher with embedded A2UI capabilities:

```bash
cargo run -p raycast-launcher
```

Features:
- **Embedded A2UI Bridge**: Runs a2ui-bridge directly in-process (no separate server needed)
- **Built-in Commands**: Todo list management, Chat interface
- **Live Design UI**: Uses Makepad's live_design for hot-reloadable UI

Key files:
- `crates/raycast-launcher/src/main.rs` - Main launcher panel with search input
- `crates/raycast-launcher/src/a2ui_bridge_embed.rs` - Embedded bridge integration
- `crates/raycast-launcher/src/todo.rs` - Todo command implementation
- `crates/raycast-launcher/src/chat.rs` - Chat command implementation

---

### TARGET DEMO: 购物商城示例

最终目标是实现一个与 A2UI 示例项目（`/Users/zhangalex/Work/Projects/fw/A2UI/samples/agent/adk/`）相同的购物商城 Demo。

#### 示例 JSON 输入（产品列表）

```json
[
  {
    "beginRendering": {
      "surfaceId": "product-catalog",
      "root": "root-column",
      "styles": { "primaryColor": "#007BFF", "font": "Roboto" }
    }
  },
  {
    "surfaceUpdate": {
      "surfaceId": "product-catalog",
      "components": [
        {
          "id": "root-column",
          "component": {
            "Column": {
              "children": { "explicitList": ["header", "product-list"] }
            }
          }
        },
        {
          "id": "header",
          "component": {
            "Text": {
              "text": { "literalString": "Products" },
              "usageHint": "h1"
            }
          }
        },
        {
          "id": "product-list",
          "component": {
            "List": {
              "direction": "vertical",
              "children": {
                "template": {
                  "componentId": "product-card",
                  "dataBinding": "/products"
                }
              }
            }
          }
        },
        {
          "id": "product-card",
          "component": {
            "Card": { "child": "card-row" }
          }
        },
        {
          "id": "card-row",
          "component": {
            "Row": {
              "children": { "explicitList": ["product-image", "product-info", "add-btn"] },
              "alignment": "center"
            }
          }
        },
        {
          "id": "product-image",
          "component": {
            "Image": {
              "url": { "path": "imageUrl" },
              "fit": "cover"
            }
          }
        },
        {
          "id": "product-info",
          "component": {
            "Column": {
              "children": { "explicitList": ["product-name", "product-price"] }
            }
          }
        },
        {
          "id": "product-name",
          "component": {
            "Text": {
              "text": { "path": "name" },
              "usageHint": "h3"
            }
          }
        },
        {
          "id": "product-price",
          "component": {
            "Text": { "text": { "path": "price" } }
          }
        },
        {
          "id": "add-btn-text",
          "component": {
            "Text": { "text": { "literalString": "Add to Cart" } }
          }
        },
        {
          "id": "add-btn",
          "component": {
            "Button": {
              "child": "add-btn-text",
              "primary": true,
              "action": {
                "name": "addToCart",
                "context": [
                  { "key": "productId", "value": { "path": "id" } },
                  { "key": "quantity", "value": { "literalNumber": 1 } }
                ]
              }
            }
          }
        }
      ]
    }
  },
  {
    "dataModelUpdate": {
      "surfaceId": "product-catalog",
      "path": "/",
      "contents": [
        {
          "key": "products",
          "valueMap": [
            {
              "key": "p1",
              "valueMap": [
                { "key": "id", "valueString": "SKU001" },
                { "key": "name", "valueString": "Premium Headphones" },
                { "key": "price", "valueString": "$99.99" },
                { "key": "imageUrl", "valueString": "https://example.com/headphones.jpg" }
              ]
            },
            {
              "key": "p2",
              "valueMap": [
                { "key": "id", "valueString": "SKU002" },
                { "key": "name", "valueString": "Wireless Mouse" },
                { "key": "price", "valueString": "$49.99" },
                { "key": "imageUrl", "valueString": "https://example.com/mouse.jpg" }
              ]
            },
            {
              "key": "p3",
              "valueMap": [
                { "key": "id", "valueString": "SKU003" },
                { "key": "name", "valueString": "Mechanical Keyboard" },
                { "key": "price", "valueString": "$129.99" },
                { "key": "imageUrl", "valueString": "https://example.com/keyboard.jpg" }
              ]
            }
          ]
        }
      ]
    }
  }
]
```

#### 预期 Makepad 渲染效果

```
┌─────────────────────────────────────────────────────┐
│                     Products                         │  ← h1 标题
├─────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────┐ │
│ │ [Image]  │ Premium Headphones    │ [Add to Cart]│ │  ← Card + Row
│ │          │ $99.99                │              │ │
│ └─────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────┐ │
│ │ [Image]  │ Wireless Mouse        │ [Add to Cart]│ │
│ │          │ $49.99                │              │ │
│ └─────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────┐ │
│ │ [Image]  │ Mechanical Keyboard   │ [Add to Cart]│ │
│ │          │ $129.99               │              │ │
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

#### 关键功能点

1. **动态列表渲染** - `List` 组件使用 `template` 绑定 `/products` 数据
2. **数据绑定** - `{ "path": "name" }` 自动从 DataModel 获取值
3. **组件嵌套** - Card → Row → [Image, Column, Button]
4. **用户交互** - Button 的 `action` 触发 `userAction` 消息返回服务器
5. **样式提示** - `usageHint: "h1"` / `"h3"` 控制文本样式

#### A2UI 示例项目参考

Example JSON files in this repository:
- `ui_live.json` - Live-editable A2UI JSON for testing
- `chart_test.json` - Chart component examples
- `math_test.json` - Math charts output (generated by `math-charts` binary)

---

### A2UI Adjacency List Model

Instead of deeply nested JSON, A2UI uses flat adjacency lists (LLM-friendly):

```json
{
  "updateComponents": {
    "surfaceId": "main",
    "components": [
      {"id": "root", "component": {"Column": {"children": ["title", "btn"]}}},
      {"id": "title", "component": {"Text": {"text": {"literalString": "Hello"}}}},
      {"id": "btn", "component": {"Button": {"child": "btn-text", "action": {"name": "click"}}}}
    ]
  }
}
```

### Component Mapping (A2UI → Makepad)

| A2UI Component | Makepad Widget | Notes |
|----------------|----------------|-------|
| Column | `View` (flow: Down) | Vertical layout |
| Row | `View` (flow: Right) | Horizontal layout |
| Text | `Label` | Display text with usageHint styles |
| Button | `Button` | With action binding |
| TextField | `TextInput` | Two-way data binding |
| Image | `Image` | With fit modes |
| List | `PortalList` | Scrollable with templates |
| Card | `View` + styling | Container with elevation |
| Checkbox | `CheckBox` | Boolean toggle |
| Slider | `Slider` | Range input |
| Modal | `View` + overlay | Dialog overlay |

### Data Binding

A2UI uses JSON Pointer paths for reactive data binding:

```json
// Static value
{"literalString": "Fixed text"}

// Data-bound value (reacts to DataModel changes)
{"path": "/user/name"}
```

**Makepad Implementation:**
```rust
// DataModel stores values by path
pub struct DataModel {
    data: HashMap<String, Value>,  // JSON Pointer path → Value
}

// Widgets subscribe to paths
impl A2uiText {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope) {
        // Get value from DataModel via path
        let text = self.resolve_string_value(cx, &self.text);
        self.label.set_text(&text);
        self.label.draw_walk(cx, scope, walk);
    }
}
```

### Implementation Status

The A2UI renderer is feature-complete with all core phases implemented:

| Phase | Status | Components |
|-------|--------|------------|
| Core Infrastructure | ✅ Complete | `A2uiMessage`, `DataModel`, `A2uiMessageProcessor`, `ComponentRegistry` |
| Standard Components | ✅ Complete | Column, Row, List, Card, Text, Image, Icon, Divider, Button, TextField, Checkbox, Slider, Modal, Tabs |
| Data Binding | ✅ Complete | `StringValue`, `NumberValue`, `BooleanValue` resolvers, two-way binding, template-based children |
| Actions & Events | ✅ Complete | `userAction` generation, action context resolution |
| Charts & Visualization | ✅ Complete | 29 chart types via `chart_bridge` module (2D and 3D) |
| Streaming Integration | ✅ Complete | SSE streaming, A2A client, watch server |

#### Recent Additions

- **raycast-launcher**: Raycast-style app launcher with embedded A2UI bridge, built-in Todo and Chat commands
- **Chart support**: Full makepad-plot integration with 2D charts (bar, line, pie, scatter, etc.) and 3D charts (Surface3D, Scatter3D)
- **MultipleChoice component**: Radio/checkbox group for selection scenarios

### Reference Implementations

| Platform | Package | Location |
|----------|---------|----------|
| Flutter | GenUI | External reference: `github.com/ZhangHanDong/genui` |
| Web (Lit) | A2UI Lit | External reference: `github.com/ZhangHanDong/A2UI` (renderers/lit) |
| Web (Angular) | A2UI Angular | External reference: `github.com/ZhangHanDong/A2UI` (renderers/angular) |
| Specification | A2UI Spec | External reference: `github.com/ZhangHanDong/A2UI` (specification/v0_9) |

### Skills to Use

When implementing, load these skills:

| Task | Skills to Load |
|------|----------------|
| Makepad widgets | `makepad-widgets`, `makepad-layout`, `makepad-dsl` |
| Event handling | `makepad-event-action` |
| Async/HTTP | `robius-app-architecture` |
| A2UI protocol | `a2ui-protocol`, `a2ui-components`, `a2ui-data-binding` |
| GenUI reference | `genui-a2ui-integration`, `genui-data-binding` |
| Rust patterns | `m06-error-handling`, `m02-resource` |

### Makepad Source

- **Makepad Framework**: `https://github.com/makepad/makepad` (dev branch)
- **This Project**: `/Users/sternelee/www/github/makepad-component`

---

## CRITICAL: Hook-Based Skill Loading

**IMPORTANT:** When you see a message starting with `[makepad-skills]` in the conversation, you MUST:

1. **Read the routing instruction** - e.g., `[makepad-skills] Routing to: makepad-widgets makepad-layout`
2. **Immediately call the Skill tool** for EACH skill listed before doing anything else
3. **Do not skip this step** - the skills contain essential Makepad knowledge

Example:
```
[makepad-skills] Routing to: makepad-widgets makepad-layout
```
→ Call `Skill(makepad-widgets)` then `Skill(makepad-layout)` FIRST, then answer the question.

---

## Skill Routing

For Makepad/Robius/MolyKit questions, use **context detection** and **skill dependencies** to load multiple related skills.

### Context Detection (Load Skill Bundles)

When user intent matches these contexts, load the entire skill bundle:

| Context | Trigger Keywords | Load These Skills |
|---------|------------------|-------------------|
| **Full App Development** | "build app", "create app", "从零", "完整应用", "app architecture" | makepad-basics, makepad-dsl, makepad-layout, makepad-widgets, makepad-event-action, robius-app-architecture |
| **UI Design** | "ui design", "界面设计", "design ui" | makepad-dsl, makepad-layout, makepad-widgets, makepad-animation, makepad-shaders |
| **Widget/Component Creation** | "create widget", "创建组件", "自定义组件", "custom component" | makepad-widgets, makepad-dsl, makepad-layout, makepad-animation, makepad-shaders, makepad-font, makepad-event-action |
| **Production Patterns** | "best practice", "robrix pattern", "实际项目", "production" | robius-app-architecture, robius-widget-patterns, robius-state-management, robius-event-action |

### Skill Dependencies (Auto-Load Related Skills)

When loading a skill, automatically include its dependencies:

| Primary Skill | Also Load |
|---------------|-----------|
| makepad-widgets | makepad-layout, makepad-dsl |
| makepad-animation | makepad-shaders |
| makepad-shaders | makepad-widgets |
| makepad-font | makepad-widgets |
| robius-app-architecture | makepad-basics, makepad-event-action |
| robius-widget-patterns | makepad-widgets, makepad-layout |
| robius-event-action | makepad-event-action |

### Single Skill Keywords (Fallback)

For specific questions, match keywords to individual skills:

| Keywords | Skill |
|----------|-------|
| getting started, `live_design!`, `app_main!` | makepad-basics |
| DSL syntax, inheritance, `<Widget>`, `Foo = { }` | makepad-dsl |
| layout, Flow, Walk, padding, center, align | makepad-layout |
| View, Button, Label, widget | makepad-widgets |
| event, action, Hit, FingerDown, handle_event | makepad-event-action |
| animator, state, transition, hover | makepad-animation |
| shader, draw_bg, Sdf2d, gradient, glow | makepad-shaders |
| platform, macOS, Android, iOS, WASM | makepad-platform |
| font, text, glyph, typography | makepad-font |
| splash, script, cx.eval | makepad-splash |
| Tokio, async, submit_async_request | robius-app-architecture |
| apply_over, modal, collapsible, pageflip | robius-widget-patterns |
| custom action, MatchEvent, post_action | robius-event-action |
| AppState, persistence, Scope::with_data | robius-state-management |
| Matrix SDK, sliding sync, MatrixRequest | robius-matrix-integration |
| BotClient, OpenAI, SSE streaming | molykit |
| deploy, package, APK, IPA | makepad-deployment |
| troubleshoot, error, debug | makepad-reference |

### Extended Skills

**Note:** Production patterns are integrated into robius-* skills:
- Widget patterns (modal, collapsible, drag-drop) → `robius-widget-patterns/_base/`
- State patterns (theme switching, state machine) → `robius-state-management/_base/`
- Async patterns (streaming, tokio) → `robius-app-architecture/_base/`

## Usage Examples

### Full App Development (Bundle)
```
User: "我想从零开发一个 Makepad 应用"
-> Detect: Full app context
-> Load: makepad-basics, makepad-dsl, makepad-layout, makepad-widgets,
         makepad-event-action, robius-app-architecture
-> Answer with complete app structure, widgets, events, and async patterns
```

### Widget Creation (Bundle)
```
User: "帮我创建一个自定义按钮组件"
-> Detect: Widget creation context
-> Load: makepad-widgets, makepad-dsl, makepad-layout, makepad-animation,
         makepad-shaders, makepad-font, makepad-event-action
-> Answer with widget structure, styling, animations, and event handling
```

### Simple Question (Single + Dependencies)
```
User: "如何设置字体大小"
-> Match: makepad-font
-> Auto-load dependency: makepad-widgets
-> Load: makepad-font, makepad-widgets
-> Answer with text_style, font_size, and widget context
```

### Production App (Bundle)
```
User: "参考 Robrix 的最佳实践"
-> Detect: Production context
-> Load: robius-app-architecture, robius-widget-patterns,
         robius-state-management, robius-event-action
         + dependencies: makepad-basics, makepad-widgets, makepad-layout, makepad-event-action
-> Answer with production-ready patterns from Robrix/Moly codebases
```

## Key Patterns

### Makepad Widget Definition
```rust
#[derive(Live, LiveHook, Widget)]
pub struct MyWidget {
    #[deref] view: View,
    #[live] property: f64,
    #[rust] internal_state: State,
    #[animator] animator: Animator,
}
```

### Robius Async Pattern
```rust
// UI -> Async
submit_async_request(MatrixRequest::SendMessage { ... });

// Async -> UI
Cx::post_action(MessageSentAction { ... });
SignalToUI::set_ui_signal();
```

### MolyKit Cross-Platform Async
```rust
// Platform-agnostic spawning
spawn(async move {
    let result = fetch_data().await;
    Cx::post_action(DataReady(result));
    SignalToUI::set_ui_signal();
});
```

## Default Project Settings

When creating Makepad projects:

```toml
[package]
edition = "2021"  # This project uses 2021

[dependencies]
makepad-widgets = { git = "https://github.com/makepad/makepad", branch = "dev" }

[features]
default = []
```

## Source Codebases

For deeper reference, check these codebases:

- **Makepad**: `https://github.com/makepad/makepad` - Framework source
- **A2UI Protocol**: `https://github.com/ZhangHanDong/A2UI` - A2UI spec and renderers

## Local Skills

This project includes custom Claude Code skills in `skills/`:

| Skill | Purpose |
|-------|---------|
| `makepad-screenshot` | Automated screenshot debugging for Makepad GUI apps |
| `xor-shader-techniques` | Shader techniques and patterns |

Invoke with `/screenshot` to capture and analyze running Makepad apps.
