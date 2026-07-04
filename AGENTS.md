# AGENTS.md

## What this is

QS-Vibe (qs-vibe-mg) — Tauri 2 desktop app (Rust + Vue 3 + TypeScript) for managing AI coding agent skills via a unified library at `~/.vibe-skills/`. Skills are linked to agent directories through symlinks. Follows the [Agent Skills](https://agentskills.io) open standard.

## Commands

```bash
pnpm install          # Install dependencies
pnpm dev              # Start Vite dev server (frontend only)
pnpm build            # vue-tsc type-check + Vite production build (frontend only)
pnpm tauri dev        # Dev with Tauri (first Rust compile: 3-5 min)
pnpm tauri build      # Production build (bundled)
pnpm preview          # Vite preview of built frontend
cargo test            # Run Rust tests (from src-tauri/)
cargo check           # Rust type-check only (from src-tauri/)
```

**Windows requirement**: Set Rust env vars before any cargo/tauri command:
```
$env:RUSTUP_HOME = "D:\environment\rust\.rustup"
$env:CARGO_HOME = "D:\environment\rust\.cargo"
```

**Windows symlink requirement**: Developer mode must be enabled, or run terminal as admin.

## Architecture

### Rust backend (`src-tauri/src/`)

| Module | Purpose |
|--------|---------|
| `commands/` | Tauri IPC handlers; register new commands in `lib.rs` `invoke_handler` macro |
| `models/` | Data types (serde Serialize/Deserialize) |
| `parsers/` | SKILL.md YAML frontmatter parsing (serde_yaml) |
| `errors.rs` | `VabError` enum — serialized as string for frontend via custom `Serialize` impl |
| `utils/` | `path.rs` (~ expansion), `config.rs` (JSON read/write), `fs.rs` (cross-platform symlinks + `copy_dir_all`), `history.rs` (undo/redo), `datetime.rs` (ISO 8601 formatting) |

### Vue frontend (`src/`)

| Module | Purpose |
|--------|---------|
| `components/layout/` | App shell (AppLayout, TabBar navigation) |
| `components/skills/` | Skill list, card, preview, install dialog |
| `components/agents/` | Agent list, card, add dialog |
| `components/symlink/` | Hierarchical batch symlink config, sync preview |
| `components/dashboard/` | Dashboard showing skill distribution across agents |
| `components/history/` | Undo/redo bar |
| `components/settings/` | Settings page |
| `components/common/` | Shared dialogs (ConfirmDialog, ErrorBanner) |
| `stores/` | Pinia stores: `skills.ts`, `agents.ts`, `app.ts`, `history.ts` |
| `locales/` | `zh.json`, `en.json`, `zh-TW.json` — vue-i18n |
| `types/index.ts` | Shared TypeScript interfaces |

### Supported agents

Default paths: `~/.claude/skills/`, `~/.hermes/skills/` (Windows: `%LOCALAPPDATA%\hermes\skills`), `~/.pi/agent/skills/`, `~/.config/opencode/skills/`, `~/.codex/skills/`, `~/.config/mimocode/skills/`, `~/.agents/skills/`.

### Data flow

- Frontend calls Rust via `invoke()` from `@tauri-apps/api/core`
- Skill scanning: merges `~/.vibe-skills/` with all agent directories, deduplicates by folder name
- Two symlink directions:
  - **Sync** (`sync_agent_to_vibe`): agent_dir → vibe — creates symlinks at `~/.vibe-skills/{agent_id}/{skill}` pointing to agent's real directory
  - **Link** (`create_link`): vibe → agent_dir — creates symlink at `agent_dir/{skill}` pointing to `~/.vibe-skills/{skill}`
- No database — config in `~/.vibe-skills/.vibe-config.json`, history in `~/.vibe-skills/.vibe-history.json`

## Conventions

- **Language**: UI text must be i18n-compatible. Add keys to all three locale files (`zh.json`, `en.json`, `zh-TW.json`)
- **Rust errors**: Use `VabError` enum. Add variant to `errors.rs`, it auto-serializes to string for frontend
- **Frontend state**: All backend calls go through Pinia stores, not directly from components
- **SKILL.md format**: YAML frontmatter (`---` delimiters) with `name`, `description`, optional `license`, `compatibility`, `metadata` fields
- **Package manager**: pnpm only (not npm/yarn)
- **CSS**: Tailwind CSS 4 via `@tailwindcss/vite` plugin — use utility classes, no custom CSS files
- **Rust tests**: Inline `#[cfg(test)] mod tests` (no separate test directory). Files with tests: `utils/path.rs`, `utils/fs.rs`, `utils/datetime.rs`, `parsers/skill_md.rs`