# QS-Vibe 产品分析报告

> **版本**: v0.1 | **日期**: 2026-07-03  
> **视角**: 产品经理 | **定位**: 轻量化多 Agent Skill 管理工具

---

## 1. 产品现状

### 1.1 核心链路

```
安装 Skill → 统一库 ~/.vibe-skills/ → symlink 分发到各 Agent 目录
```

### 1.2 功能模块（5 个 Tab）

| Tab | 功能 | 完成度 |
|-----|------|--------|
| CLI | Agent 管理（自动检测 + 自定义添加） | 高 |
| Skills | Skill 列表、搜索、安装、预览、删除 | 高 |
| Dashboard | 总览统计、Agent-Skill 分布矩阵 | 中 |
| Symlink | 树形浏览、批量同步、分类同步 | 高 |
| History | 操作历史、undo/redo、搜索过滤 | 高 |

### 1.3 技术亮点

- Tauri 2 + Vue 3 + Tailwind 4，技术栈现代且轻量
- 三语 i18n（中/英/繁）
- 亮色/暗色主题
- undo/redo 历史系统
- 递归扫描 + 去重合并
- 跨平台 symlink/junction/copy 降级策略

---

## 2. 信息架构 🔴 高优先级

### 2.1 当前 Tab 结构

```
CLI (🔧) → Skills (📁) → Dashboard (📊) → Symlink (🔗) → History (🕐)
```

### 2.2 问题分析

**问题 1: CLI Tab 命名与功能不匹配**
- **现状**: Tab 名为 "CLI"，但实际管理的是 Agent（Claude Code、Codex、Hermes 等）
- **代码证据**: 
  - `TabBar.vue:16` 定义 `id: "cli"`
  - `CLITab.vue` 调用 `agentsStore.fetchAgents()`
  - `AddCLIDialog.vue` 表单字段是 `name` 和 `skillsDir`，本质是添加 Agent
- **影响**: 用户心智模型混乱 — "CLI" 暗示命令行工具，但实际操作的是 AI 助手实例

**问题 2: CLI 与 Symlink 的功能割裂**
- **现状**: 
  - CLI Tab: 管理 Agent 注册（添加/删除 Agent）
  - Symlink Tab: 管理 Agent 维度的 Skill 同步状态
- **用户路径**: 添加 Agent 后，需切换到 Symlink Tab 才能查看同步状态，信息流断裂
- **代码证据**: `SymlinkTab.vue:35` 只显示 `agents.filter(a => a.detected)`，未检测的 Agent 完全隐藏

**问题 3: Dashboard Tab 定位模糊**
- **现状**: TabBar 中定义了 Dashboard，但未找到对应实现组件
- **风险**: v0.1 阶段如果是空壳，会让用户看到空白页，体验差
- **建议**: 要么隐藏，要么提供简单统计面板（Skills 总数、Agents 数、同步状态概览）

**问题 4: 5 个 Tab 对轻量工具偏多**
- 用户核心心智模型只有两件事：**我有哪些 Skill** 和 **它们分发给了谁**
- Symlink 暴露了实现细节（symlink/junction/copy），普通用户不关心
- History 对轻量工具来说偏高级

### 2.3 建议方案

**方案 A: 重命名 + 合并（推荐）**
```
Agents (🔧) → Skills (📁) → History (🕐) → Settings (⚙️)
```
- 将 "CLI" 重命名为 "Agents"，统一术语
- 将 Symlink 功能合并到 Agents Tab 的详情页（每个 Agent 显示其 Skill 同步状态）
- History 保留为独立 Tab
- Settings 从弹窗改为独立 Tab 或保留弹窗但增加入口

**方案 B: 保持当前结构，优化命名**
```
Agents (🔧) → Skills (📁) → Sync (🔗) → History (🕐)
```
- "CLI" → "Agents"
- "Symlink" → "Sync"（更直观）
- Dashboard 暂时隐藏，v0.2 再实现

**方案 C: 精简为 2 个主 Tab**
```
Skills（主列表 + 搜索 + 安装 + 分发操作）
Agents（管理 Agent + 查看其 skill 分布）
```
Symlink 细节和 History 下沉到二级（设置/右键菜单）。

**优先级**: 🔴 高  
**工作量**: 方案 A 中等（2-3 天），方案 B 小（0.5 天），方案 C 中等  
**影响范围**: TabBar、路由、i18n 文案、用户文档

---

## 3. 核心操作路径 🔴 高优先级

### 3.1 主路径分析

**路径 1: 安装 Skill**
```
Skills Tab → 点击 "+ 安装" → InstallDialog 弹窗 → 选择 Skill → 确认安装
```
- **步数**: 3 步（点击按钮 → 选择 → 确认）
- **问题**: 
  - 安装后没有"关联到 Agent"的下一步引导
  - InstallDialog 只有文本输入框让用户手动粘贴路径，没有文件夹选择器、拖拽导入、从 URL/GitHub 安装
- **代码证据**: `SkillList.vue:65` 触发 `showInstall = true`，但 InstallDialog 关闭后没有后续动作

**路径 2: 添加 Agent**
```
CLI Tab → 点击 "+ 添加" → AddCLIDialog → 填写 name + skillsDir → 保存
```
- **步数**: 4 步（点击 → 填写名称 → 选择目录 → 保存）
- **问题**: 
  - 需要手动填写 `name` 和 `skillsDir`，对新手不友好
  - `skillsDir` 默认值是 `${cliPath}/skills`，但用户可能不知道这个约定
- **代码证据**: `AddCLIDialog.vue:21-26` 自动推导 `skillsDir`，但依赖用户先填写 `cliPath`

**路径 3: 关联 Skill 到 Agent（最高频操作）**
```
Symlink Tab → 选择 Agent → 查看 Skill 树 → 勾选/取消勾选 → 同步
```
- **步数**: 4-5 步
- **问题**: 操作路径较长，且需要在 SkillList 和 SymlinkTab 之间切换
- **这个路径太深了，应该能在 Skill 卡片上直接完成**

### 3.2 建议方案

**优化 1: SkillCard 直接支持 Link/Unlink（最高优先级）**

在 Skill 卡片上直接展示已关联的 Agent 标签，点击可 toggle：

```
[Skill Name]
Description...
[Claude ✓] [Codex ✓] [+ 添加]
```

这样用户在 Skills 列表就能完成 80% 的分发操作。

**优化 2: 安装后引导关联**
- 在 InstallDialog 关闭后，弹出"是否立即关联到 Agent？"的提示
- 或在 SkillCard 上直接显示"关联到 Agent"按钮

**优化 3: 安装对话框加文件夹选择器**
- 复用 `@tauri-apps/plugin-dialog` 的 `open()` 方法（CLICard 编辑模式已有类似实现）
- 理想情况下支持拖拽文件夹到窗口安装

**优化 4: Agent 添加流程简化**
- 提供"自动检测已安装的 Agent"功能（扫描常见路径）
- 或提供预设模板（Claude Code、Codex、Hermes 等一键添加）

**优化 5: 批量操作暴露**
- History 中有 `batch_link` / `batch_unlink` 记录，说明后端支持批量操作
- 但前端 SkillList 和 SymlinkTab 未见批量选择 UI
- 建议: 在 SymlinkTab 的 Skill 树中增加多选框，支持批量同步

**优先级**: 🔴 高  
**工作量**: 中等（3-5 天）  
**影响范围**: InstallDialog、AddCLIDialog、SymlinkTab、SkillCard

---

## 4. 命名一致性 🔴 高优先级

### 4.1 术语混乱现状

| 位置 | 术语 | 问题 |
|------|------|------|
| TabBar | `cli` | 与实际功能（Agent 管理）不符 |
| agentsStore | `agents` | 正确，但与 Tab 名不一致 |
| AddCLIDialog | `cli` | 表单字段是 Agent 属性 |
| SettingsPage | `vibe_skills_path` | "VAB" 术语突然出现 |
| SkillSource.from | `vibe-lib` / agent id | 混用 |
| errors.rs | `AgentNotFound` / `SkillNotFound` | 正确 |

### 4.2 代码证据

**证据 1: Tab 名 vs Store 名**
```typescript
// TabBar.vue:16
{ id: "cli", icon: "🔧", labelKey: "tabs.cli" }

// agents.ts (store)
export const useAgentsStore = defineStore('agents', ...)
```

**证据 2: SkillSource 的 from 字段**
```typescript
// SkillList.vue:19-28
for (const src of skill.sources) {
  if (src.from !== "vibe-lib") {
    const agent = agentsStore.agents.find(a => a.id === src.from);
    tags.add(agent ? agent.name : src.from);
  }
}
```
- `from` 可能是 `"vibe-lib"` 或 agent id，混用导致需要条件判断

**证据 3: 设置中的 VAB 术语**
```json
// locales/zh.json
"settings.vibe_skills_path": "Vibe Skills 路径"
```
- 突然出现 "Vibe" 术语，与 "CLI" / "Agent" 不统一

### 4.3 建议方案

**建立统一术语表**:
```
Agent: AI 编码助手实例（Claude Code、Codex、Hermes 等）
Skill: 可复用的技能包（SKILL.md + 相关文件）
Vibe Skills Library: 中心技能库（~/.vibe-skills/）
Link/Sync: Skill 与 Agent 的关联关系（通过 symlink 实现）
```

**全栈统一**:
- Tab 名: `cli` → `agents`
- Store 名: 保持 `agents`
- 组件名: `AddCLIDialog` → `AddAgentDialog`
- i18n key: `cli.*` → `agents.*`
- 设置项: `vibe_skills_path` → `library_path` 或 `skills_library_path`
- 分发操作统一叫 "Link" 而非 "Symlink/Sync"

**优先级**: 🔴 高  
**工作量**: 小（1-2 天，主要是重命名）  
**影响范围**: 全栈（前端组件、Store、i18n、Rust 命令名可选）

---

## 5. 首次体验 🟡 中优先级

### 5.1 当前状态

**空状态处理**:
- SkillList: 有 `no_skills` 提示 + `no_skills_hint` 引导文案 ✅
- CLITab: 未见专门的空状态处理（如果 `agents.length === 0`，显示空白网格）
- SymlinkTab: 有 `select_agent` 提示 ✅
- HistoryTab: 有 `empty` 和 `no_results` 两种空状态 ✅

**引导流程**:
- 无 onboarding 流程
- 无"快速开始"引导
- 新用户打开应用后，面对 5 个 Tab，可能不知道从何开始

### 5.2 问题分析

**问题 1: CLITab 空状态缺失**
- 如果用户没有安装任何 Agent，CLITab 显示空白网格，没有引导
- 对比: SkillList 有 `no_skills_hint` 提示"点击安装按钮添加 Skill"

**问题 2: 缺乏首次运行引导**
- 新用户可能不知道:
  1. 什么是 Agent？如何添加？
  2. 什么是 Skill？如何安装？
  3. 如何将 Skill 关联到 Agent？
- 当前设计假设用户已经理解这些概念

### 5.3 建议方案

**方案 1: 增加 Onboarding 流程（推荐）**
- 首次运行时显示 3 步引导:
  1. "添加你的第一个 Agent"（自动检测已安装的 Agent）
  2. "安装一些 Skill"（推荐热门 Skill）
  3. "关联 Skill 到 Agent"（演示同步流程）
- 实现: 在 `appStore` 中增加 `hasOnboarded` 标志，存储在 localStorage

**方案 2: 空状态优化**
- CLITab 空状态: 显示"还没有添加任何 Agent，点击添加按钮开始"
- 增加"自动检测已安装 Agent"按钮

**方案 3: 交互式教程**
- 在 Dashboard 或首页增加"交互式教程"入口
- 通过高亮 + 提示引导用户完成核心操作

**优先级**: 🟡 中  
**工作量**: 方案 1 中等（2-3 天），方案 2 小（0.5 天）  
**影响范围**: CLITab、appStore、新增 Onboarding 组件

---

## 6. 扩展性 🟡 中优先级

### 6.1 当前架构评估

**数据模型层（✅ 良好）**:
```rust
// models/agent.rs
pub struct Agent {
    pub id: String,
    pub name: String,
    pub skills_dir: String,  // 当前只支持 skills 目录
    pub detected: bool,
    pub enabled: bool,
    pub auto_detected: bool,
    pub linked_skills: Vec<String>,
}
```
- Agent 模型清晰，但 `skills_dir` 是单一字段
- 未来如果支持 memory/plugin/mcp 同步，需要扩展为:
  ```rust
  pub struct Agent {
      pub skills_dir: Option<String>,
      pub memory_dir: Option<String>,
      pub plugin_dir: Option<String>,
      pub mcp_config: Option<String>,
  }
  ```

**配置系统（✅ 良好）**:
- 使用 JSON 配置文件（`.vibe-config.json`）
- 支持动态添加/删除 Agent
- 易于扩展新字段

**History 系统（✅ 良好）**:
- 已实现完整的 undo/redo 机制
- 支持多种操作类型（install、delete、link、unlink、batch_link、batch_unlink）
- 易于扩展新操作类型

**Tab 结构（🟡 需调整）**:
- 当前 5 个 Tab 已经比较拥挤
- 未来如果增加 Memory/Plugin/MCP 管理，Tab 数量会爆炸
- 建议: 考虑分组或折叠（如 "Agent 管理" 分组下包含 Agents/Sync）

### 6.2 建议方案

**建议 1: Agent 模型抽象**
- 将 `skills_dir` 改为 `directories: HashMap<String, String>`
- key 可以是 `"skills"`, `"memory"`, `"plugin"`, `"mcp"`
- 便于未来扩展

**建议 2: Tab 结构优化**
- 当前: `CLI | Skills | Dashboard | Symlink | History`
- 建议: `Agents | Skills | Sync | History`（4 个 Tab）
- 未来扩展: `Agents | Skills | Memory | Plugins | MCP | History`（6 个 Tab）
- 或采用分组: `Agent 管理 ▼ | Skills | History`

**建议 3: Plugin 架构预留**
- 当前所有功能硬编码在组件中
- 建议: 设计 Plugin 接口，未来 Memory/Plugin/MCP 管理可以作为插件加载

**建议 4: 路径命名抽象**
- `~/.vibe-skills/` 在 config 层做抽象，概念上叫 "library" 或 "hub"

**建议 5: Manifest 解析扩展**
- `parsers/skill_md.rs` 做成 trait/enum 分发，支持不同 manifest 格式

**优先级**: 🟡 中  
**工作量**: 大（1-2 周，涉及数据模型重构）  
**影响范围**: Rust 模型、数据库 schema（如果有）、前端类型定义

---

## 7. UX 细节 🟡 中优先级

### 7.1 错误处理

**问题: 前端错误展示不统一**
```typescript
// SettingsPage.vue:59
alert(String(e));  // 原生 alert，与设计语言不一致

// AddCLIDialog.vue:61
addError.value = String(e);  // inline 错误提示

// HistoryTab.vue:151-155
historyStore.operationMessage = {
  type: "error",
  text: String(e),
};  // 自定义 message 组件
```

**建议**: 
- 统一使用自定义 Toast/Message 组件
- 将 Rust 端的 `VabError` 映射为用户友好的文案
  ```typescript
  // errors.ts
  export function mapError(error: string): string {
    if (error.includes('AgentNotFound')) return 'Agent 不存在';
    if (error.includes('SkillNotFound')) return 'Skill 不存在';
    if (error.includes('PermissionDenied')) return '权限不足，请检查目录权限';
    // ...
    return error;
  }
  ```

### 7.2 加载态

**问题: 缺少骨架屏**
- 当前所有 loading 状态都显示文本 `t('app.loading')`
- 对比: 现代应用普遍使用骨架屏（Skeleton）提升感知性能

**建议**:
- SkillList、CLITab、SymlinkTab 增加骨架屏
- 实现: 使用 Tailwind 的 `animate-pulse` + 占位块

### 7.3 交互一致性

**问题 1: 主题切换有两个入口**
- Header 上的快速切换按钮（☀️/🌙）
- Settings 页面的主题选择器
- 建议: 保留两个入口，但确保状态同步（已通过 `appStore.theme` 实现 ✅）

**问题 2: Dialog 风格不统一**
- SettingsPage: 使用原生 `@tauri-apps/plugin-dialog` 的文件选择器
- AddCLIDialog: 使用自定义弹窗
- 建议: 统一使用自定义弹窗，保持视觉一致性

**问题 3: 删除确认不统一**
- 部分用了 `alert()`，部分用了 `ConfirmDialog`

### 7.4 其他细节

**问题 1: SymlinkTab 隐藏未检测的 Agent**
```typescript
// SymlinkTab.vue:35
v-for="agent in agentsStore.agents.filter(a => a.detected)"
```
- 用户添加了一个 Agent，但路径不存在时，在 SymlinkTab 中完全看不到
- 建议: 显示所有 Agent，未检测的显示警告图标 + 提示"路径不存在"

**问题 2: SkillCard 缺少快捷操作**
- 当前 SkillCard 只显示信息，没有"删除"、"编辑"、"关联到 Agent"等快捷按钮
- 建议: 增加 hover 时显示操作菜单

**问题 3: 搜索只匹配 name**
- Rust 端 `search_skills` 应同时匹配 name 和 description：
  ```rust
  // 当前：只匹配 name
  .filter(|s| s.name.to_lowercase().contains(&q))
  
  // 建议：同时匹配 description
  .filter(|s| s.name.to_lowercase().contains(&q) || s.description.to_lowercase().contains(&q))
  ```

**问题 4: Dashboard RelationGraph 定位**
- 当前矩阵表格在 agent 少时够用，但应改名为「共享视图」而非「关系图」
- 如果后续要做真正的关系图，可用 D3 力导向图，但优先级不高

**问题 5: History 的 undo 只支持最新一条**
- 实用性受限，建议支持多条 undo

**优先级**: 🟡 中  
**工作量**: 小-中（2-3 天）  
**影响范围**: 全局错误处理、加载态组件、Dialog 组件

---

## 8. 功能建议（轻量高价值）

| 功能 | 价值 | 成本 |
|------|------|------|
| SkillCard 上 Link/Unlink | 最高频操作，不应藏在二级 Tab | 低 |
| 安装对话框加文件夹选择器 | 最低成本的体验提升 | 低 |
| Tab 精简 + 命名统一 | 降低认知负担 | 中 |
| 搜索支持 description | 一行 Rust 代码改动 | 极低 |
| 空状态引导 | 首次体验很重要 | 低 |
| Skill 编辑 | 改 description/name，不用删了重装 | 低 |
| 批量 Link UI 暴露 | store 已有 `batchLink`，只需 UI | 低 |
| 架构预留（plugin/memory） | 不急着实现，但数据结构要留口子 | 中 |

---

## 9. 总结与优先级排序

### 9.1 优先级矩阵

| 优先级 | 问题 | 工作量 | 影响 | 建议 |
|--------|------|--------|------|------|
| 🔴 P0 | SkillCard 上直接做 Link/Unlink 操作 | 低 | 高 | 立即实现，缩短核心操作路径 |
| 🔴 P0 | 安装对话框加文件夹选择器 | 低 | 高 | 立即实现，最低成本体验提升 |
| 🔴 P0 | 命名混乱（CLI vs Agent） | 小 | 高 | 立即修复，统一术语 |
| 🔴 P0 | Dashboard Tab 空壳 | 小 | 中 | 隐藏或实现简单统计 |
| 🔴 P1 | 首次体验缺乏引导 | 中 | 高 | 增加 Onboarding 流程 |
| 🔴 P1 | 核心操作路径过长 | 中 | 高 | 优化安装/关联流程 |
| 🟡 P2 | 错误处理不统一 | 小 | 中 | 统一使用 Toast 组件 |
| 🟡 P2 | 缺少骨架屏 | 小 | 中 | 增加 Skeleton 组件 |
| 🟡 P2 | SymlinkTab 隐藏未检测 Agent | 小 | 中 | 显示所有 Agent + 警告 |
| 🟡 P2 | 搜索支持 description | 极低 | 中 | 一行代码改动 |
| 🟡 P3 | Agent 模型扩展性 | 大 | 低（当前） | v0.2 再重构 |

### 9.2 建议实施路线

**v0.1.1 (1 周)**:
- SkillCard 上直接做 Link/Unlink 操作
- 安装对话框加文件夹选择器
- 修复命名混乱（CLI → Agents）
- 隐藏或实现 Dashboard Tab
- 优化 CLITab 空状态
- 统一错误处理
- 搜索支持 description

**v0.2 (2-3 周)**:
- 增加 Onboarding 流程
- 优化核心操作路径（安装后引导关联、批量操作）
- 增加骨架屏
- SymlinkTab 显示所有 Agent
- Tab 精简 + 命名统一

**v0.3 (1-2 月)**:
- Agent 模型重构（支持 memory/plugin/mcp）
- Tab 结构优化（分组或折叠）
- Plugin 架构预留
- 为 plugin/memory 扩展做架构预留

---

## 10. 亮点与值得保留的设计

### 10.1 History 系统 ✅
- 完整的 undo/redo 机制
- 分页 + 筛选 + 搜索
- 操作类型图标 + 颜色区分
- **建议**: 作为核心特性保留，未来可以增加"操作统计"面板

### 10.2 i18n 支持 ✅
- 三语支持（中文、英文、繁体中文）
- 所有 UI 文本都已国际化
- **建议**: 保持这个标准，未来增加新文本时必须同步更新三语

### 10.3 主题系统 ✅
- 支持 system/light/dark 三种模式
- Header 快速切换 + Settings 详细设置
- CSS 变量统一管理
- **建议**: 保持，可以考虑增加"高对比度"主题

### 10.4 Rust 后端架构 ✅
- 清晰的模块划分（commands/models/parsers/utils）
- 错误类型化（VabError 枚举）
- 配置系统灵活（JSON + 动态 Agent）
- **建议**: 保持，未来可以增加 API 文档生成（如 OpenAPI）

---

## 11. 附录：代码证据索引

| 问题 | 文件 | 行号 |
|------|------|------|
| CLI Tab 命名 | `src/components/layout/TabBar.vue` | 16 |
| AddCLIDialog 表单 | `src/components/cli/AddCLIDialog.vue` | 84-133 |
| SkillSource.from 混用 | `src/components/skills/SkillList.vue` | 19-28 |
| Settings 用 alert() | `src/components/settings/SettingsPage.vue` | 59 |
| SymlinkTab 过滤未检测 Agent | `src/components/symlink/SymlinkTab.vue` | 35 |
| History undo/redo | `src/components/history/HistoryTab.vue` | 140-176 |
| Agent 模型 | `src-tauri/src/models/agent.rs` | 4-19 |
| VabError 枚举 | `src-tauri/src/errors.rs` | 6-63 |

---

## 12. 产品总结

产品核心定位（轻量多 Agent Skill 管理）正确，技术实现扎实。主要问题在于：

1. **信息架构偏重** — 5 个 Tab 对轻量工具来说太多
2. **核心操作路径偏深** — 分发操作藏在 Symlink Tab 的树形结构里
3. **命名概念混乱** — CLI/Agent/Symlink/Sync 等术语不统一

**最关键的改进**：把「分发 skill 到 agent」这个最高频操作从 Symlink Tab 提到 SkillCard 上，整个产品的可用性会上一个台阶。

---

**报告结束**  
**下一步**: 与开发团队讨论优先级，制定 v0.1.1 修复计划
