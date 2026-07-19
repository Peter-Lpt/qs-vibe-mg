# Skill 安装与更新验证记录

## 目标

覆盖当前支持的三种安装方式：本地目录、Git URL、命令（包括 `npx` 和 Claude/Marketplace 命令），并确认安装失败时界面有可读错误、后端日志保留诊断信息。

## 安装方式

| 方式 | 当前入口 | 来源记录 | 更新策略 |
| --- | --- | --- | --- |
| 本地目录 | 安装 Skill → 目录 | `local-folder`；若目录自身是 Git 仓库则记录为 `git` | 本地目录不能自动更新；Git 目录可检查/拉取 |
| Git URL | 安装 Skill → Git URL | 记录远端 URL、分支、commit 和受管源目录 | `git fetch` 检查；更新时要求工作区干净并执行 `git pull --ff-only` |
| 命令 | 安装 Skill → 命令 | 记录完整命令和受管源目录；`npx`、npm、Marketplace 通过命令内容识别 | 仅能回放已记录命令；不会把 npx/npm 当成 `git pull` |

## 失败场景与诊断

1. 路径不存在、Git URL 无法克隆、命令找不到、命令未生成包含 `SKILL.md` 的目录：安装弹窗显示后端错误，失败时清理受管临时源目录。
2. `npx` 失败常见原因包括 Node/npm 不在 PATH、网络/代理不可用、包不存在或版本解析失败、命令需要交互输入、命令输出目录不符合约定。运行环境会设置 `CI=1` 和 npm yes 配置，尽量避免安装确认提示；命令仍有 300 秒超时。错误优先返回 stderr；stderr 为空时返回 stdout 和退出码提示。
3. 每次命令启动、成功输出、启动失败、非零退出或超时都会写入应用日志。Windows 默认日志目录为 `%LOCALAPPDATA%\qs-vibe-mg\logs\app.*.log`。
4. Git 更新检查失败（无远端、远端不可达、权限不足、没有对应远端分支）不会误报“有更新”，详情页会保留失败提示，并在日志中记录原因。

## 更新入口

- 管理页工具栏提供“检查全部 Skill 更新”，执行全量检查；发现 Git 更新后在列表行显示“有可用更新”。
- Skill 详情页的“来源”区域提供单 Skill “检查更新”。Git 来源执行远端检查并显示“有可用更新”。
- 来源具备可回放 `update_command` 时显示刷新按钮；Git、npx/npm、Marketplace 命令分别按其来源命令更新。
- 本地目录且没有可回放命令时明确提示需要重新安装，不自动猜测来源。

实现参考了 `F:\workspace\demo\ref\skills-manager` 的边界：Git 技能做 upstream 检查并在列表标记更新，本地来源走重新导入，命令/Marketplace 来源不强行套用 Git 更新语义；当前 QS-Vibe 已实现前两项中的 Git 检查和列表标记，手动重新安装仍是本地来源的明确路径。

## 手工验证清单

### 本地目录

- [ ] 选择包含有效 `SKILL.md` 的目录，默认复制安装成功。
- [ ] 选择不存在目录，弹窗显示路径错误，日志无静默失败。
- [ ] 选择缺少 `SKILL.md` 的目录，弹窗显示格式错误。

### Git URL

- [ ] 使用可访问仓库 URL 安装，来源显示 Git、URL 和 commit。
- [ ] 在远端新增提交后点击“检查更新”，显示“有可用更新”。
- [ ] 本地源目录有未提交修改时点击更新，操作失败且不覆盖本地修改。
- [ ] 无网络/权限不足时检查失败，不显示有更新，并查看日志中的 fetch 原因。

### npx / Marketplace 命令

- [ ] 使用会在当前目录生成 `SKILL.md` 的命令安装成功，来源显示 npx 或 Marketplace。
- [ ] 使用不存在的命令，弹窗显示命令 stderr/stdout，日志记录命令和退出码。
- [ ] 使用成功但不生成 `SKILL.md` 的命令，弹窗说明找不到技能目录。
- [ ] 点击来源刷新按钮，确认回放原命令而不是执行 Git 更新。

## 自动验证

在仓库根目录执行：

```powershell
pnpm build
$env:RUSTUP_HOME = "D:\environment\rust\.rustup"
$env:CARGO_HOME = "D:\environment\rust\.cargo"
Set-Location src-tauri
cargo test
cargo check
```

本次开发重点验证：IPC 命令注册、TypeScript 类型、三份 locale key 完整性、Git 更新检查编译，以及命令失败输出不会丢失。

## 当前开发验证状态（2026-07-19）

- [x] 本地目录、Git URL、命令三类安装入口均存在并完成前端/后端编译验证。
- [x] `npx`/npm/Marketplace 命令失败时保留 stderr/stdout、退出码和日志记录。
- [x] Git 单 Skill 检查入口与全部 Skill 检查入口已注册到 Tauri IPC。
- [x] 管理页工具栏提供“检查全部 Skill 更新”，发现更新后在列表行显示标记。
- [x] Git 更新检查不会将 npx/npm/Marketplace 命令误判为远端 Git 更新。
- [x] `pnpm build`、`cargo test`（25 passed）、`cargo check`、`git diff --check` 已通过。`n- [x] 新增 `mock_git_update_is_detected` 单测，验证 mock Git 检测和实际更新替换。
- [x] 本机 smoke 验证：本地 `SKILL.md` 可识别；临时 Git 仓库可 `fetch` 并检测本地/远端 commit 不同；`npx` 成功生成测试 `SKILL.md`。
- [x] 本机 npx 失败验证：不存在的包返回退出码 1 和 npm 404 stderr，未出现无限等待。
- [ ] 需要真实外网/用户凭据的 npx 包和 Claude Marketplace 包仍需在目标机器执行手工清单；构建验证不能替代供应商网络行为验证。
