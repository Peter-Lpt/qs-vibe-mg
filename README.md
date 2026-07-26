# QS-Vibe

跨平台桌面应用，用于统一管理多个 AI Coding Agent 的 Skills。

基于 **Tauri 2 + Rust + Vue 3** 构建，以 `~/.vibe-skills/` 作为统一技能库，通过符号链接实现多 Agent 共享。

> 遵循 [Agent Skills](https://agentskills.io/) 开放标准。

## 功能特性

### Skill 管理

- 统一查看所有 Agent 的 Skills，按来源、状态、Agent 关系筛选
- 从 Agent 目录同步到统一技能库，或从技能库关联到 Agent
- 支持批量操作：同步、关联、修复冲突和断链
- Skill 工作台视图，自适应单 Agent / 多 Agent 布局

### Plugin 支持

- 扫描 Claude Code 和 Codex 的 Plugin 市场缓存
- 显示 Plugin 启用/禁用状态
- 认领 Plugin Skills 到统一技能库
- 检测 Plugin 更新（内容哈希比较）
- 同一市场的 Plugin 批量更新

### Skill 更新检测

- **Git 来源**：`git fetch` + commit 比对，`git pull --ff-only` 执行更新
- **NPX/NPM 来源**：`npm view` 检查远程版本，重新执行安装命令更新
- **Plugin 来源**：内容哈希比较，从缓存目录同步最新文件
- TTL 缓存机制，避免频繁检测

### 来源溯源

- 记录每个 Skill 的安装来源（Git / 本地目录 / NPM / NPX / Plugin 市场）
- 支持 Git 远程 URL、commit SHA、branch 等元数据
- 来源可信度推断和更新状态标记

### Agent 管理

- 自动检测常见 Agent 的技能目录
- 支持自定义 Agent 配置
- Agent 启用/禁用控制

### 其他

- 操作历史记录，支持撤销/重做
- 浅色/深色主题切换
- 简体中文 / English / 繁體中文

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面容器 | Tauri 2 |
| 后端 | Rust |
| 前端 | Vue 3 + TypeScript |
| 构建工具 | Vite |
| 样式 | Tailwind CSS 4 |
| 状态管理 | Pinia |
| 国际化 | vue-i18n |
| 图标 | Lucide |

## 快速开始

### 环境要求

- Node.js 18+
- pnpm
- Rust stable
- Tauri 2 系统依赖（[官方文档](https://v2.tauri.app/start/prerequisites/)）

### 安装与运行

```bash
# 安装依赖
pnpm install

# 启动开发服务器（仅前端）
pnpm dev

# 启动完整 Tauri 应用
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

### Windows 注意事项

- 创建符号链接需要开启开发者模式或以管理员权限运行
- 如使用非默认 Rust 路径，需设置 `RUSTUP_HOME` 和 `CARGO_HOME` 环境变量

## 支持的 Agent

| Agent | 默认技能目录 |
|-------|-------------|
| Claude Code | `~/.claude/skills/` |
| Hermes | `%LOCALAPPDATA%/hermes/skills/` |
| Pi Agent | `~/.pi/agent/skills/` |
| OpenCode | `~/.config/opencode/skills/` |
| Codex CLI | `~/.codex/skills/` |
| MiMo Code | `~/.config/mimocode/skills/` |
| 公共目录 | `~/.agents/skills/` |

可在设置中自定义路径。

## License

[MIT](LICENSE)
