# Step 0.1 — Scrivere probe.ps1 e probe.sh

**Fase:** 0 — Verifica degli endpoint (bloccante)
**Stato:** COMPLETATO

## Task originale

Scrivere `scripts/probe.ps1` (e gemello `probe.sh`) che chiama i tre endpoint
con le credenziali locali e stampa il JSON grezzo indentato, senza mai
stampare il token. Deve funzionare in isolamento, prima dell'app.

## Cosa è stato fatto

- [scripts/probe.sh](../scripts/probe.sh): macOS/Linux. Claude da Keychain
  (macOS, servizio `Claude Code-credentials`) o file (Linux,
  `~/.claude/.credentials.json`); Codex da `~/.codex/auth.json`; Copilot da
  `gh auth token` con fallback `$GITHUB_TOKEN`/`$GH_TOKEN`.
- [scripts/probe.ps1](../scripts/probe.ps1): gemello Windows, stessa logica,
  path `%USERPROFILE%`/`%LOCALAPPDATA%`. Non testato (nessun Windows/pwsh
  disponibile in questo ambiente) — da verificare al primo giro su Windows.
- Nessuno dei due stampa mai il token, solo lo status HTTP e il body.
