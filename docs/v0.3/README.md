# QS-Vibe v0.3 Docs

v0.3 主题：**前端 UI 交互重构**（过滤体系、Plugin 解耦、批量操作、信息架构）。后端命令不变，仅允许配置级微调。

## Index

| No. | Doc | Notes |
| --- | --- | --- |
| 01 | [UI 交互重构设计（v1 初稿）](./01-ui-interaction-redesign.md) | 背景/痛点/方案雏形；已被 v2 取代，仅留档 |
| 01.v2 | [UI 交互重构设计（v2 事实核对版）](./01-ui-interaction-redesign.v2.md) | Round 1 审查：5 处事实修正 + Q1–Q5 裁定 + 风险清单 |
| 01.v3 | [UI 交互重构设计（v3.1 实施稿）](./01-ui-interaction-redesign.v3.md) | **最终版**：组件契约 / Composable 签名 / 数据模型 / 伪代码 / i18n 计划 / P0–P3 任务卡 / 验收清单 / 生命周期适配 / 跨平台 / 性能预算 |

## 方案一句话

应用级**左侧导航（智能视图）+ 右侧内容区**：过滤条件升级为侧边栏视图预设，高级组合收进工具栏筛选弹层；Plugin 技能经互斥 `domain` 维度独立成一等视图；批量操作由 Modal 改为**右侧抽屉三步流**（选动作 → 选目标矩阵 → 预览执行）；顶部 TabBar 废弃，历史记录成为侧边栏视图。

## 关键约束

- 判定同源：状态/动作只由 `useSkillAgentStatus`、`manageFilters` 谓词、`skillActionRegistry`、新增 `useBatchCellActions` 产出。
- 不加后端命令；批量无 force 参数是结构事实，UI 按"诚实失败"设计。
- domain 谓词与移除 `matchesStatusPreset` 插件硬排除必须在 P1 内**原子交付**。

## 决策记录（2026-07-24 定案，见 v3.1 §20）

1. 遗留 i18n key `batch_repair_uncovered/only_agent` → **删除**，P2 用新的 `batch.repair_*` 系列替代。
2. `tauri.conf.json` → **加** `minWidth: 800 / minHeight: 480`（P0）。
3. 侧边栏分组名 → **新增 `sidebar.*` 全套 key**（`sidebar.skills_library`="技能库"），`tabs.*` 随 TabBar 清理。
4. `project_only` 谓词 → **首期不做**，"仅项目"组视图内名单展示；进 v0.4 backlog。

## v3.1 增补（2026-07-24）

- **§17 生命周期与弃用/禁用架构适配**：依据《Skill 的弃用与禁用逻辑》调研，引入 `SkillLifecycle`（active/disabled/stale/archived/deprecated）与 `DisableCapability` 模型。v0.3 只做架构预留（serde 可选字段、filter `lifecycle` 维度占位、SkillRow 徽标插槽、action registry 槽位、`lifecycle.*` i18n 命名空间）；读取探测与写入操作进 v0.4+ backlog——落地时无需再大改架构。
- **§18 跨平台适配**：symlink/junction/copy 三 mode 透传、路径比较小写化对齐、Ctrl/Cmd 统一守卫、mac traffic lights 避让、双平台验收矩阵。
- **§19 性能预算**：查询（冷扫 <3s / 缓存 <500ms）、刷新（facet <16ms，分桶+惰性计数）、执行（串行合同 + 进度 x/y，预留后端并行）、渲染（>300 项虚拟滚动、矩阵 1000 格上限）。
