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
            pixel: fn() {
                let p = self.pos * self.rect_size
                let pan = self.cam_pan
                let zoom = self.cam_zoom
                let g = self.grid_size
                let w = (p - self.rect_size * 0.5) / zoom + pan
                // distance to nearest grid line (fract-based mod)
                let fx = fract(w.x / g) * g
                let fy = fract(w.y / g) * g
                let dx = min(fx, g - fx)
                let dy = min(fy, g - fy)
                let d = min(dx, dy)
                let a = 1.0 - smoothstep(0.0, 1.5 / zoom, d)
                // axes (world 0,0)
                let axis = 1.0 - smoothstep(0.0, 2.0 / zoom, min(abs(w.x), abs(w.y)))
                let col = mix(self.grid_col, self.axis_col, axis)
                return vec4(col.x, col.y, col.z, col.w * max(a, axis))
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
                    height: 32
                    empty_text: "@name text · /new terminal NAME · /new browser URL · /help"
                    draw_text +: {
                        text_style: theme.font_regular{font_size: 13}
                        color: mod.tc.text_primary
                    }
                }
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
                .text_input(cx, ids!(main_window.body.canvas.command_wrap.command_bar.command_input))
                .set_key_focus(cx);
            if let Some(mut panel) = self.ui.widget(cx, ids!(main_window.body.canvas)).borrow_mut::<CanvasPanel>() {
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
