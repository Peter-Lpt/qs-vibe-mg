# VAB Skills Manager v0.1 - 开发需求清单

> 从计划中提取的当前版本待开发需求，按优先级排列。

---

## P0 - 核心功能（必须完成）

### R01 Skill 统一库管理
- [ ] ~/.vab-skills/ 目录作为 skill 实体文件存储位置
- [ ] 每个 skill 一个子文件夹，包含 SKILL.md 及相关资源
- [ ] .vab-config.json 存储 agent 配置和全局设置
- [ ] .vab-history.json 存储操作历史（最近 50 条）

### R02 SKILL.md 解析
- [ ] 解析 SKILL.md 的 YAML frontmatter（name、description、license、compatibility、metadata）
- [ ] frontmatter 缺失或格式错误时优雅降级（用文件夹名作为 name，空 description）
- [ ] Rust 端用 serde_yaml 解析，通过 Tauri command 传给前端

### R03 Agent 自动检测
- [ ] 启动时扫描以下目录判断 agent 是否已安装：
  - claude-code → ~/.claude/
  - hermes → ~/.hermes/
  - pi-agent → ~/.pi/
  - opencode → ~/.config/opencode/
  - codex → ~/.codex/
  - mimocode → ~/.config/mimocode/
  - agents-shared → ~/.agents/
- [ ] 检测结果更新到 .vab-config.json 的 detected 字段
- [ ] 未检测到的 agent 在 UI 中灰显

### R04 Symlink 创建与管理
- [ ] 从 ~/.vab-skills/{skill} 创建 symlink 到 agent 的 skills 目录
- [ ] 跨平台支持：Unix 用 symlink，Windows 用 symlink_dir
- [ ] agent 的 skills 目录不存在时自动创建
- [ ] symlink 创建前检测是否已存在（避免重复）
- [ ] symlink 有效性检测：存在且目标有效 → 有效，目标不存在 → 断裂

### R05 复制模式
- [ ] 除 symlink 外支持直接复制文件夹到 agent 目录
- [ ] skill 卡片顶部切换按钮：symlink / copy
- [ ] 切换只影响后续新建关联，已存在的不动

### R06 Dashboard 界面
- [ ] 左右分栏布局：左侧 Skills 列表，右侧 Agent 面板
- [ ] Skill 卡片展示：名称、描述、路径、关联的 agent 列表、同步模式
- [ ] Agent 卡片展示：名称、检测状态、已关联的 skill 列表
- [ ] 已关联 skill 在 agent 卡片上显示 [symlink/copy] 标记 + [×] 删除按钮

### R07 Skill 安装
- [ ] 点击 [+ 安装 Skill] 按钮
- [ ] 弹窗选择本地文件夹路径
- [ ] 将文件夹拷贝到 ~/.vab-skills/ 下
- [ ] 自动解析 SKILL.md frontmatter
- [ ] 安装记录写入操作历史

### R08 Skill 删除
- [ ] 删除 ~/.vab-skills/ 下的实体文件夹
- [ ] 删除前自动清理所有已创建的 symlink/副本
- [ ] 确认弹窗防止误删
- [ ] 删除记录写入操作历史

---

## P1 - 交互功能（重要）

### R09 拖拽关联
- [ ] 从左侧拖拽 skill 到右侧 agent 卡片 → 创建关联（symlink 或 copy）
- [ ] 拖拽反馈：agent 卡片高亮可放置区域
- [ ] 创建关联后实时更新两侧状态

### R10 删除关联
- [ ] Agent 卡片上 [×] 按钮 → 删除单个 symlink/副本
- [ ] 确认弹窗
- [ ] 删除记录写入操作历史

### R11 批量删除关联
- [ ] 点击 [批量删除关联] 按钮 → 进入多选模式
- [ ] 勾选要解除的关联项
- [ ] 确认后批量删除
- [ ] 操作记录写入历史

### R12 操作历史与撤销/重做
- [ ] 底部历史栏展示最近操作（时间 + 描述）
- [ ] [↩ 撤销] 按钮：逆向执行最近一条未撤销操作
- [ ] [↪ 重做] 按钮：正向执行最近一条已撤销操作
- [ ] 最多保留 50 条历史
- [ ] 撤销映射：
  - link → 删除 symlink/副本
  - unlink → 重新创建 symlink/副本
  - install → 删除 skill 文件夹
  - delete → 从 snapshot 恢复文件夹

### R13 SKILL.md 预览
- [ ] 点击 skill 卡片 → 弹窗显示 SKILL.md 完整内容
- [ ] Markdown 渲染展示

---

## P2 - 设置与国际化（完善体验）

### R14 主题切换
- [ ] 支持亮色 / 暗色 / 跟随系统
- [ ] Tailwind dark mode + CSS 变量
- [ ] 用户选择持久化到 .vab-config.json

### R15 语言切换
- [ ] 支持简体中文 / English / 繁體中文
- [ ] vue-i18n + 三语言 JSON 文件
- [ ] 语言选择持久化到 .vab-config.json

### R16 设置页
- [ ] 自定义 Agent：添加/编辑/删除（name + skills_dir）
- [ ] 主题设置
- [ ] 语言设置
- [ ] 全局默认同步模式设置

### R17 Windows Symlink 权限处理
- [ ] 首次启动检测 symlink 权限
- [ ] 无权限时弹窗提示开启开发者模式
- [ ] 提供开启开发者模式的步骤说明

---

## P3 - 打包发布

### R18 跨平台打包
- [ ] Mac 打包为 .dmg
- [ ] Win 打包为 .msi / .exe
- [ ] Mac 未签名处理（xattr -cr 提示）

---

## 需求依赖关系

```
R01 (统一库) ──→ R02 (解析) ──→ R06 (Dashboard)
    │                                ↑
    ├──→ R03 (Agent检测) ────────────┘
    │
    ├──→ R04 (Symlink) ──→ R09 (拖拽) ──→ R11 (批量)
    │         │
    ├──→ R05 (复制)       R10 (删除关联)
    │
    ├──→ R07 (安装) ──→ R08 (删除)
    │
    └──→ R12 (撤销) ←── 所有写操作

R06 (Dashboard) ──→ R13 (预览)
R14 (主题) + R15 (语言) ──→ R16 (设置页)
R17 (权限) ──→ R04 (Symlink)
```

## 开发顺序建议

```
第一批：R01 → R02 → R03 → R04 → R05 → R06（后端 + 基础 UI）
第二批：R07 → R08 → R13（安装删除 + 预览）
第三批：R09 → R10 → R11 → R12（交互 + 撤销）
第四批：R14 → R15 → R16 → R17（设置 + 国际化 + 权限）
第五批：R18（打包）
```
