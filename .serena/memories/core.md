Graph root. Read this first every session.

## What

`ai-usage-notch` — a macOS-first desktop widget (**Tauri v2**, Rust backend + static HTML/CSS/JS
frontend, no Node toolchain) that shows live AI-provider usage percentages in a small always-on-top
floating window ("Pill"), with a per-provider detail "Panel" on click.

- Canonical domain vocabulary (Pill / Visible / Collapsed / Wake / Visibility mode / Panel /
  Open-Closed) is defined in `CONTEXT.md` at repo root. Use those exact terms; do not drift to
  synonyms the glossary lists under `_Avoid_`.
- Providers: **claude, codex, copilot** (exactly these three — a 4th, "Gemini", was removed;
  see `mem:architecture/providers`).

## Source map (repo root)

- `src/` — Rust backend (Tauri commands, provider fetchers, cache, settings, notch positioning).
- `dist/` — static frontend: `index.html`+`main.js`+`style.css` (Pill+Panel), `settings.html`+
  `settings.js`+`settings.css` (settings window). Served as `frontendDist`.
- `fixtures/<provider>-usage.json` — real captured API responses; tests run offline against these.
- `docs/` — `endpoints.md` (real shapes of the unofficial endpoints), `research/`, `agents/`,
  `adr/` (none yet), `plans/` (temporary handoff artifacts, deleted after execution).
- `scripts/probe.{sh,ps1}` — manual re-probe of endpoints when a provider breaks.
- `capabilities/`, `tauri.conf.json`, `build.rs` — Tauri config. `gen/`, `target/` gitignored.

## Further memories

- `mem:build-and-test` — build / run / test / lint / format commands + the "task done" checklist.
- `mem:architecture/overview` — module-by-module backend + frontend layout; points to
  `mem:architecture/providers`.
- `mem:architecture/providers` — provider trait, the `SUPPORTED_PROVIDER_IDS` source-of-truth
  pattern (settings + cache normalisation, frontend mirror), how to add a provider.
- `mem:conventions` — non-obvious project conventions (issue tracker, triage labels, domain-doc
  workflow, commit language, working branch).

Sanity-check references anytime with `serena memories check` from repo root.
