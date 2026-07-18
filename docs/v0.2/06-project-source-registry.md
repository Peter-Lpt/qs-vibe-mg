# Project Source Registry

## Goal

Make project-level skill discovery explicit, manageable, and conservative.

## Current decision

- Keep `project_roots` as a real source registry.
- Do not silently fall back to `cwd` when the list is empty.
- Provide suggested roots from the current folder and ancestor folders.
- Keep automatic discovery as a suggestion, not as implicit scanning.

## UX rules

1. Primary control: structured list editor.
2. Secondary control: browse current folder / add suggested root.
3. Advanced control: raw text editor hidden behind a toggle.
4. Empty state: show suggestions instead of auto-scanning.

## Scan scope

Default project conventions:

- `.claude/skills`
- `.agents/skills`
- `.codex/skills`
- `.github/skills`
- `skills`

Notes:

- The scan scope stays narrow by default.
- Do not recurse into arbitrary folders.
- Do not treat empty `project_roots` as implicit permission to scan the working directory.

## Data model

- `project_roots`: configured roots saved in `.vibe-config.json`
- `suggest_project_roots`: runtime suggestions from the current folder and parent folders
- `project:<root>`: internal project source identifier

## Follow-up ideas

- Add per-root validation badges.
- Show the number of skills contributed by each root.
- Add a dedicated Project Sources page if this area keeps growing.
