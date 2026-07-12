# QS-Vibe 前端重构计划

> 版本: v0.1 | 更新: 2026-06-27
> 状态: 执行中

---

## 1. 重构背景

### 1.1 现有问题

当前 UI 采用左右分栏布局（SkillLibrary | AgentPanel），存在以下问题：

| 问题 | 说明 |
|------|------|
| 功能混杂 | CLI 管理放在设置弹窗中，与主界面割裂 |
| 缺少看板 | 无法直观看到各 agent 的 skill 分布和关联关系 |
| 软连接配置原始 | 仅有单个 skill 对单个 agent 的 link/unlink，缺少批量层级操作 |
| 搜索能力弱 | 仅支持 name + description 搜索，不支持 SKILL.md 正文搜索 |
| 界面结构不清晰 | 没有明确的功能分区，用户难以理解各模块用途 |

### 1.2 重构目标

按功能模块划分为 4 个独立子页面，统一放在「Skill 管理」大页签下：

```
QS-Vibe 管理
└── Skill 管理（Tab 页签）
    ├── CLI 管理      — 发现、添加、配置 CLI 工具
    ├── Skill 列表     — 搜索、预览、管理所有 skill
    ├── 看板          — 可视化展示 skill 分布与关联
    └── 软连接配置     — 层级式批量软连接管理
```

---

## 2. 四大模块设计

### 2.1 CLI 管理

**目标**：管理已知的 CLI 工具（自动检测 + 用户自定义），每个 CLI 作为一个标签（tag）出现在 skill 上。

**功能**：
- 自动检测已安装的 CLI（claude-code, hermes, codex 等）
- 用户可添加自定义 CLI（通过文件夹选择器选路径）
- 默认 skills 路径 = `{选中路径}/skills`，支持自定义修改
- 用户配置 CLI 名称 → 该名称作为 tag 显示在每个 skill 卡片上

**数据结构扩展**：
```rust
// AgentConfig 新增字段
struct AgentConfig {
    id: String,
    name: String,           // CLI 名称，作为 tag 显示
    skills_dir: String,     // skills 目录（支持 ~ 展开）
    enabled: bool,
    auto_detected: bool,
    cli_path: Option<String>,  // 新增：CLI 可执行文件路径（可选）
    tag_color: Option<String>, // 新增：标签颜色（可选，前端自动分配）
}
```

**UI 设计**：
```
┌─────────────────────────────────────────────────────┐
│ CLI 管理                                    [+ 添加] │
│                                                       │
│ ┌─── Claude Code ──────────── 已检测 ─────────────┐ │
│ │  ● ~/.claude/skills/          12 skills  [编辑]  │ │
│ │  标签: claude-code                              │ │
│ └─────────────────────────────────────────────────┘ │
│                                                       │
│ ┌─── Hermes ───────────────── 已检测 ─────────────┐ │
│ │  ● ~/.hermes/skills/           8 skills  [编辑]  │ │
│ │  标签: hermes                                   │ │
│ └─────────────────────────────────────────────────┘ │
│                                                       │
│ ┌─── My Custom CLI ────────── 已检测 ─────────────┐ │
│ │  ● /path/to/cli/skills/       3 skills  [编辑]  │ │
│ │  标签: my-custom  [🗑]                           │ │
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

**添加自定义 CLI 流程**：
1. 点击「+ 添加」
2. 弹窗：输入 CLI 名称 → 选择路径（native file picker）→ 自动填充 skills 路径 → 用户可修改 → 保存

**Rust 新增命令**：
- `add_custom_agent(name, cli_path, skills_dir)` — 传入 cli_path，skills_dir 默认为 `{cli_path}/skills`
- `update_agent(agent_id, name?, skills_dir?, cli_path?)` — 更新配置

### 2.2 Skill 列表

**目标**：展示所有 skill，支持按名称和内容搜索。

**功能**：
- 全量 skill 列表（合并 vibe-skills + 所有 agent 目录）
- 搜索支持：name 匹配 + SKILL.md body 全文搜索
- 每个 skill 卡片显示：名称、描述、tag（所属 CLI）、license、特性标记
- 点击预览 SKILL.md 内容
- 支持安装、删除、关联操作

**搜索增强**：
```rust
#[tauri::command]
pub fn search_skills(query: String) -> Result<Vec<Skill>, VibeError> {
    // 1. 扫描所有 skill
    // 2. 匹配条件：name contains query OR SKILL.md body contains query
    // 3. 返回匹配的 skill 列表
}
```

**UI 设计**：
```
┌─────────────────────────────────────────────────────┐
│ Skill 列表                        🔍 搜索 skills...  │
│                                                       │
│ ┌─── my-skill ───────────────────────────────────┐  │
│ │  网页搜索和内容提取                              │  │
│ │  [claude-code] [hermes]    MIT  📄📁🎮         │  │
│ └────────────────────────────────────────────────┘  │
│                                                       │
│ ┌─── another-skill ──────────────────────────────┐  │
│ │  数据分析工具                                    │  │
│ │  [codex]                   Apache-2.0  📄      │  │
│ └────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 2.3 看板

**目标**：可视化展示 skill 在各 agent 之间的分布关系。

**展示形式**：采用「面板+圆点」方案
- 每个 agent 一个面板列
- 面板内用圆点/卡片展示其拥有的 skill
- 跨 agent 共享的 skill 用连线或相同颜色标识
- 顶部统计：总 skill 数、共享 skill 数、各 agent skill 数

**数据结构**：
```typescript
interface DashboardData {
  agents: DashboardAgent[];
  shared_skills: SharedSkillInfo[];
  stats: {
    total_skills: number;
    shared_count: number;
    per_agent_count: Record<string, number>;
  };
}

interface DashboardAgent {
  agent_id: string;
  agent_name: string;
  skill_count: number;
  skills: DashboardSkill[];
}

interface SharedSkillInfo {
  skill_id: string;
  skill_name: string;
  agent_ids: string[];  // 拥有该 skill 的 agent 列表
  count: number;
}
```

**UI 设计**：
```
┌─────────────────────────────────────────────────────────────────┐
│ 看板                                                             │
│                                                                   │
│  总计 23 skills  |  5 共享  |  Claude: 12  Hermes: 8  Codex: 6   │
│                                                                   │
│  ┌── Claude Code ──┐  ┌── Hermes ──────┐  ┌── Codex ──────────┐ │
│  │  ● my-skill ─────┼──┼── ● ──────────┼──┼── ● ───────────── │ │
│  │  ● tool-a        │  │  ● tool-b     │  │  ● tool-c         │ │
│  │  ● tool-d ───────┼──┼───────────────┘  │  ● tool-e         │ │
│  │  ● tool-f        │  │  ● tool-g        │  ● tool-f ─────────┤ │
│  │  ● ...           │  │  ● ...           │  ● ...             │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                   │
│  共享 Skill: my-skill (3), tool-d (2), tool-f (2)                 │
└─────────────────────────────────────────────────────────────────┘
```

**Rust 新增命令**：
```rust
#[tauri::command]
pub fn get_dashboard_data() -> Result<DashboardData, VibeError> {
    // 1. 获取所有 agent 及其 skills
    // 2. 计算共享 skill（出现在多个 agent 中）
    // 3. 返回统计数据
}
```

### 2.4 软连接配置

**目标**：层级式管理 agent → vibe-skills 的软连接，支持批量操作。

**交互方式**：
- 不使用树形控件（树样式不好看）
- 使用「分层卡片/手风琴」布局
- 左侧：agent 列表（可展开）
- 右侧：选中 agent 的 skills 层级视图

**核心操作**：
1. **点击 agent 名称** → 将该 agent 所有 skills 软连接到 `~/.vibe-skills/{agent_id}/`
2. **点击 agent 下的分类文件夹** → 将该分类的 skills 软连接到 `~/.vibe-skills/{agent_id}/{category}/`
3. **点击单个 skill** → 可单独操作

**软连接方向**：
```
agent/skills/{category?}/{skill} → symlink → ~/.vibe-skills/{agent_id}/{category?}/{skill}
```

> 注意：与旧版方向相反。旧版是 agent/skills/skill → vibe-skills/skill（单 skill 级别）
> 新版是 vibe-skills/agent/category/skill → agent/skills/category/skill（层级级别）

**目录结构不变原则**：
- 源目录（agent/skills/）结构不动
- 目标目录（~/.vibe-skills/{agent}/）保持相同层级结构
- 操作只是创建/删除软连接，不移动文件

**UI 设计**：
```
┌─────────────────────────────────────────────────────────────────┐
│ 软连接配置                                                        │
│                                                                   │
│  ┌── Agent 列表 ─────────┐  ┌── 同步预览 ─────────────────────┐ │
│  │                         │  │                                   │ │
│  │  ● Claude Code  [同步]  │  │  目标: ~/.vibe-skills/claude-code/│ │
│  │    ● skills/            │  │                                   │ │
│  │      ● github/    [同步]│  │  源: ~/.claude/skills/            │ │
│  │        ● code-review    │  │                                   │ │
│  │        ● pr-manager     │  │  即将同步:                        │ │
│  │      ● web/       [同步]│  │  ├── github/                     │ │
│  │        ● search         │  │  │   ├── code-review/            │ │
│  │        ● fetch          │  │  │   └── pr-manager/             │ │
│  │                         │  │  └── web/                        │ │
│  │  ● Hermes      [同步]   │  │      ├── search/                 │ │
│  │    ● skills/            │  │      └── fetch/                   │ │
│  │      ● coding/    [同步]│  │                                   │ │
│  │        ● debug          │  │  [执行同步] [取消]                 │ │
│  │      ● research/  [同步]│  │                                   │ │
│  │        ● summarize      │  │  已同步状态:                      │ │
│  │                         │  │  ✅ claude-code/github (2 skills) │ │
│  │  ● Codex        [同步]  │  │  ✅ claude-code/web (2 skills)    │ │
│  │    ...                 │  │  ⏳ hermes/coding (pending)        │ │
│  └─────────────────────────┘  └─────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

**Rust 新增命令**：
```rust
#[derive(Serialize, Deserialize)]
struct SkillsTreeNode {
    name: String,
    path: String,
    is_dir: bool,
    skill_count: usize,      // 子树中 skill 的数量
    synced: bool,             // 是否已同步到 vibe-skills
    synced_count: usize,      // 已同步的 skill 数
    children: Vec<SkillsTreeNode>,
}

#[tauri::command]
pub fn get_skills_tree(agent_id: String) -> Result<SkillsTreeNode, VibeError> {
    // 扫描 agent 的 skills 目录，构建层级结构
    // 检查每个节点是否已同步到 vibe-skills
}

#[tauri::command]
pub fn sync_agent_to_vibe(agent_id: String) -> Result<SyncResult, VibeError> {
    // 将 agent 所有 skills 创建软连接到 ~/.vibe-skills/{agent_id}/
    // 保持原始目录结构
}

#[tauri::command]
pub fn sync_category_to_vibe(agent_id: String, category_path: String) -> Result<SyncResult, VibeError> {
    // 将 agent 的特定分类 skills 创建软连接到 ~/.vibe-skills/{agent_id}/{category}/
}

#[tauri::command]
pub fn remove_sync(agent_id: String, path: Option<String>) -> Result<(), VibeError> {
    // 移除软连接
    // path=None → 移除整个 agent 的同步
    // path=Some("github") → 移除特定分类的同步
}
```

---

## 3. Rust 后端变更

### 3.1 新增依赖

```toml
# Cargo.toml
tauri-plugin-dialog = "2"   # 原生文件夹选择器
```

```json
// package.json
"@tauri-apps/plugin-dialog": "^2"
```

### 3.2 新增/修改命令

| 命令 | 模块 | 说明 |
|------|------|------|
| `search_skills(query)` | skills.rs | 按名称+内容搜索 |
| `update_agent(agent_id, name?, skills_dir?)` | agents.rs | 更新 agent 配置 |
| `get_skills_tree(agent_id)` | agents.rs | 获取层级结构 |
| `get_dashboard_data()` | skills.rs | 获取看板数据 |
| `sync_agent_to_vibe(agent_id)` | sync.rs | 批量同步 |
| `sync_category_to_vibe(agent_id, category)` | sync.rs | 分类同步 |
| `remove_sync(agent_id, path?)` | sync.rs | 移除同步 |

### 3.3 新增模型

```rust
// models/dashboard.rs
pub struct DashboardData {
    pub agents: Vec<DashboardAgent>,
    pub shared_skills: Vec<SharedSkillInfo>,
    pub stats: DashboardStats,
}

pub struct DashboardAgent {
    pub agent_id: String,
    pub agent_name: String,
    pub skill_count: usize,
    pub skills: Vec<DashboardSkill>,
}

pub struct DashboardSkill {
    pub skill_id: String,
    pub skill_name: String,
    pub shared_with: Vec<String>,
}

pub struct SharedSkillInfo {
    pub skill_id: String,
    pub skill_name: String,
    pub agent_ids: Vec<String>,
}

pub struct DashboardStats {
    pub total_skills: usize,
    pub shared_count: usize,
    pub per_agent_count: std::collections::HashMap<String, usize>,
}

// models/sync.rs
pub struct SkillsTreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub skill_count: usize,
    pub synced: bool,
    pub synced_count: usize,
    pub children: Vec<SkillsTreeNode>,
}

pub struct SyncResult {
    pub synced_count: usize,
    pub errors: Vec<String>,
}
```

### 3.4 lib.rs 注册变更

```rust
invoke_handler(tauri::generate_handler![
    // ...existing commands...
    commands::skills::search_skills,
    commands::skills::get_dashboard_data,
    commands::agents::update_agent,
    commands::agents::get_skills_tree,
    commands::sync::sync_agent_to_vibe,
    commands::sync::sync_category_to_vibe,
    commands::sync::remove_sync,
])
```

---

## 4. 前端重构

### 4.1 目录结构变更

```
src/
├── App.vue                          # 重写：Tab 布局
├── main.ts
├── style.css
├── i18n.ts
├── types/
│   └── index.ts                     # 新增类型
├── stores/
│   ├── app.ts                       # 新增 activeTab 状态
│   ├── skills.ts                    # 新增 searchSkills, getDashboardData
│   ├── agents.ts                    # 新增 updateAgent, getSkillsTree, sync 操作
│   └── history.ts
├── components/
│   ├── layout/
│   │   ├── AppLayout.vue            # 重写：Header + TabBar + Content
│   │   └── TabBar.vue               # 新增：Tab 导航
│   ├── cli/                         # 新增目录
│   │   ├── CLITab.vue               # CLI 管理页面
│   │   ├── CLICard.vue              # CLI 卡片
│   │   └── AddCLIDialog.vue         # 添加自定义 CLI 弹窗
│   ├── skills/
│   │   ├── SkillList.vue            # 重命名+重写：Skill 列表页面
│   │   ├── SkillCard.vue            # 重写：显示 tag
│   │   ├── SkillPreview.vue         # 保留
│   │   └── InstallDialog.vue        # 保留
│   ├── dashboard/                   # 新增目录
│   │   ├── DashboardTab.vue         # 看板页面
│   │   ├── AgentColumn.vue          # Agent 列
│   │   └── SharedSkillBar.vue       # 共享 skill 展示
│   ├── symlink/                     # 新增目录
│   │   ├── SymlinkTab.vue           # 软连接配置页面
│   │   ├── AgentExpandable.vue      # Agent 展开卡片
│   │   └── SyncPreview.vue          # 同步预览面板
│   ├── history/
│   │   └── HistoryBar.vue           # 保留
│   ├── common/
│   │   ├── ConfirmDialog.vue        # 保留
│   │   ├── ErrorBanner.vue          # 保留
│   │   └── TagBadge.vue             # 新增：通用标签组件
│   └── settings/
│       └── SettingsPage.vue         # 精简：移除 agent 管理（移到 CLI tab）
└── locales/
    ├── zh.json                      # 扩展
    ├── en.json                      # 扩展
    └── zh-TW.json                   # 扩展
```

### 4.2 状态管理

```typescript
// stores/app.ts 新增
interface AppState {
  theme: ThemeMode;
  locale: Locale;
  activeTab: 'cli' | 'skills' | 'dashboard' | 'symlink';
  showSettings: boolean;
}

// stores/skills.ts 新增
interface SkillsState {
  skills: Skill[];
  searchResults: Skill[];
  searchQuery: string;
  dashboardData: DashboardData | null;
  // ...existing
  searchSkills(query: string): Promise<void>;
  getDashboardData(): Promise<void>;
}

// stores/agents.ts 新增
interface AgentsState {
  agents: Agent[];
  skillsTree: SkillsTreeNode | null;
  syncResult: SyncResult | null;
  // ...existing
  updateAgent(agentId: string, updates: Partial<Agent>): Promise<void>;
  getSkillsTree(agentId: string): Promise<void>;
  syncAgentToVibe(agentId: string): Promise<void>;
  syncCategoryToVibe(agentId: string, category: string): Promise<void>;
  removeSync(agentId: string, path?: string): Promise<void>;
}
```

### 4.3 TypeScript 类型扩展

```typescript
// types/index.ts 新增
interface DashboardData {
  agents: DashboardAgent[];
  shared_skills: SharedSkillInfo[];
  stats: DashboardStats;
}

interface DashboardAgent {
  agent_id: string;
  agent_name: string;
  skill_count: number;
  skills: DashboardSkill[];
}

interface DashboardSkill {
  skill_id: string;
  skill_name: string;
  shared_with: string[];
}

interface SharedSkillInfo {
  skill_id: string;
  skill_name: string;
  agent_ids: string[];
}

interface DashboardStats {
  total_skills: number;
  shared_count: number;
  per_agent_count: Record<string, number>;
}

interface SkillsTreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  skill_count: number;
  synced: boolean;
  synced_count: number;
  children: SkillsTreeNode[];
}

interface SyncResult {
  synced_count: number;
  errors: string[];
}
```

### 4.4 App.vue 重写

```vue
<template>
  <AppLayout>
    <template #header>
      <!-- 标题 + 设置 + 主题切换 -->
    </template>
    <template #tabs>
      <TabBar v-model="appStore.activeTab" />
    </template>
    <template #content>
      <CLITab v-if="appStore.activeTab === 'cli'" />
      <SkillList v-else-if="appStore.activeTab === 'skills'" />
      <DashboardTab v-else-if="appStore.activeTab === 'dashboard'" />
      <SymlinkTab v-else-if="appStore.activeTab === 'symlink'" />
    </template>
    <template #footer>
      <HistoryBar />
    </template>
  </AppLayout>
  <SettingsPage v-if="appStore.showSettings" />
</template>
```

---

## 5. i18n 扩展

新增 keys（三个语言文件同步）：

```json
{
  "app": {
    "title": "QS-Vibe 管理"
  },
  "tabs": {
    "cli": "CLI 管理",
    "skills": "Skill 列表",
    "dashboard": "看板",
    "symlink": "软连接配置"
  },
  "cli": {
    "title": "CLI 管理",
    "add": "添加 CLI",
    "edit": "编辑",
    "detected": "已检测",
    "not_detected": "未检测到",
    "name": "CLI 名称",
    "name_hint": "作为 tag 显示在 skill 卡片上",
    "path": "CLI 路径",
    "skills_dir": "Skills 目录",
    "skills_dir_hint": "默认为 CLI 路径下的 skills 目录",
    "skill_count": "{count} skills",
    "pick_folder": "选择文件夹"
  },
  "dashboard": {
    "title": "看板",
    "total_skills": "总计 {count} skills",
    "shared_count": "{count} 共享",
    "per_agent": "{agent}: {count}",
    "shared_skills": "共享 Skill",
    "no_data": "暂无数据"
  },
  "symlink": {
    "title": "软连接配置",
    "sync_all": "全部同步",
    "sync_category": "同步分类",
    "sync_single": "同步",
    "remove_sync": "移除同步",
    "synced": "已同步",
    "pending": "待同步",
    "sync_preview": "同步预览",
    "target_dir": "目标目录",
    "source_dir": "源目录",
    "execute_sync": "执行同步",
    "cancel": "取消"
  }
}
```

---

## 6. 实施步骤

### Phase 1: Rust 后端 (T1)
1. ✅ 添加 tauri-plugin-dialog 依赖
2. ✅ 更新 lib.rs 注册
3. [ ] 添加 models/dashboard.rs, models/sync.rs
4. [ ] 实现 search_skills 命令
5. [ ] 实现 update_agent 命令
6. [ ] 实现 get_skills_tree 命令
7. [ ] 实现 get_dashboard_data 命令
8. [ ] 实现 sync_agent_to_vibe, sync_category_to_vibe, remove_sync 命令
9. [ ] cargo check 验证

### Phase 2: TypeScript 类型 + Stores (T2)
1. [ ] 更新 types/index.ts
2. [ ] 更新 stores/app.ts (activeTab)
3. [ ] 更新 stores/skills.ts (search, dashboard)
4. [ ] 更新 stores/agents.ts (tree, sync)

### Phase 3: Vue 前端 (T3)
1. [ ] 重写 App.vue + AppLayout.vue
2. [ ] 新增 TabBar.vue
3. [ ] 新增 CLI 模块 (CLITab, CLICard, AddCLIDialog)
4. [ ] 重写 SkillList + SkillCard
5. [ ] 新增 Dashboard 模块
6. [ ] 新增 Symlink 模块
7. [ ] 新增 TagBadge 通用组件

### Phase 4: i18n (T4)
1. [ ] 更新 zh.json
2. [ ] 更新 en.json
3. [ ] 更新 zh-TW.json

### Phase 5: README (T5)
1. [ ] 编写 README.md

### Phase 6: 验证
1. [ ] vue-tsc --noEmit 类型检查
2. [ ] pnpm tauri dev 运行验证

---

## 7. 风险与注意事项

| 风险 | 应对 |
|------|------|
| 软连接方向变更 | 需要兼容旧数据，检测并提示用户迁移 |
| 大量 skill 时性能 | 搜索使用 Rust 端过滤，避免前端全量加载 |
| Windows 软连接权限 | 继续使用 junction 作为降级方案 |
| 看板数据量 | 限制显示数量，支持滚动加载 |
| 原有功能兼容 | 保留 undo/redo, install, delete 等功能 |
