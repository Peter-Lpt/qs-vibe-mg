# v0.2 Skill Workbench 重构

## 目标

弃用原有“列表 / 树 / 只读矩阵”三套主视图，改为一个以 Skill 为唯一行、以用户配置 Agent 为动态列的 Skill Workbench。

## 自适应规则

| Agent 数量 | 展示方式 |
| --- | --- |
| 0 | 添加 Agent 引导空状态 |
| 1 | 单 Agent 工作台，避免空矩阵 |
| 2-4 | 多 Agent 关系工作台 |
| 5+ | 横向滚动关系工作台，Skill 列和操作列保持可见 |

## 交互

- 每个 Skill 只显示一次，避免树结构在 Library、Agent、Project 下重复占用空间。
- Skill 行按“需要处理 / 正常”分组，问题数量增加时仍保持主列表可控。
- 点击 Agent 状态单元格或右侧操作可以展开行内 `SkillDetail`。
- 工作台展开详情使用嵌入模式，隐藏重复的来源总表；Agent 状态、来源路径和可执行动作集中在同一详情区域。
- 批量选择继续复用原有浮动批量栏和 `BatchSyncPanel`。
- 工作台标题和底部批量栏都支持“全选当前筛选结果”，不会只选当前分组或整库。
- Agent 超过四个时显示明确的横向滚动提示和可见滚动条。
- Agent 列完全来自 `agents` store，不写死 Agent 名称、数量或路径。
- 树视图和旧只读矩阵不再出现在管理页主流程；相关组件暂保留，后续可删除或改造成独立来源诊断页。

## 代码入口

- `src/components/manage/SkillWorkbench.vue`
- `src/components/manage/SkillWorkbenchRow.vue`
- `src/components/manage/ManageTab.vue`
- `src/composables/useSkillAgentStatus.ts`
- `src/composables/useSkillActions.ts`

## 验收重点

- 自定义 Agent 名称和数量可以直接渲染。
- 只有一个 Agent 时不出现宽阔空矩阵。
- Agent 超过四个时不压缩到不可读，支持横向滚动。
- Skill 详情、选择、批量操作和修复入口保持可用。
