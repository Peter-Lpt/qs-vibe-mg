# v0.3 前端 UI 交互重构设计（v3.1，实施稿）

> v3.1 增补（2026-07-24）：§17 生命周期与弃用/禁用架构适配、§18 跨平台适配、§19 性能预算、§20 决策记录（原 4 项开放问题已代用户定案）。§1–§16 与 v3 一致。

> 状态：v3。Round 2 完成——在 v2 已裁定结论基础上加深到施工粒度（组件契约 / Composable 签名 / 数据模型 / 伪代码 / 状态迁移 / i18n / 任务卡 / 验收清单），可直接按 §13 任务卡开工。
> 基准：本文档以 v2（`01-ui-interaction-redesign.v2.md`）为唯一基准，**完整继承其全部结论与裁定**（v2 修订摘要 9 条、Q1–Q5、§8.6 风险清单），不推翻任何一条；v2 的 §1–§5 原样保留于本文 §1–§5，v2 的 §8 原样保留于本文 §15–§16。v2 的 §6（文件改动清单）升级为本文 §6，v2 的 §7（实施阶段）升级为本文 §13。
> 约束：后端命令不变（不加新业务功能）；所有状态/动作判定必须复用既有同源逻辑（`useSkillAgentStatus`、`manageFilters` 谓词、`skillActionRegistry`），不得另写一套。
> 行内引用格式 `文件:行号` 均对应仓库当前代码，供施工时回跳。
> 术语约定：面向用户的文案用 10 秒能懂的话（如"收进技能库""同步到最新状态"）；技术标识符（`sync_to_vibe`、`batch_skill_action` 等）保留英文原名。

## v2 修订摘要（冻结继承）

1. **§4.1 动作卡映射纠错**：v1 把"清理失效链接"映射为 `unlink_only` + dangling 预选——技术错误。`unlink_only` 下 dangling 单元格不可选（`BatchSyncPanel.vue:143-146`），`remove_dangling` 只能在 `sync` 模式下作为 dangling 的基础动作到达。已改为"动作卡 = (mode, cellScope) 二元组"。
2. **§4.1 冲突/force 语义诚实化**：单条 `sync_to_vibe` 有 `force` 参数（`sync.rs:745-750`），前端行内动作传 `force=true` 且 `SkillDetail` 在 hash 不同时先弹覆盖确认；无 overwrite 能力的是 `batch_skill_action`（无 force 参数，内部固定 `force=false`，`sync.rs:910-911`）。
3. **§4.1 冲突格行为纠偏**：冲突格**始终**不可选；知情确认后冲突项也不执行，结果进入"冲突"分组而非"失败明细"（`BatchSyncPanel.vue:399-413, 463`）。
4. **§3.4 去重声明纠偏**：v0.2/18 §15.3 的去重合同存在，但现有 `IssueRepairPanel` **未实现**——组间可重叠，且 `totalIssueSkills` 直接按组长度求和（`IssueRepairPanel.vue:65`）。去重是"已立合同、尚未落地"，本次必须实现。
5. **Q1 裁定**：侧边栏删除"Agent 范围"区块，只保留筛选弹层一个入口。
6. **Q2 裁定**：`domain` 改为互斥分区，淘汰现状"混合副本 skill 在普通区与插件区双份渲染"（`ManageTab.vue:86-88`）。
7. **§6 暂保留说明纠偏**：`SkillTree/SkillWorkbench/AgentMatrix`（及 `SkillCard/SkillWorkbenchRow`）当前**未被任何文件引用**，是孤儿组件；`SkillDetail` 被现役 `SkillRow` 引用，不受影响。
8. **§3.3 偏离声明**：筛选弹层用 Popover 是对 v0.2/18 §5.2 的有意识偏离，已补充理由与约束。
9. **新发现风险**：见本文 §16（v2 §8.6）。

## 1. 背景与痛点（同 v2 §1）

当前管理页（`ManageTab.vue`）是单页纵向堆叠：统计卡 → 筛选控制台 → 问题修复面板 → 技能列表（本地技能 + 底部紫色 Plugin 分组）→ 底部 fixed 浮动批量栏 → `BatchSyncPanel` 弹窗。主要矛盾：

1. **过滤体系与展示交织**：筛选控制台与列表、问题修复、批量选择挤在同一纵向流里，职责不清。
2. **Plugin 来源交织**：插件技能被 `matchesStatusPreset` 硬排除在状态筛选外（`manageFilters.ts:110-112`），又以紫色分组拼在列表底部；Agent 筛选通过 `PLUGIN_AGENT_MAP` 把插件映射回 Agent（`manageFilters.ts:32-35, 155-157`）；状态点对插件给出 `status: "synced"` + 紫色插件标签（`useSkillAgentStatus.ts:106-117`），但 `linked_any` 筛选因硬排除永远不含插件——"状态点显示已同步、状态筛选里找不到"的自相矛盾确实存在。混合副本 skill（`from_plugin && hasNonPluginSource`）当前在普通区与插件区**双份渲染**（`ManageTab.vue:86-88`）。
3. **批量弹窗交互差**：`BatchSyncPanel` 是 Modal（`h-[min(88vh,720px)] w-[min(92vw,920px)]`，`BatchSyncPanel.vue:519`），完全遮挡列表上下文；三个总开关 `sync / link_only / unlink_only` 是工程术语；矩阵、dry-run 预览（限高 140px）、结果明细（限高 160px）纵向争抢高度。
4. **导航层级混乱**：顶部 TabBar 只有 manage/history（`TabBar.vue:15-18`），快捷键 tabs 数组在 `App.vue:26` 硬编码。

## 2. 方案总览（同 v2 §2）

**结论：采用"左侧导航（视图即 Tab）+ 右侧内容区"的应用级布局。** 视图 = 一组预设筛选（智能视图 Smart Views）；仅安装技能（Modal）与批量操作（右侧抽屉）保留浮层形态；修复中心不做独立页，是"待处理"视图内的引导流。

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

规则（含 Q1 裁定）：

- 侧边栏是应用唯一导航系统，**不含** "Agent 范围"区块；Agent 范围只存在于工具栏筛选弹层。
- `manage`/`history` 两个顶部 Tab 废弃，历史记录成为侧边栏视图；设置保持浮层（现状入口 `AppLayout.vue:72-77` + `appStore.showSettings`，侧边栏底部入口复用同一状态）。
- 内容区沿用现有 `SkillRow` 列表与行内展开 `SkillDetail`。
- 批量操作条从"fixed 覆盖底部"（`ManageTab.vue:765-799`）改为内容区文档流内的普通行；点击展开右侧批量抽屉。
- 侧边栏导航项与快捷键共用同一份导航注册表（替代 `App.vue:26` 硬编码 `tabs` 数组），见 §15 Q4 与本文 §11。

## 3. 过滤体系重设计（同 v2 §3，结论冻结）

### 3.1 视图模型（新状态 `activeView`）

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

- `attention/linked/unlinked` 复用 `matchesStatusPreset`；`missing_library/library_only` 复用 `matchesLibraryScope`。
- 新增唯一谓词维度 `domain`，按 Q2 裁定为**互斥分区**（沿用 `ManageTab.vue:77-84` 的 `hasNonPluginSource` 定义）：
  - `local = !skill.from_plugin || hasNonPluginSource(skill)`；
  - `plugin = skill.from_plugin && !hasNonPluginSource(skill)`。
- 每个 skill 恰好属于一个 domain，视图计数可加和；双份渲染随之淘汰。
- 侧边栏各视图计数复用 `computeFacetCounts` 的合同（应用其他组、忽略当前组，`manageFilters.ts:225-251`），**且必须在本视图 domain 作用域内计算**。domain 成为 facet 计算的新前置维度。施工实现见 §8.1 / §8.3 / §10.1。

### 3.2 Plugin 解耦（结论冻结）

- 插件技能不再出现在"全部"等本地视图；`domain` 默认 `local`。
- "插件技能"是一等侧边栏视图，进入后按 `plugin_source` 分组展示（沿用 `ManageTab.vue:91-159` 的 `pluginGroups` 计算与分组交互），操作保留"收进技能库"。
- 混合副本 skill 归 local 域，其"收进技能库"入口由 `SkillRow` 现有 `@sync-plugin` → `handleSyncPlugin`（`ManageTab.vue:211-225`，走 `installSkill(pluginSource.path, false)`）继续承担。
- 高级筛选中保留"来源类型：本地 / 插件"维度：语义为**对当前视图 domain 的临时覆盖**，覆盖时计入 `activeFilterCount` 并出可移除 Token；切换视图时重置为该视图默认 domain。
- 删除 `matchesStatusPreset` 中 `if (skill.from_plugin) return false` 的硬排除（`manageFilters.ts:111-112`；注意 `preset === "all"` 在其之前的 `manageFilters.ts:110` 已直接放行，故移除只影响非 all 预设）。删除方式见 §8.3。
- **移除硬排除的连带影响（必须与 domain 同阶段原子上线）**：
  1. `statusPriority`（`manageFilters.ts:174-180`）内部调用 `matchesStatusPreset(skill, "linked_any")`：插件 skill 因 `hasMarketplace` 将归为 linked（优先级 3→4），排序分档变化——与状态点"已同步"一致，属预期修正。
  2. `computeFacetCounts.status` 各组计数将包含插件 skill——必须配合 domain 作用域，否则本地视图计数被污染。
  3. 若先移除硬排除、后上 domain，插件会短暂泄入状态筛选列表；因此 P1 内两者必须原子交付。

### 3.3 工具栏与高级筛选（结论冻结）

- 工具栏固定三项：搜索框（含独立清除）、排序、筛选按钮（带激活计数）。
- 筛选弹层（Popover）：问题类型（组内 OR，`manageFilters.ts:121-128`）、来源范围（未入库/仅库中）、来源类型（本地/插件，domain 覆盖）、Agent 范围（any/exclude，`manageFilters.ts:143-162`）。全部复用 `useManageFilters` 现有字段与 facet 计数。
- **偏离声明**：v0.2/18 §5.2 要求 inline disclosure，本方案有意识用 Popover。约束：非模态、Esc/外点关闭、不得遮挡 Token 行与批量操作条。
- 已启用条件以可移除 Token 显示在工具栏下；侧边栏视图本身也是可移除 Token（点击回到"全部"）。
- Agent 数量自适应沿用 v0.2/18 §5.3（0 隐藏 / 1-5 紧凑 / 6+ 可搜索多选）。

### 3.4 待处理视图 = 修复中心入口（结论冻结）

- "待处理"视图顶部展示问题分组卡，沿用 `IssueRepairPanel` 四组定义（`IssueRepairPanel.vue:17-63`）。
- **必须同步落地的两个修正**：
  1. **组间去重**：按唯一 skill id 去重的总数与展示（v0.2/18 §15.3 合同，现状未实现）。
  2. **谓词同源**：四组谓词改用 `classifySkillSources`（`manageFilters.ts:70-103`）重写；**组计数会因此变化**（插件/外部来源不再计入 Agent），验收用例需覆盖。
- 点击分组 = 应用对应筛选（显式动作，不自动全选、不自动弹批量）。冲突/断链 → `issues`；未入库 → `libraryScope: missing_library`。
- **"仅项目"组无对应筛选谓词**：首期点击"仅项目"不写入 filter state，在待处理视图内直接展示该组 skill 名单（数据来自同一去重后的分组计算）；是否补 `project_only` 谓词留待后续评估。
- 勾选后点"批量操作"进入抽屉**保留问题分组上下文**（首次真正接通 `repairContext` 通路：现状 `selectIssueGroup` 对 conflict/dangling/missing_lib 应用筛选后直接 return，`BatchSyncPanel` 的 `repairContext` 恒为 null，`"uncovered" → link_only` 预设（`BatchSyncPanel.vue:111-119`）及 `batch_repair_uncovered/only_agent` 文案为无触发方遗留代码）。接线方案见 §9.2 / §12.3。

## 4. 批量交互重设计（同 v2 §4，结论冻结）

**形态：右侧滑出抽屉（非 Modal）**，默认宽度 `min(520px, 42vw)`；可展开为全宽；宽度预算与最小视口退化见 §15 Q5。内部三步流：选动作 → 选目标（矩阵）→ 预览与执行。

### 4.1 动作卡 ↔ 既有模式映射（判定同源，不换逻辑）

动作卡内部模型为 **(mode, cellScope) 二元组**：

| 动作卡（用户话术） | mode | cellScope | 说明 |
| --- | --- | --- | --- |
| 同步到最新状态 | `sync` | `all` | 默认；单元格有效动作由 `applySwitch` 决定（`BatchSyncPanel.vue:126-147`） |
| 只建立链接 | `link_only` | `all` | 仅 `unlinked` 且已有库副本的格可选 → `link` |
| 只断开链接 | `unlink_only` | `all` | 仅 `synced`/`linked_elsewhere` 格可选 → `unlink`；代码中唯一有意覆盖基础动作处（relink→unlink，`BatchSyncPanel.vue:143-146`）保留 |
| 清理失效链接 | `sync` | `dangling` | dangling 的基础动作即 `remove_dangling`，只有 `sync` 模式可达；"dangling 预选"为新行为——现状 `defaultSelection`（`BatchSyncPanel.vue:153-165`）勾选全部可选格，不按上下文收窄 |

- 单元格有效动作继续由 `applySwitch(mode, status, action, vibe)` 计算；`applySwitch`/`isConflictCell`/`defaultSelection`/`hasVibe` 提取到 `composables/useBatchCellActions.ts`（签名见 §8.2）。
- **冲突与 force 语义（诚实版）**：
  1. 单条 `sync_to_vibe(skill_id, agent_id, force, source_path)` 有 `force` 参数；行内动作传 `force=true` 但 `SkillDetail` 先弹覆盖确认（`useSkillActions.ts:82-84`、`SkillDetail.vue:344-373`）。
  2. 批量命令 `batch_skill_action(skill_id, agent_ids, action)` **无 force 参数**，`sync_to_vibe`/`replace_with_link` 在批量内固定 `force=false`（`sync.rs:910-911`），冲突逐 Agent 写入 `errors`（格式 `"agentId: message"`，`sync.rs:942`）。**批量"诚实失败"是后端结构保证的**。
  3. 冲突格**始终不可选**（`cellOf` 对 `isConflict` 返回 `selectable: false`，`BatchSyncPanel.vue:226`），矩阵提供"逐条解决"入口（emit `resolve-conflict`）。
  4. dry-run 的 conflict/blocked 行全量列出、不依赖勾选；计划含冲突项时执行前弹知情确认；确认后冲突项**仍不执行**，结果进入 `result.conflicts` 分组如实呈现。抽屉步骤 3 的文案必须保持此语义。

### 4.2 交互改进点（结论冻结）

1. 动作卡带一句结果描述，消除 `sync/link_only/unlink_only` 术语。
2. 矩阵沿用现有行/列/单元格勾选与 dry-run 计算；表头 sticky、单元格直接显示"将发生的动作 + 当前状态"两行文本。
3. 步骤 3 预览成为主内容区，按 执行/跳过/冲突/需先入库 分组；执行结果内联展示，失败项提供"回到步骤 2 定位"。
4. 抽屉打开期间列表保持可见可操作；切换视图/后台刷新导致 `rows` 重建时给出"上下文已变化"提示而非静默沿用旧选择；重建沿用现状策略（重置为 `defaultSelection` 并清空旧结果，`BatchSyncPanel.vue:89-104`），不做勾选保留。伪代码见 §10.4。
5. 执行管线不变：按 `(skillId, 有效动作)` 分组串行调用 `batchSkillAction(..., silent = true)`，全部完成后统一 `refreshSkills()` + `fetchAgents()`（`BatchSyncPanel.vue:415-448`；`stores/skills.ts:213-225`）。伪代码见 §10.3。

## 5. 不变量与回归保护（同 v2 §5）

1. **判定同源**：状态/动作/来源分类只允许 `useSkillAgentStatus`、`manageFilters.ts` 谓词、`skillActionRegistry`、（新）`useBatchCellActions` 四处产出；`IssueRepairPanel` 的自解析谓词必须随本次修复。
2. **选择作用域**：沿用 `useManageSelection`——选择永远限制在当前筛选结果（`manageFilters.ts:372-424`，含 `visibleSkills` 变更自动 `pruneInvisible` 的 watch，`manageFilters.ts:411`）；切换视图立即裁剪。
3. **后端命令不变**：首批不动 Rust。批量无 force 参数是结构事实，UI 设计以此为准。
4. **i18n**：所有新文案写入 `zh/en/zh-TW` 三份 locale。
5. **删除语义**在详情区保持现有四类动作的明确区分，不在本次合并按钮。
6. **刷新竞态合同**：`refreshSkills()` 以 `refreshRequestId` 保证仅最新一次提交（`stores/skills.ts:37-48`）；批量执行遵循"分组串行 → 单次统一刷新"，抽屉内不得引入逐组刷新。

## 6. 组件树与文件落位

### 6.1 最终目录结构（P2 完成态）

```text
src/
  App.vue                                【修改】TabBar→AppSidebar；KeepAlive 保留；快捷键迁移
  types/index.ts                         【修改】TabId → ViewId / SmartViewId（§11.1）
  stores/app.ts                          【修改】activeTab → activeView + lastSkillsView（§11.2）
  composables/
    useSmartViews.ts                     【新增】视图注册表 + viewToFilterPreset + viewCounts（§8.1）
    useBatchCellActions.ts               【新增】批量单元格计算与勾选（§8.2）
    useSkillAgentStatus.ts               【不变】
    skillActionRegistry.ts               【不变】
    useSkillActions.ts                   【不变】
  components/
    layout/
      AppLayout.vue                      【修改】删 tabs slot；改为 侧边栏+内容 双栏
      AppSidebar.vue                     【新增】应用级左侧导航（§7.1）
      TabBar.vue                         【删除】仅被 App.vue:12,124 引用
    manage/
      SkillsView.vue                     【改名+重构】由 ManageTab.vue 而来（§7.3）
      manageFilters.ts                   【修改】domain 谓词 + 移除硬排除（§8.3）
      FilterPopover.vue                  【新增】高级筛选弹层（§7.2）
      IssueRepairPanel.vue               【修改】谓词同源 + 去重 + "仅项目"名单
      SkillRow.vue                       【不变】
      SkillDetail.vue                    【不变】
      BatchSyncPanel.vue                 【删除】仅被 ManageTab.vue:8,862 引用；逻辑迁入 batch/
    batch/
      BatchDrawer.vue                    【新增】三步流抽屉容器（§7.4）
      BatchActionStep.vue                【新增】步骤 1 动作卡（§7.5）
      BatchTargetMatrix.vue              【新增】步骤 2 目标矩阵（§7.6）
      BatchPreviewStep.vue               【新增】步骤 3 预览与执行（§7.7）
    history/HistoryTab.vue               【不变】（成为侧边栏视图）
  locales/{zh,en,zh-TW}.json             【修改】§12
src-tauri/tauri.conf.json                【可选】minWidth:800 / minHeight:480（§15 Q5）
```

### 6.2 新组件职责一句话

| 组件 | 职责 |
| --- | --- |
| `AppSidebar` | 渲染视图导航项与计数徽标，点击写入 `appStore.setActiveView`，底部放历史/设置入口 |
| `FilterPopover` | 非模态弹层，编辑 `useManageFilters` 的高级条件（问题/来源范围/来源类型/Agent 范围） |
| `SkillsView` | 单个智能视图的内容区：工具栏、Token、（attention 时）修复分组卡、技能列表/插件分组、内联批量条、BatchDrawer 宿主 |
| `BatchDrawer` | 三步流容器：持有 `DrawerState`，编排三个步骤组件，执行管线与一致性守卫 |
| `BatchActionStep` | 步骤 1：四张动作卡（(mode, cellScope) 二元组）单选，带一句结果描述 |
| `BatchTargetMatrix` | 步骤 2：Skill × Agent 勾选矩阵，单元格显示"将发生的动作 + 当前状态" |
| `BatchPreviewStep` | 步骤 3：dry-run 分组预览、知情确认、执行按钮、内联结果分组 |

### 6.3 孤儿组件处置（继承 v2 裁定）

`SkillTree/SkillWorkbench/AgentMatrix`（及 `SkillCard/SkillWorkbenchRow`）当前未被任何文件引用，是孤儿组件，v0.2/17 已标记暂保留，**本次不扩大范围、不删除**；`SkillDetail` 被 `SkillRow.vue:9` 引用，不受影响。

## 7. 组件契约

> 约定：`Mode`、`CellScope`、`BatchActionCard`、`BatchRow`、`CellView`、`DryRunItem`、`DryRunCounts`、`BatchResult`、`RepairContext` 等类型集中定义于 `useBatchCellActions.ts`（§8.2）并 re-export；`SmartViewId`、`ViewId`、`DomainScope`、`SmartViewDef` 定义于 `useSmartViews.ts`（§8.1）；`ManageFilterModel = ReturnType<typeof useManageFilters>`。

### 7.1 `AppSidebar.vue`

```ts
defineProps<{
  activeView: ViewId;                              // 当前视图（含 "history"）
  counts: Record<SmartViewId, number>;             // 各视图徽标计数（§10.1）
}>();

defineEmits<{
  (e: "select", view: ViewId): void;               // 点击导航项；App 层调 setActiveView
  (e: "open-settings"): void;                      // 底部设置入口 → appStore.showSettings = true
}>();

defineExpose: 无
```

### 7.2 `FilterPopover.vue`

```ts
defineProps<{
  open: boolean;
  filterModel: ManageFilterModel;                  // useManageFilters 返回对象整体传入（单一数据源）
  facetCounts: FacetCounts;                        // domain 作用域后的计数（§8.3）
  agents: Agent[];                                 // detected && enabled
  defaultDomain: DomainScope;                      // 当前视图默认域，用于"来源类型"覆盖态展示
}>();

defineEmits<{
  (e: "close"): void;                              // Esc/外点/再次点击触发按钮
}>();

defineExpose: 无
// 约束：非模态；不遮挡 Token 行与批量操作条；Agent 组按 v0.2/18 §5.3 自适应
```

### 7.3 `SkillsView.vue`

```ts
defineProps<{
  view: SmartViewId;                               // App 层按 activeView 传入（"history" 时本组件不渲染）
}>();

defineEmits: 无
// 说明：批量抽屉由本组件宿主，resolve-conflict 内部闭环（关抽屉 + expandedSkillId = skillId），
// 与现状 resolveConflictFromBatch（ManageTab.vue:364-369）等价，不向 App 层冒泡。

defineExpose<{
  clearSelection(): void;                          // 透传 selectionModel.clearSelection
}>();
```

### 7.4 `BatchDrawer.vue`

```ts
defineProps<{
  open: boolean;
  selectedSkillIds: string[];                      // 与 BatchSyncPanel 现状 props 对齐（BatchSyncPanel.vue:17-20）
  repairContext?: RepairContext | null;            // 问题分组上下文（首次真正接线，§3.4）
  overlay?: boolean;                               // 800–959px 覆盖模式（§15 Q5），默认 false
}>();

defineEmits<{
  (e: "close"): void;
  (e: "remove-skill", skillId: string): void;
  (e: "resolve-conflict", skillId: string): void;
  (e: "applied"): void;                            // 执行完成后 SkillsView 做 pruneMissing
}>();

defineExpose<{
  resetToStep(step: DrawerStep): void;             // "回到步骤 2 定位"等外部跳转
}>();
```

### 7.5 `BatchActionStep.vue`

```ts
defineProps<{
  modelValue: BatchActionCard;                     // 当前选中的动作卡（含默认值 "sync"/"all"）
  repairContext?: RepairContext | null;            // 非空时顶部显示上下文提示条（沿用 batch_repair_* 视觉）
}>();

defineEmits<{
  (e: "update:modelValue", card: BatchActionCard): void;
}>();

defineExpose: 无
```

### 7.6 `BatchTargetMatrix.vue`

```ts
defineProps<{
  rows: BatchRow[];                                // 由 BatchDrawer 经 buildBatchRows 构建
  agents: Agent[];                                 // detected（列头）
  actionCard: BatchActionCard;                     // mode 决定可选性，cellScope 决定默认勾选
  selectedCells: ReadonlySet<string>;              // "skillId::agentId"
}>();

defineEmits<{
  (e: "toggle-cell", skillId: string, agentId: string): void;
  (e: "toggle-row", skillId: string): void;
  (e: "toggle-col", agentId: string): void;
  (e: "select-all"): void;                         // = defaultSelection(cellScope)
  (e: "clear"): void;
  (e: "remove-skill", skillId: string): void;
  (e: "resolve-conflict", skillId: string): void;  // 冲突格"逐条解决"入口
}>();

defineExpose: 无
// 渲染约束：表头与首列 sticky；单元格两行文本（将发生的动作 / 当前状态）；
// 冲突格 selectable=false 恒成立，渲染为"去处理冲突"按钮
```

### 7.7 `BatchPreviewStep.vue`

```ts
defineProps<{
  items: DryRunItem[];                             // 全量 dry-run（含 skip/conflict/blocked，不依赖勾选）
  counts: DryRunCounts;
  result: BatchResult | null;                      // 执行后内联展示
  operating: boolean;
}>();

defineEmits<{
  (e: "execute"): void;                            // 内部仍走知情确认 → runExecute（§10.3）
  (e: "back-to-matrix", key?: string): void;       // 失败项"回到步骤 2 定位"，key="skillId::agentId"
}>();

defineExpose: 无
// 文案约束：冲突分组标题必须为"冲突（不会执行）"语义；知情确认文案不得暗示确认后冲突会被执行
```

## 8. Composable 签名

### 8.1 `src/composables/useSmartViews.ts`（新增）

```ts
import type { ComputedRef, Ref } from "vue";
import type { Agent, Skill } from "../types";
import type { ManageFilterState } from "../components/manage/manageFilters";

// ── 类型 ─────────────────────────────────────────────
export type SmartViewId =
  | "all" | "attention" | "linked" | "unlinked"
  | "missing_library" | "library_only" | "plugins";

export type ViewId = SmartViewId | "history";

export type DomainScope = "local" | "plugin";

export interface SmartViewDef {
  id: SmartViewId;
  domain: DomainScope;        // 视图默认域（plugins 为 "plugin"，其余为 "local"）
  labelKey: string;           // sidebar.view_* i18n key（§12.1）
  icon: string;               // lucide 图标名
  showBadge?: boolean;        // attention=true：计数>0 时用警示色徽标
}

// ── 视图注册表（侧边栏与快捷键的单一真源，替代 App.vue:26 硬编码） ──
export const SMART_VIEWS: readonly SmartViewDef[];

// ── 视图 → 筛选预设（不写状态，纯映射） ──────────────────
export function viewToFilterPreset(
  view: SmartViewId
): Partial<ManageFilterState> & { domain: DomainScope };
// 返回约定：
//   all            → { domain: "local", statusPreset: "all" }
//   attention      → { domain: "local", statusPreset: "needs_attention" }
//   linked         → { domain: "local", statusPreset: "linked_any" }
//   unlinked       → { domain: "local", statusPreset: "unlinked_all" }
//   missing_library→ { domain: "local", statusPreset: "all", libraryScope: new Set(["missing_library"]) }
//   library_only   → { domain: "local", statusPreset: "all", libraryScope: new Set(["library_only"]) }
//   plugins        → { domain: "plugin", statusPreset: "all" }
// query/issues/agentIds/agentMatch/sort 不出现在预设里——属用户高级条件，跨视图保留。

// ── 视图计数（domain 作用域内，复用 computeFacetCounts 合同） ──
export function useSmartViews(
  skills: Ref<Skill[]> | ComputedRef<Skill[]>,
  agents: Ref<Agent[]> | ComputedRef<Agent[]>,
  state: ComputedRef<ManageFilterState>   // 当前筛选状态（弹层高级条件参与计数）
): {
  views: typeof SMART_VIEWS;
  viewCounts: ComputedRef<Record<SmartViewId, number>>;  // 算法见 §10.1
  attentionCount: ComputedRef<number>;                   // = viewCounts.attention，徽标别名
};
```

### 8.2 `src/composables/useBatchCellActions.ts`（新增，自 BatchSyncPanel.vue 提取）

```ts
import type { ComputedRef, Ref } from "vue";
import type { Agent, Skill } from "../types";
import type { AgentStatus, AgentStatusType } from "./useSkillAgentStatus";
import type { AgentAction, TFunc } from "./skillActionRegistry";

// ── 类型 ─────────────────────────────────────────────
export type Mode = "sync" | "link_only" | "unlink_only";
export type CellScope = "all" | "dangling";

export type BatchActionCardId = "sync" | "link" | "unlink" | "clean_dangling";

export interface BatchActionCard {
  id: BatchActionCardId;
  mode: Mode;
  cellScope: CellScope;
  labelKey: string;           // batch.action_*
  descKey: string;            // batch.action_*_desc（一句结果描述）
}

export const BATCH_ACTION_CARDS: readonly BatchActionCard[];
//  = [
//    { id: "sync",           mode: "sync",        cellScope: "all"      },
//    { id: "link",           mode: "link_only",   cellScope: "all"      },
//    { id: "unlink",         mode: "unlink_only", cellScope: "all"      },
//    { id: "clean_dangling", mode: "sync",        cellScope: "dangling" },
//  ]

export type RepairContext = "conflict" | "dangling" | "missing_lib";
// "only_project" 不进批量（§3.4 裁定：视图内名单展示）；遗留 "uncovered"/"only_agent" 见 §12.3

export const REPAIR_PRESETS: Record<RepairContext, { cardId: BatchActionCardId; labelKey: string; hintKey: string }>;
//  conflict    → cardId "sync"           （冲突格恒不可选，提示逐条解决）
//  dangling    → cardId "clean_dangling" （首次落地"dangling 预选"新行为）
//  missing_lib → cardId "sync"           （先收进技能库再按预览执行）

export interface BatchRow {
  skill: Skill;
  statuses: AgentStatus[];                 // useSkillAgentStatus 的 allAgentStatuses
}

export interface CellView {                 // 与 BatchSyncPanel.vue:171-182 现状一致
  skillId: string;
  agentId: string;
  selectable: boolean;                     // isConflict 时恒 false
  effectiveAction: AgentAction;
  isConflict: boolean;
  needsImport: boolean;
  checked: boolean;
  label: string;
  color: string;
  muted: boolean;
}

export type DryRunCategory = "execute" | "skip" | "conflict" | "blocked";

export interface DryRunItem {               // 与 BatchSyncPanel.vue:43-52 现状一致
  key: string;                              // "skillId::agentId"
  skillId: string;
  skillName: string;
  agentId: string;
  agentName: string;
  action: AgentAction | "conflict" | "needs_import" | "skipped";
  category: DryRunCategory;
  reason: string;
}

export interface DryRunCounts {             // 与 BatchSyncPanel.vue:325-334 现状一致
  execute: number; link: number; relink: number; clean: number; sync: number;
  skipped: number; conflict: number; blocked: number;
}

export interface BatchResult {              // 与 BatchSyncPanel.vue:54-62 现状一致
  synced: number;
  success: DryRunItem[];
  failed: { item: DryRunItem | null; skillId: string; agentId: string; message: string }[];
  warnings: { skillId: string; message: string }[];
  skipped: DryRunItem[];
  conflicts: DryRunItem[];
  blocked: DryRunItem[];
}

// ── 纯函数（直接搬迁，逻辑一字不改） ─────────────────────
export function hasVibe(skill: Skill): boolean;
//   skill.sources.some((s) => s.from === "vibe-lib")            （BatchSyncPanel.vue:121-123）

export function applySwitch(
  m: Mode,
  status: AgentStatusType | string,
  action: AgentAction,
  vibe: boolean
): { effectiveAction: AgentAction; selectable: boolean };
//   总开关覆盖规则                                               （BatchSyncPanel.vue:126-147）

export function isConflictCell(status: string, action: AgentAction, vibe: boolean): boolean;
//   status === "independent" && action === "sync_to_vibe" && vibe（BatchSyncPanel.vue:149-151）

export function buildBatchRows(
  skills: readonly Skill[],
  agents: readonly Agent[],               // detected
  t: TFunc
): BatchRow[];
//   逐 skill 调 useSkillAgentStatus 取 allAgentStatuses.value    （BatchSyncPanel.vue:89-104 的提取）

// ── 响应式主体 ────────────────────────────────────────
export function useBatchCellActions(
  rows: Ref<BatchRow[]>,
  actionCard: Ref<BatchActionCard>,       // mode/cellScope 的唯一来源
  selectedCells: Ref<Set<string>>,
  t: TFunc
): {
  cellOf(row: BatchRow, agent: Agent): CellView;          // BatchSyncPanel.vue:184-235
  defaultSelection(): Set<string>;                        // 按 actionCard.cellScope 收窄（§10.2，新行为仅 dangling 分支）
  dryRunItems: ComputedRef<DryRunItem[]>;                 // BatchSyncPanel.vue:265-323
  dryRunCounts: ComputedRef<DryRunCounts>;                // BatchSyncPanel.vue:325-334
  selectedTargetAgentIds: ComputedRef<Set<string>>;       // BatchSyncPanel.vue:256-263
  toggleCell(skillId: string, agentId: string): void;     // :337-343
  toggleRow(row: BatchRow): void;                         // :355-361
  toggleCol(agentId: string): void;                       // :379-385
  selectAll(): void;                                      // = defaultSelection()
  clearSelection(): void;
};
```

> 执行管线（`execute`/`runExecute`）不进入本 composable——它依赖 stores 与知情确认 UI 状态，归 `BatchDrawer` 持有（§10.3）。这样 `useBatchCellActions` 保持纯计算，详情页未来复用零负担（backlog 第 4 项）。

### 8.3 `manageFilters.ts` 变更（P1，与 domain 原子上线）

```ts
// ── 新增类型与谓词 ─────────────────────────────────────
export type DomainScope = "local" | "plugin";   // 自 useSmartViews re-export，避免循环依赖时以本文件为准

export function hasNonPluginSource(skill: Skill): boolean;
//   自 ManageTab.vue:77-84 上提，实现不变：
//   sources.some(s => s.from === "vibe-lib" || s.source_kind === "agent" ||
//     (!s.source_kind && !s.from.startsWith("claude-plugin:") && !s.from.startsWith("codex-plugin:")))

export function matchesDomain(skill: Skill, domain: DomainScope): boolean;
//   domain === "local"  → !skill.from_plugin || hasNonPluginSource(skill)
//   domain === "plugin" →  skill.from_plugin && !hasNonPluginSource(skill)

// ── ManageFilterState 增加 domain 字段 ──────────────────
export interface ManageFilterState {
  query: string;
  statusPreset: StatusPreset;
  issues: Set<IssueFilter>;
  libraryScope: Set<LibraryScope>;
  agentIds: Set<string>;
  agentMatch: AgentMatch;
  sort: SortMode;
  domain: DomainScope;                          // 新增；默认 "local"
}

// ── filterSkills：domain 为第一个前置谓词 ───────────────
export function filterSkills(skills: readonly Skill[], state: ManageFilterState, agents: readonly Agent[]): Skill[];
//   skills.filter(s =>
//     matchesDomain(s, state.domain) &&          // 新增，永远生效（视图默认域或弹层覆盖域）
//     matchesQuery(s, state.query) &&
//     matchesStatusPreset(s, state.statusPreset, agents) &&
//     matchesIssues(s, state.issues) &&
//     matchesLibraryScope(s, state.libraryScope, agents) &&
//     matchesAgentScope(s, state.agentIds, state.agentMatch)
//   ) → sortSkills(...)

// ── computeFacetCounts：domain 属前置维度，without* 变体不清除 ──
export function computeFacetCounts(skills, state, agents): FacetCounts;
//   withoutStatus / withoutIssues / withoutLibrary 均保留 state.domain；
//   即在"当前 domain 作用域 + 应用其他组"下计数，合同（manageFilters.ts:225-251）不变。

// ── 硬排除的删除方式（与以上 domain 改动同一提交，原子上线） ──
export function matchesStatusPreset(skill, preset, agents = []): boolean {
//   if (preset === "all") return true;
// - if (skill.from_plugin) return false;        ← 删除 manageFilters.ts:111-112 这两行
//   const sources = classifySkillSources(skill, agents);
//   ...（其余分支不变）
}
// 验收锚点：删除后 linked_any 对纯插件 skill 返回 true（hasMarketplace），
// 但纯插件副本因 matchesDomain("local") = false 不会泄入本地视图/计数。

// ── useManageFilters：新增 domain 状态与覆盖计数 ────────
export function useManageFilters(
  skills: ComputedRef<Skill[]> | Ref<Skill[]>,
  agents: ComputedRef<Agent[]> | Ref<Agent[]>,
  defaultDomain: Ref<DomainScope> | ComputedRef<DomainScope>   // 新增第三参：当前视图默认域
): {
  // ……现状返回字段全部保留（manageFilters.ts:347-369）
  domain: Ref<DomainScope>;                      // 新增
  clearFilters(): void;                          // 行为扩展：domain 重置为 defaultDomain.value
  // activeFilterCount 扩展：+ Number(domain.value !== defaultDomain.value)
  //   ——弹层"来源类型"覆盖时计数并出 Token；切视图时 SkillsView 重置 domain=视图默认域
};
```

## 9. 数据模型

### 9.1 `ViewState`（stores/app.ts，详见 §11）

```ts
interface ViewState {
  activeView: ViewId;             // 当前视图；持久化 localStorage "vibe-active-view"
  lastSkillsView: SmartViewId;    // Ctrl+1 返回技能库时恢复；持久化 "vibe-last-skills-view"
}
```

### 9.2 `DrawerState`（BatchDrawer 内部持有）

```ts
type DrawerStep = 1 | 2 | 3;

interface DrawerState {
  step: DrawerStep;                 // 当前步骤；打开时恒为 1
  actionCard: BatchActionCard;      // 动作卡 (mode, cellScope)；默认 BATCH_ACTION_CARDS[0]，
                                    // repairContext 非空时被 REPAIR_PRESETS 覆盖
  selectedCells: Set<string>;       // "skillId::agentId"；rows 重建时重置为 defaultSelection()
  dryRunItems: DryRunItem[];        // useBatchCellActions 的 computed 快照，步骤 3 消费
  result: BatchResult | null;       // 执行结果；rows 重建 / 换卡时清空
  contextStale: boolean;            // 一致性守卫标志（§10.4）：rows 因后台刷新/切视图重建后置 true，
                                    // 步骤 2/3 顶部显示"上下文已变化"提示条，用户看一眼后手动 dismiss
  operating: boolean;               // 执行中（禁用执行按钮与关闭时的误触）
  confirmAck: boolean;              // 冲突知情确认已勾选（沿用 BatchSyncPanel.vue:38 语义）
}

// repairContext 接线（首次落地，BatchSyncPanel.vue:111-119 的升级）：
//   SkillsView 打开抽屉时传入 repairContext；
//   watch(repairContext, { immediate: true }) → actionCard = REPAIR_PRESETS[ctx] 对应卡，
//   selectedCells = defaultSelection()，result = null。
```

步骤 gating：步骤 1 必有默认卡故恒可前进；步骤 2 → 3 不强制勾选数（预览如实列出 skip/conflict/blocked）；执行按钮沿用现状——`execute` 分类为空时 toast 警告并留在步骤 3（`BatchSyncPanel.vue:404-407`）。

## 10. 关键伪代码

### 10.1 侧边栏视图计数（复用 computeFacetCounts 合同 + domain 前置）

```ts
// useSmartViews.ts —— viewCounts 的实现
const viewCounts = computed<Record<SmartViewId, number>>(() => {
  // ① domain 前置：一次性分出两个互斥作用域（每个 skill 恰好落一个）
  const localScoped  = skills.value.filter((s) => matchesDomain(s, "local"));
  const pluginScoped = skills.value.filter((s) => matchesDomain(s, "plugin"));

  // ② local 域：一次 computeFacetCounts 同时服务 6 个本地视图。
  //    computeFacetCounts 内部合同 = "应用其他组、忽略当前组"（manageFilters.ts:225-251）：
  //      status.*  忽略 statusPreset；library.* 忽略 libraryScope；
  //      query / issues / agentIds / agentMatch / sort / domain 始终生效。
  //    state.value.domain 此时必为 "local" 或被弹层覆盖——
  //    计数语义应与"用户若切到该视图会看到什么"一致，因此这里显式构造 local 域状态：
  const localState: ManageFilterState = { ...state.value, domain: "local" };
  const facets = computeFacetCounts(localScoped, localState, agents.value);

  // ③ plugin 域：plugins 视图无对应 facet 组，直接 filterSkills 计数。
  //    保留 query/issues/libraryScope/agentIds（用户对插件同样可用的弹层条件），
  //    清掉 statusPreset——等价于 facets.status.all 的语义（硬排除删除后 status 谓词对插件有定义）。
  const pluginState: ManageFilterState = { ...state.value, domain: "plugin", statusPreset: "all" };
  const pluginCount = filterSkills(pluginScoped, pluginState, agents.value).length;

  return {
    all:             facets.status.all,
    attention:       facets.status.needs_attention,   // 侧边栏"待处理"徽标 = 此值，不吸插件域冲突项
    linked:          facets.status.linked_any,
    unlinked:        facets.status.unlinked_all,
    missing_library: facets.library.missing_library,
    library_only:    facets.library.library_only,
    plugins:         pluginCount,
  };
});
```

### 10.2 矩阵单元格计算与勾选

```ts
// useBatchCellActions.ts
function cellOf(row: BatchRow, agent: Agent): CellView {
  const key = `${row.skill.id}::${agent.id}`;
  const st = row.statuses.find((s) => s.agent.id === agent.id);
  const vibe = hasVibe(row.skill);
  if (!st) return { /* muted 空格，同 BatchSyncPanel.vue:188-201 */ } as CellView;

  const sw = applySwitch(actionCard.value.mode, st.status, st.action, vibe);
  const isConflict = isConflictCell(st.status, st.action, vibe);   // 恒 → selectable: false
  const needsImport = st.status === "unlinked" && !vibe;
  // label/color/muted 推导同 BatchSyncPanel.vue:210-222：
  //   冲突 → "有冲突"；不可选 → 当前状态文案；可选 → 将发生的动作(actionLabel)+actionColor
  return { skillId: row.skill.id, agentId: agent.id,
           selectable: isConflict ? false : sw.selectable,
           effectiveAction: sw.effectiveAction, isConflict, needsImport,
           checked: selectedCells.value.has(key), label, color, muted };
}

function defaultSelection(): Set<string> {
  const sel = new Set<string>();
  for (const row of rows.value) {
    const vibe = hasVibe(row.skill);
    for (const st of row.statuses) {
      // ★ 新行为（v2 §4.1 裁定的"dangling 预选"）：cellScope 收窄默认勾选
      if (actionCard.value.cellScope === "dangling" && st.status !== "dangling") continue;
      const sw = applySwitch(actionCard.value.mode, st.status, st.action, vibe);
      if (sw.selectable && !isConflictCell(st.status, st.action, vibe)) {
        sel.add(`${row.skill.id}::${st.agent.id}`);
      }
    }
  }
  return sel;
}
// 行/列/全选切换 selectableKeysForRow/Col 与 toggleRow/toggleCol/selectAll
// 逐行搬迁 BatchSyncPanel.vue:345-393，mode 改读 actionCard.value.mode。
```

### 10.3 执行管线（BatchDrawer 持有）

```ts
async function runExecute(plan: DryRunItem[]) {
  // ① 只取 execute 分类；conflict/blocked/skip 如实进结果分组，不执行
  const cells = plan
    .filter((i): i is DryRunItem & { action: AgentAction } => i.category === "execute")
    .map((i) => ({ skillId: i.skillId, agentId: i.agentId, action: i.action }));
  if (cells.length === 0) { toast.show(t("batch.no_selection"), "warning"); return; }

  // ② 知情确认：计划含冲突项且未确认 → 弹 ConfirmDialog；确认后冲突项仍不执行
  if (plan.some((i) => i.category === "conflict") && !state.confirmAck) {
    state.showConflictConfirm = true; return;
  }

  state.operating = true;
  // ③ 按 (skillId, 有效动作) 分组 —— 同一 skill 行内混合动作拆成独立调用，不串味
  const groups = new Map<string, { skillId: string; action: AgentAction; agentIds: string[] }>();
  for (const c of cells) {
    const k = `${c.skillId}::${c.action}`;
    (groups.get(k) ?? groups.set(k, { skillId: c.skillId, action: c.action, agentIds: [] }).get(k)!)
      .agentIds.push(c.agentId);
  }

  // ④ 串行调用 silent=true 抑制逐组刷新（stores/skills.ts:213-225）
  const errors: BatchResult["failed"] = []; const warnings: BatchResult["warnings"] = [];
  let totalSynced = 0;
  for (const g of groups.values()) {
    try {
      const res = await skillsStore.batchSkillAction(g.skillId, g.agentIds, g.action, true);
      totalSynced += res.synced_count;
      warnings.push(...res.warnings.map((message) => ({ skillId: g.skillId, message })));
      for (const e of res.errors) {
        // 已知脆弱点（§16.3）：按首个 ": " 切分 "agentId: message"（BatchSyncPanel.vue:436-439 ↔ sync.rs:942）
        const ci = e.indexOf(": ");
        errors.push({ item: null, skillId: g.skillId,
                      agentId: ci >= 0 ? e.slice(0, ci) : "",
                      message: ci >= 0 ? e.slice(ci + 2) : e });
      }
    } catch (e: unknown) {
      errors.push({ item: null, skillId: g.skillId, agentId: "", message: String(e) });
    }
  }

  // ⑤ 全部完成后统一刷新（合同 §5.6；不得逐组刷新）
  await skillsStore.refreshSkills();
  await agentsStore.fetchAgents();

  // ⑥ 组装 BatchResult：success 按 failedKeys 反推；conflicts 来自 plan 而非 errors（诚实语义）
  state.result = assembleResult(plan, totalSynced, errors, warnings);  // 同 BatchSyncPanel.vue:452-465
  state.operating = false; state.confirmAck = false;
  toast.show(/* 三档结果文案，同 BatchSyncPanel.vue:467-476 */);
  emit("applied");
}
```

### 10.4 一致性守卫

```ts
// ── SkillsView：切换视图 → 选择裁剪 + 抽屉提示 ─────────────
watch(() => props.view, (view) => {
  const preset = viewToFilterPreset(view);
  filterModel.statusPreset.value = preset.statusPreset ?? "all";
  filterModel.libraryScope.value = (preset.libraryScope as Set<LibraryScope> | undefined) ?? new Set();
  filterModel.domain.value = preset.domain;          // 弹层"来源类型"覆盖随切视图重置（§3.2）
  expandedSkillId.value = null;
  // 选择裁剪无需手写：useManageSelection 内部 watch(visibleSkills) 自动 pruneInvisible
  // （manageFilters.ts:411，合同 §5.2），filteredSkills 变化即触发。
  if (drawerOpen.value) drawerRef.value?.markContextStale();
});

// ── BatchDrawer：后台刷新 → rows 重建 + "上下文已变化"提示 ──
let firstBuild = true;
watch(
  () => [panelSkills.value, detectedAgents.value] as const,
  ([skills, agents]) => {
    rows.value = buildBatchRows(skills, agents, t);   // §8.2
    selectedCells.value = cellActions.defaultSelection(); // 沿用现状：重置勾选、不保留（§4.2.4）
    if (firstBuild) { firstBuild = false; return; }   // 初次构建不提示
    state.result = null;
    state.contextStale = true;                        // 步骤 2/3 顶部提示条：
                                                      // "列表数据已更新，目标与勾选已按最新状态重算"
  },
  { immediate: true }
);
// 触发源：抽屉打开期间列表侧单条操作 / 后台 refreshSkills（§16.4）、切视图（上方 watch）。
// 对策刻意简单：重置勾选 + 显式提示；不做勾选保留（v2 §4.2.4 裁定，避免过度设计）。
```

## 11. 状态迁移（stores/app.ts / types/index.ts / App.vue）

### 11.1 类型调整（types/index.ts:119）

```ts
// types/index.ts
export type SmartViewId =
  | "all" | "attention" | "linked" | "unlinked"
  | "missing_library" | "library_only" | "plugins";
export type ViewId = SmartViewId | "history";
// 删除 export type TabId = "manage" | "history"；
// 全仓 TabId 引用点（stores/app.ts:4,22-32,62、App.vue:10,26、TabBar.vue）随删除一并迁移。
```

### 11.2 activeTab → activeView：统一 setter 与持久化

```ts
// stores/app.ts（替换 app.ts:21-32, 62-65）
const storedView = localStorage.getItem("vibe-active-view") as ViewId | null;
// 旧 key 一次性迁移："vibe-active-tab" 仅存 manage/history（含 legacy 映射 overview/symlink/
// skills/agents/dashboard → manage，app.ts:21-30）， manage → 落 "all"，history → "history"。
const legacyTab = localStorage.getItem("vibe-active-tab");
const initialView: ViewId =
  isValidViewId(storedView) ? storedView
    : legacyTab === "history" ? "history" : "all";

const activeView = ref<ViewId>(initialView);
const lastSkillsView = ref<SmartViewId>(
  (localStorage.getItem("vibe-last-skills-view") as SmartViewId) || "all"
);

function setActiveView(view: ViewId) {          // 统一写入路径（修复 setActiveTab 无调用方问题，
  activeView.value = view;                      //  现状 App.vue:50,124 两处直接写 ref / v-model）
  if (view !== "history") {
    lastSkillsView.value = view;
    localStorage.setItem("vibe-last-skills-view", view);
  }
  localStorage.setItem("vibe-active-view", view);
}
// 迁移完成后旧 key "vibe-active-tab" 不再写入；保留读取兼容一个版本周期，之后随清理删除。
```

持久化策略：`activeView` 与 `lastSkillsView` 均写 localStorage（与 theme/locale 同级）；`activeView === "history"` 也持久化，刷新后回到历史视图（与现状 `activeTab` 持久化意图一致，本次使其名实相符）。

### 11.3 快捷键新 gating（App.vue，落实 Q4）

```ts
// 导航注册表单一真源：SMART_VIEWS（§8.1）+ 底部 history/settings 静态项；
// App.vue:26 硬编码 tabs 数组删除。数字键只配两个应用级目的地：
if (e.ctrlKey && !e.shiftKey && e.key === "1") {
  e.preventDefault();
  appStore.setActiveView(appStore.lastSkillsView);   // 恢复上次技能库子视图，无记录则 "all"
  return;
}
if (e.ctrlKey && !e.shiftKey && e.key === "2") {
  e.preventDefault();
  appStore.setActiveView("history");
  return;
}

// Ctrl+Z / Ctrl+Shift+Z：语义不变，gating 改 activeView，并补可编辑目标守卫（Q4 附带小修）
if (e.ctrlKey && !e.shiftKey && e.key === "z"
    && appStore.activeView === "history"
    && !isEditableTarget(e.target)) {                 // isEditableTarget 复用 App.vue:80-85 现有实现
  e.preventDefault(); /* undo 流程同 App.vue:56-65 */
}
if (e.ctrlKey && e.shiftKey && e.key === "Z"
    && appStore.activeView === "history"
    && !isEditableTarget(e.target)) {
  e.preventDefault(); /* redo 流程同 App.vue:68-77 */
}
```

### 11.4 KeepAlive 保留策略

- `App.vue:127-130` 的 `<KeepAlive>` **保留**，包裹 `SkillsView` 与 `HistoryTab`：

```vue
<KeepAlive>
  <SkillsView v-if="appStore.activeView !== 'history'" :view="appStore.activeView" />
  <HistoryTab v-else />
</KeepAlive>
```

- `SkillsView` 在 7 个技能库子视图间切换是**同一组件实例的 prop 变化**（不重建），筛选/滚动状态天然保留；`ManageTab.vue:288-302` 的 `onMounted` 数据加载整体迁入 `SkillsView`（`fetchSkills/fetchAgents/fetchIssues/normalizeAgents`），仍只执行一次。
- 切视图的筛选语义见 §10.4：视图预设覆盖 `statusPreset/libraryScope/domain`，`query/issues/agentIds/agentMatch/sort` 作为高级条件跨视图保留（Token 行可见可移除）。

## 12. i18n key 计划

### 12.1 新增 key 清单（zh 文案示例；en / zh-TW 同 key 同阶段落地）

```text
# ── sidebar.* 侧边栏 ──
sidebar.skills_library            技能库
sidebar.view_all                  全部
sidebar.view_attention            待处理
sidebar.view_linked               已链接
sidebar.view_unlinked             未链接
sidebar.view_missing_library      未入库
sidebar.view_library_only         仅库中
sidebar.section_source            来源
sidebar.plugins                   插件技能
sidebar.view_token_tip            点击回到「全部」

# ── filter.* 筛选弹层 ──
filter.trigger                    筛选
filter.group_issue                问题类型
filter.group_library              来源范围
filter.group_domain               来源类型
filter.domain_local               本地
filter.domain_plugin              插件
filter.group_agent                Agent 范围

# ── batch.* 批量抽屉 ──
batch.drawer_title                批量操作（{count}）
batch.step_action                 第 1 步 · 选动作
batch.step_target                 第 2 步 · 选目标
batch.step_preview                第 3 步 · 预览并执行
batch.action_sync                 同步到最新状态
batch.action_sync_desc            让每个勾选的 Agent 与技能库保持一致：能链接的链接，指错地方的改回来。
batch.action_link                 只建立链接
batch.action_link_desc            把技能库里的副本链接到还没链接的 Agent，其他一律不动。
batch.action_unlink               只断开链接
batch.action_unlink_desc          断开勾选 Agent 与技能库的链接，技能库副本保留。
batch.action_clean_dangling       清理失效链接
batch.action_clean_dangling_desc  删除指向已不存在位置的失效链接，不会删除真实文件。
batch.next_step                   下一步
batch.prev_step                   上一步
batch.context_stale               列表数据已更新，目标和勾选已按最新状态重算。
batch.conflict_group_note         冲突项不会执行；确认后仍列入「冲突」分组。
batch.back_to_matrix              回到上一步定位
batch.no_selection                没有可执行的目标，请回到上一步勾选。
batch.repair_conflict             修复冲突
batch.repair_conflict_hint        冲突不会批量处理，请在列表中逐条解决。
batch.repair_dangling             修复断链
batch.repair_dangling_hint        已预设「清理失效链接」，不会删除真实来源文件。
batch.repair_missing_lib          收进技能库
batch.repair_missing_lib_hint     已预设「同步到最新状态」，先把 Agent 副本收进技能库再按预览执行。

# ── plugins.* 插件视图 ──
plugins.view_title                插件技能
plugins.view_hint                 来自插件市场的技能副本，收进技能库后才能统一管理和链接。
plugins.sync_one                  收进技能库
plugins.group_sync_all            全部收进技能库
plugins.empty                     没有插件来源的技能
```

### 12.2 复用的既有 key（不重命名、不新增）

- `tabs.history`（"操作历史"）→ 侧边栏"历史记录"项直接复用。
- `tabs.manage`（"管理"）语义与侧边栏分组名不符，**不复用**，分组名用新增的 `sidebar.skills_library`。
- 筛选 Token 与弹层内既有 key：`manage.status_*`、`manage.quick_filter_missing_lib/only_lib`、`manage.filter_include/filter_exclude`、`manage.sort_*`、`manage.more_filters`、`manage.clear_filters`、`manage.filter_active_summary`。
- 批量矩阵与结果分组既有 key：`manage.batch_panel_*`（col_tip/row_tip/remove/conflict/resolve_conflict/needs_import/no_skills/no_agents 等）、`manage.batch_dry_run_*`、`manage.batch_result_*`、动作 label 的 `manage.btn_*`（经 `actionLabel`/`skillActionRegistry` 消费）。抽屉直接复用，避免双语义并存。
- `manage.batch_panel_execute/close` 复用为抽屉执行/关闭按钮文案。

### 12.3 待清理遗留 key（无触发方）

- `manage.batch_repair_uncovered`(+`_hint`)、`manage.batch_repair_only_agent`(+`_hint`)：现状 `repairContext` 恒 null（v2 §3.4 已查明），这两条预设无触发方。
- 处置建议（P2 执行）：**删除**这两条 key，其语义分别由"只建立链接"卡（`batch.action_link`，对应 uncovered 意图）与"收进技能库"流程（对应 only_agent 意图）承载；三份 locale 同步删除。若评审希望保留 key 以防回滚，则移到 `batch.legacy_*` 并标注 deprecated——默认按删除处理。
- `manage.batch_repair_dangling/missing_lib`(+`_hint`) 有真实触发方（四组分组卡），**迁移**为 `batch.repair_dangling/missing_lib`（§12.1），旧 key 删除。

## 13. 分阶段任务卡

> 每阶段结束 `pnpm build` 必须通过；四场景（本地正常/本地冲突/插件/项目源）+ 0/1/多 Agent 回归；不引入页面级横向滚动。

### P0 IA 骨架（可独立交付）

| # | 任务 | 改动文件 | 要点 |
| --- | --- | --- | --- |
| 1 | 类型迁移 | `src/types/index.ts` | `TabId` → `ViewId`/`SmartViewId`（§11.1）；grep 全仓 `TabId` 引用点一并改 |
| 2 | 视图状态 | `src/stores/app.ts` | `activeView` + `lastSkillsView` + `setActiveView` 统一 setter + 旧 key 迁移（§11.2） |
| 3 | 视图注册表 | `src/composables/useSmartViews.ts` | `SMART_VIEWS` + `viewToFilterPreset`（本阶段**不含** plugins 项与 domain；视图映射仅写 `statusPreset/libraryScope`，与现状状态筛选行为等价） |
| 4 | 侧边栏 | `src/components/layout/AppSidebar.vue` | 契约 §7.1；徽标计数暂由 `filterSkills` 直算（domain 前置在 P1 接入）；底部历史/设置入口 |
| 5 | 布局改造 | `src/components/layout/AppLayout.vue`、`src/App.vue` | 删 tabs slot 与 `TabBar.vue`；KeepAlive 按 §11.4；Ctrl+1/2、Ctrl+Z gating + `isEditableTarget`（§11.3） |
| 6 | 视图容器改名 | `ManageTab.vue` → `src/components/manage/SkillsView.vue` | 仅改名 + 接收 `view` prop + `watch(view)` 应用预设（§10.4 前半）；筛选控制台/插件分组/fixed 批量栏本阶段原样保留 |
| 7 | i18n | 三份 locale | `sidebar.*`（§12.1） |

**验收命令**：`pnpm build`；手测：Ctrl+1/2 往返、刷新后视图保持、历史视图输入框内 Ctrl+Z 不被劫持、侧边栏计数与列表条数一致。

### P1 插件视图与筛选弹层（domain 与移除硬排除**原子交付**）

| # | 任务 | 改动文件 | 要点 |
| --- | --- | --- | --- |
| 1 | domain 谓词 | `src/components/manage/manageFilters.ts` | `DomainScope`/`hasNonPluginSource` 上提/`matchesDomain`；`ManageFilterState.domain`；`filterSkills` 前置；`computeFacetCounts` 保留 domain（§8.3） |
| 2 | 移除硬排除 | 同上，**与 #1 同一提交** | 删除 `manageFilters.ts:111-112` 两行（§8.3 锚点）；确认 `statusPriority` 对插件归 linked 的排序变化（§3.2 连带影响 1） |
| 3 | 视图计数 | `src/composables/useSmartViews.ts` | views 增加 `plugins`；`viewCounts` 按 §10.1 实现 domain 前置 |
| 4 | 弹层 | `src/components/manage/FilterPopover.vue` | 契约 §7.2；`useManageFilters` 第三参 `defaultDomain` 与 `activeFilterCount` 扩展（§8.3）；Agent 组自适应 |
| 5 | SkillsView 瘦身 | `SkillsView.vue` | 删筛选控制台（搜索/排序移入工具栏）；domain 接入；插件视图按 `pluginGroups`（`ManageTab.vue:91-159`）迁移分组 UI；**删双份渲染**（`ManageTab.vue:86-88`）；`handleSyncPlugin` 保留 |
| 6 | 修复面板同源化 | `IssueRepairPanel.vue` | 四组谓词改 `classifySkillSources`；`totalIssueSkills` 按唯一 skill id 去重；"仅项目"组改视图内名单展示，不再走 `selectIssueGroup` 的违规路径（`ManageTab.vue:391-397`） |
| 7 | i18n | 三份 locale | `filter.*`、`plugins.*`（§12.1） |

**验收命令**：`pnpm build`；断言：混合副本 skill 全列表只出现一次；"待处理"徽标不含纯插件冲突项；移除硬排除后插件视图内 `linked` 计数正确；四组计数之和 ≥ 去重后总数。

### P2 批量抽屉

| # | 任务 | 改动文件 | 要点 |
| --- | --- | --- | --- |
| 1 | 计算提取 | `src/composables/useBatchCellActions.ts` | §8.2 全量签名；`defaultSelection` 新增 cellScope 收窄（§10.2，仅 dangling 分支为新行为） |
| 2 | 抽屉四件套 | `src/components/batch/BatchDrawer.vue`、`BatchActionStep.vue`、`BatchTargetMatrix.vue`、`BatchPreviewStep.vue` | 契约 §7.4–7.7；执行管线 §10.3；一致性守卫 §10.4；960px 分档（≥960 并存 / 800–959 overlay + 收起行内详情，§15 Q5） |
| 3 | 批量条改造 | `SkillsView.vue` | fixed 浮动栏（`ManageTab.vue:765-799`）→ 文档流内联操作条；`repairContext` 首次接线（`REPAIR_PRESETS`，§9.2）；"仅项目"不进批量 |
| 4 | 删旧面板 | `src/components/manage/BatchSyncPanel.vue` | 确认无引用后删除 |
| 5 | i18n | 三份 locale | `batch.*`（§12.1）；按 §12.3 删除 `batch_repair_uncovered/only_agent`、迁移 `batch_repair_dangling/missing_lib` |

**验收命令**：`pnpm build`；断言：四张动作卡默认勾选集正确（clean_dangling 仅勾 dangling 格）；冲突格不可选、知情确认后仍进"冲突"分组且不执行；执行按 `(skillId, action)` 分组串行 `silent=true`、结束后仅一次 `refreshSkills+fetchAgents`；抽屉打开期间行内操作触发后台刷新时"上下文已变化"提示出现；800px 宽窗口抽屉为覆盖层。

### P3 打磨

| # | 任务 | 改动文件 | 要点 |
| --- | --- | --- | --- |
| 1 | 键盘/焦点 | `App.vue`、`SkillsView.vue`、弹层/抽屉 | Esc 关弹层/抽屉、焦点落点、Ctrl+K 聚焦搜索（沿用 `ManageTab.vue:271-282`），对齐 v0.2/18 §15.9 |
| 2 | 空态/错误态 | `SkillsView.vue`、抽屉 | 0 Agent / 0 Skill / 弹层零结果文案 |
| 3 | 边界回归 | — | 0/1/20 Agent × 0/1/1000 Skill |
| 4 | 缩放 | — | 100%/125%/150% 缩放走查 |
| 5 | 最小视口 | `src-tauri/tauri.conf.json` | 增加 `minWidth: 800 / minHeight: 480`（配置级微调，§15 Q5 建议项） |
| 6 | 三语言 | 三份 locale | zh/en/zh-TW 全量走查，重点：动作卡 desc、context_stale、冲突分组文案 |

**验收命令**：`pnpm build` + §14 全清单人工勾选。

## 14. 验收清单

> 来源：v2 §7 回归要求 + §8.6 风险清单 + v0.2/18 边界合同。全部可勾选。

**domain 分区与计数（P1）**
- [ ] 混合副本 skill（`from_plugin && hasNonPluginSource`）在"全部"视图出现且仅出现一次；插件视图只含纯插件副本
- [ ] 任一本地视图列表 + 插件视图列表 = 全量 skill，无交集（互斥分区可加和）
- [ ] "待处理"徽标数 = local 域 `needs_attention` 计数，不吸入插件域冲突项（§3.1）
- [ ] 移除硬排除后，插件视图内 `linked_any` 计数包含 marketplace 来源；`statusPriority` 排序分档与状态点"已同步"一致（§3.2 连带影响 1）
- [ ] 弹层"来源类型"覆盖时计入 `activeFilterCount`、出可移除 Token；切视图后重置为视图默认 domain

**修复分组（P1）**
- [ ] 四组谓词与 `classifySkillSources` 同源；missing_lib/only_project 组计数反映"插件/外部来源不再计入 Agent"的行为变更（§3.4）
- [ ] 问题摘要总数按唯一 skill id 去重，≤ 四组长度之和（v0.2/18 §15.3）
- [ ] 点击冲突/断链/未入库组 = 应用对应筛选，不自动全选、不自动弹批量（v0.2/18 §15.6）
- [ ] 点击"仅项目"组不写 filter state，视图内展示名单（§3.4）

**批量抽屉（P2）**
- [ ] 四张动作卡 ↔ (mode, cellScope) 映射正确；"清理失效链接"默认仅勾 dangling 格
- [ ] 冲突格始终不可选；知情确认后冲突项不执行、进"冲突"分组而非失败明细（§4.1.3/4.1.4）
- [ ] dry-run 的 conflict/blocked 行不依赖勾选全量列出
- [ ] 执行按 `(skillId, 有效动作)` 分组串行、`silent=true`、结束后统一 `refreshSkills()+fetchAgents()`；无逐组刷新（§5.6）
- [ ] `repairContext` 接线：断链组进抽屉预设"清理失效链接"卡；未入库组预设"同步到最新状态"卡（§9.2）
- [ ] 后台刷新/切视图 → rows 重建 + "上下文已变化"提示；勾选不保留（§10.4）
- [ ] 错误串按 `": "` 解析在 agentId 含冒号时错位——已知脆弱点，行为与现状一致，未扩大（§16.3）

**导航与状态（P0）**
- [ ] Ctrl+1 返回上次技能库子视图、Ctrl+2 进历史；子视图不配数字键（§15 Q4）
- [ ] Ctrl+Z/Ctrl+Shift+Z 仅在 `activeView === "history"` 且目标不可编辑时触发（`isEditableTarget` 守卫）
- [ ] `activeView` 统一走 `setActiveView` 写入；刷新后恢复；旧 key `vibe-active-tab` 一次性迁移（§11.2）
- [ ] KeepAlive 保留：技能库子视图间切换筛选/滚动状态不丢（§11.4）
- [ ] 选择作用域：切视图/改筛选后选中集自动 `pruneInvisible`（§5.2）

**视口与语言（P3）**
- [ ] ≥960px：抽屉与列表并存、行内详情可同时展开；800–959px：抽屉覆盖层 + 自动收起行内详情（§15 Q5）
- [ ] 800×480 最小视口无页面级横向滚动；`tauri.conf.json` minWidth/minHeight 生效（若采纳）
- [ ] 筛选 Popover 非模态、Esc/外点关闭、不遮挡 Token 行与批量操作条（§3.3 约束）
- [ ] 100%/125%/150% 缩放、0/1/20 Agent、0/1/1000 Skill 边界走查通过
- [ ] zh/en/zh-TW 三语言无 missing key、无英文硬编码残留；遗留 key `batch_repair_uncovered/only_agent` 已按 §12.3 清理

## 15. 已裁定问题（Round 1 结论，原文保留）

### Q1 侧边栏 Agent 范围 vs 工具栏筛选弹层是否重复

**裁定：重复，只保留筛选弹层；侧边栏不放 Agent 范围控件。**
理由：① `agentIds`/`agentMatch` 是同一份 filter state，双入口必然产生同步负担与不一致风险，违背单一数据源；② Agent 范围带 any/**exclude** 高级语义，不属于"80% 一次点击"的智能视图，属于高级组合，归属弹层；③ 弹层已承载 facet 计数与 Token 移除 UI，侧边栏再放一份是第三处重复展示。侧边栏空间让给视图与计数徽标。若未来证明需要快捷 Agent 过滤，只允许以"写入同一 filter state 的快捷入口"形式回归，不做独立状态控件。

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

## 16. Round 1 新发现风险（v2 §8.6，原文保留）

1. **"仅项目"组无筛选谓词可映射**：四组中 conflict/dangling/missing_lib 可映射现有谓词，`only_project` 不能（v0.2/18 §15.2 明确 `project_only` 不进筛选模型）；现状对该组走"全选+展开第一条"路径，违反 §15.6。裁定见 §3.4：首期视图内名单展示，不写 filter state。
2. **IssueRepairPanel 谓词与 `classifySkillSources` 不同源**：其 `hasAgent` 含 marketplace/external 来源，迁移后 missing_lib/only_project 组计数会变化——是行为变更，需纳入验收（§14"修复分组"首条）。
3. **批量错误解析的字符串耦合**：前端按 `": "` 切分 `"agentId: message"`（`BatchSyncPanel.vue:436-439` ↔ `sync.rs:942`），agentId 含冒号或错误文案含 `": "` 时会错位。抽屉重构沿用该解析即可，但列为已知脆弱点；若后端愿微调为结构化错误则更稳（非必须）。
4. **抽屉常驻后的数据过期**：抽屉打开期间，列表侧单条操作触发的后台 `refreshSkills` 会使抽屉内 `rows` 重建并重置勾选（现状 watch 行为）。§4.2.4 的"上下文已变化"提示即为对策，不做勾选保留。施工实现见 §10.4。
5. **activeTab 持久化名存实亡**：`setActiveTab` 无调用方（`stores/app.ts:62`），`App.vue` 两处直接写 ref；`activeView` 迁移时统一走 setter 并明确持久化策略。施工实现见 §11.2。
6. **KeepAlive 策略**：history 变视图后，`App.vue:127-130` 的 `KeepAlive` 需保留，以维持技能库视图的筛选/滚动状态；`ManageTab` 的 `onMounted` 数据加载（`ManageTab.vue:288-302`）随视图拆分迁移到 `SkillsView`。施工实现见 §11.4。

---

## 17. 生命周期与弃用/禁用架构适配（backlog-ready）

> 依据：`F:\workspace\demo\Skill 的弃用与禁用逻辑.md`（v2.1，2026-07-24）对各工具 Skill/Plugin 启用/禁用机制的调研。
> 定位：v0.3 **只做架构预留**（类型、接口、扩展点、命名空间），功能实现列入 v0.4+ backlog。目标是让"启用/禁用/弃用"落地时**不再需要大范围重构**。

### 17.1 调研结论：禁用语义是按 Agent 异构的

| Agent | 禁用机制 | 粒度 | 持久化位置 |
| --- | --- | --- | --- |
| Claude Code | `enabledPlugins`（4 级 scope：managed>local>project>user）+ `skillOverrides`（on/name-only/user-invocable-only/off，**不适用插件 skill**）+ SKILL.md `disable-model-invocation`/`user-invocable` | 插件级 + skill 级 | `settings.json`（按 scope 分文件） |
| Codex | `[plugins."x@y"] enabled = false` | 插件级 | `config.toml` |
| Hermes | `.usage.json` 的 `<skill>.state`；Curator 自动陈旧（30d）/归档（90d → `.archive/`） | skill 级 | `skills/.usage.json`、`.archive/` |
| Pi / OpenCode / Cursor(skills 层) | 无持久化开关，文件存在即启用 | — | 禁用 = 移除/断链 |

弃用（deprecated）在各工具的 plugin.json 中**均无原生字段**；最接近的生命周期信号是 Hermes Curator 的 stale/archive 与 Codex marketplace 的 `policy.installation: NOT_AVAILABLE`。

### 17.2 架构模型（v0.3 预留）

**① Skill 生命周期枚举**（读取层语义，后端计算给出，前端只读消费）：

```ts
// src/types/index.ts（预留，v0.4 启用）
export type SkillLifecycle =
  | "active"      // 正常（默认，缺省值，保证旧数据兼容）
  | "disabled"    // 被目标 agent 的配置禁用
  | "stale"       // 长期未用/来源长期未更新（阈值可配，默认 90 天，对齐 Hermes archive_after_days）
  | "archived"    // 已归档，不在任何 agent 生效
  | "deprecated"; // 来源方或本库声明弃用
```

**② Agent 禁用能力模型**（声明式，决定"该 agent 能不能禁用、怎么禁用"）：

```ts
// src/types/index.ts（预留）
export interface DisableCapability {
  kind: "none" | "skill_state_file" | "plugin_config" | "skill_config" | "unlink_only";
  scope: "skill" | "plugin";
  // claude-code: plugin_config+skill_config（scope 双制）; codex: plugin_config;
  // hermes: skill_state_file; pi/opencode/cursor: unlink_only
}
```

后端 `models/agent.rs` 的 `Agent` 预留 `disable_capability`（serde `Option`，缺省 `unlink_only`——与现状"断链即禁用"语义一致，零行为变更）。

**③ 数据流扩展点（全部 serde 可选、向后兼容，不破坏现有命令合同）**：

- `models/skill.rs`：`SkillSource` 增加 `lifecycle: Option<String>`；`Skill` 增加派生 `lifecycle`（默认 `active`）。
- 新 `utils/lifecycle.rs`（backlog 实现）：按 agent capability 读取 `enabledPlugins` / `config.toml` / `.usage.json`，带**缓存 + 并发 + 失败静默**（配置文件缺失/解析失败 = 全部 active，绝不让探测失败阻塞 `list_skills`）。
- stale 判定数据源：`skill.modified_at`、`SkillOrigin.last_checked_at`、Hermes `.usage.json.last_used_at`；`deprecated` 写入 `.vibe-origin.json` 扩展字段 `{ deprecated: { since, replacement?, note? } }`（复用既有 origin sidecar 机制，无新存储）。
- 归档动作复用现有 `.trash/` 通路；v0.4 可引入 `~/.vibe-skills/.archive/` 对齐 Hermes 语义（断开所有链接 + 移档 + 记历史）。

### 17.3 前端预留点（v0.3 施工时一并埋好）

1. **筛选模型**：`ManageFilterState` 预留可选字段 `lifecycle?: Set<"disabled" | "stale">`（缺省不限制、不进 `activeFilterCount`）；`filterSkills` 内部按"字段不存在即放行"处理——v0.4 启用时**不改函数签名**，只在弹层"状态"组加 chip。
2. **视图模型**：`SmartViewId` 不新增"已禁用"视图（避免视图爆炸）；`useSmartViews` 的 `SmartViewDef` 结构不变，v0.4 若确需独立视图走注册表追加即可（注册表本就是单一真源）。
3. **列表展示**：`SkillRow` 徽标位（现有 conflict/dangling/duplicate/plugin 徽标同一插槽区）预留 `lifecycle` 徽标渲染分支，默认 `active` 不渲染。
4. **动作注册表**：`skillActionRegistry` 预留 `disable` / `enable` 两个 `AgentAction` 槽位（`mutatesLibrary: false`、`removesTarget: false`），v0.3 不出现在任何 UI；`useBatchCellActions` 的 `applySwitch` 不变。
5. **i18n**：预留命名空间 `lifecycle.*`（`lifecycle.disabled/stale/archived/deprecated`），v0.3 仅建 key 不引用。
6. **插件视图**：Claude/Codex 的插件级禁用状态（读取层）落地后，插件分组卡预留"已禁用"置灰样式，不改分组数据结构。

### 17.4 不做的事（防止过度设计）

- v0.3 不实现任何配置文件的**写入**（enabledPlugins/config.toml/.usage.json 写入属 v0.4+，需先解决 Claude 4 级 scope 优先级与 managed 只读策略）。
- 不做自动陈旧清理（Hermes Curator 式自动化不做；stale 只做标记与归档建议）。
- 不为 Pi/OpenCode 发明"禁用"概念——它们的禁用就是断链，复用现有 `unlink`。

## 18. 跨平台适配（Windows / macOS）

> 现状：开发与验收主要在 Windows。v0.3 布局与交互重构必须一次把双平台差异埋好，避免 P3 后返工。

### 18.1 路径与链接层

1. **symlink 能力差异**：Windows 需开发者模式或管理员（`AGENTS.md` 已载），失败时 `create_symlink_with_report` 已返回 `mode`（symlink/junction/copy）与 warning——v0.3 的批量抽屉步骤 3 与单条动作结果必须**继续透传该 mode/warning**（现状 dry-run 已有 warnings 通道，沿用）；mac 原生 symlink 无限制，预期 `mode = "symlink"`。验收必须包含"Windows 无开发者模式"场景下的 junction/copy 兜底文案。
2. **路径比较**：`samePath`/`normalizePath`（`useSkillAgentStatus.ts:295-301`、`manageFilters.ts:66-68`）统一 `/` 分隔 + 去尾斜杠；Windows 需额外 **大小写不敏感**（盘符与路径），mac 默认 APFS 同样大小写不敏感——裁定：比较一律 `toLocaleLowerCase()`（`manageFilters.ts:67` 现状即如此，`useSkillAgentStatus.ts` 的版本未做小写化，P1 顺手对齐，属缺陷修正）；展示层保留原始路径。
3. **Windows MAX_PATH(260)**：深层 skill 目录 + 长名称下复制/链接可能失败——错误文案需给出"路径过长"指引（沿用 VibeError 字符串透传，不新增错误类型）。
4. **目录大小写重名**：mac/Win 上 `Foo/` 与 `foo/` 视为同一目录，扫描去重（按文件夹名）在 Linux 开发机上可能表现不同——列为已知差异，不处理。

### 18.2 交互层

1. **快捷键 Ctrl vs Cmd**：全局判定统一 `e.ctrlKey || e.metaKey`（`ManageTab.vue:272` 已有先例），P0 的 `Ctrl+1/2`、`Ctrl+Z` 迁移与 P3 的 `Ctrl/Cmd+K` 全部走同一守卫函数；提示文案按平台渲染 `Ctrl`/`⌘`（`@tauri-apps/plugin-os` 或 `navigator.platform` 判定，集中一个 `usePlatform()` composable）。
2. **标题栏与窗口控制**：自绘 min/max/close 按钮（`AppLayout.vue:79-88`）在 mac 上与 traffic lights 冲突——裁定：mac 下左侧品牌区 `padding-left` 避让（约 72px），自绘按钮保留（Tauri 无原生最大化禁用差异问题）；`tauri.conf.json` 评估 `titleBarStyle: "Overlay"`（P3 实测后定）。
3. **右键菜单抑制**：`App.vue` 的 `handleGlobalContextMenu` 双平台行为一致，无需分支。
4. **滚动条样式**：mac 覆盖式滚动条不占宽度，抽屉/弹层布局不得依赖滚动条宽度计算；矩阵 sticky 列双平台走查。

### 18.3 Agent 默认路径差异

Hermes：Windows `%LOCALAPPDATA%\hermes\skills` vs mac `~/.hermes/skills`（后端 `agents` 配置已处理）；插件市场缓存路径同理。P3 验收矩阵覆盖双平台的 agent 探测结果。

### 18.4 验收矩阵（并入 §14 P3）

- 平台：Windows 11、macOS（arm64 + 尽可能 x86_64）。
- 每平台走查：symlink/junction/copy 三 mode、Ctrl/Cmd 快捷键、最小视口 800×480、125%/150%（Win）与 1x/2x（mac）缩放、深浅主题。

## 19. 性能预算（查询 / 刷新 / 执行 / 渲染）

> 原则：先定预算与测量点，超标再优化；优化不得破坏 §5 不变量（判定同源、单次统一刷新）。

### 19.1 查询效率（`list_skills`）

- 现状：递归扫描（MAX_SCAN_DEPTH=12）+ `.hash-cache` 哈希缓存。
- 预算：1000 skills / 20 agents 下，冷扫描 < 3s；缓存命中刷新 < 500ms。
- §17 的 lifecycle 探测（backlog）必须并发读取 + 结果缓存 + 失败静默，不得串行阻塞主扫描；探测结果并入 hash 缓存失效策略（agent 配置文件 mtime 变化即失效）。

### 19.2 刷新效率（前端计算）

- 现状：`filterSkills`/`computeFacetCounts` 每次 state 变化全量 O(n×g)；`refreshSkills` 有 `refreshRequestId` 竞态保护（符合 v0.2/18 §15.4）。
- 预算：1000 skills 下单次过滤 + facet 计算 < 16ms（一帧）；实测超标时按序启用：① 按 domain 分桶预聚合（domain 是互斥分区，天然可分桶）；② 非激活视图的 `viewCounts` 惰性计算（当前视图精确算，其余视图用上一次值 + 数据版本戳标记）；③ Web Worker。
- 预留（backlog）：后端增量刷新 `refresh_skill(skill_id)`，批量/单条操作后只重扫受影响 skill，避免全量扫描——v0.3 前端刷新通路（统一 `refreshSkills()`）不变，未来接入点唯一。

### 19.3 执行效率（批量）

- 合同不变：按 `(skillId, 有效动作)` 分组**串行**调用 `batchSkillAction(silent=true)`，结束统一刷新（§10.3）。串行是历史记录精确性与文件系统争用之间的保守选择。
- 预算：单组 < 2s（本地 IO）；执行中禁用重复触发（沿用 `operating` ref）+ 步骤 3 显示进度 `已完成 x/y 组`。
- 预留（backlog）：后端 `batch_skill_action` 内部对 `agent_ids` 循环可并行化——v0.3 的 dry-run/结果模型已按 agent 粒度建模，未来并行不需要改前端结构。
- 取消能力：backlog（需后端命令级支持，不在本期）。

### 19.4 渲染效率

- 列表：>300 项启用虚拟滚动（沿用 v0.2/18"先测量后虚拟列表"的裁定，P3 以 1000 skills 实测为准）；插件分组视图同样适用。
- 矩阵：单元格 = skills × agents，预算 1000 格内直接渲染（50×20）；超出时步骤 1 提示"请缩小选择范围"。
- `viewCounts`：7 视图 × facet 全量计算每次 state 变化都跑——按 §19.2 的分桶 + 惰性策略控制。
- 长列表展开详情：KeepAlive 下保留滚动位置（§11.4），不重挂整列表。

## 20. 决策记录（2026-07-24 定案，代用户决定）

| # | 问题 | 定案 | 落地阶段 |
| --- | --- | --- | --- |
| 1 | 遗留 i18n key `batch_repair_uncovered/only_agent` | **删除**。无触发方（`repairContext` 恒 null）；P2 接通修复上下文时启用新的 `batch.repair_conflict/dangling/missing_lib` 系列（§12.1） | P2 |
| 2 | `tauri.conf.json` 是否加最小窗口 | **加**：`minWidth: 800, minHeight: 480`，把 v0.2/18 §15.8 的最低视口从设计目标变为强制约束 | P0 |
| 3 | 侧边栏分组命名 | **新增 `sidebar.*` 全套 key**（含 `sidebar.skills_library`="技能库"），不复用语义不符的 `tabs.manage`；`tabs.*` 随 TabBar 删除一并清理 | P0 |
| 4 | 是否补 `project_only` 筛选谓词 | **首期不做**，"仅项目"组在待处理视图内名单展示（§3.4 裁定维持）；补谓词进 v0.4 backlog，与 lifecycle 维度一同评估 | backlog |

补充定案（本轮新增）：

| # | 问题 | 定案 |
| --- | --- | --- |
| 5 | 弃用/禁用功能范围 | v0.3 只做 §17 架构预留（类型/接口/扩展点/i18n 命名空间）；读取层探测与写入层操作全部进 v0.4+ backlog |
| 6 | `useSkillAgentStatus.ts` 的 `normalizePath` 未小写化 | 与 `manageFilters.ts` 对齐补 `toLocaleLowerCase()`，P1 顺手修（缺陷修正，非行为变更） |
