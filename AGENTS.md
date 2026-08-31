## Agent skills

### Issue tracker

Issues and specs live as GitHub issues in `sabhalo/ai-usage-notch` (via the `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Default canonical vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.

### Serena workflow

At the start of a session, activate the current repository with Serena and read its initial
instructions. Versioned domain memories live in `.serena/memories/`; read `mem:core` first — it is
the graph root and points to the rest (build/test, architecture, conventions).

Use Serena's symbolic tools for code navigation, reference discovery, and symbol-level edits when
they fit the task. Use regular filesystem tools for non-code files and when Serena has no suitable
tool. Update memories through Serena only when a stable, non-obvious convention changes, following
the threshold in `mem:memory_maintenance`.
