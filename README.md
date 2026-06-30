# QS-Vab 管理

跨平台 AI Agent Skill 统一管理工具，基于 [Agent Skills](https://agentskills.io) 开放标准构建。

## 功能概览

| 模块 | 说明 |
|------|------|
| **CLI 管理** | 发现、添加、配置 CLI 工具，支持自动检测和自定义 |
| **Skill 列表** | 搜索、预览、管理所有 skill，支持名称和内容搜索 |
| **看板** | 可视化展示 skill 在各 agent 之间的分布与关联关系 |
| **软连接配置** | 层级式批量软连接管理，一键同步 agent skills |

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | Tauri 2 |
| 后端 | Rust |
| 前端 | Vue 3 + TypeScript + Vite |
| 样式 | Tailwind CSS 4 |
| 状态管理 | Pinia |
| i18n | vue-i18n（中文/英文/繁体中文）|

## 构建与运行

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri dev
```

> 首次编译 Rust 需要 3-5 分钟。

### 生产构建

```bash
pnpm tauri build
```

### Windows 注意事项

设置 Rust 环境变量：

```powershell
$env:RUSTUP_HOME = "D:\environment\rust\.rustup"
$env:CARGO_HOME = "D:\environment\rust\.cargo"
```

Windows 创建软连接需要开发者模式或以管理员身份运行。

## 项目结构

```
qs-vibe-mg/
├── src-tauri/                        # Rust 后端
│   ├── src/
│   │   ├── commands/
│   │   │   ├── skills.rs             # skill CRUD、搜索、看板数据
│   │   │   ├── sync.rs               # 软连接创建、删除、批量同步
│   │   │   ├── agents.rs             # agent 管理、层级树
│   │   │   ├── history.rs            # 操作历史、撤销/重做
│   │   │   └── config.rs             # 配置读写
│   │   ├── models/
│   │   │   ├── skill.rs              # Skill 数据模型
│   │   │   ├── agent.rs              # Agent 数据模型
│   │   │   ├── history.rs            # HistoryEntry 数据模型
│   │   │   ├── dashboard.rs          # 看板数据模型
│   │   │   └── sync.rs               # 同步数据模型
│   │   ├── parsers/
│   │   │   └── skill_md.rs           # SKILL.md frontmatter 解析
│   │   ├── errors.rs                 # 统一错误类型
│   │   └── utils/
│   │       ├── fs.rs                 # 跨平台软连接操作
│   │       ├── config.rs             # 配置文件读写
│   │       ├── path.rs               # 路径处理、~ 展开
│   │       └── history.rs            # 历史记录管理
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                              # Vue 前端
│   ├── App.vue                       # 主入口
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppLayout.vue         # 整体布局
│   │   │   └── TabBar.vue            # Tab 导航
│   │   ├── cli/                      # CLI 管理模块
│   │   │   ├── CLITab.vue
│   │   │   ├── CLICard.vue
│   │   │   └── AddCLIDialog.vue
│   │   ├── skills/                   # Skill 列表模块
│   │   │   ├── SkillList.vue
│   │   │   ├── SkillCard.vue
│   │   │   ├── SkillPreview.vue
│   │   │   └── InstallDialog.vue
│   │   ├── dashboard/                # 看板模块
│   │   │   ├── DashboardTab.vue
│   │   │   ├── AgentColumn.vue
│   │   │   └── SharedSkillBar.vue
│   │   ├── symlink/                  # 软连接配置模块
│   │   │   ├── SymlinkTab.vue
│   │   │   ├── AgentExpandable.vue
│   │   │   └── SyncPreview.vue
│   │   ├── history/
│   │   │   └── HistoryBar.vue
│   │   ├── common/
│   │   │   ├── ConfirmDialog.vue
│   │   │   └── ErrorBanner.vue
│   │   └── settings/
│   │       └── SettingsPage.vue
│   ├── stores/                       # Pinia 状态管理
│   │   ├── app.ts
│   │   ├── skills.ts
│   │   ├── agents.ts
│   │   └── history.ts
│   ├── types/
│   │   └── index.ts
│   └── locales/
│       ├── zh.json
│       ├── en.json
│       └── zh-TW.json
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## 数据流

```
~/.vibe-skills/                      # 统一库
├── .vibe-config.json                # 全局配置
├── .vibe-history.json               # 操作历史
├── my-skill/                       # skill 实体
│   ├── SKILL.md
│   ├── scripts/
│   └── references/
└── {agent_id}/                     # agent 同步目录
    └── {skill} → symlink → agent/skills/{skill}
```

## 支持的 Agent

| Agent | 默认路径 |
|-------|---------|
| Claude Code | `~/.claude/skills/` |
| Hermes | `~/.hermes/skills/` |
| Pi Agent | `~/.pi/agent/skills/` |
| OpenCode | `~/.config/opencode/skills/` |
| Codex CLI | `~/.codex/skills/` |
| MiMo Code | `~/.config/mimocode/skills/` |
| Shared | `~/.agents/skills/` |

## SKILL.md 格式

遵循 [Agent Skills](https://agentskills.io/specification) 开放标准：

```yaml
---
name: my-skill
description: A useful skill for code review
license: MIT
metadata:
  author: example-org
  version: "1.0"
---

# Instructions

具体的 skill 执行指令...
```

## i18n

支持三种语言：
- 中文（简体）
- English
- 繁體中文

语言设置保存在 localStorage，支持运行时切换。

## License

MIT
