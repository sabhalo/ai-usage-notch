# Step 1.1 — Trait UsageProvider

**Fase:** 1 — Refactor architetturale
**Stato:** DA FARE
**Dipende da:** Fase 0 chiusa (fixture + parser verificati in `src/main.rs`)

## Task originale

Estrarre un trait `UsageProvider` con `fn id()`, `async fn fetch() ->
Result<UsageResult, ProviderError>`, e implementarlo in
`src/providers/{claude,codex,copilot}.rs`.

## Contesto

Tutto il codice sorgente oggi vive in un solo file,
[src/main.rs](../src/main.rs) (~600 righe dopo la Fase 0): tre comandi Tauri
(`get_claude_usage`, `get_codex_usage`, `get_copilot_usage`), tre funzioni di
parsing pure già testate (`parse_claude_usage`, `parse_codex_usage`,
`parse_copilot_usage`), più helper condivisi (`fmt_reset`, `fmt_window`,
`parse_iso_utc_epoch`, `resets_in_from_iso`, `err`).

Questo step sposta la logica di fetch+parse in tre moduli sotto
`src/providers/`, dietro un trait comune. Le funzioni di parsing esistenti
vanno spostate così come sono (sono già corrette e testate, vedi Fase 0) —
non riscriverle da zero.

`UsageResult`/`UsageWindow` restano condivisi (probabilmente in
`src/providers/mod.rs` o in un `src/types.rs`).

## Nota — `ProviderError` non ancora tipizzato

Questo step dipende in parte dallo step 1.3 (tipizzare `ProviderError`): ha
senso definire `ProviderError` per primo (o insieme) così che `fetch()`
ritorni subito il tipo giusto invece di essere rifatto due volte. Valutare
se accorpare 1.1 e 1.3 in un solo passaggio.

## Criterio di uscita

`src/providers/claude.rs`, `src/providers/codex.rs`,
`src/providers/copilot.rs` esistono, ciascuno implementa il trait, `cargo
build` passa. I test della Fase 0 (spostati insieme alle funzioni) restano
verdi.
