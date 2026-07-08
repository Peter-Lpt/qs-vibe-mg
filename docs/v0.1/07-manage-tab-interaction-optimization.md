# 管理 Tab 交互优化方案 v0.4

> 版本：v0.4  
> 日期：2026-07-08  
> 状态：设计阶段

---

## 一、变更概述

基于 v0.3 实施后的用户反馈：

| 问题 | 方案 |
|------|------|
| 缺少跨 skill 的批量操作 | 新增 skill 多选 + 浮动批量操作栏 |
| Agent 概览和 Agent 筛选 chips 功能重复 | 合并：Agent 概览卡片本身支持选中/排除，删除独立的 agent chips |
| 状态筛选 UI 不够精致 | 优化为分组 chips + 小图标 + 更清晰的视觉层级 |
| Agent 概览位置不对 | 移到最顶部（header 下方第一行） |

---

## 二、Skill 多选 + 浮动批量操作栏

### 2.1 设计

在每个 SkillRow/SkillCard 左上角增加 checkbox，选中后屏幕底部出现浮动批量操作栏：

```
┌─────────────────────────────────────────────────────┐
│ Header: 软连接管理 (12/15)            [安装] [≡][⊞]  │
├─────────────────────────────────────────────────────┤
│ Agent 概览: [Claude✓] [Hermes] [Codex] ...         │
│ ...                                                 │
│ skill list...                                       │
│                                                     │
│ ┌─────────────────────────────────────────────────┐ │
│ │ [全选] 已选 5 个 skill    [同步到库] [取消选择]  │ │  ← 浮动栏
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### 2.2 浮动批量操作栏

- 位置：`fixed` 定位在屏幕底部，`z-index: 40`
- 显示条件：`selectedSkills.size > 0`
- 内容：
  - 左侧：全选 checkbox + "已选 N 个 skill"
  - 右侧：操作按钮 + 取消选择按钮
- 操作按钮：根据选中的 skill 状态自动判断
  - 如果选中的 skill 都有"真实文件未链接到库" → 显示"同步到库"
  - 如果选中的 skill 都是"已链接到库" → 显示"取消链接"
  - 混合状态 → 显示"同步到库"（最高优先级操作）

### 2.3 批量执行逻辑

```typescript
async function batchSyncSelected() {
  const skills = displaySkills.value.filter(s => selectedSkills.value.has(s.id));
  
  // 按 skill 分组：每个 skill 找出需要操作的 agent
  const operations: { skillId: string; agentIds: string[] }[] = [];
  
  for (const skill of skills) {
    const agentIds = skill.sources
      .filter(src => !src.is_symlink && src.from !== 'vibe-lib')
      .map(src => src.from);
    
    if (agentIds.length > 0) {
      operations.push({ skillId: skill.id, agentIds });
    }
  }
  
  // 每个 skill 一次 IPC 调用（batchSkillAction）
  let total = 0;
  const errors: string[] = [];
  const toast = useToast();
  
  for (const op of operations) {
    try {
      const r = await skillsStore.batchSkillAction(op.skillId, op.agentIds, 'sync_to_vibe');
      total += r.synced_count;
      errors.push(...r.errors);
    } catch (e) {
      errors.push(`${op.skillId}: ${String(e)}`);
    }
  }
  
  await skillsStore.refreshSkills();
  selectedSkills.value.clear();
  
  if (errors.length > 0) {
    toast.show(`成功 ${total} 个，失败 ${errors.length} 个`, 'warning');
  } else {
    toast.show(`已同步 ${total} 个 skill`, 'success');
  }
}
```

### 2.4 SkillRow/SkillCard 变更

- `SkillRow.vue`：折叠状态左侧增加 checkbox（在展开箭头左边）
- `SkillCard.vue`：右上角增加 checkbox
- 选中状态：卡片/行 border 变 primary 色
- 全选/取消全选：浮动栏左侧的 checkbox 控制

### 2.5 全选逻辑

全选只作用于当前 `displaySkills`（筛选后的列表），不是所有 skill。

---

## 三、Agent 概览卡片合并 Agent 筛选

### 3.1 当前问题

v0.3 有两套 agent 交互：
1. Agent 概览卡片（顶部）：显示统计 + 点击选中筛选
2. Agent 筛选 chips（筛选区）：多选 + include/exclude

两者功能重叠。

### 3.2 新方案：Agent 概览卡片 = Agent 筛选

将 Agent 概览卡片升级为 agent 筛选的唯一入口：

```
┌──────────┐ ┌──────────┐ ┌──────────┐
│ ✓ Claude │ │ ○ Hermes │ │ ○ Codex  │  ← 选中态 = 筛选
│ 12 skills│ │ 5 skills │ │ 8 skills │
│ 8 linked │ │ 3 linked │ │ 5 linked │
│ 2 冲突   │ │          │ │ 1 冲突   │
└──────────┘ └──────────┘ └──────────┘

选中样式：border 变 primary + 背景变 primary-light
排除模式：卡片右上角显示 [排除] 标记
```

**交互**：
- 单击卡片 → 切换选中/取消（和现在一样）
- 右键或长按 → 切换 include/exclude 模式
- 或者：卡片下方增加 include/exclude 切换按钮（当有任何卡片选中时显示）

**删除**：独立的 Agent 筛选 chips 行

### 3.3 include/exclude 切换

当有任何 agent 卡片被选中时，在 agent 概览区下方显示切换按钮：

```
已选 Claude, Hermes    [包含] [排除] 切换
```

### 3.4 Agent 卡片选中状态视觉

| 状态 | 样式 |
|------|------|
| 未选中 | `border: var(--c-border)`, `background: var(--c-surface)` |
| 选中(include) | `border: var(--c-primary)`, `background: var(--c-primary-light)` |
| 选中(exclude) | `border: var(--c-danger)`, `background: var(--c-danger-light)` |

---

## 四、状态筛选 UI 优化

### 4.1 设计参考

参考 Material Design 3 Filter Chips + Ant Design Tag/TagGroup：

- **分组**：用 divider 分隔不同类别的筛选
- **图标**：每个 chip 前增加小图标，增强识别
- **计数**：每个 chip 右侧显示匹配的 skill 数量

### 4.2 新布局

```
[⚠ 冲突 (3)] [❌ 断链 (1)] │ [● 需同步 (5)] [○ 未链接 (4)] [● 已链接 (8)] │ [📋 重复 (2)]
  ─── 异常组 ───              ─── 状态组 ───                                   ─── 其他 ───
```

### 4.3 Chip 设计

```
┌──────────────────┐
│ ⚠ 冲突        3 │  ← 图标 + 文字 + 计数
└──────────────────┘
  选中态：实心背景
  未选中：描边 + 浅色背景
  置灰：opacity + pointer-events: none
```

**分组规则**：
- 异常组：`冲突`、`断链`（warning/danger 色系）
- 状态组：`需同步`、`未链接`、`已链接`（primary/secondary 色系）
- 其他：`未入库`、`仅库中`、`重复`

**每个 chip 的计数**：
```typescript
const chipCounts = computed(() => ({
  conflict: skillsStore.skills.filter(s => s.has_conflict).length,
  dangling: skillsStore.skills.filter(s => s.has_dangling).length,
  independent: skillsStore.skills.filter(s => s.sources.some(src => !src.is_symlink && src.from !== 'vibe-lib')).length,
  // ...
}));
```

### 4.4 清除筛选

当有任何筛选激活时，显示"清除筛选"按钮（放在 chips 行末尾）。

---

## 五、布局调整

### 5.1 新布局顺序

```
┌─────────────────────────────────────────────────────┐
│ Header: 软连接管理 (12/15)     [清除筛选] [安装][≡][⊞]│
├─────────────────────────────────────────────────────┤
│ Agent 概览 (最顶部)                                  │
│ [Claude✓] [Hermes] [Codex] [Pi] ...                │
│ 已选 Claude    [包含/排除切换]                        │
├─────────────────────────────────────────────────────┤
│ 状态: [⚠冲突(3)] [❌断链(1)] │ [●需同步(5)] ...     │
├─────────────────────────────────────────────────────┤
│ [需操作优先 ▾]  [搜索___________]                    │
├─────────────────────────────────────────────────────┤
│ 共 15 个 skill | 8 共享 | 4 独立 | 3 异常             │
├─────────────────────────────────────────────────────┤
│ skill list / card grid                              │
│                                                     │
│ ┌─────────────────────────────────────────────────┐ │
│ │ 已选 5 个    [同步到库] [取消选择]               │ │  ← 浮动栏
│ └─────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────┤
│ ▶ Agent-Skill 关系矩阵                     [展开]    │
└─────────────────────────────────────────────────────┘
```

### 5.2 各区块说明

| 顺序 | 区块 | 说明 |
|------|------|------|
| 1 | Header | 标题 + 计数 + 清除筛选 + 安装 + 视图切换 |
| 2 | Agent 概览 | 可折叠，默认展开。卡片=筛选入口 |
| 3 | 状态筛选 chips | 分组 + 图标 + 计数 |
| 4 | 工具行 | 排序 + 搜索 |
| 5 | Stats bar | 总数 / 共享 / 独立 / 异常 |
| 6 | Skill 列表/卡片 | 主内容区 |
| 7 | 浮动批量操作栏 | 选中 skill 时显示 |
| 8 | 关系矩阵 | 可折叠，默认收起 |

---

## 六、文件变更

### 删除

| 文件 | 原因 |
|------|------|
| 无 | 本轮不删除文件 |

### 修改

| 文件 | 变更 |
|------|------|
| `ManageTab.vue` | 合并 agent 筛选到概览卡片 + 浮动批量操作栏 + 状态 chips 优化 + 布局调整 |
| `SkillRow.vue` | 增加 skill checkbox + 选中态样式 |
| `SkillCard.vue` | 增加 skill checkbox + 选中态样式 |
| `src/locales/zh.json` | 新增 keys |
| `src/locales/en.json` | 新增 keys |
| `src/locales/zh-TW.json` | 新增 keys |

---

## 七、审查修正项（子 agent 审查后补充）

### 7.1 batchSkillAction silent 模式

`batchSkillAction` 内部已调用 `refreshSkills()` + `fetchAgents()`。批量循环调用时需增加 `silent` 参数跳过内部 refresh，循环结束后手动调一次。

### 7.2 SkillRow/SkillCard 接口定义

```typescript
// 新增 props
selected?: boolean;
// 新增 emits
(e: "toggle:select", skillId: string): void;
```

### 7.3 浮动栏 scroll padding

浮动栏 `fixed` 定位时，skill list 容器需增加 `padding-bottom: 56px` 补偿遮挡。

### 7.4 Chip 计数基于筛选后数据

计数应基于 `displaySkills`（应用了其他筛选条件后的列表），而非全量 `skillsStore.skills`。

### 7.5 Agent 筛选用点击 + 切换按钮

不用右键/长按。单击卡片切换选中，卡片下方显示 include/exclude 切换按钮。

### 7.6 ref<Set> 响应式更新

`selectedSkills.value.clear()` 不触发响应式更新，必须替换为新 Set。

---

## 八、实施计划

### Phase 1：Agent 概览卡片合并筛选（0.5 天）
1. 删除独立的 Agent chips 行
2. Agent 卡片增加选中态样式（include/exclude 两种高亮色）
3. Agent 卡片下方增加 include/exclude 切换按钮
4. 确保 Agent 卡片点击正确驱动筛选逻辑

### Phase 2：状态筛选 chips 优化（0.5 天）
1. 分组（divider 分隔异常组/状态组/其他）
2. 每个 chip 增加计数
3. 清除筛选按钮

### Phase 3：Skill 多选 + 浮动批量操作栏（1 天）
1. SkillRow 增加 checkbox
2. SkillCard 增加 checkbox
3. ManageTab 增加 `selectedSkills` ref
4. 浮动批量操作栏组件（或内联在 ManageTab）
5. 批量执行逻辑（`batchSkillAction` per skill）

### Phase 4：布局调整 + i18n（0.5 天）
1. 调整区块顺序
2. Stats bar 位置调整
3. i18n key 补全

---

## 九、验收标准

1. Agent 概览卡片在最顶部，点击即可筛选，无独立 agent chips 行
2. Agent 卡片有选中/排除两种视觉状态
3. 状态筛选 chips 有分组和计数
4. Skill 列表/卡片支持多选 checkbox
5. 选中 skill 后底部出现浮动批量操作栏
6. 批量操作使用 `batchSkillAction`，不循环调用 `syncToVibe`
7. 全选只作用于当前筛选后的列表
8. 清除筛选按钮在有筛选时显示
9. Build 通过
