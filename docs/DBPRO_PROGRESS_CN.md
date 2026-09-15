# DbPro 开发进度

> TablePro 风格的跨平台数据库 GUI — Makepad 2.0 `script_mod!` + Rust。
> 日期：2026-09-15 ｜ 代码量 ~7,100 行（app 3,722 / db 2,539 / grid 360 / tab_bar 447）

## 当前状态：**MVP 功能完整，可日常使用** ✅

- 构建：`cargo check -p dbpro` / `cargo build -p dbpro` ✅（需 CLT clang 在 PATH 前面）
- 测试：`cargo test -p dbpro` — **10/10 通过** ✅
- 运行时：`grep -c '\[E\]'` 应用日志 = **0**（DSL/shader/apply 全干净）、无 panic、无缺字形 ✅
- 未验证：视觉细节依赖用户截图反馈（本环境 screencapture 返回黑屏，TCC 限制）

## 功能清单（全部已实现）

### 连接管理
- [x] 连接库持久化 `~/.dbpro/connections.json`（密码不落盘）
- [x] 连接对话框：Test / Connect / Edit / Delete / Disconnect（编辑已连接配置时可见）
- [x] 分组（Group 字段 → 侧栏 📁 文件夹，按首现顺序）
- [x] SSH 隧道（实验性）：`ssh -N -L` 子进程 + BatchMode 密钥认证 + 端口轮询 10s；重连自动替换旧隧道
- [x] 断开连接（⏏ 移入 Edit 对话框）、重新连接（点击侧栏 ○ 连接）
- [x] Schema 刷新（侧栏 "Sync"，不重连）
- [x] 后台 COUNT 扫描（cap 200 表）→ 侧栏显示行数 `users (48)`
- [x] 侧栏过滤框（表名小写包含匹配）

### 数据浏览
- [x] 表/视图树（视图带 `(view)` 后缀）；点击表打开标签页
- [x] 每标签独立状态（页码/过滤/排序/缓存数据/DDL/索引/FK 元数据），切标签零重查
- [x] 标签绑定所属连接 → 多连接并排浏览；多连接时查询标签带连接名
- [x] 动态标签条 DbTabBar（自绘组件：标题+× 成组居中、选中亮色+ELEMENT_ACTIVE 底板、宽度自适应+截断）
- [x] 网格 = makepad DataGrid：行号列、斑马纹、单元格选择（拖框选）、键盘导航、列宽拖拽
- [x] **列宽铺满面板**（`cx.turtle().rect()` 实测容器宽，内容宽+均摊，版本/缩放时一次应用无闪跳）
- [x] 数值列右对齐 + 浮点显示清理（>4 位小数 round4+去尾零；编辑/导出保留全精度）
- [x] 服务端分页 200 行/页、点表头服务端 ORDER BY（升降循环、指示符跨页保留）
- [x] 服务端文本过滤（前 10 列 LIKE/ILIKE，0.4s 防抖）
- [x] ⌘C 复制选区 → TSV（set_copy_provider）

### 写回
- [x] 单元格行内编辑（双击/F2/直接输入）：`UPDATE … WHERE pk` + 乐观更新 + 服务端校验（0 行命中/失败自动重载）
- [x] Enter 提交 / Esc 取消 / 点击别处自动提交 / 提交后焦点回网格
- [x] Delete 键置 NULL（单选格，nullable 校验）
- [x] ＋ Row：`INSERT … DEFAULT VALUES`（MySQL 用 `() VALUES ()`）→ 跳末页行内补值
- [x] ⧉ Duplicate：整行复制（整型 PK 省略自动分配新键）
- [x] － Row：删除确认对话框（显示确切 DELETE 语句）
- [x] FK 感知编辑：外键列下拉 `id — name`（500 条），MpDropdown 单元格模板，弹层 overlay 不受裁剪
- [x] 护栏：无 PK（视图/PK-less）只读、NOT NULL 拒空、FK 违规如实报错

### 查询
- [x] 多查询标签，各自记忆 SQL；真编辑器 TextInput（multiline，⌘Enter=Returned 原生）
- [x] 结果网格（≤1000 行）+ rows affected + 耗时；示例查询按钮
- [x] 历史持久化 `~/.dbpro/history.json`（100 条）+ ⌘↑↓ 翻阅
- [x] ⌘R 刷新/重跑

### 结构视图（Data | Struct 切换）
- [x] 列表（名称/类型/可空/PK）
- [x] 索引列表（SQLite PRAGMA / MySQL SHOW INDEX 分组 / PG pg_indexes 解析）
- [x] DDL（SQLite sqlite_master / MySQL SHOW CREATE TABLE / PG 提示 pg_dump）

### 数据进出
- [x] ⬇ CSV 当前页导出；⬇ All 全表流式导出（worker 逐页写文件 + 实时进度 + **二次点击取消**）
- [x] ⬆ Import CSV：RFC4180 解析（引号内逗号/换行、"" 转义）+ 表头↔列名匹配（大小写不敏感、未知列报错）+ 多行 VALUES 单语句原子插入 → 跳末页
- [x] ⌕ Detail 行详情对话框（全列）
- [x] Data|Struct 双按钮可见性切换（Secondary/Ghost）

### Demo
- [x] 首跑自动 seed SQLite demo（users 48/products 36/orders 120/settings 3 + order_summary 视图）并自动连接、自动打开首表

## 测试覆盖（10 个）

| 测试 | 覆盖 |
|---|---|
| test_demo_seed_and_sqlite_roundtrip | seed + 分页/搜索/排序/任意 SQL/DDL/DML + **FK 元数据两对 + 视图无 FK + 选项 48 条** + **索引（users 空/tagged UNIQUE）** |
| test_writeback_roundtrip | UPDATE→NULL→DELETE→FK 报错 + DEFAULT VALUES 插 settings + users 上失败 + table_count + fetch_ddl |
| test_insert_copy_roundtrip | 复制行（PK 省略断言 + 行数+1 + 名字副本） |
| test_parse_csv | 5 组解析断言（引号/转义/CRLF/无尾换行/空字段） |
| test_build_insert_rows_mapping | 列映射/大小写/未知列/空 header（**抓到过列错位真 bug**） |
| test_sql_builders | UPDATE/DELETE/转义（MySQL 反斜杠）/组合键/无键拒绝 |
| test_connection_persistence_roundtrip | connections.json 读写 |
| sidebar_tests ×3 | 未分组/分组+计数+过滤/空 schema（纯函数 build_sidebar_model） |

## 架构

```
crates/dbpro/
├── db.rs     驱动层：DbConn(rusqlite/mysql/postgres) + SQL builders(UPDATE/DELETE/INSERT/
│             DEFAULT/insert-copy) + CSV parse + workers(std::thread + Cx::post_action)
│             + SSH 隧道 + 历史/连接持久化 + demo seed
├── grid.rs   DbGridHost：宿主 makepad DataGrid（模板 Editor/FkEditor、copy provider、
│             数值格式化/右对齐、列宽铺满）
├── tab_bar.rs DbTabBar：动态标签条（自绘 + Areas 命中 + actions）
└── app.rs    App：script_mod! UI + 全部逻辑（per-tab 状态机、模式切换、对话框×4）
```

线程模型：连接在 `HashMap<u64, Arc<Mutex<DbConn>>>`，worker 线程锁连接执行、`Cx::post_action(DbAction)` 回 UI；响应带请求标识丢弃过期结果。

## 迭代日志（21 轮）

| 轮 | 内容 |
|---|---|
| 1 | 骨架：三驱动 + 侧栏 + 分页网格 + SQL 编辑器 + 连接对话框 + demo |
| 2 | 每标签独立状态、多连接绑定、Structure 视图、历史、CSV 导出、防抖、状态灯 |
| 3 | 可编辑网格（DataGrid 双模板）+ UPDATE/DELETE 回写 + FK 顺序 |
| 4 | 复制行 / 全表导出 / schema 刷新 / SSH 隧道 |
| 5 | ＋ Row(DEFAULT VALUES) + DDL 查看器 + settings 表 |
| 6 | ⌘C TSV 复制 / 删除确认 / 历史持久化 |
| 7 | Delete 置 NULL / 导出取消 / 侧栏过滤 / 焦点回归 |
| 8 | CSV 导入（解析器抓到列错位 bug） |
| 9 | 连接分组（build_sidebar_model 纯函数+测试） |
| 10 | FK 感知编辑（三驱动 FK 元数据 + MpDropdown 单元格模板） |
| 11-13 | 截图驱动 UX：tofu 清理（⟳▤◫⏏⌕→Reload/(view)/Sync）、工具栏重排、数值右对齐+格式化 |
| 14-15 | 崩溃修复（area() 双借用）、树居中、字形清单 |
| 16-21 | 网格铺满（turtle 宽）、模式切换双按钮、标签居中+选中态、**透明色根因**（c_text DSL 默认值缺失） |

## 关键经验（踩坑记录，详 AGENTS.md §3 gotchas）

1. `#[live]` 颜色字段**必须**有 DSL 默认值，否则全零透明（元素"消失"）——加完 grep 验证
2. `MpButton` 无 `set_visible` 实现（trait 默认空操作）→ 用 View 包裹；DSL 也不支持 `visible` 属性
3. `DrawQuad::begin/end` = push/pop turtle（`Layout::default()`=Down+原点对齐）；文本用 `fit()` + 板级 turtle align 居中，fill 文本在该作用域不可靠
4. DataGrid DSL 块内对象键=组件模板（Editor/FkEditor）；未知对象键静默成模板不报错
5. `MpTextArea` 纯展示；真编辑器用 `TextInput{is_multiline:true}`（⌘Enter 发 Returned）
6. makepad 字体缺字形：⟳▾▸⏏⌕▤◫●；可用：⚡⚙◉○⬇⬆⧉◀▶＋－—·
7. Script derive 字段解析 token 级：禁 doc 注释、泛型逗号用 type alias
8. python 批量编辑：assert 失败=改动全丢；小步 + 每步 grep 落盘确认

## 待办（优先级排序）

- [ ] 查询编辑器语法高亮（需自绘 token 着色层，makepad 无现成组件）
- [ ] 视觉验证清单：网格铺满/标签居中选中/模式切换（等用户下轮截图）
- [ ] INSERT 空白行表单（非复制/默认值路径）
- [ ] wasm 后端抽象（rusqlite/mysql/pg 均不能编译 wasm32，需 sql.js 方案，大工程）
- [ ] 连接树右键菜单（重命名/移动分组）
- [ ] 查询结果多结果集（MySQL 多语句）
- [ ] 打包（cargo makepad 各平台产物）
