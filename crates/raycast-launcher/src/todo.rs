use makepad_script::{ScriptHeap, ScriptObject, ScriptTrap::NoTrap};
use makepad_widgets::*;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, RwLock};

// ==================== JSON File Path ====================

static TODO_JSON_PATH: LazyLock<RwLock<Option<std::path::PathBuf>>> =
    LazyLock::new(|| RwLock::new(None));

// ==================== Configuration Types ====================

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct TodoTheme {
    #[serde(default = "default_empty_text")]
    pub(crate) empty_text: String,
    #[serde(default = "default_text_color")]
    pub(crate) text_color: String,
    #[serde(default = "default_text_done_color")]
    pub(crate) text_done_color: String,
    #[serde(default = "default_tag_text_color")]
    pub(crate) tag_text_color: String,
    #[serde(default = "default_row_bg_normal")]
    pub(crate) row_bg_normal: String,
    #[serde(default = "default_row_bg_done")]
    pub(crate) row_bg_done: String,
    #[serde(default = "default_row_stroke")]
    pub(crate) row_stroke: String,
}

fn default_empty_text() -> String {
    "No tasks yet — add one below".to_string()
}
fn default_text_color() -> String {
    "#xe2e8f0".to_string()
}
fn default_text_done_color() -> String {
    "#xa0d0a4".to_string()
}
fn default_tag_text_color() -> String {
    "#xffffff".to_string()
}
fn default_row_bg_normal() -> String {
    "#x272c34".to_string()
}
fn default_row_bg_done() -> String {
    "#x1f3a2a".to_string()
}
fn default_row_stroke() -> String {
    "#x3b424d".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct TodoItemJson {
    pub(crate) text: String,
    #[serde(default)]
    pub(crate) done: bool,
    #[serde(default)]
    pub(crate) tag: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub(crate) struct TodoConfig {
    #[serde(default)]
    pub(crate) items: Vec<TodoItemJson>,
    #[serde(default)]
    pub(crate) theme: TodoTheme,
}

// ==================== Runtime Data ====================

#[derive(Clone, Debug)]
pub(crate) struct TodoItemData {
    pub(crate) text: String,
    pub(crate) done: bool,
    pub(crate) tag: String,
}

pub(crate) static TODO_CONFIG: LazyLock<RwLock<TodoConfig>> =
    LazyLock::new(|| RwLock::new(load_todo_config()));

pub(crate) static TODOS: LazyLock<RwLock<Vec<TodoItemData>>> =
    LazyLock::new(|| RwLock::new(initial_todos()));

/// Load todo configuration from JSON file.
/// Searches: ./todo.json, <crate-dir>/todo.json, <exe-dir>/todo.json
pub(crate) fn load_todo_config() -> TodoConfig {
    let mut paths = vec![std::path::PathBuf::from("todo.json")];

    // Crate source directory (compile-time, works for `cargo run`)
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    paths.push(crate_dir.join("todo.json"));

    // Exe directory
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            paths.push(dir.join("todo.json"));
        }
    }

    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            match serde_json::from_str::<TodoConfig>(&content) {
                Ok(config) => {
                    log!("Loaded todo config from: {}", path.display());
                    *TODO_JSON_PATH.write().unwrap() = Some(path.clone());
                    return config;
                }
                Err(e) => {
                    log!("Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }

    // No file found — default to crate dir so save_todos() can create it
    let default_path = crate_dir.join("todo.json");
    log!(
        "No todo.json found, using defaults; will create at: {}",
        default_path.display()
    );
    *TODO_JSON_PATH.write().unwrap() = Some(default_path);
    TodoConfig::default()
}

/// Save current todos back to the JSON file.
pub(crate) fn save_todos() {
    let path_guard = TODO_JSON_PATH.read().unwrap();
    let Some(ref path) = *path_guard else {
        log!("No todo.json path known, cannot save");
        return;
    };

    let todos = TODOS.read().unwrap();
    let config = TODO_CONFIG.read().unwrap();

    let items: Vec<TodoItemJson> = todos
        .iter()
        .map(|t| TodoItemJson {
            text: t.text.clone(),
            done: t.done,
            tag: t.tag.clone(),
        })
        .collect();

    let save_config = TodoConfig {
        items,
        theme: config.theme.clone(),
    };

    match serde_json::to_string_pretty(&save_config) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, json) {
                log!("Failed to save todo.json: {}", e);
            } else {
                log!("Saved todo.json to: {}", path.display());
            }
        }
        Err(e) => {
            log!("Failed to serialize todo config: {}", e);
        }
    }
}

/// Reload configuration from disk and refresh data.
#[allow(dead_code)]
pub(crate) fn reload_todo_config() {
    let config = load_todo_config();
    let items: Vec<TodoItemData> = config
        .items
        .iter()
        .map(|item| TodoItemData {
            text: item.text.clone(),
            done: item.done,
            tag: item.tag.clone(),
        })
        .collect();
    *TODOS.write().unwrap() = items;
    *TODO_CONFIG.write().unwrap() = config;
}

pub(crate) fn initial_todos() -> Vec<TodoItemData> {
    let config = TODO_CONFIG.read().unwrap();
    config
        .items
        .iter()
        .map(|item| TodoItemData {
            text: item.text.clone(),
            done: item.done,
            tag: item.tag.clone(),
        })
        .collect()
}

// ==================== Helpers ====================

/// Parse a hex color string to vec4.
/// Supports: #RRGGBB, #xRRGGBB, #RRGGBBAA, #xRRGGBBAA
fn hex_to_vec4(hex: &str) -> Vec4 {
    let hex = hex.trim();
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let hex = hex.strip_prefix('x').unwrap_or(hex);

    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
        vec4(r, g, b, 1.0)
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
        let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f32 / 255.0;
        vec4(r, g, b, a)
    } else {
        vec4(1.0, 1.0, 1.0, 1.0)
    }
}

/// Read a string property from a script object.
fn read_script_string(heap: &ScriptHeap, obj: ScriptObject, key: LiveId) -> Option<String> {
    let val = heap.value(obj, ScriptValue::from_id(key), NoTrap);
    let s = val.as_string()?;
    Some(heap.string(s).to_string())
}

/// Read theme from `mod.state.app.theme` or `mod.state.todo.theme` in the script VM.
fn read_theme_from_script(cx: &mut Cx2d) -> TodoTheme {
    let mut theme = TodoTheme::default();

    cx.cx.with_vm(|vm| {
        let mod_obj = vm.module(id!(mod));
        let heap = vm.heap();

        // mod.state
        let state_val = heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
        let Some(state_obj) = state_val.as_object() else {
            return;
        };

        // Try mod.state.app.theme first (dynamically loaded apps), fallback to mod.state.todo.theme
        let mut theme_obj = None;

        // mod.state.app
        let app_val = heap.value(state_obj, ScriptValue::from_id(id!(app)), NoTrap);
        if let Some(app_obj) = app_val.as_object() {
            let t = heap.value(app_obj, ScriptValue::from_id(id!(theme)), NoTrap);
            theme_obj = t.as_object();
        }

        // Fallback: mod.state.todo.theme
        if theme_obj.is_none() {
            let todo_val = heap.value(state_obj, ScriptValue::from_id(id!(todo)), NoTrap);
            if let Some(todo_obj) = todo_val.as_object() {
                let t = heap.value(todo_obj, ScriptValue::from_id(id!(theme)), NoTrap);
                theme_obj = t.as_object();
            }
        }

        let Some(theme_obj) = theme_obj else {
            return;
        };

        if let Some(v) = read_script_string(heap, theme_obj, id!(text_color)) {
            theme.text_color = v;
        }
        if let Some(v) = read_script_string(heap, theme_obj, id!(text_done_color)) {
            theme.text_done_color = v;
        }
        if let Some(v) = read_script_string(heap, theme_obj, id!(tag_text_color)) {
            theme.tag_text_color = v;
        }
        if let Some(v) = read_script_string(heap, theme_obj, id!(row_bg_normal)) {
            theme.row_bg_normal = v;
        }
        if let Some(v) = read_script_string(heap, theme_obj, id!(row_bg_done)) {
            theme.row_bg_done = v;
        }
        if let Some(v) = read_script_string(heap, theme_obj, id!(row_stroke)) {
            theme.row_stroke = v;
        }
        if let Some(v) = read_script_string(heap, theme_obj, id!(empty_text)) {
            theme.empty_text = v;
        }
    });

    theme
}

// ==================== TodoList Custom Widget ====================

#[derive(Script, ScriptHook, Widget)]
pub struct TodoList {
    #[deref]
    view: View,
}

impl Widget for TodoList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let todos = TODOS.read().unwrap();
        let theme = read_theme_from_script(cx);
        let text_color = hex_to_vec4(&theme.text_color);
        let text_done_color = hex_to_vec4(&theme.text_done_color);
        let tag_text_color = hex_to_vec4(&theme.tag_text_color);

        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                if todos.is_empty() {
                    list.set_item_range(cx, 0, 1);
                    while let Some(item_id) = list.next_visible_item(cx) {
                        let item = list.item(cx, item_id, id!(Empty));
                        item.label(cx, ids!(empty_label))
                            .set_text(cx, &theme.empty_text);
                        item.draw_all_unscoped(cx);
                    }
                } else {
                    list.set_item_range(cx, 0, todos.len());
                    while let Some(item_id) = list.next_visible_item(cx) {
                        let Some(todo) = todos.get(item_id) else {
                            continue;
                        };
                        let item = list.item(cx, item_id, id!(Item));

                        item.check_box(cx, ids!(check)).set_active(cx, todo.done);
                        item.label(cx, ids!(label)).set_text(cx, &todo.text);
                        item.label(cx, ids!(tag_label)).set_text(cx, &todo.tag);
                        item.view(cx, ids!(tag))
                            .set_visible(cx, !todo.tag.is_empty());

                        if let Some(mut row_view) = item.view(cx, ids!(row)).borrow_mut() {
                            let done_value = if todo.done { 1.0 } else { 0.0 };
                            row_view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(done),
                                &[done_value],
                            );
                            let bg_normal = hex_to_vec4(&theme.row_bg_normal);
                            let bg_done = hex_to_vec4(&theme.row_bg_done);
                            let stroke = hex_to_vec4(&theme.row_stroke);
                            row_view.draw_bg.draw_vars.set_uniform(
                                cx,
                                live_id!(bg_normal),
                                &[bg_normal.x, bg_normal.y, bg_normal.z, bg_normal.w],
                            );
                            row_view.draw_bg.draw_vars.set_uniform(
                                cx,
                                live_id!(bg_done),
                                &[bg_done.x, bg_done.y, bg_done.z, bg_done.w],
                            );
                            row_view.draw_bg.draw_vars.set_uniform(
                                cx,
                                live_id!(stroke_color),
                                &[stroke.x, stroke.y, stroke.z, stroke.w],
                            );
                        }

                        if let Some(mut label) = item.label(cx, ids!(label)).borrow_mut() {
                            label.draw_text.color = if todo.done {
                                text_done_color
                            } else {
                                text_color
                            };
                        }

                        if let Some(mut tag_label) = item.label(cx, ids!(tag_label)).borrow_mut() {
                            tag_label.draw_text.color = tag_text_color;
                        }

                        item.draw_all_unscoped(cx);
                    }
                }
            }
        }

        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}
