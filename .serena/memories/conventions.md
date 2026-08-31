Non-obvious project conventions. (Rust style is stock `cargo fmt` — nothing custom, no rustfmt.toml.)

## Issue tracker = GitHub issues

Issues/specs live as GitHub issues in `sabhalo/ai-usage-notch`, driven via the `gh` CLI (repo
inferred from `git remote`). Full command patterns in `docs/agents/issue-tracker.md`. "Publish to
the issue tracker" = create a GitHub issue; "fetch the ticket" = `gh issue view <n> --comments`.
PRs are NOT a request/triage surface here.

## Triage labels

Canonical vocabulary = actual label strings (1:1): `needs-triage`, `needs-info`, `ready-for-agent`,
`ready-for-human`, `wontfix`. Table in `docs/agents/triage-labels.md`.

## Domain docs (single-context repo)

`CONTEXT.md` at root is the glossary — when naming a domain concept in an issue title, test name,
hypothesis, or proposal, use its exact term and avoid the listed synonyms. ADRs would go in
`docs/adr/` (none exist yet); if output contradicts an ADR, surface it, don't silently override.
These docs are created lazily by the `/domain-modeling` skill — if a file is absent, proceed
silently, don't flag it. See `docs/agents/domain.md`.

## Misc

- Working branch: `develop` (PRs target `main`).
- Commit messages + code comments + user-facing docs (README) are mostly **Italian**; agent-facing
  docs (`docs/agents/`, `AGENTS.md`) are English. Match the language of the file you're editing.
- Code comments frequently cite `plan/step-N.md` — a historical plan series not in the repo; treat
  as provenance notes, not live docs.
- `docs/plans/*.md` are single-use handoff specs meant to be deleted once executed, not permanent
  docs.
- `AGENTS.md` is the entry point for agent conventions; `CLAUDE.md` just `@`-includes it.
