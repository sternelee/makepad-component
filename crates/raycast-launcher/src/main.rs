use makepad_script::Apply;
pub use makepad_widgets;
use makepad_widgets::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

mod a2ui_bridge_embed;
mod app_loader;
mod chat;
mod chat_list;
mod todo;

script_mod! {
    use mod.prelude.widgets.*

    let state = {
        todo: {
            theme: {
                empty_text: "No tasks yet — add one below"
                text_color: "#xe2e8f0"
                text_done_color: "#xa0d0a4"
                tag_text_color: "#xffffff"
                row_bg_normal: "#x272c34"
                row_bg_done: "#x1f3a2a"
                row_stroke: "#x3b424d"
            }
        }
    }
    mod.state = state

    mod.widgets.TodoListBase = #(todo::TodoList::register_widget(vm))
    mod.widgets.TodoList = set_type_default() do mod.widgets.TodoListBase{
        width: Fill
        height: Fill
        list := PortalList{
            width: Fill
            height: Fill
            scroll_bar: ScrollBar{}

            Item := View{
                width: Fill
                height: Fit
                flow: Right
                spacing: 8
                align: VCenter
                padding: Inset{left: 8 right: 8 top: 8 bottom: 8}
                margin: Inset{top: 5 bottom: 5}
                show_bg: true
                draw_bg +: {
                    done: instance(0.0)
                    bg_normal: uniform(#x272c34)
                    bg_done: uniform(#x1f3a2a)
                    stroke_color: uniform(#x3b424d)
                    pixel: fn(){
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 8.0)
                        sdf.fill(mix(self.bg_normal self.bg_done self.done))
                        sdf.stroke(self.stroke_color 1.0)
                        return sdf.result
                    }
                }
                check := CheckBox{text: ""}
                label := Label{
                    width: Fill
                    text: ""
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 12}
                        color: #xe2e8f0
                    }
                }
                tag := View{
                    width: Fit
                    height: Fit
                    padding: Inset{left: 6 right: 6 top: 2 bottom: 2}
                    show_bg: true
                    draw_bg +: {
                        pixel: fn(){
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 4.0)
                            sdf.fill(#x3b82f6)
                            return sdf.result
                        }
                    }
                    tag_label := Label{
                        text: ""
                        draw_text +: {
                            text_style: theme.font_regular {font_size: 9}
                            color: #xffffff
                        }
                    }
                }
                del := Button{
                    text: "Remove"
                    padding: Inset{left: 8 right: 8 top: 6 bottom: 6}
                }
            }

            Empty := View{
                width: Fill
                height: Fit
                align: Center
                padding: Inset{top: 40 bottom: 40}
                empty_label := Label{
                    text: "No tasks yet - add one below"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 12}
                        color: #x8f9caf
                    }
                }
            }
        }
    }

    mod.widgets.ChatListBase = #(chat_list::ChatList::register_widget(vm))
    mod.widgets.ChatList = set_type_default() do mod.widgets.ChatListBase{
        width: Fill
        height: Fill

        list := PortalList{
            width: Fill
            height: Fill
            flow: Down
            drag_scrolling: false
            auto_tail: true
            smooth_tail: true
            selectable: true

            User := View{
                width: Fill
                height: Fit
                margin: Inset{top: 4 bottom: 4 left: 50 right: 8}
                padding: Inset{left: 12 top: 8 right: 12 bottom: 8}
                flow: Overlay
                show_bg: true
                draw_bg +: {
                    color: #x3a5a8a
                    radius: 8.0
                }

                selectable := Markdown{
                    width: Fill
                    height: Fit
                    selectable: true
                    body: ""
                    splash_block := View{
                        width: Fill
                        height: Fit
                        splash_view := Splash{
                            width: Fill
                            height: Fit
                        }
                    }
                }
            }

            Assistant := View{
                width: Fill
                height: Fit
                margin: Inset{top: 4 bottom: 4 left: 8 right: 50}
                padding: Inset{left: 12 top: 8 right: 12 bottom: 8}
                flow: Overlay
                show_bg: true
                draw_bg +: {
                    color: #x2a2a3a
                    radius: 8.0
                }

                selectable := Markdown{
                    width: Fill
                    height: Fit
                    selectable: true
                    body: ""
                    splash_block := View{
                        width: Fill
                        height: Fit
                        splash_view := Splash{
                            width: Fill
                            height: Fit
                        }
                    }
                }
            }
        }
    }

    mod.widgets.LauncherPanelBase = #(LauncherPanel::register_widget(vm))
    mod.widgets.LauncherPanel = set_type_default() do mod.widgets.LauncherPanelBase{
        width: Fill
        height: Fill
        flow: Down
        spacing: 12
        padding: Inset{left: 16 right: 16 top: 14 bottom: 14}
        show_bg: true
        draw_bg +: {
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 13.0)
                sdf.fill(#x181a1f)
                sdf.stroke(#x2f343d 1.0)
                return sdf.result
            }
        }

        mode_input_row := View{
            width: Fill
            height: Fit
            flow: Right
            align: VCenter
            spacing: 8
            margin: Inset{top: 10}

            mode_back_wrap := View{
                visible: false
                width: Fit
                height: Fit
                mode_back_btn := Button{
                    text: "‹ Back"
                    padding: Inset{left: 10 right: 10 top: 7 bottom: 7}
                }
            }

            mode_input := TextInput{
                width: Fill
                height: Fit
                empty_text: "Search applications and commands…"
                padding: Inset{left: 12 right: 12 top: 10 bottom: 10}
                draw_bg +: {
                    border_color: instance(#x3e4653)
                    focus: instance(0.0)
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 10.0)
                        let fill_base = #x1f2329
                        let fill_focus = #x242a33
                        let border_focus = #x6ea9ff
                        sdf.fill(mix(fill_base fill_focus self.focus * 0.65))
                        sdf.stroke(mix(self.border_color border_focus self.focus) 1.0)
                        return sdf.result
                    }
                }
                draw_text +: {
                    text_style: theme.font_bold {font_size: 13}
                    color: #xf4f6fb
                }
            }

            mode_action_wrap := View{
                visible: false
                width: Fit
                height: Fit
                mode_action_btn := Button{
                    text: "Add"
                    padding: Inset{left: 12 right: 12 top: 7 bottom: 7}
                }
            }

            mode_hint_label := Label{
                text: ""
                draw_text +: {
                    text_style: theme.font_regular {font_size: 10}
                    color: #x8f9caf
                }
            }
        }

        launcher_view := View{
            width: Fill
            height: Fill
            flow: Down
            spacing: 10

            View{
                width: Fill
                height: Fit
                flow: Down
                spacing: 2
                Label{
                    text: "Launcher"
                    draw_text +: {
                        text_style: theme.font_bold {font_size: 17}
                        color: #xf9fbff
                    }
                }
                Label{
                    text: "Quickly open apps and run commands"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 10}
                        color: #x8f9caf
                    }
                }
            }

            top_count_row := View{
                visible: false
                width: Fill
                height: Fit
                flow: Right
                align: VCenter
                padding: Inset{left: 10 right: 10 top: 6 bottom: 6}
                show_bg: true
                draw_bg +: {
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 7.0)
                        sdf.fill(#x1b2028)
                        sdf.stroke(#x303745 1.0)
                        return sdf.result
                    }
                }
                result_count := Label{
                    text: ""
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 11}
                        color: #xa7b0c1
                    }
                }
            }

            empty_state := View{
                visible: false
                width: Fill
                height: Fit
                flow: Down
                spacing: 6
                padding: Inset{left: 14 right: 14 top: 14 bottom: 14}
                show_bg: true
                draw_bg +: {
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 9.0)
                        sdf.fill(#x1b2028)
                        sdf.stroke(#x303745 1.0)
                        return sdf.result
                    }
                }
                empty_title := Label{
                    text: "No Results"
                    draw_text +: {
                        text_style: theme.font_bold {font_size: 13}
                        color: #xe6ebf5
                    }
                }
                empty_desc := Label{
                    text: "Try another keyword, or use /todo and /chat"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 11}
                        color: #x8f9caf
                    }
                }
            }

            results := PortalList{
                width: Fill
                height: Fill
                flow: Down
                spacing: 1

                ResultRow := View{
                    width: Fill
                    height: Fit
                    margin: Inset{top: 2 bottom: 2}

                    row_bg := View{
                        width: Fill
                        height: Fit
                        flow: Down
                        spacing: 2
                        padding: Inset{left: 11 right: 11 top: 8 bottom: 8}
                        show_bg: true
                        draw_bg +: {
                            selected: instance(0.0)
                            hovered: instance(0.0)
                            command: instance(0.0)
                            builtin: instance(0.0)
                            pixel: fn() {
                                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                                sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 7.0)
                                let base = #x242a32
                                let hover = #x2b333f
                                let command_tint = #x3a3530
                                let builtin_tint = #x2d3240
                                let tinted = mix(base command_tint self.command * 0.42)
                                let tinted = mix(tinted builtin_tint self.builtin * 0.62)
                                let active = mix(#x3a79de #x4f7de0 self.builtin)
                                let hovered_mix = mix(base hover self.hovered)
                                let hover_tinted = mix(hovered_mix tinted max(self.command self.builtin) * 0.32)
                                sdf.fill(mix(hover_tinted active self.selected))
                                let stroke_color = mix(#x323a45 #x4b5d78 self.hovered)
                                let stroke_color = mix(stroke_color #x8f7044 self.command * 0.5)
                                let stroke_color = mix(stroke_color #x6177a1 self.builtin * 0.6)
                                sdf.stroke(mix(stroke_color #x79adff self.selected) 1.0)
                                return sdf.result
                            }
                        }

                        group_label := Label{
                            text: "Applications"
                            margin: Inset{bottom: 2}
                            draw_text +: {
                                text_style: theme.font_bold {font_size: 9}
                                color: #x8fa0ba
                            }
                        }

                        View{
                            width: Fill
                            height: Fit
                            flow: Right
                            align: VCenter
                            spacing: 9

                            icon_wrap := View{
                                width: 24
                                height: 24
                                flow: Overlay
                                align: Center
                                show_bg: true
                                draw_bg +: {
                                    bg_color: instance(#x1a1f26)
                                    border_color: instance(#x323a45)
                                    pixel: fn() {
                                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 6.0)
                                        sdf.fill(self.bg_color)
                                        sdf.stroke(self.border_color 1.0)
                                        return sdf.result
                                    }
                                }

                                app_icon := Image{
                                    width: 22
                                    height: 22
                                    fit: ImageFit.Smallest
                                }

                                app_icon_fallback := Label{
                                    text: "A"
                                    draw_text +: {
                                        text_style: theme.font_bold {font_size: 12}
                                        color: #xdce3ee
                                    }
                                }
                            }

                            app_name := Label{
                                width: Fill
                                text: "App"
                                draw_text +: {
                                    text_style: theme.font_bold {font_size: 13}
                                    color: #xfffdff
                                }
                            }

                            app_meta_chip := View{
                                width: Fit
                                height: Fit
                                padding: Inset{left: 7 right: 7 top: 2 bottom: 2}
                                show_bg: true
                                draw_bg +: {
                                    fill_color: instance(#x202733)
                                    border_color: instance(#x3b4658)
                                    pixel: fn() {
                                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 5.0)
                                        sdf.fill(self.fill_color)
                                        sdf.stroke(self.border_color 1.0)
                                        return sdf.result
                                    }
                                }
                                app_meta := Label{
                                    text: "Category"
                                    draw_text +: {
                                        text_style: theme.font_regular {font_size: 9}
                                        color: #xa7b2c5
                                    }
                                }
                            }

                            action_hint_chip := View{
                                visible: false
                                width: Fit
                                height: Fit
                                padding: Inset{left: 7 right: 7 top: 2 bottom: 2}
                                show_bg: true
                                draw_bg +: {
                                    fill_color: instance(#x253043)
                                    border_color: instance(#x435a7f)
                                    pixel: fn() {
                                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 5.0)
                                        sdf.fill(self.fill_color)
                                        sdf.stroke(self.border_color 1.0)
                                        return sdf.result
                                    }
                                }
                                action_hint_text := Label{
                                    text: "Open ↩"
                                    draw_text +: {
                                        text_style: theme.font_regular {font_size: 9}
                                        color: #xbfd2ef
                                    }
                                }
                            }
                        }

                        app_desc := Label{
                            width: Fill
                            text: "Description"
                            draw_text +: {
                                text_style: theme.font_regular {font_size: 10}
                                color: #x8d9bae
                            }
                        }
                    }
                }
            }

            View{
                visible: true
                width: Fill
                height: Fit
                flow: Right
                align: VCenter
                spacing: 8
                padding: Inset{left: 10 right: 10 top: 6 bottom: 6}
                show_bg: true
                draw_bg +: {
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 7.0)
                        sdf.fill(#x1b2028)
                        sdf.stroke(#x303745 1.0)
                        return sdf.result
                    }
                }
                status_label := Label{
                    width: Fill
                    text: ""
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 10}
                        color: #x7fb9ff
                    }
                }
                status_keys_label := Label{
                    text: "Up/Down Select  |  Enter Open  |  Double Click Open"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 10}
                        color: #x8f9caf
                    }
                }
            }
        }

        todo_view := View{
            visible: false
            width: Fill
            height: Fill
            flow: Down
            spacing: 10
            padding: Inset{left: 2 right: 2 top: 2 bottom: 2}

            View{
                width: Fill
                height: Fit
                flow: Right
                align: VCenter
                spacing: 8

                Label{
                    text: "Todo List"
                    draw_text +: {
                        text_style: theme.font_bold {font_size: 17}
                        color: #xf9fbff
                    }
                }
            }

            Label{
                text: "Capture quick tasks for this workspace"
                draw_text +: {
                    text_style: theme.font_regular {font_size: 10}
                    color: #x8f9caf
                }
            }

            stats_card := View{
                width: Fill
                height: Fit
                flow: Down
                spacing: 8
                padding: Inset{left: 10 right: 10 top: 10 bottom: 10}
                show_bg: true
                draw_bg +: {
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 8.0)
                        sdf.fill(#x1b2028)
                        sdf.stroke(#x303745 1.0)
                        return sdf.result
                    }
                }

                todo_count_label := Label{
                    text: "0 / 0 done"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 10}
                        color: #xa7b4c8
                    }
                }
            }

            View{width: Fill height: 1 show_bg: true draw_bg +: {color: #x2e3541}}

            todo_list := mod.widgets.TodoList{}
        }

        chat_view := View{
            visible: false
            width: Fill
            height: Fill
            flow: Down
            spacing: 10
            padding: Inset{left: 2 right: 2 top: 2 bottom: 2}

            View{
                width: Fill
                height: Fit
                flow: Right
                align: VCenter
                spacing: 8

                Label{
                    width: Fill
                    text: "Chat"
                    draw_text +: {
                        text_style: theme.font_bold {font_size: 17}
                        color: #xf9fbff
                    }
                }
                chat_reset_btn := Button{
                    text: "Clear"
                    padding: Inset{left: 10 right: 10 top: 7 bottom: 7}
                }
            }

            Label{
                text: "Ask naturally or chat with LLM"
                draw_text +: {
                    text_style: theme.font_regular {font_size: 10}
                    color: #x8f9caf
                }
            }

            View{
                width: Fill
                height: Fit
                flow: Right
                align: VCenter
                spacing: 8
                padding: Inset{left: 10 right: 10 top: 8 bottom: 8}
                show_bg: true
                draw_bg +: {
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 8.0)
                        sdf.fill(#x1b2028)
                        sdf.stroke(#x303745 1.0)
                        return sdf.result
                    }
                }
                Label{
                    text: "LLM API:"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 10}
                        color: #xa7b4c8
                    }
                }
                chat_server_input := TextInput{
                    width: 280
                    height: Fit
                    empty_text: "https://openrouter.ai/api/v1/chat/completions"
                    padding: Inset{left: 10 right: 10 top: 7 bottom: 7}
                    draw_bg +: {
                        border_color: instance(#x3e4653)
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 8.0)
                            sdf.fill(#x1f2329)
                            sdf.stroke(self.border_color 1.0)
                            return sdf.result
                        }
                    }
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 11}
                        color: #xe2e8f0
                    }
                }
                chat_model_input := TextInput{
                    width: 160
                    height: Fit
                    empty_text: "openrouter/auto"
                    padding: Inset{left: 10 right: 10 top: 7 bottom: 7}
                    draw_bg +: {
                        border_color: instance(#x3e4653)
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 8.0)
                            sdf.fill(#x1f2329)
                            sdf.stroke(self.border_color 1.0)
                            return sdf.result
                        }
                    }
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 11}
                        color: #xe2e8f0
                    }
                }
            }

            chat_list := mod.widgets.ChatList{}

            View{
                width: Fill
                height: Fit
                flow: Right
                align: VCenter
                spacing: 8
                padding: Inset{left: 10 right: 10 top: 6 bottom: 6}
                show_bg: true
                draw_bg +: {
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 7.0)
                        sdf.fill(#x1b2028)
                        sdf.stroke(#x303745 1.0)
                        return sdf.result
                    }
                }
                chat_status_label := Label{
                    width: Fill
                    text: "Ready"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 10}
                        color: #x7fb9ff
                    }
                }
                save_app_wrap := View{
                    visible: false
                    width: Fit
                    height: Fit
                    save_app_btn := Button{
                        text: "Save as App"
                        padding: Inset{left: 10 right: 10 top: 6 bottom: 6}
                    }
                }
                open_app_wrap := View{
                    visible: false
                    width: Fit
                    height: Fit
                    open_app_btn := Button{
                        text: "Open App"
                        padding: Inset{left: 10 right: 10 top: 6 bottom: 6}
                    }
                }
                chat_keys_label := Label{
                    text: "Enter Send  |  Esc Back"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 10}
                        color: #x8f9caf
                    }
                }
            }

            chat_output_scroll := ScrollYView{
                width: Fill
                height: Fill
                show_bg: true
                draw_bg +: {
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 8.0)
                        sdf.fill(#x1b2028)
                        sdf.stroke(#x303745 1.0)
                        return sdf.result
                    }
                }
            }
        }
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(900 700)
                window.title: "Raycast Launcher"
                pass +: { clear_color: #x0f1115 }
                body +: {
                    bg_view := View{
                        width: Fill
                        height: Fill
                        show_bg: true
                        draw_bg +: {
                            pixel: fn() {
                                let center = vec2(0.5 0.5)
                                let d = distance(self.pos center)
                                let t = clamp(d * 1.35 0.0 1.0)
                                return mix(#x0f1115 #x20242b t)
                            }
                        }
                        launcher := mod.widgets.LauncherPanel{}
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Clone)]
enum LaunchTarget {
    OpenPath(String),
    Command { program: String, args: Vec<String> },
    OpenTodo,
    OpenChat,
    OpenSplashApp(String),
}

#[derive(Clone)]
struct LauncherItem {
    app_name: String,
    subtitle: String,
    category: String,
    search_key: String,
    bundle_path: Option<String>,
    icon_fallback: String,
    launch: LaunchTarget,
}

#[derive(Script, Widget)]
pub struct LauncherPanel {
    #[deref]
    view: View,
    #[rust]
    all_items: Vec<LauncherItem>,
    #[rust]
    filtered_indices: Vec<usize>,
    #[rust]
    query: String,
    #[rust]
    todo_draft: String,
    #[rust]
    chat_draft: String,
    #[rust]
    selected_index: usize,
    #[rust]
    icon_cache: HashMap<usize, Option<String>>,
    #[rust]
    row_hit_rects: Vec<(usize, Rect)>,
    #[rust]
    hovered_index: Option<usize>,
    #[rust]
    last_click_item: Option<usize>,
    #[rust]
    last_click_time: f64,
    #[rust]
    show_todo: bool,
    #[rust]
    show_chat: bool,
    #[rust]
    chat_messages: Vec<chat::ChatMessage>,
    #[rust]
    chat_loading: bool,
    #[rust]
    chat_server_url: String,
    #[rust]
    chat_model: String,
    #[rust]
    chat_api_key: String,
    #[rust]
    stream_timer: Option<Timer>,
    #[rust]
    stream_buffer: String,
    #[rust]
    stream_msg_index: usize,
    #[rust]
    last_saved_app_path: Option<String>,
}

impl ScriptHook for LauncherPanel {
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        // Load todo config before entering vm.with_cx_mut so we can inject it into mod.state.
        let todo_config = todo::load_todo_config();

        vm.with_cx_mut(|cx| {
            self.all_items = load_launcher_items();
            self.query.clear();
            self.todo_draft.clear();
            self.chat_draft.clear();
            self.selected_index = 0;
            self.icon_cache.clear();
            self.row_hit_rects.clear();
            self.hovered_index = None;
            self.last_click_item = None;
            self.last_click_time = 0.0;
            self.show_todo = false;
            *todo::TODOS.write().unwrap() = todo::initial_todos();
            self.show_chat = false;
            self.chat_messages = chat::default_or_history();
            self.chat_loading = false;
            self.last_saved_app_path = None;
            self.chat_server_url = std::env::var("LLM_API_URL")
                .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".to_string());
            self.chat_model =
                std::env::var("LLM_MODEL").unwrap_or_else(|_| "openrouter/auto".to_string());
            self.chat_api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
            self.rebuild_filter();
            self.sync_mode_input(cx);
            self.view.text_input(cx, ids!(mode_input)).set_key_focus(cx);
            self.sync_todo_stats(cx);
            self.sync_chat_ui(cx);
            self.view
                .text_input(cx, ids!(chat_server_input))
                .set_text(cx, &self.chat_server_url);
            self.view
                .text_input(cx, ids!(chat_model_input))
                .set_text(cx, &self.chat_model);
            self.update_labels(cx, "Ready");
        });

        // Inject todo theme from JSON into mod.state.todo.theme via runtime Splash eval.
        let theme = &todo_config.theme;
        let splash_code = format!(
            r#"mod.state.todo.theme.empty_text = "{}"
            mod.state.todo.theme.text_color = "{}"
            mod.state.todo.theme.text_done_color = "{}"
            mod.state.todo.theme.tag_text_color = "{}"
            mod.state.todo.theme.row_bg_normal = "{}"
            mod.state.todo.theme.row_bg_done = "{}"
            mod.state.todo.theme.row_stroke = "{}""#,
            theme.empty_text.replace('"', "\\\""),
            theme.text_color.replace('"', "\\\""),
            theme.text_done_color.replace('"', "\\\""),
            theme.tag_text_color.replace('"', "\\\""),
            theme.row_bg_normal.replace('"', "\\\""),
            theme.row_bg_done.replace('"', "\\\""),
            theme.row_stroke.replace('"', "\\\""),
        );
        let script_mod = makepad_script::ScriptMod {
            cargo_manifest_path: String::new(),
            module_path: String::new(),
            file: String::new(),
            line: 0,
            column: 0,
            code: String::new(),
            values: vec![],
        };
        let result = vm.eval_with_append_source(script_mod, &splash_code, ScriptValue::NIL.into());
        if let Some(err) = result.as_err() {
            log!("Failed to inject todo theme into mod.state: {:?}", err);
        } else {
            log!("Injected todo theme into mod.state via Splash runtime eval");
        }
    }
}

fn launcher_item(
    app_name: String,
    subtitle: String,
    category: String,
    bundle_path: Option<String>,
    icon_fallback_override: Option<&str>,
    launch: LaunchTarget,
    extra_keywords: &str,
) -> LauncherItem {
    let search_key = format!(
        "{} {} {} {}",
        app_name.to_lowercase(),
        subtitle.to_lowercase(),
        category.to_lowercase(),
        extra_keywords.to_lowercase()
    );

    let icon_fallback = if let Some(override_text) = icon_fallback_override {
        override_text.to_string()
    } else {
        app_name
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string())
    };

    LauncherItem {
        app_name,
        subtitle,
        category,
        search_key,
        bundle_path,
        icon_fallback,
        launch,
    }
}

fn command_items() -> Vec<LauncherItem> {
    vec![
        launcher_item(
            "Open Project Folder".to_string(),
            "Command: open current workspace".to_string(),
            "Command".to_string(),
            None,
            Some(">"),
            LaunchTarget::Command {
                program: "open".to_string(),
                args: vec![".".to_string()],
            },
            "finder folder repo workspace",
        ),
        launcher_item(
            "Run Cargo Test".to_string(),
            "Command: cargo test -p raycast-launcher".to_string(),
            "Command".to_string(),
            None,
            Some("$"),
            LaunchTarget::Command {
                program: "cargo".to_string(),
                args: vec![
                    "test".to_string(),
                    "-p".to_string(),
                    "raycast-launcher".to_string(),
                ],
            },
            "rust ci tests",
        ),
        launcher_item(
            "Todo".to_string(),
            "Open built-in todo list".to_string(),
            "Command".to_string(),
            None,
            Some("T"),
            LaunchTarget::OpenTodo,
            "tasks checklist todos 待办 任务 清单",
        ),
        launcher_item(
            "Chat".to_string(),
            "Open A2UI conversation panel".to_string(),
            "Command".to_string(),
            None,
            Some("C"),
            LaunchTarget::OpenChat,
            "a2ui chat llm assistant 对话",
        ),
    ]
}

#[cfg(target_os = "macos")]
fn scan_macos_applications() -> Vec<LauncherItem> {
    let roots = [
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
    ];

    let mut seen_paths = HashSet::new();
    let mut app_paths = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }
        let mut stack = vec![(root, 0usize)];
        while let Some((dir, depth)) = stack.pop() {
            let read_dir = match fs::read_dir(&dir) {
                Ok(rd) => rd,
                Err(_) => continue,
            };

            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".app") {
                            if seen_paths.insert(path.clone()) {
                                app_paths.push(path);
                            }
                            continue;
                        }
                    }
                    if depth < 2 {
                        stack.push((path, depth + 1));
                    }
                }
            }
        }
    }

    app_paths.sort();

    app_paths
        .into_iter()
        .filter_map(|path| {
            let app_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())?;
            let display_path = path.to_string_lossy().to_string();

            Some(launcher_item(
                app_name,
                display_path.clone(),
                "Application".to_string(),
                Some(display_path.clone()),
                None,
                LaunchTarget::OpenPath(display_path.clone()),
                &display_path,
            ))
        })
        .collect()
}

#[cfg(not(target_os = "macos"))]
fn scan_macos_applications() -> Vec<LauncherItem> {
    Vec::new()
}

fn fallback_demo_apps() -> Vec<LauncherItem> {
    vec![
        launcher_item(
            "Terminal".to_string(),
            "Fallback entry (non-macOS)".to_string(),
            "Application".to_string(),
            None,
            None,
            LaunchTarget::Command {
                program: "echo".to_string(),
                args: vec!["Terminal selected".to_string()],
            },
            "shell",
        ),
        launcher_item(
            "VS Code".to_string(),
            "Fallback entry (non-macOS)".to_string(),
            "Application".to_string(),
            None,
            None,
            LaunchTarget::Command {
                program: "echo".to_string(),
                args: vec!["VS Code selected".to_string()],
            },
            "editor",
        ),
    ]
}

fn scan_splash_apps() -> Vec<LauncherItem> {
    let mut items = Vec::new();
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with("-app.json") || name.ends_with("_app.json") {
                    if let Some(app) = app_loader::load_app_descriptor(name) {
                        items.push(launcher_item(
                            app.app.name.clone(),
                            format!("Splash App: {}", app.app.name),
                            "Splash Apps".to_string(),
                            None,
                            Some("S"),
                            LaunchTarget::OpenSplashApp(name.to_string()),
                            "splash app ui generated",
                        ));
                    }
                }
            }
        }
    }
    items
}

fn load_launcher_items() -> Vec<LauncherItem> {
    let mut items = scan_macos_applications();
    if items.is_empty() {
        items = fallback_demo_apps();
    }

    items.extend(command_items());
    items.extend(scan_splash_apps());
    items.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.app_name.to_lowercase().cmp(&b.app_name.to_lowercase()))
    });
    items
}

impl LauncherPanel {
    pub(crate) fn sync_mode_input(&mut self, cx: &mut Cx) {
        let (
            empty_text,
            text,
            read_only,
            show_back,
            show_action,
            action_text,
            action_disabled,
            row_spacing,
            mode_hint_text,
        ) = if self.show_todo {
            (
                "Add a todo and press Enter...",
                self.todo_draft.as_str(),
                false,
                true,
                true,
                "Add",
                false,
                8.0,
                "Enter Add  |  Esc Back",
            )
        } else if self.show_chat {
            (
                "Describe a UI to create, e.g. 'Build a dark calculator' or 'Design a music player'",
                self.chat_draft.as_str(),
                self.chat_loading,
                true,
                true,
                if self.chat_loading {
                    "Sending..."
                } else {
                    "Send"
                },
                self.chat_loading,
                8.0,
                if self.chat_loading {
                    "Esc Back"
                } else {
                    "Enter Send  |  Esc Back"
                },
            )
        } else {
            (
                "Search apps and commands...",
                self.query.as_str(),
                false,
                false,
                false,
                "Add",
                false,
                0.0,
                "Up/Down Select  |  Enter Open",
            )
        };

        if let Some(mut v) = self.view.view(cx, ids!(mode_input_row)).borrow_mut() {
            v.layout.spacing = row_spacing;
        }
        self.view
            .text_input(cx, ids!(mode_input))
            .set_empty_text(cx, empty_text.to_string());
        self.view
            .text_input(cx, ids!(mode_input))
            .set_text(cx, text);
        self.view
            .text_input(cx, ids!(mode_input))
            .set_is_read_only(cx, read_only);
        self.view
            .widget(cx, ids!(mode_back_wrap))
            .set_visible(cx, show_back);
        self.view
            .widget(cx, ids!(mode_back_btn))
            .set_visible(cx, show_back);
        let action_btn = self.view.button(cx, ids!(mode_action_btn));
        self.view
            .widget(cx, ids!(mode_action_wrap))
            .set_visible(cx, show_action);
        self.view
            .widget(cx, ids!(mode_action_btn))
            .set_visible(cx, show_action);
        action_btn.set_text(cx, action_text);
        action_btn.set_disabled(cx, action_disabled);
        self.view
            .label(cx, ids!(mode_hint_label))
            .set_text(cx, mode_hint_text);
    }

    fn icon_cache_path(icns_path: &Path) -> Option<PathBuf> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        // Bump version when conversion params change to avoid stale cached icons.
        "v4_sips_256".hash(&mut hasher);
        icns_path.to_string_lossy().hash(&mut hasher);
        let hash = hasher.finish();

        let cache_dir = std::env::temp_dir().join("raycast-launcher-icons");
        if fs::create_dir_all(&cache_dir).is_err() {
            return None;
        }
        Some(cache_dir.join(format!("{:016x}.png", hash)))
    }

    fn find_icns(bundle_path: &str) -> Option<PathBuf> {
        let resources = Path::new(bundle_path).join("Contents").join("Resources");
        let read_dir = fs::read_dir(resources).ok()?;

        let mut first_icns: Option<PathBuf> = None;
        for entry in read_dir.flatten() {
            let path = entry.path();
            let is_icns = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("icns"))
                .unwrap_or(false);
            if !is_icns {
                continue;
            }

            if first_icns.is_none() {
                first_icns = Some(path.clone());
            }
            let lower = path.to_string_lossy().to_lowercase();
            if lower.contains("appicon") || lower.contains("icon") {
                return Some(path);
            }
        }
        first_icns
    }

    fn resolve_icon_for_index(&mut self, source_idx: usize) -> Option<String> {
        if let Some(cached) = self.icon_cache.get(&source_idx) {
            return cached.clone();
        }

        let resolved = (|| {
            let bundle_path = self.all_items.get(source_idx)?.bundle_path.as_ref()?;
            let cache_key_path = Path::new(bundle_path).join("Contents").join("Info.plist");
            let png = Self::icon_cache_path(&cache_key_path)?;

            if !png.exists() {
                #[cfg(target_os = "macos")]
                {
                    let icns = Self::find_icns(bundle_path)?;
                    let status = Command::new("sips")
                        .args([
                            "-s",
                            "format",
                            "png",
                            &icns.to_string_lossy(),
                            "--resampleHeightWidthMax",
                            "256",
                            "--out",
                            &png.to_string_lossy(),
                        ])
                        .status()
                        .ok()?;
                    if !status.success() || !png.exists() {
                        return None;
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    return None;
                }
            }

            Some(png.to_string_lossy().to_string())
        })();

        self.icon_cache.insert(source_idx, resolved.clone());
        resolved
    }

    fn rebuild_filter(&mut self) {
        let q = self.query.trim().to_lowercase();
        let q_norm = q.trim_start_matches('/').to_string();
        self.filtered_indices = self
            .all_items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                if q.is_empty()
                    || item.search_key.contains(&q)
                    || (!q_norm.is_empty() && item.search_key.contains(&q_norm))
                {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        if q.contains("todo") || q.contains("待办") || q.contains("任务") {
            if let Some(todo_idx) = self
                .all_items
                .iter()
                .position(|it| matches!(it.launch, LaunchTarget::OpenTodo))
            {
                self.filtered_indices.retain(|idx| *idx != todo_idx);
                self.filtered_indices.insert(0, todo_idx);
            }
        }

        if q.contains("chat") || q.contains("对话") || q.contains("聊天") || q_norm == "chat" {
            if let Some(chat_idx) = self
                .all_items
                .iter()
                .position(|it| matches!(it.launch, LaunchTarget::OpenChat))
            {
                self.filtered_indices.retain(|idx| *idx != chat_idx);
                self.filtered_indices.insert(0, chat_idx);
            }
        }

        // Built-in modules are always shown first in search results.
        self.filtered_indices.sort_by_key(|idx| {
            let is_builtin = self
                .all_items
                .get(*idx)
                .map(|it| {
                    matches!(
                        it.launch,
                        LaunchTarget::OpenTodo
                            | LaunchTarget::OpenChat
                            | LaunchTarget::OpenSplashApp(_)
                    )
                })
                .unwrap_or(false);
            if is_builtin {
                0
            } else {
                1
            }
        });

        if self.filtered_indices.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.filtered_indices.len() {
            self.selected_index = self.filtered_indices.len() - 1;
        }
    }

    fn step_selection(&mut self, delta: isize) {
        if self.filtered_indices.is_empty() {
            self.selected_index = 0;
            return;
        }

        let len = self.filtered_indices.len() as isize;
        let next = (self.selected_index as isize + delta).rem_euclid(len);
        self.selected_index = next as usize;
    }

    fn selected_item(&self) -> Option<&LauncherItem> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|idx| self.all_items.get(*idx))
    }

    pub(crate) fn ensure_selection_visible(&self, cx: &mut Cx) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let target = self
            .selected_index
            .min(self.filtered_indices.len().saturating_sub(1));
        let list = self.view.portal_list(cx, ids!(results));
        let first = list.first_id();
        let visible = list.visible_items().max(1);
        let last = first.saturating_add(visible.saturating_sub(1));
        if target < first {
            list.set_first_id(target);
        } else if target > last {
            list.set_first_id(target.saturating_sub(visible.saturating_sub(1)));
        }
    }

    fn group_name_for(item: &LauncherItem) -> &'static str {
        match item.launch {
            LaunchTarget::OpenTodo | LaunchTarget::OpenChat => "Built-in",
            LaunchTarget::OpenSplashApp(_) => "Splash Apps",
            LaunchTarget::Command { .. } => "Commands",
            LaunchTarget::OpenPath(_) => "Applications",
        }
    }

    fn update_labels(&mut self, cx: &mut Cx, _status_hint: &str) {
        let has_results = !self.filtered_indices.is_empty();
        self.view
            .widget(cx, ids!(results))
            .set_visible(cx, has_results);
        self.view
            .widget(cx, ids!(empty_state))
            .set_visible(cx, !has_results);
        let mut command_count = 0usize;
        let mut app_count = 0usize;
        for idx in &self.filtered_indices {
            if let Some(item) = self.all_items.get(*idx) {
                if item.category == "Command" {
                    command_count += 1;
                } else {
                    app_count += 1;
                }
            }
        }
        let selected_text = if self.filtered_indices.is_empty() {
            "0 selected".to_string()
        } else {
            format!(
                "{} selected",
                self.selected_index
                    .saturating_add(1)
                    .min(self.filtered_indices.len())
            )
        };
        let count_text = format!(
            "{} result(s)  |  {}  |  {} app / {} cmd",
            self.filtered_indices.len(),
            selected_text,
            app_count,
            command_count
        );
        self.view.label(cx, ids!(result_count)).set_text(cx, "");
        if has_results {
            self.view
                .label(cx, ids!(empty_title))
                .set_text(cx, "No Results");
            self.view
                .label(cx, ids!(empty_desc))
                .set_text(cx, "Try another keyword, or use /todo and /chat");
        } else {
            let q = self.query.trim();
            if q.is_empty() {
                self.view
                    .label(cx, ids!(empty_title))
                    .set_text(cx, "Start Searching");
                self.view
                    .label(cx, ids!(empty_desc))
                    .set_text(cx, "Type app or command name. Try: todo, chat, terminal");
            } else {
                self.view
                    .label(cx, ids!(empty_title))
                    .set_text(cx, "No Results");
                self.view
                    .label(cx, ids!(empty_desc))
                    .set_text(cx, &format!("No match for \"{}\". Try /todo or /chat", q));
            }
        }

        self.view
            .label(cx, ids!(status_label))
            .set_text(cx, &count_text);
        self.view
            .label(cx, ids!(status_keys_label))
            .set_text(cx, "Up/Down Select  |  Enter Open  |  Double Click Open");
    }

    fn launch_selected(&mut self, cx: &mut Cx) {
        let Some(item) = self.selected_item() else {
            self.update_labels(cx, "No item to launch");
            return;
        };

        let launch = item.launch.clone();
        match launch {
            LaunchTarget::OpenPath(path) => {
                #[cfg(target_os = "macos")]
                {
                    let status = Command::new("open").arg(&path).status();
                    match status {
                        Ok(exit) if exit.success() => self.update_labels(cx, "Launched"),
                        Ok(exit) => self.update_labels(cx, &format!("Launch failed ({})", exit)),
                        Err(_) => self.update_labels(cx, "Launch failed (open command error)"),
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    self.update_labels(cx, &format!("Selected: {}", path));
                }
            }
            LaunchTarget::Command { program, args } => {
                let spawn = Command::new(&program).args(&args).spawn();
                match spawn {
                    Ok(_) => self.update_labels(cx, &format!("Executed: {}", program)),
                    Err(_) => self.update_labels(cx, &format!("Command failed: {}", program)),
                }
            }
            LaunchTarget::OpenTodo => {
                self.set_todo_mode(cx, true);
                self.update_labels(cx, "Opened");
            }
            LaunchTarget::OpenChat => {
                self.set_chat_mode(cx, true);
                self.update_labels(cx, "Opened");
            }
            LaunchTarget::OpenSplashApp(ref path) => {
                self.show_todo = true;
                self.show_chat = false;
                self.view
                    .view(cx, ids!(launcher_view))
                    .set_visible(cx, false);
                self.view.view(cx, ids!(todo_view)).set_visible(cx, true);
                self.view.view(cx, ids!(chat_view)).set_visible(cx, false);
                self.sync_mode_input(cx);
                self.load_splash_app(cx, path);
                self.sync_todo_stats(cx);
                self.redraw(cx);
                self.view.text_input(cx, ids!(mode_input)).set_key_focus(cx);
                self.update_labels(cx, "Opened");
            }
        }

        self.redraw(cx);
    }

    // ==================== Todo Methods ====================

    fn load_splash_app(&mut self, cx: &mut Cx, path: &str) {
        let Some(app) = app_loader::load_app_descriptor(path) else {
            log!("Failed to load app descriptor: {}", path);
            return;
        };

        // Eval splash code for dynamic templates (e.g. AI-generated apps).
        // Skip if empty — compile-time templates (e.g. Todo app) are already in place.
        let splash_code = app.splash_code.trim();
        if !splash_code.is_empty() && splash_code != "{}" {
            let templates_value = cx.with_vm(|vm| {
                let script_mod = app_loader::script_mod_from_code(splash_code);
                vm.eval(script_mod)
            });

            // Apply templates to PortalList with Apply::Reload
            let todo_list_widget = self.view.widget(cx, ids!(todo_list));
            let list = todo_list_widget.portal_list(cx, ids!(list));
            if let Some(mut list_inner) = list.borrow_mut() {
                cx.with_vm(|vm| {
                    list_inner.script_apply(vm, &Apply::Reload, &mut Scope::empty(), templates_value);
                });
            }
            let _ = (); // Drop temporaries before block end
        }

        // Inject state into mod.state.app
        app_loader::inject_app_state(cx, &app.state);

        // Sync todo data from JSON state into runtime TODOS (if present)
        if let Some(todos) = app.state.get("todos").and_then(|v| v.as_array()) {
            let items: Vec<todo::TodoItemData> = todos
                .iter()
                .map(|t| todo::TodoItemData {
                    text: t
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    done: t.get("done").and_then(|v| v.as_bool()).unwrap_or(false),
                    tag: t
                        .get("tag")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
                .collect();
            *todo::TODOS.write().unwrap() = items;
        }

        self.redraw(cx);
    }

    fn load_todo_app(&mut self, cx: &mut Cx) {
        self.load_splash_app(cx, "todo-app.json");
    }

    pub(crate) fn set_todo_mode(&mut self, cx: &mut Cx, show: bool) {
        self.show_todo = show;
        if show {
            self.show_chat = false;
            self.load_todo_app(cx);
        }
        self.view
            .view(cx, ids!(launcher_view))
            .set_visible(cx, !show);
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
        todo::TODOS.write().unwrap().insert(
            0,
            todo::TodoItemData {
                text: text.to_string(),
                done: false,
                tag: String::new(),
            },
        );
        self.todo_draft.clear();
        self.sync_mode_input(cx);
        self.sync_todo_stats(cx);
        self.redraw(cx);
        todo::save_todos();
    }

    pub(crate) fn sync_todo_stats(&mut self, cx: &mut Cx) {
        let todos = todo::TODOS.read().unwrap();
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
                if let Some(todo) = todo::TODOS.write().unwrap().get_mut(item_id) {
                    todo.done = checked;
                    changed = true;
                }
            }
            if item.button(cx, ids!(del)).clicked(actions) {
                let mut todos = todo::TODOS.write().unwrap();
                if item_id < todos.len() {
                    todos.remove(item_id);
                    changed = true;
                }
            }
        }

        if changed {
            self.sync_todo_stats(cx);
            self.redraw(cx);
            todo::save_todos();
        }
    }
}

impl Widget for LauncherPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        if let Event::NetworkResponses(responses) = event {
            self.handle_chat_network_responses(cx, responses);
        }

        if let Event::Timer(te) = event {
            if let Some(timer) = self.stream_timer {
                if timer.is_timer(te).is_some() {
                    if let Some(msg) = self.chat_messages.get_mut(self.stream_msg_index) {
                        let current_len = msg.text.len();
                        let target_len = self.stream_buffer.len();
                        if current_len < target_len {
                            // Append next char(s) — batch a few for speed
                            let batch = (target_len - current_len).min(3);
                            msg.text
                                .push_str(&self.stream_buffer[current_len..current_len + batch]);
                            {
                                let mut data = chat::CHAT_DATA.write().unwrap();
                                if let Some(dm) = data.messages.get_mut(self.stream_msg_index) {
                                    dm.text.clone_from(&msg.text);
                                }
                            }
                            self.redraw(cx);
                        } else {
                            // Streaming complete
                            self.stream_timer = None;
                            self.stream_buffer.clear();
                            chat::save_chat_history(&self.chat_messages);
                            self.view
                                .label(cx, ids!(chat_status_label))
                                .set_text(cx, "Response received");
                            self.sync_chat_ui(cx);
                        }
                    } else {
                        self.stream_timer = None;
                    }
                }
            }
        }

        if let Some(text) = self.view.text_input(cx, ids!(mode_input)).changed(&actions) {
            if self.show_todo {
                self.todo_draft = text;
            } else if self.show_chat {
                self.chat_draft = text;
            } else {
                self.query = text;
                self.selected_index = 0;
                self.rebuild_filter();
                self.update_labels(cx, "Filtered");
            }
            self.redraw(cx);
        }

        if self.show_todo {
            self.handle_todo_actions(cx, &actions);
            if let Event::KeyDown(key) = event {
                if key.key_code == KeyCode::Escape {
                    self.set_todo_mode(cx, false);
                    self.redraw(cx);
                }
            }
            return;
        }

        if self.show_chat {
            self.handle_chat_actions(cx, event, &actions);
            return;
        }

        if let Event::MouseDown(me) = event {
            if me.button.is_primary() {
                let hit_item = self
                    .row_hit_rects
                    .iter()
                    .find(|(_, rect)| rect.contains(me.abs))
                    .map(|(item_id, _)| *item_id);
                if let Some(item_id) = hit_item {
                    self.selected_index = item_id;
                    self.update_labels(cx, "Selected");
                    let is_double_click = self.last_click_item == Some(item_id)
                        && (me.time - self.last_click_time) <= 0.35;
                    self.last_click_item = Some(item_id);
                    self.last_click_time = me.time;
                    if is_double_click {
                        self.launch_selected(cx);
                    } else {
                        self.redraw(cx);
                    }
                    return;
                }
            }
        }

        if let Event::MouseMove(me) = event {
            let next_hover = self
                .row_hit_rects
                .iter()
                .find(|(_, rect)| rect.contains(me.abs))
                .map(|(item_id, _)| *item_id);
            if next_hover != self.hovered_index {
                self.hovered_index = next_hover;
                self.redraw(cx);
            }
        }

        if let Event::MouseLeave(_) = event {
            if self.hovered_index.is_some() {
                self.hovered_index = None;
                self.redraw(cx);
            }
        }

        let mut handled_enter = false;
        if let Some((text, _mods)) = self
            .view
            .text_input(cx, ids!(mode_input))
            .returned(&actions)
        {
            handled_enter = true;
            self.query = text;
            let q = self.query.trim().to_lowercase();
            if q == "todo" || q == "/todo" {
                self.set_todo_mode(cx, true);
                self.redraw(cx);
                return;
            }
            if q == "chat" || q == "/chat" {
                self.set_chat_mode(cx, true);
                self.redraw(cx);
                return;
            }
            self.rebuild_filter();
            self.launch_selected(cx);
        }

        if let Event::KeyDown(key) = event {
            match key.key_code {
                KeyCode::ArrowDown => {
                    self.step_selection(1);
                    self.ensure_selection_visible(cx);
                    self.update_labels(cx, "Selected");
                    self.redraw(cx);
                }
                KeyCode::ArrowUp => {
                    self.step_selection(-1);
                    self.ensure_selection_visible(cx);
                    self.update_labels(cx, "Selected");
                    self.redraw(cx);
                }
                KeyCode::ReturnKey => {
                    if !handled_enter {
                        self.launch_selected(cx);
                    }
                }
                _ => {}
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.row_hit_rects.clear();
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.borrow_mut::<PortalList>() {
                list.set_item_range(cx, 0, self.filtered_indices.len());

                while let Some(item_id) = list.next_visible_item(cx) {
                    if let Some(source_idx) = self.filtered_indices.get(item_id) {
                        let source_idx = *source_idx;
                        let (
                            app_name,
                            category,
                            subtitle,
                            fallback,
                            is_command,
                            is_builtin,
                            group_name,
                        ) = if let Some(entry) = self.all_items.get(source_idx) {
                            let is_builtin = matches!(
                                entry.launch,
                                LaunchTarget::OpenTodo
                                    | LaunchTarget::OpenChat
                                    | LaunchTarget::OpenSplashApp(_)
                            );
                            (
                                entry.app_name.clone(),
                                entry.category.clone(),
                                entry.subtitle.clone(),
                                entry.icon_fallback.clone(),
                                entry.category == "Command",
                                is_builtin,
                                Self::group_name_for(entry),
                            )
                        } else {
                            continue;
                        };

                        let icon_path = self.resolve_icon_for_index(source_idx);

                        let row = list.item(cx, item_id, live_id!(ResultRow));
                        let show_group = if item_id == 0 {
                            true
                        } else if let Some(prev_source_idx) = self.filtered_indices.get(item_id - 1)
                        {
                            if let Some(prev_entry) = self.all_items.get(*prev_source_idx) {
                                Self::group_name_for(prev_entry) != group_name
                            } else {
                                false
                            }
                        } else {
                            false
                        };
                        row.widget(cx, ids!(group_label))
                            .set_visible(cx, show_group);
                        row.label(cx, ids!(group_label)).set_text(cx, group_name);
                        let group_color = match group_name {
                            "Built-in" => vec4(0.52, 0.69, 1.0, 1.0),
                            "Commands" => vec4(0.95, 0.75, 0.46, 1.0),
                            _ => vec4(0.56, 0.63, 0.73, 1.0),
                        };
                        if let Some(mut label) = row.label(cx, ids!(group_label)).borrow_mut() {
                            label.draw_text.color = group_color;
                        }
                        row.label(cx, ids!(app_name)).set_text(cx, &app_name);
                        row.label(cx, ids!(app_meta)).set_text(cx, &category);
                        row.label(cx, ids!(app_desc)).set_text(cx, &subtitle);
                        row.label(cx, ids!(app_icon_fallback))
                            .set_text(cx, &fallback);
                        let action_text = if is_command { "Run ↩" } else { "Open ↩" };
                        row.label(cx, ids!(action_hint_text))
                            .set_text(cx, action_text);
                        let meta_color = if category == "Command" {
                            vec4(0.96, 0.75, 0.44, 1.0)
                        } else {
                            vec4(0.65, 0.70, 0.78, 1.0)
                        };
                        let meta_chip_fill = if category == "Command" {
                            vec4(0.28, 0.23, 0.17, 1.0)
                        } else {
                            vec4(0.13, 0.16, 0.20, 1.0)
                        };
                        let meta_chip_stroke = if category == "Command" {
                            vec4(0.55, 0.44, 0.27, 1.0)
                        } else {
                            vec4(0.23, 0.29, 0.36, 1.0)
                        };
                        if let Some(mut label) = row.label(cx, ids!(app_meta)).borrow_mut() {
                            label.draw_text.color = meta_color;
                        }
                        if let Some(mut view) = row.view(cx, ids!(app_meta_chip)).borrow_mut() {
                            view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(fill_color),
                                &v4a(meta_chip_fill),
                            );
                            view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(border_color),
                                &v4a(meta_chip_stroke),
                            );
                        }

                        if let Some(path) = icon_path {
                            let loaded = row
                                .image(cx, ids!(app_icon))
                                .load_image_file_by_path(cx, Path::new(&path))
                                .is_ok();
                            row.widget(cx, ids!(app_icon)).set_visible(cx, loaded);
                            row.widget(cx, ids!(app_icon_fallback))
                                .set_visible(cx, !loaded);
                            let icon_bg = if loaded {
                                vec4(0.0, 0.0, 0.0, 0.0)
                            } else {
                                vec4(0.102, 0.122, 0.149, 1.0)
                            };
                            let icon_stroke = if loaded {
                                vec4(0.0, 0.0, 0.0, 0.0)
                            } else {
                                vec4(0.196, 0.227, 0.271, 1.0)
                            };
                            if let Some(mut view) = row.view(cx, ids!(icon_wrap)).borrow_mut() {
                                view.draw_bg.draw_vars.set_dyn_instance(
                                    cx,
                                    live_id!(bg_color),
                                    &v4a(icon_bg),
                                );
                                view.draw_bg.draw_vars.set_dyn_instance(
                                    cx,
                                    live_id!(border_color),
                                    &v4a(icon_stroke),
                                );
                            }
                        } else {
                            row.widget(cx, ids!(app_icon)).set_visible(cx, false);
                            row.widget(cx, ids!(app_icon_fallback))
                                .set_visible(cx, true);
                            if let Some(mut view) = row.view(cx, ids!(icon_wrap)).borrow_mut() {
                                view.draw_bg.draw_vars.set_dyn_instance(
                                    cx,
                                    live_id!(bg_color),
                                    &v4a(vec4(0.102, 0.122, 0.149, 1.0)),
                                );
                                view.draw_bg.draw_vars.set_dyn_instance(
                                    cx,
                                    live_id!(border_color),
                                    &v4a(vec4(0.196, 0.227, 0.271, 1.0)),
                                );
                            }
                        }

                        let selected = if item_id == self.selected_index {
                            1.0
                        } else {
                            0.0
                        };
                        let hovered = if Some(item_id) == self.hovered_index {
                            1.0
                        } else {
                            0.0
                        };
                        row.widget(cx, ids!(action_hint_chip))
                            .set_visible(cx, selected > 0.5 || hovered > 0.5);
                        let hint_bg = if selected > 0.5 {
                            vec4(0.20, 0.30, 0.44, 1.0)
                        } else {
                            vec4(0.15, 0.20, 0.28, 1.0)
                        };
                        let hint_stroke = if selected > 0.5 {
                            vec4(0.45, 0.60, 0.82, 1.0)
                        } else {
                            vec4(0.27, 0.35, 0.49, 1.0)
                        };
                        if let Some(mut view) = row.view(cx, ids!(action_hint_chip)).borrow_mut() {
                            view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(fill_color),
                                &v4a(hint_bg),
                            );
                            view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(border_color),
                                &v4a(hint_stroke),
                            );
                        }
                        let title_color = if selected > 0.5 {
                            vec4(0.98, 0.99, 1.0, 1.0)
                        } else {
                            vec4(0.92, 0.94, 0.98, 1.0)
                        };
                        if let Some(mut label) = row.label(cx, ids!(app_name)).borrow_mut() {
                            label.draw_text.color = title_color;
                        }
                        let desc_color = if selected > 0.5 {
                            vec4(0.86, 0.91, 0.98, 1.0)
                        } else if hovered > 0.5 {
                            vec4(0.67, 0.74, 0.85, 1.0)
                        } else {
                            vec4(0.55, 0.61, 0.68, 1.0)
                        };
                        if let Some(mut label) = row.label(cx, ids!(app_desc)).borrow_mut() {
                            label.draw_text.color = desc_color;
                        }
                        let command = if is_command { 1.0 } else { 0.0 };
                        let builtin = if is_builtin { 1.0 } else { 0.0 };
                        if let Some(mut view) = row.view(cx, ids!(row_bg)).borrow_mut() {
                            view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(selected),
                                &[selected],
                            );
                            view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(hovered),
                                &[hovered],
                            );
                            view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(command),
                                &[command],
                            );
                            view.draw_bg.draw_vars.set_dyn_instance(
                                cx,
                                live_id!(builtin),
                                &[builtin],
                            );
                        }
                        row.draw_all(cx, &mut Scope::empty());
                        if let Some(area) = row
                            .view(cx, ids!(row_bg))
                            .borrow()
                            .map(|v| v.draw_bg.draw_vars.area)
                        {
                            self.row_hit_rects.push((item_id, area.rect(cx)));
                        }
                    }
                }
            }
        }
        DrawStep::done()
    }
}

fn v4a(v: Vec4) -> [f32; 4] {
    [v.x, v.y, v.z, v.w]
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::Startup = event {
            self.ui
                .text_input(cx, ids!(launcher.mode_input))
                .set_key_focus(cx);
        }

        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
