use makepad_widgets::*;
use makepad_clipboard::ClipboardMonitor;
use std::fs;
use std::env;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: { title: "Clipboard Manager", inner_size: vec2(800, 700) }
                show_bg: true,
                draw_bg: { fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.rect(0.0, 0.0, self.rect_size.x, self.rect_size.y); sdf.fill(#x1a1a1a); return sdf.result; } }

                body = <View> { width: Fill, height: Fill, flow: Down, spacing: 8, padding: 16,
                    title = <Label> { width: Fill, height: Fit, text: "Clipboard Manager", draw_text: { text_style: <THEME_FONT_BOLD> {font_size: 22}, color: #xFFFFFF } }

                    buttons = <View> { width: Fill, height: Fit, flow: Right, spacing: 8,
                        btn_refresh = <Button> { text: "🔄 Refresh", width: Fit, height: Fit, padding: {left: 12, right: 12, top: 6, bottom: 6} }
                        btn_auto = <Button> { text: "Auto: OFF", width: Fit, height: Fit, padding: {left: 12, right: 12, top: 6, bottom: 6} }
                        btn_clear = <Button> { text: "🗑 Clear", width: Fit, height: Fit, padding: {left: 12, right: 12, top: 6, bottom: 6} }
                        lbl_status = <Label> { width: Fill, height: Fit, text: "Ready", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 12}, color: #x888888 } }
                    }

                    divider = <View> { width: Fill, height: 1, show_bg: true, draw_bg: { color: #x333333 } }
                    latest_header = <Label> { width: Fill, height: Fit, text: "📋 Latest", draw_text: { text_style: <THEME_FONT_BOLD> {font_size: 13}, color: #x888888 } }

                    img_box = <View> { width: Fill, height: 150, show_bg: true,
                        draw_bg: { fn pixel(self) -> vec4 { let sdf = Sdf2d::viewport(self.pos * self.rect_size); sdf.rect(1.0, 1.0, self.rect_size.x - 2.0, self.rect_size.y - 2.0); sdf.fill(#x2a2a2a); return sdf.result; } } }
                    img_preview = <Image> { width: Fill, height: 150 }
                    img_info = <Label> { width: Fill, height: Fit, align: {x: 1.0, y: 1.0}, padding: {left: 0, right: 8, top: 0, bottom: 8}, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 11}, color: #xFFFFFF } }

                    latest_txt = <Label> { width: Fill, height: Fit, text: "No entries yet. Click Refresh!", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 13}, color: #xFFFFFF } }

                    history_header = <Label> { width: Fill, height: Fit, text: "📜 History", draw_text: { text_style: <THEME_FONT_BOLD> {font_size: 13}, color: #x888888 } }
                    history_list = <Label> { width: Fill, height: Fill, text: "", draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 11}, color: #x999999 } }
                }
            }
        }
    }
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] monitor: ClipboardMonitor,
    #[rust] last_entry_count: usize,
    #[rust] auto_refresh: bool,
    #[rust] current_image_path: Option<String>,
    #[rust] poll_timer: u32,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) { makepad_widgets::live_design(cx); makepad_component::live_design(cx); }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        match event {
            Event::Startup => self.poll_clipboard(cx),
            Event::KeyDown(ke) => {
                if ke.key_code == KeyCode::Space || ke.key_code == KeyCode::ReturnKey { self.poll_clipboard(cx); }
                if ke.key_code == KeyCode::Escape { self.auto_refresh = false; self.ui.button(ids!(btn_auto)).set_text(cx, "Auto: OFF"); }
            }
            _ => {}
        }
        if self.auto_refresh { self.poll_timer += 1; if self.poll_timer >= 30 { self.poll_timer = 0; self.poll_clipboard(cx); } }

        let actions = cx.capture_actions(|cx| self.ui.handle_event(cx, event, &mut Scope::empty()));
        if self.ui.button(ids!(btn_refresh)).clicked(&actions) { self.poll_clipboard(cx); }
        if self.ui.button(ids!(btn_auto)).clicked(&actions) { self.auto_refresh = !self.auto_refresh; self.ui.button(ids!(btn_auto)).set_text(cx, if self.auto_refresh { "⏸ Auto: ON" } else { "Auto: OFF" }); self.poll_timer = 0; }
        if self.ui.button(ids!(btn_clear)).clicked(&actions) { self.clear_all(cx); }
    }
}

impl App {
    fn poll_clipboard(&mut self, cx: &mut Cx) {
        if let Some(entry) = self.monitor.poll_clipboard() {
            self.monitor.add_entry(entry.clone());
            self.last_entry_count = self.monitor.get_entries().len();
            self.ui.label(ids!(lbl_status)).set_text(cx, &format!("{} entries", self.last_entry_count));
            self.update_display(cx);
        }
    }

    fn clear_all(&mut self, cx: &mut Cx) {
        self.monitor.clear();
        self.last_entry_count = 0;
        if let Some(ref path) = self.current_image_path { let _ = fs::remove_file(path); }
        self.current_image_path = None;
        self.ui.label(ids!(lbl_status)).set_text(cx, "Cleared");
        self.ui.label(ids!(latest_txt)).set_text(cx, "No entries yet. Click Refresh!");
        self.ui.view(ids!(img_box)).set_visible(cx, false);
        self.ui.image(ids!(img_preview)).set_visible(cx, false);
        self.ui.label(ids!(img_info)).set_visible(cx, false);
        self.ui.label(ids!(history_list)).set_text(cx, "");
    }

    fn update_display(&mut self, cx: &mut Cx) {
        let entries = self.monitor.get_entries();
        if entries.is_empty() {
            self.ui.label(ids!(latest_txt)).set_text(cx, "No entries yet. Click Refresh!");
            self.ui.view(ids!(img_box)).set_visible(cx, false);
            self.ui.image(ids!(img_preview)).set_visible(cx, false);
            self.ui.label(ids!(img_info)).set_visible(cx, false);
            self.ui.label(ids!(history_list)).set_text(cx, "");
            return;
        }

        // History
        if entries.len() > 1 {
            let history: Vec<String> = entries.iter().skip(1).take(15).map(|e| self.fmt(e)).collect();
            self.ui.label(ids!(history_list)).set_text(cx, &history.join("\n\n"));
        } else {
            self.ui.label(ids!(history_list)).set_text(cx, "(No history)");
        }

        // Latest
        if let Some(latest) = entries.first() {
            match &latest.content {
                makepad_clipboard::ClipboardContent::Image { width, height, data } => {
                    self.ui.label(ids!(latest_txt)).set_visible(cx, false);
                    self.ui.view(ids!(img_box)).set_visible(cx, true);
                    self.ui.image(ids!(img_preview)).set_visible(cx, true);
                    self.ui.label(ids!(img_info)).set_visible(cx, true);
                    self.ui.label(ids!(img_info)).set_text(cx, &format!("{}x{}", width, height));
                    if !data.is_empty() {
                        if let Some(ref old) = self.current_image_path { let _ = fs::remove_file(old); }
                        let path = env::temp_dir().join(format!("clip_{}.png", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
                        if fs::write(&path, data).is_ok() {
                            self.current_image_path = Some(path.to_string_lossy().to_string());
                            let _ = self.ui.image(ids!(img_preview)).load_image_file_by_path(cx, std::path::Path::new(&self.current_image_path.as_ref().unwrap()));
                        }
                    }
                }
                makepad_clipboard::ClipboardContent::Text(text) => {
                    self.ui.label(ids!(latest_txt)).set_visible(cx, true);
                    self.ui.view(ids!(img_box)).set_visible(cx, false);
                    self.ui.image(ids!(img_preview)).set_visible(cx, false);
                    self.ui.label(ids!(img_info)).set_visible(cx, false);
                    self.ui.label(ids!(latest_txt)).set_text(cx, text);
                }
                makepad_clipboard::ClipboardContent::Html(_) => {
                    self.ui.label(ids!(latest_txt)).set_visible(cx, true);
                    self.ui.view(ids!(img_box)).set_visible(cx, false);
                    self.ui.label(ids!(latest_txt)).set_text(cx, "HTML Content");
                }
                makepad_clipboard::ClipboardContent::Files(files) => {
                    self.ui.label(ids!(latest_txt)).set_visible(cx, true);
                    self.ui.view(ids!(img_box)).set_visible(cx, false);
                    self.ui.label(ids!(latest_txt)).set_text(cx, &format!("Files: {}", files.join(", ")));
                }
                makepad_clipboard::ClipboardContent::Unknown => {
                    self.ui.label(ids!(latest_txt)).set_visible(cx, true);
                    self.ui.view(ids!(img_box)).set_visible(cx, false);
                    self.ui.label(ids!(latest_txt)).set_text(cx, &latest.formats.join(", "));
                }
            }
        }
    }

    fn fmt(&self, e: &makepad_clipboard::ClipboardEntry) -> String {
        let ts = if e.timestamp.len() > 10 { &e.timestamp[e.timestamp.len()-8..] } else { &e.timestamp };
        match &e.content {
            makepad_clipboard::ClipboardContent::Text(t) => { let s = t.replace('\n', " "); format!("[{}] TEXT: {}", ts, if s.len() <= 50 { s } else { format!("{}...", &s[..50]) }) }
            makepad_clipboard::ClipboardContent::Image { width, height, .. } => format!("[{}] IMAGE: {}x{}", ts, width, height),
            makepad_clipboard::ClipboardContent::Html(_) => format!("[{}] HTML", ts),
            makepad_clipboard::ClipboardContent::Files(fs) => format!("[{}] FILES: {}", ts, fs.join(", ")),
            makepad_clipboard::ClipboardContent::Unknown => format!("[{}] {}", ts, e.formats.join(", ")),
        }
    }
}

app_main!(App);
fn main() { app_main() }
