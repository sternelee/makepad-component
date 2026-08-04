# raycast-launcher 主面板视觉重做 — 设计规格

日期：2026-08-04
范围：`crates/raycast-launcher` 主面板（launcher 模式）视觉重做，对齐 tinycast 设计系统。
方案：方案 A —— 深邃暗色浮动面板，全部用 Makepad SDF 着色器模拟，不依赖 OS 级透明/模糊。

参考实现：`/Users/sternelee/www/github/tinycast`（`docs/ui.md`、`Core/Theme.swift`）。

---

## 1. 总体结构：模拟桌面 + 浮动面板

Makepad 无法做 OS 级 behind-window blur，因此在窗口内模拟 tinycast 的层叠关系：

```
Window (inner_size ≈ 900×620, clear_color 深色)
└─ desktop_bg        模拟 macOS 桌面壁纸：柔和斜向明暗条纹（对照 tinycast 截图的黑白银渐变）
   └─ panel (750×475, 圆角 26 continuous, 水平居中、垂直居中偏上 8%)
      ├─ results             PortalList，铺满整个面板
      ├─ dissolve_top        顶部边缘溶解渐变条（76pt）
      ├─ dissolve_bottom     底部边缘溶解渐变条（80pt）
      ├─ header (44pt)       透明浮层：放大镜/chevron + 搜索输入
      └─ bottom_bar (52pt)   透明浮层：菜单圆点 + status + 动作胶囊
```

层叠顺序（底→顶）：`results` → `dissolve_*` → `header` / `bottom_bar`。

面板表面 = 纯平单色：black 0.40 遮罩叠在极暗底上，取 `#101013`。深度感由面板外的模拟壁纸与 hairline 边框承担。
面板四周 1px hairline 边框：white 0.08，continuous 圆角 26。

窗口尺寸从 900×700 调整为约 900×620，使面板四周露出模拟桌面。

## 2. 设计令牌（Theme 移植）

所有视图引用 script 模块级对象 `mod.tc`（单一来源）；Rust `draw_walk` 中需要动态设置的颜色在 Rust 侧维护对应的常量模块，数值与 `mod.tc` 保持一致。禁止散落魔法数。

颜色（白色透明度色阶；不使用灰色字面量，不使用蓝/橙 tint）：

| 令牌 | 值 | 用途 |
| --- | --- | --- |
| `panel_dim` | black 0.40 | 面板遮罩（体现为表面色 #101013 的构成说明） |
| `selection` | white 0.10 | 选中行填充 |
| `row_hover` | white 0.05 | 悬停行填充 |
| `control_surface` | white 0.10 | 填充式 keycap |
| `border` | white 0.20 | 描边式 keycap |
| `panel_hairline` | white 0.08 | 面板外边框 |
| `text_secondary` | white 0.60 | 次级标签、section header、菜单圆点 glyph |
| `text_tertiary` | white 0.40 | 占位符、行尾类别标签、status 标签 |
| `card_fill` / `card_stroke` | white 0.05 / 0.10 | 卡片类表面（本规格中仅预留） |
| `glass_top` / `glass_bottom` | white 0.14 / 0.06 | glass 控件垂直渐变填充 |
| `glass_stroke` | white 0.22 | glass 控件描边 |

尺寸：panel 750×475 · header 44 · bottom_bar 52 · row_icon 24 · keycap 18 · menu_circle 36 · capsule 高 34。
圆角（全部 continuous）：panel 26 · row 10 · keycap 6 · menu/capsule 16（capsule 实际为全圆角）。
字体：搜索框 20pt regular · 行标题 13pt regular · section header 11pt medium · 行尾/状态 11pt regular。

关键取舍：现有 command/builtin 行的蓝/橙 tint 与描边全部移除；类别差异只靠行尾文本标签表达。

## 3. 行语法与选中态

```
[24pt 图标槽] --10pt-- [标题 white 0.95, 13pt] ......... [类别 white 0.40, 11pt 纯文本]
```

- 行内边距：horizontal 8（md），vertical 6（sm）。行背景 `RoundedRectangle(10, continuous)`。
- 填充优先级：selection(white 0.10) → hover(white 0.05) → clear。**无描边**。
- 图标槽固定 24pt：有缓存 PNG 时画 24pt 图标；无图标时槽内居中画首字母（white 0.60，12pt medium）。槽本身无背景、无边框（删除现有 `icon_wrap` 方框）。
- 行尾：现有 `app_meta_chip` 带边框小标签改为纯文本类别标签（white 0.40）。删除 `action_hint_chip`（动作提示收归底部胶囊）。
- 删除行内 `app_desc` 副标题行（单行行，对齐 tinycast）。
- Section header：非选中显示行，11pt medium white 0.60；header 与首行间距 4pt；非首个 header 上方间距 12pt（`sectionSpacing`），首个 header 无上方间距。
- 悬停机制不变（行级 `hovered_index`）。

## 4. 边缘溶解

- `dissolve_top`：高 76pt（44+32），SDF 线性渐变：顶部 = 面板表面色（不透明）→ 底部 = 透明。
- `dissolve_bottom`：高 80pt（52+28），反向。
- 面板表面为纯平单色（§1），渐变条与其像素级一致 → 行滚过 bar 下方时溶解而非裁切。
- 内容不可滚动时渐变条画在表面色上，视觉不可见，无需条件逻辑。
- PortalList 滚动条重画为 hairline：2pt 宽、white 0.30，靠右内嵌（若 script 侧可覆盖其 draw_bar；否则保留默认并在实现计划中标注）。

## 5. 底部栏（透明浮层，52pt）

- 左：36pt 菜单圆点。SDF 圆，glass 着色器（垂直渐变 white .14→.06 填充、white .22 描边、顶部内侧 1px 高光），glyph "···"（white 0.60）。本规格中为纯视觉元素，不接菜单行为（非目标）。
- 圆点右：status 标签，11pt white 0.40，内容 = 结果计数（如 "12 results"），取代现有 `top_count_row` 与底部状态条。
- 右：动作胶囊，高 34、全圆角、同款 glass 着色器。内容从左到右：
  主动作标签（12pt white 0.95，"Open Application" / "Run Command"，随选中项类别切换）
  - 填充 keycap `↵`
  - hairline 垂直分隔（white 0.10）
  - `Actions`（12pt white 0.60）+ 描边 keycaps `⌘` `K`。
- Keycap 模板 `mod.widgets.KeyCap` 定义一次：`filled`（white 0.10 填充，radius 6，高 18）/ `outline`（white 0.20 描边）。
- 删除现有底部状态条的边框盒子与 "Up/Down Select | Enter Open" 文案。

## 6. 头部搜索栏（透明浮层，44pt）

- SDF 放大镜 glyph（white 0.60，20pt 槽）+ 无边框 `TextInput`：20pt regular，文字 white 0.95，`empty_text` white 0.40，`draw_bg` 全透明（删除现有边框/填充背景与 focus 蓝色）。
- 子模式（todo/chat）：放大镜替换为 SDF 返回 chevron `‹`（white 0.60），点击行为不变（替换现有 "‹ Back" 文字按钮的视觉，保留 hit 区域与回调）。
- header 无背景、无边框；与列表的分隔完全靠 §4 溶解。

## 7. 空状态

- 删除边框卡片。居中垂直堆叠：标题 13pt medium white 0.60（"No Results" / "Start Searching"），描述 11pt white 0.40。
- `top_count_row` 移除（计数在底部栏 status）。

## 8. 范围与非目标

改：窗口尺寸与壁纸层、面板表面与 hairline、header、行、section header、溶解条、底部栏、空状态、滚动条样式、`draw_walk` 颜色逻辑（去 tint、去描边）。

不改：交互行为（选择/回车/双击/Esc/模式切换、hover 机制）、Rust 数据流与 item 扫描、chat/todo 视图内部（自动继承新 chrome；内部样式后续再做）、菜单圆点行为、global hotkey 等新功能。

## 9. 验证

1. `cargo check -p raycast-launcher` 通过。
2. `cargo run -p raycast-launcher` + `skills/makepad-screenshot` 截图。
3. 目检对照 tinycast `docs/screenshot.png`：面板比例/圆角、行语法、溶解效果、glass 控件、色阶一致性。
4. 在模拟壁纸的浅色变体下检查圆角与溶解条边界（壁纸 shader 提供 light/dark 参数，默认 dark）。
