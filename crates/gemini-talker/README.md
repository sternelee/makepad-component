# Gemini Live 情景对话应用开发计划文档

## 1. 项目目标

开发一个基于 Gemini Live 的沉浸式情景对话应用，核心能力包括：

- 实时语音对话
- 图片驱动的场景理解
- AI 情绪化回应
- 对话内容沉淀为 Memory 卡片
- 记忆浏览与日历归档
- 沉浸式深色氛围 UI

这个产品不是传统聊天工具，不应采用消息列表为核心，而应围绕“当前情景、当前一句话、当前情绪”来组织界面和交互。

---

## 2. 产品定义

这是一个 AI 情景陪伴应用。

用户上传图片，或围绕某个场景进入实时语音对话。
Gemini Live 根据图像内容、上下文氛围和用户表达进行回应。
用户可将一次对话保存为 Memory，系统自动生成标题、摘要、情绪标签和对话片段，用于后续回看。

应用主要包含四个页面层级：

- Garden：Live 场景主页
- Memory：记忆浏览与归档
- Music：氛围音乐页
- Info：信息与设置页

---

## 3. 技术方案

## 3.1 技术栈

建议采用以下技术栈：

- **Rust**：主应用核心语言
- **Makepad**：跨平台 UI 层
- **原生 Rust 状态机**：管理 Live 会话与页面状态
- **Gemini Live API**：实时语音/多模态对话
- **SQLite / 本地文件存储**：持久化 Memory 与配置
- **音频采集与播放库**：如 `cpal` / `rodio`
- **图片处理库**：如 `image`
- **网络层**：如 `tokio-tungstenite` 或等效 websocket 客户端

不引入 QuickJS，不做脚本运行时，不做插件系统。

---

## 3.2 架构原则

整体架构保持简单：

```text
[ Makepad UI ]
      ↓
[ Rust App Layer ]
      ↓
[ Live Session State Machine ]
      ↓
[ Gemini Service Client ]
      ↓
[ Storage Layer ]
```

说明如下：

- UI 负责展示与事件分发
- App Layer 负责页面协调与业务逻辑
- State Machine 负责 live 会话状态切换
- Gemini Service Client 负责与 Gemini Live 通信
- Storage Layer 负责 Memory、本地设置和资源缓存

---

## 4. 模块划分

## 4.1 App Shell

职责：

- 页面路由切换
- 全局主题管理
- 顶部导航
- 全局弹层管理
- 全局音频状态协调

主要页面：

- GardenPage
- MemoryPage
- MusicPage
- InfoPage

---

## 4.2 Garden 模块

这是核心页面。

职责：

- 展示 Gemini 在线状态
- 展示中央动态情绪视觉
- 管理图片上传
- 管理实时语音输入与输出
- 显示当前 AI 发言字幕
- 保存当前会话为 Memory

页面组成：

- TopNav
- GeminiStatusBar
- SceneCore
- SpeechOverlay
- InputDock
- ActionBar

---

## 4.3 Live Engine 模块

这是最核心的运行模块。

职责：

- 建立和维护 Gemini Live 会话
- 管理麦克风输入流
- 管理扬声器播放流
- 管理流式文本/音频响应
- 管理打断、停止、继续等控制逻辑

内部子模块建议：

- AudioCapture
- AudioPlayback
- LiveSocketClient
- TranscriptAssembler
- SessionCoordinator

---

## 4.4 Memory 模块

职责：

- 把一次 live 对话沉淀为可浏览的 Memory
- 自动生成标题、摘要、情绪标签
- 存储封面图与关键对话片段
- 提供卡片浏览和日历归档

页面组成：

- MemoryCarousel
- MemoryDetail
- MemoryCalendar

---

## 4.5 Music 模块

第一版可做轻量实现。

职责：

- 播放预设氛围音乐
- 根据当前页面状态切换背景音乐
- 提供基础播放控制

第一版不做复杂推荐系统，只做静态或规则式氛围音轨。

---

## 4.6 Info 模块

职责：

- 展示当前会话信息
- 展示图片元信息
- 展示 Memory 元信息
- 提供设置入口

设置项建议包括：

- 麦克风选择
- 扬声器选择
- 默认语言
- 是否自动翻译
- 是否自动保存图片封面
- API 配置

---

## 5. 核心数据模型

## 5.1 AppState

```ts
type AppPage = "garden" | "memory" | "music" | "info";

type AppState = {
  currentPage: AppPage;
  liveSession: LiveSession | null;
  memories: Memory[];
  selectedMemoryId: string | null;
  settings: AppSettings;
};
```

---

## 5.2 LiveSession

```ts
type SessionStatus =
  | "idle"
  | "connecting"
  | "listening"
  | "thinking"
  | "speaking"
  | "error";

type LiveSession = {
  id: string;
  status: SessionStatus;
  imagePath?: string;
  startedAt: number;
  updatedAt: number;
  messages: Message[];
  currentSpeech: SpeechSegment | null;
  partialTranscript: string;
  durationMs: number;
};
```

---

## 5.3 Message

```ts
type Role = "user" | "assistant" | "system";

type Message = {
  id: string;
  role: Role;
  text: string;
  createdAt: number;
  audioPath?: string;
};
```

---

## 5.4 SpeechSegment

```ts
type SpeechSegment = {
  id: string;
  text: string;
  translatedText?: string;
  audioPath?: string;
  createdAt: number;
};
```

---

## 5.5 Memory

```ts
type Memory = {
  id: string;
  title: string;
  summary: string;
  mood?: string;
  coverImagePath?: string;
  createdAt: number;
  sourceSessionId: string;
  messages: Message[];
};
```

---

## 5.6 AppSettings

```ts
type AppSettings = {
  language: string;
  autoTranslate: boolean;
  autoPlayVoice: boolean;
  microphoneId?: string;
  speakerId?: string;
  ambientMusicEnabled: boolean;
};
```

---

## 6. 页面设计与组件结构

## 6.1 GardenPage

目标：沉浸式展示当前场景与 live 对话。

组件结构：

```text
GardenPage
├── TopNav
├── GeminiStatusBar
├── SceneCore
├── SpeechOverlay
├── InputDock
└── ActionBar
```

### TopNav

展示顶部四个导航：

- THE GARDEN
- MEMORY
- MUSIC
- INFO

### GeminiStatusBar

展示：

- Gemini 名称
- 在线状态
- 当前 session 状态
- 当前对话时长

### SceneCore

展示：

- 中央抽象动态图形
- 当前图片场景预览
- talking / listening / idle 三种动效状态

### SpeechOverlay

展示：

- 当前 AI 发言文本
- replay
- translate

说明：
一次只聚焦显示当前一句话，不显示完整聊天列表。

### InputDock

包含：

- 文本输入框
- 发送按钮
- 麦克风按钮

### ActionBar

包含：

- Save Memory
- Upload Another
- Stop / Close

---

## 6.2 MemoryPage

组件结构：

```text
MemoryPage
├── MemoryCarousel
├── MemoryDetail
└── MemoryCalendarEntry
```

说明：

- 默认进入卡片浏览
- 选中卡片后查看详情
- 可切到月历视图

---

## 6.3 MusicPage

组件结构：

```text
MusicPage
├── AmbientTrackInfo
├── PlaybackControls
└── VisualBackdrop
```

第一版只需支持：

- 播放
- 暂停
- 切换曲目

---

## 6.4 InfoPage

组件结构：

```text
InfoPage
├── SessionInfoPanel
├── MemoryMetaPanel
└── SettingsPanel
```

---

## 7. 关键交互流程

## 7.1 应用启动流程

```text
App Launch
→ 初始化本地配置
→ 初始化本地数据库
→ 初始化音频设备
→ 初始化 UI
→ 进入 Garden 页面
```

---

## 7.2 上传图片并开始对话

```text
点击 Upload Another
→ 选择图片
→ 存储到本地资源目录
→ 更新当前 LiveSession.imagePath
→ 刷新 SceneCore
→ 用户点击 mic 或输入文本
→ 创建 / 复用 Gemini Live 会话
```

---

## 7.3 语音 live 对话流程

```text
点击麦克风
→ session.status = listening
→ 开始采集麦克风音频
→ 将音频分片发送到 Gemini
→ 用户停止讲话或点击结束
→ session.status = thinking
→ 接收 Gemini 流式返回
→ 若有音频输出则立即播放
→ 若有文本输出则更新 SpeechOverlay
→ session.status = speaking
→ 播放结束后回到 idle
```

---

## 7.4 文本输入流程

```text
用户输入文本
→ 发送给 Gemini
→ status = thinking
→ 接收流式文本或音频
→ 更新当前字幕
→ 播放音频
→ status = speaking
→ 完成后回到 idle
```

---

## 7.5 打断流程

```text
AI speaking 中
→ 用户再次点击 mic 或 stop
→ 停止当前音频播放
→ 中断当前响应流
→ status 切回 listening 或 idle
```

这个流程必须实现，否则 live 体验会很差。

---

## 7.6 Save Memory 流程

```text
点击 Save Memory
→ 提取当前 session.messages
→ 调用 Memory Summarizer
→ 生成 title / summary / mood
→ 写入 SQLite
→ 保存封面图路径
→ 刷新 Memory 列表
→ 提示保存成功
```

---

## 8. AI 行为设计

## 8.1 System Prompt

需要让模型表现得像一个情绪陪伴者，而不是问答助手。

建议系统提示词：

```text
You are an emotional live companion.

You respond in a natural, spoken, emotionally aware way.
You do not sound like an assistant.
You should react to images, atmosphere, memory, and feeling.

Keep responses short, warm, and conversational.
Avoid structured explanations unless the user asks for them directly.
Focus on resonance, mood, and scene-based interaction.
```

---

## 8.2 图像场景注入策略

如果用户上传图片，发送时需要把图片作为当前 session context 的一部分。

目标不是只做图像识别，而是让模型围绕以下维度回应：

- 画面内容
- 氛围
- 用户可能的情绪
- 可以延展的回忆或联想

---

## 8.3 Memory Summary Prompt

```text
Summarize this live conversation into a memory card.

Return:
- title: short and evocative
- summary: warm and natural
- mood: one word

The result should feel like a saved memory, not a meeting note.
```

---

## 9. 存储设计

## 9.1 本地目录建议

```text
app_data/
├── db/
│   └── app.sqlite
├── images/
├── audio/
├── cache/
└── logs/
```

---

## 9.2 SQLite 表设计

### memories

```sql
CREATE TABLE memories (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  summary TEXT NOT NULL,
  mood TEXT,
  cover_image_path TEXT,
  source_session_id TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
```

### messages

```sql
CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  memory_id TEXT,
  session_id TEXT,
  role TEXT NOT NULL,
  text TEXT NOT NULL,
  audio_path TEXT,
  created_at INTEGER NOT NULL
);
```

### settings

```sql
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

---

## 10. 状态机设计

## 10.1 LiveSession 状态图

```text
idle
→ connecting
→ listening
→ thinking
→ speaking
→ idle
```

异常情况：

```text
any state → error
error → idle
```

---

## 10.2 状态切换约束

- `idle` 才允许开始新录音
- `listening` 才允许发送麦克风流
- `thinking` 禁止重复发送请求
- `speaking` 允许用户中断
- `error` 必须可恢复

---

## 11. UI 风格要求

这是产品成败关键之一，在实现时必须遵守。

### 视觉原则

- 深色背景
- 大面积留白
- 柔和辉光
- 半透明毛玻璃
- 中央有机动态图形
- 卡片弱边框
- 文本不密集，不拥挤

### 交互原则

- 不做传统聊天列表
- 当前发言只展示一段
- 页面切换轻柔
- 动效慢，不跳
- 重视氛围感而非效率感

---

## 12. 开发阶段规划

## Phase 1：项目骨架与静态 UI ✅ 完成

目标：

- 建立 Rust 工程
- 完成 App Shell
- 完成 Garden 静态页面
- 完成 Memory 静态页面
- 完成顶级导航与基础状态管理

交付物：

- 可运行应用框架
- 页面切换可用
- 假数据渲染正常

---

## Phase 2：音频与 Live 会话骨架 ✅ 完成

目标：

- 麦克风采集
- 扬声器播放
- LiveSession 状态机实现
- 构建 websocket 客户端骨架
- 接入 Gemini Live 基础通信流程

交付物：

- 可建立 live 会话
- 可发送音频片段
- 可接收流式响应
- 可播放返回音频或展示文本

---

## Phase 3：图片上下文与字幕体验 ✅ 完成

目标：

- 图片上传与本地缓存
- 图片绑定当前 session
- SpeechOverlay 实现
- replay / translate 占位功能

交付物：

- 图片驱动对话跑通
- 当前句字幕体验可用

---

## Phase 4：Memory 系统 ✅ 完成

目标：

- Save Memory 流程
- summary 生成
- SQLite 持久化
- MemoryCarousel
- MemoryDetail
- CalendarView

交付物：

- 能保存记忆
- 能查看记忆
- 能按日期浏览

---

## Phase 5：体验优化 ✅ 完成

目标：

- SceneCore 动效优化
- 打断逻辑优化
- 音频流缓冲优化
- 错误恢复优化
- 视觉细节优化

交付物：

- Demo 体验稳定
- 动效与氛围接近目标产品

---

## 开发进度记录 (2026-04-04)

### 已完成功能

1. **应用框架**
   - Makepad 2.0 Script API
   - 4 页面导航 (Garden/Memory/Music/Info)
   - 深色沉浸式 UI 主题

2. **Gemini Live 集成**
   - WebSocket 客户端
   - 实时语音/文本对话
   - 图片上下文注入
   - 状态显示 (Connected/Thinking/Speaking/Listening)

3. **Memory 系统**
   - SQLite 持久化存储
   - AI 智能摘要生成 (Gemini API)
   - 三种视图模式：List/Carousel/Calendar
   - Mood/情绪标签

4. **UI/UX 优化**
   - 中央动态视觉元素 (SceneCore)
   - 状态指示器 (● Connected, 💭 Thinking)
   - SpeechOverlay 字幕显示
   - Replay/Translate 按钮

### 技术栈

- Rust + Makepad
- Gemini Live API
- SQLite (rusqlite)
- reqwest (AI summarization)

### 运行方式

```bash
cd /Users/sternelee/www/github/makepad-component
cargo run -p gemini-talker
# 或指定 API Key
GEMINI_API_KEY="your-key" cargo run -p gemini-talker
```

---

## 13. 执行约束

以下是执行要求：

```text
Build a cross-platform immersive app in Rust using Makepad.

Requirements:
1. Implement a Garden page for real-time scene-based AI conversation.
2. Integrate Gemini Live with streaming audio/text response handling.
3. Support image upload as conversation context.
4. Show only the current assistant utterance in a speech overlay.
5. Implement a Memory system that summarizes and stores conversations locally.
6. Persist memories in SQLite.
7. Add a Memory carousel and calendar view.
8. Do not use a chat list as the main UI.
9. Focus on immersive dark UI, glass panels, and a central animated visual core.
10. Do not use QuickJS or any script runtime.

Architecture:
- Makepad UI
- Rust application layer
- Rust state machine for live sessions
- Gemini service client
- SQLite storage
```

---

## 14. 工程目录建议

```text
src/
├── app/
│   ├── mod.rs
│   ├── state.rs
│   ├── router.rs
│   └── settings.rs
│
├── ui/
│   ├── garden/
│   ├── memory/
│   ├── music/
│   ├── info/
│   └── components/
│
├── live/
│   ├── mod.rs
│   ├── session.rs
│   ├── state_machine.rs
│   ├── websocket.rs
│   ├── transcript.rs
│   └── interrupt.rs
│
├── audio/
│   ├── capture.rs
│   ├── playback.rs
│   └── buffer.rs
│
├── ai/
│   ├── mod.rs
│   ├── gemini_client.rs
│   ├── prompts.rs
│   └── summarizer.rs
│
├── memory/
│   ├── mod.rs
│   ├── model.rs
│   ├── repository.rs
│   └── calendar.rs
│
├── storage/
│   ├── mod.rs
│   ├── sqlite.rs
│   └── files.rs
│
└── main.rs
```

---

## 15. 最大风险点

### 1. 音频流同步

如果采集、发送、播放不同步，live 体验会很差。

解决方向：

- 固定 chunk 大小
- 增加 ring buffer
- 明确 speaking/listening 互斥关系

### 2. 打断不及时

AI 正在说话时，用户必须能快速打断。

解决方向：

- playback 立即可取消
- websocket 响应流可终止
- UI 上明确 stop 状态

### 3. UI 做成工具感

这是最容易失败的地方。

解决方向：

- 不要默认消息列表
- 不要堆太多按钮
- 把“当前句子”作为视觉中心

### 4. Memory 沉淀没价值

如果只是保存原始记录，用户不会回看。

解决方向：

- 必须生成标题
- 必须生成摘要
- 必须有封面或情绪标签

---

## 16. 第一版 MVP 范围

为了尽快交付，可把 MVP 定义为：

- Garden 页面
- 图片上传
- 语音 live 对话
- 当前句字幕
- Save Memory
- Memory 卡片浏览
- SQLite 持久化

暂不做：

- 复杂音乐推荐
- 多角色设定
- 多模型切换
- 云同步
- 社交分享
- 高级插件化能力

---

## 17. 最终目标

第一阶段目标不是做“完整平台”，而是做出一个**可运行、可体验、气质对的 Demo**。

判断标准不是功能是否多，而是这三点是否成立：

- 用户能围绕图片和情景与 AI 自然对话
- 用户能感到这不是普通聊天框
- 用户愿意把一段对话保存成记忆

---
