# makepad-component v2: bezel-style restyle + Table/Tree

参考 [gpui-bezel](https://github.com/) 的分层架构与中性现代设计语言，重构本组件库的主题体系并新增 Table / Tree 组件。

## 决策记录

1. **范围**：theme token 重构与新组件并行
2. **风格**：bezel 中性现代风（非 macOS HIG 彩色）
3. **新组件**：本轮仅 Table + Tree（其余进 backlog）
4. **兼容性**：全新命名，全量迁移，无 legacy alias；最终删除旧 `theme/colors.rs`、`theme/dark.rs`
5. **组织**：分层多 crate（bezel 式）

## 架构

```
crates/
├── motion/  makepad-motion     # 命名动画常量目录（FAST/BASE/SLOW…）
├── theme/   makepad-theme      # token + 色彩数学 + 双色板（仅依赖 makepad-widgets）
└── ui/      makepad-component  # 门面 crate：widgets + a2ui，re-export theme/motion
```

依赖单向向下 `ui → motion + theme`。

## Theme

单一真源在 Rust，脚本侧经 `#()` 注入生成三个命名空间：

- `mod.mpc_theme.dark = {...}` / `mod.mpc_theme.light = {...}` / `mod.mpc_theme = {...}`（活动副本）
- widget 只 import 活动副本；MpThemeState 把 light/dark 全量值堆拷贝到活动模块 + `request_script_reapply()`

### Token 清单

| 组 | Token |
|---|---|
| 表面阶梯 | BG SURFACE SURFACE_RAISED SURFACE_CARD SURFACE_DIALOG SURFACE_OVERLAY ELEMENT_HOVER ELEMENT_ACTIVE |
| 边框 | BORDER BORDER_STRONG DIVIDER |
| 文字三级 | TEXT TEXT_MUTED TEXT_FAINT |
| 反色板 | SOLID SOLID_HOVER ON_SOLID |
| 强调 | ACCENT ACCENT_HOVER ON_ACCENT |
| 状态色 | DANGER DANGER_HOVER DANGER_MUTED WARNING WARNING_MUTED SUCCESS SUCCESS_MUTED INFO INFO_MUTED BUSY |
| 组件专用 | INPUT_BG SELECTION CARET CODE_TEXT CODE_WASH TRANSPARENT |
| 布局 | RADIUS_SURFACE=12 RADIUS_PANEL=10 RADIUS_CONTROL=8 RADIUS_SMALL=6 inset_radius(outer,inset) |

色彩数学（crates/theme/src/color.rs）：`oklch(l,c,h)`、`neutral(l)`、`grey(v)` → Vec4f；`contrast_ratio(a,b)` WCAG 2.1。深色为默认外观，浅色按 "designed, not inverted" 单独设计。

### Motion

`INSTANT=0.0 FAST=0.10 BASE=0.15 SLOW=0.25`；曲线词汇 hover→Forward、瞬时→Snap、循环→Loop。animator duration 是否支持 `#()` 注入需编译探针验证；失败则 DSL 写 literal 并注释标注目录名。

## Widget 重样式规则

- 半径映射：popover/dialog/menu/card=12｜input/select/button/sheet=8｜chip/tag/badge=6；嵌套用同心规则
- hover 一律 ELEMENT_HOVER/ELEMENT_ACTIVE 半透明洗色
- 文字三级化：TEXT / TEXT_MUTED / TEXT_FAINT
- 按钮 variants 收敛：Prominent(bg=SOLID+ON_SOLID) / Ghost(muted+洗色) / Outline / Destructive；删除 Success/Warning 按钮 variant
- 旧 token 机械映射：BACKGROUND→BG FOREGROUND→TEXT MUTED_FOREGROUND→TEXT_MUTED MUTED→SURFACE MUTED_HOVER/ACTIVE→ELEMENT_* SECONDARY*→SURFACE_RAISED/ELEMENT_* PRIMARY*→ACCENT*/ON_ACCENT RING→ACCENT CARD→SURFACE_CARD INPUT→INPUT_BG SWITCH_*→SURFACE_RAISED/ELEMENT_*/SOLID SELECTION_FOREGROUND→ON_ACCENT INFO*保留

## 新组件

**MpTable**：`Column{label,width,sortable}` 配置 + `set_rows(Vec<Vec<String>>)`；sticky 表头、排序指示符、行 hover 洗色、`RowSelected(usize)` action、hairline 分隔。

**MpTree**（扁平模型）：Rust 侧 `TreeItem{label,depth}` 数组，展开态 `HashSet<usize>` 由 Rust 维护，缩进=depth×步进，chevron 旋转 FAST 动画，`ItemSelected(idx)` action。

## 迁移与验证

迁移顺序：新 crate → ui widgets 全量 token 迁移与重样式 → a2ui surface/chart_bridge 引用 → component-zoo + a2ui-demo → 删除旧模块。

验证：theme crate WCAG contrast_ratio 单测（light+dark 关键配对 ≥ AA）；cargo check 全部 crate（gemini-talker 豁免）；component-zoo 截图验收。
