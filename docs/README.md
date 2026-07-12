# VIBE Skills Manager - 文档索引

## 通用文档

| 编号 | 文档 | 说明 |
|------|------|------|
| 01 | [开发环境搭建](01-dev-environment.md) | Rust/Node/pnpm 安装、Tauri 项目初始化、Mac/Win 环境配置 |
| 02 | [模块规划](02-modules.md) | Skills 管理（当前）+ Plugin/MCP/Marketplace 预留 |
| 03 | [开发需求清单](03-requirements.md) | v0.1 所有需求项（R01-R21），含依赖关系和开发顺序 |

## 版本计划

| 版本 | 文档 | 内容 | 状态 |
|------|------|------|------|
| v0.0 | [实施计划](v0.0/01-plan.md) | Skill 展示 + Symlink 管理，架构骨架完整 | ✅ 已完成 |
| v0.1 | [实施计划](v0.1/01-plan.md) | 完整功能：安装/删除、预览、批量、撤销/重做、i18n、主题、设置 | ✅ 已完成 |

## 目录结构

```
docs/
├── README.md                ← 本文件
├── 01-dev-environment.md    ← 环境搭建
├── 02-modules.md            ← 模块规划
├── 03-requirements.md       ← 需求清单
├── v0.0/
│   └── 01-plan.md           ← v0.0 先行版计划（已完成）
└── v0.1/
    └── 01-plan.md           ← v0.1 完整版计划（已完成）
```
