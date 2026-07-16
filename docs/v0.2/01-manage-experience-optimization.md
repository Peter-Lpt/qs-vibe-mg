# 管理页体验优化方案 v0.2

> 版本：v0.2  
> 日期：2026-07-16  
> 状态：方案设计阶段  
> 范围：仅讨论产品交互和实现路线，不包含代码改动

---

## 1. 背景

当前 QS-Vibe 管理页已经具备两个主要视图：

- 列表视图：以 skill 为中心聚合展示，支持筛选、搜索、预览、展开详情、删除和批量选择。
- 树视图：以 agent/library 为来源分组，支持打开目录、复制路径、link/unlink/sync/relink/delete 等位置相关操作。

从截图和当前实现看，已有功能方向是正确的，但存在一个核心问题：

> 功能被视图绑定了。列表有预览，树有打开目录；列表适合做管理决策，树适合看位置，但两者的功能边界和操作入口还没有形成统一产品心智。

v0.2 的目标不是增加更多视图，而是把 QS-Vibe 从“展示 skill 分布”升级为“帮助用户快速整理多 agent 下的 skill 状态”。

---

## 2. 总体判断

### 2.1 保留列表和树，但重新定义职责

列表和树都应该保留，但职责必须明确：

| 视图 | 新定位 | 主要回答的问题 |
| --- | --- | --- |
| Skill 工作台 | 默认主视图，以 skill 聚合 | 这个 skill 在哪些 agent 里？状态是否健康？我该怎么处理？ |
| 来源/目录视图 | 由当前树视图升级，以位置聚合 | 这个 skill 实际在哪里？哪个目录下有什么？链接目标是什么？ |
| 批量/问题修复 | 替代底部矩阵的主价值 | 哪些问题需要处理？如何一次性处理多个 skill 和多个 agent？ |

### 2.2 矩阵不再作为主流程展示组件

当前底部关系矩阵如果只展示关系，价值偏低，因为它和列表圆点、agent 概览、树视图表达的信息重复。

矩阵只有在升级为“高级批量操作工具”时才值得保留：

- 横轴是 agents/sources。
- 纵轴是 skills。
- 单元格显示 linked、missing、independent、conflict、dangling。
- 支持多选单元格。
- 支持批量 link/unlink/sync/relink/remove_dangling。

如果短期不做这些能力，建议将矩阵降级为“高级诊断”入口，默认隐藏。

---

## 3. 当前主要问题

### 3.1 多 agent、多 skill 的批量闭环不完整

当前已经有多选和 `BatchSyncPanel` 基础，但重度用户需要的是完整闭环：

```text
选择多个 skills -> 选择目标 agents -> 选择策略 -> 预览影响 -> 执行 -> 查看结果/可重试
```

只提供多选按钮不够。用户必须在执行前知道：

- 会创建多少链接。
- 会跳过多少项。
- 哪些会修复断链。
- 哪些会触发冲突。
- 哪些因为未入库而不可操作。
- 哪些 project 文件不会被自动修改。

### 3.2 状态语义偏复杂

当前存在多组状态：

- 已链接
- 未链接
- 需要同步
- 未入库
- 仅库中
- 冲突
- 断链
- 重复
- 独立副本
- linked_elsewhere

这些状态在实现上有必要，但不能全部平铺给用户。UI 需要把它们归纳成更接近任务的语言：

- 正常
- 需要处理
- 缺失覆盖
- 来源不一致
- 链接异常
- 项目专用

底层仍保留精确状态，展示层按任务归类。

### 3.3 颜色点信息不够自解释

列表中的 agent 状态点对熟悉用户有效，但对新用户不够直观。

需要补充：

- hover tooltip：显示 agent 名称、状态、路径。
- agent 缩写或顺序图例。
- 异常状态使用明确 icon，而不是只靠颜色。

### 3.4 Project skill 尚未纳入模型

当前来源主要是用户级目录：

- `~/.vibe-skills`
- `~/.claude/skills`
- `~/.codex/skills`
- `~/.agents/skills`
- 其他 agent skills 目录

但实际使用中，项目内也可能包含 skill，例如：

- `.codex/skills`
- `.agents/skills`
- `skills`

这些 project skill 既不是全局 library，也不是 agent 用户目录。它们应该作为独立 source 类型显示，但默认只读，避免污染 git repo。

---

## 4. v0.2 产品结构

### 4.1 Skill 工作台

Skill 工作台是默认入口，继续使用当前列表视图的核心思路。

建议每行默认结构：

```text
[checkbox] skill-name    状态标签    覆盖 5/6    agent 状态    推荐操作    更多
```

默认只外露高频信息：

- skill 名称
- 覆盖情况，例如 `5/6`
- 异常标签，例如 `冲突`、`断链`、`未入库`
- agent 状态点或缩写
- 一个推荐主操作
- 更多菜单

展开或打开详情后再显示：

- 每个 agent 的具体状态
- 每个 source 的路径
- SKILL.md 预览
- 冲突内容对比
- 操作历史
- 来源信息

### 4.2 来源/目录视图

当前树视图不建议继续称为“树形式”。更准确的名称是：

- 来源
- 目录
- 位置

推荐结构：

```text
Library
  ~/.vibe-skills

Agents
  Claude Code
    ~/.claude/skills
  Codex CLI
    ~/.codex/skills
  OpenCode
    ~/.config/opencode/skills

Projects
  qs-vibe-mg
    .codex/skills
    .agents/skills
    skills
```

第一阶段可以保持当前“按来源平铺 skill”的实现，不急着做真实文件系统层级。真实目录树会引入更复杂的节点语义：

- 一个节点可能是目录，不是 skill。
- 一个 skill 可能出现在多个位置。
- symlink 节点的真实内容在别处。
- 点击节点时必须明确操作的是哪个 source。

因此 v0.2 推荐先做来源树，后续再扩展真实目录层级。

### 4.3 问题修复入口

建议新增或强化“问题修复”入口，用来替代底部矩阵的默认价值。

结构示例：

```text
需要处理
- 冲突 3
- 断链 2
- 仅存在于 agent 18
- 仅存在于 project 4
- Library 中未被任何 agent 使用 20
- 未覆盖目标 agent 105
```

每个问题组点击后进入可批量处理列表。

问题修复视图的价值比关系矩阵更直接：用户进入这里不是为了看数据，而是为了解决问题。

---

## 5. 统一动作模型

### 5.1 原则

所有视图都应该调用同一套 skill/source action。

不同视图只决定：

- 哪些动作直接外露。
- 哪些动作放进更多菜单。
- 哪些动作需要确认。
- 哪些动作在当前 source 上不可用。

### 5.2 通用动作集合

| 动作 | 说明 | 适用视图 |
| --- | --- | --- |
| 预览 SKILL.md | 查看 skill 文档内容 | 列表、来源/目录、详情 |
| 打开目录 | 用文件管理器打开 source 路径 | 列表、来源/目录、详情 |
| 复制路径 | 复制 source 路径或 symlink target | 列表、来源/目录、详情 |
| 查看来源 | 展示所有 source、hash、路径、链接目标 | 列表、详情 |
| 导入 Library | 将 agent/project 独立 skill 纳入 `~/.vibe-skills` | 来源/目录、详情、批量 |
| 链接到 agent | 从 Library 创建到 agent 的 symlink/junction | 列表、来源/目录、批量 |
| 取消链接 | 移除 agent 下的 symlink/junction | 列表、来源/目录、批量 |
| 重新链接 | 将 linked_elsewhere 改回指向 Library | 列表、来源/目录、批量 |
| 修复断链 | 移除 dangling link 或重新选择目标 | 问题修复、详情 |
| 删除 | 删除 Library skill 或 agent link/source | 详情、更多菜单 |

### 5.3 删除动作必须拆分语义

“删除”不能是一个模糊动作。必须区分：

- 删除 agent 链接。
- 删除 Library 源文件。
- 删除 project 文件。
- 删除断链入口。

其中 project 文件默认不允许直接删除；如未来支持，必须强确认并提示可能产生 git diff。

---

## 6. 批量操作闭环

### 6.1 核心流程

批量操作应该成为 v0.2 的核心能力：

```text
选择 skills -> 选择 agents -> 选择策略 -> 预览 -> 执行 -> 结果明细
```

### 6.2 策略

至少支持以下策略：

| 策略 | 行为 |
| --- | --- |
| 缺失则链接 | Library 中存在且目标 agent 缺失时，创建 symlink/junction |
| 已存在则跳过 | 目标 agent 已有真实目录或链接时，不做破坏性操作 |
| 断链则修复 | 目标为 dangling link 时，允许移除或重新链接 |
| 独立副本则提示 | agent 有真实目录时，提示导入、替换为链接或保持独立 |
| Project 默认只读 | project source 不自动覆盖、不自动删除 |

### 6.3 Dry-run 预览

执行前必须展示影响范围：

```text
将执行 42 项操作
- 创建链接 30
- 修复断链 2
- 替换为链接 5
- 需要先入库 3
- 冲突 2
```

每一项都应能定位到：

```text
skill x agent x action x reason
```

### 6.4 执行结果

执行后不要只给 toast。需要保留结果明细：

- 成功项。
- 失败项。
- 跳过项。
- 冲突项。
- 不可操作项。

失败项保留选择，方便用户调整后重试。

---

## 7. Project source 融合方案

### 7.1 定位

Project source 是独立来源，不是 agent，也不是 Library。

推荐 source 类型：

```ts
type SourceKind = "library" | "agent" | "project" | "external";
```

v0.2 最小实现可以先不完全重构数据模型，但产品语义应按此设计。

### 7.2 扫描范围

第一阶段只扫描明确目录：

- `.codex/skills`
- `.agents/skills`
- `skills`

不建议第一阶段全项目递归扫描。

如果后续做递归扫描，必须跳过：

- `.git`
- `node_modules`
- `target`
- `dist`
- `build`
- `.venv`
- `vendor`

并限制最大深度。

### 7.3 默认只读

Project source 默认只读：

- 可以预览。
- 可以打开目录。
- 可以复制路径。
- 可以导入 Library。
- 可以标记为项目专用。

默认不允许：

- 自动覆盖 project 文件。
- 自动删除 project 文件。
- 自动把 Library 写回 project。

---

## 8. 拖拽交互边界

### 8.1 可以做，但不是第一优先级

拖拽适合少量、直觉型操作；不适合几十上百个 skill 的高效整理。

多 agent、多 skill 的核心仍然应该是筛选、多选、批量预览和批量执行。

### 8.2 安全语义

推荐拖拽规则：

| 拖拽 | 行为 |
| --- | --- |
| Library -> Agent | 创建链接 |
| Project -> Library | 导入 Library |
| Agent 独立 skill -> Library | 同步/导入 Library |
| Agent -> Agent | 弹出选择，不直接执行 |

### 8.3 必须有释放后预览

拖拽释放后不应直接执行高风险操作。应先显示：

```text
将 frontend-design 链接到 Codex CLI
来源：~/.vibe-skills/frontend-design
目标：~/.codex/skills/frontend-design
操作：创建软链接
```

目标已存在时：

- 如果已是正确链接：提示无需操作。
- 如果是断链：提示修复。
- 如果是真实目录：提示冲突，不覆盖。
- 如果是指向别处的链接：提示重新链接。

---

## 9. 来源识别方案

### 9.1 能判断什么

可以较可靠判断：

- skill 目录保留 `.git/config`，可读取 remote。
- `SKILL.md` metadata 写了 repository/source/homepage。
- 安装或导入时记录过来源。

只能推断：

- README 中出现 GitHub/Gitee/GitLab URL。
- 路径中包含特定平台或工具痕迹。

不能可靠判断：

- zip 下载后复制进来。
- npx/skill.sh 拉取后只留下普通目录。
- 用户手动复制、改名或删除 metadata。
- 多次同步后来源链断裂。

### 9.2 UI 显示置信度

来源显示必须有置信度：

| 显示 | 含义 |
| --- | --- |
| GitHub | 有明确记录，例如 `.vibe-origin.json` 或 `.git/config` |
| 可能来自 GitHub | metadata/README/path 推断 |
| 未知来源 | 无可靠信息 |

不要把推断结果当成确定事实。

### 9.3 `.vibe-origin.json`

建议未来在安装、导入、同步时写入来源记录：

```json
{
  "method": "git",
  "provider": "github",
  "url": "https://github.com/example/skills",
  "commit": "abc123",
  "installed_at": "2026-07-16T00:00:00Z",
  "installed_by": "qs-vibe"
}
```

历史数据无法可靠补齐，只能做推断。

---

## 10. 实现可行性分级

### 10.1 低成本高收益

- 统一 Skill actions。
- 列表补打开目录、复制路径、更多菜单。
- 树补预览入口。
- agent 状态点补 tooltip/图例。
- 矩阵默认隐藏或移入高级诊断。

### 10.2 中等成本

- 批量操作 dry-run 预览。
- 将 `BatchSyncPanel` 正式作为批量操作中心。
- 来源识别第一阶段：`.git/config`、metadata、README 推断。
- `.vibe-origin.json` 记录新安装/导入来源。
- 来源/目录视图改成 Library/Agents/Projects 三类根。

### 10.3 高成本/高风险

- Project source 完整融合到统一 source 模型。
- 真实文件系统目录树完全替代当前 agent 平铺树。
- 拖拽移动真实目录或跨 project 覆盖。
- 无历史记录情况下强行判断 GitHub/Gitee/GitLab/skill.sh 来源。
- 一键覆盖冲突版本。

---

## 11. 分阶段路线

### Phase 1：统一动作和信息表达

目标：低风险提升一致性。

- 抽象统一 `SkillAction` 配置。
- 列表和树共用预览、打开目录、复制路径、删除、link/unlink/sync/relink。
- 列表 agent 状态点增加 tooltip 和图例。
- 删除动作明确区分删除链接、删除 Library、删除 project。
- 矩阵继续折叠，弱化为高级诊断入口。

### Phase 2：强化批量管理

目标：解决多 agent、多 skill 的核心痛点。

- 将 `BatchSyncPanel` 定位为正式批量操作中心。
- 支持选择 skills、选择 agents、选择策略。
- 增加 dry-run 预览。
- 增加成功/失败/跳过/冲突明细。
- 失败项保留选择，支持调整后重试。

### Phase 3：问题修复入口

目标：让用户从“看列表”变成“处理问题”。

- 新增问题修复入口。
- 按冲突、断链、未入库、仅 project、仅 agent、未覆盖目标 agent 分组。
- 每组支持批量处理。
- 将底部矩阵从主页面移出或降级。

### Phase 4：Project source 最小版

目标：把项目 skill 纳入视野，但不引入破坏性操作。

- 扫描 `.codex/skills`、`.agents/skills`、`skills`。
- Project 作为独立 source 显示。
- 默认只读。
- 支持预览、打开目录、复制路径、导入 Library。
- 不支持自动覆盖 project 文件。

### Phase 5：来源识别

目标：逐步建立来源追踪能力。

- 只读推断 `.git/config`、metadata、README。
- UI 显示确定/推断/未知。
- 新安装和导入写 `.vibe-origin.json`。
- 不对历史数据做虚假确定。

### Phase 6：来源/目录树升级

目标：让树视图成为可靠的位置视图。

- 树视图改名为来源/目录。
- 根节点改为 Library、Agents、Projects。
- 第一版保持来源下平铺 skill。
- 第二版再引入真实目录层级。
- 拖拽只做安全语义，不移动真实目录。

---

## 12. 验收标准

- [ ] 列表和树调用同一套 skill action，不再出现“列表能预览但树不能”“树能打开目录但列表不能”的割裂。
- [ ] 列表 agent 状态点 hover 后能明确显示 agent、状态和路径。
- [ ] 批量操作执行前有 dry-run 预览。
- [ ] 批量结果能显示成功、失败、跳过、冲突和不可操作明细。
- [ ] 问题修复入口能按冲突、断链、未入库、未覆盖等任务聚合。
- [ ] Project source 默认只读，不会自动修改项目文件。
- [ ] 矩阵不再占据主流程空间；若保留，必须作为高级批量工具。
- [ ] 来源识别结果显示置信度，不把推断当确定。
- [ ] 拖拽释放后必须显示操作预览，不直接覆盖或删除。

---

## 13. 最终结论

v0.2 的核心方向是：

> 列表负责管理决策，来源/目录负责位置排查，批量/问题修复负责高效处理。

当前 QS-Vibe 已经有足够基础能力，不需要推倒重来。最值得优先投入的是：

1. 统一动作模型。
2. 批量操作 dry-run。
3. 问题修复入口。
4. Project source 只读纳入。

这条路线能最直接解决多 agent、多 skill 管理的真实痛点，也能为后续真实目录树、拖拽增强和来源追踪打下清晰边界。

