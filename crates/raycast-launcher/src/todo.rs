use makepad_widgets::*;
use std::sync::{LazyLock, RwLock};

#[derive(Clone, Debug)]
pub(crate) struct TodoItemData {
    pub(crate) text: String,
    pub(crate) done: bool,
    pub(crate) tag: String,
}

pub(crate) fn initial_todos() -> Vec<TodoItemData> {
    vec![
        TodoItemData { text: "Review launcher UX and interaction flow".to_string(), done: false, tag: "design".to_string() },
        TodoItemData { text: "Polish icon rendering with dithering effect".to_string(), done: true, tag: "ui".to_string() },
        TodoItemData { text: "Add dark mode toggle in settings".to_string(), done: false, tag: "feature".to_string() },
        TodoItemData { text: "Optimize search indexing for large app lists".to_string(), done: true, tag: "perf".to_string() },
        TodoItemData { text: "Write user documentation and README".to_string(), done: false, tag: "docs".to_string() },
        TodoItemData { text: "Fix memory leak in icon cache system".to_string(), done: false, tag: "bug".to_string() },
        TodoItemData { text: "Add keyboard shortcuts for all modes".to_string(), done: true, tag: "feature".to_string() },
        TodoItemData { text: "Refactor mode switching logic to be more robust".to_string(), done: false, tag: "refactor".to_string() },
        TodoItemData { text: "Test cross-platform on Windows and Linux".to_string(), done: false, tag: "qa".to_string() },
        TodoItemData { text: "Prepare release v0.2.0 with changelog".to_string(), done: false, tag: "release".to_string() },
        TodoItemData { text: "Integrate AI chat with local LLM inference".to_string(), done: false, tag: "ai".to_string() },
        TodoItemData { text: "Add splash script hot-reload for development".to_string(), done: false, tag: "dev".to_string() },
    ]
}

pub(crate) static TODOS: LazyLock<RwLock<Vec<TodoItemData>>> = LazyLock::new(|| {
    RwLock::new(initial_todos())
});

// ==================== TodoList Custom Widget ====================

#[derive(Script, ScriptHook, Widget)]
pub struct TodoList {
    #[deref]
    view: View,
}

impl Widget for TodoList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let todos = TODOS.read().unwrap();

        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                if todos.is_empty() {
                    list.set_item_range(cx, 0, 1);
                    while let Some(item_id) = list.next_visible_item(cx) {
                        let item = list.item(cx, item_id, id!(Empty));
                        item.draw_all_unscoped(cx);
                    }
                } else {
                    list.set_item_range(cx, 0, todos.len());
                    while let Some(item_id) = list.next_visible_item(cx) {
                        let Some(todo) = todos.get(item_id) else { continue };
                        let item = list.item(cx, item_id, id!(Item));

                        item.check_box(cx, ids!(check)).set_active(cx, todo.done);
                        item.label(cx, ids!(label)).set_text(cx, &todo.text);
                        item.label(cx, ids!(tag_label)).set_text(cx, &todo.tag);
                        item.view(cx, ids!(tag)).set_visible(cx, !todo.tag.is_empty());

                        if let Some(mut row_view) = item.view(cx, ids!(row)).borrow_mut() {
                            let done_value = if todo.done { 1.0 } else { 0.0 };
                            row_view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(done),
                                &[done_value],
                            );
                        }

                        if let Some(mut label) = item.label(cx, ids!(label)).borrow_mut() {
                            label.draw_text.color = if todo.done {
                                vec4(0.64, 0.80, 0.66, 1.0)
                            } else {
                                vec4(0.886, 0.91, 0.941, 1.0)
                            };
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

// ==================== LauncherPanel Todo Methods ====================

use crate::LauncherPanel;

impl LauncherPanel {
    pub(crate) fn set_todo_mode(&mut self, cx: &mut Cx, show: bool) {
        self.show_todo = show;
        if show {
            self.show_chat = false;
        }
        self.view.view(cx, ids!(launcher_view)).set_visible(cx, !show);
        self.view.view(cx, ids!(todo_view)).set_visible(cx, show);
        self.view.view(cx, ids!(chat_view)).set_visible(cx, false);
        self.sync_mode_input(cx);
        if show {
            self.sync_todo_stats(cx);
            self.redraw(cx);
        }
        self.view.text_input(cx, ids!(mode_input)).set_key_focus(cx);
    }

    fn add_todo_from_input(&mut self, cx: &mut Cx) {
        let text = self.view.text_input(cx, ids!(mode_input)).text();
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        TODOS.write().unwrap().insert(
            0,
            TodoItemData {
                text: text.to_string(),
                done: false,
                tag: String::new(),
            },
        );
        self.todo_draft.clear();
        self.sync_mode_input(cx);
        self.sync_todo_stats(cx);
        self.redraw(cx);
    }

    pub(crate) fn sync_todo_stats(&mut self, cx: &mut Cx) {
        let todos = TODOS.read().unwrap();
        let done = todos.iter().filter(|t| t.done).count();
        let total = todos.len();
        self.view
            .label(cx, ids!(todo_count_label))
            .set_text(cx, &format!("{} / {} done", done, total));
    }

    pub(crate) fn handle_todo_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.view.button(cx, ids!(mode_back_btn)).clicked(actions) {
            self.set_todo_mode(cx, false);
            self.redraw(cx);
            return;
        }

        if self.view.button(cx, ids!(mode_action_btn)).clicked(actions) {
            self.add_todo_from_input(cx);
        }

        if let Some((_text, _mods)) = self.view.text_input(cx, ids!(mode_input)).returned(actions) {
            self.add_todo_from_input(cx);
        }

        let todo_list_widget = self.view.widget(cx, ids!(todo_list));
        let list = todo_list_widget.portal_list(cx, ids!(list));

        let mut changed = false;
        for (item_id, item) in list.items_with_actions(actions) {
            if let Some(checked) = item.check_box(cx, ids!(check)).changed(actions) {
                if let Some(todo) = TODOS.write().unwrap().get_mut(item_id) {
                    todo.done = checked;
                    changed = true;
                }
            }
            if item.button(cx, ids!(del)).clicked(actions) {
                let mut todos = TODOS.write().unwrap();
                if item_id < todos.len() {
                    todos.remove(item_id);
                    changed = true;
                }
            }
        }

        if changed {
            self.sync_todo_stats(cx);
            self.redraw(cx);
        }
    }
}
