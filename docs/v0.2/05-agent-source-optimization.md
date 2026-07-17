# Agent source 与参考项目优化建议

> 范围：当前 QS-Vibe agent/source 模型、`~/.agents/skills` 命名、自定义 agent 配置、`F:\workspace\demo\ref\Product-Manager-Skills` 与 `F:\workspace\demo\ref\skills-manager` 两个参考项目。

## 1. 结论摘要

当前项目已经具备多 agent 管理、自定义 agent、Library/Agent/Project source 区分、冲突/断链检测与批量操作基础。下一步最值得优化的不是继续扩大支持列表，而是把“agent 目标”“共享/发现目录”“project source”“library source”这几类概念拆清楚。

建议把默认配置里的 `agents-shared` / `Shared` 改成更准确的“Agents Common Skills”。它表达的是 `~/.agents/skills` 这个公共 skills 目录，而不是一个真实 agent。中文可显示为“Agents 公共技能目录”或“公共 Agents Skills”。英文显示名建议用 `Agents Common`，id 建议用 `agents-common`。

同时，不建议把 `~/.agents/skills` 默认宣传成“所有 agent 都会读取”。它更适合作为一个 common/discovery source：有些工具可能读取，有些工具只读取自己的专属目录，有些工具可通过环境变量、插件或版本差异读取。QS-Vibe 应该在 UI 和数据模型里明确“检测到目录存在”与“目标 agent 实际识别”是两回事。

## 2. `shared` 命名优化

### 当前问题

当前默认 agent 配置位于 `src-tauri/src/utils/config.rs`：

```rust
AgentConfig {
    id: "agents-shared".to_string(),
    name: "Shared".to_string(),
    skills_dir: "~/.agents/skills".to_string(),
    enabled: true,
    auto_detected: true,
}
```

`Shared` 这个名字的问题是语义过轻：用户看不出它是哪个目录，也容易误解成“系统确认所有 agent 共享”。但 QS-Vibe 当前只是把它当成一个 `skills_dir` 扫描目标，并不能证明任何特定 agent 会自动加载这个目录。

### 推荐命名

优先推荐：

| 字段 | 当前 | 推荐 |
| --- | --- | --- |
| id | `agents-shared` | `agents-common` |
| 英文名 | `Shared` | `Agents Common` |
| 中文名 | `Shared` | `Agents 公共技能目录` |
| 繁中名 | `Shared` | `Agents 公共技能目錄` |
| 路径 | `~/.agents/skills` | 保持不变 |

备选名称：

| 名称 | 适合度 | 说明 |
| --- | --- | --- |
| `Agents Common` | 高 | 准确表达 `.agents` 下公共目录，不承诺一定被读取 |
| `Common Skills` | 中 | 简洁，但没有体现 `.agents` 语境 |
| `Agents Shared Library` | 中 | 易懂，但仍可能暗示强共享能力 |
| `Agent Skills Hub` | 低 | 偏产品化，不如 Common 准确 |

文案建议：

- 列表显示名：`Agents Common`
- tooltip/说明：`公共 skills 目录。部分 agent 可能读取该位置；是否生效取决于具体 agent 的规则。`
- 状态标签：不要用“已安装/已识别 agent”，改成“目录存在/目录缺失”或“公共目录”。

### 兼容迁移

如果改 id，需要兼容已有用户配置和历史记录：

1. 读取配置时将内置默认的 `agents-shared` 迁移为 `agents-common`。
2. 历史记录、linked agent、source `from` 中的旧 id 可以显示层兼容：旧 id 显示为 `Agents Common (legacy)` 或映射到新显示名。
3. 不建议直接删除旧 id 的历史，否则 undo/redo 和旧配置里的链接状态可能找不到 agent。

更稳的短期方案是只改 `name`，保留 `id = "agents-shared"`；等 v0.3 做配置迁移时再改 id。

## 3. 自定义 agent 支持与配置方式

### 当前已支持

当前项目已经支持自定义 agent：

- 后端命令：`add_custom_agent(name, skills_dir)`、`update_agent(...)`、`remove_custom_agent(...)`
- 前端入口：`AddAgentDialog.vue`
- 配置位置：`~/.vibe-skills/.vibe-config.json`
- 配置字段：`id`、`name`、`skills_dir`、`enabled`、`auto_detected`

示例配置：

```json
{
  "version": 1,
  "agents": [
    {
      "id": "my-agent",
      "name": "My Agent",
      "skills_dir": "D:\\tools\\my-agent\\skills",
      "enabled": true,
      "auto_detected": false
    }
  ]
}
```

用户也可以通过 UI 添加：输入 agent 名称和 skills 目录。当前 id 由名称自动 slug 化生成。

### 可能识别不到的原因

会。当前“识别到”只代表 `skills_dir` 路径存在，不代表 agent 运行时会读取该目录。常见失败原因：

1. 用户填的是 agent 根目录，不是 skills 目录。
2. 目标 agent 的真实 skills 目录和默认路径不同。
3. 目标 agent 只读取项目级目录，不读取全局目录。
4. 目标 agent 不支持 symlink，或 Windows Developer Mode/权限不足导致链接失败。
5. 目标 agent 需要重启、刷新索引或执行命令后才加载新 skill。
6. 目标 agent 对 `SKILL.md` frontmatter、目录层级、文件名大小写有额外要求。
7. 某些公共目录只是发现回退，不是官方加载路径。

### 建议增强配置模型

参考 `skills-manager` 的 `ToolAdapter`，QS-Vibe 可以把 AgentConfig 从单一路径扩展为更明确的能力描述：

```ts
interface AgentConfig {
  id: string;
  name: string;
  skills_dir: string;
  enabled: boolean;
  auto_detected: boolean;
  kind?: "agent" | "common" | "project" | "external";
  detect_dir?: string;
  project_skills_dir?: string;
  additional_scan_dirs?: string[];
  recursive_scan?: boolean;
  sync_mode?: "symlink" | "copy";
  verification_hint?: string;
}
```

优先落地字段：

| 字段 | 价值 |
| --- | --- |
| `kind` | 区分真实 agent、公共目录、project source、外部目录 |
| `detect_dir` | 判断工具是否安装，避免仅凭 skills 目录存在误判 |
| `project_skills_dir` | 支持 `.codex/skills`、`.opencode/skills` 这类项目级路径 |
| `additional_scan_dirs` | 支持只扫描不写入的发现目录 |
| `sync_mode` | 某些工具或系统环境下默认 copy 更可靠 |

## 4. 参考项目一：Product-Manager-Skills

这个项目不是 manager，而是高质量 skills 内容库。可参考点主要在“内容组织、校验、发布、平台适配文档”。

### 可借鉴建议

1. 建立 machine-readable catalog。

   它有 `catalog/skills-index.yaml`、`skills-by-type.md`、`commands-index.yaml`，并通过脚本生成。QS-Vibe 可以为本地 Library 生成 `.vibe-catalog.json`，缓存 name、description、type、tags、source、hash、updated_at，用于搜索、筛选、导出和 diff。

2. 增加 skill 质量检查。

   它有 `check-skill-metadata.py`、`check-skill-triggers.py`、`test-a-skill.sh`、`test-library.sh`。QS-Vibe 可以在导入/扫描时给出质量告警：缺 `name`、description 太短、缺 `SKILL.md`、缺使用场景、frontmatter 非标准、目录过大、引用文件缺失等。

3. 支持“按用途找 skill”。

   它强调通过 trigger language、场景、类型来找 skill。QS-Vibe 当前主要按 name/description 搜索，可以增加 `metadata.type`、`best_for`、`scenarios`、`tags` 的搜索与过滤。

4. 保持跨平台 skill 内容格式。

   它避免强绑定某个 agent 的模板语法，强调 plain-language input。QS-Vibe 导入时可以提示“这个 skill 是否含平台专属语法”，比如 `$ARGUMENTS`、Claude-only hooks、Codex-only plugin 元数据。

5. 增加打包/导出策略。

   它支持构建不同平台包。QS-Vibe 可增加“导出为 Claude upload zip / Codex directory / generic skill folder”的向导，而不是只有 symlink/copy 到 agent 目录。

### 不建议直接照搬

- 不需要把 PM Skills 的内容分层直接搬进产品信息架构；QS-Vibe 是通用 manager，不是单领域内容库。
- 不需要优先做复杂命令系统；除非 QS-Vibe 后续要管理 slash commands。

## 5. 参考项目二：skills-manager

这个项目和 QS-Vibe 方向高度接近，参考价值更高。它已经产品化了 Library、Global Workspace、Project Workspace、Linked Workspace、Preset、CLI、Git backup、marketplace 等概念。

### 可借鉴建议

1. ToolAdapter 模型。

   它把 agent/tool 的 key、display_name、skills_dir、detect_dir、additional_scan_dirs、recursive_scan、project_relative_skills_dir、custom flag 拆开。QS-Vibe 当前 `AgentConfig` 过于扁平，建议逐步演进为 adapter-like 模型。

2. 区分 installed 与 path exists。

   `skills-manager` 对 custom path 的判断比较清楚：用户显式给 path 时可认为可管理，但内置工具需要 detect_dir 证明安装。QS-Vibe 也应该区分：

   - `directory_exists`
   - `tool_detected`
   - `can_manage`
   - `runtime_verified`

3. `~/.agents/skills` 作为 discovery fallback。

   参考项目对 Codex 的处理很重要：专属目录是主要目标，`~/.agents/skills` 更像额外扫描来源，而不是默认写入目标。这支持 QS-Vibe 把 `Agents Common` 改成 common source，而不是 agent。

4. Project Workspace。

   QS-Vibe 已经扫描 `.codex/skills`、`.agents/skills`、`skills`，但当前依赖后端 `current_dir()`。建议学习 `skills-manager` 的 project workspace：用户显式添加项目根，并保存项目配置；扫描项目级 skills 时要显示项目名、agent 适配器、只读/可写策略。

5. Linked Workspace。

   对“任意目录作为 skills root”的需求，独立建 workspace 比伪装成 agent 更清晰。QS-Vibe 可以把自定义目录分成：

   - Custom Agent：目标工具会读取，可同步写入。
   - Linked Workspace：只是外部 skills 集合，可扫描、导入、导出，不默认参与 agent sync。

6. Preset。

   预设 skill 组合适合解决“给某个 agent 一次性装一组技能”的场景。QS-Vibe 现有批量操作可以先轻量支持“收藏/分组”，后续再做 preset apply。

7. Sync engine 的安全边界。

   参考项目在复制前检查 source/destination 是否互相包含，Windows symlink 失败时尝试 junction，再 fallback copy。QS-Vibe 已有跨平台 fs 工具，但可以补齐“目标在源目录内/源在目标目录内”的防护和 copy fallback 说明。

8. CLI。

   如果 QS-Vibe 要服务 agent 自动化，CLI 很有价值。优先级低于 UI 模型整理，但长期建议提供：

   ```bash
   qs-vibe agents list --json
   qs-vibe skills list --json
   qs-vibe skills sync --agent codex --dry-run
   qs-vibe skills adopt ~/.codex/skills --dry-run
   ```

### 不建议直接照搬

- GitHub 登录、自动备份、skill-aware merge 是大功能，当前阶段不应优先。
- Marketplace/AI search 需要网络、账号、内容源治理，建议在本地模型稳定后再做。
- SQLite 会带来迁移成本；QS-Vibe 目前 JSON 配置足够，短期不必引入数据库。

## 6. 当前项目优化整理

### P0：命名与边界修正

1. 将 `Shared` 显示名改为 `Agents Common` / `Agents 公共技能目录`。
2. UI 中把该项标记为 `common source`，不要和真实 agent 混在一起表达。
3. 文案明确：目录存在不代表所有 agent 自动读取。
4. 在设置页或 agent 卡片上显示路径用途：`Agent target`、`Common source`、`Project source`、`External source`。

### P1：AgentConfig 能力模型

1. 增加 `kind` 字段。
2. 增加 `detect_dir`，内置 agent 用 detect_dir 判断安装，skills_dir 只判断可管理目录。
3. 增加 `additional_scan_dirs`，让 `~/.agents/skills` 从“伪 agent”转为 Codex 等 agent 的附加发现来源。
4. 增加 `project_skills_dir`，为 project source 配置化做准备。

### P2：自定义 agent 向导增强

1. 添加时让用户选择类型：真实 Agent / 公共目录 / 外部目录。
2. 对 skills_dir 做校验：是否绝对路径、是否存在、是否包含 `SKILL.md` 子目录。
3. 添加“验证方式”说明：提示用户在目标 agent 中执行一次测试或重启刷新。
4. 支持 path override：内置 agent 的默认目录可以被用户覆盖，而不是只能新增一个 custom agent。

### P3：Project source 配置化

1. 不再长期依赖 `std::env::current_dir()`。
2. 增加 Project Workspace 配置：项目名、root、启用的 project skill patterns。
3. 默认扫描 patterns：`.codex/skills`、`.agents/skills`、`skills`。
4. Project source 默认只读；写入必须经过显式导出/同步预览。

### P4：内容质量与 catalog

1. 扫描时生成本地 catalog cache。
2. 增加质量诊断：frontmatter、description、assets/references/scripts、文件大小、重复 name。
3. 搜索支持 tags/type/scenarios/best_for。
4. 导出支持平台包或 generic folder。

### P5：更大的产品能力

1. Preset：保存一组 skills，一键应用到 agent。
2. Linked Workspace：任意外部 skills root。
3. CLI：服务自动化与 agent 调用。
4. Git backup/merge：等本地模型稳定后再评估。

## 7. 是否需要继续查询其他开源项目

短期不需要。

理由：

1. 当前两个参考项目已经覆盖了两端：一个是成熟内容库，一个是成熟 manager。
2. QS-Vibe 当前最主要的不确定性不是“还有哪些功能可做”，而是概念模型需要收敛。
3. 继续找更多 manager 很可能带来功能清单膨胀，却不能解决 `agent/common/project/source` 的边界问题。

建议仅在以下问题出现时再补充调研：

| 触发条件 | 需要调研的对象 |
| --- | --- |
| 要确认某个 agent 的真实读取路径 | 该 agent 官方文档或源码 |
| 要做 marketplace | skills.sh、Claude plugin marketplace、Codex plugin/skill 规范 |
| 要做 Git 同步/merge | 文件级同步工具、content-addressed storage、skills-manager merge 模块 |
| 要做跨平台 symlink/copy 策略 | Windows junction/symlink 权限相关项目 |

## 8. 最终建议路线

推荐按以下顺序推进：

1. v0.2.x 文案与命名修正：保留旧 id，仅改显示名和说明。
2. v0.3 数据模型升级：引入 source kind / agent kind / detect dir / additional scan dirs。
3. v0.3 Project Workspace：项目根配置化，Project source 默认只读。
4. v0.4 Preset 与 Linked Workspace：解决批量分发与外部目录管理。
5. v0.5 CLI 与 catalog：提高自动化、搜索、质量检查能力。

最小可落地改动：

```text
Shared -> Agents Common
真实 agent -> 可同步目标
Agents Common -> 公共发现/共享目录
Project source -> 项目只读来源
External source -> 外部目录
```

这能在不大改架构的前提下，先把用户认知扶正。等模型升级后，再把 `~/.agents/skills` 从默认 agent 列表中降级为 common source 或 additional scan dir，会更自然。

## 9. 批判性取舍清单

本节从“是否真的适合 QS-Vibe”出发，而不是把参考项目能力全部搬进来。判断标准：

- 是否解决当前用户会立刻遇到的误解或操作风险。
- 是否符合 QS-Vibe 当前“本地 skills 管理 + 多 agent 链接整理”的定位。
- 是否会显著增加数据模型、迁移、网络依赖或维护复杂度。
- 是否能渐进落地，而不是要求一次性重写架构。

### 9.1 这个版本必须加

这里的“必须”指 v0.2.x 或下一个小版本就应该做，否则当前产品语义会继续误导用户。

| 项目 | 来源 | 判断 | 原因 |
| --- | --- | --- | --- |
| `Shared` 改名为 `Agents Common` / `Agents 公共技能目录` | 当前项目问题 + skills-manager 对 common dir 的处理 | 必须加 | 这是低成本、高收益修正；当前 `Shared` 太模糊，会让用户以为它是一个真实 agent 或全局生效目录。 |
| UI 文案区分“目录存在”和“agent 实际识别” | 当前项目问题 | 必须加 | 当前 `detected` 容易被理解成 agent 已可用，但实际上只检查目录是否存在。这个误解会直接造成“为什么 agent 读不到 skill”的问题。 |
| `~/.agents/skills` 不再作为强 agent 语义宣传 | skills-manager | 必须加 | 它可以是公共目录或发现目录，但不应承诺所有 agent 都读取。先改文案和展示语义，不一定马上改数据结构。 |
| 自定义 agent 添加说明与校验提示 | 当前项目问题 | 必须加 | 当前用户能添加自定义 agent，但不知道应该填根目录还是 skills 目录，也不知道可能不生效。至少要在 UI/文档里提示。 |
| Project source 默认只读说明 | 当前项目已有 project source | 必须加 | 项目目录属于用户仓库，误写入风险高。既然已经扫描 project source，就必须明确只读边界。 |

最小实施方式：

1. 保留 `agents-shared` 旧 id，只改显示名和说明。
2. AgentCard 或设置页增加一句：`检测到目录存在不代表目标工具已加载该目录。`
3. AddAgentDialog 增加 `skills_dir_hint`：必须填写 agent 实际读取的 skills 目录。
4. Project source 操作按钮只保留“导入 Library”，隐藏直接删除/覆盖类动作。

### 9.2 近期应该加

这里的“应该”指它们符合项目方向，但需要一点模型改造，适合放进 v0.3。

| 项目 | 来源 | 判断 | 原因 |
| --- | --- | --- | --- |
| `AgentConfig.kind` | skills-manager ToolAdapter + 当前 source_kind | 应该加 | 这是拆清 agent/common/project/external 的核心字段。没有 kind，后续只能靠 id/path 猜语义。 |
| `detect_dir` | skills-manager | 应该加 | 内置 agent 应该通过工具配置目录或安装痕迹检测，不能只看 skills 目录。 |
| `additional_scan_dirs` | skills-manager | 应该加 | 很适合处理 `~/.agents/skills`：对 Codex 等工具可作为附加扫描来源，但不是默认写入目标。 |
| `project_skills_dir` / project patterns | skills-manager Project Workspace | 应该加 | 当前 `project_skill_roots()` 依赖 `current_dir()`，短期能用，长期不稳。 |
| Agent 路径 override | skills-manager 设置模型 | 应该加 | 用户经常会把内置 agent 的路径改到非默认位置。override 比新增一个同名 custom agent 更清楚。 |
| Copy fallback 与 sync safety 文案 | skills-manager sync_engine | 应该加 | Windows symlink 权限是高频问题；项目已有 symlink 要求，但 UI/操作策略还可以更友好。 |

这些内容符合 QS-Vibe，因为它们都服务于“准确管理本地路径和同步关系”，没有引入外部服务或庞大产品线。

### 9.3 加不加都可以

这些能力有价值，但不是当前核心矛盾。加了会锦上添花，不加也不影响 QS-Vibe 成立。

| 项目 | 来源 | 判断 | 原因 |
| --- | --- | --- | --- |
| Machine-readable catalog cache | Product-Manager-Skills | 可选 | 对搜索、筛选、性能有帮助，但当前已有扫描模型；不必为了 catalog 先改架构。 |
| Skill 质量检查 | Product-Manager-Skills | 可选偏应该 | 如果 QS-Vibe 要从“链接管理”升级到“skill library 管理”，它很有价值；但不是解决 agent/shared 语义的前置条件。 |
| 按 type/scenarios/best_for 搜索 | Product-Manager-Skills | 可选 | 对内容库体验好，但依赖 skill frontmatter 丰富度。当前很多 skill 未必有这些字段。 |
| 导出平台包 | Product-Manager-Skills | 可选 | 适合未来做“发布/打包”，但当前用户主要需求还是本地同步到 agent。 |
| Preset | skills-manager | 可选偏应该 | 与多 agent 批量同步天然匹配，但会引入新的状态管理。建议等 source 模型清楚后做。 |
| CLI | skills-manager | 可选 | 对 agent 自动化很好，但当前项目是 Tauri GUI 优先。除非要让其他 agent 调用 QS-Vibe，否则可以后置。 |
| Linked Workspace | skills-manager | 可选偏应该 | 概念非常适合“外部 skills 目录”，但会增加一类 workspace UI。可以在 v0.4 再做。 |

### 9.4 不建议这个阶段加

这些能力不是不好，而是不适合 QS-Vibe 当前阶段。现在加会扩大复杂度，甚至冲淡产品定位。

| 项目 | 来源 | 判断 | 原因 |
| --- | --- | --- | --- |
| Marketplace / AI search | skills-manager | 暂不加 | 需要网络、内容源治理、搜索质量、代理配置、失败处理。当前项目先把本地管理做好更重要。 |
| GitHub 登录与自动备份 | skills-manager | 暂不加 | 会引入账号、token、安全、冲突、恢复流程。QS-Vibe 当前没有数据库，贸然加会过重。 |
| Skill-aware merge | skills-manager | 暂不加 | 这是多设备同步后的问题，不是当前本地 symlink 管理的核心问题。 |
| SQLite 存储 | skills-manager | 暂不加 | JSON 配置目前足够。除非要做 marketplace、复杂历史、多设备同步，否则 SQLite 是额外迁移负担。 |
| 完整复刻 Global/Project/Linked 三套 Workspace UI | skills-manager | 暂不加 | 概念值得借鉴，但一次性复刻会导致产品结构膨胀。QS-Vibe 应先从 source kind 和当前 manage 页渐进演化。 |
| 复杂 command/slash-command 管理 | Product-Manager-Skills | 暂不加 | QS-Vibe 目前定位是 skill 管理，不是 command runtime。除非后续明确支持 commands 标准。 |

### 9.5 符合 QS-Vibe 的部分

符合当前项目定位的能力有一个共同点：它们都帮助用户理解和整理本地 skills 与 agent 目录之间的关系。

| 能力 | 符合原因 |
| --- | --- |
| agent/common/project/external source 分层 | 直接解决当前模型混淆。 |
| 自定义 agent 与 path override | 用户本地环境差异大，必须允许配置。 |
| 附加扫描目录 | 适合处理公共目录、历史目录、插件目录。 |
| Project source 只读纳入 | 已经在项目规划里，且对 coding agent 很常见。 |
| 批量预览、dry-run、冲突/断链修复 | 是多 agent 管理的核心闭环。 |
| Windows symlink/junction/copy 策略 | 与 Tauri 桌面应用和 Windows 用户强相关。 |
| 轻量 catalog/质量诊断 | 能增强 library 管理，但应渐进做。 |

### 9.6 不符合 QS-Vibe 当前阶段的部分

不符合不代表永远不做，而是现在做会让项目从“本地 skill 管理器”过早变成“技能市场 + 备份同步平台”。

| 能力 | 不符合原因 |
| --- | --- |
| Marketplace | 产品边界变大，且需要外部服务与内容审核。 |
| AI search | 需要 API key、代理、模型选择、成本与隐私说明。 |
| GitHub device flow | 安全与账号流程复杂，和当前核心体验距离远。 |
| 多设备自动同步 | 需要冲突模型、快照、恢复、后台任务。 |
| SQLite 全量迁移 | 当前 JSON 足够，迁移收益不足。 |
| 完整 CLI 产品线 | 有长期价值，但短期会分散桌面端交互优化精力。 |

## 10. 最终优先级建议

### 必须做

1. 改 `Shared` 显示名和说明。
2. 明确 `~/.agents/skills` 是公共目录/发现目录，不是确定 agent。
3. 自定义 agent 添加页补“实际读取路径”提示。
4. Project source 保持只读，并在 UI 上明确。

### 应该做

1. 引入 `kind`、`detect_dir`、`additional_scan_dirs`。
2. 将 `~/.agents/skills` 从默认 agent 逐步迁移为 `common source` 或某些 agent 的 `additional_scan_dirs`。
3. 配置化 Project Workspace，不再依赖后端当前工作目录。
4. 支持内置 agent path override。

### 可以后做

1. catalog cache。
2. skill 质量检查。
3. preset。
4. linked workspace。
5. CLI。

### 暂不做

1. marketplace。
2. AI search。
3. GitHub 登录备份。
4. skill-aware merge。
5. SQLite 存储迁移。

一句话判断：QS-Vibe 当前应该优先成为“可靠、清楚、不会误导用户的本地多 agent skill 管理器”，而不是急着成为“全功能 skill 平台”。
