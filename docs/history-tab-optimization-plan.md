# 操作历史独立 Tab 优化方案

## 现状分析

当前历史操作以 `HistoryBar.vue` 形式固定在 AppLayout 底部（`#bottom` slot），特点：
- 仅显示最近5条记录的滚动标签
- 提供 Undo/Redo 按钮
- 无搜索/过滤功能
- 无完整列表查看
- TabBar 仅有4个 Tab：cli/skills/dashboard/symlink

## 优化目标

1. 将历史操作独立为完整 Tab 页面
2. 保留底部精简版 HistoryBar 用于快速操作（在 History Tab 激活时隐藏）
3. 丰富历史显示、搜索、过滤、分页
4. 采用**纯堆栈模式**：全局 undo/redo 仅操作最新记录，HistoryTab 支持查看/搜索但单条 undo/redo 仅能操作最新条目
5. 支持清空历史

## 设计决策

### 纯堆栈模式（Stack Mode）

- **全局 undo()**: 总是撤销 `entries` 中最后一条 `undone=false` 的记录（LIFO）
- **全局 redo()**: 总是重做 `entries` 中最后一条 `undone=true` 的记录
- **单条 undoById/redoById**: 只允许操作最新的未撤销/已撤销记录（与全局等效），不允许操作中间条目
  - 如果尝试操作非最新条目，返回错误提示"请先处理更近期的记录"
- 保持 `canUndo = entries.some(e => !e.undone)` 语义不变

**理由**：避免复杂的中间条目撤销导致文件状态与历史记录不一致，降低实现复杂度和出错风险。

## 变更范围

### Phase 1 — 类型和 Store 增强

#### `src/types/index.ts`
- `TabId` 增加 `"history"` 类型

#### `src/stores/history.ts`
- 新增 `filteredEntries` 计算属性（按搜索/过滤条件筛选）
- 新增 `searchQuery` / `actionFilter` 状态
- 新增 `clearHistory()` 方法（调后端 `clear_history`）
- 新增 `undoById(id)` / `redoById(id)` 方法（调后端 `undo_by_id`/`redo_by_id`）
- `updateUndoRedoState()` 保持现有逻辑不变（已验证正确）

### Phase 2 — 前端组件

#### 新增 `src/components/history/HistoryTab.vue`
- 完整历史列表，按时间倒序排列（最新的在最上）
- 列表项包含：时间、操作类型（带图标）、Skill名、Agent名（如适用）、同步模式、状态（已撤销/正常）、操作按钮（撤销/重做）
- 搜索框：按 skill_id 搜索，placeholder 提示"按 Skill ID 搜索"
- 过滤下拉：按 action 类型过滤（全部/Link/Unlink/Install/Delete/BatchLink/BatchUnlink）
- 分页控制（每页20条，数据量≤500条时前端分页，无需后端分页接口）
- 清空历史按钮（带确认弹窗）
- 空状态展示

#### 更新 `src/components/history/HistoryBar.vue`
- 保留精简底部栏，仅显示 Undo/Redo 按钮 + 最近3条记录
- 增加"查看全部"链接跳转到历史 Tab
- **在 History Tab 激活时隐藏底部栏**
- 移除 `onMounted` 中单独 fetch（由 App.vue 统一管理，及其他 Tab 操作后刷新）

#### 更新 `src/components/layout/TabBar.vue`
- 增加第5个 Tab：`"history"`（图标 🕐）

#### 更新 `src/App.vue`
- 增加 `<HistoryTab v-else-if="appStore.activeTab === 'history'" />`
- 所有 Tab 操作后自动调用 `fetchHistory` 刷新 store（通过 watch 或 store action）

### Phase 3 — 后端增强

#### 新增 `src-tauri/src/commands/history.rs` 命令
- `clear_history`: 清空所有历史记录
- `undo_by_id(id: String)`: 按ID执行撤销（含文件操作 + 标记 undone=true），仅允许操作最新未撤销记录
- `redo_by_id(id: String)`: 按ID执行重做（含文件操作 + 标记 undone=false），仅允许操作最新已撤销记录

#### 新增 `src-tauri/src/utils/history.rs` 函数
- `clear_history()` → 清空 `entries` 并保存
- 验证逻辑：检查要操作的记录是否为对应堆栈顶

#### 更新 `src-tauri/src/lib.rs`
- 注册 `clear_history`、`undo_by_id`、`redo_by_id`

### Phase 4 — 国际化

#### `zh.json` / `en.json` / `zh-TW.json`
新增以下 keys:
- `tabs.history`: "操作历史" / "History" / "操作歷史"
- `history.tab_title`: "操作历史" / "History" / "操作歷史"
- `history.search_placeholder`: "按 Skill ID 搜索..." / "Search by Skill ID..." / "按 Skill ID 搜尋..."
- `history.filter_all`: "全部类型" / "All Types" / "全部類型"
- `history.clear`: "清空" / "Clear" / "清空"
- `history.clear_confirm`: "确定清空所有历史记录？此操作不可恢复。" / "Clear all history? This cannot be undone." / "確定清空所有歷史記錄？此操作不可恢復。"
- `history.no_results`: "无匹配记录" / "No matching records" / "無匹配記錄"
- `history.entry_action_undo`: "撤销" / "Undo" / "撤銷"
- `history.entry_action_redo`: "重做" / "Redo" / "重做"
- `history.view_all`: "查看全部 →" / "View All →" / "檢視全部 →"
- `history.status_normal`: "正常" / "Active" / "正常"
- `history.status_undone`: "已撤销" / "Undone" / "已撤銷"

## 数据流

```
App.vue ──activeTab──> TabBar.vue
               │
               ├─ "history" ──> HistoryTab.vue
               │                   ├─ invoke("get_history")  ← reads .vibe-history.json
               │                   ├─ invoke("undo") / invoke("redo")  ← 全局（堆栈模式）
               │                   ├─ invoke("undo_by_id") / invoke("redo_by_id")  ← 单条
               │                   └─ invoke("clear_history")
               │
               └─ #bottom ──> HistoryBar.vue (精简版, 非history tab时显示)
                                  ├─ invoke("undo") / invoke("redo")
                                  └─ 点击"查看全部" → appStore.setActiveTab("history")
```

### 数据刷新闭环
- 所有 Tab 中的 link/unlink/install/delete 操作完成后 → 触发 `historyStore.fetchHistory()`
- HistoryTab 自身操作（undo/redo/clear）完成后 → 自动 `fetchHistory()`
- Undo/Redo 后 → 同时调用 `skillsStore.fetchSkills()` 保持技能列表同步

## UI 设计草图

```
[TabBar: 🔧 CLI | 📁 Skills | 📊 Dashboard | 🔗 Symlink | 🕐 History]
[HistoryTab 内容]
┌─────────────────────────────────────────────────────────────────────┐
│ [按 Skill ID 搜索...]          [全部类型 ▼]          [🗑 清空]     │
│                                                                     │
│ ┌─ 时间 ─────────┬─ 操作 ─┬─ Skill ──────┬─ Agent ────┬─ 状态 ─┬─┐ │
│ │ 2024-01-15     │ ↗安装   │ my-skill     │ Claude     │ ✓正常  │⟲│ │
│ │ 14:32:15       │        │              │            │        │ │ │
│ ├────────────────┼────────┼──────────────┼────────────┼────────┼─┤ │
│ │ 2024-01-15     │ ↗关联   │ tool-x       │ Hermes     │ ✓正常  │⟲│ │
│ │ 14:30:01       │        │              │            │        │ │ │
│ ├────────────────┼────────┼──────────────┼────────────┼────────┼─┤ │
│ │ 2024-01-15     │ ↘取消   │ tool-x       │ PiAgent    │ ○已撤销│↰│ │
│ │ 14:28:44       │        │              │            │        │ │ │
│ ├────────────────┼────────┼──────────────┼────────────┼────────┼─┤ │
│ │ 2024-01-15     │ 批量关联│ 3 skills     │ OpenCode   │ ✓正常  │⟲│ │
│ │ 14:25:00       │        │              │            │        │ │ │
│ └────────────────┴────────┴──────────────┴────────────┴────────┴─┘ │
│                    [← 上一页] 第1/5页 [下一页 →]                    │
└─────────────────────────────────────────────────────────────────────┘

[底部 HistoryBar (仅在非 History Tab 时显示)]
↩撤销 ↪重做  |  14:32 安装 my-skill  |  14:30 关联 tool-x  |  [查看全部 →]
```

## 审核记录

### 第一次审核（已完成）
审核方提出了以下关键改进：
1. **P0—数据一致性**：单条 undo/redo 必须包含文件系统操作，不能仅标记 JSON 字段
2. **P1—语义模型**：采用**纯堆栈模式**，避免中间条目操作导致不一致
3. **P1—命名统一**：后端使用 `undo_by_id` / `redo_by_id` 而非 `mark_undone_by_id`
4. **P2—冗余UI**：History Tab 激活时隐藏底部 HistoryBar
5. **P2—搜索提示**：搜索框提示按 Skill ID 搜索

以上改进已全部纳入本方案。

### 第二次审核（待完成）

## 待办检查清单

- [ ] Phase 1: 类型定义更新 (TabId + history)
- [ ] Phase 1: Store 增强 (搜索/过滤/清空/undoById/redoById)
- [ ] Phase 2: HistoryTab 组件创建
- [ ] Phase 2: HistoryBar 精简改造 + History Tab 时隐藏逻辑
- [ ] Phase 2: TabBar 增加 history Tab
- [ ] Phase 2: App.vue 集成
- [ ] Phase 3: 后端命令 (clear_history, undo_by_id, redo_by_id)
- [ ] Phase 3: 后端工具函数
- [ ] Phase 3: lib.rs 注册命令
- [ ] Phase 4: 三语言 locale 更新
- [ ] 验证: cargo check 编译通过
- [ ] 验证: pnpm build 前端构建通过