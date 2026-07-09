# 多选 Skill 同步交互优化方案（最终版）

> 版本：v1.0（经两轮子 agent 审查优化，implementation-ready，代码已按本方案实现并验证通过）
> 日期：2026-07-09
> 状态：已实施

> 版本沿革（本文件 = 最终 v3 内容）：
> - 初稿 v1：矩阵面板方案 + 四种场景。
> - 子 agent 第 1 轮（v2）：纠正 `synced→unlink` 同源问题、正面处理「真实文件在别的 agent」的单元格（标为「需先入库」而非静默失败）、诚实化冲突处理（`sync_to_vibe` 无 overwrite 参数，"确认即覆盖"不成立）。
> - 子 agent 第 2 轮（v3）：补充 §11「实施规格（implementation-ready）」、把术语改为用户 10 秒可读、消除 v2 一处自相矛盾（冲突项统一为"诚实失败"）、§10 风险项（尤其 R1）明确"不在本期范围内"。
> - 本文件以 v3 为权威正文；标题统一为 v0.1 文档系的 v1.0。

---

## 1. 背景与问题描述（一句话版）

用户勾选多个 skill 点「同步到库」，有时会"什么都没发生"，有时会"失败几个且说不清为什么"。根因是旧逻辑**替用户猜目标 agent**、且**出错没有明细**。

### 1.1 旧实现（`ManageTab.batchSyncSelected`）

- 自己推断目标：只挑每个 skill 里"不是软链接、且来源不是 vibe-lib"的 agent；
- 对每个 skill 调 `batch_skill_action(skillId, agentIds, "sync_to_vibe", true)`；
- 用户**完全没法自己选**要对哪些 agent 做什么。

### 1.2 失败根因

1. **乱猜目标 → 静默跳过**：若某 skill 没有可被推断的 agent（比如只在库里、或已同步、或真实文件在别的 agent 而没被猜中），`agentIds` 为空，该 skill 被静默跳过——用户以为同步了，其实啥也没干。
2. **同名 skill 内容冲突**：同一 skill 在 agent A、B 里是不同内容的真实文件。`sync_to_vibe` 对第二个 agent 触发 `Conflict`（库里已有且 hash 不一致），批量里出现"失败 N 个"。更糟的是：A 同步成功后变成软链接，重试时被排除，B 永远冲突 → "反复失败且无法恢复"。
3. **无选择、无预览、无明细**：用户不知道会对谁做什么，出错后没有逐条结果。

> v2/v3 修正：第 2 点的"反复失败"深层原因是 `sync_to_vibe` **不会**因用户确认而覆盖，它始终返回 `Conflict`（见 §4.5）。所以"确认即覆盖"在现有命令集下不成立，冲突单元格必须有一个**不会假装成功**的归宿。

---

## 2. 设计目标

- **只改交互，不新增 Rust 命令**（复用 `batch_skill_action` / `batch_link` / `batch_unlink` / `sync_to_vibe` / `create_link` / `relink` 等已注册命令）。
- 覆盖四种场景（§3），交互统一、简单、可预期。
- 杜绝"静默跳过"与"无明细失败"；冲突显式提示、**可恢复（有清晰指引）**。
- 不破坏现有单 skill 交互。
- **判定同源**：矩阵面板每个单元格的建议动作必须与 `useSkillAgentStatus` 完全一致，不出现"一处能同步、另一处说已同步"的矛盾。

---

## 3. 四种场景（一张表看懂）

| 场景 | 通俗说法 | 现状 |
|------|----------|------|
| 单独同步 | 1 个 skill → 它该做的动作 | 已支持 |
| 单个 skill 多 agent | 1 个 skill → 多个 agent | 已支持 |
| 多个 skill 单 agent | N 个 skill → 1 个 agent | **新增（矩阵面板）** |
| 多个 skill 多 agent | N 个 skill → N 个 agent（可分别指定） | **新增（矩阵面板）** |

> 重要："多个 skill 单 agent"**不是**"N 个 skill 对同一个 agent 做同一动作"的简化版。同一列（目标 agent）下，不同 skill 的状态可能完全不同：有的已链接、有的冲突、有的压根还不在库里。矩阵必须**逐格**判定（见 §4.3、§4.3.1）。

---

## 4. 交互方案：批量同步矩阵面板（核心）

用**「操作矩阵」**替代原来的浮动"同步到库"按钮，让"选 skill × 选 agent → 看预览 → 执行"成为统一入口。

### 4.1 触发

- 勾选 `selectedSkills.size > 0` 时，底部浮动栏出现「批量操作」按钮（替代原"同步到库"）。
- 点击打开 `BatchSyncPanel`（模态/抽屉），标题显示已选 skill 数。

### 4.2 面板结构（四步，10 秒上手）

普通用户按这四步操作，面板 UI 直接对应：

1. **选 skill**（左栏）：已选 skill 列表，可取消勾选某一项。
2. **选 agent**（右栏）：列出所有 detected agent，可多选；提供「全选 / 仅库有者」快捷键。
3. **看预览**（中栏矩阵 + 底部摘要）：行 = skill，列 = agent。每格显示该 (skill, agent) 的"建议动作"（见 §4.3），不同动作用颜色区分；底部实时显示"将执行 X 项操作，涉及 Y 个 skill、Z 个 agent；其中 M 项冲突；K 项不可操作（需先入库）"。
4. **确认执行**：点「执行」，若有冲突先弹一次风险提示（见 §4.5），再统一执行，最后给出成功/失败明细。

- **动作类型总开关**（可选，默认「同步到库」）：「同步到库 / 仅链接 / 取消链接」三种，切换时每格的"有效动作"按 §4.4 刷新。不切也能用——默认「同步到库」已覆盖绝大多数诉求。
- **勾选粒度**：单元格可单独勾选，支持「整行选」「整列选」「全选」。

### 4.3 单元格动作判定（与 `useSkillAgentStatus` 严格同源）

矩阵每个 (skill, agent) 单元格的状态与动作，**逐字复用** `useSkillAgentStatus.ts`（第 90–183 行）。下表是其完整映射，是矩阵判定的唯一事实来源：

| 单元格状态 | 判定条件（来自源码） | 建议动作 |
|---|---|---|
| `origin` | 该 agent 即技能库自身（`source.from === "vibe-lib"`） | `none` |
| `synced` | 软链接且指向 vibe-lib | **`unlink`** |
| `linked_elsewhere` | 软链接但指向非 vibe-lib | `relink` |
| `independent`（有 `vibeSource`，hash 相同） | 真实文件，库有且一致 | `replace_with_link` |
| `independent`（有 `vibeSource`，hash 不同） | 真实文件，库有但不一致 | `sync_to_vibe`（**冲突**） |
| `independent`（无 `vibeSource`） | 真实文件，库里没有 | `sync_to_vibe`（**首次入库**） |
| `dangling` | 软链接失效/指向空 | `remove_dangling` |
| `unlinked`（有 `vibeSource`） | agent 无该 skill，但库里有 | `link` |
| `unlinked`（无 `vibeSource`） | agent 无该 skill，库里也没有 | **`none`（标「需先入库」）** |

> **修正点 A**：v1 把 `synced` 错写成 `none`。源码实际返回 `unlink`（`:169`）。矩阵若当 `none` 会与其行内按钮"取消链接"矛盾，违反"判定同源"。v3 统一为 `unlink`。
>
> **修正点 B**：v1 把"无 source"笼统归 `none`。源码在 `unlinked` 分支按 `vibeSource` 是否存在区分 `link` 与 `none`（`:100`）。v3 明确上表最后两行，并把"无 `vibeSource` 的 `unlinked`"定性为**「需先入库」**。

### 4.3.1 重点场景：「多个 skill 单 agent」中的「需先入库」单元格

**问题**：某被选中 skill 的真实文件在「另一个 agent（如 B）」而非目标 agent（如 A）时，矩阵这一格怎么处理？

它既不能用 `link`（`create_link` 要求 skill 已在 vibe-lib，否则 `SkillNotFound`），也不能用 `sync_to_vibe` 到 A（A 目录里没有真实文件，`sync_to_vibe` 在 `source_path` 不存在时返回 `SkillNotFound`）。

**结论**：该格 = `unlinked` + 无 `vibeSource` → 动作 **`none`**，标记为 **「不可操作 / 需先入库」**（置灰、不可勾选，hover 提示："该 skill 尚未入库，且目标 agent 内无真实文件；请先在某 agent 将其同步入库，再回来链接到本 agent"）。

**必须显式可见**（对应审查重点）：这种格不能在执行阶段才因命令报错"悄悄没做"。预览摘要将其计入 **"K 项不可操作（需先入库）"**，让用户执行前就知道这些格不会、也不该被执行。执行时它们不进入任何 `batch_skill_action` 调用。

**用户如何真正完成它（闭环工作流，不新增命令）**：
1. 先勾选持有真实文件的那个 agent（如 B）对应的格——状态 `independent`（无 `vibeSource`）→ 动作 `sync_to_vibe`（首次入库）。执行后 skill 进入 vibe-lib。
2. 执行后**自动刷新全局状态**（§7 I3）。再看 (skill, A) 格：因 `vibeSource` 已存在，状态变为 `unlinked`（有 `vibeSource`）→ 动作 `link`，即可勾选链接到 A。
3. 再次执行 (skill, A) 的 `link`。

> 矩阵只需保证：① 未入库格**清晰标记不可操作**；② 执行后**刷新**使格状态随之更新。

### 4.4 总开关与单元格关系（有效动作覆盖规则）

每格**基础动作**由 §4.3 决定；「动作类型总开关」在此之上决定**哪些格可勾选、有效动作是什么**。注意：总开关只改"可勾选性"与"有效动作文本"，**不改基础状态判定**，也不让 `none`/`origin`/「需先入库」变成可操作。

- **「同步到库」（默认）**：
  - 可勾选：`link` / `sync_to_vibe`（首次入库与冲突）/ `replace_with_link` / `relink` / `remove_dangling` 单元格。
  - 不可勾选（`none`）：`origin`、`synced`（已是最新链接，无需再同步）、「需先入库」。
  - 说明：`synced` 基础动作是 `unlink`，但「同步到库」语义下不应被自动勾选；想取消请切「取消链接」。
- **「仅链接」**：
  - 仅 `unlinked` + 有 `vibeSource`（基础动作 `link`）可勾选；其余置灰。
  - 天然保证被链接的 skill 已在 vibe-lib（`create_link` 要求），因为只有"有 `vibeSource`"的 `unlinked` 才会被赋 `link`。
- **「取消链接」**：
  - 有效动作**统一覆写为 `unlink`**；可勾选 = `synced` 与 `linked_elsewhere`（二者底层都是"指向某处的软链接"，`remove_link` 可移除）。
  - 不可勾选：`origin`、「需先入库」、以及基础动作为 `link`/`sync_to_vibe`/`relink`/`remove_dangling` 的格。
  - 注意：`linked_elsewhere` 基础动作是 `relink`，在「取消链接」下覆写为 `unlink`——这是**唯一一处有意覆盖基础动作**，因为用户意图是"去掉这个链接"而非"改指 vibe-lib"。`batch_skill_action("unlink")` → `remove_link` 对任意软链接均合法（审查重点③已确认无串味风险）。

### 4.5 冲突处理（诚实、可恢复）

**v3 修正（消除 v2 自相矛盾）**：v2 §4.5 同时写了"确认后只执行非冲突项、冲突项跳过"和"冲突单元格走 `sync_to_vibe` 返回 Conflict 进失败明细"——二者冲突，且与"冲突项如实进失败明细"的已确认事实不符。v3 统一为**诚实失败**一条路径（见下）。

- **冲突定义**：`sync_to_vibe` 且 hash 不一致（即 `independent` + 有 `vibeSource` 且 hash 不同）。`sync_to_vibe` 在 hash 不一致时**返回 `Conflict` 且无 overwrite/force 参数**，故"确认即覆盖"不成立。
- **显式标记**：冲突格在矩阵中红色高亮，并计入预览摘要"M 项为冲突"。
- **非冲突项始终成功**：执行按 (skillId, 有效动作) 分组调 `batch_skill_action`，每个 agent 独立处理，单个 `Conflict` 只进该 skill 该 agent 的 `errors[]`，**不会连累同批其它项**。
- **执行前确认（风险提示，不改结果）**：若勾选了任意冲突格，弹一次确认："这些 skill 在 agent 中的内容与技能库不同，执行会以失败（Conflict）告终，建议先解决冲突，或取消勾选这些冲突项后再执行。"
  - 用户仍确认 → 执行**全部勾选项**，冲突项如实返回 `Conflict` 进失败明细（**不假装成功**）；用户也可在确认前手动取消勾选冲突项以避免失败。
  - 用户取消 → 什么都不执行。
- **冲突项不假装成功**：执行后冲突格走 `sync_to_vibe` 返回 `Conflict`，进失败明细，格式 `"<agent_id>: Conflict(...)"`，面板据此把 `agent_id` 解析回对应格高亮。
- **恢复指引（现有命令下可行的闭环路径）**：
  1. **改用库版本**：单 skill 冲突入口（SkillRow）选"使用库版本"（该 path 仍需底层支持，见 §10 R4）；或手动保留 vibe-lib 现有版本，对持有旧版本的 agent 执行 `unlink` 让其变回独立副本。
  2. **用某 agent 版本覆盖库**：先 `delete_skill(skill_id)` 删库副本并移除所有 agent 的软链接（备份至 `.trash`），再 `sync_to_vibe(skill_id, 该 agent)` 重新入库。此序列会动到其它 agent 的链接，**矩阵不自动执行**，仅作手动指引。

> 结论：冲突的"非冲突成功 + 失败明细 + 可重试"已闭环；"一键确认覆盖"需 `sync_to_vibe` 增 overwrite 参数或新 force 命令——**不在本期范围内**（§10 R1）。

---

## 5. 执行逻辑（与现有命令映射，不加功能）

执行前，将勾选的单元格按 **(skillId, 有效动作)** 分组，然后：

- 对每个 `skillId`，按 `action` 归类其 agentIds，调用 `batch_skill_action(skillId, agentIds, action)` 一次（已是"单 skill 多 agent"标准通道）。
  - `action` 恒为 `link` / `unlink` / `sync_to_vibe` / `replace_with_link` / `relink` / `remove_dangling` 之一，均为 `batch_skill_action` 已支持的字符串，**无遗漏、无不可执行组合**。
  - `replace_with_link` 在 Rust 端映射到 `sync_to_vibe`，且只会在 hash 一致时出现，不会触发 Conflict。
- **"多个 skill 多 agent"** = 多行"单 skill 多 agent"组合，循环各行即可。
- **"多个 skill 单 agent"**：等价于矩阵只勾选某一 agent 列的可操作格，自然落到分组执行；"需先入库"格不进分组（§4.3.1）。
- 全程 `silent = true`（即 `batchSkillAction(..., true)`），**全部批次完成后**最后一次统一 `refreshSkills()` + `useAgentsStore().fetchAgents()`。

每项返回 `SyncResult { synced_count, errors }`，前端汇总：

- 成功项总数 / 失败项总数；
- 失败明细（解析 `errors[]` 中 `agent_id` → 定位到具体 (skill, agent) 格并高亮），在结果 Toast / 面板内展开，**不自动清除选择**（便于调整重试）。
- 不可操作（需先入库）格数在摘要中单列，不参与执行计数。

> **混合动作拆分（审查重点③一致性校验）**：当同一 skill 一行里混合动作（如 A 列 `link`、B 列 `sync_to_vibe`），分组的 key 为 `(skillId, 有效动作)`，因此自然拆成两个 `batch_skill_action` 调用——`(skillId, link)→[A]` 与 `(skillId, sync_to_vibe)→[B]`——互不影响、不串味。详见 §11.4。

---

## 6. 与现有交互的兼容（不破坏整体功能）

- **保留项（完全不动）**：卡片主操作；`SkillRow` 行内单点操作；单个 skill 多 agent 批量（底层 `batchSkillAction`）。
- **新增项**：矩阵面板（`BatchSyncPanel`），仅作"多选跨 skill"补充入口。
- **判定同源保证**：矩阵面板**复用** `useSkillAgentStatus` 计算每格状态/动作（与 `SkillRow` 同一 composable），保证单 skill 与多 skill 的"建议动作"完全一致。
- **不改动 Rust**；矩阵用到的 action 均在 `batch_skill_action` 已支持范围内。

---

## 7. 不变量 / 回归保护

- **I1**：`sync_to_vibe` 对已存在同名 skill 在 hash 一致时自动"替换链接"，不重复复制——矩阵不为已 `synced` 的 (skill, agent) 生成新 `sync_to_vibe`（其动作是 `unlink`，§4.3）。
- **I2**：`none`（含 `origin` 与「需先入库」）格**永不执行**，避免误调命令。
- **I3**：每次打开面板、及每次执行批次完成后都 `refreshSkills()` + `fetchAgents()`，确保矩阵与实际文件系统一致（使"先入库再链接"工作流能刷新出新 `link` 格）。
- **I4**：批量执行只调已注册命令，不调未注册命令；宏动作如"用 agent 版本覆盖库"因涉及 `delete_skill` 多步影响，不在矩阵自动执行范围（仅作指引）。
- **I5**：冲突格**不会**被伪造成成功——它们要么被用户取消勾选，要么执行后如实进失败明细。

---

## 8. 验收标准

1. 单独同步：卡片主操作可正常同步/链接一个 skill。
2. 单个 skill 多 agent：SkillRow 内勾选多个 agent 批量应用成功。
3. 多个 skill 单 agent：选多 skill + 面板勾选单 agent 列**可操作**格 → 该 agent 下这些 skill 全部链接/同步成功；"需先入库"格显式置灰且计入摘要，不被静默跳过。
4. 多个 skill 多 agent：矩阵跨多行多列勾选 → 按预览执行成功；冲突项提示并可确认/取消勾选。
5. 无"静默跳过"：预览摘要实时反映将执行数、跳过 `none` 数、不可操作（需先入库）数、冲突数。
6. 失败有明细：个别 (skill, agent) 出错（含 Conflict）给出"skill × agent × 原因"明细且保留选择可重试。
7. 判定同源：矩阵格动作与 SkillRow 行内动作对同一 (skill, agent) 完全一致（重点核对 `synced→unlink`、`unlinked→link/none`）。
8. 混合动作不串味：同一 skill 一行内不同动作正确拆成独立 `batch_skill_action` 调用（§5、§11.4）。
9. `pnpm build` 与 `cargo check` / `cargo test` 全部通过（文档不改 Rust，Rust 侧本应通过）。

---

## 9. 实施步骤（文件改动，仅前端）

- 新增 `src/components/manage/BatchSyncPanel.vue`（操作矩阵面板；单元格判定复用 `useSkillAgentStatus`）。
- 修改 `src/components/manage/ManageTab.vue`：
  - 浮动栏「同步到库」改为「批量操作」，打开面板；
  - 移除 `batchSyncSelected` 的直接推断逻辑（改由面板负责）；
  - 将 `selectedSkills`、agents、skills 传入面板。
- 复用：`src/composables/useSkillAgentStatus.ts`（单元格判定）、`src/stores/skills.ts#batchSkillAction`。
- i18n：在 `zh.json` / `en.json` / `zh-TW.json` 增加 `manage.batch_panel.*` 系列键（含「需先入库」「不可操作」「冲突将失败」等）。
- **不改动 Rust**。

---

## 10. 风险与遗留项（明确范围）

- **R1（首要）：冲突"一键覆盖"未闭环。** 现有 `sync_to_vibe` 无 overwrite 参数（返回 `Conflict`）。本方案保证"非冲突成功 + 失败明细 + 可重试"，但"确认即以某 agent 版本覆盖库"需 `sync_to_vibe` 增 `overwrite: bool` 形参或新增 `force_sync_to_vibe` 命令——**均超出"只改前端"范围，不在本期范围内**。建议作为后续独立任务评估。
- **R2：「需先入库」格依赖用户两步操作**（先对持有真实文件的 agent 入库，刷新后再链接目标 agent）。命令集约束下的必然设计，已在 §4.3.1 给工作流；UX 上可考虑面板内"一键先入库再链接"编排（仍只调现有命令），列为可选增强。
- **R3：`delete_skill` 影响面。** §4.5 恢复指引 2 用到 `delete_skill`，会移除所有 agent 对该 skill 的软链接（虽备份至 `.trash`）。矩阵不自动执行此序列，仅作手动指引，故不会误伤；若未来做自动化覆盖需先评估此影响面。
- **R4：单 skill 冲突入口覆盖能力同样受限。** `SkillRow.useThisVersion` 当前也调 `sync_to_vibe`，hash 冲突时同样返回 `Conflict`（与 R1 同源）。修复 R1 后两端同时受益。

---

## 11. 实施规格（implementation-ready）

> 本节为可直接照写的代码骨架。字段/函数签名以 TS 风格给出，命名与现有 codebase 保持一致（`batchSkillAction`、`useSkillAgentStatus`、`useAgentsStore`、`refreshSkills`）。

### 11.1 `BatchSyncPanel.vue` 的 props / emits

```ts
// props：父组件（ManageTab）已持有的数据直接传入，面板不做数据获取
const props = defineProps<{
  skills: Skill[]            // 已选中的 skill 列表（ManageTab 的 selectedSkills）
  agents: Agent[]            // 所有 detected agent（ManageTab 从 useAgentsStore 取）
  selectedSkillIds: string[] // 与 skills 对应的 id 集合，便于去重/对照
}>()

// emits：只回传"关闭"与"执行结果"，具体调度在面板内部完成
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'applied', payload: {
    syncedCount: number
    errors: Array<{ skillId: string; agentId: string; message: string }>
  }): void
}>()
```

### 11.2 单元格数据模型 `Cell`

```ts
type AgentStatusType =
  | 'origin' | 'synced' | 'linked_elsewhere'
  | 'independent' | 'dangling' | 'unlinked'

type AgentAction =
  | 'none' | 'link' | 'unlink' | 'sync_to_vibe'
  | 'replace_with_link' | 'relink' | 'remove_dangling'

interface Cell {
  skillId: string
  agentId: string
  status: AgentStatusType      // useSkillAgentStatus 原始状态（§4.3）
  action: AgentAction          // useSkillAgentStatus 原始建议动作（§4.3）
  effectiveAction: AgentAction // §4.4 总开关覆写后的"可执行动作"；用于执行分组
  selectable: boolean          // 是否可勾选（§4.4 规则）
  isConflict: boolean          // sync_to_vibe 且 hash 不一致（§4.5）
  needsImport: boolean         // unlinked 且无 vibeSource（§4.3.1「需先入库」）
}
```

> `effectiveAction` 是执行分组的关键：默认「同步到库」下等于 `action`；「取消链接」下 `linked_elsewhere`/`synced` 被覆写为 `unlink`；`none`/「需先入库」格的 `effectiveAction` 恒为 `none` 且不进分组。

### 11.3 矩阵计算伪代码

```ts
type SwitchMode = 'sync_to_vibe' | 'link_only' | 'unlink_only'

function buildMatrix(
  skills: Skill[], agents: Agent[], mode: SwitchMode
): Cell[][] {
  return skills.map(skill =>
    agents.map(agent => {
      const { status, action } = useSkillAgentStatus(skill, agent)
      const hasVibe = !!skill.vibeSource                 // 库里是否已有该 skill
      const hashDiff = skill.hash !== agent.localHash     // 真实文件与库版本是否不同
      const isConflict =
        status === 'independent' && hasVibe && hashDiff    // sync_to_vibe 且 hash 不一致
      const needsImport = status === 'unlinked' && !hasVibe
      const { effectiveAction, selectable } =
        applySwitch(mode, status, action, isConflict, needsImport)
      return { skillId: skill.id, agentId: agent.id, status,
               action, effectiveAction, selectable, isConflict, needsImport }
    })
  )
}

// §4.4 总开关覆盖规则
function applySwitch(
  mode: SwitchMode, status: AgentStatusType,
  action: AgentAction, isConflict: boolean, needsImport: boolean
): { effectiveAction: AgentAction; selectable: boolean } {
  if (mode === 'sync_to_vibe') {            // 默认「同步到库」
    if (status === 'origin' || status === 'synced' || needsImport)
      return { effectiveAction: 'none', selectable: false }
    return { effectiveAction: action, selectable: true }
  }
  if (mode === 'link_only') {               // 「仅链接」
    if (status === 'unlinked' && !needsImport)
      return { effectiveAction: 'link', selectable: true }
    return { effectiveAction: 'none', selectable: false }
  }
  // mode === 'unlink_only' 「取消链接」
  if (status === 'synced' || status === 'linked_elsewhere')
    return { effectiveAction: 'unlink', selectable: true }  // 覆写 relink→unlink
  return { effectiveAction: 'none', selectable: false }
}
```

### 11.4 执行伪代码（含混合动作拆分校验）

```ts
async function execute(selectedCells: Cell[]) {
  // 1) 按 (skillId, effectiveAction) 分组；跳过 effectiveAction==='none' 与不可勾选格
  const groups = new Map<string, { skillId: string; action: AgentAction; agentIds: string[] }>()
  for (const cell of selectedCells) {
    if (!cell.selectable || cell.effectiveAction === 'none') continue
    const key = `${cell.skillId}::${cell.effectiveAction}`   // 关键：key 含有效动作
    if (!groups.has(key))
      groups.set(key, { skillId: cell.skillId, action: cell.effectiveAction, agentIds: [] })
    groups.get(key)!.agentIds.push(cell.agentId)
  }

  // 2) 每组一次 batch_skill_action；冲突项如实进 errors，不串味
  let totalSynced = 0
  const allErrors: Array<{ skillId: string; agentId: string; message: string }> = []
  for (const g of groups.values()) {
    const res = await batchSkillAction(g.skillId, g.agentIds, g.action, true)
    totalSynced += res.synced_count
    for (const err of res.errors) {
      allErrors.push({ skillId: g.skillId, agentId: err.agent_id, message: err.message })
    }
  }

  // 3) 全部完成后统一刷新，使"先入库再链接"工作流能刷新出新状态
  await refreshSkills()
  await useAgentsStore().fetchAgents()

  // 4) 回传明细（把 agent_id 解析回具体格高亮由面板负责）
  emit('applied', { syncedCount: totalSynced, errors: allErrors })
}
```

> **混合动作拆分证明**：同一 skill 一行内 A 列 `link`、B 列 `sync_to_vibe` → 生成 key `(skill,link)→[A]` 与 `(skill,sync_to_vibe)→[B]` 两个独立组 → 两次独立 `batch_skill_action` 调用，`unlink`/`link`/`sync_to_vibe` 等动作之间零交叉污染。

### 11.5 摘要计数（预览底部，对应 §4.2 第 3 步）

```ts
function summarize(cells: Cell[]) {
  const exec = cells.filter(c => c.selectable && c.effectiveAction !== 'none').length
  const conflict = cells.filter(c => c.isConflict && c.selectable).length
  const importNeeded = cells.filter(c => c.needsImport).length
  return { exec, conflict, importNeeded }
}
```
