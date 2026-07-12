# VIBE Skills Manager - 实施计划

> 版本: v0.1 | 更新: 2026-06-27
> 基于 Agent Skills 开放标准（agentskills.io）构建的跨 Agent Skill 统一管理工具

---

## 1. 技术栈

| 层 | 技术 | 用途 |
|----|------|------|
| 桌面框架 | Tauri 2 | 跨平台窗口、系统 API |
| 后端 | Rust | symlink 操作、文件管理、agent 检测 |
| 前端 | Vue 3 + TypeScript + Vite | UI 界面 |
| 样式 | Tailwind CSS 4 | 样式 + dark mode |
| 状态管理 | Pinia | 前端状态 |
| i18n | vue-i18n | 中/英/繁体 |
| 主题 | Tailwind dark mode | 亮色/暗色 |
| 拖拽 | vue-draggable-plus | skill 拖拽关联 |
| Markdown | marked / markdown-it | SKILL.md 预览 |
| YAML 解析 | serde_yaml (Rust) | SKILL.md frontmatter 解析 |
| 错误处理 | thiserror (Rust) | 后端结构化错误类型 |
| 日志 | tracing (Rust) | 后端日志 |

---

## 2. 核心概念

### 2.1 Agent Skills 开放标准

本项目遵循 [Agent Skills](https://agentskills.io) 开放标准。该标准已被 30+ AI 工具采用，包括：

| Agent | Skills 目录 | 备注 |
|-------|------------|------|
| Claude Code | `~/.claude/skills/` | Agent Skills 标准 |
| Hermes | `~/.hermes/skills/` | Nous Research 开源 agent |
| Pi Agent | `~/.pi/agent/skills/` | 支持跨 harness 指向其他目录 |
| OpenCode | `~/.config/opencode/skills/` | 同时读取 `~/.claude/skills/` |
| Codex CLI | `~/.codex/skills/` | OpenAI 官方 |
| MiMo Code | `~/.config/mimocode/skills/` | 小米 MiMo，基于 OpenCode fork |
| 共享目录 | `~/.agents/skills/` | 多 agent 共享 |

**关键发现**：多个 agent 共享相同的目录路径（如 `~/.agents/skills/`），这意味着 symlink 策略需要考虑这种共享行为。

### 2.2 SKILL.md 规范（Agent Skills 标准）

```yaml
---
name: my-skill                    # 必填，1-64字符，小写字母+数字+连字符
description: 做什么、何时用         # 必填，1-1024字符
license: MIT                      # 可选
compatibility: Requires git, docker  # 可选，≤500字符
metadata:                         # 可选，任意 key-value
  author: example-org
  version: "1.0"
allowed-tools: Read Grep Bash(*)  # 可选，实验性
---

# 指令内容（Markdown）

具体的 skill 执行指令...
```

**name 字段约束**：
- 1-64 字符
- 仅允许小写字母（a-z）、数字（0-9）、连字符（-）
- 不能以连字符开头或结尾
- 不能包含连续连字符（`--`）
- **必须与父目录名一致**

### 2.3 数据流

```
~/.vibe-skills/                      ← 统一库（真实文件 + 配置）
  ├── .vibe-config.json              ← 全局配置
  ├── .vibe-history.json             ← 操作历史（最近50条）
  ├── my-skill/                     ← skill 实体文件
  │   ├── SKILL.md
  │   ├── scripts/                  ← 可选：可执行脚本
  │   ├── references/               ← 可选：参考文档
  │   └── assets/                   ← 可选：模板、资源
  └── another-skill/
      └── SKILL.md

# Symlink 目标（各 agent 的 skills 目录）
~/.claude/skills/my-skill           → symlink → ~/.vibe-skills/my-skill
~/.config/opencode/skills/my-skill  → symlink → ~/.vibe-skills/my-skill
~/.pi/agent/skills/my-skill         → symlink → ~/.vibe-skills/my-skill
~/.codex/skills/my-skill            → symlink → ~/.vibe-skills/my-skill
~/.agents/skills/my-skill           → symlink → ~/.vibe-skills/my-skill
```

### 2.4 同步模式

- **软连接模式（默认）**：agent 目录创建 symlink 指向 ~/.vibe-skills/，更新源文件所有 agent 同步生效
- **复制模式**：直接拷贝文件到 agent 目录，独立副本，更新需重新同步
- **切换规则**：切换按钮在 skill 卡片顶部，切换后只影响新建的关联，已存在的不动

---

## 3. 项目结构

```
qs-vibe-mg/
├── src-tauri/                        # Rust 后端
│   ├── src/
│   │   ├── main.rs                   # 入口
│   │   ├── lib.rs                    # Tauri 命令注册
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── skills.rs             # skill CRUD、安装、删除
│   │   │   ├── sync.rs               # symlink/复制 创建、删除、检测
│   │   │   ├── agents.rs             # agent 检测、自定义 agent 管理
│   │   │   └── history.rs            # 操作记录、撤销、重做
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── skill.rs              # Skill 结构体
│   │   │   ├── agent.rs              # Agent 结构体
│   │   │   └── history.rs            # HistoryEntry 结构体
│   │   ├── parsers/
│   │   │   ├── mod.rs
│   │   │   └── skill_md.rs           # SKILL.md frontmatter 解析
│   │   ├── errors.rs                 # 统一错误类型（thiserror）
│   │   └── utils/
│   │       ├── fs.rs                 # 跨平台 symlink、目录操作
│   │       ├── config.rs             # .vibe-config.json 读写
│   │       ├── path.rs               # ~ 展开、跨平台路径处理
│   │       └── watcher.rs            # 文件系统监听（可选）
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                              # Vue 前端
│   ├── App.vue
│   ├── main.ts
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppHeader.vue         # 顶栏：标题、设置、主题切换、语言切换
│   │   │   └── AppLayout.vue         # 整体布局
│   │   ├── skills/
│   │   │   ├── SkillLibrary.vue      # 左侧 skill 列表
│   │   │   ├── SkillCard.vue         # skill 卡片
│   │   │   ├── SkillPreview.vue      # SKILL.md 内容预览弹窗
│   │   │   └── InstallDialog.vue     # 安装 skill 弹窗
│   │   ├── agents/
│   │   │   ├── AgentPanel.vue        # 右侧 agent 面板
│   │   │   └── AgentCard.vue         # agent 卡片
│   │   ├── history/
│   │   │   └── HistoryBar.vue        # 底部操作历史 + 撤销/重做
│   │   ├── common/
│   │   │   ├── ConfirmDialog.vue     # 通用确认弹窗
│   │   │   ├── ToggleSwitch.vue      # 复制/软连接切换
│   │   │   ├── ErrorBanner.vue       # 错误提示横幅
│   │   │   └── LoadingSpinner.vue    # 加载状态
│   │   └── settings/
│   │       └── SettingsPage.vue      # 设置页
│   ├── composables/
│   │   ├── useSkills.ts
│   │   ├── useAgents.ts
│   │   ├── useHistory.ts
│   │   └── useTheme.ts
│   ├── stores/
│   │   ├── skills.ts
│   │   ├── agents.ts
│   │   └── app.ts
│   ├── locales/
│   │   ├── zh.json
│   │   ├── en.json
│   │   └── zh-TW.json
│   └── types/
│       └── index.ts
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.ts
└── index.html
```

---

## 4. 数据模型

### 4.1 Skill

```rust
struct Skill {
    id: String,                        // 文件夹名（= name 字段）
    name: String,                      // SKILL.md frontmatter name
    description: String,               // SKILL.md frontmatter description
    path: String,                      // ~/.vibe-skills/{id} 绝对路径
    license: Option<String>,
    compatibility: Option<String>,
    metadata: Option<HashMap<String, String>>,  // 改为 HashMap，符合标准
    linked_agents: Vec<AgentLink>,
    modified_at: String,
    has_scripts: bool,                 // 是否包含 scripts/ 目录
    has_references: bool,              // 是否包含 references/ 目录
    has_assets: bool,                  // 是否包含 assets/ 目录
}

struct AgentLink {
    agent_id: String,
    mode: String,                      // "symlink" | "copy" | "junction"
    target_path: String,
    valid: bool,                       // symlink 是否有效（未断链）
    created_at: String,
}
```

### 4.2 Agent

```rust
struct Agent {
    id: String,
    name: String,
    skills_dir: String,
    detected: bool,                    // agent 目录是否存在
    enabled: bool,                     // 用户是否启用
    auto_detected: bool,               // 是否为自动检测（非用户自定义）
    linked_skills: Vec<String>,        // 已关联的 skill id 列表
    config_path: Option<String>,       // agent 配置文件路径（如 .claude/settings.json）
}
```

### 4.3 HistoryEntry

```rust
struct HistoryEntry {
    id: String,
    timestamp: String,                 // ISO 8601 格式
    action: HistoryAction,
    skill_id: String,
    agent_id: Option<String>,
    mode: Option<String>,              // "symlink" | "copy" | "junction"
    snapshot: Option<String>,          // 操作前状态快照路径（用于 undo）
    undone: bool,
}

enum HistoryAction {
    Link,       // 创建 symlink/副本
    Unlink,     // 删除 symlink/副本
    Install,    // 安装 skill（从外部导入）
    Delete,     // 删除 skill
    BatchLink,  // 批量关联
    BatchUnlink,// 批量取消关联
}
```

---

## 5. 配置文件结构

### 5.1 `.vibe-config.json`

```json
{
  "version": 1,
  "sync_mode_default": "symlink",
  "agents": [
    {
      "id": "custom-agent",
      "name": "My Custom Agent",
      "skills_dir": "~/.custom/skills/",
      "auto_detected": false,
      "enabled": true
    }
  ],
  "ui": {
    "theme": "system",
    "locale": "zh-CN"
  },
  "history": {
    "max_entries": 50,
    "snapshot_max_size_mb": 100
  }
}
```

### 5.2 `.vibe-history.json`

```json
{
  "version": 1,
  "entries": []
}
```

> history 条目直接追加到 entries 数组，超过 max_entries 时从头部删除。

---

## 6. 默认 Agent 列表

| Agent | ID | 检测目录 | Skills 路径 | 备注 |
|-------|-----|---------|------------|------|
| Claude Code | claude-code | ~/.claude/ | ~/.claude/skills/ | Agent Skills 标准 |
| Hermes | hermes | ~/.hermes/ | ~/.hermes/skills/ | Nous Research 开源 |
| Pi Agent | pi-agent | ~/.pi/ | ~/.pi/agent/skills/ | 支持跨 harness |
| OpenCode | opencode | ~/.config/opencode/ | ~/.config/opencode/skills/ | 同时读 ~/.claude/skills/ |
| Codex CLI | codex | ~/.codex/ | ~/.codex/skills/ | OpenAI 官方 |
| MiMo Code | mimocode | ~/.config/mimocode/ | ~/.config/mimocode/skills/ | 小米 MiMo |
| 共享目录 | agents-shared | ~/.agents/ | ~/.agents/skills/ | 多 agent 共享 |

### Agent 检测逻辑

```rust
fn detect_agent(agent: &AgentConfig) -> bool {
    // 1. 检查 agent 基础目录是否存在
    let base_dir = expand_path(&agent.detection_dir);
    if !base_dir.exists() {
        return false;
    }

    // 2. 检查 skills 目录是否存在（不存在也视为已检测，可能是首次使用）
    let skills_dir = expand_path(&agent.skills_dir);
    // skills_dir 可以不存在，创建时自动建立

    // 3. 可选：检查 agent 的标志性文件
    //    如 Claude Code 检查 ~/.claude/settings.json
    //    如 Pi 检查 ~/.pi/package.json
    true
}
```

### 自定义 Agent 管理

用户可通过设置页添加自定义 agent：

```rust
struct CustomAgentInput {
    name: String,           // 显示名称
    skills_dir: String,     // skills 目录路径（支持 ~ 展开）
    detection_dir: Option<String>,  // 检测目录（可选）
}
```

UI 流程：设置页 → "添加 Agent" → 填写名称 + skills 路径 → 保存到 `.vibe-config.json` → 主面板显示新 agent。

---

## 7. Windows 权限与降级策略

### 7.1 权限检测

```rust
fn check_symlink_permission() -> SymlinkCapability {
    let test_src = std::env::temp_dir().join("vibe_test_src");
    let test_dst = std::env::temp_dir().join("vibe_test_dst");
    let _ = std::fs::create_dir_all(&test_src);

    // 尝试创建 directory symlink
    let result = create_symlink(&test_src, &test_dst);
    let _ = std::fs::remove_dir_all(&test_src);
    let _ = std::fs::remove_dir_all(&test_dst);

    match result {
        Ok(()) => SymlinkCapability::Symlink,
        Err(_) => {
            // 尝试 junction（不需要管理员权限）
            let test_junc = std::env::temp_dir().join("vibe_test_junc");
            let result = create_junction(&test_src, &test_junc);
            let _ = std::fs::remove_dir_all(&test_junc);
            match result {
                Ok(()) => SymlinkCapability::Junction,
                Err(_) => SymlinkCapability::CopyOnly,
            }
        }
    }
}

enum SymlinkCapability {
    Symlink,      // 完整 symlink 支持
    Junction,     // 仅支持 junction（Windows，不需要管理员权限）
    CopyOnly,     // 仅支持复制模式
}
```

### 7.2 降级策略

```
启动时检测 →
  ├── Symlink 可用 → 使用 symlink 模式（默认）
  ├── Junction 可用 → 使用 junction 模式
  │   └── 提示用户："当前系统不支持 symlink，已降级为 junction 模式"
  └── 仅 Copy → 使用复制模式
      └── 提示用户："当前系统仅支持复制模式，skill 更新不会自动同步"
          └── 提供"以管理员身份运行"按钮（重新检测）
```

### 7.3 Junction vs Symlink

| 特性 | Symlink | Junction | Copy |
|------|---------|----------|------|
| 需要管理员权限 | ✅ 是 | ❌ 否 | ❌ 否 |
| 支持文件 | ✅ 文件+目录 | ⚠️ 仅目录 | ✅ 文件+目录 |
| 跨文件系统 | ✅ 支持 | ❌ 不支持 | ✅ 支持 |
| 删除时行为 | 删除链接 | 删除链接 | 删除副本 |
| Agent 兼容性 | 最佳 | 良好 | 一般 |

---

## 8. 错误处理

### 8.1 后端错误类型

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VibeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Skill not found: {skill_id}")]
    SkillNotFound { skill_id: String },

    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: String },

    #[error("Invalid SKILL.md format: {reason}")]
    InvalidSkillMd { reason: String },

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("Symlink creation failed: {reason}")]
    SymlinkFailed { reason: String },

    #[error("Config file corrupted: {path}")]
    ConfigCorrupted { path: String },

    #[error("History snapshot too large: {size_mb}MB exceeds limit")]
    SnapshotTooLarge { size_mb: u64 },
}

// 前端看到的结构化错误
#[derive(Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum FrontendError {
    #[serde(rename_all = "camelCase")]
    Io { message: String },
    #[serde(rename_all = "camelCase")]
    SkillNotFound { skill_id: String },
    #[serde(rename_all = "camelCase")]
    PermissionDenied { operation: String },
    #[serde(rename_all = "camelCase")]
    SymlinkFailed { reason: String },
    #[serde(rename_all = "camelCase")]
    ConfigCorrupted { path: String },
}
```

### 8.2 前端错误展示

```
┌─────────────────────────────────────────────────┐
│ ⚠️ Symlink 创建失败                              │
│                                                   │
│ 检测到当前系统不支持 symlink 创建。               │
│                                                   │
│ 可选操作：                                        │
│ [以管理员身份运行]  [切换为 Junction]  [使用复制]  │
└─────────────────────────────────────────────────┘
```

---

## 9. 撤销系统

### 9.1 Snapshot 策略

| 操作 | Snapshot 内容 | 存储位置 | 大小控制 |
|------|-------------|---------|---------|
| install | skill 文件夹的元数据（SKILL.md 内容 + 文件列表） | `.vibe-history.json` 内嵌 | 单条 ≤ 1MB |
| delete | skill 文件夹完整备份 | `.vibe-history/snapshots/{id}/` | 总量 ≤ 100MB |
| link | 无需 snapshot（删除 symlink 即可 undo） | - | - |
| unlink | 无需 snapshot（重新创建即 undo） | - | - |

### 9.2 撤销逻辑

```
操作 → 记录到 history → 执行
撤销 → 读取最后一条未撤销记录 → 执行逆操作 → 标记 undone
重做 → 读取最后一条已撤销记录 → 执行正向操作 → 取消 undone

逆操作映射：
  link        → 删除 symlink/副本/junction
  unlink      → 重新创建 symlink/副本/junction
  install     → 删除 skill 文件夹（从 ~/.vibe-skills/ 移除）
  delete      → 从 snapshot 恢复文件夹到 ~/.vibe-skills/
  batch_link  → 批量删除 symlink/副本/junction
  batch_unlink→ 批量重新创建 symlink/副本/junction
```

### 9.3 Snapshot 大小限制

- 单条 snapshot 最大 1MB（超过时仅记录元数据，不记录完整内容）
- 所有 snapshot 总量最大 100MB
- 超出时从最旧的开始清理
- 安装操作的 snapshot 仅记录 SKILL.md 内容和文件列表（不备份大文件）

---

## 10. Dashboard 界面设计

### 10.1 正常状态

```
┌──────────────────────────────────────────────────────────────┐
│  🛠 VIBE Skills Manager        [🌐 中文▾] [🌙] [⚙ 设置]     │
├────────────────────────────────┬─────────────────────────────┤
│ Skills 统一库                   │ Agent 关联面板              │
│                                  │                             │
│ [+ 安装Skill] [批量删除关联]     │  ┌─ claude-code ● 已检测 ──┐│
│ 🔍 搜索 skills...               │  │  my-skill    [symlink][×]││
│                                  │  │  another-sk  [symlink][×]││
│ ┌─ my-skill ──────────────────┐ │  └──────────────────────────┘│
│ │ 📄 my-skill                  │ │                             │
│ │ 网页搜索和内容提取            │ │  ┌─ opencode ● 已检测 ────┐│
│ │ ~/.vibe-skills/my-skill       │ │  │  my-skill    [symlink][×]││
│ │ 关联：claude✓ opencode✓      │ │  └──────────────────────────┘│
│ │ [symlink ▾]                  │ │                             │
│ └──────────────────────────────┘ │  ┌─ pi-agent ○ 未检测 ────┐ │
│                                  │  │  （未关联任何 skill）    │ │
│ ┌─ another-sk ────────────────┐ │  └──────────────────────────┘│
│ │ 📄 another-sk                │ │                             │
│ │ 数据分析工具                  │ │  ┌─ agents-shared ● 已检测┐ │
│ │ ~/.vibe-skills/another-sk     │ │  │  （未关联任何 skill）    │ │
│ │ 关联：无                      │ │  └──────────────────────────┘│
│ │ [symlink ▾]                  │ │                             │
│ └──────────────────────────────┘ │                             │
├────────────────────────────────┴─────────────────────────────┤
│ 操作历史                    [↩ 撤销] [↪ 重做]                 │
│ 14:30 linked my-skill → claude-code (symlink)                │
│ 14:28 installed another-sk                                    │
└──────────────────────────────────────────────────────────────┘
```

### 10.2 错误状态

```
┌──────────────────────────────────────────────────────────────┐
│ ⚠️ 部分 skill 关联已失效                                     │
│ my-skill → claude-code: symlink 目标不存在                    │
│ [重新关联] [删除失效关联] [忽略]                              │
├──────────────────────────────────────────────────────────────┤
```

### 10.3 Agent 未检测到引导

```
┌──────────────────────────────────────────────────┐
│  ○ pi-agent 未检测到                              │
│                                                    │
│  未找到 ~/.pi/ 目录。                              │
│  如果已安装 Pi Agent，请检查安装路径。             │
│  或者手动添加 skills 目录：                        │
│  [手动添加路径]                                    │
└──────────────────────────────────────────────────┘
```

### 10.4 权限不足提示

```
┌──────────────────────────────────────────────────┐
│ 🔒 当前系统不支持创建 symlink                     │
│                                                    │
│ Windows 需要管理员权限或开启开发者模式。          │
│                                                    │
│ [以管理员身份重启] [使用 Junction 模式] [使用复制]│
└──────────────────────────────────────────────────┘
```

---

## 11. 拖拽关联交互

### 11.1 交互流程

```
从 SkillCard 拖拽 →
  ├── 拖到 AgentCard → 创建单个关联
  │   └── 弹出选择：symlink / junction / copy
  ├── 拖到 AgentPanel 空白区域 → 创建关联（使用默认模式）
  └── 拖到 AgentPanel 外 → 取消

从 AgentCard 拖拽 SkillChip →
  ├── 拖到另一个 AgentCard → 复制关联到新 agent
  └── 拖到 trash 区域 → 删除关联
```

### 11.2 批量操作

- 选中多个 skill（checkbox）→ 拖拽到 agent → 批量关联
- 选中多个 agent（checkbox）→ 点击"批量关联"→ 选择 skill → 批量关联

---

## 12. 关键技术点

### 12.1 SKILL.md Frontmatter 解析（修复版）

```rust
use serde::{Deserialize, Serialize};
use crate::errors::VibeError;

#[derive(Debug, Deserialize, Serialize)]
struct SkillFrontmatter {
    name: String,                       // 必填
    description: String,                // 必填
    license: Option<String>,
    compatibility: Option<String>,
    #[serde(default)]
    metadata: Option<HashMap<String, String>>,
    #[serde(rename = "allowed-tools")]
    allowed_tools: Option<String>,
}

/// 解析 SKILL.md，返回 frontmatter 和 body
fn parse_skill_md(content: &str) -> Result<(SkillFrontmatter, String), VibeError> {
    // 找到第一个 --- 和第二个 ---
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(VibeError::InvalidSkillMd {
            reason: "Missing frontmatter opening delimiter".into(),
        });
    }

    // 跳过开头的 ---
    let after_first = &trimmed[3..];

    // 查找第二个 ---
    let second_delim = after_first
        .find("\n---")
        .or_else(|| after_first.find("\r\n---"))
        .ok_or_else(|| VibeError::InvalidSkillMd {
            reason: "Missing frontmatter closing delimiter".into(),
        })?;

    let yaml_part = &after_first[..second_delim];
    let body_start = second_delim + 4; // 跳过 \n---

    // 跳过 --- 后的换行符
    let body = content[content.len() - (trimmed.len() - 3 - second_delim - 4)..].trim();

    let frontmatter: SkillFrontmatter = serde_yaml::from_str(yaml_part)
        .map_err(|e| VibeError::InvalidSkillMd {
            reason: format!("YAML parse error: {}", e),
        })?;

    // 验证 name 字段
    validate_skill_name(&frontmatter.name)?;

    Ok((frontmatter, body.to_string()))
}

/// 验证 skill name 符合 Agent Skills 标准
fn validate_skill_name(name: &str) -> Result<(), VibeError> {
    if name.is_empty() || name.len() > 64 {
        return Err(VibeError::InvalidSkillMd {
            reason: format!("Name must be 1-64 characters, got {}", name.len()),
        });
    }
    if name.starts_with('-') || name.ends_with('-') {
        return Err(VibeError::InvalidSkillMd {
            reason: "Name cannot start or end with hyphen".into(),
        });
    }
    if name.contains("--") {
        return Err(VibeError::InvalidSkillMd {
            reason: "Name cannot contain consecutive hyphens".into(),
        });
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(VibeError::InvalidSkillMd {
            reason: "Name must contain only lowercase letters, digits, and hyphens".into(),
        });
    }
    Ok(())
}
```

### 12.2 跨平台链接创建

```rust
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinkMode {
    Symlink,
    Junction,
    Copy,
}

fn create_link(original: &Path, link: &Path, mode: LinkMode) -> Result<(), std::io::Error> {
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 如果链接已存在，先删除
    if link.exists() || link.symlink_metadata().is_ok() {
        std::fs::remove_dir_all(link)?;
    }

    match mode {
        LinkMode::Symlink => {
            #[cfg(unix)]
            std::os::unix::fs::symlink(original, link)?;
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(original, link)?;
        }
        LinkMode::Junction => {
            #[cfg(windows)]
            std::os::windows::fs::junction(original, link)?;
            #[cfg(not(windows))]
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Junctions are only supported on Windows",
            ));
        }
        LinkMode::Copy => {
            copy_dir_all(original, link)?;
        }
    }
    Ok(())
}

fn remove_link(link: &Path, mode: LinkMode) -> Result<(), std::io::Error> {
    match mode {
        LinkMode::Copy => {
            std::fs::remove_dir_all(link)?;
        }
        _ => {
            // symlink 和 junction 都只删除链接本身
            std::fs::remove_dir(link)?;
        }
    }
    Ok(())
}

/// 检查链接是否有效（目标存在）
fn is_link_valid(link: &Path) -> bool {
    if let Ok(metadata) = std::fs::symlink_metadata(link) {
        if metadata.file_type().is_symlink() {
            return std::fs::read_link(link)
                .ok()
                .and_then(|target| {
                    // 对 junction，检查目标是否存在
                    if target.exists() { Some(()) } else { None }
                })
                .is_some();
        }
        // junction 检查
        #[cfg(windows)]
        {
            return link.exists();
        }
    }
    false
}
```

### 12.3 Tauri 命令注册

```rust
// lib.rs
mod commands;
mod errors;
mod models;
mod parsers;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(models::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::skills::list_skills,
            commands::skills::install_skill,
            commands::skills::delete_skill,
            commands::skills::preview_skill,
            commands::sync::link_skill,
            commands::sync::unlink_skill,
            commands::sync::check_link_status,
            commands::agents::list_agents,
            commands::agents::detect_agents,
            commands::agents::add_custom_agent,
            commands::agents::remove_custom_agent,
            commands::history::get_history,
            commands::history::undo,
            commands::history::redo,
            commands::config::get_config,
            commands::config::update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 12.4 文件系统监听（可选增强）

```rust
// 监听 ~/.vibe-skills/ 目录变化，自动刷新前端
// 使用 notify crate
use notify::{Watcher, RecursiveMode, watcher};

fn start_watching(path: &Path, app_handle: tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;
    watcher.watch(path, RecursiveMode::Recursive)?;

    std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            // 通知前端刷新
            let _ = app_handle.emit("skills-changed", event);
        }
    });

    Ok(())
}
```

---

## 13. 日志方案

### 13.1 Rust 后端日志

```rust
// 使用 tracing
use tracing::{info, warn, error, debug};

// 初始化
tracing_subscriber::fmt()
    .with_file(true)
    .with_line_number(true)
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "qs_vibe_mg=debug".into())
    )
    .init();

// 日志文件位置
// Windows: %LOCALAPPDATA%/qs-vibe-mg/logs/
// macOS: ~/Library/Logs/qs-vibe-mg/
// Linux: ~/.local/share/qs-vibe-mg/logs/
```

### 13.2 前端调试

- 开发模式：Vite devtools + Vue DevTools
- 生产模式：Tauri 调试窗口（Ctrl+Shift+I）
- 操作日志：通过 `HistoryBar.vue` 查看最近操作

---

## 14. i18n 语言包

```json
{
  "app": {
    "title": "VIBE Skills 管理器",
    "settings": "设置",
    "loading": "加载中...",
    "error": "错误"
  },
  "skills": {
    "library": "Skills 统一库",
    "install": "安装 Skill",
    "delete": "删除 Skill",
    "delete_link": "删除关联",
    "batch_delete": "批量删除关联",
    "preview": "预览",
    "search": "搜索 skills...",
    "no_skills": "暂无 Skill",
    "sync_mode": "同步模式",
    "symlink": "软连接",
    "junction": "Junction",
    "copy": "复制",
    "linked_agents": "关联：",
    "none": "无",
    "has_scripts": "包含脚本",
    "has_references": "包含参考文档",
    "has_assets": "包含资源文件",
    "invalid_link": "关联已失效",
    "relink": "重新关联"
  },
  "agents": {
    "panel": "Agent 关联面板",
    "detected": "已检测到",
    "not_detected": "未检测到",
    "no_links": "未关联任何 Skill",
    "add_custom": "添加自定义 Agent",
    "remove": "移除 Agent",
    "manual_path": "手动添加路径",
    "not_found_hint": "未找到 {path} 目录。如果已安装 {name}，请检查安装路径。"
  },
  "history": {
    "title": "操作历史",
    "undo": "撤销",
    "redo": "重做",
    "empty": "暂无操作记录",
    "linked": "已关联 {skill} → {agent} ({mode})",
    "unlinked": "已取消关联 {skill} → {agent}",
    "installed": "已安装 {skill}",
    "deleted": "已删除 {skill}"
  },
  "errors": {
    "permission_denied": "权限不足",
    "symlink_failed": "Symlink 创建失败",
    "junction_failed": "Junction 创建失败",
    "skill_not_found": "Skill 未找到: {id}",
    "agent_not_found": "Agent 未找到: {id}",
    "invalid_skill_md": "SKILL.md 格式无效: {reason}",
    "admin_required": "需要管理员权限",
    "fallback_to_junction": "已降级为 Junction 模式",
    "fallback_to_copy": "已降级为复制模式",
    "restart_as_admin": "以管理员身份重启",
    "use_junction": "使用 Junction 模式",
    "use_copy": "使用复制模式"
  },
  "settings": {
    "title": "设置",
    "theme": "主题",
    "language": "语言",
    "default_sync_mode": "默认同步模式",
    "max_history": "最大历史记录数",
    "snapshot_limit": "快照大小限制 (MB)",
    "add_agent": "添加 Agent",
    "agent_name": "Agent 名称",
    "skills_dir": "Skills 目录路径"
  },
  "drag": {
    "drop_to_agent": "拖到 Agent 以关联",
    "drop_to_trash": "拖到这里取消关联",
    "select_mode": "选择关联模式"
  }
}
```

---

## 15. 主题方案

Tailwind CSS dark mode + CSS 变量：

```css
:root {
  --color-bg: #ffffff;
  --color-surface: #f5f5f5;
  --color-surface-hover: #eeeeee;
  --color-text: #1a1a1a;
  --color-text-secondary: #6b7280;
  --color-border: #e5e7eb;
  --color-primary: #3b82f6;
  --color-primary-hover: #2563eb;
  --color-success: #22c55e;
  --color-warning: #f59e0b;
  --color-danger: #ef4444;
  --color-danger-hover: #dc2626;
  --color-symlink: #8b5cf6;
  --color-junction: #06b6d4;
  --color-copy: #f97316;
}

.dark {
  --color-bg: #0f0f0f;
  --color-surface: #1a1a1a;
  --color-surface-hover: #252525;
  --color-text: #e5e5e5;
  --color-text-secondary: #9ca3af;
  --color-border: #2d2d2d;
  --color-primary: #60a5fa;
  --color-primary-hover: #93bbfd;
  --color-success: #4ade80;
  --color-warning: #fbbf24;
  --color-danger: #f87171;
  --color-danger-hover: #fca5a5;
  --color-symlink: #a78bfa;
  --color-junction: #22d3ee;
  --color-copy: #fb923c;
}
```

---

## 16. 并发安全

### 16.1 文件锁

```rust
use std::sync::Mutex;

// 全局操作锁，防止并发写入冲突
pub struct OperationLock {
    inner: Mutex<()>,
}

impl OperationLock {
    pub fn new() -> Self {
        Self { inner: Mutex::new(()) }
    }

    pub fn with_lock<F, R>(&self, f: F) -> Result<R, VibeError>
    where
        F: FnOnce() -> Result<R, VibeError>,
    {
        let _guard = self.inner.lock().map_err(|_| VibeError::LockPoisoned)?;
        f()
    }
}
```

### 16.2 多实例检测

- 启动时检查 `.vibe-skills/.vibe-lock` 文件
- 如果锁文件存在且进程仍运行 → 提示用户关闭其他实例
- 如果锁文件存在但进程已退出 → 清理锁文件，正常启动

---

## 17. 测试策略

### 17.1 单元测试

```rust
// parsers/skill_md.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_skill_md() {
        let content = r#"---
name: my-skill
description: A test skill
license: MIT
---
# Instructions
Do something."#;
        let (fm, body) = parse_skill_md(content).unwrap();
        assert_eq!(fm.name, "my-skill");
        assert_eq!(fm.description, "A test skill");
        assert!(body.contains("Do something"));
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# Just a markdown file";
        assert!(parse_skill_md(content).is_err());
    }

    #[test]
    fn test_validate_name_rules() {
        assert!(validate_skill_name("my-skill").is_ok());
        assert!(validate_skill_name("a").is_ok());
        assert!(validate_skill_name("skill123").is_ok());
        assert!(validate_skill_name("-bad").is_err());
        assert!(validate_skill_name("bad-").is_err());
        assert!(validate_skill_name("bad--name").is_err());
        assert!(validate_skill_name("BAD").is_err());
    }

    #[test]
    fn test_parse_with_metadata() {
        let content = r#"---
name: test
description: Test skill
metadata:
  author: test-org
  version: "2.0"
---
Body content"#;
        let (fm, _) = parse_skill_md(content).unwrap();
        let meta = fm.metadata.unwrap();
        assert_eq!(meta.get("author").unwrap(), "test-org");
        assert_eq!(meta.get("version").unwrap(), "2.0");
    }
}
```

### 17.2 集成测试

- symlink/junction/copy 创建和删除
- 跨平台路径处理
- 配置文件读写
- 撤销/重做流程
- Agent 检测

### 17.3 E2E 测试

- 使用 Tauri 的 WebDriver 支持
- 安装 skill → 关联 agent → 验证 symlink 存在
- 撤销操作 → 验证状态恢复

---

## 18. 数据迁移策略

### 18.1 版本号规则

- 配置文件和历史文件都有 `version` 字段
- 每次格式变更递增版本号

### 18.2 迁移示例

```rust
fn migrate_config(config: &mut serde_json::Value) {
    let version = config["version"].as_i64().unwrap_or(0);
    if version < 1 {
        // v0 → v1: 添加 version 字段
        config["version"] = serde_json::json!(1);
        // 添加默认值
        if config.get("history").is_none() {
            config["history"] = serde_json::json!({
                "max_entries": 50,
                "snapshot_max_size_mb": 100
            });
        }
    }
    // 未来版本迁移...
}
```

---

## 19. 预估工期

| 阶段 | 内容 | 时间 |
|------|------|------|
| Phase 1 | 环境搭建 + 项目初始化 | 半天 |
| Phase 2 | Rust 后端核心（解析、链接、配置） | 2 天 |
| Phase 3 | Vue 前端基础界面 | 1 天 |
| Phase 4 | Skill 安装与删除 | 0.5 天 |
| Phase 5 | 拖拽关联 + 批量操作 | 1 天 |
| Phase 6 | 撤销系统 + 错误处理 | 1 天 |
| Phase 7 | 设置页 + 自定义 Agent | 0.5 天 |
| Phase 8 | 打包 + 测试 + 修复 | 1 天 |
| **总计** | | **7-8 天** |

---

## 20. 参考资料

| 资源 | 链接 | 说明 |
|------|------|------|
| Agent Skills 标准 | https://agentskills.io/specification | SKILL.md 格式规范 |
| Agent Skills 标准仓库 | https://github.com/agentskills/agentskills | 开放标准讨论 |
| Claude Code Skills 文档 | https://code.claude.com/docs/en/skills | Claude Code 的 skills 实现 |
| OpenCode Skills | https://opencode.ai/docs/skills/ | OpenCode 的 skills 实现 |
| Pi Agent Skills | https://github.com/badlogic/pi-mono | Pi 的 skills 实现 |
| Tauri 2 命令系统 | https://v2.tauri.app/develop/calling-rust/ | Tauri 命令定义和调用 |
| Windows Symbolic Links | https://learn.microsoft.com/en-us/windows/win32/fileio/symbolic-links | Windows symlink 文档 |
| thiserror | https://docs.rs/thiserror | Rust 错误处理 |
| tracing | https://docs.rs/tracing | Rust 日志 |
