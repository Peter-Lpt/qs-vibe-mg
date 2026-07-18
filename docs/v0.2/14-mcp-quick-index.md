# QS-Vibe MCP / Agent 速读索引

> 这是给 agent 快速定位功能的阅读路径，不是运行时配置。

## 优先阅读

### 1. 安装 / 更新

- `src-tauri/src/commands/skills.rs`
- `src-tauri/src/utils/origin.rs`
- `src/components/skills/InstallDialog.vue`
- `src/components/manage/SkillDetail.vue`
- `src/stores/skills.ts`

重点看：

- `install_skill`
- `install_skill_from_source`
- `update_skill`
- `update_from_git_source`
- `update_from_command_source`
- `build_install_origin`
- `build_git_origin`
- `build_command_origin`

### 2. 同步 / 链接

- `src-tauri/src/commands/sync.rs`
- `src-tauri/src/utils/fs.rs`
- `src/components/manage/SkillDetail.vue`
- `src/components/manage/BatchSyncPanel.vue`

### 3. 来源和 provenance

- `src-tauri/src/models/origin.rs`
- `src-tauri/src/utils/origin.rs`

重点看：

- `SkillOrigin`
- `read_skill_origin`
- `write_skill_origin`
- `write_skill_origin_sidecar`
- `update_status_for`

### 4. 技能列表和冲突检测

- `src-tauri/src/commands/skills.rs`
- `src-tauri/src/models/skill.rs`
- `src/components/manage/ManageTab.vue`

### 5. 配置 / 目录发现

- `src-tauri/src/commands/config.rs`
- `src-tauri/src/utils/config.rs`
- `src-tauri/src/utils/path.rs`

## 速查目录

| 想查的东西 | 先看这里 |
| --- | --- |
| 技能从哪来 | `src-tauri/src/utils/origin.rs` |
| 安装入口在哪 | `src/components/skills/InstallDialog.vue` |
| 更新按钮怎么判断 | `src/components/manage/SkillDetail.vue` |
| 后端命令注册在哪 | `src-tauri/src/lib.rs` |
| symlink / copy 怎么做 | `src-tauri/src/utils/fs.rs` |
| 目录扫描怎么做 | `src-tauri/src/commands/skills.rs` |
| 配置路径怎么找 | `src-tauri/src/utils/path.rs` |

## 建议阅读路径

1. `docs/v0.2/12-multi-source-skill-install-update-technical-plan.md`
2. `src-tauri/src/models/origin.rs`
3. `src-tauri/src/utils/origin.rs`
4. `src-tauri/src/commands/skills.rs`
5. `src/components/skills/InstallDialog.vue`

