use makepad_component::widgets::button::*;
use makepad_widgets::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

mod a2ui_bridge_embed;
mod chat;
mod todo;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use makepad_component::a2ui::surface::widget::*;
    use makepad_component::widgets::button::*;
    use makepad_component::widgets::checkbox::*;
    use makepad_component::widgets::progress::*;

    pub LauncherPanel = {{LauncherPanel}} {
        width: Fill,
        height: Fill,
        flow: Down,
        spacing: 12,
        padding: {left: 16, right: 16, top: 14, bottom: 14},
        show_bg: true,
	                    draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 13.0);
                sdf.fill(#x181a1f);
                sdf.stroke(#x2f343d, 1.0);
                return sdf.result;
            }
        }

        mode_input_row = <View> {
            width: Fill,
            height: Fit,
            flow: Right,
            align: {y: 0.5},
            spacing: 8,
            margin: {top: 10},

            mode_back_wrap = <View> {
                visible: false,
                width: Fit,
                height: Fit,
                mode_back_btn = <MpButtonSecondary> {
                    text: "‹ Back"
                    padding: {left: 10, right: 10, top: 7, bottom: 7}
                }
            }

            mode_input = <TextInput> {
            width: Fill,
            height: Fit,
            empty_text: "Search applications and commands…",
            padding: {left: 12, right: 12, top: 10, bottom: 10},
            draw_bg: {
                instance border_color: #x3e4653,
                instance focus: 0.0,
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 10.0);
                    let fill_base = #x1f2329;
                    let fill_focus = #x242a33;
                    let border_focus = #x6ea9ff;
                    sdf.fill(mix(fill_base, fill_focus, self.focus * 0.65));
                    sdf.stroke(mix(self.border_color, border_focus, self.focus), 1.0);
                    return sdf.result;
                }
            }
            draw_text: {
                text_style: <THEME_FONT_REGULAR> {font_size: 13},
                color: #xf4f6fb
            }
            }

            mode_action_wrap = <View> {
                visible: false,
                width: Fit,
                height: Fit,
                mode_action_btn = <MpButtonPrimary> {
                    text: "Add"
                    padding: {left: 12, right: 12, top: 7, bottom: 7}
                }
            }

            mode_hint_label = <Label> {
                text: "",
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> {font_size: 10},
                    color: #x8f9caf
                }
            }
        }

        launcher_view = <View> {
            width: Fill,
            height: Fill,
            flow: Down,
            spacing: 10,

            <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: 2,
                <Label> {
                    text: "Launcher",
                    draw_text: {
                        text_style: <THEME_FONT_BOLD> {font_size: 17},
                        color: #xf9fbff
                    }
                }
                <Label> {
                    text: "Quickly open apps and run commands",
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 10},
                        color: #x8f9caf
                    }
	                        }
	                    }

	            top_count_row = <View> {
	                        visible: false,
	                        width: Fill,
	                        height: Fit,
		            flow: Right,
	            align: {y: 0.5},
	            padding: {left: 10, right: 10, top: 6, bottom: 6},
	            show_bg: true,
            draw_bg: {
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 7.0);
                    sdf.fill(#x1b2028);
                    sdf.stroke(#x303745, 1.0);
                    return sdf.result;
                }
            }
	            result_count = <Label> {
	                text: "",
	                draw_text: {
	                    text_style: <THEME_FONT_REGULAR> {font_size: 11},
	                    color: #xa7b0c1
	                }
	            }
	            }

            empty_state = <View> {
                visible: false,
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: 6,
                padding: {left: 14, right: 14, top: 14, bottom: 14},
                show_bg: true,
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 9.0);
                        sdf.fill(#x1b2028);
                        sdf.stroke(#x303745, 1.0);
                        return sdf.result;
                    }
                }
                empty_title = <Label> {
                    text: "No Results",
                    draw_text: {
                        text_style: <THEME_FONT_BOLD> {font_size: 13},
                        color: #xe6ebf5
                    }
                }
                empty_desc = <Label> {
                    text: "Try another keyword, or use /todo and /chat",
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 11},
                        color: #x8f9caf
                    }
                }
            }

            results = <PortalList> {
            width: Fill,
            height: Fill,
            flow: Down,
            spacing: 1,

            ResultRow = <View> {
                width: Fill,
                height: Fit,
                margin: {top: 2, bottom: 2},

	                row_bg = <View> {
	                    width: Fill,
	                    height: Fit,
	                    flow: Down,
	                    spacing: 2,
	                    padding: {left: 11, right: 11, top: 8, bottom: 8},
                    show_bg: true,
                    draw_bg: {
                        instance selected: 0.0
                        instance hovered: 0.0
                        instance command: 0.0
                        instance builtin: 0.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 7.0);
                            let base = #x242a32;
                            let hover = #x2b333f;
                            let command_tint = #x3a3530;
                            let builtin_tint = #x2d3240;
                            let tinted = mix(base, command_tint, self.command * 0.42);
                            let tinted = mix(tinted, builtin_tint, self.builtin * 0.62);
                            let active = mix(#x3a79de, #x4f7de0, self.builtin);
                            let hovered_mix = mix(base, hover, self.hovered);
                            let hover_tinted = mix(hovered_mix, tinted, max(self.command, self.builtin) * 0.32);
                            sdf.fill(mix(hover_tinted, active, self.selected));
                            let stroke_color = mix(#x323a45, #x4b5d78, self.hovered);
                            let stroke_color = mix(stroke_color, #x8f7044, self.command * 0.5);
                            let stroke_color = mix(stroke_color, #x6177a1, self.builtin * 0.6);
                            sdf.stroke(mix(stroke_color, #x79adff, self.selected), 1.0);

                            return sdf.result;
	                        }
	                    }

	                    group_label = <Label> {
	                        visible: false,
	                        text: "Applications",
	                        margin: {bottom: 2},
	                        draw_text: {
	                            text_style: <THEME_FONT_BOLD> {font_size: 9},
	                            color: #x8fa0ba
	                        }
	                    }

	                    <View> {
	                        width: Fill,
	                        height: Fit,
	                        flow: Right,
	                        align: {y: 0.5},
	                        spacing: 9,

	                        icon_wrap = <View> {
	                            width: 24,
	                            height: 24,
	                            flow: Overlay,
	                            align: {x: 0.5, y: 0.5},
	                            show_bg: true,
	                            draw_bg: {
	                                instance bg_color: #x1a1f26
	                                instance border_color: #x323a45
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 6.0);
                                    sdf.fill(self.bg_color);
                                    sdf.stroke(self.border_color, 1.0);
                                    return sdf.result;
                                }
                            }

                            app_icon = <Image> {
                                width: 22,
                                height: 22,
                                fit: Smallest,
                            }

                            app_icon_fallback = <Label> {
                                text: "A",
                                draw_text: {
                                    text_style: <THEME_FONT_BOLD> {font_size: 12},
                                    color: #xdce3ee
                                }
                            }
                        }

                        app_name = <Label> {
                            width: Fill,
                            text: "App",
                            draw_text: {
                                text_style: <THEME_FONT_BOLD> {font_size: 13},
                                color: #xfffdff
                            }
                        }

	                        app_meta_chip = <View> {
	                            width: Fit,
	                            height: Fit,
	                            padding: {left: 7, right: 7, top: 2, bottom: 2},
                            show_bg: true,
                            draw_bg: {
                                instance color: #x202733
                                instance border_color: #x3b4658
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 5.0);
                                    sdf.fill(self.color);
                                    sdf.stroke(self.border_color, 1.0);
                                    return sdf.result;
                                }
                            }
                            app_meta = <Label> {
                                text: "Category",
                                draw_text: {
                                    text_style: <THEME_FONT_REGULAR> {font_size: 9},
                                    color: #xa7b2c5
                                }
	                            }
	                        }

	                        action_hint_chip = <View> {
	                            visible: false,
	                            width: Fit,
	                            height: Fit,
	                            padding: {left: 7, right: 7, top: 2, bottom: 2},
	                            show_bg: true,
	                            draw_bg: {
	                                instance color: #x253043
	                                instance border_color: #x435a7f
	                                fn pixel(self) -> vec4 {
	                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
	                                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 5.0);
	                                    sdf.fill(self.color);
	                                    sdf.stroke(self.border_color, 1.0);
	                                    return sdf.result;
	                                }
	                            }
	                            action_hint_text = <Label> {
	                                text: "Open ↩",
	                                draw_text: {
	                                    text_style: <THEME_FONT_REGULAR> {font_size: 9},
	                                    color: #xbfd2ef
	                                }
	                            }
	                        }
	                    }

	                    app_desc = <Label> {
	                        width: Fill,
	                        text: "Description",
                        draw_text: {
                            text_style: <THEME_FONT_REGULAR> {font_size: 10},
                            color: #x8d9bae
                        }
                    }
                }
            }
            }

            <View> {
            visible: true,
            width: Fill,
            height: Fit,
            flow: Right,
            align: {y: 0.5},
            spacing: 8,
            padding: {left: 10, right: 10, top: 6, bottom: 6},
            show_bg: true,
            draw_bg: {
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 7.0);
                    sdf.fill(#x1b2028);
                    sdf.stroke(#x303745, 1.0);
                    return sdf.result;
                }
            }
            status_label = <Label> {
                width: Fill,
                text: "",
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> {font_size: 10},
                    color: #x7fb9ff
                }
            }
            status_keys_label = <Label> {
                text: "Up/Down Select  |  Enter Open  |  Double Click Open",
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> {font_size: 10},
                    color: #x8f9caf
                }
            }
            }
            }

        todo_view = <View> {
            visible: false,
            width: Fill,
            height: Fill,
            flow: Down,
            spacing: 10,
            padding: {left: 2, right: 2, top: 2, bottom: 2},

            <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                align: {y: 0.5},
                spacing: 8,

                <Label> {
                    text: "Todo List",
                    draw_text: {
                        text_style: <THEME_FONT_BOLD> {font_size: 17},
                        color: #xf9fbff
                    }
                }
            }

            <Label> {
                text: "Capture quick tasks for this workspace",
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> {font_size: 10},
                    color: #x8f9caf
                }
            }

            stats_card = <View> {
                width: Fill,
                height: Fit,
                flow: Down,
                spacing: 8,
                padding: {left: 10, right: 10, top: 10, bottom: 10},
                show_bg: true,
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);
                        sdf.fill(#x1b2028);
                        sdf.stroke(#x303745, 1.0);
                        return sdf.result;
                    }
                }

                todo_count_label = <Label> {
                    text: "0 / 0 done",
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 10},
                        color: #xa7b4c8
                    }
                }

                todo_progress = <MpProgressSuccess> {
                    width: Fill,
                    height: 6,
                    value: 0.0,
                }
            }

            <View> { width: Fill, height: 1, show_bg: true, draw_bg: {color: #x2e3541} }

            todo_rows = <View> {
                width: Fill,
                height: Fill,
                flow: Down,
                spacing: 5,

                row_0 = <View> { width: Fill, height: Fit, flow: Right, spacing: 8, align: {y: 0.5}, visible: false, padding: {left: 8, right: 8, top: 8, bottom: 8}, show_bg: true,
                    draw_bg: { instance done: 0.0 fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0); sdf.fill(mix(#x272c34, #x1f3a2a, self.done)); sdf.stroke(#x3b424d, 1.0); return sdf.result; } }
                    check_0 = <MpCheckbox> { text: "" }
                    label_0 = <Label> { width: Fill, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 12}, color: #xe2e8f0 } }
                    del_0 = <MpButtonGhost> { text: "Remove", padding: {left: 8, right: 8, top: 6, bottom: 6} }
                }
                row_1 = <View> { width: Fill, height: Fit, flow: Right, spacing: 8, align: {y: 0.5}, visible: false, padding: {left: 8, right: 8, top: 8, bottom: 8}, show_bg: true,
                    draw_bg: { instance done: 0.0 fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0); sdf.fill(mix(#x272c34, #x1f3a2a, self.done)); sdf.stroke(#x3b424d, 1.0); return sdf.result; } }
                    check_1 = <MpCheckbox> { text: "" }
                    label_1 = <Label> { width: Fill, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 12}, color: #xe2e8f0 } }
                    del_1 = <MpButtonGhost> { text: "Remove", padding: {left: 8, right: 8, top: 6, bottom: 6} }
                }
                row_2 = <View> { width: Fill, height: Fit, flow: Right, spacing: 8, align: {y: 0.5}, visible: false, padding: {left: 8, right: 8, top: 8, bottom: 8}, show_bg: true,
                    draw_bg: { instance done: 0.0 fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0); sdf.fill(mix(#x272c34, #x1f3a2a, self.done)); sdf.stroke(#x3b424d, 1.0); return sdf.result; } }
                    check_2 = <MpCheckbox> { text: "" }
                    label_2 = <Label> { width: Fill, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 12}, color: #xe2e8f0 } }
                    del_2 = <MpButtonGhost> { text: "Remove", padding: {left: 8, right: 8, top: 6, bottom: 6} }
                }
                row_3 = <View> { width: Fill, height: Fit, flow: Right, spacing: 8, align: {y: 0.5}, visible: false, padding: {left: 8, right: 8, top: 8, bottom: 8}, show_bg: true,
                    draw_bg: { instance done: 0.0 fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0); sdf.fill(mix(#x272c34, #x1f3a2a, self.done)); sdf.stroke(#x3b424d, 1.0); return sdf.result; } }
                    check_3 = <MpCheckbox> { text: "" }
                    label_3 = <Label> { width: Fill, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 12}, color: #xe2e8f0 } }
                    del_3 = <MpButtonGhost> { text: "Remove", padding: {left: 8, right: 8, top: 6, bottom: 6} }
                }
                row_4 = <View> { width: Fill, height: Fit, flow: Right, spacing: 8, align: {y: 0.5}, visible: false, padding: {left: 8, right: 8, top: 8, bottom: 8}, show_bg: true,
                    draw_bg: { instance done: 0.0 fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0); sdf.fill(mix(#x272c34, #x1f3a2a, self.done)); sdf.stroke(#x3b424d, 1.0); return sdf.result; } }
                    check_4 = <MpCheckbox> { text: "" }
                    label_4 = <Label> { width: Fill, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 12}, color: #xe2e8f0 } }
                    del_4 = <MpButtonGhost> { text: "Remove", padding: {left: 8, right: 8, top: 6, bottom: 6} }
                }
                row_5 = <View> { width: Fill, height: Fit, flow: Right, spacing: 8, align: {y: 0.5}, visible: false, padding: {left: 8, right: 8, top: 8, bottom: 8}, show_bg: true,
                    draw_bg: { instance done: 0.0 fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0); sdf.fill(mix(#x272c34, #x1f3a2a, self.done)); sdf.stroke(#x3b424d, 1.0); return sdf.result; } }
                    check_5 = <MpCheckbox> { text: "" }
                    label_5 = <Label> { width: Fill, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 12}, color: #xe2e8f0 } }
                    del_5 = <MpButtonGhost> { text: "Remove", padding: {left: 8, right: 8, top: 6, bottom: 6} }
                }
                row_6 = <View> { width: Fill, height: Fit, flow: Right, spacing: 8, align: {y: 0.5}, visible: false, padding: {left: 8, right: 8, top: 8, bottom: 8}, show_bg: true,
                    draw_bg: { instance done: 0.0 fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0); sdf.fill(mix(#x272c34, #x1f3a2a, self.done)); sdf.stroke(#x3b424d, 1.0); return sdf.result; } }
                    check_6 = <MpCheckbox> { text: "" }
                    label_6 = <Label> { width: Fill, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 12}, color: #xe2e8f0 } }
                    del_6 = <MpButtonGhost> { text: "Remove", padding: {left: 8, right: 8, top: 6, bottom: 6} }
                }
                row_7 = <View> { width: Fill, height: Fit, flow: Right, spacing: 8, align: {y: 0.5}, visible: false, padding: {left: 8, right: 8, top: 8, bottom: 8}, show_bg: true,
                    draw_bg: { instance done: 0.0 fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0); sdf.fill(mix(#x272c34, #x1f3a2a, self.done)); sdf.stroke(#x3b424d, 1.0); return sdf.result; } }
                    check_7 = <MpCheckbox> { text: "" }
                    label_7 = <Label> { width: Fill, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 12}, color: #xe2e8f0 } }
                    del_7 = <MpButtonGhost> { text: "Remove", padding: {left: 8, right: 8, top: 6, bottom: 6} }
                }
            }
        }

        chat_view = <View> {
            visible: false,
            width: Fill,
            height: Fill,
            flow: Down,
            spacing: 10,
            padding: {left: 2, right: 2, top: 2, bottom: 2},

            <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                align: {y: 0.5},
                spacing: 8,

                <Label> {
                    width: Fill,
                    text: "A2UI Chat",
                    draw_text: {
                        text_style: <THEME_FONT_BOLD> {font_size: 17},
                        color: #xf9fbff
                    }
                }
                chat_reset_btn = <MpButtonGhost> {
                    text: "Clear"
                    padding: {left: 10, right: 10, top: 7, bottom: 7}
                }
            }

            <Label> {
                text: "Ask naturally or render A2UI UI from responses",
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> {font_size: 10},
                    color: #x8f9caf
                }
            }

            <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                align: {y: 0.5},
                spacing: 8,
                padding: {left: 10, right: 10, top: 8, bottom: 8},
                show_bg: true,
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);
                        sdf.fill(#x1b2028);
                        sdf.stroke(#x303745, 1.0);
                        return sdf.result;
                    }
                }
                <Label> {
                    text: "LLM API:",
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 10},
                        color: #xa7b4c8
                    }
                }
                chat_server_input = <TextInput> {
                    width: 280,
                    height: Fit,
                    empty_text: "https://openrouter.ai/api/v1/chat/completions",
                    padding: {left: 10, right: 10, top: 7, bottom: 7},
                    draw_bg: {
                        instance border_color: #x3e4653,
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);
                            sdf.fill(#x1f2329);
                            sdf.stroke(self.border_color, 1.0);
                            return sdf.result;
                        }
                    }
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 11},
                        color: #xe2e8f0
                    }
                }
                chat_model_input = <TextInput> {
                    width: 160,
                    height: Fit,
                    empty_text: "kimi-k2.5",
                    padding: {left: 10, right: 10, top: 7, bottom: 7},
                    draw_bg: {
                        instance border_color: #x3e4653,
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);
                            sdf.fill(#x1f2329);
                            sdf.stroke(self.border_color, 1.0);
                            return sdf.result;
                        }
                    }
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 11},
                        color: #xe2e8f0
                    }
                }
            }

            chat_history_scroll = <ScrollYView> {
                width: Fill,
                height: 120,
                show_bg: true,
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);
                        sdf.fill(#x1b2028);
                        sdf.stroke(#x303745, 1.0);
                        return sdf.result;
                    }
                }
                <View> {
                    width: Fill,
                    height: Fit,
                    padding: {left: 10, right: 10, top: 10, bottom: 10}
                    chat_history_label = <Label> {
                        width: Fill,
                        text: "",
                        draw_text: {
                            text_style: <THEME_FONT_REGULAR> {font_size: 11},
                            color: #xd3d9e6
                            wrap: Word
                        }
                    }
                }
            }

            <View> {
                width: Fill,
                height: Fit,
                flow: Right,
                align: {y: 0.5},
                spacing: 8,
                padding: {left: 10, right: 10, top: 6, bottom: 6},
                show_bg: true,
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 7.0);
                        sdf.fill(#x1b2028);
                        sdf.stroke(#x303745, 1.0);
                        return sdf.result;
                    }
                }
                chat_status_label = <Label> {
                    width: Fill,
                    text: "Ready",
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 10},
                        color: #x7fb9ff
                    }
                }
                chat_keys_label = <Label> {
                    text: "Enter Send  |  Esc Back",
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 10},
                        color: #x8f9caf
                    }
                }
            }

            chat_surface_wrap = <ScrollYView> {
                width: Fill,
                height: Fill,
                show_bg: true,
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 8.0);
                        sdf.fill(#x1b2028);
                        sdf.stroke(#x303745, 1.0);
                        return sdf.result;
                    }
                }
                <View> {
                    width: Fill,
                    height: Fit,
                    padding: {left: 10, right: 10, top: 10, bottom: 10}
                    chat_surface = <A2uiSurface> {
                        width: Fill,
                        height: Fit,
                    }
                }
            }
        }
    }

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                width: Fill,
                height: Fill,
                show_bg: true,
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let center = vec2(0.5, 0.5);
                        let d = distance(self.pos, center);
                        let t = clamp(d * 1.35, 0.0, 1.0);
                        return mix(#x0f1115, #x20242b, t);
                    }
                }

                body = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Down,

                    launcher = <LauncherPanel> {}
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

#[derive(Live, Widget)]
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
    todos: Vec<todo::TodoItem>,
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
}

impl LiveHook for LauncherPanel {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
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
        self.todos = todo::default_todos();
        self.show_chat = false;
        self.chat_messages = chat::default_chat_messages();
        self.chat_loading = false;
        self.chat_server_url = std::env::var("LLM_API_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".to_string());
        self.chat_model =
            std::env::var("LLM_MODEL").unwrap_or_else(|_| "openrouter/auto".to_string());
        self.chat_api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
        self.rebuild_filter();
        self.sync_mode_input(cx);
        self.view.text_input(ids!(mode_input)).set_key_focus(cx);
        self.sync_todo_ui(cx);
        self.sync_chat_ui(cx);
        self.view
            .text_input(ids!(chat_server_input))
            .set_text(cx, &self.chat_server_url);
        self.view
            .text_input(ids!(chat_model_input))
            .set_text(cx, &self.chat_model);
        self.update_labels(cx, "Ready");
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

fn load_launcher_items() -> Vec<LauncherItem> {
    let mut items = scan_macos_applications();
    if items.is_empty() {
        items = fallback_demo_apps();
    }

    items.extend(command_items());
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
                "Ask for UI, e.g. 'Create a task dashboard with charts'",
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

        self.view
            .view(ids!(mode_input_row))
            .apply_over(cx, live! { spacing: (row_spacing) });
        self.view
            .text_input(ids!(mode_input))
            .apply_over(cx, live! { empty_text: (empty_text) });
        self.view.text_input(ids!(mode_input)).set_text(cx, text);
        self.view
            .text_input(ids!(mode_input))
            .set_is_read_only(cx, read_only);
        self.view
            .widget(ids!(mode_back_wrap))
            .set_visible(cx, show_back);
        self.view
            .widget(ids!(mode_back_btn))
            .set_visible(cx, show_back);
        let action_btn = self.view.mp_button(ids!(mode_action_btn));
        self.view
            .widget(ids!(mode_action_wrap))
            .set_visible(cx, show_action);
        self.view
            .widget(ids!(mode_action_btn))
            .set_visible(cx, show_action);
        action_btn.set_text(action_text);
        action_btn.set_disabled(cx, action_disabled);
        self.view
            .label(ids!(mode_hint_label))
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
                .map(|it| matches!(it.launch, LaunchTarget::OpenTodo | LaunchTarget::OpenChat))
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

    fn group_name_for(item: &LauncherItem) -> &'static str {
        match item.launch {
            LaunchTarget::OpenTodo | LaunchTarget::OpenChat => "Built-in",
            LaunchTarget::Command { .. } => "Commands",
            LaunchTarget::OpenPath(_) => "Applications",
        }
    }

    fn update_labels(&mut self, cx: &mut Cx, _status_hint: &str) {
        let has_results = !self.filtered_indices.is_empty();
        self.view.widget(ids!(results)).set_visible(cx, has_results);
        self.view.widget(ids!(empty_state)).set_visible(cx, !has_results);
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
                self.selected_index.saturating_add(1).min(self.filtered_indices.len())
            )
        };
        let count_text = format!(
            "{} result(s)  |  {}  |  {} app / {} cmd",
            self.filtered_indices.len(),
            selected_text,
            app_count,
            command_count
        );
        self.view.label(ids!(result_count)).set_text(cx, "");
        if has_results {
            self.view
                .label(ids!(empty_title))
                .set_text(cx, "No Results");
            self.view
                .label(ids!(empty_desc))
                .set_text(cx, "Try another keyword, or use /todo and /chat");
        } else {
            let q = self.query.trim();
            if q.is_empty() {
                self.view
                    .label(ids!(empty_title))
                    .set_text(cx, "Start Searching");
                self.view
                    .label(ids!(empty_desc))
                    .set_text(cx, "Type app or command name. Try: todo, chat, terminal");
            } else {
                self.view
                    .label(ids!(empty_title))
                    .set_text(cx, "No Results");
                self.view.label(ids!(empty_desc)).set_text(
                    cx,
                    &format!("No match for \"{}\". Try /todo or /chat", q),
                );
            }
        }

        self.view.label(ids!(status_label)).set_text(cx, &count_text);
        self.view
            .label(ids!(status_keys_label))
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
        }

        self.redraw(cx);
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

        if let Some(text) = self.view.text_input(ids!(mode_input)).changed(&actions) {
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
        if let Some((text, _mods)) = self.view.text_input(ids!(mode_input)).returned(&actions) {
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
                    self.update_labels(cx, "Selected");
                    self.redraw(cx);
                }
                KeyCode::ArrowUp => {
                    self.step_selection(-1);
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
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.filtered_indices.len());

                while let Some(item_id) = list.next_visible_item(cx) {
	                    if let Some(source_idx) = self.filtered_indices.get(item_id) {
	                        let source_idx = *source_idx;
	                        let (app_name, category, subtitle, fallback, is_command, is_builtin, group_name) =
                            if let Some(entry) = self.all_items.get(source_idx) {
                                let is_builtin = matches!(
                                    entry.launch,
                                    LaunchTarget::OpenTodo | LaunchTarget::OpenChat
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
	                        } else if let Some(prev_source_idx) = self.filtered_indices.get(item_id - 1) {
	                            if let Some(prev_entry) = self.all_items.get(*prev_source_idx) {
	                                Self::group_name_for(prev_entry) != group_name
	                            } else {
	                                false
	                            }
	                        } else {
	                            false
	                        };
	                        row.widget(ids!(group_label)).set_visible(cx, show_group);
	                        row.label(ids!(group_label)).set_text(cx, group_name);
	                        let group_color = match group_name {
	                            "Built-in" => vec4(0.52, 0.69, 1.0, 1.0),
	                            "Commands" => vec4(0.95, 0.75, 0.46, 1.0),
	                            _ => vec4(0.56, 0.63, 0.73, 1.0),
	                        };
	                        row.label(ids!(group_label)).apply_over(
	                            cx,
	                            live! {
	                                draw_text: { color: (group_color) }
	                            },
	                        );
	                        row.label(ids!(app_name)).set_text(cx, &app_name);
	                        row.label(ids!(app_meta)).set_text(cx, &category);
	                        row.label(ids!(app_desc)).set_text(cx, &subtitle);
	                        row.label(ids!(app_icon_fallback)).set_text(cx, &fallback);
	                        let action_text = if is_command { "Run ↩" } else { "Open ↩" };
	                        row.label(ids!(action_hint_text)).set_text(cx, action_text);
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
	                        row.label(ids!(app_meta)).apply_over(
	                            cx,
	                            live! {
	                                draw_text: { color: (meta_color) }
	                            },
	                        );
	                        row.view(ids!(app_meta_chip)).apply_over(
	                            cx,
	                            live! {
	                                draw_bg: {
	                                    color: (meta_chip_fill),
	                                    border_color: (meta_chip_stroke)
	                                }
	                            },
	                        );

	                        if let Some(path) = icon_path {
	                            let loaded = row
	                                .image(ids!(app_icon))
	                                .load_image_file_by_path(cx, Path::new(&path))
	                                .is_ok();
	                            row.widget(ids!(app_icon)).set_visible(cx, loaded);
	                            row.widget(ids!(app_icon_fallback)).set_visible(cx, !loaded);
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
	                            row.view(ids!(icon_wrap)).apply_over(
	                                cx,
	                                live! {
	                                    draw_bg: { bg_color: (icon_bg), border_color: (icon_stroke) }
	                                },
	                            );
	                        } else {
	                            row.widget(ids!(app_icon)).set_visible(cx, false);
	                            row.widget(ids!(app_icon_fallback)).set_visible(cx, true);
	                            row.view(ids!(icon_wrap)).apply_over(
	                                cx,
	                                live! {
	                                    draw_bg: {
	                                        bg_color: (vec4(0.102, 0.122, 0.149, 1.0)),
	                                        border_color: (vec4(0.196, 0.227, 0.271, 1.0))
	                                    }
	                                },
	                            );
	                        }

	                        let selected = if item_id == self.selected_index { 1.0 } else { 0.0 };
	                        let hovered = if Some(item_id) == self.hovered_index {
	                            1.0
	                        } else {
	                            0.0
	                        };
	                        row.widget(ids!(action_hint_chip))
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
	                        row.view(ids!(action_hint_chip)).apply_over(
	                            cx,
	                            live! {
	                                draw_bg: { color: (hint_bg), border_color: (hint_stroke) }
	                            },
	                        );
	                        let title_color = if selected > 0.5 {
	                            vec4(0.98, 0.99, 1.0, 1.0)
	                        } else {
	                            vec4(0.92, 0.94, 0.98, 1.0)
	                        };
	                        row.label(ids!(app_name)).apply_over(
	                            cx,
	                            live! {
	                                draw_text: { color: (title_color) }
	                            },
	                        );
	                        let desc_color = if selected > 0.5 {
	                            vec4(0.86, 0.91, 0.98, 1.0)
	                        } else if hovered > 0.5 {
	                            vec4(0.67, 0.74, 0.85, 1.0)
	                        } else {
	                            vec4(0.55, 0.61, 0.68, 1.0)
	                        };
	                        row.label(ids!(app_desc)).apply_over(
	                            cx,
	                            live! {
	                                draw_text: { color: (desc_color) }
	                            },
	                        );
	                        let command = if is_command { 1.0 } else { 0.0 };
	                        let builtin = if is_builtin { 1.0 } else { 0.0 };
	                        row.view(ids!(row_bg)).apply_over(
	                            cx,
	                            live! {
	                                draw_bg: {selected: (selected), hovered: (hovered), command: (command), builtin: (builtin)}
	                            },
	                        );
                        row.draw_all(cx, &mut Scope::empty());
                        self.row_hit_rects
                            .push((item_id, row.view(ids!(row_bg)).area().rect(cx)));
                    }
                }
            }
        }
        DrawStep::done()
    }
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        makepad_component::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::Startup = event {
            self.ui
                .text_input(ids!(launcher.mode_input))
                .set_key_focus(cx);
        }

        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

fn main() {
    app_main()
}
