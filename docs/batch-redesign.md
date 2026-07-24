# 批量操作重构方案

## 一、现状问题

### 1. 滚动问题
批量操作条在列表底部，选中技能后要滚到底部才能操作。

### 2. 点击过多
选技能 → 点批量按钮 → 下拉选动作 → 确认弹窗 → 点执行 = 4 次点击

### 3. 信息不透明
下拉只显示动作名和数量，不知道影响哪些技能/Agent。确认弹窗才显示详情，但已经选了动作。

### 4. 动作名称抽象
"同步到库""替换为链接""重新链接" — 用户不理解具体会发生什么。

### 5. 按动作分组而非意图
用户想的是"修复所有问题""统一所有副本"，不是"对 5 个技能执行 sync_to_vibe"。

### 6. 缺少常见意图
- 一键修复所有问题
- 统一所有副本到库
- 链接到指定 Agent
- 批量导入插件技能

---

## 二、场景矩阵

### 状态 × 动作 完整映射

| 状态 | 含义 | 可用动作 | 动作结果 |
|------|------|---------|---------|
| **origin** | 源头在库中 | 无 | — |
| **synced** | symlink 指向库 | `unlink` | 移除链接 |
| **linked_elsewhere** | symlink 指向别处 | `relink` | 改指向库 |
| **independent (同hash)** | 独立副本，内容同库 | `replace_with_link` | 删副本，建 symlink |
| **independent (冲突)** | 独立副本，内容不同 | `sync_to_vibe` | 覆盖库，建 symlink |
| **independent (无库)** | 独立副本，库中无 | `sync_to_vibe` | 复制到库，建 symlink |
| **dangling** | symlink 失效 | `remove_dangling` | 移除失效链接 |
| **unlinked (有库)** | 无链接，库中有 | `link` | 创建 symlink |
| **unlinked (无库)** | 无链接，库中无 | 无 | — |
| **plugin (无库)** | 插件来源，未入库 | `sync_from_plugin` | 复制到库 |

### 多 Agent 场景

| 场景 | Agent A | Agent B | Agent C | 批量操作 |
|------|---------|---------|---------|---------|
| 库中 skill，A 链接，B 未链接 | synced | unlinked | — | "链接到 B" |
| 库中 skill，A 链接正确，B 链接错误 | synced | linked_elsewhere | — | "修复 B 的链接" |
| 库中 skill，A 独立副本(同hash)，B 断链 | independent | dangling | — | "统一到库" |
| 库中 skill，A 独立(冲突)，B 已链接，C 未链接 | independent | synced | unlinked | "同步 A 到库" + "链接 C" |
| 仅 agent 有(无库)，A 独立，B 未链接 | independent | unlinked | — | "入库 + 链接" |

### 用户意图场景

| 意图 | 触发场景 | 涉及动作 | 优先级 |
|------|---------|---------|--------|
| **修复所有问题** | 待处理视图，有冲突/断链/错链 | `remove_dangling` + `relink` + `sync_to_vibe`(冲突) | 高 |
| **统一到库** | 有多个独立副本想合并 | `sync_to_vibe` + `replace_with_link` | 高 |
| **链接到 Agent** | 新增 Agent，想把库中 skill 都链接过去 | `link` | 中 |
| **断开链接** | 不想某个 Agent 继续用这些 skill | `unlink` | 中 |
| **导入插件** | 插件视图，想把插件技能入库 | `sync_from_plugin` | 中 |
| **清理断链** | 有失效 symlink 需清理 | `remove_dangling` | 低 |

---

## 三、设计方案

### 核心原则

1. **浮动操作条** — 固定在视口底部，不随列表滚动
2. **意图优先** — 先展示"你想做什么"，再展开具体操作
3. **智能推荐** — 根据选中技能的状态自动推荐最合适的操作
4. **最少点击** — 安全操作一键执行，危险操作才确认
5. **即时反馈** — 操作条内直接显示影响范围

### UI 布局

```
┌─────────────────────────────────────────────────────────────┐
│ 标题栏：搜索 · 排序 · 清除筛选                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  技能列表                                                    │
│  ☑ skill-a  [synced] [synced] [unlinked]                   │
│  ☑ skill-b  [dangling] [independent] [synced]              │
│  ☐ skill-c  [synced] [synced] [synced]                     │
│  ☑ skill-d  [linked_elsewhere] [unlinked] [unlinked]       │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ 浮动操作条（固定在视口底部）                                    │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 已选 3 个技能 · 影响 7 个 Agent 关系                      │ │
│ │                                                         │ │
│ │ [修复问题 2] [统一副本 2] [链接 3] [断开 1] [更多 ▾]     │ │
│ │                                                         │ │
│ │ ▸ 展开详情：                                             │ │
│ │   skill-a → Hermes: 链接                                │ │
│ │   skill-b → Claude: 修复断链; → Hermes: 替换为链接       │ │
│ │   skill-d → Claude: 修复错链; → Hermes: 链接; → Pi: 链接│ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 操作条设计

#### 第一行：摘要
```
已选 3 个技能 · 影响 7 个 Agent 关系
```

#### 第二行：意图按钮（智能推荐）
根据选中技能的状态自动计算：

| 按钮 | 显示条件 | 图标 | 点击行为 |
|------|---------|------|---------|
| **修复问题 N** | 有 dangling/linked_elsewhere/independent(冲突) | 🔧 | 一键执行修复，确认后直接执行 |
| **统一副本 N** | 有 independent(同hash/无库) | 📦 | 一键统一到库 |
| **链接到 ▾** | 有 unlinked(有库) | 🔗 | 展开 Agent 列表，选目标 Agent |
| **断开 N** | 有 synced | ✂️ | 确认后执行 |
| **更多 ▾** | 有其他操作 | ⋯ | 展开完整操作列表 |

#### 第三行：可展开详情
点击展开，显示每个 (skill, agent) 对将发生什么：
```
skill-a → Hermes: 链接
skill-b → Claude: 修复断链; → Hermes: 替换为链接
```

### 操作分类

#### 确认操作（所有操作都需确认，因为都有副作用）
- **修复问题** — 组合动作，会修改链接和可能覆盖库
- **统一副本** — 会删除本地副本或覆盖库内容
- **链接到 Agent** — 会修改 agent 目录
- **断开链接** — 会移除 agent 目录中的 symlink
- **导入插件** — 会复制文件到库
- **清理断链** — 会移除失效 symlink

注：所有操作都需确认，因为都会修改文件系统。确认弹窗显示具体影响的 (skill, agent) 对。

### 交互流程

#### 流程 1：修复问题（一键）
```
选中技能 → 点"修复问题" → 弹窗显示将修复的内容 → 确认 → 执行 → 显示结果
```

#### 流程 2：链接到 Agent（选目标）
```
选中技能 → 点"链接到 ▾" → 展开 Agent 列表 → 点击目标 Agent → 执行 → 显示结果
```

#### 流程 3：断开链接（确认）
```
选中技能 → 点"断开" → 弹窗确认 → 执行 → 显示结果
```

#### 流程 4：查看更多（展开）
```
选中技能 → 点"更多 ▾" → 展开所有可用操作 → 点击具体操作 → 执行
```

### 技术实现

#### 新 composable：useBatchActions.ts

```typescript
interface BatchIntent {
  id: string;
  labelKey: string;
  icon: string;
  count: number;
  pairs: [string, string][];
  actions: Record<string, [string, string][]>;  // 按动作分组的 pairs（用对象而非 Map）
  needsConfirm: boolean;  // 基于 skillActionRegistry.removesTarget
}

interface BatchSummary {
  totalSkills: number;
  totalPairs: number;
  intents: BatchIntent[];
}
```

#### 意图计算逻辑

关键修正：
1. 使用 `action` 字段而非 `status` 字段做意图分类（因为 independent 子类型通过 action 区分）
2. `replace_with_link` 归为需确认（removesTarget=true，会删除本地副本）
3. 增加 `sync_from_plugin` 意图
4. 使用纯函数而非 composable 获取状态

```typescript
// 纯函数：获取 skill 对所有 agent 的状态（非 composable）
function getAgentStatuses(skill: Skill, agents: Agent[], t: TFunc): AgentStatus[] {
  // 复用 useSkillAgentStatus 的逻辑，但不创建响应式依赖
  // 实现方式：提取 useSkillAgentStatus 中的 allAgentStatuses 计算逻辑为独立函数
}

function computeIntents(selectedSkills: Skill[], agents: Agent[], t: TFunc): BatchIntent[] {
  const intents: BatchIntent[] = [];
  
  // 收集所有 (skillId, agentId, action, statusType) 四元组
  const triples: Array<{
    skillId: string; skillName: string;
    agentId: string; agentName: string;
    action: AgentAction; statusType: AgentStatusType;
  }> = [];
  
  for (const skill of selectedSkills) {
    const statuses = getAgentStatuses(skill, agents, t);
    for (const status of statuses) {
      if (status.action !== 'none') {
        triples.push({ 
          skillId: skill.id, skillName: skill.name || skill.id,
          agentId: status.agent.id, agentName: status.agent.name,
          action: status.action, statusType: status.status 
        });
      }
    }
  }
  
  // 按意图分组（基于 action 字段，避免 status 子类型歧义）
  
  // 修复问题：dangling + 错链 + 冲突副本（independent + sync_to_vibe）
  const repairPairs = triples.filter(t => 
    t.action === 'remove_dangling' || 
    t.action === 'relink'
  );
  // 注意：冲突副本（independent + sync_to_vibe）单独处理，避免与 unify 重叠
  
  // 统一副本：同hash独立副本（replace_with_link）+ 无库独立副本（sync_to_vibe 且无 vibe-lib source）
  const unifyPairs = triples.filter(t =>
    t.action === 'replace_with_link' || 
    (t.action === 'sync_to_vibe' && t.statusType === 'independent')
  );
  
  // 链接：未链接且有库
  const linkPairs = triples.filter(t => t.action === 'link');
  
  // 断开：已链接
  const unlinkPairs = triples.filter(t => t.action === 'unlink');
  
  // 导入插件
  const pluginPairs = triples.filter(t => t.action === 'sync_from_plugin');
  
  // 构建意图（needsConfirm 基于 skillActionRegistry.removesTarget）
  if (repairPairs.length > 0) {
    intents.push({
      id: 'repair', labelKey: 'batch.intent_repair', icon: 'Wrench',
      count: repairPairs.length,
      pairs: repairPairs.map(t => [t.skillId, t.agentId]),
      actions: groupByAction(repairPairs),
      needsConfirm: true,  // remove_dangling 和 relink 都有副作用
    });
  }
  // ... 类似构建其他意图
  
  return intents;
}
```

### i18n Key 设计

```json
{
  "batch": {
    "selected_summary": "已选 {skills} 个技能 · 影响 {pairs} 个关系",
    "intent_repair": "修复问题 {count}",
    "intent_unify": "统一副本 {count}",
    "intent_link": "链接到",
    "intent_unlink": "断开 {count}",
    "intent_plugin": "导入插件 {count}",
    "intent_more": "更多",
    "confirm_repair_title": "确认修复",
    "confirm_repair_msg": "将修复 {count} 个问题（断链清理、错链修正）",
    "confirm_unify_title": "确认统一",
    "confirm_unify_msg": "将 {count} 个独立副本统一到库（库内容可能被覆盖）",
    "confirm_unlink_title": "确认断开",
    "confirm_unlink_msg": "将断开 {count} 个链接",
    "confirm_plugin_title": "确认导入",
    "confirm_plugin_msg": "将 {count} 个插件技能导入到库",
    "result_title": "操作结果",
    "result_success": "成功 {count} 项",
    "result_error": "失败 {count} 项",
    "no_actions": "选中的技能无可执行操作"
  }
}
```

### 文件变更

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/composables/useBatchActions.ts` | 重写 | 意图计算 + 执行逻辑 |
| `src/components/manage/SkillsView.vue` | 修改 | 浮动操作条 + 意图按钮 |
| `src/locales/*.json` | 修改 | 新增意图相关 key |
| `src/components/manage/IssueRepairPanel.vue` | 修改 | "修复问题"一键执行 |

### 代码量估算

| 组件 | 行数 |
|------|------|
| useBatchActions.ts | ~200 行 |
| SkillsView.vue 批量部分 | ~150 行 |
| 总计 | ~350 行 |

比当前实现（~300 行）略多，但功能和 UX 大幅提升。
