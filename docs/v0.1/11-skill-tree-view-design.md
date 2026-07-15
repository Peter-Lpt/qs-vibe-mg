# 11 · 软链接管理「目录树视图」设计方案

> 目标：用「文件系统目录树」取代卡片网格视图（`⊞` 切换的 card 模式），解决卡片展开导致的
> 「第一行空出」交互缺陷，并让软链接/真实文件夹的分布与同步状态以最贴近真实磁盘结构的形态呈现。
> 同时把现有列表/矩阵中成熟的同步能力迁移到树，并给出 mac / win 双平台适配与验收标准。
>
> 版本：v2（经子agent审计，修订 P0×4 / P1×7，见文末「审计修订记录」）。

---

## 1. 背景与问题

### 1.1 现状
- `ManageTab.vue` 的 `viewMode` 为 `list | card`：
  - `list`：`SkillRow` 单列行式，点击行就地展开 `SkillDetail`（`expandedSkillId`）。
  - `card`：`SkillCard` `grid auto-fill minmax(240px,1fr)` 多列卡片；展开时设
    `gridColumn:'1/-1'` 占满整行。
- 缺陷（用户反馈 + 实测）：card 模式展开第 2/3 个卡片会占满整行，把同行其余卡片推到下一行，
  导致**第一行出现空洞**。卡片网格与列表信息密度重叠，且引入窄卡片溢出、按钮换行等样式问题。

### 1.2 结论
卡片网格是「列表的脆弱重复形态」。改为**目录树**（单列表）后，不存在同排占满整行的问题；
详情统一改为**右侧抽屉**，列表模式也走抽屉，彻底消除行内大块展开导致的重排。

---

## 2. 设计目标
1. 用目录树替换卡片网格视图（`card` 模式移除；保留 `list` 作为「平面列表」补充）。
2. 以 agent 的 `skills_dir` 为一级根（如 `claude → ~/.claude/skills`），库 `~/.vibe-skills` 为
   特殊「Library」根；逐文件夹列出 skill，标注链接状态/冲突/悬空。
3. **节点状态严格逐 source 派生**（见 §3/§4），复用 `useSkillAgentStatus` 的判定口径，不复用
   skill 级的 `has_conflict`/`has_dangling`。
4. 详情走右侧抽屉，复用既有 `SkillDetail.vue`。
5. 迁移现有能力：单 skill 动作、批量选中+批量同步、冲突并排、搜索、状态筛选、Agent 概览、
   AgentMatrix 联动。
6. mac / win 双平台适配。

---

## 3. 数据模型（前端由 `Skill[]` 派生，无需改后端）

后端 `list_skills` 已返回每个 `Skill` 的 `sources[]`（`from`/`path`/`is_symlink`/
`symlink_target`/`content_hash`），`useSkillAgentStatus.ts` 已逐 source 判定状态。
前端据此构建树，**无需新增 Tauri 命令**。

> 注：后端 `agents.rs:115` 已有 `get_skills_tree`/`build_tree_node`（含 `MAX_SCAN_DEPTH`、
> `truncated`、`is_source_link`、`link_target`），但**只覆盖单个 agent 根、不含 library 根、
> 也不含 sources 的跨 agent 语义**。前端派生能一次性覆盖 library + 所有 agent 根 + 逐 source 状态，
> 故采用前端派生。（见审计 P1-10）

```ts
// 新增 src/types/tree.ts
export type NodeLinkState =
  | 'origin'            // library 自身（from === 'vibe-lib'）
  | 'unlinked'          // 该 agent 无任何来源（仅用于库缺）
  | 'independent'       // 真实文件夹，库无同名
  | 'independent_same'  // 真实文件夹，库有同名且 hash 相同 → replace_with_link
  | 'independent_conflict' // 真实文件夹，库有同名且 hash 不同 → sync_to_vibe
  | 'dangling'          // 软链但 symlink_target 不存在 → remove_dangling
  | 'synced'            // 软链指向库 vibe-lib 同名 → unlink
  | 'linked_elsewhere'; // 软链指向其它 agent/目录 → relink

export interface TreeRoot {
  kind: 'agent' | 'library';
  id: string;                 // agent.id 或 'library'
  label: string;
  dirPath: string;            // skills_dir 或 ~/.vibe-skills
  detected: boolean;
  children: TreeSkillNode[];
  stats: { total: number; synced: number; independent: number; conflict: number; dangling: number };
}

export interface TreeSkillNode {
  kind: 'skill';
  nodeKey: string;            // `${rootId}/${skill.id}` —— 跨根唯一（审计 P0-2）
  rootId: string;
  id: string;                 // Skill.id（= 文件夹名）
  name: string;
  dirName: string;
  path: string;               // 该根下的目录路径（来自对应 source.path）
  isSymlink: boolean;
  symlinkTarget?: string;     // canonicalize 后的真实目标（见 §7）
  linkState: NodeLinkState;   // 逐 source 派生（审计 P0-1）
  hasConflict: boolean;       // 仅当该 source 参与冲突时为真（非 skill 级布尔）
  skill: Skill;
}
```

构建算法 `buildSkillTree(skills, agents, vibeDir)`：
1. 取检测到的 agents + library 为根。
2. 对每个根，遍历在该根有 source 的 skill；取**该根对应的 source**（按 `from === rootId`，
   library 根取 `from === 'vibe-lib'`），据该 source 单点判定 `linkState`（逻辑照搬
   `useSkillAgentStatus.ts:105-180`：比对 `source.symlink_target`/`content_hash` 与
   `vibeSource.path`/`content_hash`）。
3. `hasConflict` 以**该 source 的 hash 与其它 source 的 hash** 比较得出，而非 skill 级 `has_conflict`
   （避免健康 agent 节点被误标冲突，审计 P0-3）。
4. `nodeKey = ${rootId}/${id}`。

---

## 4. 节点与状态可视化（严格对应 composable 口径）

| linkState | 图标/标记 | 颜色 | 推荐动作 |
|-----------|-----------|------|----------|
| `origin` | 📦 | 灰 | 无（库本体） |
| `synced` | 🔗 | 绿 | 取消链接（unlink） |
| `linked_elsewhere` | 🔗↪ | 橙 | 重新链接（relink） |
| `independent_same` | 📁 | 灰 | 替换为链接（replace_with_link） |
| `independent_conflict` | 📁⚠ | 黄 | 同步到库（sync_to_vibe） |
| `independent` | 📁 | 蓝 | 同步到库（sync_to_vibe） |
| `dangling` | 💔 | 红 | 删除悬空（remove_dangling） |
| `unlinked` | ○ | 灰 | 链接（link） |

- 根节点折叠后显示聚合计数（如 `claude (12) · 10🔗 2📁`），与 AgentOverview 一致。
- 节点行尾：状态点 + 行内主操作图标（link/unlink/sync/⋯）。
- library 根节点额外标注「← 来自库 / 已被链接到 N 个 agent」，区分库视角与 agent 视角（审计 P1-9）。

---

## 5. 交互设计

### 5.1 展开 / 折叠
- 根箭头展开/折叠子列表（单列表，无网格重排 bug）。
- 「展开全部 / 折叠全部」；搜索命中自动展开并高亮。
- 展开状态记忆到 `localStorage`（按 rootId）；**选中态不持久化**（当前代码仅内存，刷新清空，见 §11）。

### 5.2 选中 / 批量
- 节点行首复选框；根节点「全选/取消本根」。复用 `selectedSkills:Set<string>` 与现有
  底部浮动 `BatchSyncPanel`（批量 link/unlink/sync 复用 `sync.rs`）。

### 5.3 详情抽屉（替代行内展开，修复 bug）
- 点击节点行（非复选框/动作）→ 右侧滑出 Detail Drawer（~420px），复用 `SkillDetail.vue`。
- **列表模式也统一走抽屉**：`SkillRow` 当前点击行就地展开（`expandedSkillId`）改为打开抽屉，
  列表/树体验一致。
- 抽屉关闭不触发树重排 —— 彻底修复「第一行空出」。

### 5.4 行内 / 右键动作
- 行内主操作：`＋链接`、`－取消`、`⇄同步`、`⋯更多`（删除/揭示/复制路径）。
- **删除须经 `ConfirmDialog`**（复用 `SkillCard`/`SkillDetail` 既有确认，审计 P1-6），禁止直删。
- 右键上下文菜单（与行内一致），前端实现，不依赖原生菜单（兼顾 mac/win）。

### 5.5 搜索
- 顶部搜索（已有）在树模式按 name/description/id/path 本地过滤（复用 `searchSkills`，P3 已落地）；
  命中节点高亮并自动展开所属根。

### 5.6 状态筛选 / Agent 概览 / AgentMatrix
- 状态 chips 筛选、`agentOverview` 原样保留，叠加在树上方，作用于树节点显隐。
- **Agent 概览在树形态下的语义（审计 P0-3）**：因「agent 即一级根」，include/exclude 整表隐藏
  会与树结构冲突。明确为：**点击 Agent 概览卡片 = 聚焦/高亮对应根**（滚动并短暂高亮），
  `selectedAgentFilter` 仅作「聚焦」而非「隐藏其它根」；若用户需要隐藏，提供独立的
  「仅看选中 Agent」开关（exclude 模式才真正隐藏其它根）。`displaySkills` 派生需据此重写：
  树模式默认显示所有根，聚焦仅高亮。
- **AgentMatrix 联动（审计 P0-2）**：`handleMatrixExpand` 现通过 `id="skill-${skillId}"` +
  `scrollIntoView` 定位；树中同一 skill 在 library 与多 agent 根**各出现一次**，会产生
  **重复 DOM id**（非法）。必须改为复合 id：`id="skill-${rootId}-${skillId}"`，且
  `AgentMatrix` 的 `expand-skill` 需明确高亮**所有出现该 skill 的根**（而非单一 id）。

### 5.7 排序（审计 P1-5）
- 树内每个根下子节点排序：异常优先（dangling/conflict/linked_elsewhere 在前）→ 名称；
  对齐现有 `sortBy`（status/name/sources）。

### 5.8 冲突并排
- 冲突节点以 `⚠` 提示；点开抽屉复用 `SkillDetail` 的「默认并排」对比各 source SKILL.md。

---

## 6. 头脑风暴：树形态下更好/更方便/更有效的同步操作

1. **揭示文件管理器（Reveal）**：节点「在文件管理器中打开」→ `tauri-plugin-opener` 的
   `openPath(dir)`（权限 `opener:default` 已具备，`Cargo.toml:17`/`capabilities/default.json:8`）。
   mac 开 Finder、win 开资源管理器，跨平台一致。**本期实现**。
2. **复制路径**：右键/更多「复制绝对路径」「复制 `~` 缩写」。**本期实现**（低成本）。
3. **树内拖拽链接（DnD）**：拖一个 skill 节点到某 agent 根 = 调用 `link_skill`。
   **本期实现，但拖放前必须校验目标根是否已存在实体文件夹/软链**（审计 P0-4）：若已存在真实
   文件夹，先提示「将覆盖/需先删除」，避免 `link_skill` 误删用户数据；若已存在软链则视为 no-op。
   （**不做跨进程拖拽到外部文件管理器**：受平台限制，文档已说。）
4. **一键全同步 / 全链接**：根「⋯」菜单「将本 agent 下所有独立 skill 全链接」「全部同步到库」，
   批量复用现有命令。**本期实现**。
5. **Pin 置顶**：虚拟「Pinned」分组。**后续**。
6. **冲突根红徽标 + 一键并排**：根 `conflict>0` 红徽标；「全部展开并排」。**本期徽标 + 后续一键并排**。
7. **库↔agent 双向分组**：根分组维度切换（agent / library）。**本期提供切换**（sources 足够）。
8. **树与 AgentMatrix 联动**：见 §5.6（复合 id + 全根高亮）。**本期实现**。
9. **展开深度保护**：`build_tree_node` 已有 `MAX_SCAN_DEPTH=12`+`visited`；根显示 `truncated` 徽标。
10. **撤销/历史**：写操作已记录 history（`sync.rs` 的 `record_action_with_skills`），树动作后在
    History Tab 撤销，复用即可。

---

## 7. mac / win 双平台适配

| 项 | macOS | Windows | 处理 |
|----|-------|---------|------|
| 软链呈现 | 真实 symlink | junction（后端 `is_link` 已含 junction，`fs.rs:150`） | 前端统一 🔗；`symlinkTarget` 展示用 **canonicalize 后的真实库路径**（去 `\\?\`，审计 P1-8） |
| 揭示文件 | `openPath(dir)` → Finder | `openPath(dir)` → 资源管理器 | 直接用 opener 插件，跨平台一致 |
| 路径分隔符 | `/` | `\` | 展示层 `path.split(/[\\/]/)`；复制路径保留原样 |
| 大小写 | 敏感 | 不敏感 | `id` 比较已统一；hash 由后端算，前端无需 |
| DnD 跨进程 | Tauri2 默认**未启用 mac 沙箱**（当前 `tauri.conf.json` 无 entitlements），`openPath` 不受限 | 跨进程拖拽需额外 IPC | **本期仅树内拖拽链接**；仅将来上架 Mac App Store 才需 `com.apple.security.files.*` 授权（审计 P1-7 勘误） |
| 快捷键 ⌘/Ctrl+K | `metaKey` | `ctrlKey` | 现有 `handleKeydown` 已兼顾 |
| 右键菜单 | 前端统一实现 | 同 | 不依赖原生菜单 |
| 沙箱路径 | 用户目录 agent dir 通常可访问 | 用户目录/其它盘按需 | reveal/预览沿用现有沙箱约束（库 ∪ agent dirs） |
| 编码 | UTF-8 | GBK 系统区可能乱码 | Tauri `invoke` 走 JSON UTF-8，无影响 |

**关键**：所有磁盘读写仍走既有 `commands/`（含 normalize、sandbox、`\\?\` 剥离、junction 识别），
树视图只新增 UI 表达与 `openPath` 揭示，不新增破坏文件系统的逻辑。

---

## 8. UI 布局（树模式）

```
[标题 软链接管理 (n/N)]  [＋安装]  [⊞ list | 🌳 tree]   ← viewMode: list / tree（card 移除）
[Agent 概览 chips（聚焦用）] [状态筛选 chips] [排序] [搜索 ⌘K]
[Stats bar: 总/共享/独立/异常]
───────────────────────────────
🌳 树区（单列表）：
  ▾ claude  (~/.claude/skills)  [12 · 10🔗 2📁]  ⋯(全链接/全同步/揭示/复制路径)
     ☑ 🔗 weather        [－][⇄][⋯]        ← synced
     ☑ 📁⚠ summarize     [⇄][⋯]            ← independent_conflict
     ☐ 🔗↪ translator    [relink][⋯]        ← linked_elsewhere
  ▾ codex   (~/.codex/skills)  [5 · 5🔗]
  ▾ Library (~/.vibe-skills)  [30]  ← 库根，标注链接方向
[底部浮动批量条：选中 n · 批量操作 · 取消]   ← 复用 BatchSyncPanel
[AgentMatrix 折叠区]                         ← 点击单元格 → 树复合 id 全根高亮
[右侧 Detail Drawer]                         ← 复用 SkillDetail
```

---

## 9. 功能迁移清单

| 现有功能 | 迁移方式 |
|----------|----------|
| SkillRow 行 + 就地展开 | 改为树节点行 + 右侧抽屉（列表也走抽屉） |
| SkillCard 网格 + 占满整行展开 | **下线**，由 tree 取代 |
| SkillDetail（per-agent/冲突/批量/删除） | 抽屉内复用 |
| 状态点 / 动作按钮 | 树节点行尾小图标 |
| 批量选中 + BatchSyncPanel | 复用（选中源改为树复选框） |
| 搜索 / chips / Agent 概览 / AgentMatrix | 保留；Agent 概览改为聚焦语义（§5.6） |
| 冲突并排 | 抽屉内复用 |
| useSkillAgentStatus 状态 | 复用到树节点逐 source 着色 |
| 排序 sortBy | 树内根下排序（§5.7） |
| 删除 ConfirmDialog | 树节点删除复用确认（§5.4） |

---

## 10. 实现验收标准
- [ ] `viewMode` 为 `list | tree`；`card` 移除。
- [ ] 树由 `Skill[]` 前端派生，无新增 Tauri 命令（reveal 用既有 `openPath`）。
- [ ] **节点 `linkState` 逐 source 派生**（8 态），与 `useSkillAgentStatus` 口径一致；
      不复用 skill 级 `has_conflict`/`has_dangling`。
- [ ] `nodeKey = rootId/id`；`AgentMatrix` 联动改复合 DOM id，全根高亮。
- [ ] 点击节点 → 右侧抽屉复用 `SkillDetail`；列表模式也走抽屉。
- [ ] 批量选中 + 浮动条 + `BatchSyncPanel` 树模式可用。
- [ ] 搜索命中高亮并自动展开所属根；树内排序异常优先+名称。
- [ ] 根菜单：reveal / 复制路径 / 全链接 / 全同步，mac/win 可用。
- [ ] 树内拖拽 skill→agent 根 = `link_skill`，**拖放前校验目标已存在实体**（防误覆盖）。
- [ ] AgentMatrix 单元格点击 → 树定位高亮（复合 id）。
- [ ] `vue-tsc --noEmit` 0 错误；`pnpm build` 通过。
- [ ] mac（symlink）/ win（junction）下 reveal / 状态展示一致。

---

## 11. 风险与开放问题
- **树内 DnD 仅限前端**：跨进程拖到文件管理器因平台限制本期不做。
- **深度**：`MAX_SCAN_DEPTH=12` 足够；根显示 `truncated` 徽标。
- **性能**：树一次性派生 O(skills×sources)，与 `list_skills` 同量级；展开/折叠纯前端显隐，
  无额外后端调用；保持 `refreshSkills` 防抖。
- **重复 skill**：同一 skill 在 library 与多 agent 根各出现一次（预期，体现链接关系），以
  `linked` 状态点 + library 根「链接方向」标注区分（§4/§7）。
- **选中态持久化**：当前无持久化（仅内存），刷新清空；`localStorage` 仅记展开态，文档已区分。
- **card 模式移除**：推荐移除（用户明确网格问题多）；如后续需要，card 改为「仅图标缩略」而非带展开。

---

## 审计修订记录（v1 → v2）
- **P0-1**：节点状态改为逐 source 派生（`NodeLinkState` 8 态），照搬 `useSkillAgentStatus.ts:105-180`，
  移除对 skill 级 `has_conflict`/`has_dangling` 的直接复用。
- **P0-2**：新增 `nodeKey = rootId/id`；`AgentMatrix` 联动改复合 DOM id `skill-${rootId}-${id}`，全根高亮。
- **P0-3**：明确 Agent 概览在树形态下为「聚焦/高亮」语义，exclude 才隐藏其它根；`displaySkills` 派生重写。
- **P0-4**：树内拖拽链接前校验目标根是否已存在实体文件夹/软链，防 `link_skill` 误覆盖。
- **P1-5**：补树内排序（异常优先+名称）。
- **P1-6**：树节点删除须经 `ConfirmDialog`。
- **P1-7**：勘误 mac 沙箱论断——当前 Tauri2 未启用 sandbox，reveal 不受限，仅 MAS 上架需 entitlement。
- **P1-8**：junction 目标展示用 canonicalize 后的真实库路径。
- **P1-9**：library 根补充「链接方向」标注。
- **P1-10**：说明 `get_skills_tree` 已存在但未采用的原因（仅单 agent 根、缺 library/多 source 语义）。
- **P1-11**：注明选中态刷新即清空（无持久化）。
```
