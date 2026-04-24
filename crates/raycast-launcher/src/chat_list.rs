use makepad_widgets::*;

use crate::chat::{extract_runsplash, strip_runsplash, ChatRole, CHAT_DATA};

#[derive(Script, ScriptHook, Widget)]
pub struct ChatList {
    #[deref]
    view: View,
    #[rust]
    #[allow(dead_code)]
    animating_msg: Option<usize>,
}

impl Widget for ChatList {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let data = CHAT_DATA.read().unwrap();

        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                let msg_count = data.messages.len();
                list.set_item_range(cx, 0, msg_count);

                while let Some(item_id) = list.next_visible_item(cx) {
                    if let Some(msg) = data.messages.get(item_id) {
                        let template = match msg.role {
                            ChatRole::User => id!(User),
                            ChatRole::Assistant => id!(Assistant),
                        };
                        let item_widget = list.item(cx, item_id, template);

                        if msg.role == ChatRole::Assistant {
                            if let Some(splash_code) = extract_runsplash(&msg.text) {
                                // Render the Splash app inline below the text
                                item_widget
                                    .view(cx, ids!(splash_block))
                                    .set_visible(cx, true);
                                item_widget
                                    .widget(cx, ids!(splash_view))
                                    .set_text(cx, &splash_code);
                                // Show the explanatory text without the code block
                                let display_text = strip_runsplash(&msg.text);
                                let mut markdown = item_widget.markdown(cx, ids!(selectable));
                                markdown.set_text(cx, &display_text);
                            } else {
                                item_widget
                                    .view(cx, ids!(splash_block))
                                    .set_visible(cx, false);
                                let mut markdown = item_widget.markdown(cx, ids!(selectable));
                                markdown.set_text(cx, &msg.text);
                            }
                        } else {
                            let mut markdown = item_widget.markdown(cx, ids!(selectable));
                            markdown.set_text(cx, &msg.text);
                        }

                        item_widget.draw_all_unscoped(cx);
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
