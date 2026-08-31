How the provider layer works and how to add/change one. All in `src/providers/`.

## Contract (`mod.rs`)

- `trait UsageProvider { fn id(&self) -> &'static str; async fn fetch(&self) -> Result<UsageResult, FetchError>; }`
  — each provider is a concrete zero-size struct: `ClaudeProvider`, `CodexProvider`, `CopilotProvider`.
- Shared types: `UsageReport` (what the frontend consumes), `UsageResult`, `UsageWindow`,
  `ProviderError` (enum, typed failure reasons), `FetchError`. Shared helpers:
  `parse_iso_utc_epoch`, `resets_in_from_iso`, `parse_retry_after_header`, `too_many_requests`,
  `fmt_reset`, `fmt_window`. `#[cfg(test)] mod tests` here covers the shared helpers.
- No dynamic registry. Providers are wired by hand in `src/main.rs::get_all_usage` as a fixed
  `tokio::join!` of `maybe_fetch(want("<id>"), <Provider>)` — concrete types differ, deliberately
  not generic.

## Supported-providers source of truth

`pub(crate) const SUPPORTED_PROVIDER_IDS: [&str; 3] = ["claude", "codex", "copilot"]` in `mod.rs` is
the backend authority. Consumed by:

- `settings.rs::normalize()` — run on every `load()` AND `save()`: drops `active_providers` keys not
  in the list, adds missing supported ones defaulting to `true`. Prevents stale/unknown providers
  persisting.
- `cache.rs::supported_reports()` — run on every `load()` AND `save()`: filters `reports` to
  supported providers only.
- `settings.rs::default_active_providers()`.

Frontend mirrors this with `const PROVIDER_IDS = ["claude","codex","copilot"]` in `dist/main.js`
(single source: drives `state`/`everSucceeded`/`backoff`/`activeProviders` — keep centralised, do
not fan back out). `loadCachedUsage()` and `tick()` skip any report whose provider is not in
`PROVIDER_IDS`; settings load rebuilds `activeProviders` strictly from `PROVIDER_IDS`.

## To add a provider

1. `src/providers/<id>.rs`: struct + `impl UsageProvider`, parser fn, `#[cfg(test)]` tests reading
   the fixture.
2. Add `<id>` to `SUPPORTED_PROVIDER_IDS`; register module in `mod.rs` (`pub mod <id>; pub use
   <id>::<Id>Provider;`); add a `tokio::join!` arm in `main.rs::get_all_usage`.
3. `fixtures/<id>-usage.json` — real captured response (capture via `scripts/probe.*`).
4. `dist/icons/<id>.<ext>` + wire into `dist/index.html`, `dist/main.js` `PROVIDER_IDS`/
   `PROVIDER_TITLES`/`RING_IDS` + click listener, `dist/settings.html` + `dist/settings.js` checkbox.
5. `docs/endpoints.md` — document the real endpoint shape. Optional `docs/research/<id>-*.md`.

## History

A 4th provider "Gemini" (module `gemini`, `GeminiProvider`, local `agy`/Antigravity RPC, GitHub
issue #7) was removed entirely in Sept 2026. Do not reintroduce it. Git history and issue #7 remain
as the only references.
