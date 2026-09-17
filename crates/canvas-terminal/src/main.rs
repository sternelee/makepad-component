pub use makepad_widgets;
use makepad_widgets::*;

mod camera;
mod canvas;
mod chat;
mod command;
mod daemon;
mod daemon_persist;
mod ipc;
mod ipc_cli;
mod items;
mod note;
mod persist;
mod terminal;

use canvas::CanvasPanel;

script_mod! {
    use mod.prelude.widgets.*

    mod.tc = {
        bg: #x0d0e12ff
        panel: #x15161cff
        border: #x262a35ff
        text_primary: #xe2e6efff
        text_secondary: #x9aa3b2ff
        accent: #x4d9fff
        input_bg: #x1a1c24ff
    }

    mod.widgets.CanvasPanelBase = #(CanvasPanel::register_widget(vm))
    mod.widgets.CanvasPanel = set_type_default() do mod.widgets.CanvasPanelBase{
        width: Fill
        height: Fill
        flow: Overlay

        draw_grid +: {
            color: #x0d0e12ff
            cam_pan: uniform(vec2(0.0 0.0))
            cam_zoom: uniform(1.0)
            grid_size: uniform(24.0)
            grid_col: uniform(#x1a1c24ff)
            axis_col: uniform(#x2a2e3aff)
            time: uniform(0.0)
            grid_enabled: uniform(1.0)
            pixel: fn() {
                let p = self.pos * self.rect_size
                let pan = self.cam_pan
                let zoom = self.cam_zoom
                let time = self.time
                let g = self.grid_size
                let w = (p - self.rect_size * 0.5) / zoom + pan

                // Pixelate world coords for a retro CNVS feel.
                let px = 3.0
                let wp = floor(w / px) * px

                // Sky gradient (screen-space).
                let sky_t = clamp(self.pos.y, 0.0, 1.0)
                let sky_top = vec3(0.02, 0.04, 0.10)
                let sky_bot = vec3(0.05, 0.08, 0.18)
                let col = mix(sky_top, sky_bot, sky_t)

                // Moon with subtle parallax.
                let moon_pos = vec2(260.0, -160.0)
                let moon_screen = (moon_pos - pan * 0.08) * zoom + self.rect_size * 0.5
                let moon_d = length(p - moon_screen)
                let moon_r = 42.0 * zoom
                let moon = 1.0 - smoothstep(0.0, moon_r, moon_d)
                let moon_glow = (1.0 - smoothstep(moon_r, moon_r * 3.0, moon_d)) * 0.22
                let moon_col = vec3(0.96, 0.95, 0.82)
                let col = mix(col, moon_col, moon * 0.85 + moon_glow)

                // Twinkling stars.
                let star_seed = wp * 0.017
                let star_hash = fract(sin(dot(star_seed, vec2(12.9898, 78.233))) * 43758.5453)
                let star_bright = step(0.994, star_hash)
                let twinkle = 0.5 + 0.5 * sin(time * 2.5 + star_hash * 100.0)
                let star = star_bright * twinkle
                let col = mix(col, vec3(1.0, 1.0, 0.95), star * 0.9)

                // Distant hills.
                let hill_y1 = sin(wp.x * 0.012) * 25.0 + 90.0
                let hill_y2 = sin(wp.x * 0.018 + 2.0) * 18.0 + 130.0
                let hill1 = 1.0 - smoothstep(hill_y1 - 8.0, hill_y1 + 8.0, w.y)
                let hill2 = 1.0 - smoothstep(hill_y2 - 6.0, hill_y2 + 6.0, w.y)
                let col = mix(col, vec3(0.03, 0.06, 0.12), hill1 * 0.7)
                let col = mix(col, vec3(0.02, 0.04, 0.08), hill2 * 0.6)

                // Optional grid overlay.
                let grid_alpha = self.grid_enabled
                let fx = fract(w.x / g) * g
                let fy = fract(w.y / g) * g
                let dx = min(fx, g - fx)
                let dy = min(fy, g - fy)
                let d = min(dx, dy)
                let ga = 1.0 - smoothstep(0.0, 1.5 / zoom, d)
                let axis = 1.0 - smoothstep(0.0, 2.0 / zoom, min(abs(w.x), abs(w.y)))
                let grid_col = self.grid_col
                let axis_col = self.axis_col
                let gcol = mix(grid_col, axis_col, axis)
                let col = mix(col, gcol.xyz, gcol.w * max(ga, axis) * 0.22 * grid_alpha)

                return vec4(col.x, col.y, col.z, 1.0)
            }
        }

        draw_item_bg +: {
            color: #x1a1c24ff
        }
        draw_title +: {
            text_style: theme.font_bold{font_size: 13.0}
            color: #xe2e6efff
        }
        // Note card body: regular for prose, italic for `*emphasis*`. Bold runs
        // reuse `draw_title` and `code` spans reuse `draw_cell_text`, matching
        // the faces the theme already ships.
        draw_note_text +: {
            text_style: theme.font_regular{font_size: 13.0}
            color: #x2a2620ff
        }
        draw_note_italic +: {
            text_style: theme.font_italic{font_size: 13.0}
            color: #x2a2620ff
        }
        draw_cell_bg +: {
            color: #xff0000ff
        }
        draw_cell_text +: {
            text_style: TextStyle{
                font_family: FontFamily{
                    // JetBrains Mono is the face makepad's own terminal ships
                    // on: it covers everything this grid draws — the shell's
                    // ➜ ✗ ❯ ⌘ ▸ ◎ and box-drawing — at exactly one cell of
                    // advance. The old `latin` pointed at
                    // `resources/Menlo-Regular.ttf`, which exists nowhere in
                    // the makepad tree: the family then loaded incompletely and
                    // ASCII fell through to the CJK member, whose proportional
                    // glyphs are wider than a cell (the horizontal squeeze) and
                    // cover no prompt symbol (the tofu boxes).
                    latin := FontMember{
                        res: crate_resource("makepad_widgets:resources/jetbrains_mono_variable.ttf")
                        asc: 0.0 desc: 0.0 weight: 400.0
                    }
                    // Symbols the mono face is missing (✗ ⌘ ⚠ …).
                    symbols := FontMember{res: crate_resource("makepad_widgets:resources/Inter.ttf") asc: 0.0 desc: 0.0}
                    icons := FontMember{res: crate_resource("makepad_widgets:resources/fa-solid-900.ttf") asc: 0.0 desc: 0.0}
                    // `lazy` (1 = CJK, 2 = emoji) keeps the large faces out of
                    // the eager family: they are requested only after a real
                    // glyph miss, so they can never become the Latin fallback.
                    chinese := FontMember{lazy: 1.0 res: crate_resource("makepad_widgets:resources/LXGWWenKaiRegular.ttf") asc: 0.0 desc: 0.0}
                    emoji := FontMember{lazy: 2.0 res: crate_resource("makepad_widgets:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
                }
                font_size: 12.5
                line_spacing: 1.0
            }
            color: #xe2e6efff
        }
        draw_cursor +: {
            color: #x4d9fff
        }
        draw_browser_page +: {
            color: #x29303dff
        }

        // ── Tool palette tooltip bubble (styling mirrors MpTooltip: dark
        // bubble, 1px #374151 border, 6px radius, 12px regular text) ──
        draw_tooltip +: {
            color: #x1f2937ff
            border_color: #x374151ff
            radius: instance(6.0)
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                sdf.fill(self.color)
                sdf.stroke(self.border_color, 1.0)
                return sdf.result
            }
        }
        draw_tooltip_text +: {
            text_style: theme.font_regular{font_size: 12.0}
            color: #xf9fafbff
        }

        // ── Embedded CEF browser slots (absolute-positioned by CanvasPanel) ──
        // Keep them `visible: false` so the overlay flow never lays them out;
        // CanvasPanel toggles visibility and draws each active one with an
        // abs_pos walk at the browser item's screen rect.
        browser_slot_0 := Browser{
            width: Fill
            height: Fill
            backend: BrowserBackend.CEF
            visible: false
            url: "about:blank"
        }
        browser_slot_1 := Browser{
            width: Fill
            height: Fill
            backend: BrowserBackend.CEF
            visible: false
            url: "about:blank"
        }
        browser_slot_2 := Browser{
            width: Fill
            height: Fill
            backend: BrowserBackend.CEF
            visible: false
            url: "about:blank"
        }
        browser_slot_3 := Browser{
            width: Fill
            height: Fill
            backend: BrowserBackend.CEF
            visible: false
            url: "about:blank"
        }

        // ── Dropped-image preview slots (absolute-positioned by CanvasPanel) ──
        // Same pattern as the browser slots: hidden from the overlay flow,
        // drawn manually at each image item's content rect. The Image widget
        // handles async/async-free decoding (png/jpg/webp/gif/bmp/ico/qoi/svg)
        // and aspect-preserving letterboxing (ImageFit.Smallest).
        image_slot_0 := Image{visible: false fit: ImageFit.Smallest width: Fill height: Fill}
        image_slot_1 := Image{visible: false fit: ImageFit.Smallest width: Fill height: Fill}
        image_slot_2 := Image{visible: false fit: ImageFit.Smallest width: Fill height: Fill}
        image_slot_3 := Image{visible: false fit: ImageFit.Smallest width: Fill height: Fill}
        image_slot_4 := Image{visible: false fit: ImageFit.Smallest width: Fill height: Fill}
        image_slot_5 := Image{visible: false fit: ImageFit.Smallest width: Fill height: Fill}
        image_slot_6 := Image{visible: false fit: ImageFit.Smallest width: Fill height: Fill}
        image_slot_7 := Image{visible: false fit: ImageFit.Smallest width: Fill height: Fill}

        // ── Native video preview slots (absolute-positioned by CanvasPanel) ──
        // The Video widget has no `visible` field, so these are declared 0×0
        // to stay invisible in the overlay flow pass; CanvasPanel draws each
        // one with an explicit abs_pos Walk at its item rect (after the child
        // pass, so the widget's recorded area — and thus its controls' hit
        // testing — points at the item rect, not the 0×0 slot).
        //
        // No `autoplay` here: autoplay prepares the player with whatever
        // source the slot was created with, and an empty Filesystem path
        // makes AVPlayerItem return nil (crash). CanvasPanel sets the real
        // source and calls begin_playback once an item claims the slot.
        video_slot_0 := Video{
            width: 0
            height: 0
            source: VideoDataSource.Filesystem{path: ""}
            mute: true
        }
        video_slot_1 := Video{
            width: 0
            height: 0
            source: VideoDataSource.Filesystem{path: ""}
            mute: true
        }

        // ── Native PDF preview slots (absolute-positioned by CanvasPanel) ──
        // The low-level PdfPageView (makepad's own PDF renderer, `pdf`
        // feature): a simple draw-call widget like Video, so it is declared
        // 0×0 and drawn off-flow at the pdf item's page rect (two-phase:
        // draw_walk → render_page → draw_walk, see CanvasPanel::draw_pdf_slot).
        pdf_slot_0 := PdfPageView{width: 0 height: 0}
        pdf_slot_1 := PdfPageView{width: 0 height: 0}

        // ── Hidden inline note editor (IME/text-input sink) ──
        // Kept at zero size and transparent; the canvas draws the note body
        // itself, but this widget owns the focus and IME composition so that
        // multi-byte input and composition events are handled by Makepad's
        // TextInput instead of raw key events.
        note_editor := TextInput{
            width: 0
            height: 0
            is_multiline: true
            empty_text: ""
            draw_text +: { color: #00000000 }
            draw_cursor +: { color: #00000000 }
            draw_selection +: { color: #00000000 }
        }

        // ── Hidden agent composer (IME/text-input sink) ──
        // Same mechanism as note_editor: a zero-size TextInput owns focus and
        // IME composition while the canvas draws the input line itself. Enter
        // submits (prompt, or steer when the agent is running), Escape cancels.
        agent_composer := TextInput{
            width: 0
            height: 0
            is_multiline: false
            empty_text: ""
            draw_text +: { color: #00000000 }
            draw_cursor +: { color: #00000000 }
            draw_selection +: { color: #00000000 }
        }

        // ── Status strip: the app's one line of feedback ──
        // Commands are summoned now (see the palette below), so the docked bar
        // left only this behind: "sent", a card's new name and the shortcut
        // hint, over the canvas' bottom-left corner.
        status_wrap := View{
            width: Fill
            height: Fill
            flow: Down
            align: Align{x: 0.0 y: 1.0}
            padding: Inset{left: 14 bottom: 10}

            status_label := Label{
                width: Fit
                height: Fit
                text: "Ready — ⌘K for commands, drag to move, scroll to pan, ⌘+scroll to zoom"
                draw_text +: {
                    text_style: theme.font_regular{font_size: 11}
                    color: mod.tc.text_secondary
                }
            }
        }

        // ── Command palette (⌘K) ──
        // Commands are summoned, not docked: the canvas keeps the whole window
        // and this scrim is hidden until the shortcut arrives. The panel keeps
        // the docked bar's ids, so the input, the suggestions and the history
        // all still work; only where it appears has changed.
        command_wrap := View{
            width: Fill
            height: Fill
            visible: false
            flow: Down
            align: Align{x: 0.5 y: 0.0}
            padding: Inset{top: 54}
            show_bg: true
            draw_bg +: {
                color: #x05060add
                pixel: fn() {
                    return vec4(self.color.x, self.color.y, self.color.z, self.color.w)
                }
            }

            command_bar := View{
                width: 620
                height: Fit
                flow: Down
                spacing: 8
                show_bg: true
                draw_bg +: {
                    color: #x15161cff
                    border_color: #x2a2e3aff
                    radius: instance(12.0)
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                        sdf.fill(self.color)
                        sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                        sdf.stroke(self.border_color, 1.0)
                        return sdf.result
                    }
                }
                padding: Inset{left: 12 right: 12 top: 12 bottom: 12}

                input_row := View{
                    width: Fill
                    height: Fit
                    flow: Right
                    spacing: 8
                    align: Align{y: 0.5}

                    // ── Search capsule (the docked input, same ids) ──
                    input_capsule := View{
                        width: Fill
                        height: Fit
                        flow: Right
                        spacing: 6
                        align: Align{y: 0.5}
                        padding: Inset{left: 12 right: 10 top: 6 bottom: 6}
                        show_bg: true
                        draw_bg +: {
                            color: #x1a1c24ff
                            border_color: #x2a2e3aff
                            radius: instance(6.0)
                            pixel: fn() {
                                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                                sdf.fill(self.color)
                                sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                                sdf.stroke(self.border_color, 1.0)
                                return sdf.result
                            }
                        }

                        prompt_label := Label{
                            text: "⌘"
                            width: Fit
                            height: Fit
                            draw_text +: {
                                text_style: theme.font_bold{font_size: 14}
                                color: mod.tc.accent
                            }
                        }

                        command_input := TextInput{
                            width: Fill
                            height: 26
                            empty_text: "Search commands — /new terminal, /new note, @card message"
                            // Transparent background/border so the capsule's rounded corners show through.
                            draw_bg +: {
                                color: #x00000000
                                border_color: #x00000000
                            }
                            draw_text +: {
                                text_style: theme.font_regular{font_size: 13}
                                color: mod.tc.text_primary
                            }
                        }
                    }
                }

                // ── Command catalogue ──
                // One label per row, `PALETTE_ROW_H` tall in canvas.rs: that
                // stride is how a press maps back to a row, so the two stay in
                // step (and the panel's own padding is zero for the same
                // reason).
                suggestion_list := View{
                    width: Fill
                    height: Fit
                    flow: Down
                    visible: false
                    suggestion_0 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_1 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_2 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_3 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_4 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_5 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_6 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_7 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_8 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_9 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_10 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_11 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_12 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_13 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_14 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_15 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                    suggestion_16 := Label{width: Fill height: 24 padding: Inset{left: 6 right: 6 top: 4 bottom: 4} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                }
            }
        }


        // ── Right-side properties panel container (CNVS style) ──
        right_panel_container := View{
            width: Fill
            height: Fill
            flow: Down
            align: Align{x: 1.0 y: 0.0}
            padding: Inset{top: 46 right: 12 bottom: 120 left: 0}

            properties_panel := View{
                width: 220
                height: Fill
                flow: Down
                visible: false
                padding: Inset{top: 12 bottom: 12 left: 12 right: 12}
                show_bg: true
                draw_bg +: {
                    color: #x15161cff
                    border_color: #x2a2e3aff
                    radius: instance(12.0)
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.radius)
                        sdf.fill(self.color)
                        sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                        sdf.stroke(self.border_color, 1.0)
                        return sdf.result
                    }
                }

                prop_header := Label{
                text: "Properties"
                width: Fit
                height: Fit
                margin: Inset{bottom: 12}
                draw_text +: {
                    text_style: theme.font_bold{font_size: 14}
                    color: mod.tc.text_primary
                }
            }

            prop_title_label := Label{
                text: "Title"
                width: Fit
                height: Fit
                margin: Inset{bottom: 4}
                draw_text +: {text_style: theme.font_regular{font_size: 11} color: mod.tc.text_secondary}
            }
            prop_title := TextInput{
                width: Fill
                height: 28
                empty_text: "title"
                draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}
            }

            prop_body_label := Label{
                text: "Body"
                width: Fit
                height: Fit
                margin: Inset{top: 10 bottom: 4}
                draw_text +: {text_style: theme.font_regular{font_size: 11} color: mod.tc.text_secondary}
            }
            prop_body := TextInput{
                width: Fill
                height: 80
                empty_text: "note body"
                draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}
            }

            prop_color_label := Label{
                text: "Color"
                width: Fit
                height: Fit
                margin: Inset{top: 10 bottom: 4}
                draw_text +: {text_style: theme.font_regular{font_size: 11} color: mod.tc.text_secondary}
            }
            prop_color_row := View{
                width: Fill
                height: Fit
                flow: Right
                spacing: 6

                prop_color_0 := Button{text: "●" width: 24 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 16} color: #xe2e6efff}}
                prop_color_1 := Button{text: "●" width: 24 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 16} color: #x4d9fffff}}
                prop_color_2 := Button{text: "●" width: 24 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 16} color: #x9ee06fff}}
                prop_color_3 := Button{text: "●" width: 24 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 16} color: #xf2bf47ff}}
                prop_color_4 := Button{text: "●" width: 24 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 16} color: #xf27a51ff}}
                prop_color_5 := Button{text: "●" width: 24 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 16} color: #xe664b2ff}}
            }

            prop_font_label := Label{
                text: "Font size"
                width: Fit
                height: Fit
                margin: Inset{top: 10 bottom: 4}
                draw_text +: {text_style: theme.font_regular{font_size: 11} color: mod.tc.text_secondary}
            }
            prop_font_row := View{
                width: Fill
                height: Fit
                flow: Right
                spacing: 6

                prop_font_0 := Button{text: "11" width: 32 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 10} color: mod.tc.text_primary}}
                prop_font_1 := Button{text: "13" width: 32 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 11} color: mod.tc.text_primary}}
                prop_font_2 := Button{text: "16" width: 32 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                prop_font_3 := Button{text: "20" width: 32 height: 24 draw_text +: {text_style: theme.font_regular{font_size: 13} color: mod.tc.text_primary}}
            }
        }
    }
}

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(1400.0 900.0)
                window.title: "Canvas Terminal"
                pass +: { clear_color: #x0d0e12ff }
                body +: {
                    canvas := mod.widgets.CanvasPanel{}
                }
            }
        }
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
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::WindowCloseRequested(_) | Event::WindowClosed(_) = event {
            // The canvas layout is the user's work: persist it on the way
            // out, best-effort.
            if let Some(panel) = self
                .ui
                .widget(cx, ids!(main_window.body.canvas))
                .borrow_mut::<CanvasPanel>()
            {
                panel.save_canvas();
            }
        }
        if let Event::Startup = event {
            // No input holds the keyboard at startup — the palette is summoned
            // — so the canvas takes it on its first draw, and a restored card
            // sees typing without a click first.
            if let Some(mut panel) = self
                .ui
                .widget(cx, ids!(main_window.body.canvas))
                .borrow_mut::<CanvasPanel>()
            {
                panel.set_grid_enabled(false);
                // Restore the saved canvas: re-attach live sessions, relaunch
                // agent CLIs against their persisted conversations, rebuild
                // plain cards, camera and whiteboard. Falls back to a fresh
                // default terminal on first run.
                panel.restore_canvas(cx);
            }
        }
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // `canvas-terminal --daemon` runs the bundled PTY daemon (no GUI). The
    // GUI spawns itself in this mode detached so terminal sessions survive
    // GUI restarts; see `terminal::session::ensure_daemon`.
    if args.iter().any(|a| a == "--daemon") {
        std::process::exit(match daemon::run() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("canvas-terminal daemon: {e}");
                1
            }
        });
    }
    // `canvas-terminal ipc <verb>` is the inter-agent/script control surface:
    // a one-shot CLI client over the daemon protocol, so a CLI agent running
    // inside a card (see `ipc::ENV_MARKER_VAR`) can message, wait on, or
    // read a sibling card without linking against the GUI. See `ipc_cli`.
    if args.len() > 1 && args[1] == "ipc" {
        std::process::exit(ipc_cli::run(&args[2..]));
    }
    app_main();
}

/// Fonts this app ships, as a `makepad.font-assets.v1` payload. The
/// `app_main!` macro emits this section for crates that can use the macro;
/// this crate hand-rolls its entry point for the CEF bootstrap, so the
/// manifest is spelled out here: the cell face, its symbol fallbacks, and the
/// two large faces that load only after a glyph miss.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_env = "ohos")))]
const CANVAS_FONT_ASSETS: &[&str] = &[
    "makepad_widgets/resources/jetbrains_mono_variable.ttf",
    "makepad_widgets/resources/fa-solid-900.ttf",
    "makepad_widgets/resources/Inter.ttf",
    "makepad_widgets/resources/LXGWWenKaiRegular.ttf",
    "makepad_widgets/resources/NotoColorEmoji.ttf",
];

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_env = "ohos")))]
#[used]
#[cfg_attr(target_vendor = "apple", link_section = "__DATA,__mp_font_v1")]
#[cfg_attr(not(target_vendor = "apple"), link_section = ".makepad.font-assets.v1")]
static MAKEPAD_FONT_ASSETS_V1: [u8;
    makepad_widgets::makepad_platform::font_policy::font_asset_manifest_len(
        makepad_widgets::makepad_platform::font_policy::INTERNATIONAL_FONT_ASSET_MANIFEST,
        CANVAS_FONT_ASSETS,
    )] = makepad_widgets::makepad_platform::font_policy::extend_font_asset_manifest(
    makepad_widgets::makepad_platform::font_policy::INTERNATIONAL_FONT_ASSET_MANIFEST,
    CANVAS_FONT_ASSETS,
);

/// Desktop entry point with CEF bootstrap (embedded browser support).
///
/// Mirrors `app_main!` but inserts the makepad-cef bootstrap/initialize
/// steps, and calls `makepad_cef::shutdown()` after the event loop exits.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_env = "ohos")))]
pub fn app_main() {
    // CEF needs a proper app bundle (Chromium frameworks + ICU data). If we
    // are running from target/debug (plain executable), re-exec into a
    // synthetic bundle that contains the CEF distribution.
    if let Err(err) = makepad_cef::reexec_into_app_bundle_if_needed() {
        log!("canvas-terminal: CEF bundle re-exec failed: {err}");
        std::process::exit(1);
    }

    match makepad_cef::bootstrap() {
        Ok(makepad_cef::BootstrapResult::Continue) => {}
        Ok(makepad_cef::BootstrapResult::Exit(code)) => std::process::exit(code),
        Err(err) => {
            log!("canvas-terminal: CEF bootstrap failed: {err}");
            std::process::exit(1);
        }
    }

    Cx::init_log();
    if Cx::pre_start() {
        return;
    }

    if let Err(err) = makepad_cef::initialize() {
        log!("canvas-terminal: CEF initialize failed: {err}");
        std::process::exit(1);
    }

    // Same startup/live-edit handling as the `app_main!` macro, with the
    // `App` instance owned by the event closure.
    let app = std::rc::Rc::new(std::cell::RefCell::new(None::<App>));
    let app_value: std::rc::Rc<std::cell::RefCell<Option<ScriptObjectRef>>> =
        std::rc::Rc::new(std::cell::RefCell::new(None));
    let cx = std::rc::Rc::new(std::cell::RefCell::new(Cx::new(Box::new(
        move |cx: &mut Cx, event: &Event| {
            if let Event::Startup = event {
                *app.borrow_mut() = Some(cx.with_vm(|vm| {
                    let value = <App as AppMain>::script_mod(vm);
                    if let Some(obj) = value.as_object() {
                        *app_value.borrow_mut() = Some(vm.heap_mut().new_object_ref(obj));
                    }
                    let mut app = <App as ScriptNew>::script_from_value(vm, value);
                    <App as AppMain>::after_new_from_script(vm, &mut app);
                    app
                }));
                cx.start_hot_reload_file_observer_if_requested();
            }
            if let Event::LiveEdit = event {
                let mut app_ref = app.borrow_mut();
                if let Some(app) = app_ref.as_mut() {
                    cx.with_vm(|vm| {
                        let value = vm.with_reload(<App as AppMain>::script_mod);
                        if let Some(obj) = value.as_object() {
                            *app_value.borrow_mut() = Some(vm.heap_mut().new_object_ref(obj));
                        }
                        <App as ScriptApply>::script_apply(
                            app,
                            vm,
                            &Apply::Reload,
                            &mut Scope::empty(),
                            value,
                        );
                    });
                }
            }
            if let Some(app) = &mut *app.borrow_mut() {
                <dyn AppMain>::handle_event(app, cx, event);
            }
        },
    ))));
    let studio_http = makepad_widgets::resolve_studio_http();
    cx.borrow_mut().init_websockets(&studio_http);
    if makepad_widgets::should_run_stdin_loop_from_env() {
        cx.borrow_mut().in_makepad_studio = true;
    }
    cx.borrow_mut().init_cx_os();
    // `--remote`: a localhost HTTP control surface for agents / tests.
    //
    // `app_main!` calls this for you, but this crate hand-rolls its entry
    // point to insert the CEF bootstrap/initialize steps, so the call has to
    // be repeated here — without it `canvas-terminal --remote` silently
    // exposes no control surface at all. Same placement as the macro: after
    // `init_cx_os`, before the event loop.
    makepad_widgets::makepad_platform::remote::start_if_requested();
    Cx::event_loop(cx);
    makepad_cef::shutdown();
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_env = "ohos"))]
pub fn app_main() {
    panic!("canvas-terminal is desktop-only");
}
