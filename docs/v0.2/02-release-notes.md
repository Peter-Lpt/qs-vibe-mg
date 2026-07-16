# QS-Vibe v0.2 Release Notes

> 版本：0.2.0  
> 日期：2026-07-16  
> 状态：已整理为版本升级提交

## 版本定位

v0.2 是一次以管理页体验为核心的版本升级，重点不是新增大量运行时代码，而是明确后续产品架构、更新应用识别资产，并为多 agent、多 skill 的批量管理闭环建立文档基线。

## 用户可见变化

- 应用版本显示更新为 `v0.2`。
- 应用图标更新为新的 skill 图标。
- Tauri 平台图标资源已重新生成，包括 Windows `.ico`、macOS `.icns`、PNG 和 Windows Store logo 尺寸。

## 文档变化

- 新增 `docs/v0.2/README.md` 作为 v0.2 文档索引。
- 新增 `docs/v0.2/01-manage-experience-optimization.md`，沉淀管理页体验优化方案。
- 新增本 release notes。
- 更新 `docs/README.md`，将 v0.2 纳入版本计划。

## 设计决策

- 列表视图定位为 Skill 工作台，负责管理决策。
- 树视图改为来源/目录视图，负责位置排查。
- 矩阵不再作为主流程展示组件，后续应降级为高级诊断或升级为可操作批量矩阵。
- 批量操作应具备 dry-run 预览、执行明细和失败重试能力。
- Project skill 应作为独立 source 纳入，但默认只读，避免污染 git repo。
- 来源识别必须显示置信度，不将推断当作确定事实。

## 实现说明

- `package.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`src-tauri/tauri.conf.json` 版本号更新为 `0.2.0`。
- `src/locales/*` 中应用版本文案更新为 `v0.2`。
- 新增 `src/assets/skill-icon.svg` 和 `src/assets/skill-icon.png` 作为图标源资产。
- 使用 `pnpm tauri icon src/assets/skill-icon.png` 重新生成 `src-tauri/icons/` 下的平台图标。

## 验证记录

- `cargo check` 已通过。
- 图标 PNG 已本地检查，渲染正常。
- 未运行完整 `pnpm build`，因为本次主要是文档、版本号和图标资产更新。

