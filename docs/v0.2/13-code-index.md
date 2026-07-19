# QS-Vibe 代码索引

> 作用：给人和 agent 快速定位“功能在哪个文件、关键逻辑在哪个函数”。

## 1. 启动入口

- `src-tauri/src/main.rs`：Tauri 桌面程序入口。
- `src-tauri/src/lib.rs`：注册全部 IPC 命令。
- `src/main.ts`：前端入口。
- `src/App.vue`：应用壳层。
- `src/components/layout/AppLayout.vue`：主布局。

## 2. 后端核心

### 技能主链路

- `src-tauri/src/commands/skills.rs`
  - `list_skills`
  - `search_skills`
  - `install_skill`
  - `install_skill_from_source`
  - `update_skill`
  - `delete_library_skill`
  - `detect_issues`
  - `get_dashboard_data`

### 同步 / 链接

- `src-tauri/src/commands/sync.rs`
  - `create_link`
  - `remove_link`
  - `sync_to_vibe`
  - `relink`
  - `replace_with_library`
  - `batch_skill_action`

### 配置 / 历史 / 日志

- `src-tauri/src/commands/config.rs`
- `src-tauri/src/commands/history.rs`
- `src-tauri/src/commands/logger.rs`

## 3. 后端基础能力

- `src-tauri/src/models/skill.rs`：Skill / SkillSource / Issue 模型。
- `src-tauri/src/models/origin.rs`：来源 provenance 模型。
- `src-tauri/src/utils/origin.rs`：来源识别、Git/命令更新、provenance 读写。
- `src-tauri/src/utils/fs.rs`：复制、symlink、路径处理。
- `src-tauri/src/utils/hash.rs`：目录 hash 缓存。
- `src-tauri/src/utils/path.rs`：vibe-skills 根目录。
- `src-tauri/src/utils/config.rs`：配置读写、project roots。

## 4. 前端核心

### 布局 / 页面

- `src/components/layout/TabBar.vue`
- `src/components/manage/ManageTab.vue`
- `src/components/settings/SettingsPage.vue`
- `src/components/history/HistoryTab.vue`
- `src/components/manage/IssueRepairPanel.vue`（同步控制台内的紧凑问题修复入口）

### v0.2 管理页体验优化

- `docs/v0.2/15-manage-ui-optimization.md`
- `docs/v0.2/16-manage-ui-audit-and-optimization.md`
- `docs/v0.2/17-skill-workbench-redesign.md`
- `src/components/manage/ManageTab.vue`（搜索、Agent 范围、低频筛选与问题修复的合并布局）
- `src/components/manage/SkillWorkbench.vue`（自适应 Agent 关系工作台）
- `src/components/manage/SkillWorkbenchRow.vue`（Skill 行、状态关系和行内详情）

### 技能管理

- `src/components/manage/SkillCard.vue`
- `src/components/manage/SkillRow.vue`
- `src/components/manage/SkillDetail.vue`
- `src/components/manage/SkillTree.vue`
- `src/components/skills/InstallDialog.vue`

### 共享状态

- `src/stores/skills.ts`
- `src/stores/agents.ts`
- `src/stores/app.ts`
- `src/stores/history.ts`

## 5. 读代码顺序

1. `docs/03-requirements.md`
2. `docs/v0.2/12-multi-source-skill-install-update-technical-plan.md`
3. `src-tauri/src/utils/origin.rs`
4. `src-tauri/src/commands/skills.rs`
5. `src/components/skills/InstallDialog.vue`
6. `src/components/manage/SkillDetail.vue`
7. `src/stores/skills.ts`

## 6. 跨平台注意点

- Windows symlink 可能降级为 junction。
- 命令执行使用 `cmd /C` 或 `sh -lc`。
- Rust 环境在 Windows 下依赖 `RUSTUP_HOME` / `CARGO_HOME`。
- 所有路径逻辑尽量走 `utils/fs.rs` 和 `utils/path.rs`。
