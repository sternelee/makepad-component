# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Gemini Talker is an immersive AI companion app built with Makepad. It uses Google's Gemini Live API for real-time voice/text conversation via WebSocket. The app has 4 pages: Garden (live scene), Memory (saved conversations), Music (ambient), and Info (settings). See `README.md` for the full product spec.

## Commands

```bash
cargo build -p gemini-talker
cargo run -p gemini-talker
GEMINI_API_KEY="your-key" cargo run -p gemini-talker
cargo check -p gemini-talker
```

## Architecture

```
src/
├── main.rs           # App struct, script_mod! UI definition, all event handling
├── lib.rs            # Module re-exports
├── gemini_live.rs    # WebSocket client for Gemini Live API (bidirectional voice/text)
├── audio_input.rs    # Microphone capture (PCM 16kHz mono)
├── audio_output.rs   # Audio playback of AI responses
└── memory.rs         # JSON-file-based memory storage (save/load/list conversations)
```

## Key Patterns (Makepad 2.0 Script API)

- **`script_mod!`** replaces old `live_design!` — UI is defined in script syntax (no angle brackets)
- **`#[derive(Script, ScriptHook)]`** replaces old `Live, LiveHook`
- **`AppMain::script_mod()`** replaces old `LiveRegister::live_register()`
- **`script_eval!`** for runtime state manipulation and UI updates
- **`self.ui.button(cx, ids!(...)).clicked(actions)`** — widget accessors take `cx` parameter
- **Page switching**: visibility toggle pattern (`set_visible`) on sibling Views
- **State**: managed via `mod.state` in script_mod, accessed with `state.field_name`

## Dependencies

- `makepad-widgets` — referenced by absolute local path to `/Users/sternelee/www/github/makepad/widgets`
- No dependency on `makepad-component` — uses makepad-widgets directly
- `tokio` + `tokio-tungstenite` — async runtime + WebSocket for Gemini Live
- `rfd` — native file dialog for image upload
- `dirs` — platform data directory for memory storage

## Development Phases (from README.md)

- Phase 1: App shell + 4-page static UI (complete)
- Phase 2: Gemini Live WebSocket integration + text chat (complete)
- Phase 3: Image context + speech overlay (complete)
- Phase 4: Memory system + SQLite persistence
- Phase 5: Polish + shader animations
