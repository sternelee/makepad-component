pub use makepad_widgets;
use makepad_widgets::*;

mod camera;
mod canvas;
mod command;
mod items;
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
        draw_cell_bg +: {
            color: #xff0000ff
        }
        draw_cell_text +: {
            text_style: TextStyle{
                font_family: FontFamily{
                    // Menlo is monospace AND covers Dingbats (➜✗⌘);
                    // CJK + emoji fall back to the bundled fonts.
                    latin := FontMember{res: crate_resource("makepad_widgets:resources/Menlo-Regular.ttf") asc: 0.0 desc: 0.0}
                    chinese := FontMember{res: crate_resource("makepad_widgets:resources/LXGWWenKaiRegular.ttf") asc: 0.0 desc: 0.0}
                    emoji := FontMember{res: crate_resource("makepad_widgets:resources/NotoColorEmoji.ttf") asc: 0.0 desc: 0.0}
                }
                font_size: 12.5
                line_spacing: 1.2
            }
            color: #xe2e6efff
        }
        draw_cursor +: {
            color: #x4d9fff
        }
        draw_browser_page +: {
            color: #x29303dff
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

        // ── Unified command bar (bottom overlay) ──
        command_wrap := View{
            width: Fill
            height: Fill
            flow: Down
            align: Align{y: 1.0}

            command_bar := View{
                width: Fill
                height: Fit
                flow: Down
                show_bg: true
                draw_bg +: {
                    color: #x15161cff
                    pixel: fn() {
                        let p = self.pos * self.rect_size
                        let d = self.rect_size.y - p.y
                        let a = clamp(d / 1.0, 0.0, 1.0)
                        return vec4(self.color.x, self.color.y, self.color.z, self.color.w * a)
                    }
                }
                padding: Inset{left: 16 right: 16 top: 8 bottom: 10}

            // ── "New item" popup menu (hidden by default; shown above the
            // input row when menu_button is clicked) ──
            new_item_menu := View{
                width: 190
                height: Fit
                flow: Down
                spacing: 2
                visible: false
                margin: Inset{bottom: 8}
                show_bg: true
                draw_bg +: {
                    color: #x1c1f28ff
                    pixel: fn() {
                        let p = self.pos * self.rect_size
                        let d = min(min(p.x, self.rect_size.x - p.x), min(p.y, self.rect_size.y - p.y))
                        let a = 1.0 - smoothstep(0.0, 1.0, d)
                        return vec4(self.color.x, self.color.y, self.color.z, self.color.w * a)
                    }
                }
                padding: Inset{top: 4 bottom: 4 left: 4 right: 4}

                menu_new_terminal := Button{
                    text: "Terminal"
                    width: Fill
                    height: 30
                    draw_text +: {
                        text_style: theme.font_regular{font_size: 13}
                        color: mod.tc.text_primary
                    }
                }
                menu_new_note := Button{
                    text: "Note"
                    width: Fill
                    height: 30
                    draw_text +: {
                        text_style: theme.font_regular{font_size: 13}
                        color: mod.tc.text_primary
                    }
                }
                menu_new_browser := Button{
                    text: "Browser"
                    width: Fill
                    height: 30
                    draw_text +: {
                        text_style: theme.font_regular{font_size: 13}
                        color: mod.tc.text_primary
                    }
                }
            }

            input_row := View{
                width: Fill
                height: Fit
                flow: Right
                spacing: 10
                align: Align{y: 0.5}

                // ── "New item" menu button ──
                menu_button := Button{
                    text: "＋"
                    width: 32
                    height: 32
                    draw_text +: {
                        text_style: theme.font_regular{font_size: 16}
                        color: mod.tc.text_primary
                    }
                }

                // ── CNVS-style rounded command capsule ──
                input_capsule := View{
                    width: Fill
                    height: Fit
                    flow: Right
                    spacing: 6
                    align: Align{y: 0.5}
                    padding: Inset{left: 12 right: 8 top: 4 bottom: 4}
                    show_bg: true
                    draw_bg +: {
                        color: #x1a1c24ff
                        border_color: #x2a2e3aff
                        radius: instance(6.0)
                        pixel: fn() {
                            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.radius)
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
                        empty_text: "@name text · /new terminal NAME · /new browser URL · /help"
                        // Transparent background/border so the capsule's rounded corners show through.
                        draw_bg +: {
                            color: #00000000
                            border_color: #00000000
                        }
                        draw_text +: {
                            text_style: theme.font_regular{font_size: 13}
                            color: mod.tc.text_primary
                        }
                    }

                    mic_button := Button{
                        text: "🎤"
                        width: 26
                        height: 26
                        draw_text +: {
                            text_style: theme.font_regular{font_size: 12}
                            color: mod.tc.text_secondary
                        }
                    }
                }
            }

            // ── Command suggestion dropdown ──
            suggestion_list := View{
                width: Fill
                height: Fit
                flow: Down
                visible: false
                margin: Inset{left: 46 bottom: 6}
                padding: Inset{top: 4 bottom: 4 left: 4 right: 4}
                show_bg: true
                draw_bg +: {
                    color: #x1a1c24ff
                    border_color: #x2a2e3aff
                    radius: instance(10.0)
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, self.radius)
                        sdf.fill(self.color)
                        sdf.box(0.5, 0.5, self.rect_size.x - 1.0, self.rect_size.y - 1.0, self.radius)
                        sdf.stroke(self.border_color, 1.0)
                        return sdf.result
                    }
                }

                suggestion_0 := Label{width: Fill height: Fit padding: Inset{left: 8 right: 8 top: 5 bottom: 5} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                suggestion_1 := Label{width: Fill height: Fit padding: Inset{left: 8 right: 8 top: 5 bottom: 5} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                suggestion_2 := Label{width: Fill height: Fit padding: Inset{left: 8 right: 8 top: 5 bottom: 5} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                suggestion_3 := Label{width: Fill height: Fit padding: Inset{left: 8 right: 8 top: 5 bottom: 5} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                suggestion_4 := Label{width: Fill height: Fit padding: Inset{left: 8 right: 8 top: 5 bottom: 5} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
                suggestion_5 := Label{width: Fill height: Fit padding: Inset{left: 8 right: 8 top: 5 bottom: 5} visible: false draw_text +: {text_style: theme.font_regular{font_size: 12} color: mod.tc.text_primary}}
            }

            status_label := Label{
                width: Fill
                height: Fit
                margin: Inset{top: 6}
                text: "Ready — drag items, scroll to pan, hold ⌘ + scroll to zoom"
                draw_text +: {
                    text_style: theme.font_regular{font_size: 11}
                    color: mod.tc.text_secondary
                }
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
        if let Event::Startup = event {
            self.ui
                .text_input(
                    cx,
                    ids!(
                        main_window
                            .body
                            .canvas
                            .command_wrap
                            .command_bar
                            .input_row
                            .input_capsule
                            .command_input
                    ),
                )
                .set_key_focus(cx);
            if let Some(mut panel) = self
                .ui
                .widget(cx, ids!(main_window.body.canvas))
                .borrow_mut::<CanvasPanel>()
            {
                panel.set_grid_enabled(false);
                panel.spawn_terminal(cx, "claude", None, "zsh");
            }
        }
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

fn main() {
    app_main();
}

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
    Cx::event_loop(cx);
    makepad_cef::shutdown();
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_env = "ohos"))]
pub fn app_main() {
    panic!("canvas-terminal is desktop-only");
}
