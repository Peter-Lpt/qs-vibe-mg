# VAB Skills Manager v0.0 - 先行版实施计划

> 版本: v0.0 | 更新: 2026-06-27
> 最小可用版本：Skill 展示 + Symlink 管理，架构骨架完整，后续版本在此基础上扩展。

---

## 1. 目标

**做什么：**
- Skill 列表展示（从 ~/.vab-skills/ 扫描，解析 SKILL.md）
- Agent 列表展示（自动检测已安装的 agent）
- 创建/删除 symlink（skill → agent 目录）
- 关联状态展示（哪些 skill 关联了哪些 agent）

**不做什么：**
- ~~拖拽关联~~（v0.1）
- ~~复制模式~~（v0.1）
- ~~Junction 降级~~（v0.1）
- ~~撤销/重做~~（v0.1）
- ~~批量操作~~（v0.1）
- ~~安装/删除 Skill~~（v0.1）
- ~~设置页~~（v0.1）
- ~~i18n~~（v0.1）
- ~~主题切换~~（v0.1）

---

## 2. 技术栈（与 v0.1 一致）

| 层 | 技术 | 用途 |
|----|------|------|
| 桌面框架 | Tauri 2 | 跨平台窗口 |
| 后端 | Rust | 文件操作、symlink |
| 前端 | Vue 3 + TypeScript + Vite | UI |
| 样式 | Tailwind CSS 4 | 样式 |
| 状态管理 | Pinia | 前端状态 |

> v0.0 不引入 vue-i18n、vue-draggable-plus、marked 等，后续版本按需加入。

---

## 3. 项目结构（骨架）

```
qs-vab-mg/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── skills.rs         # list_skills（扫描所有目录，合并去重）
│   │   │   ├── sync.rs           # create_link, remove_link
│   │   │   └── agents.rs         # list_agents
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── skill.rs          # Skill, SkillSource
│   │   │   └── agent.rs
│   │   ├── parsers/
│   │   │   ├── mod.rs
│   │   │   └── skill_md.rs       # SKILL.md frontmatter 解析
│   │   ├── errors.rs             # 统一错误类型
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── fs.rs             # symlink/junction 操作
│   │       ├── config.rs         # .vab-config.json 读写
│   │       └── path.rs           # ~ 展开
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── App.vue
│   ├── main.ts
│   ├── style.css
│   ├── components/
│   │   ├── layout/
│   │   │   └── AppLayout.vue
│   │   ├── skills/
│   │   │   ├── SkillLibrary.vue
│   │   │   └── SkillCard.vue
│   │   └── agents/
│   │       ├── AgentPanel.vue
│   │       └── AgentCard.vue
│   ├── stores/
│   │   ├── skills.ts
│   │   └── agents.ts
│   └── types/
│       └── index.ts
├── package.json
├── vite.config.ts
├── tsconfig.json
└── index.html
```

**骨架设计原则：**
- commands/ 按模块拆分，v0.1 可直接在现有文件中追加命令
- models/ 独立定义，v0.1 追加字段不影响已有结构
- errors.rs 统一错误类型，v0.1 追加变体即可
- 前端 components/ 按功能目录拆分，v0.1 新增组件放到对应目录
- stores/ 预留扩展点

---

## 4. 数据模型

### Skill
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,                // 文件夹名
    pub name: String,              // SKILL.md frontmatter name
    pub description: String,       // SKILL.md frontmatter description
    pub path: String,              // 绝对路径
    pub linked_agents: Vec<String>, // 已关联的 agent id 列表
    pub sources: Vec<SkillSource>, // 来源列表（vab-lib + 各 agent 目录）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSource {
    pub from: String,  // vab-lib 或 agent id
    pub path: String,  // 该来源下的绝对路径
}
```

### Agent
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub skills_dir: String,
    pub detected: bool,
    pub enabled: bool,
}
```

> v0.0 不需要 HistoryEntry、AgentLink 详细结构，v0.1 追加。

---

## 5. 功能清单

### F01 Skill 扫描与展示
- [x] 扫描 ~/.vab-skills/ 下所有子文件夹
- [x] 扫描所有已检测 agent 的 skills 目录
- [x] 每个子文件夹读取 SKILL.md，解析 frontmatter（name、description）
- [x] 解析失败时用文件夹名作为 name，description 为空
- [x] 合并去重，每个 skill 标注来源（vab-lib / agent id）
- [x] 返回 Skill 列表给前端

### F02 Agent 检测与展示
- [x] 硬编码 7 个默认 agent 及其检测目录
- [x] 检测目录存在 → detected = true
- [x] 返回 Agent 列表给前端

### F03 Symlink 创建
- [x] 前端选择 skill + agent → 调用后端创建 symlink
- [x] 后端：~/.vab-skills/{skill} → {agent.skills_dir}/{skill}
- [x] agent skills 目录不存在时自动创建
- [x] 创建前检测是否已存在

### F04 Symlink 删除
- [x] 前端点击已关联的 agent 标签 → 调用后端删除 symlink
- [x] 后端：删除 {agent.skills_dir}/{skill}（仅删链接，不删源文件）

### F05 关联状态扫描
- [x] 启动时扫描每个 agent 的 skills 目录
- [x] 检测哪些是 symlink 且指向 ~/.vab-skills/
- [x] 更新 Skill.linked_agents 和 Agent 关联信息

### F06 Dashboard 界面
```
┌──────────────────────────────────────────────────────┐
│  VAB Skills Manager                          v0.0    │
├────────────────────────┬─────────────────────────────┤
│ Skills (3)             │ Agents (2/7)                 │
│                        │                              │
│ ┌─ my-skill ────────┐ │  ● claude-code  Detected     │
│ │ description        │ │    ~/.claude/skills           │
│ │ Library  hermes    │ │    my-skill [×]               │
│ │ [+ Link ▾]        │ │                              │
│ └────────────────────┘ │  ● hermes  Detected          │
│                        │    ~/.hermes/skills           │
│ ┌─ web-srch ────────┐ │    my-skill [×]               │
│ │ description        │ │                              │
│ │ claude-code        │ │  ○ pi-agent  Not installed   │
│ │ [+ Link ▾]        │ │    ~/.pi/agent/skills         │
│ └────────────────────┘ │                              │
└────────────────────────┴─────────────────────────────┘
```

交互：
- Skill 卡片点击 [+ Link] → 下拉选择 agent → 创建 symlink
- Agent 卡片上 [×] → 删除 symlink
- 操作后自动刷新两侧状态
- 每个 skill 显示来源标签（Library / agent 名称）

---

## 6. Tauri Commands

```rust
// v0.0 实际暴露的命令
#[tauri::command]
fn list_skills() -> Result<Vec<Skill>, VabError>  // 扫描所有目录，合并去重

#[tauri::command]
fn list_agents() -> Result<Vec<Agent>, VabError>

#[tauri::command]
fn create_link(skill_id: String, agent_id: String) -> Result<(), VabError>

#[tauri::command]
fn remove_link(skill_id: String, agent_id: String) -> Result<(), VabError>
```

> v0.1 在此基础上追加：install_skill、delete_skill、preview_skill、batch_link、undo、redo 等。

---

## 7. 错误处理（骨架）

```rust
#[derive(Debug, Error)]
pub enum VabError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Skill not found: {skill_id}")]
    SkillNotFound { skill_id: String },

    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: String },

    #[error("Invalid SKILL.md: {reason}")]
    InvalidSkillMd { reason: String },

    #[error("Link already exists: {skill_id} -> {agent_id}")]
    LinkAlreadyExists { skill_id: String, agent_id: String },

    #[error("Config error: {0}")]
    Config(String),

    #[error("Path error: {0}")]
    Path(String),
}
```

> v0.1 追加 PermissionDenied、SymlinkFailed 等变体。

---

## 8. 配置文件

### ~/.vab-skills/.vab-config.json
```json
{
  "version": 1,
  "agents": [
    { "id": "claude-code", "name": "Claude Code", "skills_dir": "~/.claude/skills", "enabled": true },
    { "id": "hermes", "name": "Hermes", "skills_dir": "~/.hermes/skills", "enabled": true },
    { "id": "pi-agent", "name": "Pi Agent", "skills_dir": "~/.pi/agent/skills", "enabled": true },
    { "id": "opencode", "name": "OpenCode", "skills_dir": "~/.config/opencode/skills", "enabled": true },
    { "id": "codex", "name": "Codex CLI", "skills_dir": "~/.codex/skills", "enabled": true },
    { "id": "mimocode", "name": "MiMo Code", "skills_dir": "~/.config/mimocode/skills", "enabled": true },
    { "id": "agents-shared", "name": "Shared", "skills_dir": "~/.agents/skills", "enabled": true }
  ]
}
```

> v0.1 追加 custom_agents、ui、history 等配置项。

---

## 9. 开发步骤

```
Step 1: 项目初始化 ✅
  - 创建 Tauri 2 + Vue 3 + TS 项目
  - 配置 Tailwind CSS 4
  - pnpm tauri dev 跑通

Step 2: Rust 后端骨架 ✅
  - 定义 models（Skill、SkillSource、Agent）
  - 定义 errors（VabError）
  - 实现 parsers/skill_md.rs（frontmatter 解析）
  - 实现 utils/（path.rs、fs.rs、config.rs）
  - 实现 commands/（skills、agents、sync）
  - 注册 Tauri commands

Step 3: Vue 前端骨架 ✅
  - AppLayout 整体布局（左右分栏）
  - SkillLibrary + SkillCard 组件
  - AgentPanel + AgentCard 组件
  - stores（skills、agents）

Step 4: 功能串联 ✅
  - 前端调用 Tauri commands
  - Skill 列表展示（扫描所有目录，合并去重）
  - Agent 列表展示 + 检测状态
  - 创建/删除 symlink
  - 关联状态实时刷新

Step 5: 验证 ✅
  - pnpm tauri dev 启动成功
  - 界面正常显示
```

---

## 10. 预估工期

| 步骤 | 内容 | 预估 | 实际 |
|------|------|------|------|
| Step 1 | 项目初始化 | 半天 | ✅ |
| Step 2 | Rust 后端骨架 | 1 天 | ✅ |
| Step 3 | Vue 前端骨架 | 0.5 天 | ✅ |
| Step 4 | 功能串联 | 0.5 天 | ✅ |
| Step 5 | 验证 | 0.5 天 | ✅ |
| **总计** | | **3 天** | **✅ 完成** |

---

## 11. 与 v0.1 的关系

v0.0 建立的骨架直接被 v0.1 继承：

```
v0.0 已有                    v0.1 追加
─────────────────────────────────────────
commands/skills.rs           + install_skill, delete_skill, preview_skill
commands/sync.rs             + junction, copy 模式
commands/agents.rs           + add_custom_agent
commands/                    + history.rs（撤销/重做）
models/skill.rs              + license, compatibility, metadata, modified_at
models/agent.rs              + auto_detected, linked_skills 详情
models/                      + history.rs（HistoryEntry）
errors.rs                    + PermissionDenied, SymlinkFailed 等
components/skills/           + InstallDialog, SkillPreview
components/agents/           拖拽支持
components/                  + history/, common/, settings/
stores/                      + app.ts
locales/                     新增（i18n）
```

v0.0 不写任何会被 v0.1 废弃的代码。
