Real commands. macOS (Darwin), zsh. Prereqs: Rust toolchain + `cargo install tauri-cli --version "^2"`.
No `justfile`/`Makefile`/`package.json` — everything is cargo / cargo-tauri.

## Dev / run / build

- `cargo tauri dev` — run the widget in dev.
- `cargo tauri build` — bundle (`target/release/bundle/{macos,dmg,nsis}/`).
- `cargo build` — backend compile check only (no bundle).

## Tests

- `cargo test` — full suite. **Offline by design**: every provider test reads
  `fixtures/<provider>-usage.json`, no network. Do not add tests that hit the real endpoints or
  touch the user's real `settings.json` / cache files under `~/Library/Application Support`.
- Provider-parsing tests live next to each provider (`src/providers/<p>.rs` `#[cfg(test)]`), plus a
  `tests` module in `src/providers/mod.rs`.

## Task-completion checklist (run before declaring a change done)

1. `cargo fmt --check`  (run `cargo fmt` first if it fails; then re-check the diff for stray edits)
2. `cargo test`  — criterion is "all present tests pass", never a fixed count
3. `cargo clippy --all-targets -- -D warnings`  — CI treats warnings as errors
4. `cargo build`  — no new warnings/errors

CI (`.github/workflows/release.yml`) runs exactly 2+3+`cargo tauri build` on macOS+Windows, but only
on `v*` tag push — there is no PR/CI check on ordinary pushes, so run the checklist locally.

## Endpoint drift

Providers use undocumented endpoints. If one starts reporting "response received but no recognised
field": re-run `scripts/probe.sh`, diff against `docs/endpoints.md`, fix parsing in the provider
module. See `mem:architecture/providers`.
