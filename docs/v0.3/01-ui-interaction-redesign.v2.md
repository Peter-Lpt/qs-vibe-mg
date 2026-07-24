# v0.3 前端 UI 交互重构设计（v2，事实核对修订版）

> 状态：v2。Round 1 技术事实核对完成，§8 五个问题已裁定；待 Round 2（implementation-ready）审查。
> 约束：后端命令可配合微调，但**不加新业务功能**；所有状态/动作判定必须复用既有同源逻辑（`useSkillAgentStatus`、`manageFilters` 谓词、`skillActionRegistry`），不得另写一套。
> 行内引用格式 `文件:行号` 均对应仓库当前代码，供审查时回跳。

## v2 修订摘要（相对 v1 的事实修正）

1. **§4.1 动作卡映射纠错**：v1 把"清理失效链接"映射为 `unlink_only` + dangling 预选——技术错误。`unlink_only` 下 dangling 单元格不可选（`BatchSyncPanel.vue:143-146`），`remove_dangling` 只能在 `sync` 模式下作为 dangling 的基础动作到达。已改为"动作卡 = (mode, cellScope) 二元组"。
2. **§4.1 冲突/force 语义诚实化**：v1"后端无 overwrite 参数"表述不准。事实：单条 `sync_to_vibe` 有 `force` 参数（`sync.rs:745-750`），前端行内动作传 `force=true` 且 `SkillDetail` 在 hash 不同时先弹覆盖确认；无 overwrite 能力的是 `batch_skill_action`（无 force 参数，内部固定 `force=false`，`sync.rs:910-911`）。
3. **§4.1 冲突格行为纠偏**：冲突格**始终**不可选（非"默认不可选"）；知情确认后冲突项也不执行，结果进入"冲突"分组而非"失败明细"（`BatchSyncPanel.vue:399-413, 463`）。
4. **§3.4 去重声明纠偏**：v0.2/18 §15.3 的去重合同存在，但现有 `IssueRepairPanel` **未实现**——组间可重叠，且 `totalIssueSkills` 直接按组长度求和（`IssueRepairPanel.vue:65`）。去重是"已立合同、尚未落地"，本次必须实现，不是"沿用现状"。
5. **Q1 裁定**：侧边栏删除"Agent 范围"区块，只保留筛选弹层一个入口（§2.1 布局草图已同步修改）。
6. **Q2 裁定**：`domain` 改为互斥分区（§3.1），淘汰现状"混合副本 skill 在普通区与插件区双份渲染"（`ManageTab.vue:86-88`）。
7. **§6 暂保留说明纠偏**：`SkillTree/SkillWorkbench/AgentMatrix`（及 `SkillCard/SkillWorkbenchRow`）当前**未被任何文件引用**（含 ManageTab），是孤儿组件；`SkillDetail` 被现役 `SkillRow` 引用，不受影响。
8. **§3.3 偏离声明**：筛选弹层用 Popover 是对 v0.2/18 §5.2"inline disclosure、不使用遮挡表格的 Popover"的有意识偏离，已补充理由与约束。
9. **新发现风险**：见 §8.6（"仅项目"组无筛选谓词可映射、IssueRepairPanel 谓词不同源、批量错误串解析耦合、Ctrl+Z 无输入框守卫、activeTab 持久化名存实亡等）。

## 1. 背景与痛点

当前管理页（`ManageTab.vue`）是单页纵向堆叠：统计卡 → 筛选控制台 → 问题修复面板 → 技能列表（本地技能 + 底部紫色 Plugin 分组）→ 底部 fixed 浮动批量栏 → `BatchSyncPanel` 弹窗。用户确认的主要矛盾：

1. **过滤体系与展示交织**：筛选控制台（搜索/排序/状态预设/问题/来源/Agent）与列表、问题修复、批量选择挤在同一纵向流里，职责不清（`docs/v0.2/18` 已做语义收敛，但信息架构未动）。
2. **Plugin 来源交织**：插件技能被 `matchesStatusPreset` 硬排除在状态筛选外（`manageFilters.ts:110-112`：`preset === "all"` 直接放行，其余预设 `if (skill.from_plugin) return false`），又以紫色分组拼在列表底部；Agent 筛选通过 `PLUGIN_AGENT_MAP` 把插件映射回 Agent（`manageFilters.ts:32-35, 155-157`）；状态点对插件给出 `status: "synced"` + 紫色插件标签（`useSkillAgentStatus.ts:106-117`），但 `linked_any` 筛选因硬排除永远不含插件——"状态点显示已同步、状态筛选里找不到"的自相矛盾确实存在。同一对象在筛选、分组、状态三处被特殊处理，且混合副本 skill（`from_plugin && hasNonPluginSource`）当前在普通区与插件区**双份渲染**（`ManageTab.vue:86-88`，注释自述为有意设计）。
3. **批量弹窗交互差**：`BatchSyncPanel` 是 Modal（`h-[min(88vh,720px)] w-[min(92vw,920px)]`，`BatchSyncPanel.vue:519`），完全遮挡列表上下文；三个总开关 `sync / link_only / unlink_only` 是工程术语；矩阵、dry-run 预览（限高 140px）、结果明细（限高 160px）在同一弹窗内纵向争抢高度。
4. **导航层级混乱**：顶部 TabBar 只有 manage/history（`TabBar.vue:15-18`），快捷键 tabs 数组在 `App.vue:26` 硬编码；manage 内部又有"视图筛选 + 问题修复 + Agent 管理 + 设置"等多种入口，缺乏统一的信息架构。

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
│  插件技能▸ │   （选中 N 项时）底部内联操作条（非 fixed 覆盖）   │
│   按市场分组│        [批量操作 → 右侧抽屉]                    │
│ ──────── │                                                │
│ 历史记录   │                                                │
│ 设置       │                                                │
└───────────┴────────────────────────────────────────────────┘
```

> v2 变更：按 §8 Q1 裁定，侧边栏**不再包含** "Agent 范围（多选）" 区块；Agent 范围只存在于工具栏筛选弹层。

规则：

- 侧边栏是应用唯一导航系统；`manage`/`history` 两个顶部 Tab 废弃，历史记录成为侧边栏视图，设置保持浮层（现状入口在标题栏，`AppLayout.vue:72-77` + `appStore.showSettings`；侧边栏底部入口为新增，复用同一状态）。
- 内容区沿用现有 `SkillRow` 列表与行内展开 `SkillDetail`（展示、删除、关联、预览、更新、冲突解决入口不变）。
- 批量操作条从"fixed 覆盖底部"（现状 `ManageTab.vue:765-799`）改为内容区文档流内的普通行（遵循 `v0.2/18` §15.7 的滚动契约）；点击展开**右侧批量抽屉**。
- 侧边栏导航项与快捷键共用同一份导航注册表（替代 `App.vue:26` 硬编码的 `tabs` 数组），见 §8 Q4。

## 3. 过滤体系重设计：智能视图 + 谓词复用

### 3.1 视图模型（新状态 `activeView`）

在 `useManageFilters` 之上新增视图层，**每个视图是对既有谓词的组合预设，不引入新判定**：

```ts
type SmartView =
  | "all"            // 本地技能全集（不含纯插件副本）
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
- 新增唯一谓词维度 `domain`，按 §8 Q2 裁定为**互斥分区**（沿用 `ManageTab.vue:77-84` 现有 `hasNonPluginSource` 定义）：
  - `local = !skill.from_plugin || hasNonPluginSource(skill)`（含"插件+本地副本"的混合 skill）；
  - `plugin = skill.from_plugin && !hasNonPluginSource(skill)`（仅纯插件副本）。
- 每个 skill 恰好属于一个 domain，视图计数可加和；现状的双份渲染（`ManageTab.vue:86-88`）随之淘汰。
- 侧边栏各视图计数复用 `computeFacetCounts` 的合同（应用其他组、忽略当前组，`manageFilters.ts:225-251`），**且必须在本视图 domain 作用域内计算**——否则移除硬排除后，"待处理"徽标（= `facetCounts.status.needs_attention`）会吸入插件域的冲突项。domain 成为 facet 计算的新前置维度。

### 3.2 Plugin 解耦（痛点 2 的答案）

- **插件技能不再出现在"全部"等本地视图**——`domain` 默认 `local`，纯插件副本彻底离开主列表，"交织"在一维结构里无解的问题在视图维度自然消失。
- **"插件技能"是一等侧边栏视图**：进入后按 `plugin_source` 分组展示（沿用现有 `pluginGroups` 计算与分组交互，`ManageTab.vue:91-159`），操作保留"同步到中心库"。
- 混合副本 skill（`from_plugin && hasNonPluginSource`）归 local 域，其"同步到中心库"入口由 local 视图 `SkillRow` 现有的 `@sync-plugin` → `handleSyncPlugin`（`ManageTab.vue:211-225`，走 `installSkill(pluginSource.path, false)`）继续承担，不丢失能力。
- 高级筛选中保留"来源类型：本地 / 插件"维度：语义为**对当前视图 domain 的临时覆盖**，覆盖时计入 `activeFilterCount` 并出可移除 Token；切换视图时重置为该视图默认 domain。
- 删除 `matchesStatusPreset` 中 `if (skill.from_plugin) return false` 的硬排除（`manageFilters.ts:111-112`，注意 `preset === "all"` 在其之前的 `manageFilters.ts:110` 已直接放行，故移除只影响非 all 预设）——状态谓词对本地/插件统一语义，解耦交给 `domain` 维度（这同时消除"状态点显示已同步、但状态筛选里找不到"的自相矛盾）。
- **移除硬排除的连带影响（必须与 domain 同阶段原子上线）**：
  1. `statusPriority`（`manageFilters.ts:174-180`）内部调用 `matchesStatusPreset(skill, "linked_any")`：插件 skill 因 `hasMarketplace` 将归为 linked（优先级 3→4），排序分档变化——与状态点"已同步"一致，属预期修正。
  2. `computeFacetCounts.status` 各组计数将包含插件 skill——必须配合 §3.1 的 domain 作用域，否则本地视图计数被污染。
  3. 若先移除硬排除、后上 domain，插件会短暂泄入状态筛选列表；因此 P1 内两者必须原子交付。

### 3.3 工具栏与高级筛选

- 工具栏固定三项：搜索框（含独立清除）、排序、筛选按钮（带激活计数）。
- 点击"筛选"弹出**筛选弹层**（Popover，非展开区挤占列表）：问题类型（冲突/断链/重复，组内 OR，`manageFilters.ts:121-128`）、来源范围（未入库/仅库中）、来源类型（本地/插件，domain 覆盖）、Agent 范围（any/exclude，`manageFilters.ts:143-162`）。全部复用 `useManageFilters` 现有字段与 facet 计数。
- **偏离声明**：v0.2/18 §5.2 要求"更多筛选"用 inline disclosure、不使用遮挡表格的 Popover。本方案有意识地偏离，理由：① v1 痛点 1 正是"展开区与列表挤在同一纵向流"，inline disclosure 在视图化布局中重新引入该问题；② 已启用条件有常驻 Token 行兜底，Popover 短暂遮挡列表顶部不造成状态丢失。约束：Popover 非模态、Esc/外点关闭、不得遮挡 Token 行与批量操作条。
- 已启用条件仍以可移除 Token 显示在工具栏下；侧边栏视图本身也是可移除 Token（点击回到"全部"）。
- Agent 数量自适应规则沿用 `v0.2/18` §5.3（0 隐藏 / 1-5 紧凑 / 6+ 可搜索多选），在弹层内实现。

### 3.4 待处理视图 = 修复中心入口（不拆独立页）

- "待处理"视图顶部展示问题分组卡，沿用 `IssueRepairPanel` 的四组定义（`IssueRepairPanel.vue:17-63`）：
  - `conflict`：`skill.has_conflict`；
  - `dangling`：`skill.has_dangling`；
  - `missing_lib`：无库来源 且 有 Agent 来源 且 有项目来源；
  - `only_project`：无库来源 且 有项目来源 且 无 Agent 来源。
- **必须同步落地的两个修正（v1 误述为"沿用现状"）**：
  1. **组间去重**：v0.2/18 §15.3 末条合同"问题摘要总数按唯一 Skill ID 去重，不使用组长度直接求和"**在现有代码中未实现**——四组过滤互不互斥（同一 skill 可同时进入 conflict 与 dangling 等组），`totalIssueSkills` 按组长度求和（`IssueRepairPanel.vue:65`）。本次必须实现按唯一 skill id 去重的总数与展示。
  2. **谓词同源**：`IssueRepairPanel` 目前自行解析 `Skill.sources`（`IssueRepairPanel.vue:40-58`），其 `hasAgent` 判定（`from !== "vibe-lib" && !from.startsWith("project:")`）把 marketplace/插件、external 来源都算作 Agent，与 `classifySkillSources`（`manageFilters.ts:70-103`）**不同源**，违反 §5.1 不变量。迁入"待处理"视图时必须改用 `classifySkillSources` 重写四组谓词；**组计数会因此变化**（插件/外部来源不再计入 Agent），验收用例需覆盖。
- 点击分组 = 应用对应筛选（显式动作，不自动全选、不自动弹批量，符合 v0.2/18 §15.6）。三组有现成谓词映射：冲突/断链 → `issues`，未入库 → `libraryScope: missing_library`。
- **"仅项目"组无对应筛选谓词**（v0.2/18 §15.2 明确 `project_only` 首期不进筛选模型，现状 `selectIssueGroup` 对该组走"全选+展开第一条"的违规路径，`ManageTab.vue:391-397`）。裁定：首期点击"仅项目"不写入 filter state，而是在待处理视图内直接展示该组的 skill 名单（数据来自同一去重后的分组计算）；是否补 `project_only` 谓词留待后续评估。
- 在筛选结果上勾选后点"批量操作"进入抽屉，**保留问题分组上下文**（落地 backlog 第 6 项：批量模式按修复意图预设动作）。注意现状 `repairContext` 通路实际为空转：`selectIssueGroup` 对 conflict/dangling/missing_lib 直接应用筛选后 return，从不打开批量面板（`ManageTab.vue:375-389`），`BatchSyncPanel` 的 `repairContext` 目前恒为 null，`"uncovered" → link_only` 预设（`BatchSyncPanel.vue:111-119`）及 `batch_repair_uncovered/only_agent` 文案为无触发方的遗留代码。本次为该通路首次真正接线。

## 4. 批量交互重设计：右侧抽屉式批量工作台

**形态：右侧滑出抽屉（非 Modal）**，默认宽度 `min(520px, 42vw)`，不遮挡左侧列表，执行中可对照列表状态；可展开为全宽。宽度预算与最小视口退化规则见 §8 Q5。内部三步流，替代当前单弹窗多区块：

```text
步骤 1 选动作          步骤 2 选目标（矩阵）        步骤 3 预览与执行
┌──────────────┐     ┌────────────────────┐     ┌──────────────────┐
│ 动作卡（平实话术）│→  │ Skill × Agent 矩阵   │→  │ 分组 dry-run 列表  │
│ 同步到最新状态 │     │ 行/列/单元格勾选      │     │ 执行 + 内联结果    │
│ 只建立链接    │     │ 每格显示将发生的动作   │     │ 冲突/跳过如实分组  │
│ 只断开链接    │     │ 冲突格显式标出        │     │ 失败可定位重试     │
│ 清理失效链接  │     └────────────────────┘     └──────────────────┘
└──────────────┘
```

### 4.1 动作卡 ↔ 既有模式映射（判定同源，不换逻辑）

动作卡内部模型为 **(mode, cellScope) 二元组**（v2 修正）：

| 动作卡（用户话术） | mode | cellScope | 说明 |
| --- | --- | --- | --- |
| 同步到最新状态 | `sync` | 全部行 | 默认；单元格有效动作由 `applySwitch` 决定（`BatchSyncPanel.vue:126-147`） |
| 只建立链接 | `link_only` | 全部行 | 仅 `unlinked` 且已有库副本的格可选 → `link`；对应 backlog 修复预设 |
| 只断开链接 | `unlink_only` | 全部行 | 仅 `synced`/`linked_elsewhere` 格可选 → `unlink`；代码中唯一有意覆盖基础动作处（relink→unlink，`BatchSyncPanel.vue:143-146`）保留 |
| 清理失效链接 | `sync` | 仅 dangling 行/格预设 | dangling 的基础动作即 `remove_dangling`，只有 `sync` 模式可达；**v1 原映射"`unlink_only` + dangling 预选"错误**：`unlink_only` 下 dangling 格返回 `none` 不可选，该路径功能不成立。"dangling 预选"为新行为——现状 `defaultSelection`（`BatchSyncPanel.vue:153-165`）勾选全部可选格，不按上下文收窄 |

- 单元格有效动作继续由 `applySwitch(mode, status, action, vibe)` 计算；**将 `applySwitch`/`isConflictCell`/`defaultSelection`/`hasVibe` 从 `BatchSyncPanel.vue` 提取到 `composables/useBatchCellActions.ts`**，使详情页与批量抽屉共用同一计算（落地 backlog 第 4 项"统一动作模型"）。
- **冲突与 force 语义（诚实版，替代 v1 的模糊表述）**：
  1. 单条命令 `sync_to_vibe(skill_id, agent_id, force, source_path)`（`sync.rs:745-776`）：库中已有同名且 hash 不同时，`force=false` 返回 `VibeError::Conflict`，`force=true` 用 Agent 版本覆盖库副本（`sync.rs:127-145`）。前端行内动作 `runAgentAction` 对 `sync_to_vibe` 传 `force=true`（`useSkillActions.ts:82-84`），但 `SkillDetail` 在 hash 不同时先弹覆盖确认（`SkillDetail.vue:344-373`），并非静默覆盖。
  2. 批量命令 `batch_skill_action(skill_id, agent_ids, action)` **无 force/overwrite 参数**（`sync.rs:878-882`）；`sync_to_vibe`/`replace_with_link` 在批量内固定 `force=false`（`sync.rs:910-911`），冲突逐 Agent 写入 `errors`（格式 `"agentId: message"`，`sync.rs:942`）。**批量"诚实失败"是后端结构保证的**，抽屉不得试图绕过（也不存在可绕过的参数）。
  3. 冲突格**始终不可选**（`cellOf` 对 `isConflict` 返回 `selectable: false`，`BatchSyncPanel.vue:226`），矩阵中提供"逐条解决"入口（emit `resolve-conflict` → 行内冲突解决流）。
  4. dry-run 的 conflict/blocked 行对"选中 skill × 目标 Agent"全量列出、不依赖勾选；当计划中含冲突项时执行前弹知情确认（`BatchSyncPanel.vue:399-413`）。确认后冲突项**仍不执行**（`cells` 只含 `category === "execute"`），结果进入 `result.conflicts` 分组（`BatchSyncPanel.vue:463`）如实呈现——不是"失败明细"，也不是强制覆盖。抽屉步骤 3 的文案必须保持此语义，不得暗示确认后冲突会被执行。

### 4.2 交互改进点

1. 动作卡带一句结果描述（"会在每个勾选 Agent 的目录创建指向技能库的链接"），消除 `sync/link_only/unlink_only` 术语。
2. 步骤 2 矩阵沿用现有行/列/单元格勾选与 dry-run 计算（`BatchSyncPanel.vue:337-393, 265-323`），但表头 sticky、单元格直接显示"将发生的动作 + 当前状态"两行文本，不再仅靠颜色。
3. 步骤 3 预览成为主内容区（不再与矩阵抢 140px 高度），按 执行/跳过/冲突/需先入库 分组；执行结果内联展示，失败项提供"回到步骤 2 定位"。
4. 抽屉打开期间列表保持可见可操作；切换视图/清除选择时抽屉给出"上下文已变化"提示而非静默沿用旧选择。重建 `rows` 时沿用现状策略（重置为 `defaultSelection` 并清空旧结果，`BatchSyncPanel.vue:89-104`）+ 显式提示；不做勾选保留，避免过度设计。
5. 执行管线不变：按 `(skillId, 有效动作)` 分组串行调用 `batchSkillAction(..., silent = true)` 抑制逐组刷新，全部完成后统一 `refreshSkills()` + `fetchAgents()`（`BatchSyncPanel.vue:415-448`；`stores/skills.ts:213-225` 中 `silent` 仅控制前端自动刷新）。

## 5. 不变量与回归保护

1. **判定同源**：状态/动作/来源分类只允许 `useSkillAgentStatus`、`manageFilters.ts` 谓词、`skillActionRegistry`、（新）`useBatchCellActions` 四处产出；任何组件不得自行解析 `Skill.sources` 推断动作。**现有违规点必须随本次修复**：`IssueRepairPanel` 的自解析谓词（见 §3.4）。
2. **选择作用域**：沿用 `useManageSelection`——选择永远限制在当前筛选结果（`manageFilters.ts:372-424`，含 `visibleSkills` 变更自动 `pruneInvisible` 的 watch）；切换视图立即裁剪。
3. **后端命令不变**：首批不动 Rust；如需配合仅限参数级微调（当前评估：不需要新命令）。批量无 force 参数是结构事实，UI 设计以此为准。
4. **i18n**：所有新文案写入 `zh/en/zh-TW` 三份 locale。
5. **删除语义**（backlog 第 5 项）在详情区保持现有四类动作的明确区分，不在本次合并按钮。
6. **刷新竞态合同**：`refreshSkills()` 以 `refreshRequestId` 保证仅最新一次提交（`stores/skills.ts:37-48`，符合 v0.2/18 §15.4）；批量执行遵循"分组串行 → 单次统一刷新"顺序，抽屉内不得引入逐组刷新。

## 6. 文件改动清单（预估）

新增：

- `src/components/layout/AppSidebar.vue`（视图导航 + 计数徽标 + 底部历史/设置；**不含** Agent 范围区块，见 §8 Q1）
- `src/components/manage/FilterPopover.vue`（高级筛选弹层）
- `src/components/batch/BatchDrawer.vue` + `BatchActionStep.vue` + `BatchTargetMatrix.vue` + `BatchPreviewStep.vue`
- `src/composables/useBatchCellActions.ts`（从 BatchSyncPanel 提取 applySwitch/isConflictCell/defaultSelection/hasVibe）
- `src/composables/useSmartViews.ts`（视图→筛选预设映射 + domain 作用域内的视图计数）

修改：

- `src/App.vue`（TabBar → AppSidebar；history 成为视图；`tabs` 硬编码数组 → 共享导航注册表；`Ctrl+1/2` 重绑定；`Ctrl+Z/Ctrl+Shift+Z` gating 改 `activeView === "history"` 并补可编辑目标守卫，见 §8 Q4）
- `src/components/layout/AppLayout.vue`（去掉 tabs slot，内容区双栏）
- `src/components/manage/ManageTab.vue` → 重构为 `SkillsView.vue`（删筛选控制台、插件分组段、fixed 批量栏，接入视图/弹层/抽屉）
- `src/components/manage/manageFilters.ts`（加 `domain` 谓词并接入 `filterSkills`/`computeFacetCounts` 的域作用域；移除 `matchesStatusPreset` 的插件硬排除——与 domain 原子上线，见 §3.2）
- `src/components/manage/IssueRepairPanel.vue`（迁入"待处理"视图顶部；四组谓词改用 `classifySkillSources` 重写；实现按唯一 skill id 去重；"仅项目"组改为视图内名单展示，见 §3.4）
- `src/stores/app.ts`（`activeTab` → `activeView`；统一写入路径——现状 `setActiveTab` 无任何调用方、`App.vue` 直接写 ref，localStorage 持久化名存实亡，本次一并修正；保留或清理 legacy tab id 映射 `overview/symlink/skills/agents/dashboard → manage`，`stores/app.ts:21-30`）
- `src/types/index.ts`（`TabId = "manage" | "history"`，`types/index.ts:119` → 视图 id 类型）
- `src-tauri/tauri.conf.json`（可选：增加 `minWidth: 800 / minHeight: 480`，落实 v0.2/18 §15.8 的最低目标视口；配置级微调，非新命令）
- `src/locales/{zh,en,zh-TW}.json`（含复用 `tabs.manage/tabs.history` 于侧边栏；清理无触发方的 `batch_repair_uncovered/only_agent` 或重新接线）

删除（功能被取代后）：

- `src/components/layout/TabBar.vue`（仅被 `App.vue:12,124` 引用，无其他引用方）
- `src/components/manage/BatchSyncPanel.vue`（仅被 `ManageTab.vue:8,862` 引用；逻辑迁入抽屉）
- `ManageTab.vue` 中插件分组段与 fixed 浮动批量栏

暂保留（v2 纠偏描述）：`SkillTree/SkillWorkbench/AgentMatrix`（及 `SkillCard/SkillWorkbenchRow`）当前**未被任何文件引用**（含 ManageTab），是孤儿组件，v0.2/17 已标记暂保留，本次不扩大范围；`SkillDetail` 被现役 `SkillRow`（`SkillRow.vue:9`）引用，不受孤儿组件未来清理的影响。

## 7. 实施阶段

- **P0 IA 骨架**：AppSidebar + 视图模型 + 工具栏/Token；列表不变；history/设置接入；导航注册表与快捷键迁移。可独立交付。
- **P1 插件视图与筛选弹层**：domain 维度、插件视图分组、FilterPopover、移除硬排除（**与 domain 原子交付**，见 §3.2）；IssueRepairPanel 谓词同源化 + 去重。
- **P2 批量抽屉**：提取 useBatchCellActions、三步流、替换 BatchSyncPanel、问题上下文预设（首次接通 repairContext 通路）。
- **P3 打磨**：键盘/焦点（沿用 v0.2/18 §15.9）、空态/错误态、0/1/20 Agent 与 0/1/1000 Skill 边界、100%/125%/150% 缩放、800×480 最小视口退化、三语言验收。

每阶段验收：`pnpm build` 通过；四场景（本地正常/本地冲突/插件/项目源）+ 0/1/多 Agent 回归；不引入页面级横向滚动；新增回归：混合副本 skill 的 domain 分区、四组去重计数、批量冲突诚实失败（冲突项进"冲突"分组且不执行）。

## 8. 已裁定问题（Round 1 结论）

### Q1 侧边栏 Agent 范围 vs 工具栏筛选弹层是否重复

**裁定：重复，只保留筛选弹层；侧边栏不放 Agent 范围控件。**
理由：① `agentIds`/`agentMatch` 是同一份 filter state，双入口必然产生同步负担与不一致风险，违背单一数据源；② Agent 范围带 any/**exclude** 高级语义，不属于"80% 一次点击"的智能视图，属于高级组合，归属弹层；③ 弹层已承载 facet 计数与 Token 移除 UI，侧边栏再放一份是第三处重复展示。侧边栏空间让给视图与计数徽标。若未来证明需要快捷 Agent 过滤，只允许以"写入同一 filter state 的快捷入口"形式回归，不做独立状态控件。§2.1 布局草图已据此修改。

### Q2 `from_plugin && hasNonPluginSource` 的 skill 归入哪个视图

**裁定：归 local 域；domain 为互斥分区。**
`local = !from_plugin || hasNonPluginSource(skill)`；`plugin = from_plugin && !hasNonPluginSource(skill)`。理由：① 混合副本有真实本地/库副本需要链接/同步/清理，管理动作集中在 local 视图，其"同步到中心库"入口由 `SkillRow` 现有 `@sync-plugin` 通路保留，能力不丢失；② 互斥分区保证每个 skill 恰好出现一次，视图计数可加和，`useManageSelection` 的可见性裁剪语义不被破坏；③ 现状双份渲染（`ManageTab.vue:86-88`）正是痛点 2 的"交织"来源之一，随分区淘汰。plugin 视图只含纯插件副本，聚焦"发现并入库"。

### Q3 "只断开链接"与"清理失效链接"两张动作卡是否合并

**裁定：不合并；两张卡保留，且 v1 的映射必须修正。**
两卡不共享 Mode，无法实现为"一张卡 + 预选差异"：
- 只断开链接 → `("unlink_only", 全部行)`：仅 `synced`/`linked_elsewhere` 格可选，有效动作 `unlink`。
- 清理失效链接 → `("sync", dangling 行/格预设)`：dangling 的基础动作是 `remove_dangling`，只在 `sync` 模式可达。
v1 把后者映射为"`unlink_only` + dangling 预选"在 `applySwitch` 下不成立——`unlink_only` 对 dangling 格返回 `{ effectiveAction: "none", selectable: false }`（`BatchSyncPanel.vue:143-146`），合并后该卡默认勾选集恒空，功能直接失效。内部模型统一为 (mode, cellScope) 二元组，见 §4.1。

### Q4 history 变侧边栏视图后快捷键语义

**裁定：**
- `Ctrl+1` / `Ctrl+2` 重绑定为两个应用级目的地：`Ctrl+1` = 技能库（返回时恢复上次技能库子视图，无记录则 `all`），`Ctrl+2` = 历史记录。技能库的子视图**不配数字键**（7 个子视图逐一键位记忆成本高、易与浏览器习惯冲突），侧边栏单击即达。
- `Ctrl+Z` / `Ctrl+Shift+Z` 语义不变（撤销/重做最近可撤销历史动作），生效条件由 `appStore.activeTab === "history"`（`App.vue:56,68`）改为 `activeView === "history"`。
- `App.vue:26` 硬编码 `tabs` 数组废弃，侧边栏与快捷键共用同一导航注册表（单一真源）。
- 附带小修：现状 `Ctrl+Z` 处理不检查可编辑目标，在历史视图内的输入框中按 `Ctrl+Z` 会触发历史撤销而非文本撤销；迁移时复用 `isEditableTarget` 守卫（`App.vue:80-85` 已有实现）。

### Q5 抽屉与行内详情并存的最小宽度预算

**裁定：以 v0.2/18 §15.8 的 800×480 CSS px 为最低目标视口；并存/覆盖按 960px 分档。**
事实前提：`tauri.conf.json:16-17` 只设了默认 `width: 1100 / height: 720`，**没有** `minWidth/minHeight`，"800×480"目前只是设计目标而非强制约束。
宽度账（CSS px）：侧边栏 ~200 + 抽屉 `min(520px, 42vw)` + 列表最小可用 ~320（复选框 + 名称 + 状态点 + 行内展开详情）。
- **窗口 ≥ 960px**：抽屉与列表并存（非模态），行内详情可同时展开。校验：960 时 200 + 403 + 320 = 923 ✓；默认 1100 时 200 + 462 + 438 ✓。
- **800–959px**：抽屉退化为覆盖层（半透明遮罩，Esc/点击遮罩关闭），不与列表并存；打开抽屉时自动收起行内展开详情。
- 建议在 `tauri.conf.json` 增加 `minWidth: 800 / minHeight: 480`（配置级微调），使最低视口成为强制约束；不加则 800 以下行为未定义。

### 8.6 Round 1 新发现风险（v1 未覆盖）

1. **"仅项目"组无筛选谓词可映射**：四组中 conflict/dangling/missing_lib 可映射现有谓词，`only_project` 不能（v0.2/18 §15.2 明确 `project_only` 不进筛选模型）；现状对该组走"全选+展开第一条"路径，违反 §15.6。裁定见 §3.4：首期视图内名单展示，不写 filter state。
2. **IssueRepairPanel 谓词与 `classifySkillSources` 不同源**：其 `hasAgent` 含 marketplace/external 来源，迁移后 missing_lib/only_project 组计数会变化——是行为变更，需纳入验收。
3. **批量错误解析的字符串耦合**：前端按 `": "` 切分 `"agentId: message"`（`BatchSyncPanel.vue:436-439` ↔ `sync.rs:942`），agentId 含冒号或错误文案含 `": "` 时会错位。抽屉重构沿用该解析即可，但列为已知脆弱点；若后端愿微调为结构化错误则更稳（非必须）。
4. **抽屉常驻后的数据过期**：抽屉打开期间，列表侧单条操作触发的后台 `refreshSkills` 会使抽屉内 `rows` 重建并重置勾选（现状 watch 行为）。§4.2.4 的"上下文已变化"提示即为对策，不做勾选保留。
5. **activeTab 持久化名存实亡**：`setActiveTab` 无调用方（`stores/app.ts:62`），`App.vue` 两处直接写 ref；`activeView` 迁移时统一走 setter 并明确持久化策略。
6. **KeepAlive 策略**：history 变视图后，`App.vue:127-130` 的 `KeepAlive` 需保留，以维持技能库视图的筛选/滚动状态；`ManageTab` 的 `onMounted` 数据加载（`ManageTab.vue:288-302`）随视图拆分迁移到 `SkillsView`。
