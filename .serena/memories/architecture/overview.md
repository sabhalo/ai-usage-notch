Module map. Backend in `src/`, frontend in `dist/`. IPC = Tauri commands (async fns marked
`#[tauri::command]` in `src/main.rs`, invoked from JS via `withGlobalTauri`).

## Backend (`src/`)

- `main.rs` — Tauri command surface + `main()` setup (window creation, notch positioning). Commands:
  `get_cached_usage`, `get_settings`, `save_settings`, `save_window_position`, `get_all_usage`.
  `get_all_usage(skip)` fans out to the 3 providers in parallel via `tokio::join!` + `maybe_fetch`,
  then merges fresh results with on-disk cache so a backed-off provider never loses its last value.
- `providers/` — one module per provider + shared `mod.rs` (trait `UsageProvider`, `UsageReport`,
  `UsageResult`, `UsageWindow`, `ProviderError`/`FetchError`, shared ISO-date helpers, and the
  `SUPPORTED_PROVIDER_IDS` const). Details + how to add a provider: `mem:architecture/providers`.
- `credentials.rs` — per-platform token resolution: macOS Keychain (Claude), credentials file
  (Codex), `gh auth token` (Copilot). No API keys are ever configured by hand.
- `cache.rs` — last known-good `Vec<UsageReport>` on disk, shown instantly at startup before the
  first fetch. `load()`/`save()` both filter to `SUPPORTED_PROVIDER_IDS` (`supported_reports()`).
- `settings.rs` — active providers, refresh interval, alert threshold, visibility mode,
  auto-collapse delay, window position. JSON in the app config dir. `load()`/`save()` both run
  `normalize()` against `SUPPORTED_PROVIDER_IDS`.
- `notch.rs` — macOS-only, positions the Pill next to the physical notch via `objc2-app-kit`
  (`NSScreen.safeAreaInsets` / `auxiliaryTopLeftArea`).

## Frontend (`dist/`, static, no build step)

- `index.html` + `main.js` + `style.css` — the Pill (one ring per active provider) and the Panel.
  `main.js` centralises provider ids in `PROVIDER_IDS` (keep it centralised; don't fan back out to
  per-provider arrays for state/backoff/etc.). Frontend owns the per-provider backoff and defends
  against unknown providers in cache/tick reports.
  - **`style.css` geometry is in `rem` (base 16), not px.** The `pill_scale` setting drives the
    `--pill-scale` custom property → `html { font-size: calc(16px * var(--pill-scale)) }`, so every
    rem measure scales together and `applyLayout()`'s DOM measurement stays truthful (no
    `transform: scale`/`zoom`). Write new Pill/Panel rules in `rem`. Exceptions kept in px: hairlines
    (`border: 1px`, `.divider` width, `.detail-error` `border-left: 3px`) and SVG `stroke-width` /
    `r` / viewBox coords (already scale with the element's CSS size). `main.js: applyPillScale()`
    clamps to 0.7–2.0, mirroring `settings::PILL_SCALE_MIN/MAX`.
- `settings.html` + `settings.js` + `settings.css` — the settings window (one checkbox per
  provider + the other settings).
- `icons/` — per-provider icons referenced by the markup (`claude.png`, `codex.svg`,
  `copilot.webp`; stale `.svg` variants may linger).

## Windows (`tauri.conf.json`)

Two: `main` (transparent, always-on-top, no decorations, `macOSPrivateApi` — this rules out Mac App
Store distribution) and `settings` (hidden until opened).
