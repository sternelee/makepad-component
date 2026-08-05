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
script_mod! {
    use mod.prelude.widgets.*
    use mod.draw.MacosWindowChrome
    use mod.draw.MacosWindowConfig

    // ── tinycast Theme 移植令牌（单一来源；着色器内用 uniform(mod.tc.*) 引用）──
    mod.tc = {
        surface: #x101013ff          // 面板表面（black 0.40 叠极暗底）#xRRGGBBAA
        hairline: #xffffff14         // white 0.08 面板外边框
        selection: #xffffff1a        // white 0.10 选中行
        row_hover: #xffffff0d        // white 0.05 悬停行
        control_surface: #xffffff1a  // white 0.10 填充 keycap
        border: #xffffff33           // white 0.20 描边 keycap
        text_primary: #xfffffff2     // white 0.95
        text_secondary: #xffffff99   // white 0.60
        text_tertiary: #xffffff66    // white 0.40
        glass_top: #xffffff24        // white 0.14
        glass_bottom: #xffffff10     // white 0.06
        glass_stroke: #xffffff38     // white 0.22
        separator: #xffffff1a        // white 0.10
        scrollbar: #xffffff4d        // white 0.30
    }

    mod.widgets.KeyCap = View{
        width: 22
        height: 20
        align: Center
        padding: Inset{left: 7 right: 7}
        show_bg: true
        draw_bg +: {
            filled: uniform(1.0)
            fill_col: uniform(mod.tc.control_surface)
            stroke_col: uniform(mod.tc.border)
            pixel: fn() {
                // 解析圆角矩形 SDF（不用 Sdf2d.box，其半径行为不可靠）
                let p = self.pos * self.rect_size
                let r = 6.0
                let qx = max(abs(p.x - self.rect_size.x * 0.5) - (self.rect_size.x * 0.5 - r), 0.0)
                let qy = max(abs(p.y - self.rect_size.y * 0.5) - (self.rect_size.y * 0.5 - r), 0.0)
                let d = length(vec2(qx qy)) - r
                let fill_a = clamp(-d, 0.0, 1.0)
                let band = min(clamp((d + 1.0) * 2.0, 0.0, 1.0), clamp(-d * 2.0, 0.0, 1.0))
                if self.filled > 0.5 {
                    let a = self.fill_col.w * fill_a
                    return vec4(self.fill_col.xyz * a, a)
                } else {
                    let a = self.stroke_col.w * band
                    return vec4(self.stroke_col.xyz * a, a)
                }
            }
        }
        caption := Label{
            text: ""
            draw_text +: {
                text_style: theme.font_bold {font_size: 10}
                color: mod.tc.text_secondary
            }
        }
    }
    mod.widgets.KeyCapOutline = mod.widgets.KeyCap{
        draw_bg +: { filled: uniform(0.0) }
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
                }
            }
        }
    }

    mod.widgets.LauncherPanelBase = #(LauncherPanel::register_widget(vm))
    mod.widgets.LauncherPanel = set_type_default() do mod.widgets.LauncherPanelBase{
        width: 750
        height: 475
        flow: Overlay
        show_bg: true
        draw_bg +: {
            surface_col: uniform(mod.tc.surface)
            hairline_col: uniform(mod.tc.hairline)
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 26.0)
                sdf.fill(self.surface_col)
                sdf.stroke(self.hairline_col 1.0)
                return sdf.result
            }
        }



        launcher_view := View{
            width: Fill
            height: Fill
            flow: Overlay

            empty_state := View{
                visible: false
                width: Fill
                height: Fill
                flow: Down
                align: Center
                spacing: 6

                empty_title := Label{
                    text: "No Results"
                    draw_text +: {
                        text_style: theme.font_bold {font_size: 13}
                        color: mod.tc.text_secondary
                    }
                }
                empty_desc := Label{
                    text: "Try another keyword, or use /todo and /chat"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 11}
                        color: mod.tc.text_tertiary
                    }
                }
            }

            results := PortalList{
                width: Fill
                height: Fill
                flow: Down
                spacing: 0
                margin: Inset{top: 44 bottom: 34}

                scroll_bar := ScrollBar{
                    draw_bg +: {
                        size: uniform(2.0)
                        border_radius: uniform(1.0)
                        color: uniform(mod.tc.scrollbar)
                        color_hover: uniform(#xffffff6b)   // white 0.42
                        color_drag: uniform(#xffffff80)   // white 0.50
                    }
                }

                ResultRow := View{
                    width: Fill
                    height: Fit
                    margin: Inset{top: 2 bottom: 2 left: 8 right: 8}

                    row_bg := View{
                        width: Fill
                        height: Fit
                        flow: Down
                        spacing: 4
                        padding: Inset{left: 8 right: 8 top: 6 bottom: 6}
                        show_bg: true
                        draw_bg +: {
                            selected: instance(0.0)
                            hovered: instance(0.0)
                            sel_col: uniform(mod.tc.selection)
                            hov_col: uniform(mod.tc.row_hover)
                            pixel: fn() {
                                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                                sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 10.0)
                                let c = mix(#x00000000 self.hov_col self.hovered)
                                let c = mix(c self.sel_col self.selected)
                                sdf.fill(c)
                                return sdf.result
                            }
                        }

                        group_label := Label{
                            text: "Applications"
                            margin: Inset{bottom: 4}
                            draw_text +: {
                                text_style: theme.font_bold {font_size: 11}
                                color: mod.tc.text_secondary
                            }
                        }

                        View{
                            width: Fill
                            height: Fit
                            flow: Right
                            align: VCenter
                            spacing: 10

                            icon_wrap := View{
                                width: 24
                                height: 24
                                flow: Overlay
                                align: Center

                                app_icon := Image{
                                    width: 24
                                    height: 24
                                    fit: ImageFit.Smallest
                                }

                                app_icon_fallback := Label{
                                    text: "A"
                                    draw_text +: {
                                        text_style: theme.font_bold {font_size: 12}
                                        color: mod.tc.text_secondary
                                    }
                                }
                            }

                            app_name := Label{
                                width: Fill
                                text: "App"
                                draw_text +: {
                                    text_style: theme.font_regular {font_size: 13}
                                    color: mod.tc.text_primary
                                }
                            }

                            app_meta := Label{
                                text: "Category"
                                draw_text +: {
                                    text_style: theme.font_regular {font_size: 11}
                                    color: mod.tc.text_tertiary
                                }
                            }
                        }
                    }
                }
            }

            dissolve_top := View{
                width: Fill
                height: 76
                show_bg: true
                draw_bg +: {
                    surface_col: uniform(mod.tc.surface)
                    pixel: fn() {
                        // 预乘 alpha：顶部不透明 → 底部透明
                        let a = 1.0 - self.pos.y
                        let c = self.surface_col
                        // 圆角遮罩：面板顶部两角 radius 26，避免直角盖住圆角
                        let p = self.pos * self.rect_size
                        let r = 26.0
                        let e = max(max(r - p.x, p.x - (self.rect_size.x - r)), 0.0)
                        let f = max(r - p.y, 0.0)
                        let d = length(vec2(e f)) - r
                        let m = clamp(-d, 0.0, 1.0)
                        let aa = a * m
                        return vec4(c.x * aa c.y * aa c.z * aa aa)
                    }
                }
            }

            dissolve_bottom := View{
                width: Fill
                height: Fill
                show_bg: true
                draw_bg +: {
                    surface_col: uniform(mod.tc.surface)
                    pixel: fn() {
                        // 预乘 alpha：仅底部 80pt 参与渐变，向上透明
                        let py = self.pos.y * self.rect_size.y
                        let a = clamp((py - (self.rect_size.y - 80.0)) / 80.0 0.0 1.0)
                        let c = self.surface_col
                        // 圆角遮罩：面板底部两角 radius 26
                        let p = self.pos * self.rect_size
                        let r = 26.0
                        let e = max(max(r - p.x, p.x - (self.rect_size.x - r)), 0.0)
                        let f = max(p.y - (self.rect_size.y - r), 0.0)
                        let d = length(vec2(e f)) - r
                        let m = clamp(-d, 0.0, 1.0)
                        let aa = a * m
                        return vec4(c.x * aa c.y * aa c.z * aa aa)
                    }
                }
            }
        }

        todo_view := View{
            visible: false
            width: Fill
            height: Fill
            flow: Down
            spacing: 0
            padding: Inset{left: 2 right: 2 top: 2 bottom: 2}

            // ── App nav header ──────────────────────────────────────────────
            splash_header := View{
                width: Fill
                height: Fit
                flow: Right
                align: VCenter
                spacing: 8
                padding: Inset{left: 12 right: 12 top: 8 bottom: 8}
                show_bg: true
                draw_bg +: {
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 0.0)
                        sdf.fill(#x151c26)
                        sdf.stroke(#x2a3341 1.0)
                        return sdf.result
                    }
                }

                app_name_label := Label{
                    width: Fill
                    text: "App"
                    draw_text +: {
                        text_style: theme.font_bold {font_size: 15}
                        color: #xf1f5f9
                    }
                }

                View{
                    width: Fit
                    height: Fit
                    padding: Inset{left: 7 right: 7 top: 3 bottom: 3}
                    show_bg: true
                    draw_bg +: {
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 4.0)
                            sdf.fill(#x1d3048)
                            sdf.stroke(#x2a4a70 1.0)
                            return sdf.result
                        }
                    }
                    Label{
                        text: "Splash"
                        draw_text +: {
                            text_style: theme.font_bold {font_size: 9}
                            color: #x60a5fa
                        }
                    }
                }
            }

            View{width: Fill height: 1 show_bg: true draw_bg +: {color: #x2e3541}}

            todo_list := Splash{
                width: Fill
                height: Fill
            }
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
                new_batch: true
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
                chat_keys_label := Label{
                    text: "Enter Send  |  Esc Back"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 10}
                        color: #x8f9caf
                    }
                }
            }

            chat_action_bar := View{
                visible: false
                width: Fill
                height: Fit
                flow: Right
                align: VCenter
                spacing: 8
                padding: Inset{left: 10 right: 10 top: 4 bottom: 6}
                show_bg: true
                new_batch: true
                draw_bg +: {
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 7.0)
                        sdf.fill(#x131a24)
                        sdf.stroke(#x1d4ed8 1.0)
                        return sdf.result
                    }
                }
                Label{
                    text: "App ready:"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 10}
                        color: #x60a5fa
                    }
                }
                save_app_wrap := View{
                    visible: false
                    width: Fit
                    height: Fit
                    save_app_btn := Button{
                        text: "Save as App"
                        padding: Inset{left: 10 right: 10 top: 5 bottom: 5}
                        draw_bg +: { color: #x1d4ed8 radius: 5.0 }
                        draw_text +: { color: #xffffff }
                    }
                }
                open_app_wrap := View{
                    visible: false
                    width: Fit
                    height: Fit
                    open_app_btn := Button{
                        text: "Open App"
                        padding: Inset{left: 10 right: 10 top: 5 bottom: 5}
                        draw_bg +: { color: #x22c55e radius: 5.0 }
                        draw_text +: { color: #xffffff }
                    }
                }
            }

        }
        mode_input_row := View{
            width: Fill
            height: 44
            flow: Right
            align: VCenter
            spacing: 8
            padding: Inset{left: 16 right: 16}

            mode_back_wrap := View{
                visible: false
                width: Fit
                height: Fit
                mode_back_btn := Button{
                    width: 24
                    height: 24
                    text: ""
                    draw_bg +: {
                        glyph_col: uniform(mod.tc.text_secondary)
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            // 返回 chevron ‹
                            sdf.move_to(14.0 5.0)
                            sdf.line_to(8.0 12.0)
                            sdf.line_to(14.0 19.0)
                            sdf.stroke(self.glyph_col 1.6)
                            return sdf.result
                        }
                    }
                }
            }

            search_glyph := View{
                width: 20
                height: 24
                show_bg: true
                draw_bg +: {
                    glyph_col: uniform(mod.tc.text_secondary)
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        // 放大镜
                        sdf.circle(8.5 9.5 5.5)
                        sdf.stroke(self.glyph_col 1.6)
                        sdf.move_to(12.8 13.8)
                        sdf.line_to(17.0 18.0)
                        sdf.stroke(self.glyph_col 1.6)
                        return sdf.result
                    }
                }
            }

            mode_input := TextInput{
                width: Fill
                height: Fit
                empty_text: "Search for apps and commands..."
                draw_bg +: {
                    pixel: fn() {
                        return #x00000000
                    }
                }
                draw_text +: {
                    text_style: theme.font_regular {font_size: 20}
                    color: mod.tc.text_primary
                }
                draw_selection +: {
                    color: mod.tc.selection
                }
            }

            mode_action_wrap := View{
                visible: false
                width: Fit
                height: Fit
                mode_action_btn := Button{
                    text: "Send"
                    padding: Inset{left: 12 right: 12 top: 6 bottom: 6}
                    draw_bg +: {
                        top_col: uniform(mod.tc.glass_top)
                        bot_col: uniform(mod.tc.glass_bottom)
                        stroke_col: uniform(mod.tc.glass_stroke)
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            let r = self.rect_size.y * 0.5
                            sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y r)
                            let g = mix(self.top_col self.bot_col self.pos.y)
                            sdf.fill(g)
                            sdf.stroke(self.stroke_col 1.0)
                            return sdf.result
                        }
                    }
                    draw_text +: {
                        text_style: theme.font_bold {font_size: 12}
                        color: mod.tc.text_primary
                    }
                }
            }
        }
        bottom_bar := View{
            width: Fill
            height: 52
            flow: Right
            align: VCenter
            spacing: 10
            padding: Inset{left: 12 right: 12}
            margin: Inset{top: 423}   // 475 - 52，Overlay 钉底

            menu_circle := View{
                width: 36
                height: 36
                flow: Overlay
                align: Center
                show_bg: true
                draw_bg +: {
                    top_col: uniform(mod.tc.glass_top)
                    bot_col: uniform(mod.tc.glass_bottom)
                    stroke_col: uniform(mod.tc.glass_stroke)
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.circle(self.rect_size.x * 0.5 self.rect_size.y * 0.5 18.0)
                        let g = mix(self.top_col self.bot_col self.pos.y)
                        sdf.fill(g)
                        sdf.stroke(self.stroke_col 1.0)
                        return sdf.result
                    }
                }
                Label{
                    text: "···"
                    draw_text +: {
                        text_style: theme.font_bold {font_size: 12}
                        color: mod.tc.text_secondary
                    }
                }
            }

            status_label := Label{
                width: Fill
                text: ""
                draw_text +: {
                    text_style: theme.font_regular {font_size: 11}
                    color: mod.tc.text_tertiary
                }
            }

            action_capsule := View{
                width: Fit
                height: 34
                flow: Right
                align: VCenter
                spacing: 8
                padding: Inset{left: 14 right: 14}
                show_bg: true
                draw_bg +: {
                    top_col: uniform(mod.tc.glass_top)
                    bot_col: uniform(mod.tc.glass_bottom)
                    stroke_col: uniform(mod.tc.glass_stroke)
                    pixel: fn() {
                        // 解析体育场形 SDF（不用 Sdf2d.box，其半径行为不可靠）
                        let p = self.pos * self.rect_size
                        let r = self.rect_size.y * 0.5
                        let cx = min(max(p.x, r), self.rect_size.x - r)
                        let d = length(vec2(p.x - cx, p.y - r)) - r
                        let fill_a = clamp(-d, 0.0, 1.0)
                        let band = min(clamp((d + 1.0) * 2.0, 0.0, 1.0), clamp(-d * 2.0, 0.0, 1.0))
                        let g = mix(self.top_col self.bot_col self.pos.y)
                        // 顶部内侧高光
                        let hl = pow(1.0 - self.pos.y 3.0) * 0.12
                        let g = g + vec4(hl hl hl hl)
                        let a1 = g.w * fill_a
                        let a2 = self.stroke_col.w * band
                        let a = a1 + a2 * (1.0 - a1)
                        let rgb = (g.xyz * a1 + self.stroke_col.xyz * a2 * (1.0 - a1)) / max(a, 0.0001)
                        return vec4(rgb * a, a)
                    }
                }

                primary_action_label := Label{
                    text: "Open Application"
                    draw_text +: {
                        text_style: theme.font_bold {font_size: 12}
                        color: mod.tc.text_primary
                    }
                }

                mod.widgets.KeyCap{ caption := Label{ text: "↵" } }

                View{
                    width: 1
                    height: 16
                    show_bg: true
                    draw_bg +: { color: mod.tc.separator }
                }

                Label{
                    text: "Actions"
                    draw_text +: {
                        text_style: theme.font_regular {font_size: 12}
                        color: mod.tc.text_secondary
                    }
                }

                mod.widgets.KeyCapOutline{ caption := Label{ text: "⌘" } }
                mod.widgets.KeyCapOutline{ caption := Label{ text: "K" } }
            }
        }
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                // 真透明无边框窗口：窗口即面板，真实桌面透出，只剩一层圆角
                window.inner_size: vec2(750 475)
                window.title: "Raycast Launcher"
                window.transparent: true
                window.macos: MacosWindowConfig{chrome: MacosWindowChrome.Borderless}
                pass +: { clear_color: #x00000000 }
                body +: {
                    launcher := mod.widgets.LauncherPanel{}
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
    OpenChat,
    OpenSplashApp(String),
}

#[derive(Clone)]
struct LauncherItem {
    app_name: String,
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
    splash_reload_version: u32,
    #[rust]
    last_todo_version: i64,
    #[rust]
    splash_has_search: bool,
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
        vm.with_cx_mut(|cx| {
            self.all_items = load_launcher_items();
            self.query.clear();
            self.chat_draft.clear();
            self.selected_index = 0;
            self.icon_cache.clear();
            self.row_hit_rects.clear();
            self.hovered_index = None;
            self.last_click_item = None;
            self.last_click_time = 0.0;
            self.show_todo = false;
            self.show_chat = false;
            self.splash_reload_version = 0;
            self.last_todo_version = 0;
            self.splash_has_search = false;
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
            self.sync_chat_ui(cx);
            self.view
                .text_input(cx, ids!(chat_server_input))
                .set_text(cx, &self.chat_server_url);
            self.view
                .text_input(cx, ids!(chat_model_input))
                .set_text(cx, &self.chat_model);
            self.update_labels(cx, "Ready");
        });
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
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some(app) = app_loader::load_app_descriptor(name) else {
                continue;
            };
            if app.app.name.trim().is_empty() || app.splash_code.trim().is_empty() {
                continue;
            }
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
        let (empty_text, text, read_only, show_back, show_action, action_text, action_disabled) =
            if self.show_todo && self.splash_has_search {
                ("Search tasks...", "", false, true, false, "Add", false)
            } else if self.show_chat {
                (
                    "Describe a UI to create",
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
                )
            } else {
                (
                    "Search for apps and commands...",
                    self.query.as_str(),
                    false,
                    false,
                    false,
                    "Add",
                    false,
                )
            };

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
        // 子模式显示 chevron，launcher 模式显示放大镜
        self.view
            .widget(cx, ids!(search_glyph))
            .set_visible(cx, !show_back);
        let action_btn = self.view.button(cx, ids!(mode_action_btn));
        self.view
            .widget(cx, ids!(mode_action_wrap))
            .set_visible(cx, show_action);
        self.view
            .widget(cx, ids!(mode_action_btn))
            .set_visible(cx, show_action);
        action_btn.set_text(cx, action_text);
        action_btn.set_disabled(cx, action_disabled);
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
                        LaunchTarget::OpenChat | LaunchTarget::OpenSplashApp(_)
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
            LaunchTarget::OpenChat => "Built-in",
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

        if !has_results {
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
            .set_text(cx, &format!("{} results", self.filtered_indices.len()));

        // 动作胶囊主动作随选中项类别切换
        let action_text = match self.selected_item() {
            Some(item) if item.category == "Command" => "Run Command",
            Some(_) => "Open Application",
            None => "Open Application",
        };
        self.view
            .label(cx, ids!(primary_action_label))
            .set_text(cx, action_text);
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
            LaunchTarget::OpenChat => {
                self.set_chat_mode(cx, true);
                self.update_labels(cx, "Opened");
            }
            LaunchTarget::OpenSplashApp(ref path) => {
                self.open_splash_app(cx, path);
                self.update_labels(cx, "Opened");
            }
        }

        self.redraw(cx);
    }

    // ==================== Splash App Methods ====================

    fn default_todo_app_path() -> String {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("app-todo.json")
            .to_string_lossy()
            .to_string()
    }

    fn open_splash_app(&mut self, cx: &mut Cx, path: &str) {
        self.show_todo = true;
        self.show_chat = false;
        self.view
            .view(cx, ids!(launcher_view))
            .set_visible(cx, false);
        self.view.view(cx, ids!(todo_view)).set_visible(cx, true);
        self.view.view(cx, ids!(chat_view)).set_visible(cx, false);
        self.sync_mode_input(cx);
        self.load_splash_app(cx, path);
        self.redraw(cx);
        self.view.text_input(cx, ids!(mode_input)).set_key_focus(cx);
    }

    fn refresh_todo_stats(&mut self, _cx: &mut Cx) {
        // Stats card removed — no-op kept for call-site compatibility
    }

    fn set_app_nav_header(&mut self, cx: &mut Cx, app_name: &str) {
        self.view
            .label(cx, ids!(app_name_label))
            .set_text(cx, app_name);
    }

    fn save_current_splash_app(&mut self, cx: &mut Cx) {
        let Some(path) = self.last_saved_app_path.clone() else {
            return;
        };
        match app_loader::save_app_state(cx, &path) {
            Ok(()) => {
                self.refresh_todo_stats(cx);
                self.update_labels(cx, "Saved");
            }
            Err(error) => {
                log!("Failed to save splash app state: {}", error);
                self.update_labels(cx, "Save failed");
            }
        }
        self.redraw(cx);
    }

    fn load_splash_app(&mut self, cx: &mut Cx, path: &str) {
        let Some(app) = app_loader::load_app_descriptor(path) else {
            log!("Failed to load app descriptor: {}", path);
            return;
        };

        self.last_saved_app_path = Some(path.to_string());
        app_loader::inject_app_state(cx, &app.state);
        // Read initial version from injected state
        self.last_todo_version = app_loader::read_todo_version(cx);
        // Detect if this app has a search field (enables mode_input search)
        self.splash_has_search = app.state.get("search").is_some();
        // Sync mode_input hint with search capability
        self.sync_mode_input(cx);

        // Append version comment to force re-evaluation on reload cycles
        self.splash_reload_version += 1;
        let code = format!(
            "{}\n// reload:{}",
            app.splash_code.trim(),
            self.splash_reload_version
        );
        self.view.widget(cx, ids!(todo_list)).set_text(cx, &code);
        self.set_app_nav_header(cx, &app.app.name);
        self.redraw(cx);
    }

    /// Force re-evaluate the Splash widget using current VM state (after a state change).
    fn reload_splash_widget(&mut self, cx: &mut Cx) {
        let path = match &self.last_saved_app_path {
            Some(p) => p.clone(),
            None => return,
        };
        let Some(app) = app_loader::load_app_descriptor(&path) else {
            return;
        };
        // Increment version so set_text detects a change and re-evaluates
        self.splash_reload_version += 1;
        let code = format!(
            "{}\n// reload:{}",
            app.splash_code.trim(),
            self.splash_reload_version
        );
        // Don't re-inject state — VM already has updated state from Splash on_click handlers
        self.view.widget(cx, ids!(todo_list)).set_text(cx, &code);
        // mode_input is outside the Splash widget, so it never loses focus on rebuild.
        // For apps with search, ensure mode_input stays focused after reload.
        if self.splash_has_search {
            self.view.text_input(cx, ids!(mode_input)).set_key_focus(cx);
        }
        self.redraw(cx);
    }

    pub(crate) fn set_todo_mode(&mut self, cx: &mut Cx, show: bool) {
        if show {
            let path = Self::default_todo_app_path();
            self.open_splash_app(cx, &path);
        } else {
            self.show_todo = false;
            self.view
                .view(cx, ids!(launcher_view))
                .set_visible(cx, true);
            self.view.view(cx, ids!(todo_view)).set_visible(cx, false);
            self.view.view(cx, ids!(chat_view)).set_visible(cx, false);
            self.sync_mode_input(cx);
            self.view.text_input(cx, ids!(mode_input)).set_key_focus(cx);
        }
    }

    fn persist_todo_app_if_needed(&mut self, cx: &mut Cx, actions: &Actions) {
        // Version-change detection + reload is now handled at the top of handle_event
        // (before capture_actions) so on_click closures have already been executed.
        // This function is kept for any future use but does nothing currently.
        let _ = (cx, actions);
    }

    pub(crate) fn handle_todo_actions(&mut self, cx: &mut Cx, actions: &Actions) -> bool {
        // Back button
        if self.view.button(cx, ids!(mode_back_btn)).clicked(actions) {
            self.set_todo_mode(cx, false);
            self.redraw(cx);
            return true;
        }
        false
    }
}

impl Widget for LauncherPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // ── Todo: detect Splash on_click state changes ──────────────────────────
        // Splash on_click closures are queued async and executed by the script pump
        // AFTER the current handle_event returns. By checking version HERE (before
        // capture_actions), we catch changes from the *previous* event's script pump.
        if self.show_todo && self.last_saved_app_path.is_some() {
            let v = app_loader::read_todo_version(cx);
            if v != self.last_todo_version {
                self.last_todo_version = v;
                self.save_current_splash_app(cx);
                self.reload_splash_widget(cx);
            }
        }

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
                            // Advance by up to 3 Unicode characters (not bytes).
                            // Slicing by raw bytes panics on multi-byte chars (e.g. CJK = 3 bytes).
                            let safe_end = self.stream_buffer[current_len..]
                                .char_indices()
                                .nth(2) // take up to 3 chars (index 0,1,2 → nth(2))
                                .map(|(i, c)| current_len + i + c.len_utf8())
                                .unwrap_or(target_len)
                                .min(target_len);
                            msg.text
                                .push_str(&self.stream_buffer[current_len..safe_end]);
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
            if self.show_chat {
                self.chat_draft = text;
            } else if !self.show_todo {
                self.query = text;
                self.selected_index = 0;
                self.rebuild_filter();
                self.update_labels(cx, "Filtered");
            }
            self.redraw(cx);
        }

        if self.show_todo {
            let handled = self.handle_todo_actions(cx, &actions);
            if handled {
                return;
            }
            // Search via mode_input — mode_input is OUTSIDE the Splash widget so it
            // never loses focus when the Splash rebuild happens on version change.
            if self.splash_has_search {
                if let Some(text) = self.view.text_input(cx, ids!(mode_input)).changed(&actions) {
                    app_loader::set_splash_search(cx, &text);
                    self.reload_splash_widget(cx);
                    return;
                }
            }
            self.persist_todo_app_if_needed(cx, &actions);
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
                        let (app_name, category, fallback, is_command, group_name) =
                            if let Some(entry) = self.all_items.get(source_idx) {
                                (
                                    entry.app_name.clone(),
                                    entry.category.clone(),
                                    entry.icon_fallback.clone(),
                                    entry.category == "Command",
                                    Self::group_name_for(entry),
                                )
                            } else {
                                continue;
                            };

                        let icon_path = self.resolve_icon_for_index(source_idx);

                        let row = list.item(cx, item_id, live_id!(ResultRow));

                        // section header：组切换时显示；非首个组 header 上方加 12pt 间距
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
                        if let Some(mut v) = row.view(cx, ids!(row_bg)).borrow_mut() {
                            v.walk.margin.top = if show_group && item_id != 0 {
                                12.0
                            } else {
                                0.0
                            };
                        }

                        row.label(cx, ids!(app_name)).set_text(cx, &app_name);
                        row.label(cx, ids!(app_meta)).set_text(cx, &category);
                        row.label(cx, ids!(app_icon_fallback))
                            .set_text(cx, &fallback);

                        if let Some(path) = icon_path {
                            let loaded = row
                                .image(cx, ids!(app_icon))
                                .load_image_file_by_path(cx, Path::new(&path))
                                .is_ok();
                            row.widget(cx, ids!(app_icon)).set_visible(cx, loaded);
                            row.widget(cx, ids!(app_icon_fallback))
                                .set_visible(cx, !loaded);
                        } else {
                            row.widget(cx, ids!(app_icon)).set_visible(cx, false);
                            row.widget(cx, ids!(app_icon_fallback))
                                .set_visible(cx, true);
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
                        }
                        row.draw_all(cx, &mut Scope::empty());
                        if let Some(area) = row
                            .view(cx, ids!(row_bg))
                            .borrow()
                            .map(|v| v.draw_bg.draw_vars.area)
                        {
                            self.row_hit_rects.push((item_id, area.rect(cx)));
                        }

                        let _ = is_command; // 类别差异仅靠行尾文本表达
                    }
                }
            }
        }
        DrawStep::done()
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        crate::makepad_widgets::script_mod(vm);
        self::script_mod(vm);

        // Create mod.state on the modules root object (what Splash `mod` resolves to).
        // vm.module(id!(mod)) returns ZERO because "mod" is a scope variable,
        // not a registered module name. The scope variable points to heap.modules.
        let heap = vm.heap_mut();
        let mod_obj = heap.modules;
        let state_obj = heap.new_object();
        heap.set_value_def(mod_obj, ScriptValue::from_id(id!(state)), state_obj.into());

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
