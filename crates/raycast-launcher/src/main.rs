use makepad_widgets::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    pub LauncherPanel = {{LauncherPanel}} {
        width: 720,
        height: 540,
        flow: Down,
        spacing: 10,
        padding: {left: 16, right: 16, top: 16, bottom: 14},
        show_bg: true,
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 14.0);
                sdf.fill(#x14181d);
                sdf.stroke(#x2a313b, 1.0);
                return sdf.result;
            }
        }

        <Label> {
            text: "Launcher",
            draw_text: {
                text_style: <THEME_FONT_BOLD> {font_size: 18},
                color: #xeff3ff
            }
        }

        search_input = <TextInput> {
            width: Fill,
            height: Fit,
            empty_text: "Search apps and commands...",
            padding: {left: 12, right: 12, top: 11, bottom: 11},
            draw_bg: {
                instance border_color: #x334155,
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 10.0);
                    sdf.fill(#x0f1318);
                    sdf.stroke(self.border_color, 1.0);
                    return sdf.result;
                }
            }
            draw_text: {
                text_style: <THEME_FONT_REGULAR> {font_size: 13},
                color: #xe2e8f0
            }
        }

        result_count = <Label> {
            text: "",
            draw_text: {
                text_style: <THEME_FONT_REGULAR> {font_size: 11},
                color: #x94a3b8
            }
        }

        results = <PortalList> {
            width: Fill,
            height: Fill,
            flow: Down,
            spacing: 0,

            ResultRow = <View> {
                width: Fill,
                height: Fit,
                margin: {top: 3, bottom: 3},

                row_bg = <View> {
                    width: Fill,
                    height: Fit,
                    flow: Down,
                    spacing: 4,
                    padding: {left: 12, right: 12, top: 10, bottom: 10},
                    show_bg: true,
                    draw_bg: {
                        instance selected: 0.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 9.0);
                            let base = #x1a2028;
                            let active = #x2c66dd;
                            sdf.fill(mix(base, active, self.selected));
                            return sdf.result;
                        }
                    }

                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        align: {y: 0.5},
                        spacing: 10,

                        icon_wrap = <View> {
                            width: 26,
                            height: 26,
                            flow: Overlay,
                            align: {x: 0.5, y: 0.5},

                            app_icon = <Image> {
                                width: 24,
                                height: 24,
                                fit: Smallest,
                            }

                            app_icon_fallback = <Label> {
                                text: "A",
                                draw_text: {
                                    text_style: <THEME_FONT_BOLD> {font_size: 12},
                                    color: #xe2e8f0
                                }
                            }
                        }

                        app_name = <Label> {
                            width: Fill,
                            text: "App",
                            draw_text: {
                                text_style: <THEME_FONT_BOLD> {font_size: 14},
                                color: #xf8fafc
                            }
                        }

                        app_meta = <Label> {
                            text: "Category",
                            draw_text: {
                                text_style: <THEME_FONT_REGULAR> {font_size: 11},
                                color: #xcbd5e1
                            }
                        }
                    }

                    app_desc = <Label> {
                        text: "Description",
                        draw_text: {
                            text_style: <THEME_FONT_REGULAR> {font_size: 10},
                            color: #xb8c2d3
                        }
                    }
                }
            }
        }

        status_label = <Label> {
            text: "",
            draw_text: {
                text_style: <THEME_FONT_REGULAR> {font_size: 11},
                color: #x93c5fd
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
                        return mix(#x101215, #x1c2127, t);
                    }
                }

                body = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Overlay,
                    align: {x: 0.5, y: 0.5},

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
    selected_index: usize,
    #[rust]
    icon_cache: HashMap<usize, Option<String>>,
}

impl LiveHook for LauncherPanel {
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.all_items = load_launcher_items();
        self.query.clear();
        self.selected_index = 0;
        self.icon_cache.clear();
        self.prewarm_icons(48);
        self.rebuild_filter();
        self.view.text_input(ids!(search_input)).set_key_focus(cx);
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
    #[cfg(target_os = "macos")]
    fn render_quicklook_icon(bundle_path: &str, out_png: &Path) -> bool {
        let ql_dir = std::env::temp_dir().join("raycast-launcher-ql");
        if fs::create_dir_all(&ql_dir).is_err() {
            return false;
        }

        let status = Command::new("qlmanage")
            .args([
                "-t",
                "-s",
                "256",
                "-o",
                &ql_dir.to_string_lossy(),
                bundle_path,
            ])
            .status();

        let Ok(exit) = status else {
            return false;
        };
        if !exit.success() {
            return false;
        }

        let bundle_file = Path::new(bundle_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app");

        let expected_png = ql_dir.join(format!("{}.png", bundle_file));
        let source_png = if expected_png.exists() {
            expected_png
        } else {
            let mut candidate = None;
            if let Ok(rd) = fs::read_dir(&ql_dir) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    let is_png = p
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("png"))
                        .unwrap_or(false);
                    if is_png {
                        candidate = Some(p);
                    }
                }
            }
            let Some(p) = candidate else {
                return false;
            };
            p
        };

        fs::copy(source_png, out_png).is_ok()
    }

    fn prewarm_icons(&mut self, max_items: usize) {
        for idx in 0..self.all_items.len().min(max_items) {
            let _ = self.resolve_icon_for_index(idx);
        }
    }

    fn icon_cache_path(icns_path: &Path) -> Option<PathBuf> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        // Bump version when conversion params change to avoid stale cached icons.
        "v3_quicklook_256".hash(&mut hasher);
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
                    if !Self::render_quicklook_icon(bundle_path, &png) {
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
        self.filtered_indices = self
            .all_items
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                if q.is_empty() || item.search_key.contains(&q) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

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

    fn update_labels(&mut self, cx: &mut Cx, status_hint: &str) {
        let count_text = format!(
            "{} result(s)  |  Up/Down to navigate  |  Enter to launch",
            self.filtered_indices.len()
        );
        self.view
            .label(ids!(result_count))
            .set_text(cx, &count_text);

        let status = if self.filtered_indices.is_empty() {
            "No results. Try another keyword.".to_string()
        } else if let Some(item) = self.selected_item() {
            format!("{}  ->  {}", status_hint, item.app_name)
        } else {
            status_hint.to_string()
        };
        self.view.label(ids!(status_label)).set_text(cx, &status);
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
        }

        self.redraw(cx);
    }
}

impl Widget for LauncherPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        if let Some(text) = self.view.text_input(ids!(search_input)).changed(&actions) {
            self.query = text;
            self.selected_index = 0;
            self.rebuild_filter();
            self.update_labels(cx, "Filtered");
            self.redraw(cx);
        }

        if let Some((text, _mods)) = self.view.text_input(ids!(search_input)).returned(&actions) {
            self.query = text;
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
                _ => {}
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.filtered_indices.len());

                while let Some(item_id) = list.next_visible_item(cx) {
                    if let Some(source_idx) = self.filtered_indices.get(item_id) {
                        let source_idx = *source_idx;
                        let (app_name, category, subtitle, fallback) =
                            if let Some(entry) = self.all_items.get(source_idx) {
                                (
                                    entry.app_name.clone(),
                                    entry.category.clone(),
                                    entry.subtitle.clone(),
                                    entry.icon_fallback.clone(),
                                )
                            } else {
                                continue;
                            };

                        let icon_path = self.resolve_icon_for_index(source_idx);

                        let row = list.item(cx, item_id, live_id!(ResultRow));
                        row.label(ids!(app_name)).set_text(cx, &app_name);
                        row.label(ids!(app_meta)).set_text(cx, &category);
                        row.label(ids!(app_desc)).set_text(cx, &subtitle);
                        row.label(ids!(app_icon_fallback)).set_text(cx, &fallback);

                        if let Some(path) = icon_path {
                            let loaded = row
                                .image(ids!(app_icon))
                                .load_image_file_by_path(cx, Path::new(&path))
                                .is_ok();
                            row.widget(ids!(app_icon)).set_visible(cx, loaded);
                            row.widget(ids!(app_icon_fallback)).set_visible(cx, !loaded);
                        } else {
                            row.widget(ids!(app_icon)).set_visible(cx, false);
                            row.widget(ids!(app_icon_fallback)).set_visible(cx, true);
                        }

                        let selected = if item_id == self.selected_index {
                            1.0
                        } else {
                            0.0
                        };
                        row.view(ids!(row_bg)).apply_over(
                            cx,
                            live! {
                                draw_bg: {selected: (selected)}
                            },
                        );
                        row.draw_all(cx, &mut Scope::empty());
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
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::Startup = event {
            self.ui
                .text_input(ids!(launcher.search_input))
                .set_key_focus(cx);
        }

        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

fn main() {
    app_main()
}
