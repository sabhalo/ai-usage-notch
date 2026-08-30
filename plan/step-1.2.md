# Step 1.2 — src/credentials.rs multi-piattaforma

**Fase:** 1 — Refactor architetturale
**Stato:** DA FARE
**Dipende da:** step-1.1.md (struttura provider)

## Task originale

Spostare la risoluzione delle credenziali in `src/credentials.rs`, con
implementazioni `#[cfg(target_os)]` distinte (file su Windows/Linux,
Keychain via `security find-generic-password` su macOS per Claude).

## Contesto — punto di partenza già scritto

In `src/main.rs` (Fase 0) esiste già una funzione `claude_token()` con due
varianti `#[cfg(target_os = "macos")]` / `#[cfg(not(target_os = "macos"))]`:
la versione macOS chiama `security find-generic-password -s "Claude
Code-credentials" -w` e parsa il JSON risultante (`claudeAiOauth.accessToken`
o `accessToken` come fallback). Funziona, verificata su questa macchina.

Questo step la sposta in `src/credentials.rs` insieme alla lettura di
`~/.codex/auth.json` (Codex) e al recupero del token GitHub (`gh auth token`
→ `$GITHUB_TOKEN`/`$GH_TOKEN`, già scritto come `copilot_token()` in
`main.rs`).

## Attenzione — bug reale già incontrato

Su macOS possono esistere **più voci Keychain con lo stesso nome servizio**
`Claude Code-credentials` (è successo in Fase 0: una vecchia con solo
`mcpOAuth`, una nuova con `claudeAiOauth`). `security
find-generic-password -s ...` senza `-a <account>` ne prende una a caso.
Non è stato ancora deciso come gestirlo in modo robusto (chiedere
esplicitamente l'account? Provare tutte le voci finché una ha
`claudeAiOauth`?) — valutare in questo step, non ignorare il problema.

## Criterio di uscita

`src/credentials.rs` con una funzione per provider, usata dai moduli in
`src/providers/`. Nessuna logica di credenziali rimasta in `main.rs`.
