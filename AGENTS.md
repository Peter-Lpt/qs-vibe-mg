# AGENTS.md

## What this is

QS-Vibe (qs-vibe-mg) — Tauri 2 桌面应用（Rust 后端 + Vue 3 / TypeScript 前端），用于通过统一技能库 `~/.vibe-skills/` 集中管理多个 AI 编码 agent 的 skills（Claude Code、Hermes、Pi Agent、OpenCode、Codex CLI、MiMo Code、共享 agents 等）。Skills 通过符号链接（symlink）挂接到各 agent 目录，并支持 git / 文件夹 / 命令 / 插件市场等多种安装来源与来源溯源（provenance）。遵循 [Agent Skills](https://agentskills.io) 开放标准。

当前版本：v0.3.0（package.json）。v0.1 功能（安装/删除、预览、批量操作、撤销/重做、i18n、主题、设置、自定义 agent）已完成。

## Commands

```bash
pnpm install          # 安装依赖
pnpm dev              # 启动 Vite 开发服务器（仅前端）
pnpm build            # vue-tsc 类型检查 + Vite 生产构建（仅前端）
pnpm tauri dev        # 带 Tauri 的运行（首次 Rust 编译 3-5 分钟）
pnpm tauri build      # 生产构建（打包）
pnpm preview          # Vite 预览已构建前端
cargo test            # 运行 Rust 测试（在 src-tauri/ 下）
cargo check          # 仅 Rust 类型检查（在 src-tauri/ 下）
```

**Windows 要求**：执行任何 cargo/tauri 命令前设置 Rust 环境变量：
```
$env:RUSTUP_HOME = "D:\environment\rust\.rustup"
$env:CARGO_HOME = "D:\environment\rust\.cargo"
```
**Windows 符号链接要求**：需开启开发者模式，或以管理员身份运行终端。

## Architecture

整体为「前端通过 Tauri IPC 调用 Rust 后端」的单进程架构。前端不直接访问文件系统；所有磁盘操作都经 `invoke()` 转发到 Rust 命令。后端无数据库，状态保存在用户目录的 JSON 文件（`~/.vibe-skills/.vibe-config.json`、`.vibe-history.json`，每个 skill 的 `.vibe-origin.json` / `.vibe-origin/` 溯源文件）。

---

### 前端（Vue 3 + TypeScript + Vite + Pinia + Tailwind 4 + vue-i18n）

**技术栈**：Vue 3 `<script setup>`、TypeScript、Vite 8、Pinia 3、Tailwind CSS 4（`@tailwindcss/vite`，仅用工具类 + CSS 变量主题，无自定义 CSS 文件）、vue-i18n 9、lucide 图标、marked（SKILL.md 渲染）。

**入口与外壳**
- `src/main.ts`：挂载 `App.vue`，注册 Pinia、i18n、图标。
- `src/App.vue`：应用外壳。两个主 Tab：`manage`（技能工作区）与 `history`（撤销/重做）；`SettingsPage` 以浮层形式弹出。挂载时依次 `appStore.init()` → `fetchAgents()` → `fetchSkills()` → `fetchHistory()`。全局快捷键：Ctrl+1/2 切 Tab；在 history Tab 内 Ctrl+Z 撤销、Ctrl+Shift+Z 重做。

**状态层（Pinia stores，`src/stores/`）—— 所有后端调用都经 store，组件不直接 invoke**
- `skills.ts`：技能列表/搜索/仪表盘/问题检测；链接管理 `createLink`/`removeLink`/`detachKeepLocalCopy`/`removeAgentSkillCopy`；安装 `installSkill`/`installSkillFromSource`（folder/git/command 三种来源）/更新 `updateSkill`/更新检查 `checkSkillUpdate`/`checkAllSkillUpdates`/删除 `deleteSkill`/预览 `previewSkill`/`previewSkillAtPath`；同步 `syncToVibe`/`relink`/`replaceWithLibrary`/`batchSkillAction`。变更后自动 `refreshSkills()` + `fetchAgents()`。
- `agents.ts`：`listAgents`/`addCustomAgent`/`updateAgent`/`removeCustomAgent`/`getSkillsTree`。
- `history.ts`：`getHistory`/`undo`/`redo`（含 by_id 版本）/`clearHistory`；`updateUndoRedoState()`。
- `app.ts`：当前 Tab、locale、主题、`showSettings`、`init()`。

**组件层（`src/components/`）**
- `layout/`：`AppLayout`（外壳 + 主题 CSS 变量）、`AppSidebar`（左侧导航）。
- `manage/`（核心工作区）：
  - `SkillsView.vue`：主组合视图——顶部统计卡、筛选中心（关键词防抖、排序、按问题/状态/来源/Agent 分面，逻辑在 `manageFilters.ts`）、`IssueRepairPanel`（冲突/断链/重复修复入口）、技能列表 `SkillRow`（可展开为 `SkillDetail`）、底部批量操作条、`InstallDialog`、回到顶部。
  - `PluginSkillsView.vue`：Plugin 技能独立视图（认领/更新/启用状态）。
- `history/`：`HistoryTab.vue`（撤销/重做时间线）。
- `settings/`：`SettingsPage.vue`（含内联的自定义 Agent 增删改）。
- `common/`：`ConfirmDialog`、`EmptyState`、`SkeletonCard`、`ToastContainer`。

**组合式函数（`src/composables/`）**：`useToast`（轻提示）、`useSkillActions`、`useSkillAgentStatus`、`skillActionRegistry`（动作注册表）、`useBatchActions`（意图驱动批量）、`useSmartViews`（侧边栏视图注册表）、`useEscapeKey`、`useFileLogger`（前端日志 → 后端 `log_message`）。

**i18n / 类型**：`src/i18n.ts` + `src/locales/{zh,en,zh-TW}.json`（所有 UI 文案入 i18n）；`src/types/index.ts` 为共享 TS 接口。

---

### 后端（Rust + Tauri 2，`src-tauri/src/`）

**技术栈**：Tauri 2、serde / serde_yaml（SKILL.md 解析）、sha2（目录哈希）、uuid（历史 ID）、thiserror 风格 `VibeError`、tracing + tracing-appender（日志写入 `%LOCALAPPDATA%/qs-vibe-mg/logs/app.log`，不进仓库）、dirs。

**命令注册（`lib.rs`）**：约 50 个 `#[tauri::command]`，分 6 个模块（`invoke_handler!` 中注册）。`init_logger()` 用按天滚动的文件日志。

**命令模块（`commands/`）**
- `skills.rs`：
  - `list_skills` —— 递归扫描 `~/.vibe-skills/`（vibe-lib）、所有 agent 技能目录、项目根（`project:` 来源）、插件市场（claude-plugin / codex-plugin 缓存），按文件夹名去重合并为 `Skill`；检测冲突（多来源内容哈希不同）、断链（symlink 目标不存在）、重复（同名不同 SKILL.md 或 plugin+本地并存）；使用哈希缓存避免重复读取。
  - `preview_skill` / `preview_skill_at_path`（路径沙箱：仅允许读取 vibe/agent/项目根内文件）。
  - `check_skill_update` / `check_all_skill_updates` —— 对 git 来源做远程 commit 比对（带 30s fetch 超时）。
  - `install_skill` / `install_skill_from_source` —— 支持 folder / git / command 三种来源；`reference=true` 以 symlink 引用源码，否则复制；写入 `SkillOrigin` 溯源。
  - `update_skill` —— git 来源 `git pull --ff-only`（冲突时要求 force），命令来源执行 `update_command`。
  - `delete_library_skill` —— 先快照到 `.trash/` 再删除，并清理各 agent 上的链接。
  - `detect_issues`；`load_update_checks`（读取上次落盘的更新检测结果）。
- `sync.rs`（符号链接管理，核心）：
  - 链接方向 **vibe→agent**：`create_link` / `remove_link` 在 `agent_dir/{skill}` 创建/移除指向 `~/.vibe-skills/{skill}` 的链接；`detach_keep_local_copy` 断开链接但保留本地副本；`remove_agent_skill_copy` 删除 agent 本地副本（进 trash）。
  - 镜像方向 **agent→vibe**：`sync_agent_to_vibe` / `sync_category_to_vibe` / `remove_sync` / `remove_sync_skills` 在 `~/.vibe-skills/{agent_id}/` 下为 agent 技能建镜像 symlink（镜像同步类操作不进 undo 链）。
  - 单技能整理：`sync_to_vibe`（把 agent 副本并入库并改指库的 symlink）、`relink`、`replace_with_library`；`batch_link` / `batch_unlink` / `batch_skill_action`（同一 skill 对多 agent 批量操作，每个 agent 记独立历史）。
- `agents.rs`：`list_agents` / `add_custom_agent`(+`_with_options`) / `update_agent` / `remove_custom_agent`。
- `history.rs`：`get_history` / `undo` / `redo` / `undo_by_id` / `redo_by_id` / `clear_history`。
- `config.rs`：`get_config` / `suggest_project_roots` / `update_config` / `set_vibe_skills_path`。
- `logger.rs`：`log_message`（前端日志转发到 tracing）。

**数据模型（`models/`）**：`skill.rs`（`Skill`、`SkillSource`、`ConflictType`、`SkillIssue`）、`agent.rs`（`Agent`）、`history.rs`（`HistoryAction`）、`origin.rs`（`SkillOrigin` 溯源记录）、`sync.rs`（`SyncResult`）。

**解析（`parsers/`）**：`skill_md.rs` 解析 SKILL.md 的 YAML frontmatter（`name`、`description`，可选 `license`/`compatibility`/`metadata`）。

**工具（`utils/`）**
- `path.rs`：`~` 展开、`vibe_skills_dir()`。
- `config.rs`：读写 `.vibe-config.json`；从配置构建 agents；`scan_linked_skills`、`project_skill_roots`、`suggest_project_roots`（带缓存与 `invalidate_agents_cache`）。
- `fs.rs`：跨平台 symlink（`create_symlink_with_report`、`remove_symlink`、`is_link`、`read_link_target`、`copy_dir_all`/`copy_skill_dir_all`、`clear_skill_dir_contents`、`detach_link_keep_copy`、`is_path_within` 路径守卫）。
- `hash.rs`：目录 SHA-256 哈希 + 哈希缓存（`.hash-cache`，性能优化，避免重复读文件）。
- `origin.rs`：来源溯源——`SkillOrigin` 的 probe/read/write，`GitProbe`（remote_url/commit/branch），`trust_level`/`update_status` 推断，`build_*_origin`，git fetch/pull 命令，`normalize_source_method`（git / local-folder / npm / npx / marketplace）。
- `history.rs`：`record_action`（含 with_skills / with_source 变体）、undo/redo 实现。
- `datetime.rs`：ISO 8601 / chrono 格式化。

**错误**：`errors.rs` 的 `VibeError` 枚举，自定义 `Serialize` 实现序列化为字符串传给前端。

---

### 跨切面：数据流与关键概念

1. **技能扫描与合并**：后端把 vibe 库、各 agent 目录、项目根、插件市场统一扫描，按文件夹名去重为单个 `Skill`，其 `sources: Vec<SkillSource>` 记录每个出现位置（from / 是否 symlink / 内容哈希 / 溯源）。前端据此做去重、冲突检测与 Agent 矩阵。
2. **两种（实为多种）链接方向**：
   - **Link（库→agent）**：`create_link` 在 agent 目录建立指向库的 symlink。
   - **Sync（agent→库）**：`sync_agent_to_vibe` 在 `~/.vibe-skills/{agent_id}/` 镜像 agent 技能为 symlink。
   - **整理类**：`sync_to_vibe`（并入库 + 重定向为库 symlink，实现单一事实来源）、`relink`、`replace_with_library`、`detach_keep_local_copy`。
3. **来源溯源（provenance）**：每个 skill 安装/更新时写入 `SkillOrigin`（`.vibe-origin.json` 或 sidecar `.vibe-origin/{name}.json`），记录 method、url、commit、branch、source_path、update_command、trust_level、sync_mode。由此支持更新检测（git 远程 diff）与自动更新。来源方法：`git` / `local-folder` / `npm` / `npx` / `marketplace`（插件）。
4. **异常检测**：冲突（多非插件来源内容哈希不同）、断链（symlink 目标缺失）、重复（同名不同 name 或 plugin+本地并存）。`detect_issues` 暴露给前端 `IssueRepairPanel` 修复。
5. **撤销/重做**：每个变更命令记录 `HistoryAction` 到 `.vibe-history.json`；删除先快照 `.trash/`，撤销/重做按 id 通过 trash 快照 + symlink 操作精确回放。
6. **无数据库**：配置 `.vibe-config.json`、历史 `.vibe-history.json`、各 skill 溯源文件、trash `.trash/` 均在用户目录。

### 支持的 agent 与来源

默认路径：`~/.claude/skills/`、`~/.hermes/skills/`（Windows：`%LOCALAPPDATA%\hermes\skills`）、`~/.pi/agent/skills/`、`~/.config/opencode/skills/`、`~/.codex/skills/`、`~/.config/mimocode/skills/`、`~/.agents/skills/`。此外还扫描**项目根**下的 `.claude/skills`/`.agents/skills`/`.codex/skills`/`.github/skills`/`skills`，以及 **Claude / Codex 插件市场缓存**（`~/.claude/plugins/cache`、`~/.codex/plugins/cache`）。

## Conventions

- **语言**：UI 文案必须 i18n 兼容，新增 key 须同步写入 `zh.json`/`en.json`/`zh-TW.json` 三份。
- **Rust 错误**：使用 `VibeError` 枚举，新增变体后在 `errors.rs` 增补，自动序列化为字符串传给前端。
- **前端状态**：所有后端调用经 Pinia store，组件不直接 `invoke()`。
- **SKILL.md 格式**：`---` 分隔的 YAML frontmatter，含 `name`、`description`，可选 `license`/`compatibility`/`metadata`。
- **包管理器**：仅用 pnpm（不用 npm/yarn）。
- **CSS**：Tailwind CSS 4 via `@tailwindcss/vite`，用工具类 + CSS 变量（`--c-text`、`--c-primary` 等）做主题，无独立 CSS 文件（除 `style.css`）。
- **Rust 测试**：内联 `#[cfg(test)] mod tests`（无独立测试目录）。现有测试覆盖：`utils/path.rs`、`utils/fs.rs`、`utils/datetime.rs`、`parsers/skill_md.rs`、`commands/skills.rs`。
- **Windows symlink**：开发者模式或管理员；Rust 命令前设 `RUSTUP_HOME`/`CARGO_HOME`。

## Version Roadmap

- **v0.1**（Full）：安装/删除、预览、批量操作、撤销/重做、i18n、主题、设置、自定义 agent —— 已完成。
- **v0.2**（Full）：来源溯源（git/命令/插件市场）、更新检测与自动更新、异常检测与一键修复、Plugin Skills 分组 —— 已完成。
- **v0.3**（Current）：package.json 0.3.0。

## Documentation Index

- `docs/README.md` —— 文档枢纽
- `docs/01-dev-environment.md` —— 环境搭建
- `docs/02-modules.md` —— 模块路线图
- `docs/03-requirements.md` —— v0.1 需求清单（R01-R21）
- `docs/v0.1/01-plan.md`、`docs/v0.0/01-plan.md` —— 各版本计划
