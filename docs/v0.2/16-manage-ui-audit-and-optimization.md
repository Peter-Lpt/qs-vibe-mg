# v0.2 管理页第二轮 UI 审计与优化

> 审计范围：样式统一、字体层级、滚动条、问题修复分组、树/列表一致性、路径展示、弹窗骨架和筛选滚动体验。

## 审计结论

- **P0：基础视觉统一**。颜色 token 已存在，但字体、滚动条、弹窗遮罩和容器间距此前仍有多套写法。
- **P0：问题修复可扩展**。问题组不应假设只有两项；需要响应式列、最大高度和内部滚动，避免整页被问题卡片推长。
- **P1：树/列表同风格**。两者都采用同一套 surface、border、radius、hover 和操作按钮语义；树节点仍保留层级关系。
- **P1：路径显示统一**。界面显示统一为 `/` 分隔，真实调用仍保留原始路径，避免破坏 Windows 操作。
- **P2：筛选固定**。只固定管理页控制台，不固定全局应用顶栏，避免与 Tauri 标题栏和弹窗层叠冲突。

## 本轮实现

- `src/style.css` 增加字体、字号、等宽路径、弹窗、树节点、操作组和滚动条 token/class。
- `ManageTab.vue` 的同步控制台增加局部 sticky，长列表滚动时保持搜索和筛选可达。
- `IssueRepairPanel.vue` 改为响应式单列/双列、最大高度和内部滚动，并展示待处理技能总数。
- `SkillTree.vue` 统一根节点、技能节点和操作按钮风格，路径显示统一为 `/`。
- `SkillRow.vue`、`SkillDetail.vue` 的路径展示统一为 `/`，仅改变显示，不改变实际文件操作路径。
- `ConfirmDialog.vue`、`InstallDialog.vue`、`AddAgentDialog.vue`、`SettingsPage.vue`、`BatchSyncPanel.vue` 和 Agent 管理弹窗统一使用 modal backdrop/shell/header/body/actions 骨架。

## 验收标准

- 复杂问题分组不会因为数量增加而无限拉长主页面。
- 树和列表的卡片、边框、圆角、按钮尺寸和 hover 反馈一致。
- 路径展示不再混用 `\\` 与 `/`。
- 管理页滚动时筛选控制台保持在内容区顶部；弹窗打开时不受 sticky 层干扰。
- `pnpm build`、`cargo check`、locale JSON 解析和 `git diff --check` 全部通过。
