# 技能列表「展开详情 / 同步到库」交互重构方案

> 版本：v0.3-simplified（设计讨论 + 两轮子 agent 审计已闭合）
> 日期：2026-07-13
> 关联：07（多选/批量栏）、08（多选跨 agent 矩阵面板）。本文只管**单 skill 行内展开 + 同步语义**，与 08 不冲突（i18n 键互不重叠）。

---

## 0. 结论速览

| 用户问题 | 结论 | 关键动作 |
|----------|------|----------|
| ① 展开后空白大/变形 | 卡片**嵌套整张 `SkillRow`**（含重复 header）塞进 240px 格子 + 列表 `justify-between` 中段留白 | 抽 **`SkillDetail.vue`**，列表/卡片共用；行内改 `flex-1` 对齐 |
| ② 不知同步了什么 | 文案像库级操作，且**卡片主操作根本不弹成功 toast** | 文案点明对象 + toast 必带 agent + scope 提示 |
| ③ 列表形式是否必要 | **保留**切换，废除重复的展开渲染 | 见 §2 |
| ④ 重构交互 | 见 §4 | 抽 `SkillDetail` + 术语/反馈修正 |
| ⑤ 文档输出 | 本文件 `docs/v0.1/09` | — |

**一句话**：问题不在"两种形态"，而在"两套展开实现、且卡片把整张行组件又嵌一遍"。统一为一个 `SkillDetail`，术语讲清"哪一个 skill 的哪一个 agent"，UI 即不乱、语义即显式。

---

## 1. 根因（逐条对照代码）

**① 变形（卡片为主）**：`SkillCard.vue:174-176` 展开时嵌入整张 `<SkillRow :expanded="true">`，而 `SkillRow` 展开体是按全宽 list 设计的 per-agent 矩阵（`SkillRow.vue:436-567`）。网格单元 `minmax(240px,1fr)` 把全宽矩阵压扁、且下方卡片不回流 → 两侧/右侧大片空白。

**① 第二段空白（列表也有）**：`SkillRow.vue:500` 的 agent 行用 `flex ... justify-between`，左侧 label 短 + 右侧按钮短时中段被撑开 → "左右空白很大"。

**② 同步语义模糊（审计修正后的准确根因，三重）**：
1. `btn_sync="同步到技能库"`（`zh/en/zh-TW:52`）读起来像库级操作；实际动作永远是 **单 skill × 单 agent**（`skillsStore.syncToVibe(skillId, agentId)`）。
2. **locale `synced_to_vibe` 三语字符串值均不含 `{agent}` 占位符**（`zh/en/zh-TW:42`）。因此 `SkillRow.vue:118` 传 `{agent}` 也是**空操作**——列表行内 toast 实际也不带 agent 名。
3. **`SkillCard.handlePrimaryAction` 成功路径完全无 toast**，仅 `catch` 有错误提示（`SkillCard.vue:80`）——卡片主操作是"静默"的。

---

## 2. Q3 决策

**保留 list/card 切换，废除"卡片内嵌套 SkillRow"的重复展开实现。** 列表（紧凑、扫 agent 徽章）与卡片（带 description）是密度差异，都有价值且 `viewMode` 已持久化。真正负债是同一份 per-agent 详情被实现两次、且卡片用"嵌套整组件"复用 → header 重复 + 宽度不匹配。正确做法：抽**单一 `SkillDetail.vue`** 共用。

---

## 3. 重构方案

### 3.1 抽 `SkillDetail.vue`（核心）
- 新文件 `src/components/manage/SkillDetail.vue`，承载 `SkillRow.vue:436-567`（per-agent 分组列表 + 勾选 + 逐 agent 同步/链接/取消按钮 + 冲突预览 `376-434`）。
- `SkillRow` 与 `SkillCard` 展开体都改为 `<SkillDetail :skill :agents />`；**卡片不再嵌 `SkillRow`**。
- 展开事件链保持 intact：`ManageTab` 的 `expandedSkillId`（`ManageTab.vue:506-508`）控制 `SkillRow`/`SkillCard` 的 `expanded`，`SkillDetail` 只渲染内容、不接管展开态。

### 3.2 布局修复（不变形、无中段空白）
- agent 行：外层 `flex items-center gap-2`，内层左侧 `flex-1 min-w-0`（含 checkbox/icon/name/status/symlink，`truncate`），右侧按钮 `shrink-0 whitespace-nowrap`。**禁止 `justify-between`**；移除 `SkillRow.vue:496` 的 `pl-3.5`。
- **窄卡片（~216px 内容区）防溢出**：名称 span 必加 `min-w-0`；`linked_elsewhere` 的「→ {agent}」span 改 `flex-1 min-w-0 truncate`、**去掉** `shrink-0` 与 `max-w-[120px]`（其后的 `|` 与 `→ 库` 保持 `shrink-0`）。

### 3.3 同步语义与反馈显式化
1. **按钮文案点明对象**：per-agent 同步按钮改「**从 {agent} 同步**」。因 `actionLabel(t, action)` 不接受 agent 参数（`useSkillAgentStatus.ts:284`），在 `SkillDetail` 内用 `cellBtnLabel()` 对 `sync_to_vibe` 取 `t("manage.btn_sync_from", { agent })`。原 `btn_sync` 保留给无单 agent 上下文处（如 `useThisVersion`）。
2. **toast 必带 agent（关键：改字符串值）**：把三语 `synced_to_vibe` **值**改为含 `{agent}`（见 §3.4）。`SkillCard.handlePrimaryAction` 当前**六分支全缺成功 toast**，须对称补齐（对照 `SkillRow.vue:106-131`）：`link`→`skills.linked`、`unlink`/`remove_dangling`→`skills.unlinked`/`manage.dangling_removed`、`sync_to_vibe`/`replace_with_link`→`manage.synced_to_vibe`/`replaced_with_link`、`relink`→`manage.relinked`，均带 `status.agent.name`。`useThisVersion`（`SkillRow.vue:236`）也补 agent。
3. **scope 提示**：同步类按钮加 `:title="t('manage.sync_scope_tip', { skill, agent })"`，文案"将 {skill} 从 {agent} 同步到技能库（仅此一个 Agent）"。

### 3.4 i18n 变更（三语具体新值，须同步 zh/en/zh-TW）
| Key | zh | en | zh-TW | 备注 |
|-----|----|----|-------|------|
| `btn_sync_from`（新增） | 从 {agent} 同步 | Sync from {agent} | 從 {agent} 同步 | `SkillDetail` per-agent 同步按钮 |
| `synced_to_vibe`（改值） | 已同步 {agent} 到技能库 | Synced {agent} to library | 已同步 {agent} 到技能庫 | **改值**（关键），原值无 `{agent}` |
| `sync_scope_tip`（新增） | 将 {skill} 从 {agent} 同步到技能库（仅此一个 Agent） | Sync {skill} from {agent} to library (this one agent only) | 將 {skill} 從 {agent} 同步到技能庫（僅此一個 Agent） | 按钮 `:title` |
| `btn_sync` / `expand_detail` | 保留 | 保留 | 保留 | 不动 |

---

## 4. 风险（须落地）

- **R1（展开链）**：`SkillDetail` 不接管 `expanded`，`ManageTab.expandedSkillId` 单选展开逻辑不变。
- **R2b（卡片删除丢失，审计 HIGH）**：**`SkillCard` 根本没有删除按钮**（组件仅 178 行，无 `ConfirmDialog`），删除能力全靠嵌套的 `SkillRow` header。删嵌套后将丢失卡片删除。修复推荐 **(A)**：在 `SkillCard` summary 加删除按钮 + `ConfirmDialog`（与 `SkillRow` 对称），`SkillDetail` 不含删除。验收须覆盖卡片模式。
- **R2（主操作 vs per-agent）**：卡片 header 主操作 = 该 skill 首选推荐动作（作用于首个 `action!=='none'` 的 agent，确定性、不暗示同步全部 agent）；`SkillDetail` 内 = 逐 agent 精细操作。二者语义不冲突，header 按钮文案也复用「从 {agent} 同步」。

---

## 5. 验收标准（可测）

1. 卡片点「展开详情」：仅一个 header，下方 `SkillDetail`；展开后下方卡片正常回流，无大片空白。
2. 列表展开：agent 行中段无大段空白（左占满、按钮右对齐）。
3. 卡片与列表 per-agent 详情视觉一致（同源 `SkillDetail`）。
4. per-agent 同步按钮含 agent 名（「从 Claude 同步」）。
5. 卡片主操作**所有 6 类动作**均弹成功 toast 且**带 agent 名**。
6. 点击后能从文案 + toast 明确知道"哪一个 skill × 哪一个 agent"。
7. 删除入口不丢失：**卡片模式可达**（R2b 决议 A）。
8. `pnpm build` 通过；`pnpm dev` 手验两种模式展开均不变形。

---

## 6. 审计迭代记录

**Round 1（3 个 High 前提错误，已修正）**：
1. `synced_to_vibe` 三语值无 `{agent}` 占位符 → 原"行内 toast 已带 agent"为误判，须改字符串值（§3.4）。
2. `SkillCard` 无删除按钮 → 删嵌套会丢卡片删除（R2b）。
3. `SkillCard` 主操作成功路径无任何 toast（仅静默）。
4. `useThisVersion` 冲突入口也漏 agent；i18n §6 须给三语值且明确改值。

**Round 2（核实 + 补强，已修正）**：
- `btn_sync` vs `btn_sync_from`：正文 §3.3/§3.4 现已一致（保留 `btn_sync` + 新增 `btn_sync_from`），无矛盾。
- M1：卡片主操作 6 分支全补 toast（§3.3.2）。
- M2：窄卡片长名 / `linked_elsewhere` 防溢出（§3.2）。
- M3：验收 #7 显式覆盖卡片删除（§5）。
- L2/L3：`sync_scope_tip` 绑定（§3.3.3）；与 doc 08 不冲突声明（§0）。

**终态**：不变形 / 语义显式 / 功能完整 / 可回归 四方面闭合，可进入实现评估。
