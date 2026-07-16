# QS-Vibe 文档索引

## 通用文档

| 编号 | 文档 | 说明 |
| --- | --- | --- |
| 01 | [开发环境搭建](01-dev-environment.md) | Rust、Node、pnpm、Tauri 项目环境配置 |
| 02 | [模块规划](02-modules.md) | Skills 管理、Plugin/MCP/Marketplace 预留规划 |
| 03 | [开发需求清单](03-requirements.md) | 需求条目、依赖关系和开发顺序 |

## 版本计划

| 版本 | 文档 | 内容 | 状态 |
| --- | --- | --- | --- |
| v0.0 | [实施计划](v0.0/01-plan.md) | Skill 展示和 symlink 管理基础版 | 已完成 |
| v0.1 | [实施计划](v0.1/01-plan.md) | 安装、删除、预览、批量、撤销/重做、i18n、主题、设置 | 已完成 |
| v0.2 | [版本索引](v0.2/README.md) | 管理页体验优化、批量闭环、来源/目录视图、Project source、应用图标更新 | 当前版本 |

## v0.2 重点

- 将列表定位为 Skill 工作台。
- 将树视图定位为来源/目录视图。
- 将矩阵降级为高级批量/诊断工具。
- 补齐多 skill、多 agent 的批量操作 dry-run 闭环。
- 将 Project skill 作为只读来源纳入后续规划。
- 更新应用 skill 图标和平台图标资源。

## 目录结构

```text
docs/
├── README.md
├── 01-dev-environment.md
├── 02-modules.md
├── 03-requirements.md
├── v0.0/
│   └── 01-plan.md
├── v0.1/
│   ├── 01-plan.md
│   └── ...
└── v0.2/
    ├── README.md
    ├── 01-manage-experience-optimization.md
    └── 02-release-notes.md
```

