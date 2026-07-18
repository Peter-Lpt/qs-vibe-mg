# 多来源 Skill 安装与更新技术方案

> 来源：`docs/v0.2/07-feature-backlog.md`
>
> 目标：把 `local-folder`、`git`、`npx/npm`、`marketplace` 这几类 Skill 来源统一收敛成可执行的安装 / 更新 / 同步方案，并明确边界场景。

## 原始需求摘要

用户希望解决的不是单一“安装 Skill”入口，而是以下真实场景：

1. 开源 Skill 先在本地仓库维护，然后安装到 Codex / Claude Code。
2. 再同步到 QS-Vibe 的中心库。
3. 后续希望只改一个位置，其它位置都能更新。
4. 中心库可能只是一个软链接，真实文件在 Git 仓库里。
5. 还存在 `npx` 安装、Claude 市场安装等不同来源。

核心问题不是“能不能复制文件”，而是：

- 当前软件能否识别这个 Skill 的真实来源。
- 能否判断它是可自动更新、仅 best-effort 更新，还是只能手动更新。
- 能否在不同来源之间保持一致的同步语义。

## 初步判断

这不是一个单点功能，而是一个来源模型问题。

如果只靠 `symlink` 判断来源，结果一定不稳：

- `symlink` 只能说明“当前指向哪里”，不能说明“来源是什么”。
- `git`、`npx/npm`、`marketplace`、`local-folder` 需要不同的更新策略。
- 默认复制安装更稳，但必须保留 provenance，否则后续无法升级。

因此建议把能力拆成三层：

1. **安装语义**：复制安装 / 引用安装
2. **来源语义**：local-folder / git / npm(npx) / marketplace
3. **更新语义**：pull / reinstall / refresh / manual / unknown

## 任务类型路由

- 多来源安装与更新模型：`incremental_requirement`
- symlink / Git / npx / marketplace 边界识别：`incremental_requirement`
- 默认复制安装与可选引用安装：`refactor_or_optimization`
- 更新策略与失败回退：`bug_fix_plan` + `incremental_requirement`
- 文档整理与边界说明：`docs_only`

## 需求分组

### A. 安装语义

#### 必须加

- 默认复制安装到中心库。
- 提供显式“引用安装”模式，允许中心库直接指向真实 Skill 仓库。
- 安装时写入 `.vibe-origin.json`。
- 安装结果必须区分“实体副本”和“引用副本”。

#### 可延后

- 安装来源自动推断向导。
- 复杂的导入模板。

#### 不建议加

- 默认把所有安装都做成 symlink。
- 没有 provenance 时自动猜测来源并直接升级为自动更新。

### B. 来源类型

#### 必须加

- `local-folder`
- `git`
- `npm` / `npx`
- `marketplace`

#### 设计要点

1. `SkillOrigin.method` 作为主来源类型。
2. `provider` 用于区分 GitHub / Gitee / Claude / 自定义市场 / 包管理器。
3. `url`、`commit`、`version`、`command`、`update_command` 作为补充信息。
4. `trust_level` 只能是明确记录、启发式、未知三类，不允许默认过度乐观。

#### 缺失约束

- 不能把 `symlink` 当作来源类型。
- 不能把 `git`、`npx`、`marketplace` 混成一个“自动更新”分支。
- 不能没有 `origin` 就给出“可自动更新”的结论。

### C. 更新语义

#### 必须加

- `auto_update`：有明确来源和可回放命令。
- `best_effort`：能推断，但不能保证。
- `unknown`：没有足够信息，必须手动处理。

#### 场景建议

1. `git`
   - 可以自动检查远端。
   - 可以拉取更新。
   - 前提是仓库干净、远端可达、权限有效。

2. `npm` / `npx`
   - 可记录包名、版本、安装命令。
   - 更新应走重新安装或升级包。
   - 不应直接等同于 `git pull`。

3. `marketplace`
   - 更新应走重新下载 / 重新安装 / 刷新。
   - 如果市场没有标准接口，只能 best-effort。

4. `local-folder`
   - 默认只能重新选择源目录或手动复制。
   - 如果源目录本身是 `git`，可在确认 provenance 后升级为 `git` 语义。

#### 缺失约束

- 不能把复制安装后的副本直接当成可追踪源。
- 不能在 dirty 仓库上默认执行覆盖式更新。
- 不能把失败原因吞成一个笼统报错。

### D. 中心库与 agent 同步

#### 必须加

- 中心库可以是副本，也可以是引用。
- agent 同步时必须明确“保留引用”还是“落地复制”。
- 不能让同步步骤静默解引用所有 symlink。

#### 推荐策略

1. 默认同步到中心库时采用复制。
2. 用户显式选择“引用安装”时，中心库保留 symlink。
3. 中心库再同步到 agent 时，如果中心库是引用，应优先保留引用链。

#### 缺失约束

- 不能出现“中心库是 symlink，但同步到 agent 时又被复制成副本”的隐式变化。
- 不能让 agent 侧出现不可解释的链路。
- 不能让同一个 skill 在不同层级被同时认为是“源”和“副本”。

### E. 边界场景

#### 1. 本地开源仓库

场景：

- Skill 真实文件在 `D:/repo/my-skill`
- 中心库为 `~/.vibe-skills/my-skill -> D:/repo/my-skill`
- Codex / Claude 指向中心库

结论：

- 可支持。
- 前提是记录 `method = git` 或至少记录真实路径。

#### 2. 本地目录搬家

场景：

- 原仓库从 `D:/repo/my-skill` 移动到 `E:/repo/my-skill`

结论：

- symlink 会断。
- 必须显示断链状态。
- 只能通过重新定位或重新安装恢复。

#### 3. Git dirty

场景：

- 仓库有本地修改
- 用户仍尝试更新

结论：

- 不能默认强拉。
- 必须提示可能冲突，给出 dry-run 或手动确认。

#### 4. Git remote 缺失

场景：

- 仓库存在，但没有 remote origin

结论：

- 只能 best-effort。
- 不应显示为稳定的 auto_update。

#### 5. 私有仓库 / 认证失败

场景：

- 远端需要登录或 token

结论：

- 更新必须失败可见。
- 需要明确提示认证问题，而不是直接归类成“网络错误”。

#### 6. npx 安装

场景：

- Skill 通过 `npx` / `npm` 命令安装

结论：

- 可支持，但更新语义必须是“重新安装 / 升级包”。
- 需要记录包名、版本、命令、可回放参数。

#### 7. Claude 市场安装

场景：

- Skill 来自市场，不是本地 Git 仓库

结论：

- 可支持，但通常只能 best-effort。
- 需要记录市场标识、安装入口、可重装命令。

#### 8. 旧数据没有 provenance

场景：

- 历史 skill 没有 `.vibe-origin.json`

结论：

- 可以继续加载。
- 只能标为 `unknown` 或保守推断。
- 不得自动升级为可自动更新源。

#### 9. 多层链接

场景：

- A 指向 B，B 又指向 C

结论：

- 必须防循环引用。
- 扫描深度要有限制。
- 不能把递归链当成单层来源。

#### 10. Windows symlink / junction

场景：

- 跨盘符、权限不足、开发者模式关闭

结论：

- 必须允许 fallback 到复制。
- 但 fallback 结果要显式告知，不可静默。

### F. 数据模型

#### 必须加

在 `SkillOrigin` 中保留：

- `method`
- `provider`
- `url`
- `commit`
- `installed_at`
- `installed_by`
- `trust_level`
- `source_path`
- `command`
- `update_command`
- `last_checked_at`

#### 建议补充

- `version`
- `package_name`
- `branch`
- `refresh_command`
- `sync_mode`

#### 缺失约束

- 不能只用一个 `method` 字段覆盖全部更新语义。
- 不能把 `source_path` 当作唯一可信来源。

### G. UI 表达

#### 必须加

- 在详情页标出来源类型。
- 标出更新状态：`auto_update / best_effort / unknown`。
- 标出是否引用安装、是否断链、是否可回放更新。

#### 不建议加

- 把“能否更新”只做成一个绿色勾。
- 把推断来源伪装成确定来源。

## 初始优先级

1. 来源类型与 provenance 模型
2. 安装语义拆分：复制 / 引用
3. 更新语义拆分：pull / reinstall / refresh / manual
4. 中心库与 agent 同步策略
5. 边界场景与失败提示
6. UI 表达

## 审计结果

结论：通过，但不能直接按“一个安装功能”理解。

这个问题本质上是一个多来源编排系统，必须先把来源类型和更新策略拆开，否则后面会出现：

- `git` 和 `npx` 混用
- symlink 被误判成来源
- 复制副本没有更新路径
- 市场安装没有回放命令
- dirty 仓库被误强更

## 最终优先级

### 必须加

1. 来源类型建模
2. provenance 记录
3. 复制 / 引用安装分流
4. 更新策略分流
5. 断链 / dirty / 认证失败等边界提示

### 可延后

1. 更强的来源自动推断
2. 市场深度适配
3. 安装向导美化

### 不建议加

1. 没有 provenance 直接自动更新
2. 默认全量 symlink 化
3. 把所有来源强行抽成一条统一命令链

## 下一步

如果进入开发，建议先做：

1. `SkillOrigin` 扩展
2. 安装时 provenance 写入
3. 读取时来源类型识别
4. 更新按钮和更新策略分流
5. 复制 / 引用安装切换
6. 再补 `git`、`npx`、`marketplace` 的专用更新路径

