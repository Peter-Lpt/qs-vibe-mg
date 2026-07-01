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

| Module | Files | Purpose |
|--------|-------|---------|
| `commands/` | `skills.rs`, `sync.rs`, `agents.rs`, `history.rs`, `config.rs` | Tauri IPC handlers; register new commands in `lib.rs` `invoke_handler` macro |
| `models/` | `skill.rs`, `agent.rs`, `history.rs`, `dashboard.rs`, `sync.rs` | Data types |
| `parsers/` | `skill_md.rs` | SKILL.md YAML frontmatter parsing (serde_yaml) |
| `errors.rs` | — | `VabError` enum — serialized as string for frontend |
| `utils/` | `path.rs`, `config.rs`, `fs.rs`, `history.rs` | Path resolution (~ expansion), JSON config read/write, cross-platform symlinks, undo/redo history |

### Vue frontend (`src/`)

| Module | Purpose |
|--------|---------|
| `components/layout/` | App shell (AppLayout, TabBar navigation) |
| `components/skills/` | Skill list, card, preview, install dialog |
| `components/symlink/` | Hierarchical batch symlink config, sync preview |
| `components/dashboard/` | Dashboard showing skill distribution across agents |
| `components/cli/` | CLI tool discovery and management |
| `components/history/` | Undo/redo bar |
| `components/settings/` | Settings page |
| `components/common/` | Shared dialogs (ConfirmDialog, ErrorBanner) |
| `stores/` | Pinia stores: `skills.ts`, `agents.ts`, `app.ts`, `history.ts` |
| `locales/` | `zh.json`, `en.json`, `zh-TW.json` — vue-i18n |
| `types/index.ts` | Shared TypeScript interfaces |

### Supported agents

Default paths: `~/.claude/skills/`, `~/.hermes/skills/`, `~/.pi/agent/skills/`, `~/.config/opencode/skills/`, `~/.codex/skills/`, `~/.config/mimocode/skills/`, `~/.agents/skills/`.

### Data flow

- Frontend calls Rust via `invoke()` from `@tauri-apps/api/core`
- Skill scanning: merges `~/.vibe-skills/` with all agent directories, deduplicates by folder name
- Symlink direction: `agent_dir/skills/{skill_name}` → symlink → `~/.vibe-skills/{skill_name}`
- No database — config in `~/.vibe-skills/.vibe-config.json`, history in `~/.vibe-skills/.vibe-history.json`

## Conventions

- **Language**: UI text must be i18n-compatible. Add keys to all three locale files (`zh.json`, `en.json`, `zh-TW.json`)
- **Rust errors**: Use `VabError` enum. Add variant to `errors.rs`, it auto-serializes to string for frontend
- **Frontend state**: All backend calls go through Pinia stores, not directly from components
- **SKILL.md format**: YAML frontmatter (`---` delimiters) with `name`, `description`, optional `license`, `compatibility`, `metadata` fields
- **Package manager**: pnpm only (not npm/yarn)
- **CSS**: Tailwind CSS 4 via `@tailwindcss/vite` plugin — use utility classes, no custom CSS files

## Notes

(Add quick notes here as needed — link related memories with [[mention]].)