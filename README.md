# QS-Vibe

QS-Vibe 是一个基于 **Tauri 2 + Rust + Vue 3** 的桌面应用，用于统一管理多个 AI Coding Agent 的 Skill。

它以 `~/.vibe-skills/` 作为统一技能库，扫描本机各个 Agent 的技能目录，并通过软链接、同步、批量修复和可视化工作台管理 Skill 的来源与关联关系。

> 项目遵循 [Agent Skills](https://agentskills.io/) 开放标准。

## 功能概览

### Skill 工作台

- 统一查看 Skill 名称、路径、来源和 Agent 关联状态。
- 根据 Agent 数量自适应布局：单 Agent、矩阵视图和多 Agent 横向工作台。
- 支持展开 Skill 详情，查看来源、更新时间、来源可信度和更新能力。
- 对冲突、断链、独立副本等需要处理的 Skill 进行醒目标记。
- 支持多选和批量同步、关联、修复操作。

### 筛选与搜索

- 按名称、Skill ID 和描述搜索。
- 状态预设：全部、需要处理、已关联任一 Agent、未关联任何 Agent。
- 问题筛选：冲突、断链、重复。
- 来源筛选：未进入统一技能库、仅存在于技能库。
- Agent 范围筛选：任一匹配或排除所选 Agent。
- 筛选条件支持单独移除、全部清除和结果数量提示。
- 全选只作用于当前筛选结果，筛选变化后不会误操作隐藏 Skill。

### Agent 管理

- 自动检测常见 Agent 的技能目录。
- 支持新增和自定义 Agent。
- 支持 Agent 名称、技能目录、检测目录和附加扫描目录配置。
- 适配没有 Agent、只有一个 Agent 和多个 Agent 的情况。

### 来源与关联管理

- 识别 Library、Agent、Project 和 External 来源。
- 支持从 Agent 同步到统一技能库。
- 支持将统一技能库中的 Skill 关联到 Agent。
- 支持重新关联、移除断链和替换为技能库版本。
- 对外部来源和来源未知的 Skill 使用保守的来源标识，不伪造来源信息。

### 操作历史

- 记录 Skill 关联、同步、安装和修复操作。
- 支持查看操作历史。
- 支持撤销和重做可逆操作。

### 多语言与主题

当前支持：

- 简体中文
- English
- 繁體中文
- 浅色/深色主题切换

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 桌面容器 | Tauri 2 |
| 后端 | Rust |
| 前端 | Vue 3 + TypeScript |
| 构建工具 | Vite |
| 样式 | Tailwind CSS 4 + 项目设计变量 |
| 状态管理 | Pinia |
| 国际化 | vue-i18n |
| 图标 | Lucide |
| 包管理器 | pnpm |

## 环境要求

- Node.js 18 或更高版本
- pnpm
- Rust stable
- Tauri 2 所需的系统依赖
- Windows 开发时建议开启开发者模式，以便创建软链接

Tauri 官方环境准备说明：

- [Tauri 2 Prerequisites](https://v2.tauri.app/start/prerequisites/)

## 安装与运行

### 安装前端依赖

```bash
pnpm install
```

### 仅启动前端开发服务器

```bash
pnpm dev
```

### 启动完整 Tauri 应用

```bash
pnpm tauri dev
```

首次编译 Rust 依赖可能需要几分钟。

### 构建前端

```bash
pnpm build
```

该命令会执行 TypeScript 类型检查和 Vite 生产构建。

### 构建桌面安装包

```bash
pnpm tauri build
```

### Rust 检查与测试

在 `src-tauri/` 目录执行：

```bash
cargo check
cargo test
```

### Windows Rust 环境变量

如果本机使用非默认 Rust 工具链路径，在 PowerShell 中设置：

```powershell
$env:RUSTUP_HOME = "D:\environment\rust\.rustup"
$env:CARGO_HOME = "D:\environment\rust\.cargo"
```

Windows 创建软链接需要满足以下条件之一：

- 开启 Windows 开发者模式。
- 使用管理员权限运行终端或应用。

## 数据目录与关联模型

### 默认目录

统一技能库：

```text
~/.vibe-skills/
```

配置文件：

```text
~/.vibe-skills/.vibe-config.json
```

操作历史：

```text
~/.vibe-skills/.vibe-history.json
```

### 常见 Agent 技能目录

默认支持的目录包括：

| Agent | 默认技能目录 |
| --- | --- |
| Claude Code | `~/.claude/skills/` |
| Hermes | Windows 通常为 `%LOCALAPPDATA%/hermes/skills/` |
| Pi Agent | `~/.pi/agent/skills/` |
| OpenCode | `~/.config/opencode/skills/` |
| Codex CLI | `~/.codex/skills/` |
| Mimocode | `~/.config/mimocode/skills/` |
| 公共 Agent 目录 | `~/.agents/skills/` |

实际目录可以在 Agent 管理中自定义。

### 两种主要同步方向

#### Agent → 统一技能库

从 Agent 目录同步到 `~/.vibe-skills/`，用于把已有 Agent Skill 纳入统一技能库。

#### 统一技能库 → Agent

从 `~/.vibe-skills/` 创建指向 Agent 技能目录的软链接，用于让多个 Agent 共享同一份 Skill。

应用执行关联和取消关联时会校验路径边界，避免误操作技能目录之外的文件。

## Skill 格式

Skill 目录通常包含一个 `SKILL.md` 文件，使用 YAML frontmatter：

```markdown
---
name: example-skill
description: A short description of the skill.
license: MIT
compatibility: Requires a local development environment.
metadata:
  author: example
---

# Example Skill

Skill instructions go here.
```

必填字段：

- `name`
- `description`

可选字段：

- `license`
- `compatibility`
- `metadata`

## 项目结构

```text
qs-vibe-mg/
├─ src/                              # Vue 前端
│  ├─ components/
│  │  ├─ layout/                    # 应用壳层和导航
│  │  ├─ manage/                    # Skill 工作台、筛选、批量操作
│  │  ├─ skills/                    # Skill 安装和详情
│  │  ├─ agents/                    # Agent 管理
│  │  ├─ history/                   # 操作历史
│  │  ├─ settings/                  # 设置页
│  │  └─ common/                    # 通用组件
│  ├─ composables/                  # Vue 组合式逻辑
│  ├─ stores/                       # Pinia stores
│  ├─ locales/                      # 三种语言包
│  ├─ types/                        # 前端共享类型
│  ├─ icons.ts                      # 全局图标注册
│  └─ style.css                     # 全局设计变量和组件样式
├─ src-tauri/
│  └─ src/
│     ├─ commands/                  # Tauri IPC 命令
│     ├─ models/                    # Rust 数据模型
│     ├─ parsers/                   # SKILL.md 解析
│     ├─ utils/                     # 路径、文件、配置、历史工具
│     ├─ errors.rs                  # 统一错误类型
│     └─ lib.rs                     # 命令注册和应用入口
├─ docs/                            # 版本文档和设计审计
├─ package.json
└─ src-tauri/Cargo.toml
```

## 开发约定

- 前端界面文字必须通过 `vue-i18n`，并同步更新三份语言文件。
- 前端调用后端命令必须经过 Pinia store，不在普通组件中直接调用 `invoke`。
- Rust 错误使用统一的 `VibeError` 类型。
- 筛选、计数和选择状态应使用统一的业务模型，避免在多个组件中重复解析 `Skill.sources`。
- UI 修改优先使用现有设计变量和 Tailwind 工具类。
- 修改软链接、路径边界和文件删除逻辑时必须补充跨平台边界检查。
- 包管理器统一使用 pnpm，不使用 npm 或 yarn。

## 相关文档

- `AGENTS.md`：项目开发约定和目录说明。
- `docs/v0.2/18-filter-system-refactor-plan.md`：筛选系统重构方案、子 Agent 审计结果和验收标准。
- `docs/v0.2/17-skill-workbench-redesign.md`：Skill 工作台设计方案。
- `docs/v0.2/16-manage-ui-audit-and-optimization.md`：管理页 UI 审计记录。

## License

本项目使用仓库中的 `LICENSE` 文件所声明的许可证。
