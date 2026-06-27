# VAB Skills Manager v0.1 - 开发需求清单

> 从实施计划中提取的当前版本待开发需求，按优先级排列。
> 与 docs/v0.1/01-plan.md 保持同步。

---

## P0 - 核心功能（必须完成）

### R01 Skill 统一库管理
- [x] ~/.vab-skills/ 目录作为 skill 实体文件存储位置
- [x] 每个 skill 一个子文件夹，包含 SKILL.md 及相关资源（scripts/、references/、assets/）
- [x] .vab-config.json 存储 agent 配置和全局设置
- [x] .vab-history.json 存储操作历史（最近 50 条）

### R02 SKILL.md 解析
- [x] 解析 SKILL.md 的 YAML frontmatter（name、description、license、compatibility、metadata、allowed-tools）
- [x] frontmatter 缺失或格式错误时优雅降级（用文件夹名作为 name，空 description）
- [x] Rust 端用 serde_yaml 解析，通过 Tauri command 传给前端
- [ ] name 字段校验：1-64字符、小写字母+数字+连字符、不能以连字符开头/结尾、不能有连续连字符

### R03 Agent 自动检测
- [x] 启动时扫描以下目录判断 agent 是否已安装：
  - claude-code → ~/.claude/
  - hermes → ~/.hermes/
  - pi-agent → ~/.pi/
  - opencode → ~/.config/opencode/
  - codex → ~/.codex/
  - mimocode → ~/.config/mimocode/
  - agents-shared → ~/.agents/
- [ ] 检测结果更新到 .vab-config.json 的 detected 字段
- [x] 未检测到的 agent 在 UI 中灰显

### R04 链接创建与管理（Symlink + Junction）
- [x] 从 ~/.vab-skills/{skill} 创建链接到 agent 的 skills 目录
- [ ] 三种模式：Symlink（Unix/Win）、Junction（Win 免管理员）、Copy
- [ ] 启动时自动检测系统支持的模式（Symlink → Junction → Copy 降级）
- [x] agent 的 skills 目录不存在时自动创建
- [x] 链接创建前检测是否已存在（避免重复）
- [ ] 链接有效性检测：存在且目标有效 → 有效，目标不存在 → 断裂

### R05 复制模式
- [ ] 除 symlink/junction 外支持直接复制文件夹到 agent 目录
- [ ] skill 卡片顶部切换按钮：symlink / junction / copy
- [ ] 切换只影响后续新建的关联，已存在的不动

### R06 Dashboard 界面
- [x] 左右分栏布局：左侧 Skills 列表，右侧 Agent 面板
- [x] 顶部搜索栏：搜索 skill 名称/描述
- [x] Skill 卡片展示：名称、描述、路径、关联的 agent 列表、同步模式、是否包含 scripts/references/assets
- [x] Agent 卡片展示：名称、检测状态、已关联的 skill 列表
- [x] 已关联 skill 在 agent 卡片上显示标记 + 删除按钮
- [ ] 断裂链接在 UI 中标红，提供 [重新关联] [删除失效关联] 操作

### R07 Skill 安装
- [x] 点击 [+ 安装 Skill] 按钮
- [x] 弹窗输入本地文件夹路径
- [x] 将文件夹拷贝到 ~/.vab-skills/ 下
- [x] 自动解析 SKILL.md frontmatter
- [x] 安装记录写入操作历史

### R08 Skill 删除
- [x] 删除 ~/.vab-skills/ 下的实体文件夹
- [x] 删除前自动清理所有已创建的链接/副本
- [x] 确认弹窗防止误删
- [ ] 删除前备份到 .vab-history/snapshots/{id}/（支持撤销恢复）
- [x] 删除记录写入操作历史

---

## P1 - 交互功能（重要）

### R09 拖拽关联
- [ ] 从左侧拖拽 skill 到右侧 agent 卡片 → 创建关联
- [ ] 拖拽反馈：agent 卡片高亮可放置区域
- [ ] 创建关联后实时更新两侧状态
- [ ] 支持从 AgentCard 拖拽 SkillChip 到另一个 AgentCard → 复制关联
- [ ] 支持从 AgentCard 拖拽 SkillChip 到 trash 区域 → 删除关联

### R10 删除关联
- [x] Agent 卡片上 [×] 按钮 → 删除单个链接/副本
- [ ] 确认弹窗
- [x] 删除记录写入操作历史

### R11 批量操作
- [x] 点击 [批量删除关联] 按钮 → 进入多选模式
- [x] 勾选要解除的关联项
- [ ] 确认后批量删除
- [x] 支持选中多个 skill → 批量关联到 agent
- [x] 操作记录写入历史

### R12 操作历史与撤销/重做
- [x] 底部历史栏展示最近操作（时间 + 描述）
- [x] [↩ 撤销] 按钮：逆向执行最近一条未撤销操作
- [x] [↪ 重做] 按钮：正向执行最近一条已撤销操作
- [x] 最多保留 50 条历史
- [x] 撤销映射：
  - link → 删除链接/副本
  - unlink → 重新创建链接/副本
  - install → 删除 skill 文件夹
  - delete → 从 snapshot 恢复文件夹（暂不支持）
  - batch_link → 批量删除链接
  - batch_unlink → 批量重新创建链接

### R13 SKILL.md 预览
- [x] 点击 skill 卡片 → 弹窗显示 SKILL.md 完整内容
- [x] Markdown 渲染展示

---

## P2 - 设置与国际化（完善体验）

### R14 主题切换
- [x] 支持亮色 / 暗色 / 跟随系统
- [x] Tailwind dark mode + CSS 变量
- [x] 用户选择持久化到 localStorage

### R15 语言切换
- [x] 支持简体中文 / English / 繁體中文
- [x] vue-i18n + 三语言 JSON 文件
- [x] 语言选择持久化到 localStorage

### R16 设置页
- [x] 自定义 Agent：添加/删除（name + skills_dir）
- [x] 主题设置
- [x] 语言设置
- [ ] 全局默认同步模式设置
- [ ] 最大历史记录数设置
- [ ] 快照大小限制设置

### R17 Windows 权限与降级
- [ ] 首次启动检测 symlink 权限
- [ ] 检测链：Symlink → Junction → CopyOnly
- [ ] 无 symlink 权限时尝试 Junction（免管理员）
- [ ] Junction 也不可用时降级为 Copy 模式
- [ ] 弹窗提示当前模式和可选操作（以管理员重启 / 使用 Junction / 使用复制）

---

## P3 - 健壮性

### R18 错误处理
- [x] 后端统一错误类型（thiserror）：Io、SkillNotFound、AgentNotFound、InvalidSkillMd、PermissionDenied、SymlinkFailed、ConfigCorrupted
- [ ] 前端结构化错误展示（ErrorBanner 组件）
- [ ] 错误提示含可操作按钮（如"以管理员身份重启"、"切换为 Junction"）

### R19 并发安全
- [ ] 全局操作锁（Mutex），防止并发写入冲突
- [ ] 多实例检测：.vab-lock 文件 + 进程检查
- [ ] 多实例时提示用户关闭其他实例

### R20 日志
- [ ] Rust 后端使用 tracing 输出日志
- [ ] 日志文件位置：Windows %LOCALAPPDATA%/qs-vab-mg/logs/，Mac ~/Library/Logs/qs-vab-mg/
- [x] 开发模式：Vite devtools + Vue DevTools
- [x] 生产模式：Tauri 调试窗口（Ctrl+Shift+I）

---

## P4 - 打包发布

### R21 跨平台打包
- [ ] Mac 打包为 .dmg
- [x] Win 打包为 .msi / .exe
- [ ] Mac 未签名处理（xattr -cr 提示）

---

## 需求依赖关系

```
R01 (统一库) ──→ R02 (解析) ──→ R06 (Dashboard)
    │                                ↑
    ├──→ R03 (Agent检测) ────────────┘
    │
    ├──→ R04 (链接) ──→ R09 (拖拽) ──→ R11 (批量)
    │         │
    ├──→ R05 (复制)       R10 (删除关联)
    │
    ├──→ R07 (安装) ──→ R08 (删除)
    │
    ├──→ R12 (撤销) ←── 所有写操作
    │
    └──→ R18 (错误处理) ←── 所有操作

R06 (Dashboard) ──→ R13 (预览)
R14 (主题) + R15 (语言) ──→ R16 (设置页)
R17 (权限) ──→ R04 (链接)
R19 (并发) ←── 多写操作
```

## 开发顺序建议

```
第一批：R01 → R02 → R03 → R04 → R05 → R06（后端 + 基础 UI）
第二批：R07 → R08 → R13（安装删除 + 预览）
第三批：R09 → R10 → R11 → R12（交互 + 撤销）
第四批：R14 → R15 → R16 → R17（设置 + 国际化 + 权限）
第五批：R18 → R19 → R20（错误处理 + 并发 + 日志）
第六批：R21（打包）
```
