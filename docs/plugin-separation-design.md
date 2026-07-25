# Plugin Skills 与 Regular Skills 分离设计

## 问题分析

### 当前架构的问题

1. **合并逻辑导致混乱**：Plugin skills 和 regular skills 按文件夹名合并为同一个 `Skill` 条目，导致：
   - 同名 skill 出现多个来源（plugin + local），标记为 `is_duplicate`
   - 用户不确定哪个是"真实"版本
   - Plugin 更新可能覆盖本地修改

2. **Domain 分区逻辑复杂**：`matchesDomain()` 有特殊规则——同时有 plugin 和 local 来源的 skill 归入 `local` 域，不符合直觉

3. **Sync to Library 风险**：将 plugin skill 复制到 `~/.vibe-skills/` 后，plugin 更新会造成数据不一致

4. **命名冲突处理不佳**：同名 skill 的冲突检测只是标记警告，没有提供清晰的解决路径

### 用户场景分析

| 用户类型 | 典型行为 | 当前痛点 |
|---------|---------|---------|
| Plugin 用户 | 使用 agent 官方 plugin，不关心中心库管理 | Plugin skill 混入列表，被误操作 |
| 混合用户 | 同时使用 plugin 和自定义 skill | 同名 skill 冲突，不确定哪个生效 |
| 纯本地用户 | 只使用中心库 skill | 看到 plugin skill 感到困惑 |

## 设计目标

1. **完全分离**：Plugin skills 和 regular skills 在数据层和 UI 层完全分开
2. **清晰归属**：每个 skill 明确属于一个管理域（plugin-managed 或 library-managed）
3. **安全隔离**：Plugin 更新不影响中心库数据
4. **保留功能**：不破坏现有 skill 管理、链接、批量操作等功能
5. **可选集成**：允许用户将 plugin skill "认领"到中心库（带警告）

## 架构设计

### 数据层分离

```
┌─────────────────────────────────────────────────────────┐
│                    Skill Sources                         │
├─────────────────────┬───────────────────────────────────┤
│   Library Sources   │      Plugin Sources               │
│  (vibe-lib, agent)  │  (claude-plugin, codex-plugin)    │
└─────────────────────┴───────────────────────────────────┘
         │                        │
         ▼                        ▼
┌─────────────────────┐  ┌─────────────────────────────┐
│   Regular Skills    │  │     Plugin Skills            │
│  (library-managed)  │  │   (plugin-managed, read-only)│
└─────────────────────┘  └─────────────────────────────┘
```

### 后端改动

#### 1. 修改 `list_skills` 合并逻辑

**当前逻辑**（简化）：
```rust
// 所有来源合并到同一个 HashMap
map.entry(id).and_modify(|e| e.sources.push(new_source));
```

**新逻辑**：
```rust
// Plugin skills 单独收集，不与 regular skills 合并
if is_plugin_source(&new_source) {
    plugin_map.entry(id).or_default().sources.push(new_source);
} else {
    regular_map.entry(id).or_default().sources.push(new_source);
}
```

#### 2. 修改 Skill 模型

```rust
pub struct Skill {
    // ... 现有字段 ...
    pub from_plugin: bool,           // 保留：标记是否来自 plugin
    pub plugin_source: Option<String>, // 保留：plugin 来源名
    pub plugin_managed: bool,        // 新增：是否完全由 plugin 管理（不可编辑）
}
```

#### 3. 新增 API 端点

```rust
#[tauri::command]
pub async fn list_plugin_skills() -> Result<Vec<Skill>, VibeError> {
    // 返回所有 plugin-managed skills（独立列表）
}

#[tauri::command] 
pub async fn adopt_plugin_skill(skill_id: String) -> Result<(), VibeError> {
    // 将 plugin skill 复制到中心库，并标记为 library-managed
    // 带警告：后续 plugin 更新不会同步到此副本
}
```

### 前端改动

#### 1. 新增 PluginSkillsView 组件

独立的 plugin skills 视图，与 SkillsView 并列：

```
┌─────────────────────────────────────────────────────────┐
│  Plugin Skills                          [刷新] [全部认领] │
├─────────────────────────────────────────────────────────┤
│  ┌─ Plugin: claude-plugins ──────────────────────────┐  │
│  │  □ skill-a  [Plugin]  [查看详情]                   │  │
│  │  □ skill-b  [Plugin]  [查看详情]                   │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌─ Plugin: codex-plugins ───────────────────────────┐  │
│  │  □ skill-c  [Plugin]  [查看详情]                   │  │
│  └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

#### 2. 修改侧边栏导航

```
技能库
├── 全部 (regular skills only)
├── 待处理
├── 已链接
├── 未链接
├── ─────────────
├── Plugin Skills (独立入口，带 badge)
├── ─────────────
└── 来源
    └── 插件技能 (保留，但只显示纯 plugin skills)
```

#### 3. 修改 SkillRow 组件

Plugin-managed skills 显示为：
- 紫色边框 + Puzzle 图标
- "Plugin-managed" 标签（不可编辑提示）
- "认领到中心库" 按钮（带确认对话框）
- 不显示"同步到库"按钮（避免混淆）

#### 4. 修改 SkillsView

- 移除 `isPluginsView` 的特殊渲染逻辑
- Plugin skills 不再出现在主列表中
- 简化 `matchesDomain()` 逻辑

### 交互流程

#### 流程 1：查看 Plugin Skills

```
用户点击侧边栏 "Plugin Skills"
    ↓
显示 PluginSkillsView
    ↓
按 plugin_source 分组显示
    ↓
每个 skill 显示：
  - 名称、描述
  - Plugin 来源
  - Agent 状态（只读）
  - "认领" 按钮（可选）
```

#### 流程 2：认领 Plugin Skill

```
用户点击 "认领到中心库"
    ↓
显示确认对话框：
  "将 plugin skill 'xxx' 复制到中心库。
   注意：后续 plugin 更新不会自动同步到此副本。
   您需要手动管理此 skill 的更新。"
    ↓
用户确认
    ↓
调用 adopt_plugin_skill API
    ↓
Skill 出现在 regular skills 列表中
    ↓
Plugin skills 列表中该项显示为 "已认领"
```

#### 流程 3：处理命名冲突

当前：同名 skill 合并，标记 `is_duplicate`

新方案：
- Plugin skills 和 regular skills 完全分开，不会合并
- 如果用户认领了一个与现有 skill 同名的 plugin skill：
  - 显示冲突对话框
  - 选项：重命名、覆盖、取消
  - 选择覆盖时，备份现有 skill 到 `.trash/`

## 迁移策略

### 阶段 1：数据层分离（后端）
1. 修改 `list_skills` 逻辑，分离 plugin 和 regular skills
2. 新增 `list_plugin_skills` API
3. 新增 `adopt_plugin_skill` API
4. 修改 `detect_issues` 逻辑（移除 plugin+local 冲突检测）

### 阶段 2：UI 层分离（前端）
1. 新增 `PluginSkillsView.vue` 组件
2. 修改侧边栏导航
3. 修改 `SkillRow.vue` 支持 plugin-managed 模式
4. 简化 `SkillsView.vue`（移除 plugin 特殊逻辑）

### 阶段 3：交互优化
1. 添加认领确认对话框
2. 添加命名冲突处理
3. 优化 plugin skill 的只读提示

## 影响评估（完整清单）

### 后端影响

| 文件 | 函数/逻辑 | 影响说明 |
|------|----------|---------|
| `commands/skills.rs` | `list_skills()` | 合并逻辑需改为分离收集 |
| `commands/skills.rs` | `from_plugin` 检测 | 保留，但不再用于合并 |
| `commands/skills.rs` | `is_duplicate` 检测 | 移除 plugin+local 条件 |
| `commands/skills.rs` | `non_plugin_sources` 过滤 | 简化：regular skills 不含 plugin 来源 |
| `commands/skills.rs` | `scan_plugin_marketplace_skills()` | 保留，但结果单独返回 |
| `commands/skills.rs` | 排序逻辑 | Plugin skills 不再参与主列表排序 |
| `commands/sync.rs` | `sync_from_plugin_impl()` | 保留：支持认领功能 |
| `commands/sync.rs` | `batch_skill_action` | Plugin skills 不参与批量链接/取消链接 |
| `utils/origin.rs` | `SOURCE_METHOD_MARKETPLACE` | 保留不变 |
| `models/skill.rs` | `Skill` 结构体 | 新增 `plugin_managed` 字段 |

### 前端影响

| 文件 | 函数/组件 | 影响说明 |
|------|----------|---------|
| `types/index.ts` | `Skill` 接口 | 新增 `plugin_managed` 字段 |
| `manageFilters.ts` | `DomainScope` 类型 | 简化：plugin 域独立 |
| `manageFilters.ts` | `matchesDomain()` | 简化：不再有混合来源情况 |
| `manageFilters.ts` | `hasNonPluginSource()` | 简化：regular skills 不含 plugin |
| `manageFilters.ts` | `classifySkillSources()` | 移除 `hasMarketplace` 逻辑 |
| `manageFilters.ts` | `matchesStatusPreset()` | 简化：移除 plugin 特殊处理 |
| `manageFilters.ts` | `matchesIssues()` | 移除 `is_duplicate` 条件 |
| `manageFilters.ts` | `matchesLibraryScope()` | 简化：`library_only` 不含 plugin |
| `useSmartViews.ts` | `SMART_VIEWS` | Plugin 视图独立 |
| `useSmartViews.ts` | `viewToFilterPreset()` | Plugin 视图预设简化 |
| `useSmartViews.ts` | `viewCounts` | Plugin 计数独立 |
| `skillActionRegistry.ts` | `sync_from_plugin` | 保留：支持认领功能 |
| `useSkillAgentStatus.ts` | `sourceBelongsToAgent()` | 简化：plugin skills 独立 |
| `useSkillAgentStatus.ts` | Plugin status detection | 保留：显示 plugin 来源 |
| `useBatchActions.ts` | Plugin batch group | 移除：plugin skills 不参与批量 |
| `SkillsView.vue` | `isPluginsView` | 移除：plugin 有独立视图 |
| `SkillsView.vue` | `pluginGroups` | 移除：plugin 有独立视图 |
| `SkillsView.vue` | `handleSyncPlugin()` | 移除：改用认领功能 |
| `SkillsView.vue` | Plugin 渲染逻辑 | 移除：plugin 有独立视图 |
| `SkillRow.vue` | `skill-row-plugin` 样式 | 保留：plugin skills 仍显示紫色 |
| `SkillRow.vue` | Sync to Library 按钮 | 改为"认领"按钮 |
| `SkillDetail.vue` | `isPluginSource()` | 保留：显示 plugin 来源信息 |
| `SkillCard.vue` | `hasLibrarySource` | 保留：判断是否可删除 |

### 筛选逻辑影响

| 筛选维度 | 当前行为 | 新行为 |
|---------|---------|--------|
| Domain | Plugin skills 在 `plugin` 域 | Plugin skills 独立视图，不参与筛选 |
| Status Preset | Plugin skills 有特殊处理 | 简化：只处理 regular skills |
| Issues | `is_duplicate` 包含 plugin+local | 移除：只检测 regular skill 冲突 |
| Library Scope | `library_only` 排除 plugin | 简化：regular skills 不含 plugin |
| Agent Scope | Plugin 映射到 agent | 保留：显示 plugin 来源 agent |

### 设置页面影响

| 设置项 | 影响说明 |
|--------|---------|
| Skills 路径设置 | 无影响：只管理 `~/.vibe-skills/` |
| Agent 管理 | 无影响：agent 检测不变 |
| 主题设置 | 无影响 |
| 语言设置 | 需新增 plugin 相关 i18n key |

### 批量操作影响

| 操作类型 | 当前行为 | 新行为 |
|---------|---------|--------|
| 批量链接 | Plugin skills 可参与 | 移除：只处理 regular skills |
| 批量取消链接 | Plugin skills 可参与 | 移除：只处理 regular skills |
| 批量同步到库 | Plugin skills 可参与 | 改为"批量认领" |
| 批量删除 | Plugin skills 不可删除 | 保持：只删除 regular skills |

### 搜索影响

| 搜索范围 | 当前行为 | 新行为 |
|---------|---------|--------|
| 全局搜索 | 包含 plugin skills | 分离：regular 和 plugin 独立搜索 |
| Plugin 视图搜索 | 只搜索 plugin skills | 保留 |

### 历史/撤销影响

| 操作 | 影响说明 |
|------|---------|
| 认领 plugin skill | 记录历史，可撤销 |
| 删除 regular skill | 无影响 |
| 链接/取消链接 | 无影响：只处理 regular skills |

### 新增功能
- Plugin skills 独立视图
- Plugin skill 认领功能（带确认对话框）
- 命名冲突解决对话框（重命名/覆盖/取消）

## 测试要点

1. **数据分离测试**：
   - Plugin skills 不出现在 regular skills 列表中
   - Regular skills 不出现在 plugin skills 列表中
   - 同名 skill 在两个列表中独立存在

2. **认领功能测试**：
   - 认领后 skill 出现在 regular 列表
   - 认领后 plugin 列表显示"已认领"
   - 认领同名 skill 时的冲突处理

3. **兼容性测试**：
   - 现有 regular skill 操作不受影响
   - Agent 链接状态正确显示
   - 批量操作只作用于 regular skills

4. **边界情况**：
   - Plugin skill 认领后，原 plugin 被删除
   - 同时认领多个同名 plugin skills
   - 认领后执行撤销操作
