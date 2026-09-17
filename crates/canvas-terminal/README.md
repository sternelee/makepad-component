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
- ✅ 左侧白板工具栏（Move/Pen/Rect/Circle/Text/Eraser 等，手绘风矢量图标）
- ✅ 手绘（rough.js 风格）白板笔触：确定性抖动 + 双描边，seed 固定不闪烁
- ✅ 便利贴风格 Note 卡片：奶油纸面、铅笔抖动边、和纸胶带、深墨文字
- ✅ 嵌入 CEF 浏览器
- ✅ 通过 `＋` 菜单或命令创建终端/浏览器/便签/音乐播放器
- ✅ 持久终端会话：daemon 跨 GUI 重启持有 PTY，下次启动自动 re-attach 并回放 scrollback
- ✅ 拖拽 PNG/JPG/WebP/GIF/SVG 等图片到画布原生预览（`Image` 组件槽位，等比缩放）
- ✅ 拖拽 MP4/MOV/WebM 等视频到画布原生播放（`Video` 组件 + 平台播放后端，静音自动播放）
- ✅ 拖拽 PDF 到画布原生渲染（makepad `PdfPageView`，首页按卡片尺寸 letterbox），标题栏带打印按钮（`lp` 发送系统打印队列）
- ✅ 拖拽 HTML 到画布内嵌浏览器预览（CEF `file://` 槽位，完整网页渲染）
- ✅ 拖拽文本文件（txt/md/代码/配置等 40+ 扩展名）为等宽文本卡片，支持滚轮滚动、截断标记
- ✅ 会话恢复：daemon 自身持久化一份会话注册表（名字/argv/cwd/托管的 CLI），
  即使 daemon 进程重启（而不仅仅是 GUI 重启），也能按名 `Resume` 出会话——
  普通终端卡片恢复到原来的工作目录，agent 卡片继续用 `--resume`/`--session`
  接上原来的对话。
- ✅ Agent 互相通信：`canvas-terminal ipc` 子命令（`list`/`send`/`wait`/`read`）
  是 daemon 协议之上的一层薄 CLI；daemon 给每个卡片的 PTY 都注入
  `CANVAS_TERMINAL_ENV=1`，卡片里跑的 CLI agent 据此判断可以安全地调用这个
  子命令去给同画布里的另一张卡片发消息、等它读到空闲、或读它的转录。

## 运行

```bash
cargo run -p canvas-terminal
```

需要桌面环境（macOS / Linux / Windows）。首次运行会拉取 Makepad 与 CEF 依赖。
终端会话由同一二进制以 `--daemon` 模式持有——GUI 启动时自动以 detach 方式重新
执行自己并加上 `--daemon`，无需系统安装 rmux。如需单独运行/调试 daemon：

```bash
cargo run -p canvas-terminal -- --daemon
```

## 命令栏

底部输入框支持以下命令：

| 命令 | 说明 |
| ------ | ------ |
| `@name message` | 向名为 `name` 的终端发送原始文本 |
| `/new note` | 在画布中心创建便签 |
| `/new terminal NAME[:CWD]` | 创建终端，可指定工作目录 |
| `/new browser URL` | 创建嵌入浏览器 |
| `/new music TITLE` | 创建音乐播放器卡片 |
| `/focus NAME` | 聚焦到指定终端 |
| `/rename OLD NEW` | 重命名终端 |
| `/open FILE` | 打开本地图片/视频/PDF/文本为媒体卡片（按扩展名分流） |
| `/zoom 1.5` | 设置画布缩放 |
| `/grid` | 切换网格/星空背景叠加 |
| `/clear` | 清除所有白板涂鸦 |
| `/help` | 显示用法 |
| 其他文本 | 发送到当前激活的终端 |

## Agent 互相通信（`canvas-terminal ipc`）

卡片内的 CLI agent（或任意脚本）可以直接调用同一个二进制来控制画布上的其他
卡片，不需要链接 GUI 代码，只是 daemon 协议之上的一层薄客户端：

```bash
canvas-terminal ipc list                               # 列出所有会话及其 busy 状态
canvas-terminal ipc send --to NAME --text "go on"      # 给名为 NAME 的会话发一条消息
canvas-terminal ipc wait --for NAME --until idle       # 等到 NAME 变为 idle（阻塞，daemon 端事件驱动，非轮询）
canvas-terminal ipc read --from NAME --lines 50        # 读取 NAME 的转录（折叠后的纯文本）
```

`wait`/`send`/`list`/`read` 内部分别对应 daemon 新增的 `AgentWait` /
`ChatSend` / `AgentList` / `ChatSync` 请求；`wait` 的退出码：`0` 已到达目标
状态、`1` 超时（`--timeout-ms`，默认 30000）、`2` 用法或请求错误。daemon 会
给它启动的每个 PTY 注入 `CANVAS_TERMINAL_ENV=1`，一个跑在卡片里的 agent 可以
先检查这个变量再决定是否调用 `ipc`（同一台机器上、canvas-terminal 之外的
进程不会误用这个 socket）。

## 交互

- **平移**：鼠标滚轮 / 触控板滑动（默认），或中键拖拽。
- **缩放**：按住主修饰键（⌘ / Ctrl）+ 滚轮。
- **选中/拖拽卡片**：`Move` 工具或默认点击卡片标题栏。
- **调整大小**：拖动卡片右下角手柄。
- **便签编辑**：双击便签正文进入行内编辑，`Esc` 提交。
- **白板涂鸦**：左侧工具栏选择 Pen/Rect/Circle/Text/Eraser 后在画布上绘制。
- **拖放媒体**：把 Finder 里的图片/视频/PDF/文本/HTML 文件直接拖到画布上，松手即在落点创建预览卡片
  （拖入时画布出现蓝色高亮框；多个文件级联摆放，不支持的扩展名会在状态栏提示）。
- **工作区**：顶部标签栏切换、添加画布。

## 架构

```
crates/canvas-terminal/
├── src/main.rs          # Makepad script_mod! DSL、App 入口；`--daemon`/`ipc` 时转运行对应模式
├── src/daemon.rs        # PTY daemon 运行时：持有 PTY、IPC 监听、会话表、scrollback ring、agent 状态与 wait
├── src/daemon_persist.rs # daemon 自己的会话注册表（名字/argv/cwd/托管 CLI），独立于 GUI 的画布存档
├── src/ipc.rs           # GUI ↔ daemon 线协议（length-prefixed，控制帧 JSON / 字节帧内联）
├── src/ipc_cli.rs       # `canvas-terminal ipc <verb>`：daemon 协议之上的一次性 CLI 客户端
├── src/canvas.rs        # CanvasPanel：画布、项管理、事件、绘制、媒体槽位
├── src/items.rs         # CanvasItem、NoteShape、NoteTool、AgentStatus、MediaKind
├── src/command.rs       # 命令栏解析
└── src/terminal/        # 终端会话与渲染
    ├── session.rs       # IPC 客户端：连 daemon、Create/Attach/Resume、喂本地 vte 网格
    └── state.rs         # vte 解析的本地终端网格（alacritty 语义）
```

媒体预览通过 DSL 里声明的隐藏组件槽位实现：图片用 `Image` 槽 ×8
（browser-slot 模式：绘制时可见、绘完隐藏），视频用 `Video` 槽 ×2、PDF 用
`PdfPageView` 槽 ×2（二者无 `visible` 开关，声明为 0×0，由 CanvasPanel 在
子控件 pass 之后用显式 abs_pos Walk 绘制在卡片内容区）。槽位池满时新卡片
显示占位提示，关闭旧卡片即可释放。

只有一个二进制 `canvas-terminal`：默认启动 GUI，带 `--daemon` 参数时则以 PTY
daemon 模式运行。终端会话由这个 daemon 持有（跨平台 PTY 走 `rmux-pty`，本地 IPC
走 `rmux-ipc`：Unix domain socket / Windows named pipe）。GUI 启动时若连不上
daemon，会以 detach 方式重新执行自己并加上 `--daemon`，之后所有终端的 PTY 都由
它管理——不依赖系统安装的 rmux，也无需单独的 daemon 二进制。GUI 退出后 daemon
继续存活，下次启动时 GUI 会自动 `List` 存活会话并按名 `Attach`，回放 scrollback
后继续实时输出。`/rename` 会同步 daemon 里的会话名，关闭终端卡片（✕）会 `Kill`
对应会话。本地终端网格（`TerminalState`，vte 解析）仍在 GUI 侧维护，daemon 只
负责持有 PTY、转发字节、缓存重放缓冲。

daemon 自己也持久化一份会话身份注册表（`daemon-sessions.json`，与 GUI 的
`workspace.json` 分开存放）：`Create`/`Rename`/学到 agent 的 CLI 会话 id 时都
会写入。哪怕 daemon 进程本身重启（不只是 GUI），`List` 也会把注册表里"活的
会话表里没有、但注册表还记得"的名字标成 `resumable`，调用 `Resume` 就能按
记录的 argv/cwd 重新拉起会话，并在 agent 卡片的情况下自动用
`--resume`/`--session` 接回原来的 CLI 会话。

## 已知限制

- 行内便签编辑由隐藏 `TextInput` 代理，IME 候选窗口可能出现在屏幕左上角，不影响输入。
- 字体换行目前使用固定字符宽度估算，CJK/等宽字体下可能略有偏差。
- 测试覆盖主要集中在 `command` 与 `terminal/session`；UI 行为通过 `cargo run` 手动验证。
- 媒体槽位上限：图片 8、视频 2、PDF 2。槽位满时新卡片显示占位提示（不抢占已用槽位）。
- 视频编解码取决于平台播放后端（macOS AVFoundation / Windows Media Foundation /
  Linux GStreamer）；不支持的编码会在视频卡片内显示错误状态。
- PDF 目前渲染第一页；多页浏览/缩放待后续（`PdfPageView` 本身支持 zoom）。
- 文本卡片为只读预览：每文件读取上限 256KB（超出显示截断标记），长行水平裁剪不换行。
- PDF 打印走系统 `lp` 命令（CUPS）发送默认打印队列；未配置打印机会在状态栏报错。
- 视频/PDF 卡片内容区点击属于内嵌播放器（播放控制/翻页），拖动卡片请使用标题栏。
- 给 `draw_title` / `draw_cell_text` 设置文字颜色必须直接赋值 `.color` 字段；
  `draw_vars.set_dyn_instance(live_id!(color), ...)` 对这些 shader 无效。
