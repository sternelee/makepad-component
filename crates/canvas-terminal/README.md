# canvas-terminal

基于 [Makepad](https://github.com/makepad/makepad) 的无限画布终端工作区（CNVS 风格）。把多个终端、浏览器、便签、音乐播放器放在同一块画布上，用命令栏统一驱动。

## 开发状态

已可用，持续迭代中。

- ✅ 动态星空/自然背景（可替代网格）
- ✅ 卡片式布局：圆角、投影、头像、选中发光
- ✅ 底部命令栏与候选提示、命令历史
- ✅ 右侧属性面板（Note 标题/正文/颜色/字号实时编辑）
- ✅ Note 卡片：双击正文行内编辑，支持 IME/中文输入
- ✅ 音乐播放器卡片
- ✅ Agent 终端头像与状态指示
- ✅ 多画布工作区切换
- ✅ 左侧白板工具栏（Move/Pen/Rect/Circle/Text/Eraser 等）
- ✅ 嵌入 CEF 浏览器
- ✅ 通过 `＋` 菜单或命令创建终端/浏览器/便签/音乐播放器

## 运行

```bash
cargo run -p canvas-terminal
```

需要桌面环境（macOS / Linux / Windows）。首次运行会拉取 Makepad 与 CEF 依赖。

## 命令栏

底部输入框支持以下命令：

| 命令 | 说明 |
|------|------|
| `@name message` | 向名为 `name` 的终端发送原始文本 |
| `/new note` | 在画布中心创建便签 |
| `/new terminal NAME[:CWD]` | 创建终端，可指定工作目录 |
| `/new browser URL` | 创建嵌入浏览器 |
| `/new music TITLE` | 创建音乐播放器卡片 |
| `/focus NAME` | 聚焦到指定终端 |
| `/rename OLD NEW` | 重命名终端 |
| `/zoom 1.5` | 设置画布缩放 |
| `/grid` | 切换网格/星空背景叠加 |
| `/clear` | 清除所有白板涂鸦 |
| `/help` | 显示用法 |
| 其他文本 | 发送到当前激活的终端 |

## 交互

- **平移**：鼠标滚轮 / 触控板滑动（默认），或中键拖拽。
- **缩放**：按住主修饰键（⌘ / Ctrl）+ 滚轮。
- **选中/拖拽卡片**：`Move` 工具或默认点击卡片标题栏。
- **调整大小**：拖动卡片右下角手柄。
- **便签编辑**：双击便签正文进入行内编辑，`Esc` 提交。
- **白板涂鸦**：左侧工具栏选择 Pen/Rect/Circle/Text/Eraser 后在画布上绘制。
- **工作区**：顶部标签栏切换、添加画布。

## 架构

```
crates/canvas-terminal/
├── src/main.rs      # Makepad script_mod! DSL、App 入口
├── src/canvas.rs    # CanvasPanel：画布、项管理、事件、绘制
├── src/items.rs     # CanvasItem、NoteShape、NoteTool、AgentStatus
├── src/terminal/    # PTY 会话与终端渲染
├── src/command.rs   # 命令栏解析
└── Cargo.toml
```

## 已知限制

- 行内便签编辑由隐藏 `TextInput` 代理，IME 候选窗口可能出现在屏幕左上角，不影响输入。
- 字体换行目前使用固定字符宽度估算，CJK/等宽字体下可能略有偏差。
- 测试覆盖主要集中在 `command` 与 `terminal/session`；UI 行为通过 `cargo run` 手动验证。
