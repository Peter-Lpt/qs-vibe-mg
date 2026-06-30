# AGENTS.md

## What this is

VAB Skills Manager — Tauri 2 desktop app (Rust + Vue 3 + TypeScript) for managing AI coding agent skills via a unified library at `~/.vibe-skills/`. Skills are linked to agent directories through symlinks.

## Build & run

```bash
pnpm install
pnpm tauri dev        # first Rust compile: 3-5 min
pnpm tauri build      # production build
```

**Windows requirement**: Set Rust env vars before any cargo/tauri command:
```
$env:RUSTUP_HOME = "D:\environment\rust\.rustup"
$env:CARGO_HOME = "D:\environment\rust\.cargo"
```

**Windows symlink requirement**: Developer mode must be enabled, or run terminal as admin.

## Type checking & linting

- **TypeScript**: `vue-tsc --noEmit` (runs as part of `pnpm build`, no standalone script)
- **Rust**: `cargo check` and `cargo test` from `src-tauri/`
- No ESLint, Prettier, or frontend test framework configured

## Architecture

### Rust backend (`src-tauri/src/`)

| Module | Purpose |
|--------|---------|
| `commands/` | Tauri IPC handlers (skills, sync, agents, history, config) |
| `models/` | Data types (skill, agent, history) |
| `parsers/skill_md.rs` | SKILL.md YAML frontmatter parsing |
| `errors.rs` | `VabError` enum — serialized as string for frontend |
| `utils/path.rs` | `~/.vibe-skills/` path resolution, tilde expansion |
| `utils/config.rs` | JSON config read/write |

Register new commands in `lib.rs` `invoke_handler` macro.

### Vue frontend (`src/`)

| Module | Purpose |
|--------|---------|
| `stores/` | Pinia stores: `skills.ts`, `agents.ts`, `app.ts`, `history.ts` |
| `components/` | Organized by domain: `skills/`, `agents/`, `history/`, `settings/`, `layout/`, `common/` |
| `locales/` | `zh.json`, `en.json`, `zh-TW.json` — vue-i18n |
| `types/index.ts` | Shared TypeScript interfaces |

### Data flow

- Frontend calls Rust via `invoke()` from `@tauri-apps/api/core`
- Skill scanning: merges `~/.vibe-skills/` with all agent directories, deduplicates by folder name
- Symlink direction: `agent_dir/skills/{skill_name}` → symlink → `~/.vibe-skills/{skill_name}`
- No database — config stored in `~/.vibe-skills/.vibe-config.json`

## Key conventions

- **Language**: UI text must be i18n-compatible. Add keys to all three locale files (`zh.json`, `en.json`, `zh-TW.json`)
- **Rust errors**: Use `VabError` enum. Add variant to `errors.rs`, it auto-serializes to string for frontend
- **Frontend state**: All backend calls go through Pinia stores, not directly from components
- **SKILL.md format**: YAML frontmatter (`---` delimiters) with `name`, `description`, optional `license`, `compatibility`, `metadata` fields
- **Package manager**: pnpm only (not npm/yarn)
- **CSS**: Tailwind CSS 4 via `@tailwindcss/vite` plugin — use utility classes, no custom CSS files
