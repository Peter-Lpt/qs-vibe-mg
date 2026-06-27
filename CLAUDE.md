# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Status

**v0.0 completed.** Core skills management (display + symlink) is functional. See `docs/v0.0/01-plan.md` for the completed plan.

## What This Is

VAB Skills Manager — a cross-platform desktop app for managing AI coding agent skills (Claude Code, Hermes, Pi Agent, OpenCode, Codex CLI, MiMo Code, shared agents) via a unified library at `~/.vab-skills/`. Skills are linked to agent directories through symlinks. Follows the [Agent Skills open standard](https://agentskills.io).

## Tech Stack

- **Desktop**: Tauri 2 (Rust backend + web frontend)
- **Frontend**: Vue 3 + TypeScript + Vite + Pinia + Tailwind CSS 4
- **Backend**: Rust with `thiserror` (errors), `serde_yaml` (SKILL.md parsing)
- **Package manager**: pnpm 8+

## Build Commands

```bash
pnpm install          # Install dependencies
pnpm tauri dev        # Start dev server (first Rust compile: 3-5 min)
pnpm tauri build      # Production build
```

> **Windows note**: Set `RUSTUP_HOME=D:\environment\rust\.rustup` and `CARGO_HOME=D:\environment\rust\.cargo` before cargo/tauri commands.

## Architecture

### Rust Backend (`src-tauri/src/`)

```
commands/     Tauri command handlers (skills, sync, agents)
models/       Data types (skill, agent)
parsers/      SKILL.md frontmatter parsing (serde_yaml)
errors.rs     VabError enum — serialized to string for frontend
utils/        fs, config, path helpers
```

### Vue Frontend (`src/`)

```
components/   layout/, skills/, agents/
stores/       skills.ts, agents.ts (Pinia)
types/        index.ts
```

### Key Design Decisions

- **Skill scanning**: Scans `~/.vab-skills/` AND all agent directories, merges and deduplicates by folder name
- **SkillSource**: Each skill tracks which directories contain it (vab-lib or agent id)
- **Symlink direction**: agent directory → symlink → `~/.vab-skills/{skill}` (source of truth in central library)
- **Agent detection**: Checks if agent skills directory exists on disk
- **No database**: JSON config only (`~/.vab-skills/.vab-config.json`)

## Version Roadmap

- **v0.0** (MVP): `docs/v0.0/01-plan.md` — Skill display + symlink management ✅
- **v0.1** (Full): `docs/v0.1/01-plan.md` — Drag-and-drop, undo/redo, batch ops, i18n, theming

## Documentation Index

- `docs/README.md` — Documentation hub
- `docs/01-dev-environment.md` — Environment setup
- `docs/02-modules.md` — Module roadmap
- `docs/03-requirements.md` — v0.1 requirements checklist (R01-R21)
