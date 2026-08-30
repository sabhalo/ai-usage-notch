# Step 1.3 — ProviderError tipizzato

**Fase:** 1 — Refactor architetturale
**Stato:** DA FARE
**Dipende da:** step-1.1.md

## Task originale

Sostituire i `match` annidati con un `ProviderError` tipizzato
(`NotLoggedIn`, `Unauthorized`, `Network`, `UnexpectedShape`), così che la UI
possa mostrare un messaggio diverso e utile per ciascun caso.

## Contesto

Oggi (Fase 0) ogni funzione `get_*_usage` ritorna un `UsageResult` con un
campo `error: Option<String>` costruito da una funzione `err(provider, msg)`
— messaggi liberi, nessuna distinzione tipizzata tra "non loggato" e
"risposta inattesa". Questo è esattamente il problema che questo step
risolve.

Casi reali già osservati che il tipo deve coprire:
- Claude: voce Keychain assente → `NotLoggedIn`. Voce Keychain presente ma
  senza `claudeAiOauth` (è successo per davvero, vedi step-1.2.md) →
  probabilmente `UnexpectedShape` o un caso dedicato, da decidere.
- Codex: `~/.codex/auth.json` assente → `NotLoggedIn`.
- Copilot: `gh auth token` fallisce o token senza permessi → `NotLoggedIn`
  o `Unauthorized`, da distinguere se possibile (way exit code di `gh`
  diverso da mancanza di scope).
- Qualsiasi endpoint risponde 401 → `Unauthorized`. Timeout/DNS/connessione
  → `Network`. 200 ma campi mancanti → `UnexpectedShape`.

## Nota di sicurezza (dal piano originale, non derogabile)

Nessun `ProviderError` deve mai includere il token nel messaggio. I
messaggi di errore attuali in `main.rs` non lo fanno (controllato in Fase
0) — mantenere questa proprietà nel nuovo tipo.

## Criterio di uscita

Un `enum ProviderError` in uso da tutti e tre i provider, `UsageResult`
usa `Result<UsageResult, ProviderError>` invece di un campo `error: Option<String>`
libero. `cargo clippy -- -D warnings` pulito.
