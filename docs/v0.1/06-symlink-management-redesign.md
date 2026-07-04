# 软连接管理功能重构方案

> 版本：v0.1  
> 日期：2026-07-04  
> 状态：设计阶段

---

## 一、项目核心定位

QS-Vibe 的核心功能是**管理 AI coding agent 的 skill 通过软连接实现共享**。

用户设置一个中心目录（`~/.vibe-skills/`），在其中管理 skill，然后通过 symlink 将 skill 分发到各个 agent 的 skills 目录。

**不做 npm 搜索下载**，那是后续版本的功能。当前只做**删除和软连接**。

---

## 二、当前问题分析

### 2.1 后端数据模型问题

#### 同名 skill 被错误合并

`list_skills` 以文件夹名为 key 去重（`skills.rs:505-508`）：

```rust
let id = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
```

当多个 agent 有同名目录时，后续扫描到的只追加 `sources`，name/description 等元数据被覆盖丢失。

**后果**：Agent A 有 `foo`（v1），Agent B 有 `foo`（v2），用户看到的是一个合并后的 skill，版本差异完全不可见。

#### `sources` 和 `linked_agents` 语义不一致

| 字段 | 含义 | 判定方式 |
|------|------|---------|
| `sources` | skill 存在于哪些位置 | 目录下有 SKILL.md 就算 |
| `linked_agents` | 哪些 agent 通过 symlink 链接到 vibe 库 | 必须是 symlink 且目标恰好是 `~/.vibe-skills/{id}` |

- `linked_agents` 只识别指向 vibe 库的 symlink，Agent 间的直接 symlink 不追踪
- `SkillCard` 从未使用 `linked_agents` 字段，只用 `sources` 渲染 tags
- 用户无法区分"agent 自带的真实文件"和"从 vibe 库链接过去的 symlink"

#### sync 方向嵌套结构产生 id 冲突

`sync_agent_to_vibe` 创建 `~/.vibe-skills/{agent_id}/{skill}/` 路径下的 symlink，但 `list_skills` 的递归扫描会把这个 skill 的 id 也解析为文件夹名，与顶层 `~/.vibe-skills/foo/` 的 id 相同，导致两个不同的 skill 被错误合并。

### 2.2 前端展示问题

#### AgentExpandable 信息不足

左侧 Agent 卡片只显示名称和路径，不显示 skill 数量、已链接数、冲突数。用户必须逐个点击才能了解每个 agent 的状态。

#### SyncPreview 方向不明确

只展示 agent → vibe 方向的同步状态，但应用支持两种方向（sync 和 link），前端没有标注也没有选择能力。

#### 同名 skill 无冲突警告

当两个 Agent 各自有独立的同名 skill（内容不同），SyncPreview 和 Dashboard 都不提供冲突检测或警告。

#### symlink 断裂不可见

如果 vibe 库中的原始 skill 被删除，agent 目录中的 symlink 变成 dangling link，前端不会告警。

#### 完整路径信息被截断

`SyncPreview` 只显示 `link_target` 的最后一段，用户无法快速判断链接指向哪里。

#### 功能分散在多个 Tab

| 信息 | AgentsTab | SkillsTab | DashboardTab | SymlinkTab |
|------|-----------|-----------|--------------|------------|
| Agent 有哪些 skill | agent tags | agent 过滤 | AgentColumn | 树视图 |
| Skill 属于哪些 agent | - | SkillCard tags | 矩阵 | - |
| symlink 状态 | - | 未使用 | - | synced 标记 |

三个 tab 在"展示 agent-skill 关系"这件事上有大量冗余。

### 2.3 用户真实场景与现有模型的差距

| 环节 | 现状 | 缺失 |
|------|------|------|
| 发现重复 | 同名 skill 被合并，差异丢失 | 需按内容区分，而非仅文件夹名 |
| 理解关系 | sources 只记录"存在"，不记录"来源类型" | 需区分：真实文件 / symlink 到 vibe / symlink 到其他 agent |
| 判断迁移 | `linked_agents` 只识别指向 vibe 的 symlink | 需要完整的关系图 |
| 做决策 | 无冲突检测，无差异对比 | 需要同名 skill 的内容 diff、迁移影响预览 |

---

## 三、界面重构方案

### 3.1 设计原则

1. **以 skill 为主体**，不是以 agent 目录结构为中心
2. **每个按钮都有明确用途**，不放不必要的功能
3. **操作符合习惯**：先看再操作，操作完有反馈
4. **异常优先**：冲突和断链置顶显示

### 3.2 Tab 合并方案

合并前（5 tabs）：

```
agents → skills → dashboard → symlink → history
```

合并后（3 tabs）：

```
overview → manage → history
```

- **overview**：合并 agents + dashboard + symlink 的全局部分（Agent 概览 + 关系矩阵 + 冲突列表）
- **manage**：合并 symlink + skills 的操作部分（Skill 列表 + 链接操作）
- **history**：保持不变

### 3.3 软连接管理界面（manage tab）详细设计

#### 布局

整个页面就是一个列表，以 skill 为主体。没有左侧栏，没有顶部统计条，没有多层筛选。

顶部只有一个下拉筛选和搜索框：

```
状态: [全部 ▾]    搜索: [________]
```

状态选项：全部 / 有冲突 / 有断链 / 未链接 / 已链接

#### Skill 列表项

每个 skill 是一行，左侧有展开箭头。折叠状态只显示摘要：

```
▸ foo    Claude Code, Hermes, Codex    3 sources
```

异常状态增加标记：

```
⚠ bar    Claude Code, Hermes    2 sources    ⚠ 冲突
❌ baz    Claude Code    1 source    ❌ 断链
```

排序规则：冲突和断链置顶，其余按字母排序。

#### 展开状态

展开后显示每个 agent 的具体条目和操作按钮。

**正常共享示例**：

```
▸ foo    Claude Code, Hermes, Codex    3 sources
  ├── Claude Code    ● 真实文件
  ├── Hermes         🔗 symlink → ~/.vibe-skills/foo
  └── Codex          ● 真实文件
  
  [Link to Agent ▾]  [Remove Link ▾]
```

**冲突示例**：

```
⚠ bar    Claude Code, Hermes    2 sources    ⚠ 冲突
  ├── Claude Code    ● 真实文件    name: "Bar Tool"    desc: "A tool for bars"
  └── Hermes         ● 真实文件    name: "Bar Helper"  desc: "Helper for bar ops"
  
  两个 agent 有同名 skill 但内容不同
  [Use Claude Code version]  [Use Hermes version]
```

**断链示例**：

```
❌ baz    Claude Code    1 source    ❌ 断链
  └── Claude Code    ❌ symlink (broken) → ~/.vibe-skills/baz
    
  链接目标已删除
  [Remove broken link]
```

#### 操作按钮规则

每个 skill 在同一时间只显示一种操作按钮组：

| 状态 | 显示的按钮 |
|------|-----------|
| 正常（部分 agent 未链接） | `Link to Agent ▾` |
| 已链接 | `Remove Link ▾` |
| 冲突 | `Use [Agent A] version` / `Use [Agent B] version` |
| 断链 | `Remove broken link` |
| 不存在于 vibe 库 | `Sync to vibe库` |

#### 安装 skill

右上角一个 `+ 安装` 按钮，弹出文件选择对话框，选择包含 SKILL.md 的目录，安装到 vibe 库。

### 3.4 总览界面（overview tab）详细设计

#### Agent 概览卡片

纵向排列的 Agent 卡片，每个卡片显示：

- Agent 名称
- 检测状态（绿点/灰点）
- 三个数字：总 skill 数 / 已链接数 / 冲突数

点击后过滤 manage tab 的列表为仅该 agent 相关的 skill。

#### 关系矩阵

从 DashboardTab 的 RelationGraph 迁移，行是 skill 名称，列是 agent 名称，用圆点标注哪些 agent 拥有该 skill。增加冲突列标记。

#### 冲突/异常列表

显示所有检测到的冲突和断链，每条可以点击跳转到 manage tab 的对应 skill。

### 3.5 界面与操作习惯的对齐

| 操作习惯 | 设计对齐方式 |
|---------|------------|
| 先看再操作 | 折叠/展开模式，先看摘要再看详情 |
| 从上往下浏览 | 冲突/断链置顶，其余按字母排序 |
| 操作完想确认结果 | 每次操作后列表自动刷新 |
| 不想误操作 | 删除/移除操作有确认对话框 |
| 快速找某个 skill | 顶部搜索框，实时过滤 |
| 不理解 sync/link 区别 | 统一为"建立链接"和"移除链接"，方向由系统自动判断 |

### 3.6 砍掉的功能及理由

| 功能 | 理由 |
|------|------|
| Agent 侧边栏 | 这是 overview tab 的功能，manage tab 不需要 |
| 顶部统计数字 | 占空间，筛选标签已隐含数量信息 |
| tree 视图 | 目录树是文件系统视角，用户不需要 |
| 多层筛选标签 | 一个下拉就够了，标签太多分散注意力 |
| "查看差异"独立按钮 | 冲突状态已直接展示 name/description 差异 |
| sync/link 方向选择 | 用户不关心方向，系统根据当前状态自动决定 |
| 批量操作 | 低频需求，后续根据反馈再加 |

---

## 四、数据模型重构

### 4.1 SkillSource 增加字段

当前结构（`models/skill.rs`）：

```rust
pub struct SkillSource {
    pub from: String,
    pub path: String,
}
```

重构后：

```rust
pub struct SkillSource {
    pub from: String,              // "vibe-lib" 或 agent id
    pub path: String,              // 绝对路径
    pub name: String,              // 该来源下 SKILL.md 的实际 name
    pub description: String,       // 该来源下的实际 description
    pub is_symlink: bool,          // 是否为 symlink
    pub symlink_target: Option<String>, // symlink 目标路径（如有）
    pub content_hash: String,      // SKILL.md 内容 hash（用于冲突检测）
}
```

### 4.2 新增冲突和断链模型

```rust
pub enum ConflictType {
    SameNameDiffContent,    // 同名但 SKILL.md 内容不同
    DanglingLink,           // symlink 指向已删除路径
}

pub struct SkillIssue {
    pub skill_id: String,
    pub issue_type: ConflictType,
    pub sources: Vec<SkillSource>,
    pub description: String,
}
```

### 4.3 Skill 模型不变

```rust
pub struct Skill {
    pub id: String,              // 文件夹名
    pub name: String,            // SKILL.md 中的 name
    pub description: String,
    pub path: String,
    pub linked_agents: Vec<String>,
    pub sources: Vec<SkillSource>, // 每个 source 携带独立元数据
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub has_scripts: bool,
    pub has_references: bool,
    pub has_assets: bool,
    pub modified_at: String,
}
```

---

## 五、后端命令变更

### 5.1 修改现有命令

| 命令 | 变更内容 |
|------|---------|
| `list_skills` | 每个 source 携带独立元数据（name、description、is_symlink、symlink_target、content_hash） |
| `get_skills_tree` | 增加跨 agent 信息，返回每个 skill 在其他 agent 中的存在情况 |

### 5.2 新增命令

| 命令 | 用途 | 返回类型 |
|------|------|---------|
| `detect_issues` | 检测冲突和断链 | `Vec<SkillIssue>` |
| `preview_sync` | 同步前预览 | `SyncPreviewResult` |
| `diff_skill` | 对比两个 agent 中同名 skill 的差异 | `SkillDiff` |

### 5.3 删除的命令

| 命令 | 理由 |
|------|------|
| `check_link_status` | 功能被 `list_skills` 的 source 级别信息覆盖 |
| `check_updates` | 只比较修改时间，无版本号机制，功能价值有限 |

---

## 六、前端组件变更

### 6.1 删除的组件

| 组件 | 理由 |
|------|------|
| `AgentExpandable` | 合并到 overview tab 的 Agent 摘要卡片 |
| `SyncPreview` | 重构为 manage tab 的 skill 列表 |
| `AgentColumn` | 合并到 overview tab 的关系矩阵 |
| `RelationGraph` | 迁移到 overview tab |

### 6.2 重构的组件

| 组件 | 重构方向 |
|------|---------|
| `SkillCard` | 从展示 agent tags 改为展示每个来源的独立状态 |
| `SkillList` | 从按 agent 过滤改为按状态过滤（冲突/断链/未链接） |
| `DashboardTab` | 重构为 overview tab，合并 Agent 概览和关系矩阵 |
| `BatchActionBar` | 删除，后续根据反馈再加 |

### 6.3 新增的组件

| 组件 | 用途 |
|------|------|
| `SkillRow` | 单个 skill 的折叠/展开行，显示摘要和详情 |
| `AgentStatusBadge` | 每个 agent 的状态标记（真实文件/symlink/断链） |
| `ConflictWarning` | 冲突警告区域，显示差异和选择按钮 |
| `DanglingWarning` | 断链警告区域，显示原始目标和清理按钮 |

---

## 七、实施计划

### Phase 1：数据模型（后端）

- 修改 `SkillSource` 结构，增加独立元数据字段
- 修改 `scan_directory` 函数，为每个 source 填充完整信息
- 修改 `list_skills` 命令，返回增强后的数据

### Phase 2：冲突检测（后端）

- 新增 `detect_issues` 命令
- 实现 SKILL.md 内容 hash 计算
- 实现同名 skill 冲突检测
- 实现断链检测

### Phase 3：界面合并（前端）

- 合并为 3 个 tab（overview / manage / history）
- 实现 `SkillRow` 组件（折叠/展开）
- 实现 `AgentStatusBadge` 组件
- 实现 `ConflictWarning` 和 `DanglingWarning` 组件
- 重构 `SkillList` 为 manage tab

### Phase 4：体验优化

- 搜索和筛选功能
- 安装 skill 对话框
- 确认对话框
- 操作后自动刷新

---

## 八、验收标准

1. 用户打开 manage tab，能看到所有 skill 的列表
2. 冲突和断链的 skill 有明显标记，置顶显示
3. 展开 skill 后能看到每个 agent 的具体状态（真实文件/symlink/断链）
4. 操作按钮根据状态动态变化，一次只显示一种操作
5. 操作后列表自动刷新，状态即时更新
6. 搜索能实时过滤 skill 列表
7. 同名但内容不同的 skill 能被正确检测和标记
8. 断裂的 symlink 能被正确检测和标记
