# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Status

**v0.1 completed.** Full feature set including install/delete, preview, batch ops, undo/redo, i18n (zh/en/zh-TW), dark mode, settings page, and custom agent management. See `docs/v0.1/01-plan.md` for the plan.

## What This Is

VIBE Skills Manager — a cross-platform desktop app for managing AI coding agent skills (Claude Code, Hermes, Pi Agent, OpenCode, Codex CLI, MiMo Code, shared agents) via a unified library at `~/.vibe-skills/`. Skills are linked to agent directories through symlinks. Follows the [Agent Skills open standard](https://agentskills.io).

## Tech Stack

- **Desktop**: Tauri 2 (Rust backend + web frontend)
- **Frontend**: Vue 3 + TypeScript + Vite + Pinia + Tailwind CSS 4 + vue-i18n + marked
- **Backend**: Rust with `thiserror` (errors), `serde_yaml` (SKILL.md parsing), `uuid` (history IDs)
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
commands/     Tauri command handlers (skills, sync, agents, history, config)
models/       Data types (skill, agent, history)
parsers/      SKILL.md frontmatter parsing (serde_yaml)
errors.rs     VibeError enum — serialized to string for frontend
utils/        fs, config, path, history helpers
```

### Vue Frontend (`src/`)

```
components/   layout/, skills/, agents/, history/, settings/, common/
stores/       skills.ts, agents.ts, app.ts, history.ts (Pinia)
locales/      zh.json, en.json, zh-TW.json (vue-i18n)
types/        index.ts
```

### Key Design Decisions

- **Skill scanning**: Scans `~/.vibe-skills/` AND all agent directories, merges and deduplicates by folder name
- **SkillSource**: Each skill tracks which directories contain it (vibe-lib or agent id)
- **Symlink direction**: agent directory → symlink → `~/.vibe-skills/{skill}` (source of truth in central library)
- **Agent detection**: Checks if agent skills directory exists on disk
- **No database**: JSON config only (`~/.vibe-skills/.vibe-config.json`)

## Version Roadmap

- **v0.0** (MVP): `docs/v0.0/01-plan.md` — Skill display + symlink management ✅
- **v0.1** (Full): `docs/v0.1/01-plan.md` — Install/delete, preview, batch ops, undo/redo, i18n, theming, settings ✅

## Documentation Index

- `docs/README.md` — Documentation hub
- `docs/01-dev-environment.md` — Environment setup
- `docs/02-modules.md` — Module roadmap
- `docs/03-requirements.md` — v0.1 requirements checklist (R01-R21)
