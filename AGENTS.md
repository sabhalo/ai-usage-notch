## Agent skills

### Issue tracker

Issues and specs live as GitHub issues in `sabhalo/ai-usage-notch` (via the `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Default canonical vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.

### Serena memories

Versioned domain memories live in `.serena/memories/`. Read `mem:core` first — it's the graph
root and points to the rest (build/test, architecture, conventions). Update them via Serena's
`write_memory` when a stable, non-obvious convention changes; keep to the threshold in
`mem:memory_maintenance`.
