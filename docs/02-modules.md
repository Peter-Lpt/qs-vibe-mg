# VAB Agent Manager - 模块规划

## 已规划模块

### ✅ Skills 管理（v0.1.0 - 当前开发）
- 统一库 ~/.vab-skills/ 管理所有 skill 实体文件
- Symlink/复制 同步到各 agent 的 skills 目录
- Dashboard 展示、拖拽关联、操作撤销
- 状态：开发中

## 预留模块

### 📋 Plugin 管理（v0.2.0 - 规划中）
**目标**：管理各 agent 的 plugin 配置和文件

**各 Agent Plugin 机制**：
| Agent | Plugin 目录 | 格式 |
|-------|------------|------|
| Claude Code | ~/.claude/plugins/ | JSON manifest |
| OpenCode | ~/.config/opencode/plugins/ | TypeScript plugin |
| Codex | ~/.codex/plugins/ | OpenAI plugin format |
| MiMo Code | ~/.config/mimocode/plugins/ | TypeScript plugin |
| Hermes | ~/.hermes/plugins/ | YAML config |

**功能预想**：
- Plugin 发现与扫描
- Plugin 启用/禁用
- Plugin 依赖检查
- Plugin 版本管理

**状态**：待 Skills 模块完成后启动

---

### 🔌 MCP 管理（v0.3.0 - 规划中）
**目标**：统一管理各 agent 的 MCP Server 配置

**各 Agent MCP 配置**：
| Agent | MCP 配置位置 | 格式 |
|-------|-------------|------|
| Claude Code | ~/.claude/mcp.json | JSON |
| OpenCode | ~/.config/opencode/mcp.json | JSON |
| Codex | ~/.codex/mcp.json | JSON |
| Hermes | ~/.hermes/config.yaml (mcp_servers) | YAML |
| MiMo Code | ~/.config/mimocode/mcp.json | JSON |

**功能预想**：
- MCP Server 列表展示
- 添加/删除 MCP Server 配置
- MCP Server 连接状态检测
- 跨 agent 共享 MCP 配置（一键同步）

**状态**：待规划

---

### 📦 Marketplace（v0.4.0 - 远期）
**目标**：在线 skill/plugin 市场

**功能预想**：
- 浏览在线 skill 仓库
- 一键安装 skill 到本地库
- 版本更新检测
- 评分/收藏

**状态**：远期考虑

---

## 架构扩展点

当前架构为后续模块预留了扩展空间：

### 目录结构扩展
```
~/.vab/
├── skills/           # Skills 统一库（v0.1）
├── plugins/          # Plugin 统一库（v0.2 预留）
├── mcp/              # MCP 配置备份（v0.3 预留）
├── .vab-config.json  # 全局配置
└── .vab-history.json # 操作历史
```

### 前端路由扩展
```ts
// router/index.ts 预留
const routes = [
  { path: '/', redirect: '/skills' },
  { path: '/skills', component: SkillsDashboard },
  { path: '/plugins', component: PluginDashboard },   // v0.2
  { path: '/mcp', component: McpDashboard },           // v0.3
  { path: '/marketplace', component: Marketplace },     // v0.4
  { path: '/settings', component: SettingsPage },
]
```

### Rust 后端扩展
```rust
// commands/ 目录已按模块划分
commands/
├── skills.rs      # v0.1
├── sync.rs        # v0.1
├── agents.rs      # v0.1
├── history.rs     # v0.1
├── plugins.rs     # v0.2 预留
└── mcp.rs         # v0.3 预留
```
