# QS-Vibe 项目改进方案

> 基于功能体验官、UI 设计师、项目经理三个视角的综合分析
> 日期：2026-07-04

---

## 一、项目现状概述

QS-Vibe 是一个跨平台 AI Agent Skill 统一管理工具，核心功能包括：
- **Agent 管理**：自动检测 + 自定义添加
- **Skill 列表**：搜索、预览、安装、删除
- **看板**：可视化 Agent 间 Skill 分布与共享关系
- **软连接配置**：层级式批量同步
- **操作历史**：撤销/重做

技术栈：Tauri 2 + Rust + Vue 3 + TypeScript + Tailwind CSS 4

---

## 二、功能缺失分析

### 2.1 高优先级（核心体验缺口）

| 缺失功能 | 问题描述 | 影响 |
|---------|---------|------|
| **批量操作 UI** | `skills.ts` Store 已实现 `selectedIds`/`toggleSelect`/`batchLink`/`batchUnlink`，但 SkillList.vue 完全没有选择复选框和批量操作栏 | 后端能力浪费，用户需逐个操作 |
| **全局 Toast 通知** | 三处独立实现：SyncPreview 本地 toast、HistoryTab 的 operationMessage、AgentCard 的 `alert()` | 体验碎片化，维护成本高 |
| **Skill 版本更新检测** | 无法追踪 skill 版本变化，源 skill 更新后无通知 | 用户需手动重新同步 |

### 2.2 中优先级（体验增强）

| 缺失功能 | 问题描述 |
|---------|---------|
| **远程 Skill 仓库** | 只能从本地安装，无法从 GitHub 等远程源直接安装 |
| **键盘快捷键** | 无 `Escape` 关闭弹窗、无 `Ctrl+Z` 撤销、无 `Ctrl+F` 搜索 |
| **对话框焦点管理** | 所有对话框无 focus-trap，Escape 不关闭，Tab 可逃逸 |
| **SymlinkTab 搜索过滤** | Agent 拥有大量技能时，树形列表无搜索功能 |
| **加载骨架屏** | 所有加载状态仅显示 "Loading..." 文本 |
| **空状态引导** | 空状态仅有简单文字提示，缺少引导步骤 |

### 2.3 低优先级（长期规划）

- Skill 标签/分类系统
- 拖拽关联操作（Skill 卡片拖到 Agent 卡片）
- 数据导出/导入（配置 + Skill 元数据的 JSON 打包）
- Skill 模板创建
- 排序功能（名称、日期、来源 Agent）

---

## 三、功能冗余分析

### 3.1 需要调整的功能

| 功能 | 问题 | 建议 |
|-----|------|------|
| **操作历史撤销/重做** | 功能体验官认为使用率低，但后端已投入约 250 行代码 | **保留并修复**：Delete 的 undo 返回错误 "Cannot undo delete without snapshot"，需实现 Skill 快照机制 |
| **看板"关联图谱"** | 实际是矩阵交叉表（127 行），非可视化图谱 | **重命名为"共享矩阵"**，保持现状，升级为 D3/Canvas 的 ROI 极低 |

### 3.2 可以移除/简化的内容

| 内容 | 位置 | 原因 |
|-----|------|------|
| `check_link_status` 后端 API | `lib.rs:21` 已注册但前端从未调用 | 死代码，可移除或标记 deprecated |
| `update_config` 后端 API | `lib.rs:40` 已注册但前端用 localStorage 替代 | 配置不一致风险，应统一方案 |

---

## 四、UI/UX 问题分析

### 4.1 代码质量问题（应修复）

| 问题 | 严重度 | 涉及范围 |
|-----|--------|---------|
| **内联 hover 样式泛滥** | 高 | 约 246 处 `@mouseenter`/`@mouseleave` JS 操控样式，遍布 12+ 组件 |
| **硬编码颜色值** | 高 | 29 处 hex 值（`#e67e22`、`#27ae60` 等），暗色模式下对比度失调 |
| **重复确认对话框** | 中 | HistoryTab、SettingsPage 各有内联弹窗，未复用 ConfirmDialog.vue |
| **重复的树行模板** | 中 | SyncPreview.vue 中"展开文件夹内 skill"和"根级 skill"模板几乎相同 |

### 4.2 交互设计问题

| 问题 | 描述 |
|-----|------|
| **SkillCard 删除按钮隐藏过深** | `opacity-0 group-hover:opacity-100`，新用户可能不知道可以删除 |
| **图标策略不一致** | TabBar 用 Emoji，Header 设置用 SVG，删除用 SVG，文件夹用 Emoji，混用降低视觉一致性 |
| **TabBar 视觉权重不足** | 极简设计（小字 + emoji），作为主导航应更突出 |
| **操作反馈缺失** | 安装成功后对话框直接关闭无提示，删除后无 Toast 确认 |

### 4.3 信息架构问题

| 问题 | 描述 |
|-----|------|
| **Symlink 与 Link 概念混淆** | Skills 中的 Link（链接）和 Symlink 中的 Sync（同步）是两种关联方式，用户难以理解区别 |
| **Dashboard 功能较弱** | 仅展示分布数据，可考虑整合到 Skills 或 Agents 标签 |
| **设置页面内容过少** | 仅主题、语言、路径三个配置，缺少"关于"信息、检查更新、高级配置 |

---

## 五、技术债务发现

| 类型 | 描述 | 位置 |
|-----|------|------|
| 文档与代码不同步 | `ErrorBanner.vue` 在 AGENTS.md 中声明存在但实际不存在 | AGENTS.md:51 |
| i18n 硬编码 | `"Please enter a source path"` 硬编码英文 | InstallDialog.vue:35 |
| i18n 硬编码 | `ConfirmDialog.vue` 的 fallback 为英文 `'Cancel'`/`'Confirm'` | ConfirmDialog.vue:39,49 |
| 死代码 | `check_link_status` 已注册但未调用 | lib.rs:21 |
| 配置不一致 | `update_config` 已注册但前端用 localStorage | lib.rs:40 |
| 测试覆盖不足 | Rust 后端仅 9 个单元测试，前端零测试 | - |

---

## 六、改进优先级路线图

### Phase 1：代码质量基础（1-2 周，低风险高收益）

1. **实现全局 Toast 通知系统** — 创建 `useToast()` composable + `ToastContainer.vue`，替换 3 处独立实现
2. **硬编码颜色治理** — 29 处 hex 值 → CSS 变量，确保暗色模式完整性
3. **内联样式重构** — 创建 `.btn-primary`、`.card-hover` 等 Tailwind 工具类，逐步替换 246 处内联 hover
4. **ConfirmDialog 统一** — HistoryTab、SettingsPage 的内联弹窗复用 ConfirmDialog.vue
5. **修复 i18n 硬编码** — InstallDialog.vue:35、ConfirmDialog.vue:39,49

### Phase 2：交互体验增强（2-3 周，中等复杂度）

6. **实现批量操作 UI** — Store 层已就绪，SkillList 添加 checkbox 多选 + 批量操作栏
7. **键盘快捷键** — `Ctrl+K` 搜索、`Escape` 关闭、`Delete` 删除、`Ctrl+Z` 撤销
8. **对话框焦点管理** — 统一 Modal 的 focus-trap、Escape 关闭、Tab 循环
9. **加载骨架屏** — 替换纯文本 "Loading..." 为带动画的 skeleton 占位符
10. **空状态引导优化** — 添加引导性 CTA 按钮和分步引导

### Phase 3：功能增强（3-4 周，需架构决策）

11. **Delete 的 undo 支持** — 实现 Skill 快照机制（删除前复制到临时目录）
12. **SymlinkTab 搜索过滤** — Agent 列表 + Skill 树的快速过滤
13. **数据导出/导入** — 配置 + Skill 元数据的 JSON 导出/导入
14. **Skill 版本更新检测** — 基于文件哈希的变更检测，显示"有更新"提示

### Phase 4：v2.0 规划（独立项目）

15. 远程 Skill 仓库支持（GitHub URL 安装）
16. Skill 标签/分类系统
17. 信息架构重组（合并/重新划分标签页）
18. 拖拽操作支持

---

## 七、功能改进总结

### 应立即实施（收益高、风险低）

| 改进项 | 类型 | 理由 |
|-------|------|------|
| 全局 Toast 通知 | 功能补全 | 消除 3 处重复实现，统一体验 |
| 批量操作 UI | 功能补全 | Store 层已就绪，只差 UI 暴露 |
| 硬编码颜色修复 | 质量修复 | 暗色模式下对比度严重失调 |
| 内联 hover 重构 | 质量修复 | 246 处重复代码，性能差且臃肿 |

### 应保留并改进（不废弃）

| 功能 | 理由 |
|-----|------|
| 操作历史撤销/重做 | 后端已投入大量实现成本，应修复 Delete 的 undo 能力 |
| 看板共享矩阵 | 重命名即可，升级为可视化图谱 ROI 极低 |

### 应延后或放弃

| 功能 | 决策 | 理由 |
|-----|------|------|
| 远程 Skill 仓库 | 延后至 v2.0 | 需要网络层、认证、版本同步等独立子系统 |
| 拖拽关联操作 | 延后 | Tab + 表单交互已够用，实现复杂 |
| 信息架构重组 | 放弃 | 5 个 Tab 逻辑清晰，重组成本高风险大 |
| 自定义 Tooltip 组件 | 放弃 | 浏览器原生 title 属性已足够 |

---

## 八、附录：关键数据

- **前端组件数**：19 个 Vue 组件
- **Rust 后端模块**：commands(5) + models(5) + parsers(1) + utils(5) + errors(1)
- **支持的 Agent**：7 个（Claude Code、Hermes、Pi Agent、OpenCode、Codex CLI、MiMo Code、Shared）
- **i18n 语言**：3 种（中文简体、English、繁體中文）
- **Rust 单元测试**：9 个（path.rs:2、fs.rs:1、datetime.rs:2、skill_md.rs:4）
- **内联 hover 样式**：约 246 处
- **硬编码颜色值**：29 处
- **Toast 独立实现**：3 处
