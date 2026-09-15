# Agent Workbench 进度（架构转折版）

> **2026-09-16 架构转折**：agent 卡片不再是独立的会话类型，而是**终端会话的 chat 视图**。
> `claude`/`codex`/`pi` 跑在卡片的 PTY 里；chat 转录 = 它们 JSONL 输出的解析结果。
> 自研运行时（agent-core：provider 驱动、工具循环、沙箱、journal）**整体删除**，
> 模型/工具/认证/会话持久化全部归属 CLI 本身。

## 新架构

```
TerminalCard（vte 网格视图）      AgentChatCard（chat 转录 + composer）
        └──── 同一个 PTY 会话，◎/▮ 按钮切换 ────┘
                       │
        daemon: Session { chat: ChatState(adapter, on, seq, ring) }
          - PTY 字节流照旧进 ring（grid 重放无损）
          - chat 开启时，每行同时过 adapter（pi/claude/codex JSONL → ChatEvent）
          - 事件带 seq；客户端缺了就 ChatSync 补（ring 是权威）
        切换 = daemon 往 PTY 打字：/exit + 重进（resume 已知用 --session/--resume，
               未知用 -c/--continue 续最近会话）
        输入 = ChatSend → adapter 格式化（claude 常驻 JSONL stdin；pi/codex 每轮重进）
        检测 = ChatSwitch cli="auto" → ps 子树遍历找 pi/claude/codex
```

## 关键文件

- `src/chat/adapter.rs` — CliAdapter：launch/launch_continue/exit/parse_line/format_input
- `src/chat/event.rs` — ChatEvent + Sequenced（wire 形状）
- `src/chat/fold.rs` — ChatCardState：seq 去重、gap 提示、busy/usage/session_id
- `daemon.rs` — ChatState（暂停不丢弃）、fanout 双路、ChatSend/Switch/Sync、auto 检测
- `daemon_chat_tests.rs` — 3 个 e2e：解析+重放、切换脚本+输入格式化（cat 观测）、暂停保留 ring

## 历史章节

# Agent 工作台开发进度（canvas-terminal）

> 最后更新：2026-09-15 · 分支 `dev` · 基线 commit `620af9c`
> 目标：在无限画布上直接创建 agent 实例，完成完整对话与工作流。
> 实现方式：**全部自研**。参考项目 [waku](https://github.com/egoist/waku)（本地
> coding agent 控制平面）只取其架构方向，不引入其任何代码 —— waku 全部 crate
> 为 `GPL-3.0-only`，而本仓库是 `MIT OR Apache-2.0`，crate 依赖会造成许可证
> 污染；进程边界（本地 IPC + 自定义协议）则互不传染。

---

## 1. 总体进度

| 阶段 | 内容 | 状态 |
|------|------|------|
| 0 | canvas-terminal 两个 P0 修复（daemon 起不来 / `--remote` 失效） | ✅ |
| 1 | `agent-core`：无头 agent 运行时（新 crate） | ✅ |
| 2 | 接入 daemon：agent 与 PTY 平级的会话类型 | ✅ |
| 3 | 交互式审批：卡片点按钮，工具才执行 | ✅ |
| 4 | 画布卡片：`CanvasItem::Agent` + 转录渲染 + 内嵌 composer + 滚动 + Stop | ✅ 代码完成 |
| 4.1 | GUI 视觉验收 | ⏸ 被环境阻塞（见 §6） |
| 5 | 画布持久化（布局 + 转录）；工作流（goal / fork） | ⬜ 未开始 |
| 6 | 工具输出流式化（`ToolOutcome` → 增量） | ⬜ 未开始 |

验证基线：**104 个测试全部通过，连跑多次零失败**（`cargo test -p agent-core -p
canvas-terminal`）。此前曾达 101，本轮新增 3 个卡片客户端端到端测试并适配
`PromptSubmitted` 事件后为 104。

## 2. 架构

```
┌─ 画布层 canvas-terminal ────────────────────────────────┐
│ CanvasItem::Agent 卡片                                  │
│   transcript（AgentCardState 折叠）· composer · Stop ·  │
│   审批按钮 · 用量条 · 滚动                              │
└──────────────┬──────────────────────────────────────────┘
               │ 本地 IPC（既有帧协议扩展：agent 帧族）
┌──────────────▼──────────────────────────────────────────┐
│ daemon（canvas-terminal --daemon，同 binary 双模式）      │
│   AgentEntry 表 + 事件泵线程 + Approvals 审批中枢        │
│   会话跨 GUI 重启存活；journal 为转录的权威              │
└──────────────┬──────────────────────────────────────────┘
               │ AgentDriver（agent-core 的 Provider trait）
┌──────────────▼──────────────────────────────────────────┐
│ Provider：OpenAI 兼容（ureq 流式 SSE）/ Scripted（测试） │
└─────────────────────────────────────────────────────────┘
```

设计原则：`agent-core` 不知道 Makepad 的存在；daemon 不持 UI 状态；卡片只
发命令、只收事件。

## 3. 代码分布

| 位置 | 行数 | 内容 |
|------|------|------|
| `crates/agent-core/src` | 3286 | cancel / error / event / identity / message / provider(openai, scripted) / session / tool / tools(files, search, shell) |
| `crates/agent-core/tests` | 1408 | 会话循环 21 · 工具 9 · 单元 26 · 文档 1 |
| `crates/canvas-terminal/src/agent` | 1015 | `client.rs`（GUI 侧客户端）+ `card.rs`（事件折叠模型） |
| `crates/canvas-terminal/src/daemon.rs` | 1141 | agent 表 / 事件泵 / 审批中枢 |
| `crates/canvas-terminal/src/ipc.rs` | 779 | agent 帧族 + 审批帧 |
| `crates/canvas-terminal/src/daemon_agent_tests.rs` | 747 | 端到端 11 个 |
| `crates/canvas-terminal` 其余 | — | canvas/items/command/main/terminal::session 接线 |

新增依赖仅 `regex` 与 `ureq`，二者此前已在 `Cargo.lock` 中。

## 4. 关键设计决策（含理由）

### 4.1 会话并发契约（都有回归测试守着）

1. **`Drop` 永不阻塞**：worker 可能停在 provider 调用里，`JoinHandle::join`
   无超时，在 UI 线程上 drop 会卡死界面 → `Drop` 只发信号并 detach。
2. **cancel 能进到调用内部**：`Provider::complete` 收 `CancelToken`，驱动在
   每个 SSE chunk 之间检查。实测取消延迟 1.85s → 12.5ms。
3. **`TurnFinished` 发布时 `history()` 已完整、`is_busy()` 已为 false**：
   daemon 要在终态事件到达时持久化转录，读旧快照会损坏数据。
4. **steer 只进它写给的那一轮**：steer 带轮次标签，避免「检查 busy → 轮次
   恰好结束 → 消息漏进下一轮」的窗口。

### 4.2 身份与恢复：`(session_id, epoch, seq)` 三元组

只有 `seq` 会在 daemon 重启后串档（计数器从 1 重来，旧游标 37 恰好落在新
会话已发出的区间内）。`epoch` 每进程启动重铸，旧游标被判定为
`Replay::Stale`，客户端丢弃转录重来而不是错配到别的会话。

### 4.3 工具的「存在性」与「许可」分离

`enabled_tools` 决定模型**看见**哪些工具（省 schema token，也不会反复请求
拿不到的东西）；`PermissionGate` 决定**这一次准不准**。两道闸在 session
循环里先后生效，顺序不可换：存在性不过，调用方的门连问都不问。

### 4.4 daemon 不持 UI 状态，审批只转发

`Approvals` 中枢存 `approval_id → 发送端`，门在 worker 线程阻塞等待（不影响
泵、其他 agent、runtime）。失败方向固定为「拒绝」：没人连着、超时、通道断、
用户点拒绝，全都 deny。

### 4.5 agent 事件独立发送队列

终端一次 `cargo build` 可吐出数百 KB 输出，若与 agent 事件共队列会挤掉转录
事件。连接持双队列（control 512 / agents 4096）+ `tokio::select!` 写出。
丢帧语义也不同：终端丢字节就是丢了；agent 丢事件留下 `seq` 空洞，客户端
检测到后用 `after_seq` 从 journal 补齐。

## 5. 踩坑记录（后续写代码必读）

1. **`#[deref]` 后的字段顺序会破坏 GPU instance buffer**（makepad shader
   struct 约束）；自定义 draw 的非 instance 数据必须在 `#[deref]` 之前。
2. **持锁断言会毒化 mutex 并带走 worker 线程**。测试里先拷值再断言。
3. **`TurnStarted` 与 provider 调用真正开始之间有窗口**。等这个事件不等于
   provider 已被调用；需要确定性的测试应让 provider 自己发「已进入」信号。
4. **`SystemTime` 在 macOS 是微秒精度**。并行测试用 nanos 做 unique 后缀仍会
   撞名，必须叠加进程内原子计数器。
5. **clippy `approx_constant` 属 correctness 组（默认 deny）**。颜色通道
   `0.318`（#374151 的 81/255）被误判为 1/π，整个 crate 的 clippy 因此失败；
   定向 `#[allow]` + 注释即可。
6. **rmux-ipc 不创建 socket 父目录**。`endpoint_for_label` 解析到
   `<tmp>/rmux-<uid>/<label>`，bind 前必须 `create_dir_all` —— 否则在从未装
   过 rmux 的机器上 daemon 100% 起不来（这正是 bundled daemon 要服务的场景）。
7. **手写 `app_main` 会丢 `--remote`**。`remote::start_if_requested()` 在宏里
   调用；为 CEF bootstrap 手写入口的 crate 必须自己补这一行
   （参照 makepad 自家 `apps/mpbrowser`）。
8. **只读沙箱挡不住符号链接**。词法检查 `../../..` 之外，还要对「最深已存在
   祖先」做 canonicalize 复检；root 不存在时跳过（否则 macOS `/tmp` →
   `/private/tmp` 会把所有合法路径判成逃逸）。

## 6. 已知限制与阻塞

| 项 | 说明 |
|----|------|
| GUI 视觉验收未做 | 本机 macOS 弹出 Xcode 许可协议，系统级拒绝链接器与 `/usr/bin/git`，`/g` 截图路径也随之失效（component-zoo 同样超时，与代码无关）。许可解决后跑 `AGENT_PROVIDER=scripted cargo run -p canvas-terminal` → `/new agent demo` 即可验收。代码路径已被「卡片客户端端到端」测试覆盖。 |
| 交互审批是模态的 | 等待期间该 agent 的回合暂停（worker 阻塞在门上），不阻塞泵/其他 agent/runtime |
| 工具输出非流式 | `ToolOutcome` 一次性返回；60 秒的构建过程卡片无反馈。改 `Tool::invoke` 加 sink 是破坏性变更，已排期 |
| agent 卡片不能收纳到 dock | dock 的收纳槽是终端专用元组；放宽需要一并改最小化/恢复 |
| journal 双写内存 | 卡片端 log 与折叠 rows 各持一份（约 2×）。长会话可裁剪已折叠的 delta |
| composer 单行 | `is_multiline: false`；多行 prompt 需换 multiline + Shift+Enter |

## 7. 使用

```bash
# 脚本化 agent（无网络、无 key），用于 UI 验收
AGENT_PROVIDER=scripted cargo run -p canvas-terminal
# 命令栏输入：/new agent demo

# 真实模型（与 a2ui bridge 同一套环境变量）
export LLM_API_URL=https://api.moonshot.ai/v1/chat/completions
export LLM_API_KEY=sk-...
export LLM_MODEL=kimi-k2.5
cargo run -p canvas-terminal    # /new agent demo

# 卡片交互
#   双击卡片      打开 composer（Enter 发送 / Esc 取消）
#   滚轮          滚动转录历史
#   ■ stop        取消运行中的回合
#   Allow/Deny    审批工具调用
#   @demo text    点名向 agent 发送
#   ✕             关闭并终止 agent
```

## 8. 提交切分建议

1. `agent-core`: headless agent runtime（provider/tools/session/取消/恢复）
2. `canvas-terminal`: agent daemon 协议与审批（ipc/daemon + 端到端测试）
3. `canvas-terminal`: 画布 agent 卡片（items/canvas/command/client/card）
