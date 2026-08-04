# raycast-launcher 主面板视觉重做 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 将 raycast-launcher 主面板视觉重做到 tinycast 设计系统水准（模拟桌面 + 浮动暗色面板、白色透明度色阶、边缘溶解、glass 浮动控件）。

**架构：** 全部在 Makepad `script_mod!` DSL + SDF 像素着色器内完成（方案 A，无 OS 级透明）。设计令牌单源于 `mod.tc` 模块对象；着色器内通过 `uniform(mod.tc.*)` 引用，属性位置直接 `mod.tc.*` 引用。Rust 侧仅清理 `draw_walk`/`update_labels` 颜色逻辑。

**技术栈：** Rust · Makepad `script_mod!` DSL · Sdf2d 像素着色器。

**规格：** `docs/superpowers/specs/2026-08-04-raycast-launcher-tinycast-visual-design.md`

**已验证的 DSL 事实（计划前提）：**

- 模块级对象赋值可用：`mod.themes.dark = mod.themes.dark{...}`（makepad widgets lib.rs:282）。
- 着色器 uniform 可引用模块对象：`color: uniform(theme.color_outset)`（scroll_bar.rs）。
- `flow: Overlay` 中父 `align` 对每个子 individually 生效；子可用 `margin` 从共同原点偏移（turtle.rs:1302/1641）→ 底部栏用固定 margin 钉住。
- PortalList 滚动条可覆盖 uniforms：`scroll_bar := ScrollBar{ draw_bg +: { size: uniform(2.0) ... } }`（portal_list.rs:26，scroll_bar.rs uniforms）。
- hex 颜色格式 `#xAARRGGBB`。

---

## 文件结构

| 文件 | 职责 | 操作 |
| --- | --- | --- |
| `crates/raycast-launcher/src/main.rs` | 唯一改动文件：`script_mod!` UI 块 + Rust 逻辑清理 | 修改 |

不新建文件（令牌在 `mod.tc`；crate 为单文件 shell，遵循现有模式）。

---

### 任务 1：令牌 spike（验证 `mod.tc` 可引用）

**文件：**

- 修改：`crates/raycast-launcher/src/main.rs`（script_mod! 块顶部）

- [ ] **步骤 1：在 `script_mod!` 块内 `use mod.prelude.widgets.*` 之后插入令牌对象与 spike 引用**

```
    // ── tinycast Theme 移植令牌（单一来源；着色器内用 uniform(mod.tc.*) 引用）──
    mod.tc = {
        surface: #xff101013          // 面板表面（black 0.40 叠极暗底）
        hairline: #x14ffffff         // white 0.08 面板外边框
        selection: #x1affffff        // white 0.10 选中行
        row_hover: #x0dffffff        // white 0.05 悬停行
        control_surface: #x1affffff  // white 0.10 填充 keycap
        border: #x33ffffff           // white 0.20 描边 keycap
        text_primary: #xf2ffffff     // white 0.95
        text_secondary: #x99ffffff   // white 0.60
        text_tertiary: #x66ffffff    // white 0.40
        glass_top: #x24ffffff        // white 0.14
        glass_bottom: #x10ffffff     // white 0.06
        glass_stroke: #x38ffffff     // white 0.22
        separator: #x1affffff        // white 0.10
        scrollbar: #x4dffffff        // white 0.30
    }
```

并把 `launcher_view` 里 `Label{ text: "Launcher" ... }` 的 `color:` 临时改为 `mod.tc.text_primary`，把 `mod.widgets.LauncherPanel` 的 `draw_bg` 内加一行 `surface_col: uniform(mod.tc.surface)`（仅声明，不改 pixel）。

- [ ] **步骤 2：编译验证**

运行：`cargo check -p raycast-launcher`
预期：PASS。

- [ ] **步骤 3：若 FAIL（`mod.tc` 在属性位置或 uniform 位置不被接受）**

回退方案：删除 `mod.tc` 块；在 `main.rs` 顶部（Rust 侧）加：

```rust
/// tinycast Theme 移植令牌（#xAARRGGBB）。DSL 中使用同值字面量并以 `// tc.<name>` 注释标注。
pub(crate) mod tc {
    pub const SURFACE: &str = "#xff101013";
    pub const SELECTION: [f32; 4] = [1.0, 1.0, 1.0, 0.10];
    pub const ROW_HOVER: [f32; 4] = [1.0, 1.0, 1.0, 0.05];
    pub const TEXT_PRIMARY: [f32; 4] = [1.0, 1.0, 1.0, 0.95];
    pub const TEXT_SECONDARY: [f32; 4] = [1.0, 1.0, 1.0, 0.60];
    pub const TEXT_TERTIARY: [f32; 4] = [1.0, 1.0, 1.0, 0.40];
}
```

DSL 中全部改用对应 hex 字面量 + `// tc.<name>` 注释。后续任务中所有 `mod.tc.*` 引用替换为字面量。

- [ ] **步骤 4：记录结果并 commit**

```bash
git add crates/raycast-launcher/src/main.rs
git commit -m "feat(raycast): add tinycast design tokens (mod.tc) with spike verification"
```

---

### 任务 2：窗口 shell —— 模拟桌面 + 浮动面板

**文件：**

- 修改：`crates/raycast-launcher/src/main.rs`（`startup()` 块）

- [ ] **步骤 1：替换 `startup()` 块为以下实现**

```
    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.inner_size: vec2(900 620)
                window.title: "Raycast Launcher"
                pass +: { clear_color: #xff08080a }
                body +: {
                    bg_view := View{
                        width: Fill
                        height: Fill
                        flow: Down
                        align: Center
                        show_bg: true
                        draw_bg +: {
                            // 模拟 macOS 桌面壁纸：斜向明暗条纹（对照 tinycast 截图）
                            pixel: fn() {
                                let p = self.pos
                                let d = p.x * 0.72 + p.y * 0.68
                                let w1 = sin(d * 10.7 + sin(p.y * 4.0) * 0.6) * 0.5 + 0.5
                                let w2 = sin(d * 5.7 + 2.1) * 0.5 + 0.5
                                let v = smoothstep(0.35 0.65 w1) * 0.55 + smoothstep(0.4 0.7 w2) * 0.35
                                let g = v * v
                                return mix(#xff08080a #xffc9c9ce g)
                            }
                        }
                        launcher := mod.widgets.LauncherPanel{
                            margin: Inset{bottom: 30}   // 垂直居中偏上 ≈8%
                        }
                    }
                }
            }
        }
    }
```

- [ ] **步骤 2：替换 `mod.widgets.LauncherPanel = set_type_default() do mod.widgets.LauncherPanelBase{` 的根属性（保留其子树，任务 3-6 再改子树）**

把根属性改为：

```
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
```

（删除原根的 `width: Fill / height: Fill / flow: Down / spacing / padding` 与旧 draw_bg。）

- [ ] **步骤 3：编译验证**

运行：`cargo check -p raycast-launcher`
预期：PASS（Rust 侧 ids 未动，编译安全）。

- [ ] **步骤 4：commit**

```bash
git add crates/raycast-launcher/src/main.rs
git commit -m "feat(raycast): floating panel over simulated desktop wallpaper"
```

---

### 任务 3：头部搜索栏（透明浮层）

**文件：**

- 修改：`crates/raycast-launcher/src/main.rs`（`mode_input_row` 块）

- [ ] **步骤 1：把 `mode_input_row` 整块替换为（作为 Overlay 第一个子之外的浮层；将在任务 7 统一调整子顺序，此处先改内容）**

```
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
                draw_select +: {
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
```

（删除 `mode_hint_label`。）

- [ ] **步骤 2：Rust 侧 `sync_mode_input` 删除 hint/spacing 逻辑**

把 `sync_mode_input` 函数体替换为：

```rust
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
                    if self.chat_loading { "Sending..." } else { "Send" },
                    self.chat_loading,
                )
            } else {
                ("Search for apps and commands...", self.query.as_str(), false, false, false, "Add", false)
            };

        self.view
            .text_input(cx, ids!(mode_input))
            .set_empty_text(cx, empty_text.to_string());
        self.view.text_input(cx, ids!(mode_input)).set_text(cx, text);
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
```

- [ ] **步骤 3：编译验证**

运行：`cargo check -p raycast-launcher`
预期：PASS。

- [ ] **步骤 4：commit**

```bash
git add crates/raycast-launcher/src/main.rs
git commit -m "feat(raycast): transparent floating search header with SDF glyphs"
```

---

### 任务 4：行语法 + section header + draw_walk 清理

**文件：**

- 修改：`crates/raycast-launcher/src/main.rs`（`launcher_view` 的 `results` 模板 + `draw_walk` + `update_labels`）

- [ ] **步骤 1：替换 `results := PortalList{` 整块为**

```
            results := PortalList{
                width: Fill
                height: Fill
                flow: Down
                spacing: 0

                scroll_bar := ScrollBar{
                    draw_bg +: {
                        size: uniform(2.0)
                        border_radius: uniform(1.0)
                        color: uniform(mod.tc.scrollbar)
                        color_hover: uniform(#x6bffffff)   // white 0.42
                        color_drag: uniform(#x80ffffff)   // white 0.50
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
```

（删除 `app_meta_chip`、`action_hint_chip`、`app_desc`、icon_wrap 的 draw_bg。）

- [ ] **步骤 2：替换 `draw_walk` 中 `while let Some(item_id) = list.next_visible_item(cx)` 循环体为**

```rust
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
                        row.widget(cx, ids!(group_label)).set_visible(cx, show_group);
                        row.label(cx, ids!(group_label)).set_text(cx, group_name);
                        if let Some(mut v) = row.view(cx, ids!(row_bg)).borrow_mut() {
                            v.layout.margin.top = if show_group && item_id != 0 { 12.0 } else { 0.0 };
                        }

                        row.label(cx, ids!(app_name)).set_text(cx, &app_name);
                        row.label(cx, ids!(app_meta)).set_text(cx, &category);
                        row.label(cx, ids!(app_icon_fallback)).set_text(cx, &fallback);

                        if let Some(path) = icon_path {
                            let loaded = row
                                .image(cx, ids!(app_icon))
                                .load_image_file_by_path(cx, Path::new(&path))
                                .is_ok();
                            row.widget(cx, ids!(app_icon)).set_visible(cx, loaded);
                            row.widget(cx, ids!(app_icon_fallback)).set_visible(cx, !loaded);
                        } else {
                            row.widget(cx, ids!(app_icon)).set_visible(cx, false);
                            row.widget(cx, ids!(app_icon_fallback)).set_visible(cx, true);
                        }

                        let selected = if item_id == self.selected_index { 1.0 } else { 0.0 };
                        let hovered = if Some(item_id) == self.hovered_index { 1.0 } else { 0.0 };
                        if let Some(mut view) = row.view(cx, ids!(row_bg)).borrow_mut() {
                            view.draw_bg
                                .draw_vars
                                .set_dyn_instance(cx, live_id!(selected), &[selected]);
                            view.draw_bg
                                .draw_vars
                                .set_dyn_instance(cx, live_id!(hovered), &[hovered]);
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
```

- [ ] **步骤 3：删除文件底部 `fn v4a` （不再使用）**

- [ ] **步骤 4：编译验证**

运行：`cargo check -p raycast-launcher`
预期：可能报 unused warning（`update_labels` 仍引用旧 ids，下一步处理）；error 必须为 0。若 `update_labels` 编译报错（引用已删除 ids 不会报错，ids! 为运行时解析），确认 PASS。

- [ ] **步骤 5：commit**

```bash
git add crates/raycast-launcher/src/main.rs
git commit -m "feat(raycast): tinycast row grammar — flat white-alpha rows, hairline scrollbar"
```

---

### 任务 5：边缘溶解条

**文件：**

- 修改：`crates/raycast-launcher/src/main.rs`（`launcher_view` 内，`results` 之后插入）

- [ ] **步骤 1：在 `launcher_view` 中 `results` 块之后插入两个溶解条**

```
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
                        return vec4(c.x * a c.y * a c.z * a a)
                    }
                }
            }

            dissolve_bottom := View{
                width: Fill
                height: 80
                show_bg: true
                draw_bg +: {
                    surface_col: uniform(mod.tc.surface)
                    pixel: fn() {
                        let a = self.pos.y
                        let c = self.surface_col
                        return vec4(c.x * a c.y * a c.z * a a)
                    }
                }
            }
```

- [ ] **步骤 2：把 `launcher_view` 的 flow 改为 Overlay，使溶解条叠在 results 上**

```
        launcher_view := View{
            width: Fill
            height: Fill
            flow: Overlay
```

（`empty_state` 居中：给 `empty_state` 加 `width: Fill height: Fill align: Center flow: Down`，删除其 draw_bg 边框，样式见任务 7。）

- [ ] **步骤 3：编译 + 运行目检**

运行：`cargo check -p raycast-launcher && cargo run -p raycast-launcher`
预期：滚动列表时行在顶部/底部渐变消隐，无硬裁切。

- [ ] **步骤 4：commit**

```bash
git add crates/raycast-launcher/src/main.rs
git commit -m "feat(raycast): edge dissolve gradient masks on results list"
```

---

### 任务 6：底部栏（菜单圆点 + status + 动作胶囊 + KeyCap）

**文件：**

- 修改：`crates/raycast-launcher/src/main.rs`

- [ ] **步骤 1：在 `mod.tc` 之后定义 KeyCap 模板**

```
    mod.widgets.KeyCap = View{
        width: Fit
        height: 18
        align: Center
        padding: Inset{left: 6 right: 6}
        show_bg: true
        draw_bg +: {
            filled: uniform(1.0)
            fill_col: uniform(mod.tc.control_surface)
            stroke_col: uniform(mod.tc.border)
            pixel: fn() {
                let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y 6.0)
                if self.filled > 0.5 {
                    sdf.fill(self.fill_col)
                } else {
                    sdf.stroke(self.stroke_col 1.0)
                }
                return sdf.result
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
```

- [ ] **步骤 2：在 LauncherPanel 根 Overlay 中（`mode_input_row` 之后）插入底部栏，替换旧底部状态条**

删除 `launcher_view` 末尾的旧状态条 View（含 `status_label`/`status_keys_label`），在根 Overlay 插入：

```
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
                padding: Inset{left: 14 right: 10}
                show_bg: true
                draw_bg +: {
                    top_col: uniform(mod.tc.glass_top)
                    bot_col: uniform(mod.tc.glass_bottom)
                    stroke_col: uniform(mod.tc.glass_stroke)
                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size)
                        let r = self.rect_size.y * 0.5
                        sdf.box(0.0 0.0 self.rect_size.x self.rect_size.y r)
                        let g = mix(self.top_col self.bot_col self.pos.y)
                        // 顶部内侧高光
                        let hl = pow(1.0 - self.pos.y 3.0) * 0.12
                        let g = g + vec4(hl hl hl hl)
                        sdf.fill(g)
                        sdf.stroke(self.stroke_col 1.0)
                        return sdf.result
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
```

- [ ] **步骤 3：Rust `update_labels` 替换为**

```rust
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
                self.view.label(cx, ids!(empty_title)).set_text(cx, "No Results");
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
```

- [ ] **步骤 4：编译验证**

运行：`cargo check -p raycast-launcher`
预期：PASS。

- [ ] **步骤 5：commit**

```bash
git add crates/raycast-launcher/src/main.rs
git commit -m "feat(raycast): glass bottom bar with action capsule and keycaps"
```

---

### 任务 7：根 Overlay 子顺序 + 空状态 + 移除旧 chrome

**文件：**

- 修改：`crates/raycast-launcher/src/main.rs`

- [ ] **步骤 1：调整 LauncherPanel 根 Overlay 子声明顺序为（底→顶）**

```
        launcher_view   (Fill, 含 results/dissolve/empty)
        todo_view       (Fill)
        chat_view       (Fill)
        mode_input_row  (Fit, 顶)
        bottom_bar      (Fit, margin top 423)
```

即把 `mode_input_row` 与 `bottom_bar` 的声明移到三个模式 view 之后。

- [ ] **步骤 2：`launcher_view` 内删除 "Launcher / Quickly open apps..." 标题块与 `top_count_row` 整块**

- [ ] **步骤 3：`empty_state` 替换为居中无框样式**

```
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
```

- [ ] **步骤 4：`todo_view` / `chat_view` 根改为 `width: Fill height: Fill`（Overlay 层），内部结构不动**

- [ ] **步骤 5：编译 + 运行目检**

运行：`cargo check -p raycast-launcher && cargo run -p raycast-launcher`
预期：header 浮顶、底部栏钉底、列表铺满并溶解；空查询时居中空状态。

- [ ] **步骤 6：commit**

```bash
git add crates/raycast-launcher/src/main.rs
git commit -m "feat(raycast): overlay layering, centered empty state, remove old chrome"
```

---

### 任务 8：截图验证 + 收尾

**文件：** 无代码改动（除非目检发现问题）

- [ ] **步骤 1：运行并截图**

运行：`cargo run -p raycast-launcher`（后台），按 `skills/makepad-screenshot/SKILL.md` 截图保存为 `/tmp/raycast-tc.png`。

- [ ] **步骤 2：对照 tinycast `docs/screenshot.png` 目检清单**

- 面板 750×475、圆角 26、四周露出模拟壁纸 ✔
- 行：24pt 图标槽 / 单行标题 / 行尾 tertiary 类别文本；选中行 white 0.10 填充、无描边 ✔
- section header 11pt medium secondary ✔
- 顶/底溶解渐变，滚动无硬裁切 ✔
- 底部：左 glass 圆点 + tertiary status；右 glass 胶囊 + keycaps ✔
- header：放大镜 + 20pt 透明输入 ✔
- 无蓝/橙 tint、无灰色字面量 ✔

发现问题即修（只改 `main.rs`），每修一类 commit 一次。

- [ ] **步骤 3：lint/fmt**

运行：`cargo clippy -p raycast-launcher -- -D warnings && cargo fmt --all`
预期：PASS。

- [ ] **步骤 4：最终 commit**

```bash
git add -A
git commit -m "style(raycast): visual parity pass against tinycast screenshot"
```

---

## 自检

**规格覆盖度：** §1 结构→任务 2/7；§2 令牌→任务 1；§3 行→任务 4；§4 溶解+滚动条→任务 5/4；§5 底部栏→任务 6；§6 header→任务 3；§7 空状态→任务 7；§8 范围→全计划仅改 main.rs；§9 验证→任务 8。无遗漏。

**占位符扫描：** 无 TODO/待定；所有步骤含完整代码或精确命令。

**类型一致性：** ids 使用一致：`mode_input_row/mode_back_wrap/mode_back_btn/search_glyph/mode_input/mode_action_wrap/mode_action_btn/results/ResultRow/row_bg/group_label/icon_wrap/app_icon/app_icon_fallback/app_name/app_meta/empty_state/empty_title/empty_desc/status_label/primary_action_label/bottom_bar/launcher_view/todo_view/chat_view`；Rust 与 DSL 一一对应（`app_meta_chip`/`action_hint_chip`/`app_desc`/`mode_hint_label`/`result_count`/`status_keys_label` 在任务 3/4/6/7 中 DSL 与 Rust 同步删除）。
