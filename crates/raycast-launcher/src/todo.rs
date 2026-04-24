use makepad_script::{ScriptHeap, ScriptObject, ScriptTrap::NoTrap, ScriptValue};
use makepad_widgets::*;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

// ==================== Save Path ====================

static SAVE_PATH: LazyLock<RwLock<Option<PathBuf>>> = LazyLock::new(|| RwLock::new(None));

pub fn set_save_path(path: PathBuf) {
    *SAVE_PATH.write().unwrap() = Some(path);
}

// ==================== Data Types ====================

#[derive(Clone, Debug)]
pub struct TodoItemData {
    pub text: String,
    pub done: bool,
    pub tag: String,
}

#[derive(Clone, Debug, Default)]
pub struct TodoTheme {
    pub empty_text: String,
    pub text_color: String,
    pub text_done_color: String,
    pub tag_text_color: String,
    pub row_bg_normal: String,
    pub row_bg_done: String,
    pub row_stroke: String,
}

// ==================== VM State Helpers ====================

fn heap_todo_to_data(heap: &ScriptHeap, obj: ScriptObject) -> Option<TodoItemData> {
    let text = read_script_string(heap, obj, id!(text))?;
    let done_val = heap.value(obj, ScriptValue::from_id(id!(done)), NoTrap);
    let done = done_val.as_bool().unwrap_or(false);
    let tag = read_script_string(heap, obj, id!(tag)).unwrap_or_default();
    Some(TodoItemData { text, done, tag })
}

fn data_to_script_value(heap: &mut ScriptHeap, item: &TodoItemData) -> ScriptValue {
    let text = heap.new_string_from_str(&item.text);
    let tag = heap.new_string_from_str(&item.tag);
    let obj = heap.new_object();
    heap.set_value_def(obj, ScriptValue::from_id(id!(text)), text.into());
    heap.set_value_def(
        obj,
        ScriptValue::from_id(id!(done)),
        ScriptValue::from_bool(item.done),
    );
    heap.set_value_def(obj, ScriptValue::from_id(id!(tag)), tag.into());
    obj.into()
}

pub fn read_todos(cx: &mut Cx) -> Vec<TodoItemData> {
    cx.with_vm(|vm| {
        let heap = vm.heap();
        let mod_obj = heap.modules;
        let state_val = heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
        let Some(state_obj) = state_val.as_object() else {
            return Vec::new();
        };

        let mut app_obj = None;
        let app_val = heap.value(state_obj, ScriptValue::from_id(id!(app)), NoTrap);
        if let Some(obj) = app_val.as_object() {
            app_obj = Some(obj);
        }
        if app_obj.is_none() {
            let todo_val = heap.value(state_obj, ScriptValue::from_id(id!(todo)), NoTrap);
            if let Some(obj) = todo_val.as_object() {
                app_obj = Some(obj);
            }
        }
        let Some(app_obj) = app_obj else {
            return Vec::new();
        };

        let todos_val = heap.value(app_obj, ScriptValue::from_id(id!(todos)), NoTrap);
        let Some(todos_arr) = todos_val.as_array() else {
            return Vec::new();
        };
        let len = heap.array_len(todos_arr);
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let item_val = heap.array_index(todos_arr, i, NoTrap);
            if let Some(item_obj) = item_val.as_object() {
                if let Some(data) = heap_todo_to_data(heap, item_obj) {
                    result.push(data);
                }
            }
        }
        result
    })
}

fn write_todos_to_vm(cx: &mut Cx, todos: &[TodoItemData]) {
    cx.with_vm(|vm| {
        let heap = vm.heap_mut();
        let mod_obj = heap.modules;
        let state_val = heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
        let Some(state_obj) = state_val.as_object() else {
            return;
        };

        let mut app_obj = None;
        let app_val = heap.value(state_obj, ScriptValue::from_id(id!(app)), NoTrap);
        if let Some(obj) = app_val.as_object() {
            app_obj = Some(obj);
        }
        if app_obj.is_none() {
            let todo_val = heap.value(state_obj, ScriptValue::from_id(id!(todo)), NoTrap);
            if let Some(obj) = todo_val.as_object() {
                app_obj = Some(obj);
            }
        }
        let Some(app_obj) = app_obj else {
            return;
        };

        let new_arr = heap.new_array();
        for item in todos {
            let val = data_to_script_value(heap, item);
            heap.array_push(new_arr, val, NoTrap);
        }
        heap.set_value_def(app_obj, ScriptValue::from_id(id!(todos)), new_arr.into());
    });
}

pub fn add_todo(cx: &mut Cx, text: &str) {
    let mut todos = read_todos(cx);
    todos.insert(
        0,
        TodoItemData {
            text: text.to_string(),
            done: false,
            tag: String::new(),
        },
    );
    write_todos_to_vm(cx, &todos);
    save_todos(cx);
}

pub fn count_todos(cx: &mut Cx) -> (usize, usize) {
    let todos = read_todos(cx);
    let done = todos.iter().filter(|t| t.done).count();
    (done, todos.len())
}

pub fn save_todos(cx: &mut Cx) {
    let path_guard = SAVE_PATH.read().unwrap();
    let Some(ref path) = *path_guard else {
        return;
    };

    let todos = read_todos(cx);
    let theme = read_theme_from_script(cx);

    let items: Vec<serde_json::Value> = todos
        .iter()
        .map(|t| {
            serde_json::json!({
                "text": t.text,
                "done": t.done,
                "tag": t.tag,
            })
        })
        .collect();

    let json = serde_json::json!({
        "app": {
            "name": "Todo",
            "version": "1.0"
        },
        "splash_code": read_splash_code_from_file(path),
        "state": {
            "todos": items,
            "theme": {
                "empty_text": theme.empty_text,
                "text_color": theme.text_color,
                "text_done_color": theme.text_done_color,
                "tag_text_color": theme.tag_text_color,
                "row_bg_normal": theme.row_bg_normal,
                "row_bg_done": theme.row_bg_done,
                "row_stroke": theme.row_stroke,
            }
        }
    });

    match serde_json::to_string_pretty(&json) {
        Ok(s) => {
            if let Err(e) = std::fs::write(path, s) {
                log!("Failed to save todo app state: {}", e);
            } else {
                log!("Saved todo app state to: {}", path.display());
            }
        }
        Err(e) => log!("Failed to serialize todo state: {}", e),
    }
}

fn read_splash_code_from_file(path: &std::path::Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(val) => val
                .get("splash_code")
                .and_then(|v| v.as_str())
                .unwrap_or("{}")
                .to_string(),
            Err(_) => "{}".to_string(),
        },
        Err(_) => "{}".to_string(),
    }
}

// ==================== Theme Helpers ====================

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

fn read_script_string(heap: &ScriptHeap, obj: ScriptObject, key: LiveId) -> Option<String> {
    let val = heap.value(obj, ScriptValue::from_id(key), NoTrap);
    let s = val.as_string()?;
    Some(heap.string(s).to_string())
}

pub fn read_theme_from_script(cx: &mut Cx) -> TodoTheme {
    let mut theme = TodoTheme::default();

    cx.with_vm(|vm| {
        let heap = vm.heap();
        let mod_obj = heap.modules;

        let state_val = heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
        let Some(state_obj) = state_val.as_object() else {
            return;
        };

        let mut theme_obj = None;

        let app_val = heap.value(state_obj, ScriptValue::from_id(id!(app)), NoTrap);
        if let Some(app_obj) = app_val.as_object() {
            let t = heap.value(app_obj, ScriptValue::from_id(id!(theme)), NoTrap);
            theme_obj = t.as_object();
        }

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

// ==================== TodoList Widget ====================

#[derive(Clone, Debug)]
pub enum TodoListAction {
    StateChanged,
}

#[derive(Script, ScriptHook, Widget)]
pub struct TodoList {
    #[deref]
    view: View,
    #[live]
    list: WidgetRef,
}

impl Widget for TodoList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let todos = read_todos(cx.cx);
        let theme = read_theme_from_script(cx.cx);
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
                        let template = if todo.done { id!(ItemDone) } else { id!(Item) };
                        let item = list.item(cx, item_id, template);

                        item.check_box(cx, ids!(check)).set_active(cx, todo.done);
                        item.label(cx, ids!(label)).set_text(cx, &todo.text);
                        item.label(cx, ids!(tag_label)).set_text(cx, &todo.tag);
                        item.view(cx, ids!(tag))
                            .set_visible(cx, !todo.tag.is_empty());

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
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        let mut changed = false;
        let todos = read_todos(cx);

        {
            let Some(list) = self.list.borrow::<PortalList>() else {
                return;
            };

            for item_id in 0..todos.len() {
                if let Some((_, item)) = list.get_item(item_id) {
                    // CheckBox: detect state change by reading animator state directly
                    let is_checked = item
                        .child(id!(check))
                        .borrow::<CheckBox>()
                        .map_or(false, |cb| cb.active(cx));
                    let expected = todos[item_id].done;
                    if is_checked != expected {
                        cx.with_vm(|vm| {
                            let heap = vm.heap_mut();
                            let mod_obj = heap.modules;
                            let state_val =
                                heap.value(mod_obj, ScriptValue::from_id(id!(state)), NoTrap);
                            let Some(state_obj) = state_val.as_object() else {
                                return;
                            };
                            let app_val =
                                heap.value(state_obj, ScriptValue::from_id(id!(app)), NoTrap);
                            let Some(app_obj) = app_val.as_object() else {
                                return;
                            };
                            let todos_val =
                                heap.value(app_obj, ScriptValue::from_id(id!(todos)), NoTrap);
                            let Some(todos_arr) = todos_val.as_array() else {
                                return;
                            };
                            let item_val = heap.array_index(todos_arr, item_id, NoTrap);
                            if let Some(item_obj) = item_val.as_object() {
                                heap.set_value_def(
                                    item_obj,
                                    ScriptValue::from_id(id!(done)),
                                    ScriptValue::from_bool(is_checked),
                                );
                            }
                        });
                        changed = true;
                    }

                    // Delete button: use direct child borrow
                    let del_clicked = item
                        .child(id!(del))
                        .borrow::<Button>()
                        .map_or(false, |btn| btn.clicked(&actions));
                    if del_clicked {
                        let mut new_todos = read_todos(cx);
                        if item_id < new_todos.len() {
                            new_todos.remove(item_id);
                            write_todos_to_vm(cx, &new_todos);
                            changed = true;
                        }
                    }
                }
            }
        }

        if changed {
            self.redraw(cx);
            cx.widget_action(self.widget_uid(), TodoListAction::StateChanged);
        }
    }
}
