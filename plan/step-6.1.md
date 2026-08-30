# Step 6.1 — GitHub Actions build Windows/macOS

**Fase:** 6 — Distribuzione
**Stato:** DA FARE
**Dipende da:** Fase 5 chiusa

## Task originale

GitHub Actions: build su `windows-latest` e `macos-latest`, artifact
allegati alla release su tag.

## Contesto

Nessun workflow CI esiste ancora nel repo (verificare `.github/workflows/`
prima di dare per scontato). I test Rust (`cargo test`, `cargo clippy`)
scritti dalla Fase 0 in poi girano offline su fixture — nessun segreto/rete
necessario in CI per quelli, solo per il bundling finale che non serve
credenziali reali.

## Criterio di uscita

Un push di un tag produce artifact `.dmg` e installer Windows allegati a
una release GitHub, senza intervento manuale oltre al tag stesso.
