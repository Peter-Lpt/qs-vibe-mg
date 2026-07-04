# QS-Vibe v0.2 优化建议

## 一、重新理解产品

### 核心场景

用户同时在用 Claude Code、Hermes、Codex 等多个 AI 编码助手，每个 Agent 的 skills 目录里可能各自有一些 Skill。问题是：

1. **分散** — Skill 散落在各个 Agent 目录里，没有统一视图
2. **重复** — 同一个 Skill 可能在多个 Agent 下各存一份
3. **无法集中管理** — 想批量操作（比如给所有 Agent 加一个 Skill）很麻烦
4. **更新困难** — 从 GitHub 装的 Skill 有新版本了，不知道怎么更新

### 产品要解决的核心问题

> **把分散在各 Agent 下的 Skill 集中到 `~/.vibe-skills/` 统一管理，同时保持按 Agent 分组的清晰结构。**

目录结构（所有 skill 均通过 symlink 指向 agent 真实目录，不复制文件）：
```
~/.vibe-skills/
├── .vibe-config.json          # 配置
├── .vibe-history.json         # 操作历史
├── .vibe-sync-state.json      # 同步快照，用于回滚
│
├── claude-code/               # 从 Claude Code 同步过来的
│   ├── coding-standard/       # symlink → ~/.claude/skills/coding-standard
│   └── review-helper/         # symlink → ~/.claude/skills/review-helper
├── hermes/                    # 从 Hermes 同步过来的
│   └── research-tools/        # symlink → ~/.hermes/skills/research-tools
│
└── shared/                    # 用户手动安装的 skill（不属于任何 agent）
    └── utility-skill/
        └── SKILL.md
```

**Symlink 方向说明**：
```
方向1（同步 Sync）：agent → vibe
  ~/.claude/skills/my-skill → ~/.vibe-skills/claude-code/my-skill
  含义：把 agent 的 skill 收集到中心库

方向2（分发 Distribute）：vibe → agent
  ~/.claude/skills/my-skill ← ~/.vibe-skills/shared/my-skill
  含义：把中心库的 skill 推送到 agent

两个方向可并存，由用户按需操作。
```

---

## 二、当前同步逻辑的问题

### 问题 1：同步方向混乱

当前代码里存在**两个方向**的同步，但 UI 上没有区分清楚：

| 操作 | 方向 | 代码 | 含义 |
|------|------|------|------|
| Link | Vibe → Agent | `create_link()` | 把中心库的 Skill 链接到 Agent |
| Sync | Agent → Vibe | `sync_agent_to_vibe()` | 把 Agent 的 Skill 同步到中心库 |

用户困惑：「我到底是在把 Skill 推出去，还是在拉进来？」

**建议**：统一术语
- **同步（Sync）** = Agent → Vibe（把 Agent 的 Skill 收集到中心库）
- **分发（Distribute）** = Vibe → Agent（把中心库的 Skill 推送到 Agent）
- 当前 Symlink Tab 的「同步」按钮实际做的是 Sync（Agent → Vibe），但名字叫「同步」容易和分发混淆

---

### 问题 2：同步后无法清晰回退

当前 `remove_sync()` 和 `remove_sync_skills()` 可以删除同步的 symlink，但：

1. **没有「最近同步」的快速回退** — 用户同步了一批 Skill 后想撤回，需要到 History Tab 找对应的记录再 undo
2. **回退粒度不清晰** — 是按「次」回退，还是按「单个 Skill」回退？
3. **没有同步预览确认** — 同步前看不到会创建哪些 symlink

**建议**：
- 同步完成后显示「同步结果面板」，带一键「撤回本次同步」按钮
- 在 Symlink Tab 增加「同步批次」视图，按批次管理回退
- 同步前显示预览（当前已有 SyncPreview，但缺少确认步骤）

**实现成本**：低  
**代码改动**：
- `sync_agent_to_vibe()` 返回已创建的 symlink 列表
- 前端保存本次同步结果，提供「撤回」按钮
- 撤回调用 `remove_sync_skills(agent_id, skill_names)`

**同步快照文件** `.vibe-sync-state.json`（新增）：
```json
{
  "version": 1,
  "snapshots": [
    {
      "id": "sync-2026-07-03T10:00:00Z",
      "timestamp": "2026-07-03T10:00:00Z",
      "agent_id": "claude-code",
      "created_links": [
        {"path": "~/.vibe-skills/claude-code/my-skill", "target": "~/.claude/skills/my-skill"}
      ],
      "removed_links": []
    }
  ]
}
```
每次同步前自动创建快照，`sync_rollback(snapshot_id)` 读取快照执行回滚。

---

#### 2. 同步预览 + 确认

**现状**：点击「全部同步」直接执行，没有确认

**建议**：
- 点击「全部同步」→ 弹出预览面板：
  ```
  将同步以下 Skill 到 ~/.vibe-skills/claude-code/：
  ☑ coding-standard
  ☑ review-helper  
  ☐ debug-helper（已存在，跳过）
  
  [确认同步]  [取消]
  ```
- 用户可以选择性同步
- 已存在的 Skill 标记为「跳过」

**实现成本**：中  
**代码改动**：
- 新增 `preview_sync(agent_id)` 后端命令，返回待同步列表
- 前端增加确认弹窗，支持勾选

---

#### 3. Skill 来源标记

**现状**：Skill 列表不区分「独立 Skill」和「从 Agent 同步来的 Skill」

**建议**：
- Skill 卡片增加来源标签：
  - `📦 独立` — 直接放在 `~/.vibe-skills/` 根目录
  - `🔗 claude-code` — 从 Claude Code 同步来的 symlink
  - `🔗 hermes` — 从 Hermes 同步来的 symlink
- 筛选器增加「来源」筛选
- 同步来的 Skill 显示源路径

**实现成本**：低  
**代码改动**：
- `Skill` 模型增加 `source_type: "independent" | "synced"` 字段
- `SkillSource` 已有 `from` 字段，前端据此渲染标签

---

### 🟡 P1 - 体验优化

#### 4. Skill 更新功能（GitHub）

**需求**：从 GitHub 安装的 Skill 可以检查更新

**建议**：
- 安装时记录来源信息到 `.vibe-config.json`：
  ```json
  {
    "skill_sources": {
      "my-skill": {
        "type": "github",
        "repo": "user/repo",
        "path": "skills/my-skill",
        "installed_commit": "abc123",
        "installed_at": "2024-01-01T00:00:00Z"
      }
    }
  }
  ```
- Skill 卡片增加「检查更新」按钮
- 更新流程：`git fetch` → 比较 commit → 显示 diff → 确认更新
- 或者简单版：直接 `git pull` 对应目录

**Skill model 扩展**：
```rust
pub struct Skill {
    // ... 现有字段
    pub source_type: Option<String>,   // "git" | "npx" | "local"
    pub source_url: Option<String>,    // git repo URL 或 npm package name
    pub version: Option<String>,       // 当前版本
    pub latest_version: Option<String>,// 最新可用版本
    pub update_available: bool,
}
```

**SKILL.md frontmatter 扩展**（兼容现有标准）：
```yaml
---
name: my-skill
description: ...
source:
  type: git
  url: https://github.com/org/skill-repo
  branch: main
  version: "1.2.0"
---
```

**实现成本**：中  
**代码改动**：
- `SkillSource` 模型增加 `InstallMethod` 枚举（github / npx / local）
- 新增 `check_update(skill_id)` 后端命令
- 新增 `update_skill_from_git(skill_id, repo_url, branch)` 后端命令
- InstallDialog 增加 GitHub URL 输入

---

#### 5. Skill 更新功能（npx）

**需求**：通过 npx 安装的 Skill 可以更新

**建议**：
- 安装时记录 npx 包名：
  ```json
  {
    "skill_sources": {
      "my-skill": {
        "type": "npx",
        "package": "@scope/skill-package",
        "installed_version": "1.2.0"
      }
    }
  }
  ```
- 检查更新：`npm view @scope/skill-package version` → 比较版本号
- 更新：重新执行 npx 安装命令

**实现成本**：中  
**代码改动**：与 GitHub 类似，只是来源类型不同

---

#### 6. 批量同步 + 批量分发

**需求**：一次性把多个 Skill 同步到中心库，或从中心库分发到多个 Agent

**建议**：
- Symlink Tab 增加「批量模式」：
  - 勾选多个 Agent → 「同步选中 Agent 的所有 Skill」
  - 勾选多个 Skill → 「分发到选中 Agent」
- 操作前显示预览，确认后执行

**实现成本**：中  
**代码改动**：
- 前端增加批量选择 UI
- 后端已有 `batch_link` / `batch_unlink`，复用即可

---

#### 7. 同步状态指示器

**需求**：一眼看出哪些 Skill 是同步的、哪些是独立的、同步状态是否正常

**建议**：
- Skill 卡片左上角显示状态图标：
  - `🟢` — 同步正常
  - `🟡` — 源已更新，可同步新版本
  - `🔴` — 同步已断开（源文件不存在）
  - `⚪` — 独立 Skill
- Dashboard 增加「同步健康度」统计

**实现成本**：低  
**代码改动**：
- 后端增加 `check_sync_status()` 命令
- 前端根据状态渲染图标

---

### 🟢 P2 - 锦上添花

#### 8. 同步冲突处理

**需求**：如果 Agent A 和 Agent B 都有同名 Skill，同步时怎么处理？

**建议**：
- 同步时检测冲突：
  ```
  ⚠️ 冲突：coding-standard 在 claude-code 和 hermes 下都存在
  [保留两个副本]  [使用 claude-code 版本]  [使用 hermes 版本]  [跳过]
  ```
- 默认策略：保留两个副本，放在各自 Agent 子目录下

**实现成本**：中

---

#### 9. 自动同步（文件监听）

**需求**：Agent 目录下新增 Skill 时，自动同步到中心库

**建议**：
- 使用 `notify` crate 监听 Agent skills 目录
- 检测到新增 Skill → 自动创建 symlink 到 `~/.vibe-skills/{agent_id}/`
- 设置页增加「自动同步」开关

**实现成本**：高  
**备注**：v0.3 再考虑，先做手动同步

---

#### 10. Skill 市场（远程源）

**需求**：从社区仓库浏览和安装 Skill

**建议**：
- 支持配置远程 Skill 源：
  ```json
  {
    "remote_sources": [
      { "name": "Official", "url": "https://github.com/agent-skills/official" },
      { "name": "Community", "url": "https://github.com/agent-skills/community" }
    ]
  }
  ```
- InstallDialog 增加「浏览市场」Tab
- 展示 Skill 列表，一键安装

**实现成本**：高  
**备注**：v0.4 再考虑，依赖生态发展

---

## 四、架构分析与优化建议

### 现有代码问题审计

#### 1. 代码重复

| 重复项 | 位置 A | 位置 B | 影响 |
|--------|--------|--------|------|
| `copy_dir_all` | `commands/skills.rs:532` | `commands/config.rs:100` | 两份几乎相同的递归复制函数 |
| `days_to_ymd` | `commands/skills.rs:569` | `utils/history.rs:276` | 完全相同的日期转换逻辑 |
| `chrono_now` 手写 ISO 8601 | `utils/history.rs:255` | — | 项目间接依赖 `chrono` 却未直接使用，手写格式化易出错 |

**建议**：提取到 `utils/datetime.rs` 和 `utils/fs.rs` 公共模块。

#### 2. Config 读取不一致

`path.rs:8` 直接读 JSON 字符串做字段匹配获取 `vibe_skills_path`：
```rust
// path.rs — 手动 JSON 解析
if let Some(path) = config.get("vibe_skills_path").and_then(|v| v.as_str()) { ... }
```

而 `config.rs` 用 `serde_json::from_str` 反序列化为完整 `Config` 结构体。两套解析路径并行，容易数据不一致。

**建议**：`vibe_skills_dir()` 应调用 `load_config()` 而非手动解析 JSON。

#### 3. 每次 IPC 全量扫描磁盘

`list_skills()` 每次调用都遍历 `~/.vibe-skills/` + 所有 agent 目录 + 解析全部 SKILL.md。`load_config()` 在几乎每个 command 中被调用（skills.rs 3处、sync.rs 5处、agents.rs 4处），每次都从磁盘读 JSON + 反序列化，没有任何内存缓存。

**建议**：引入带 TTL 的内存缓存层（5秒过期），文件变化时主动失效。

#### 4. 错误被静默吞掉

多处 `let _ = record_action(...)` 丢弃了 history 记录失败的错误。`errors.rs` 和 `parsers/skill_md.rs` 有 `#[allow(dead_code)]` 标注，存在未使用的代码路径。

#### 5. 前端架构问题

| 问题 | 详情 |
|------|------|
| **组件状态不持久** | App.vue 用 `v-if` 切换 tab，每次切换销毁重建组件，丢失滚动位置。应改 `<KeepAlive>` |
| **Store 职责混乱** | `skills.ts` 管理列表、搜索、Dashboard、批量操作、选择状态（160+ 行）。`agents.ts` 混合 Agent CRUD 和同步操作 |
| **无事件驱动** | 前后端没有事件通道，后端文件变化无法推送到前端 |
| **双重状态存储** | 主题和语言在 `localStorage` 和后端 Config 中各存一份，没有同步机制 |

#### 6. Undo-Redo 设计缺陷

- `Delete` 操作无法撤销（`utils/history.rs:170` 直接返回错误），因为没有文件快照
- `BatchLink/BatchUnlink` 用逗号拼接 skill_id，若 skill_id 含逗号会解析错误
- 每次 undo/redo 重新 `load_config()` + `resolve_agent()`，多次磁盘读取

#### 7. 扩展性瓶颈

当前架构完全围绕 Skill 一种资产类型设计，没有抽象层。未来支持 Memory、Plugin、MCP 同步时需要：

- **资产类型抽象**：统一的 `Asset` trait 或 type 字段
- **同步引擎抽象**：通用同步管道，而非硬编码的 Skill 逻辑
- **事件总线**：前端监听后端变更事件（文件监控、同步完成等）
- **配置 Schema 版本化**：当前 `version: 1` 但无迁移逻辑

---

### 架构优化建议

#### 🔴 P0

##### 1. 统一同步模型

**现状**：Link 和 Sync 是两个独立的概念，代码逻辑分散

**建议**：抽象为统一的 `SyncOperation`：
```rust
pub enum SyncDirection {
    AgentToVibe,   // 同步：Agent → Vibe
    VibeToAgent,   // 分发：Vibe → Agent
}

pub struct SyncOperation {
    direction: SyncDirection,
    source: String,      // agent_id 或 "vibe-lib"
    target: String,      // agent_id 或 "vibe-lib"
    skill_ids: Vec<String>,
    created_links: Vec<String>,  // 用于回退
}
```

所有同步操作都通过 `SyncManager` 执行，统一记录历史和回退逻辑。

**实现成本**：中  
**价值**：高（消除方向混乱，简化回退逻辑）

---

#### 2. 同步批次管理

**现状**：每次同步操作独立记录，无法按批次回退

**建议**：
```rust
pub struct SyncBatch {
    id: String,
    timestamp: String,
    operations: Vec<SyncOperation>,
    status: SyncBatchStatus,  // Completed / RolledBack / Partial
}
```

History 增加 `batch_id` 字段，支持按批次 undo/redo。

**实现成本**：中  
**价值**：高（支持「撤回本次同步」）

---

#### 3. Skill 来源追踪

**现状**：`SkillSource` 只记录 `from` 和 `path`，不记录安装方式

**建议**：
```rust
pub struct SkillSource {
    pub from: String,           // "vibe-lib" | agent_id
    pub path: String,
    pub install_method: Option<InstallMethod>,
}

pub enum InstallMethod {
    Local { source_path: String },
    GitHub { repo: String, path: String, commit: String },
    Npx { package: String, version: String },
}
```

**实现成本**：低  
**价值**：中（支持更新功能）

---

### 🟡 P1

#### 4. 状态管理解耦

**现状**：`skillsStore.createLink()` 直接调用 `agentsStore.fetchAgents()`

**建议**：引入事件总线
```typescript
// event-bus.ts
export const events = {
  'skill:linked': (skillId: string, agentId: string) => void,
  'skill:unlinked': (skillId: string, agentId: string) => void,
  'sync:completed': (batch: SyncBatch) => void,
}

// skills store
async function createLink(skillId, agentId) {
  await invoke("create_link", { skillId, agentId });
  events.emit('skill:linked', skillId, agentId);
}

// agents store
events.on('skill:linked', () => fetchAgents());
```

**实现成本**：低  
**价值**：中（降低耦合，便于扩展）

---

#### 5. 文件系统缓存

**现状**：每次 `list_skills()` 都全量扫描文件系统

**建议**：
- 引入 `FileCache`，缓存扫描结果 5 秒
- 增加 `watch` 机制，文件变化时主动失效缓存
- 减少重复 IO，提升切换 Tab 的速度

**实现成本**：中  
**价值**：中（性能优化）

---

#### 6. 资产类型抽象层（为 v1.0 扩展做准备）

**现状**：所有代码硬编码围绕 Skill 设计，无法复用于 Memory/Plugin/MCP

**建议**：
```rust
/// 统一资产类型枚举
pub enum AssetType {
    Skill,
    Memory,
    Plugin,
    Mcp,
}

/// 统一资产 trait（v1.0+ 实现）
pub trait Asset {
    fn asset_type() -> AssetType;
    fn scan(dir: &Path) -> Vec<Self>;
    fn validate(dir: &Path) -> bool;
}
```

当前先在 Skill 上实现，v1.0 时扩展其他类型。同步引擎、快照机制可复用。

**实现成本**：低（当前只定义枚举和 trait，不改现有逻辑）  
**价值**：高（为 Memory/Plugin/MCP 同步铺路）

---

#### 7. 公共工具函数提取

**现状**：`copy_dir_all`、`days_to_ymd`、`chrono_now` 重复实现

**建议**：
- `utils/datetime.rs` — 统一时间格式化（使用 `chrono` crate）
- `utils/fs.rs` — 统一文件操作（`copy_dir_all`、`remove_dir_recursive`）
- `utils/config.rs` 中 `vibe_skills_dir()` 改为调用 `load_config()` 而非手动解析 JSON

**实现成本**：低  
**价值**：中（消除重复，降低维护成本）

---

## 五、优先级排序

| 优先级 | 功能 | 类型 | 成本 | 价值 | 建议版本 |
|--------|------|------|------|------|----------|
| **P0** | 同步回退机制（快照 + 撤回） | 体验优化 | 低 | 高 | v0.2 |
| **P0** | 同步预览 + 确认 | 体验优化 | 中 | 高 | v0.2 |
| **P0** | Skill 来源标记 | 体验优化 | 低 | 高 | v0.2 |
| **P0** | 统一同步模型 | 架构优化 | 中 | 高 | v0.2 |
| **P0** | 同步批次管理 | 架构优化 | 中 | 高 | v0.2 |
| **P0** | 公共工具函数提取（消除重复） | 架构优化 | 低 | 中 | v0.2 |
| **P0** | Config 读取路径统一 | 架构优化 | 低 | 中 | v0.2 |
| **P1** | Skill 更新（GitHub） | 新功能 | 中 | 高 | v0.2 |
| **P1** | Skill 更新（npx） | 新功能 | 中 | 中 | v0.2 |
| **P1** | 批量同步 + 分发 | 体验优化 | 中 | 中 | v0.2 |
| **P1** | 同步状态指示器 | 体验优化 | 低 | 中 | v0.2 |
| **P1** | Skill 来源追踪（InstallMethod） | 架构优化 | 低 | 中 | v0.2 |
| **P1** | 状态管理解耦（事件总线） | 架构优化 | 低 | 中 | v0.2 |
| **P1** | 文件系统缓存 | 架构优化 | 中 | 中 | v0.2 |
| **P2** | 同步冲突处理 | 新功能 | 中 | 中 | v0.3 |
| **P2** | 资产类型抽象层 | 架构优化 | 低 | 高 | v0.3 |
| **P2** | 自动同步（文件监听） | 新功能 | 高 | 中 | v0.3 |
| **P3** | Skill 市场（远程源） | 新功能 | 高 | 中 | v0.4 |

---

## 六、v0.2 路线图建议

### Phase 1：同步体验优化（2 周）
1. 统一术语：Sync（Agent → Vibe）/ Distribute（Vibe → Agent）
2. 同步预览 + 确认弹窗
3. 同步完成后显示「撤回」按钮
4. Skill 卡片增加来源标签

### Phase 2：Skill 更新功能（2 周）
1. `SkillSource` 模型增加 `install_method`
2. InstallDialog 支持 GitHub URL 输入
3. 安装时记录 commit / version
4. Skill 卡片增加「检查更新」按钮
5. 更新流程：fetch → diff → confirm → pull

### Phase 3：架构优化（1 周）
1. 统一 `SyncOperation` 模型
2. 同步批次管理（`SyncBatch`）
3. 状态管理解耦（事件总线）

---

## 七、总结

### 核心改进方向

1. **同步逻辑清晰化** — 区分 Sync（收集）和 Distribute（分发），统一术语
2. **同步可回退** — 同步后提供快捷撤回，降低用户心理负担
3. **来源可追踪** — 标记 Skill 来源（独立 / 同步 / GitHub / npx），支持更新
4. **更新机制** — 支持从 GitHub / npx 安装的 Skill 检查更新

### 不需要做的

- ❌ Skill 编辑器 — 用户不需要在应用内编辑 SKILL.md
- ❌ 重型 Dashboard — 统计信息合并到 Skill 列表顶部即可
- ❌ History 独立 Tab — 降级为全局撤销/重做按钮

### 产品边界

保持「轻量 Skill 管理工具」定位，逐步扩展资产类型：
- ✅ 同步管理（Agent ↔ Vibe）
- ✅ 分发管理（Vibe → Agent）
- ✅ 更新管理（GitHub / npx → Vibe）
- ✅ 同步回滚（快照 + 一键撤回）
- ❌ 不做 Skill 编辑
- 📋 v1.0+ 规划：Memory 同步、Plugin 同步、MCP 同步（需先建立资产类型抽象层）
