# UI 组件库开发进度（dev 分支）

> 最后更新：2026-09-05 · 分支 `dev` · 基线 commit `e046913`
> 参考项目：[gpui-component](https://github.com/longbridge/gpui-component)（shadcn 风格 Rust UI 库）
> 组件源码：`crates/ui/src/widgets/`（75 个文件）· 演示：`cargo run -p component-zoo`

---

## 1. 总体进度

| 阶段 | 内容 | 状态 |
|------|------|------|
| MpSize 尺寸系统 | 五档尺寸（XS..XLarge）+ Sizable trait 全组件接入 | ✅ 完成 |
| 菜单/显示批量 | menu_bar、dropdown_menu、context_menu、tooltip、popover 等 | ✅ 完成 |
| gpui 组件移植 | color_picker、tag、searchable_list、status_bar、number_input、avatar_group | ✅ 完成 |
| MpTree 增强 | 嵌套子树（with_children/flatten_tree）、禁用行（disabled ×0.5 淡出） | ✅ 完成 |
| **交互 bug 大扫除** | Sheet/Dialog/Select/Collapsible/Tag/Modal 全部无响应问题 | ✅ 完成（§3、§3.7） |
| a2ui 协议接入 | 新组件接入 ComponentType/processor | ⬜ 未开始 |

测试基线：`cargo test -p makepad-component` **36 passed**（含 clamp_number/snap_step/format_number 纯函数测试）。

---

## 2. 已移植组件清单（gpui → makepad）

近期批次（feat 提交链 `b71a4c5 → 461c194`）：

| 组件 | gpui 对应 | 说明 |
|------|-----------|------|
| `MpColorPicker` | color_picker.rs (607 行) | 简化为 5×9 色板网格，单 DrawMpSwatch turtle 复用，draw-call 命中测试，Sizable 14..32 |
| `MpTag` | tag.rs | 6 语义色 family，Filled/Outline 双风格 + 前置 dot |
| `MpSearchableList` | searchable_list | MpInputSearch + 预置行 + hover/selected visible 层切换 + "+N more" |
| `MpStatusBar` | 等效 | 纯 DSL 三槽容器（left/center/right），hairline 顶边，MpStatusText/MpStatusLed |
| `MpNumberInput` | number_input | 垂直 ▲▼ ghost 步进，snap/min/max clamp，DSL 尺寸变体（MpNumberInputSmall/Large） |
| `MpAvatarGroup` | avatar_group.rs | 8 头像负 margin 重叠 + "+N" tail，set_limit/set_ellipsis |
| `MpTree` 增强 | tree | 嵌套子树 flatten、disabled 行 |

明确跳过：gpui `ButtonGroup`（与现有 MpToggleGroup 重复）、TitleBar（平台耦合）、form/otp_input/dock（与 gpui Entity 状态深度耦合，不宜独立移植）。

---

## 3. 本轮修复的严重 Bug（重要度排序）

### 3.1 🔴 DSL `child: {}` 覆盖会"杀死"子件（全局性，影响最广）

**症状**：Collapsible/Select trigger 文本不渲染、点击区域塌陷；Table 无内容。

**根因**：在实例化组件时写 `label: { text: "..." }`（无 `+`），会把模板中 `label := Label{...}` 的 **widget object 替换成 plain object**。`View::on_after_apply` 吸收 children 时调用 `WidgetRef::value_is_newable_widget()` 检查失败 → **该子件被静默跳过**（无 `[E]` 输出）→ `trigger children = []`。

**修复**：所有覆盖 named child 的地方改用 **`+:` 合并语法**（`label +: { text: ... }`），保留 widget object 原型。

**排查方法（可复用）**：
```rust
// dump widget children 验证
let trig = self.ui.mp_collapsible_trigger(cx, ids!(collapsible_trigger));
trig.children(&mut |id, _w| { names.push_str(&format!("{} ", id)); });
log!("children = [{}]", names);  // 修复前: []，修复后: [label]
```
本次共修复 zoo 15 处（MpSelect trigger/options、MpCommandItem、MpSeparatorWithLabel、MpCollapsibleTrigger ×2）。

### 3.2 🔴 `find_widget_action(uid)` 被"影子 action"遮蔽

**症状**：Collapsible trigger 点击后 chevron 旋转（animator 生效）但内容永不展开。

**根因**：`find_widget_action` 返回**第一个** uid 匹配的 action。同一 widget uid 在同一批 actions 里存在**其它类型的 action**（如 focus/hover 系统发出），先于 `MpCollapsibleAction::Toggle` 出现 → `cast()` 失败 → 消费方永远 false。日志实证：同一 uid found=true 但 `btn=false coll=false`。

**修复**：不依赖 first-match，改为**直接遍历 downcast 目标类型**：
```rust
let mut hit = false;
for item in actions.iter().filter_map(|a| a.downcast_ref::<WidgetAction>()) {
    if item.widget_uid == self.widget_uid() {
        if let Some(t) = item.action.downcast_ref::<MpCollapsibleAction>() {
            if matches!(t, MpCollapsibleAction::Toggle) { hit = true; }
        }
    }
}
hit
```
已应用：collapsible（toggled）、select（toggle 分支）、sheet（opened/closed）。**dialog/modal 的 closed/close_requested 还是旧写法，待迁移**。

### 3.3 🔴 Modal 系组件根节点 `visible: false` 从不翻转

**症状**：Sheet/Dialog 点击后 action 全链路通（日志确认 set_open(true) 执行）但**什么都画不出来**。

**根因**：`MpSheet`/`MpDialog` DSL 模板根节点 `visible: false`，而 `set_open/open` 只切换了**子件**（overlay/content）的 visible，根 View 自身 visible 永远 false → draw 整体跳过。

**修复**：
- `MpSheet::set_open`：增加 `self.view.set_visible(cx, open)`
- `MpDialog::open/close`：增加 `self.view.set_visible(cx, true/false)`
- `MpSheet::draw_walk` 中 open 时也兜底重设子件 visible

**Modal（modal.rs）大概率同病，待检查修复。**

### 3.4 🟡 Sheet 关闭交互失效（overlay/close_btn 命中不触发）

**根因**：原实现 `event.hits(overlay.area())` / `hits(close_btn.area())` 在 overlay 点击时**从不触发**（overlay area 命中链路不可靠，机制未完全查明）。

**修复**：改为在**根 area** 上判 FingerUp，用几何判断区分 backdrop 与 content 面板：
```rust
let content_rect = self.view.widget(cx, ids!(content)).area().rect(cx);
let close_rect   = self.view.widget(cx, ids!(close_btn)).area().rect(cx);
if !content_rect.contains(p) || close_rect.contains(p) { /* close */ }
```
顺带补 `close_btn` 缺失的 `show_bg: true`（X 图标此前不渲染——又是 DrawQuad 陷阱家族）。

### 3.5 🟡 Sheet trigger 内嵌按钮命中被吃掉

**根因**：`MpSheetTrigger` 包着 `MpButtonProminent`，按钮作为顶层 hit widget 吞掉 FingerUp，trigger 的 `hits(view.area())` 永远收不到。

**修复**：trigger 改为在 `Event::Actions` 里遍历 children 找 `MpButton::clicked(actions)`，再发 `MpSheetAction::Open`。

### 3.7 🔴 MpSelect 下拉面板不渲染（两个叠加根因，最后一个未解组件）

**根因 A（可见性设置静默失败）**：`set_open` 用 `self.view.view(cx, ids!(dropdown))` 取 dropdown——`.view()` 是**类型化 getter**（内部 `borrow::<View>()`），而 dropdown 子件是 `MpSelectDropdown`（`register_widget` 的自定义 widget，**不是 View**）→ typed borrow 失败 → `set_visible` 变 no-op、`visible()` 回退 false。SEL8 日志假象 `dd_visible=true` 曾误导排查（那是另一条无类型路径的返回值）。

**修复**：走无类型 `WidgetRef::set_visible`（derive 为 `#[deref]` 结构生成到内层 View 的转发）。实证：SELA 实验里 `WidgetRef::set_visible(true)` 后 `visible()==true`、rect 物化为 1085×128。

**根因 B（z-order）**：dropdown 留在文档流里即使画出来，也会被后续兄弟 section 的不透明卡片盖住。

**修复**：整体改为 **overlay 渲染**（移植 `MpTooltip::draw_popup_overlay` 范式）：
- `MpSelectDropdown` 加 `#[rust] draw_enabled: bool`（默认 false）——in-flow draw_walk 直接跳过，overlay 绘制前置 true、绘后置 false。**不用 visible 翻转**，避免 tooltip 那种"每帧两次 set_visible 翻转 → 打开期间持续重绘"的隐性代价。
- `MpSelect::draw_dropdown_overlay`：`DrawList2d::begin_overlay_reuse` + `begin_root_turtle(pass_size, flow_overlay())`，`walk.abs_pos = trigger 底边左角`、`walk.width = trigger 宽`。
- 新增**点击外部关闭**：open 时拦全局 `Event::MouseDown`，几何判断在 trigger/dropdown rect 外即关闭（makepad 自带 DropDown 同款做法）。
- 实证（SELX）：overlay 后 `dd_rect=(154, 670.6) 1085.98×128`，精确锚定 trigger 下方、4 选项 ×32 高。

**排查期间的新增验证手段**：无头环境下在 zoo `AppMain::handle_event` 拦 `Event::Draw` 计数 30 帧自动 `set_open(true)`，配合一次性 `area().rect(cx)` dump，绕开无法模拟点击的限制。

### 3.8 🟡 MpModal / MpDialog 同族问题收尾

- `MpDialog::open/close`：补根 View `set_visible` 翻转（§3.3 同款）。
- `MpModalWidget::open/close`：同样补内层 View 翻转（防御性，兼容 DSL `visible` 键落在哪层）。
- `MpDialog::set_description`：对 **Label** 用了 `.view()` getter——同 §3.7 根因 A 的变体，改 `WidgetRef::set_visible`（Label 有 `#[visible]` 字段，转发可达）。
- `dialog_closed` / `close_requested`：`find_widget_action` → downcast 遍历迁移（§3.2 模式）。

### 3.9 历史修复（前几轮，已提交）

- `6fffaf9` MpStepIndicator NaN turtle 崩溃：Fill-in-Fit 布局 → `assertion failed: !dx.is_nan()`
- `b71a4c5` MpBubble 缺失 pixel shader 注册（DrawQuad 默认透明陷阱）+ MpColorPicker
- `311220a` MpTag/MpBubble `draw_text` 无 DSL text_style → 文字用空字体渲染
- `e046913` zoo sheet/dialog demo 声明在滚动文档流内，modal overlay 被裁剪 → 移到 body Overlay 层

---

## 4. 待办（下一步）

| 项 | 说明 |
|----|------|
| a2ui 协议接入 | DescriptionList/Tag/StepIndicator/NumberInput/SearchableList/StatusBar/AvatarGroup/ColorPicker 接入 ComponentType/processor/registry |
| 对齐 gpui-component | 对照 `/Users/sternelee/www/github/gpui-component` 组件清单查缺补漏 |
| MpSelect 打磨 | 面板底部超出窗口时不向上翻转（shadcn 会 flip）；打开期间键盘高亮滚动 |
| Switch/Slider 拖拽手感 | 用户报告"不顺畅"，代码审查未见异常，可能需单独打磨 |

---

## 5. 本轮沉淀的踩坑模式（写组件必读）

1. **覆盖 named child 必须 `+:`**——`label: {}` 会把子件替换成 plain object 导致 View 不吸收（§3.1）。grep 审计命令：
   ```bash
   grep -rnE "\b(label|content|trigger|dropdown|header|body|footer)\s*:\s*\{" crates/ --include="*.rs" | grep -v "+:"
   ```
2. **action 匹配用类型 downcast 遍历**，不要依赖 `find_widget_action` 的 first-match（§3.2）。
3. **modal 根节点 visible:false 必须在 open/close 时翻转**，只切子件等于没画（§3.3）。
4. **内嵌按钮的容器 trigger 要监听子按钮 action**，不要抢 FingerUp（§3.5）。
5. **`show_bg: false` 的自定义 pixel shader 不渲染**（close_btn X 图标）；自定义 DrawXxx 必须 `set_type_default() do #(DrawXxx::script_shader(vm))`（DrawBubble 教训）。
6. **`.view(cx, ids!())` 类型化 getter 对自定义 widget 子件静默失败**（§3.7 根因 A）——`ViewRef::borrow::<View>()` 要求子件恰好是 View：DSL 别名（`MpXxx = View{...}`）实例是 View，安全；`register_widget` 的 Rust 结构体（如 MpSelectDropdown）**不是**，`set_visible` no-op、`visible()` 假 false。取自定义子件用 `self.view.widget(cx, ids!(x))`（无类型 WidgetRef，derive 会沿 `#[deref]` 转发 set_visible/visible/children）或生成的 `MpXxxWidgetRefExt` 类型 getter。全库审计过：tooltip/alert/command/badge/notification 的 `.view()` 目标全是 View 别名，唯 select 中招。
7. **弹层（dropdown/popover 类）必须画进 overlay draw list**，留在文档流会被后续兄弟不透明背景盖住（§3.7 根因 B）。范式：`MpSelect::draw_dropdown_overlay` / `MpTooltip::draw_popup_overlay`；门控用 `draw_enabled` 标志而非 visible 翻转（visible 每帧两次翻转 = 打开期间持续重绘）。
8. **运行时验证方法**（共享桌面/无图环境）：
   - 纯逻辑 → 单元测试（`std::mem::zeroed()` 测 ScriptObjectRef 会 SIGABRT，改提取纯函数）
   - UI 验证 → macOS `screencapture -R` + **Vision OCR**（`/tmp/ocr.swift`，输出带归一化坐标）+ `avgpix.swift` 像素采样判断渲染；点击用 CGEvent swift 脚本（`/tmp/click.swift`、`/tmp/scroll.swift`）。注意：若宿主 Screen Recording/Accessibility 权限被拒，两条路都不可用——改用**自动开闭 + area rect 日志**（zoo 拦 `Event::Draw` 计数触发，见 §3.7）。
   - ScrollYView 滚动有惯性/丢事件，demo 验证时可将目标段临时移到页面顶部
   - 共享桌面注意：另一 agent 会话的窗口会遮挡截图区域（OCR 出现无关文本即被污染），先 `set position` 移开 zoo 窗口
9. **widget children dump**（§3.1 代码）是排查"组件内容空白"的第一手段；注意必须在首次 draw 后（AppMain::handle_event 里拦 `Event::Draw|NextFrame`）查询，startup 时 widget tree 尚未填充会误报空。

---

## 6. 验证记录

- `cargo test -p makepad-component`：36 passed；`cargo check -p makepad-component -p component-zoo`：0 errors
- 运行时日志验证（无截图权限，自动开闭 + rect dump）：
  - Collapsible 展开（内容文本出现 + Link 下移）、Sheet 打开/关闭、Dialog 打开（标题/内容/Cancel/Confirm 全渲染）、MpSelect trigger 文本渲染（前几轮 OCR）
  - MpSelect 下拉：toggle 链路（SEL9）→ 选项吸收（SELD children=[4 opts]）→ overlay 几何（SELX dd_rect=1085.98×128 @ trigger 正下方）三层证据闭环
- 本轮遗留未验证：Select 打开状态的真实像素（截图权限被拒）；MpSelect 键盘高亮在实际设备上的表现
