# 10. 性能优化方案（Performance Optimization）

> 目标版本：v0.2 性能专项 ｜ 评审：2026-07-13 ｜ 范围：`src-tauri/` + `src/`
> 经独立子 agent 审计（边界 / 跨平台 / 正确性），关键修正已并入各节开头。

## 一、热点定位（为什么慢）

- **全量递归 SHA-256**：`utils/hash.rs` 的 `dir_hash` 对每个 skill 的每个 source 都调用（`skills.rs:669`）。`search_skills`/`detect_issues` 内部再调 `list_skills`，哈希被重复触发；`get_dashboard_data` 又独立全树扫描 → 列表/搜索/看板延迟随 skill 数线性放大。
- **重复目录扫描**：`find_linked_agents`（`skills.rs:708`）与 `scan_linked_skills`（`config.rs:239`）功能重合却各扫一遍。
- **前端每次变更整体重拉**：`stores/skills.ts` 中 link/unlink/sync/relink/batch/install/delete 全部 `refreshSkills()`（整体 `list_skills`）；后端 `search_skills` 本身只是 `list_skills` + 内存过滤。
- **递归扫描无上限/无环路保护**：`agents.rs:146`、`skills.rs:304/353/441` 均无深度上限与已访问集合 → 链接环可栈溢出。
- **配置每次重读**：`load_config` + `build_agents_from_config` 在每次命令中重解析 JSON 并 `exists()` 探测各 agent 目录。

## 二、优化方案（P1–P6）

> 审计已并入的修正用「【修正】」标出，未改动原意的保留。

### P1 内容哈希惰性化 + 哈希缓存
- 做法：新增 ` ~/.vibe-skills/.vibe-hash-cache.json `，键为 `路径→(mtime,size,file_count,sha256)`；三元组全相等则直接复用 `sha256`，不读文件。写入用临时文件 + 原子 `rename`（避免并发损坏）。
- 【修正】`content_hash` 被前端 `useSkillAgentStatus.ts:117` 用于冲突判定、且参与 `has_conflict` 推导（真哈希 **必须保留**。mtime/size/count 仅作缓存失效键，不在外用；FAT/exFAT 上 mtime 仅 2s 精度，最坏只是「多算一次哈希」，不漏报冲突。

### P2 合并重复扫描 + 统一链接检测
- 做法：`detect_issues` 已复用 `list_skills`（无需再扫）；`get_dashboard_data` 与 `list_skills` 共享一次「agent→skill 映射」，dashboard 复用该映射得 `shared_skills`，不再单独全树 `collect_skills_recursive`。
- 【修正】统一 `find_linked_agents` 与 `scan_linked_skills` 为单函数，**两侧都走 `normalize_path`**（`fs.rs:7`）：Windows 下 `scan_linked_skills` 用 `read_link_target` 解析 junction、`strip_prefix(vibe_dir)` 时若 vibe_dir 未归一化会失败导致 `linked_skills` 为空，统一前必现回归。

### P3 前端增量更新 + 本地搜索
- 做法（方案 A，推荐）：把 `create_link`/`remove_link`/`sync_to_vibe`/`relink`/`batch_skill_action` 改为返回更新后的 `Skill` 或 `Option<Skill>`，前端原地替换对应项而非 `refreshSkills()`。`install_skill` 本就返回 `Skill`，直接替换。本地搜索：前端用已加载 `skills` 过滤，去掉 `invoke("search_skills")`。
- 【修正】原「仅凭返回值 patch」前提不成立：这些命令现返回 `()` 或 `{synced_count,errors}`，受影响 skill 的 `linked_agents`/`has_conflict`/`has_dangling` 客户端无法重建 → **需先改命令签名**。快速连续操作须保证返回为「落盘后最新」值，避免旧 patch 覆盖新状态（必要时批量动作后仍兜底一次 refresh）。

### P4 递归扫描加深度 / 环路保护
- 做法：所有递归入口加 `MAX_SCAN_DEPTH`（如 12）与 `visited: HashSet<PathBuf>`（存 `normalize_path` 后的规范化路径）；超限/遇环返回 `truncated=true` 给前端提示。对单 agent 超阈值（如 5000）的目录截断并提示。

### P5 配置与 agent 列表缓存
- 做法：进程内 `OnceLock<Mutex<Cached>>` 缓存配置 + agent 元数据；失效触发点为 `add/update/remove agent`、`set_vibe_skills_path` **以及所有 `sync.rs` 链接变更命令**（它们改变 `linked_skills`）。
- 【修正】失效范围**必须包含 sync 类命令**，否则缓存返回陈旧链接状态。`save_config`（`config.rs:193`）改「临时文件 + `rename`」原子写；`load_config` 解析失败回退默认（而非 `Err` 中断，当前默认配置仅文件「不存在」时生成，损坏即崩）。`save_history` 同改。

### P6 死代码清理 → 任意路径 I/O 沙箱
- 【修正】原「删除 `write_file_to_path`/`read_file_from_path`」**不成立**：二者是活的，被 `app.ts:83,87` 用于导出/导入到文件功能。真正问题是**无沙箱的任意路径读写**——`config.rs:131-138` 与 `preview_skill_at_path`（`skills.rs:426`）仅 `exists()`/直读，存在路径穿越 / 任意文件读取面。
- 做法：保留命令，但写入目标须 `normalize_path` 后 `starts_with(allowed_root)`（导出目录或 vibe 目录）校验；读取限于 vibe 目录子树，拒绝 `..` 逃逸与链接跳出。可重命名为 `export_write_file`/`import_read_file` 明确边界。

## 三、跨平台专项

| 关注点 | Windows | macOS / Linux | 处理 |
|--------|---------|---------------|------|
| 链接类型 | 无开发者模式/管理员时用 **junction**（`fs.rs:73`），否则 `symlink_dir` | POSIX symlink | 统一走 `normalize_path` 去除 `\\?\` |
| 目标读取 | junction 下 `fs::read_link` 失败回退 `canonicalize`（`fs.rs:167`） | 直接 `read_link` | P2 统一函数，避 `find`/`scan` 分歧 |
| mtime 精度 | FAT/exFAT 2s | 通常 1ns | 仅作缓存键（P1），不影响真哈希 |
| 环路 | junction 可指向祖先 → 无限递归 | symlink 环 | P4 必须加 `visited` |
| 原子写/损坏回退 | — | — | P5 临时文件 + rename + 默认回退 |
| 路径沙箱 | — | — | P6 `starts_with(allowed_root)` |

## 四、实施顺序与优先级

| 优先级 | 项 | 改动面 | 风险 | 状态（2026-07-14 实现） |
|--------|----|--------|------|------|
| 1 | P4 环路/深度保护 | 中 | 低（先防崩溃） | ✅ 已落地（`agents.rs`/`skills.rs` 递归加 `MAX_SCAN_DEPTH`+`visited`，`SkillsTreeNode`/`DashboardData` 增 `truncated`） |
| 2 | P5 原子写 + 损坏回退 | 小 | 中（必做） | ✅ 已落地（`save_config`/`save_history` 改临时文件+`rename`；`load_config` 损坏回退默认；`load_agents()`+`invalidate_agents_cache()` 缓存，所有增删/链接/迁移命令失效） |
| 3 | P1 哈希缓存 | 中 | 中（真哈希不变） | ✅ 已落地（`hash.rs` 新增 `HashCache`+`dir_hash_into`，缓存键为 mtime/size/count 三元组，对外仍返回真 SHA-256；新增单元测试） |
| 4 | P6 路径沙箱 | 小 | 中（安全） | ✅ 已落地（`preview_skill_at_path` 限制为 vibe 目录/agent 目录；`read_file_from_path`/`write_file_to_path` 拒绝 `..` 逃逸；导出/导入保持对话框路径） |
| 5 | P2 扫描合并 + 链接统一 | 中 | 中（Windows 归一化） | ✅ 已落地（`find_linked_agents` 统一复用 `scan_linked_skills`，消除 Windows junction 归一化分歧） |
| 6 | P3 前端增量 + 本地搜索 | 中 | 中（需改命令签名） | ⚠️ 安全子集已落地：本地搜索（去掉 `search_skills` 后端往返）、`refreshSkills` 防抖、`installSkill` 本地 patch 最新 `Skill`。**未改命令签名为返回 `Skill`**（文档标注的高风险项，需在真机验证后再做） |

> 实施说明：后端 `cargo check` 通过、前端 `pnpm build` 通过、Rust 单元测试通过。P3 的「命令返回 `Skill` 增量 patch」因涉及 5 个 Tauri 命令签名变更且无法在此环境做真机 IPC 回归，暂以低风险前端子集替代，保留后续升级空间。

## 五、验收 / 回归

1. 性能：固定 ~50 skill 数据集，对比 `list_skills` 首刷 vs 缓存命中、连续搜索延迟。
2. 正确性（P1）：改某 skill 文件内容后 `has_conflict` 立即翻转；再次 `list_skills` 命中新哈希。
3. 跨平台（P2/P5）：Windows（junction）与 macOS（symlink）下 `linked_skills` 与「已链接」状态一致。
4. 边界（P4）：构造指向祖先的 junction / symlink 环，验证不再栈溢出、返回 `truncated=true`。
5. 安全（P6）：`read_file_from_path("/etc/passwd")` / `preview_skill_at_path("../../secret")` 应被拒。
6. 持久化（P5）：写入 ` .vibe-config.json ` 中途 kill，重启应回退默认而非崩溃。

## 文件索引

| 文件 | 关联 |
|------|------|
| `src-tauri/src/utils/hash.rs` | P1 |
| `src-tauri/src/commands/skills.rs` | P1/P2/P3/P4 |
| `src-tauri/src/utils/config.rs` | P2/P5/P6 |
| `src-tauri/src/utils/fs.rs` | P2（normalize_path / read_link_target） |
| `src-tauri/src/commands/agents.rs` | P4 |
| `src-tauri/src/commands/sync.rs` | P3/P5 |
| `src/stores/skills.ts` | P3 |
| `src/stores/app.ts` | P6 |
| `src/composables/useSkillAgentStatus.ts` | P1（content_hash 用途） |
