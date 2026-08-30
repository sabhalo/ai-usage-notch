# Step 1.5 — Test unitari sui parser (fixture Fase 0)

**Fase:** 1 — Refactor architetturale
**Stato:** DA FARE
**Dipende da:** step-1.1.md

## Task originale

Test unitari sui parser che leggono le fixture della Fase 0. Nessuna rete
nei test.

## Contesto

Questi test **esistono già** e sono verdi, scritti durante la Fase 0 dentro
`src/main.rs` (modulo `#[cfg(test)] mod tests`): 4 test, uno per fixture
(`claude`, `codex`, `copilot`) più uno sul parser ISO 8601 scritto a mano
per Claude. Leggono i file in `fixtures/` da percorso relativo — funzionano
solo se `cargo test` gira dalla root del progetto.

Questo step **non scrive test da zero**: li sposta insieme al codice che
testano quando quel codice si sposta in `src/providers/` (step-1.1.md), e
verifica che restino verdi dopo il move. Se lo spostamento rompe il path
relativo a `fixtures/`, aggiustarlo (es. `env!("CARGO_MANIFEST_DIR")`)
invece di duplicare le fixture dentro `src/providers/`.

## Comando per verificare

```bash
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo test
```

(il primo export serve solo su questa macchina, rustup è keg-only via
Homebrew e non è in `.zshrc` di proposito — non è stato aggiunto senza
chiedere).

## Criterio di uscita

`cargo test` verde con almeno gli stessi 4 test di prima, path a `fixtures/`
robusto rispetto a dove viene lanciato `cargo test`.
