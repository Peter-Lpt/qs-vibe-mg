# QS-Vibe-MG

> Cross-platform AI agent skills manager — Tauri 2 + Vue 3 + Rust

Manage and sync skills across Claude Code, Codex CLI, Hermes, OpenCode, MiMo Code, Pi Agent and more via a unified library.

[简体中文](./README.zh.md) | English

---

## Features

### Skill Management

- Unified view of all agent skills, filter by source, status, and agent relationship
- Sync from agent directory to unified library, or link from library to agent
- Batch operations: sync, link, repair conflicts and broken links
- Skill workbench view with adaptive single/multi-agent layout

### Plugin Support

- Scan Claude Code and Codex plugin marketplace cache
- Show plugin enable/disable status
- Claim plugin skills into unified library
- Detect plugin updates (content hash comparison)
- Batch update plugins from the same marketplace

### Update Detection

- **Git**: `git fetch` + commit diff, `git pull --ff-only` for updates
- **NPX/NPM**: `npm view` for remote version check, re-run install command
- **Plugin**: content hash comparison, sync latest files from cache
- TTL cache to avoid frequent checks

### Source Provenance

- Track installation source for each skill (Git / local folder / NPM / NPX / plugin marketplace)
- Record Git remote URL, commit SHA, branch metadata
- Source trust level inference and update status marking

### Agent Management

- Auto-detect skill directories for common agents
- Custom agent configuration support
- Agent enable/disable control

### Others

- Operation history with undo/redo
- Light / dark theme
- Simplified Chinese / English / Traditional Chinese

## Supported Agents

| Agent | Default Skills Directory |
|-------|-------------------------|
| Claude Code | `~/.claude/skills/` |
| Hermes | `%LOCALAPPDATA%/hermes/skills/` |
| Pi Agent | `~/.pi/agent/skills/` |
| OpenCode | `~/.config/opencode/skills/` |
| Codex CLI | `~/.codex/skills/` |
| MiMo Code | `~/.config/mimocode/skills/` |
| Shared | `~/.agents/skills/` |

Custom paths can be configured in Settings.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop | Tauri 2 |
| Backend | Rust |
| Frontend | Vue 3 + TypeScript |
| Build | Vite |
| State | Pinia |
| Styling | Tailwind CSS 4 |
| i18n | vue-i18n |
| Icons | Lucide |

## Getting Started

### Prerequisites

- Node.js 18+
- pnpm
- Rust stable
- Tauri 2 system dependencies ([official docs](https://v2.tauri.app/start/prerequisites/))

### Install & Run

```bash
pnpm install
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

### Windows Notes

- Symlink requires Developer Mode or Administrator privileges
- If using non-default Rust paths, set `RUSTUP_HOME` and `CARGO_HOME` environment variables

## License

[MIT](LICENSE)
