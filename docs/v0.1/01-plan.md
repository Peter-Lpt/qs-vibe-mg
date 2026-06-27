# VAB Skills Manager - 实施计划

## 技术栈

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

## 核心概念

### 数据流
```
~/.vab-skills/                    ← 统一库（真实文件 + 配置）
  ├── .vab-config.json            ← agent 配置
  ├── .vab-history.json           ← 操作历史（最近50条）
  ├── my-skill/                   ← skill 实体文件
  │   └── SKILL.md
  └── another-skill/
      └── SKILL.md

~/.claude/skills/my-skill         → symlink → ~/.vab-skills/my-skill
~/.hermes/skills/my-skill         → symlink → ~/.vab-skills/my-skill
~/.config/opencode/skills/my-skill → symlink → ~/.vab-skills/my-skill
~/.codex/skills/my-skill          → symlink → ~/.vab-skills/my-skill
~/.pi/agent/skills/my-skill       → symlink → ~/.vab-skills/my-skill
~/.agents/skills/my-skill         → symlink → ~/.vab-skills/my-skill
```

### 同步模式
- **软连接模式（默认）**：agent 目录创建 symlink 指向 ~/.vab-skills/，更新源文件所有 agent 同步生效
- **复制模式**：直接拷贝文件到 agent 目录，独立副本，更新需重新同步
- 切换按钮在 skill 卡片顶部，切换后只影响新建的关联，已存在的不动

## 项目结构

```
qs-vab-mg/
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # Tauri 命令注册
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── skills.rs         # skill CRUD、安装、删除
│   │   │   ├── sync.rs           # symlink/复制 创建、删除、检测
│   │   │   ├── agents.rs         # agent 检测、自定义 agent 管理
│   │   │   └── history.rs        # 操作记录、撤销、重做
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── skill.rs          # Skill 结构体
│   │   │   ├── agent.rs          # Agent 结构体
│   │   │   └── history.rs        # HistoryEntry 结构体
│   │   ├── parsers/
│   │   │   ├── mod.rs
│   │   │   └── skill_md.rs       # SKILL.md frontmatter 解析（serde_yaml）
│   │   └── utils/
│   │       ├── fs.rs             # 跨平台 symlink、目录操作
│   │       ├── config.rs         # .vab-config.json 读写
│   │       └── path.rs           # ~ 展开、跨平台路径处理
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # Vue 前端
│   ├── App.vue
│   ├── main.ts
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppHeader.vue     # 顶栏：标题、设置、主题切换、语言切换
│   │   │   └── AppLayout.vue     # 整体布局
│   │   ├── skills/
│   │   │   ├── SkillLibrary.vue  # 左侧 skill 列表
│   │   │   ├── SkillCard.vue     # skill 卡片
│   │   │   ├── SkillPreview.vue  # SKILL.md 内容预览弹窗
│   │   │   └── InstallDialog.vue # 安装 skill 弹窗
│   │   ├── agents/
│   │   │   ├── AgentPanel.vue    # 右侧 agent 面板
│   │   │   └── AgentCard.vue     # agent 卡片
│   │   ├── history/
│   │   │   └── HistoryBar.vue    # 底部操作历史 + 撤销/重做
│   │   ├── common/
│   │   │   ├── ConfirmDialog.vue # 通用确认弹窗
│   │   │   └── ToggleSwitch.vue  # 复制/软连接切换
│   │   └── settings/
│   │       └── SettingsPage.vue  # 设置页
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

## 数据模型

### Skill
```rust
struct Skill {
    id: String,                    // 文件夹名
    name: String,                  // SKILL.md frontmatter name
    description: String,           // SKILL.md frontmatter description
    path: String,                  // ~/.vab-skills/{id} 绝对路径
    license: Option<String>,
    compatibility: Option<String>,
    metadata: Option<Value>,
    linked_agents: Vec<AgentLink>,
    modified_at: String,
}

struct AgentLink {
    agent_id: String,
    mode: String,                  // "symlink" | "copy"
    target_path: String,
    valid: bool,                   // symlink 是否有效
}
```

### Agent
```rust
struct Agent {
    id: String,
    name: String,
    skills_dir: String,
    detected: bool,
    enabled: bool,
    auto_detected: bool,
    linked_skills: Vec<String>,
}
```

### HistoryEntry
```rust
struct HistoryEntry {
    id: String,
    timestamp: String,
    action: String,                // "link" | "unlink" | "install" | "delete"
    skill_id: String,
    agent_id: Option<String>,
    mode: Option<String>,
    snapshot: Option<String>,
    undone: bool,
}
```

## 默认 Agent 列表

| Agent | ID | 检测目录 | Skills 路径 |
|-------|-----|---------|------------|
| Claude Code | claude-code | ~/.claude/ | ~/.claude/skills/ |
| Hermes | hermes | ~/.hermes/ | ~/.hermes/skills/ |
| Pi Agent | pi-agent | ~/.pi/ | ~/.pi/agent/skills/ |
| OpenCode | opencode | ~/.config/opencode/ | ~/.config/opencode/skills/ |
| Codex CLI | codex | ~/.codex/ | ~/.codex/skills/ |
| MiMo Code | mimocode | ~/.config/mimocode/ | ~/.config/mimocode/skills/ |
| 共享目录 | agents-shared | ~/.agents/ | ~/.agents/skills/ |

## Dashboard 界面设计

```
┌──────────────────────────────────────────────────────────────┐
│  🛠 VAB Skills Manager        [🌐 中文▾] [🌙] [⚙ 设置]     │
├────────────────────────────┬─────────────────────────────────┤
│ Skills 统一库               │ Agent 关联面板                  │
│                              │                                 │
│ [+ 安装Skill] [批量删除关联]  │  ┌─ claude-code ● 已检测到 ───┐ │
│                              │  │  my-skill    [symlink] [×]  │ │
│ ┌─ my-skill ──────────────┐  │  │  another-sk  [symlink] [×]  │ │
│ │ 📄 my-skill              │  │  └────────────────────────────┘ │
│ │ 网页搜索和内容提取        │  │                                 │
│ │ ~/.vab-skills/my-skill   │  │  ┌─ hermes ● 已检测到 ────────┐ │
│ │ 关联：claude✓ hermes✓    │  │  │  my-skill    [symlink] [×]  │ │
│ │ [symlink ▾]              │  │  └────────────────────────────┘ │
│ └──────────────────────────┘  │                                 │
│                              │  ┌─ pi-agent ○ 未检测到 ──────┐  │
│ ┌─ another-sk ─────────────┐  │  │  （未关联任何 skill）       │  │
│ │ 📄 another-sk            │  │  └────────────────────────────┘  │
│ │ 数据分析工具              │  │                                 │
│ │ ~/.vab-skills/another-sk │  │  ┌─ codex ● 已检测到 ────────┐  │
│ │ 关联：无                  │  │  │  （未关联任何 skill）       │  │
│ │ [symlink ▾]              │  │  └────────────────────────────┘  │
│ └──────────────────────────┘  │                                 │
├────────────────────────────┴─────────────────────────────────┤
│ 操作历史                    [↩ 撤销] [↪ 重做]                 │
│ 14:30 linked my-skill → claude-code (symlink)                │
│ 14:28 installed another-sk                                    │
└──────────────────────────────────────────────────────────────┘
```

## i18n 语言包

```json
{
  "app": { "title": "VAB Skills 管理器", "settings": "设置" },
  "skills": {
    "library": "Skills 统一库",
    "install": "安装 Skill",
    "delete_link": "删除关联",
    "batch_delete": "批量删除关联",
    "preview": "预览",
    "no_skills": "暂无 Skill",
    "sync_mode": "同步模式",
    "symlink": "软连接",
    "copy": "复制"
  },
  "agents": {
    "panel": "Agent 关联面板",
    "detected": "已检测到",
    "not_detected": "未检测到",
    "no_links": "未关联任何 Skill"
  },
  "history": {
    "title": "操作历史",
    "undo": "撤销",
    "redo": "重做"
  }
}
```

## 主题方案

Tailwind CSS dark mode + CSS 变量：
```css
:root {
  --color-bg: #ffffff;
  --color-surface: #f5f5f5;
  --color-text: #1a1a1a;
  --color-primary: #3b82f6;
  --color-success: #22c55e;
  --color-danger: #ef4444;
}
.dark {
  --color-bg: #0f0f0f;
  --color-surface: #1a1a1a;
  --color-text: #e5e5e5;
  --color-primary: #60a5fa;
  --color-success: #4ade80;
  --color-danger: #f87171;
}
```

## 关键技术点

### 1. SKILL.md Frontmatter 解析
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
    license: Option<String>,
    compatibility: Option<String>,
    metadata: Option<serde_json::Value>,
}

fn parse_skill_md(content: &str) -> Option<SkillFrontmatter> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() >= 2 {
        serde_yaml::from_str(parts[1]).ok()
    } else {
        None
    }
}
```

### 2. 跨平台 Symlink
```rust
fn create_symlink(original: &Path, link: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(original, link)?;
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(original, link)?;
    Ok(())
}
```

### 3. Windows 权限检测
```rust
fn check_symlink_permission() -> bool {
    let test_src = std::env::temp_dir().join("vab_test_src");
    let test_dst = std::env::temp_dir().join("vab_test_dst");
    let _ = std::fs::create_dir_all(&test_src);
    let result = create_symlink(&test_src, &test_dst);
    let _ = std::fs::remove_dir_all(&test_src);
    let _ = std::fs::remove_dir_all(&test_dst);
    result.is_ok()
}
```

### 4. 撤销系统
```
操作 → 记录到 history → 执行
撤销 → 读取最后一条未撤销记录 → 执行逆操作 → 标记 undone
重做 → 读取最后一条已撤销记录 → 执行正向操作 → 取消 undone

逆操作映射：
  link    → 删除 symlink/副本
  unlink  → 重新创建 symlink/副本
  install → 删除 skill 文件夹
  delete  → 从 snapshot 恢复文件夹
```

## 预估工期
| 阶段 | 内容 | 时间 |
|------|------|------|
| Phase 1 | 环境搭建 + 项目初始化 | 半天 |
| Phase 2 | Rust 后端核心 | 1.5 天 |
| Phase 3 | Vue 前端基础界面 | 1 天 |
| Phase 4 | Skill 安装与删除 | 0.5 天 |
| Phase 5 | 拖拽关联 + 批量操作 | 1 天 |
| Phase 6 | 设置页 | 0.5 天 |
| Phase 7 | 打包 + 测试 | 0.5 天 |
| **总计** | | **5-6 天** |
