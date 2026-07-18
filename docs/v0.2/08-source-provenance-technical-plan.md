# 来源溯源技术方案

> 范围：对应 backlog 里“技能安装 / 更新 / 来源边界”的第一步。

## 目标

让 QS-Vibe 能区分三种情况：

1. 明确记录过的来源
2. 只能推断的来源
3. 完全未知的来源

## 范围内

- 安装 skill 到 `~/.vibe-skills/` 时写入来源记录。
- 扫描 library、agent、project 来源时读取来源记录。
- 把来源信息挂到 `SkillSource` 上。
- 保留旧数据没有来源记录时的启发式兜底。

## 范围外

- Marketplace 导入和更新路由
- 从 Git / README 深度反查来源
- 自动更新执行
- App 壳重构

## 数据模型

```json
{
  "method": "local-folder",
  "provider": null,
  "url": null,
  "commit": null,
  "installed_at": "2026-07-18T00:00:00Z",
  "installed_by": "qs-vibe",
  "trust_level": "explicit",
  "source_path": "F:/work/repo/skill-folder"
}
```

## 文件位置

- 来源文件：`.vibe-origin.json`
- 位置：每个已安装 skill 目录内部

## 行为

### 安装

`install_skill()` 把目录复制进 library 时：

1. 复制 skill 目录
2. 写入 `.vibe-origin.json`
3. 返回带来源记录的 library source

### 扫描

扫描任意来源目录时：

1. 解析 `SKILL.md`
2. 读取 `.vibe-origin.json`
3. 如果存在就挂上来源记录
4. 如果没有，就回退到启发式判断

### UI

详情页优先展示来源记录。

- 明确记录：显示为已确认
- 启发式元数据：显示为推断
- 什么都没有：显示未知

## 验收标准

- 新安装的 library skill 一定会写 `.vibe-origin.json`。
- 旧 skill 没有来源记录时仍然可以正常加载。
- 详情页可以显示明确来源，但不会假装所有来源都已知。
- 这一步不引入自动更新。
