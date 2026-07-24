# v0.3 前端 UI 交互重构设计（v1 初稿）

> 状态：初稿，待两轮审查（事实核对 → implementation-ready）。
> 约束：后端命令可配合微调，但**不加新业务功能**；所有状态/动作判定必须复用既有同源逻辑（`useSkillAgentStatus`、`manageFilters` 谓词、`skillActionRegistry`），不得另写一套。

## 1. 背景与痛点

当前管理页（`ManageTab.vue`）是单页纵向堆叠：统计卡 → 筛选控制台 → 问题修复面板 → 技能列表（本地技能 + 底部紫色 Plugin 分组）→ 底部 fixed 浮动批量栏 → `BatchSyncPanel` 弹窗。用户确认的主要矛盾：

1. **过滤体系与展示交织**：筛选控制台（搜索/排序/状态预设/问题/来源/Agent）与列表、问题修复、批量选择挤在同一纵向流里，职责不清（`docs/v0.2/18` 已做语义收敛，但信息架构未动）。
2. **Plugin 来源交织**：插件技能被 `matchesStatusPreset` 硬排除在状态筛选外，又以紫色分组拼在列表底部；Agent 筛选通过 `PLUGIN_AGENT_MAP` 把插件映射回 Agent；状态点又把插件显示为"已同步"。同一对象在筛选、分组、状态三处被特殊处理，心智负担高，且"没有什么好的办法"在当前一维列表结构下确实无解。
3. **批量弹窗交互差**：`BatchSyncPanel` 是 Modal，完全遮挡列表上下文；三个总开关 `sync / link_only / unlink_only` 是工程术语，用户要猜语义；矩阵单元格、dry-run 预览、结果明细在同一弹窗内纵向争抢高度（`h-[min(88vh,720px)]` 内塞 4 个区块）。
4. **导航层级混乱**：顶部 TabBar 只有 manage/history，但 manage 内部又有"视图筛选 + 问题修复 + Agent 管理 + 设置"等多种入口，缺乏统一的信息架构。

## 2. 方案总览：应用级左侧导航 + 右侧内容区

**结论：采用"左侧导航（视图即 Tab）+ 右侧内容区"的应用级布局，而不是把功能拆成多个独立页面。** 理由：

- 链接/同步/修复天然是跨视图操作（同一批技能在"待处理"和"插件"视图间流动），独立页面会割裂选择上下文与批量操作。
- 左侧导航把"过滤条件"升级为**智能视图（Smart Views）**：每个视图 = 一组预设筛选，解决"过滤条件挂哪"的问题——80% 高频筛选一次点击完成，高级组合收进工具栏筛选弹层。
- 仅两个**临时任务流**保留浮层形态：安装技能（Modal）、批量操作（改为右侧抽屉）。修复中心不做独立页，而是"待处理"视图内的引导流。

### 2.1 目标布局

```text
┌────────────────────────────────────────────────────────────┐
│ Titlebar：品牌 · 主题 · 设置 · 窗口控制（移除顶部 TabBar）    │
├───────────┬────────────────────────────────────────────────┤
│ 侧边栏     │ 工具栏：搜索 · 排序 · 筛选(弹层) · 已启用 Token   │
│ ──────── ├────────────────────────────────────────────────┤
│ 技能库     │                                                │
│  全部      │   技能列表（沿用 SkillRow 行内展开详情）          │
│  待处理(3) │                                                │
│  已链接    │                                                │
│  未链接    │                                                │
│  未入库(8) │                                                │
│  仅库中    │                                                │
│ ──────── │                                                │
│ 来源       │                                                │
│  本地技能  │                                                │
│  插件技能▸ │   （选中 N 项时）底部内联操作条（非 fixed 覆盖）   │
│   按市场分组│        [批量操作 → 右侧抽屉]                    │
│ ──────── │                                                │
│ Agent 范围 │                                                │
│  (多选)    │                                                │
│ ──────── │                                                │
│ 历史记录   │                                                │
│ 设置       │                                                │
└───────────┴────────────────────────────────────────────────┘
```

规则：

- 侧边栏是应用唯一导航系统；`manage`/`history` 两个顶部 Tab 废弃，历史记录成为侧边栏视图，设置保持浮层（从标题栏与侧边栏底部均可进入）。
- 内容区沿用现有 `SkillRow` 列表与行内展开 `SkillDetail`（展示、删除、关联、预览、更新、冲突解决入口不变）。
- 批量操作条从"fixed 覆盖底部"改为内容区文档流内的普通行（遵循 `v0.2/18` §15.7 的滚动契约）；点击展开**右侧批量抽屉**。

## 3. 过滤体系重设计：智能视图 + 谓词复用

### 3.1 视图模型（新状态 `activeView`）

在 `useManageFilters` 之上新增视图层，**每个视图是对既有谓词的组合预设，不引入新判定**：

```ts
type SmartView =
  | "all"            // 本地技能全集（不含插件）
  | "attention"      // needs_attention（冲突/断链/链接他处）
  | "linked"         // linked_any
  | "unlinked"       // unlinked_all
  | "missing_library"// libraryScope: missing_library
  | "library_only"   // libraryScope: library_only
  | "plugins";       // domain = plugin（插件技能独立视图）
```

映射到既有状态：

```ts
viewToFilterPreset(view) → Partial<ManageFilterState> & { domain: "local" | "plugin" }
```

- `attention/linked/unlinked` 复用 `matchesStatusPreset`。
- `missing_library/library_only` 复用 `matchesLibraryScope`。
- 新增唯一谓词维度 `domain`：`local = !skill.from_plugin || hasNonPluginSource(skill)`（沿用 `ManageTab.vue` 现有 `hasNonPluginSource` 定义），`plugin = skill.from_plugin`。
- 侧边栏各视图计数复用 `computeFacetCounts` 的合同（应用其他组、忽略当前组），"待处理"徽标 = `facetCounts.status.needs_attention`。

### 3.2 Plugin 解耦（痛点 2 的答案）

- **插件技能不再出现在"全部"等本地视图**——`domain` 默认 `local`，插件彻底离开主列表，"交织"在一维结构里无解的问题在视图维度自然消失。
- **"插件技能"是一等侧边栏视图**：进入后按 `plugin_source` 分组展示（沿用现有 `pluginGroups` 计算与分组交互），操作保留"同步到中心库"。
- 高级筛选中保留"来源类型：本地 / 插件"维度，需要交叉分析时可在任意视图叠加，而非默认交织。
- 删除 `matchesStatusPreset` 中 `if (skill.from_plugin) return false` 的硬排除——状态谓词对本地/插件统一语义，解耦交给 `domain` 维度（这同时消除"状态点显示已同步、但状态筛选里找不到"的自相矛盾）。

### 3.3 工具栏与高级筛选

- 工具栏固定三项：搜索框（含独立清除）、排序、筛选按钮（带激活计数）。
- 点击"筛选"弹出**筛选弹层**（Popover，非展开区挤占列表）：问题类型（冲突/断链/重复，组内 OR）、来源范围（未入库/仅库中）、来源类型（本地/插件）、Agent 范围（any/exclude）。全部复用 `useManageFilters` 现有字段与 facet 计数。
- 已启用条件仍以可移除 Token 显示在工具栏下；侧边栏视图本身也是可移除 Token（点击回到"全部"）。
- Agent 数量自适应规则沿用 `v0.2/18` §5.3（0 隐藏 / 1-5 紧凑 / 6+ 可搜索多选）。

### 3.4 待处理视图 = 修复中心入口（不拆独立页）

- "待处理"视图顶部展示问题分组卡（冲突 / 断链 / 未入库 / 仅项目，来自 `IssueRepairPanel` 的四组，按唯一 skill id 去重——沿用 v0.2/18 §15.3 去重合同）。
- 点击分组 = 应用对应筛选（显式动作，不自动全选、不自动弹批量）。
- 在筛选结果上勾选后点"批量操作"进入抽屉，**保留问题分组上下文**（落地 backlog 第 6 项：批量模式按修复意图预设动作）。

## 4. 批量交互重设计：右侧抽屉式批量工作台

**形态：右侧滑出抽屉（非 Modal）**，宽度 `min(560px, 45vw)`，不遮挡左侧列表，执行中可对照列表状态；可展开为全宽。内部三步流，替代当前单弹窗四区块：

```text
步骤 1 选动作          步骤 2 选目标（矩阵）        步骤 3 预览与执行
┌──────────────┐     ┌────────────────────┐     ┌──────────────────┐
│ 动作卡（平实话术）│→  │ Skill × Agent 矩阵   │→  │ 分组 dry-run 列表  │
│ 链接到 Agent  │     │ 行/列/单元格勾选      │     │ 执行 + 内联结果    │
│ 收进技能库    │     │ 每格显示将发生的动作   │     │ 失败可定位重试     │
│ 断开链接     │     │ 冲突格显式标出        │     └──────────────────┘
│ 清理失效链接  │     └────────────────────┘
└──────────────┘
```

### 4.1 动作卡 ↔ 既有模式映射（判定同源，不换逻辑）

| 动作卡（用户话术） | 映射既有 Mode | 说明 |
| --- | --- | --- |
| 同步到最新状态 | `sync` | 默认；单元格动作由 `applySwitch` 决定 |
| 只建立链接 | `link_only` | 对应 backlog 修复预设 |
| 只断开链接 | `unlink_only` | 唯一覆盖基础动作处（relink→unlink）保留 |
| 清理失效链接 | `unlink_only` + dangling 预选 | 由"待处理→断链"上下文进入时自动预选 |

- 单元格有效动作继续由 `applySwitch(mode, status, action, vibe)` 计算；**将 `applySwitch`/`isConflictCell` 从 `BatchSyncPanel.vue` 提取到 `composables/useBatchCellActions.ts`**，使详情页与批量抽屉共用同一计算（落地 backlog 第 4 项"统一动作模型"）。
- 冲突处理保持"诚实失败"：冲突格默认不可选，提供"逐条解决"入口；用户显式确认后才进入执行并在失败明细中如实呈现（不改后端无 overwrite 参数的事实）。`sync_to_vibe` 后端的 force 语义维持现状。

### 4.2 交互改进点

1. 动作卡带一句结果描述（"会在每个勾选 Agent 的目录创建指向技能库的链接"），消除 `sync/link_only/unlink_only` 术语。
2. 步骤 2 矩阵沿用现有行/列/单元格勾选与 dry-run 计算，但表头 sticky、单元格直接显示"将发生的动作 + 当前状态"两行文本，不再仅靠颜色。
3. 步骤 3 预览成为主内容区（不再与矩阵抢 140px 高度），按 执行/跳过/冲突/需先入库 分组；执行结果内联展示，失败项提供"回到步骤 2 定位"。
4. 抽屉打开期间列表保持可见可操作；切换视图/清除选择时抽屉给出"上下文已变化"提示而非静默沿用旧选择。

## 5. 不变量与回归保护

1. **判定同源**：状态/动作/来源分类只允许 `useSkillAgentStatus`、`manageFilters.ts` 谓词、`skillActionRegistry`、（新）`useBatchCellActions` 四处产出；任何组件不得自行解析 `Skill.sources` 推断动作。
2. **选择作用域**：沿用 `useManageSelection`——选择永远限制在当前筛选结果；切换视图立即 `pruneInvisible`。
3. **后端命令不变**：首批不动 Rust；如需配合仅限参数级微调（当前评估：不需要新命令）。
4. **i18n**：所有新文案写入 `zh/en/zh-TW` 三份 locale。
5. **删除语义**（backlog 第 5 项）在详情区保持现有四类动作的明确区分，不在本次合并按钮。

## 6. 文件改动清单（预估）

新增：

- `src/components/layout/AppSidebar.vue`（视图导航 + 计数徽标 + Agent 范围 + 底部历史/设置）
- `src/components/manage/FilterPopover.vue`（高级筛选弹层）
- `src/components/batch/BatchDrawer.vue` + `BatchActionStep.vue` + `BatchTargetMatrix.vue` + `BatchPreviewStep.vue`
- `src/composables/useBatchCellActions.ts`（从 BatchSyncPanel 提取 applySwitch/isConflictCell/defaultSelection）
- `src/composables/useSmartViews.ts`（视图→筛选预设映射 + 视图计数）

修改：

- `src/App.vue`（TabBar → AppSidebar；history 成为视图）
- `src/components/layout/AppLayout.vue`（去掉 tabs slot，内容区双栏）
- `src/components/manage/ManageTab.vue` → 重构为 `SkillsView.vue`（删筛选控制台、插件分组段、fixed 批量栏，接入视图/弹层/抽屉）
- `src/components/manage/manageFilters.ts`（加 `domain` 谓词；移除 `matchesStatusPreset` 的插件硬排除）
- `src/components/manage/IssueRepairPanel.vue`（迁入"待处理"视图顶部，只发显式筛选动作）
- `src/stores/app.ts`（`activeTab` → `activeView`）
- `src/locales/{zh,en,zh-TW}.json`

删除（功能被取代后）：

- `src/components/layout/TabBar.vue`
- `src/components/manage/BatchSyncPanel.vue`（逻辑迁入抽屉）
- `ManageTab.vue` 中插件分组段与 fixed 浮动批量栏

暂保留：`SkillTree/SkillWorkbench/AgentMatrix`（v0.2/17 已标记暂保留；本次不扩大范围）。

## 7. 实施阶段

- **P0 IA 骨架**：AppSidebar + 视图模型 + 工具栏/Token；列表不变；history/设置接入。可独立交付。
- **P1 插件视图与筛选弹层**：domain 维度、插件视图分组、FilterPopover、移除硬排除。
- **P2 批量抽屉**：提取 useBatchCellActions、三步流、替换 BatchSyncPanel、问题上下文预设。
- **P3 打磨**：键盘/焦点（沿用 v0.2/18 §15.9）、空态/错误态、0/1/20 Agent 与 0/1/1000 Skill 边界、100%/125%/150% 缩放、三语言验收。

每阶段验收：`pnpm build` 通过；四场景（本地正常/本地冲突/插件/项目源）+ 0/1/多 Agent 回归；不引入页面级横向滚动。

## 8. 待审查问题（交给子代理）

1. 侧边栏 Agent 范围与工具栏筛选弹层中的 Agent 范围是否重复？是否只保留弹层？
2. `domain` 维度下，"插件技能也有本地副本"的 skill（`from_plugin && hasNonPluginSource`）应归入哪个视图？（现 `normalSkills` 把它放在普通区，需要裁定。）
3. 批量抽屉"只断开链接"与"清理失效链接"两张卡同映射 `unlink_only`，是否应合并为一张卡 + 预选差异？
4. 历史记录作为侧边栏视图后，`Ctrl+1/2` 快捷键与现有 history 内 `Ctrl+Z` 语义是否需要调整？
5. 右侧抽屉与行内展开详情同时打开时的宽度预算（最小视口 800×480 下是否允许抽屉 + 列表并存）？
