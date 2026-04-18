use makepad_widgets::*;

use crate::LauncherPanel;

#[derive(Clone)]
pub(crate) struct TodoItem {
    pub(crate) text: String,
    pub(crate) done: bool,
}

pub(crate) fn default_todos() -> Vec<TodoItem> {
    vec![
        TodoItem {
            text: "Review launcher UX".to_string(),
            done: false,
        },
        TodoItem {
            text: "Polish icon rendering".to_string(),
            done: true,
        },
    ]
}

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
            self.sync_todo_ui(cx);
        }
        self.view.text_input(cx, ids!(mode_input)).set_key_focus(cx);
    }

    fn add_todo_from_input(&mut self, cx: &mut Cx) {
        let text = self.view.text_input(cx, ids!(mode_input)).text();
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        self.todos.insert(
            0,
            TodoItem {
                text: text.to_string(),
                done: false,
            },
        );
        self.todo_draft.clear();
        self.sync_mode_input(cx);
        self.sync_todo_ui(cx);
        self.redraw(cx);
    }

    pub(crate) fn sync_todo_ui(&mut self, cx: &mut Cx) {
        let done = self.todos.iter().filter(|t| t.done).count();
        let total = self.todos.len();
        self.view
            .label(cx, ids!(todo_count_label))
            .set_text(cx, &format!("{} / {} done", done, total));
        // Progress bar removed in Makepad 2.0; label shows status

        for i in 0..8usize {
            let (visible, text, checked, is_done) = if let Some(todo) = self.todos.get(i) {
                let text = if todo.done {
                    format!("✓ {}", todo.text)
                } else {
                    todo.text.clone()
                };
                (true, text, todo.done, todo.done)
            } else {
                (false, String::new(), false, false)
            };
            let done_value = if is_done { 1.0 } else { 0.0 };

            match i {
                0 => {
                    self.view.view(cx, ids!(row_0)).set_visible(cx, visible);
                    self.view.label(cx, ids!(label_0)).set_text(cx, &text);
                    self.view
                        .check_box(cx, ids!(check_0))
                        .set_active(cx, checked);
                    if let Some(mut view) = self.view.view(cx, ids!(row_0)).borrow_mut() {
                        view.draw_bg.draw_vars.set_dyn_instance(cx, live_id!(done), &[done_value]);
                    }
                    if let Some(mut label) = self.view.label(cx, ids!(label_0)).borrow_mut() {
                        label.draw_text.color = if is_done { vec4(0.64, 0.80, 0.66, 1.0) } else { vec4(0.886, 0.91, 0.941, 1.0) };
                    }
                }
                1 => {
                    self.view.view(cx, ids!(row_1)).set_visible(cx, visible);
                    self.view.label(cx, ids!(label_1)).set_text(cx, &text);
                    self.view
                        .check_box(cx, ids!(check_1))
                        .set_active(cx, checked);
                    if let Some(mut view) = self.view.view(cx, ids!(row_1)).borrow_mut() {
                        view.draw_bg.draw_vars.set_dyn_instance(cx, live_id!(done), &[done_value]);
                    }
                    if let Some(mut label) = self.view.label(cx, ids!(label_1)).borrow_mut() {
                        label.draw_text.color = if is_done { vec4(0.64, 0.80, 0.66, 1.0) } else { vec4(0.886, 0.91, 0.941, 1.0) };
                    }
                }
                2 => {
                    self.view.view(cx, ids!(row_2)).set_visible(cx, visible);
                    self.view.label(cx, ids!(label_2)).set_text(cx, &text);
                    self.view
                        .check_box(cx, ids!(check_2))
                        .set_active(cx, checked);
                    if let Some(mut view) = self.view.view(cx, ids!(row_2)).borrow_mut() {
                        view.draw_bg.draw_vars.set_dyn_instance(cx, live_id!(done), &[done_value]);
                    }
                    if let Some(mut label) = self.view.label(cx, ids!(label_2)).borrow_mut() {
                        label.draw_text.color = if is_done { vec4(0.64, 0.80, 0.66, 1.0) } else { vec4(0.886, 0.91, 0.941, 1.0) };
                    }
                }
                3 => {
                    self.view.view(cx, ids!(row_3)).set_visible(cx, visible);
                    self.view.label(cx, ids!(label_3)).set_text(cx, &text);
                    self.view
                        .check_box(cx, ids!(check_3))
                        .set_active(cx, checked);
                    if let Some(mut view) = self.view.view(cx, ids!(row_3)).borrow_mut() {
                        view.draw_bg.draw_vars.set_dyn_instance(cx, live_id!(done), &[done_value]);
                    }
                    if let Some(mut label) = self.view.label(cx, ids!(label_3)).borrow_mut() {
                        label.draw_text.color = if is_done { vec4(0.64, 0.80, 0.66, 1.0) } else { vec4(0.886, 0.91, 0.941, 1.0) };
                    }
                }
                4 => {
                    self.view.view(cx, ids!(row_4)).set_visible(cx, visible);
                    self.view.label(cx, ids!(label_4)).set_text(cx, &text);
                    self.view
                        .check_box(cx, ids!(check_4))
                        .set_active(cx, checked);
                    if let Some(mut view) = self.view.view(cx, ids!(row_4)).borrow_mut() {
                        view.draw_bg.draw_vars.set_dyn_instance(cx, live_id!(done), &[done_value]);
                    }
                    if let Some(mut label) = self.view.label(cx, ids!(label_4)).borrow_mut() {
                        label.draw_text.color = if is_done { vec4(0.64, 0.80, 0.66, 1.0) } else { vec4(0.886, 0.91, 0.941, 1.0) };
                    }
                }
                5 => {
                    self.view.view(cx, ids!(row_5)).set_visible(cx, visible);
                    self.view.label(cx, ids!(label_5)).set_text(cx, &text);
                    self.view
                        .check_box(cx, ids!(check_5))
                        .set_active(cx, checked);
                    if let Some(mut view) = self.view.view(cx, ids!(row_5)).borrow_mut() {
                        view.draw_bg.draw_vars.set_dyn_instance(cx, live_id!(done), &[done_value]);
                    }
                    if let Some(mut label) = self.view.label(cx, ids!(label_5)).borrow_mut() {
                        label.draw_text.color = if is_done { vec4(0.64, 0.80, 0.66, 1.0) } else { vec4(0.886, 0.91, 0.941, 1.0) };
                    }
                }
                6 => {
                    self.view.view(cx, ids!(row_6)).set_visible(cx, visible);
                    self.view.label(cx, ids!(label_6)).set_text(cx, &text);
                    self.view
                        .check_box(cx, ids!(check_6))
                        .set_active(cx, checked);
                    if let Some(mut view) = self.view.view(cx, ids!(row_6)).borrow_mut() {
                        view.draw_bg.draw_vars.set_dyn_instance(cx, live_id!(done), &[done_value]);
                    }
                    if let Some(mut label) = self.view.label(cx, ids!(label_6)).borrow_mut() {
                        label.draw_text.color = if is_done { vec4(0.64, 0.80, 0.66, 1.0) } else { vec4(0.886, 0.91, 0.941, 1.0) };
                    }
                }
                7 => {
                    self.view.view(cx, ids!(row_7)).set_visible(cx, visible);
                    self.view.label(cx, ids!(label_7)).set_text(cx, &text);
                    self.view
                        .check_box(cx, ids!(check_7))
                        .set_active(cx, checked);
                    if let Some(mut view) = self.view.view(cx, ids!(row_7)).borrow_mut() {
                        view.draw_bg.draw_vars.set_dyn_instance(cx, live_id!(done), &[done_value]);
                    }
                    if let Some(mut label) = self.view.label(cx, ids!(label_7)).borrow_mut() {
                        label.draw_text.color = if is_done { vec4(0.64, 0.80, 0.66, 1.0) } else { vec4(0.886, 0.91, 0.941, 1.0) };
                    }
                }
                _ => {}
            }
        }
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

        let mut changed = false;
        let mut remove_index: Option<usize> = None;

        if let Some(v) = self.view.check_box(cx, ids!(check_0)).changed(actions) {
            if let Some(t) = self.todos.get_mut(0) {
                t.done = v;
                changed = true;
            }
        }
        if let Some(v) = self.view.check_box(cx, ids!(check_1)).changed(actions) {
            if let Some(t) = self.todos.get_mut(1) {
                t.done = v;
                changed = true;
            }
        }
        if let Some(v) = self.view.check_box(cx, ids!(check_2)).changed(actions) {
            if let Some(t) = self.todos.get_mut(2) {
                t.done = v;
                changed = true;
            }
        }
        if let Some(v) = self.view.check_box(cx, ids!(check_3)).changed(actions) {
            if let Some(t) = self.todos.get_mut(3) {
                t.done = v;
                changed = true;
            }
        }
        if let Some(v) = self.view.check_box(cx, ids!(check_4)).changed(actions) {
            if let Some(t) = self.todos.get_mut(4) {
                t.done = v;
                changed = true;
            }
        }
        if let Some(v) = self.view.check_box(cx, ids!(check_5)).changed(actions) {
            if let Some(t) = self.todos.get_mut(5) {
                t.done = v;
                changed = true;
            }
        }
        if let Some(v) = self.view.check_box(cx, ids!(check_6)).changed(actions) {
            if let Some(t) = self.todos.get_mut(6) {
                t.done = v;
                changed = true;
            }
        }
        if let Some(v) = self.view.check_box(cx, ids!(check_7)).changed(actions) {
            if let Some(t) = self.todos.get_mut(7) {
                t.done = v;
                changed = true;
            }
        }

        if self.view.button(cx, ids!(del_0)).clicked(actions) {
            remove_index = Some(0);
        }
        if self.view.button(cx, ids!(del_1)).clicked(actions) {
            remove_index = Some(1);
        }
        if self.view.button(cx, ids!(del_2)).clicked(actions) {
            remove_index = Some(2);
        }
        if self.view.button(cx, ids!(del_3)).clicked(actions) {
            remove_index = Some(3);
        }
        if self.view.button(cx, ids!(del_4)).clicked(actions) {
            remove_index = Some(4);
        }
        if self.view.button(cx, ids!(del_5)).clicked(actions) {
            remove_index = Some(5);
        }
        if self.view.button(cx, ids!(del_6)).clicked(actions) {
            remove_index = Some(6);
        }
        if self.view.button(cx, ids!(del_7)).clicked(actions) {
            remove_index = Some(7);
        }

        if let Some(i) = remove_index {
            if i < self.todos.len() {
                self.todos.remove(i);
                changed = true;
            }
        }

        if changed {
            self.sync_todo_ui(cx);
            self.redraw(cx);
        }
    }
}
