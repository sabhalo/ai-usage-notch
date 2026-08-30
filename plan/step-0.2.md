# Step 0.2 — Eseguire il probe e salvare le fixture

**Fase:** 0 — Verifica degli endpoint (bloccante)
**Stato:** COMPLETATO

## Task originale

Eseguirlo e salvare le tre risposte reali (anonimizzate) in
`fixtures/{claude,codex,copilot}-usage.json`. Diventano la ground truth per
il parsing e per i test.

## Cosa è stato fatto

Le tre fixture esistono e sono reali (non inventate):

- [fixtures/claude-usage.json](../fixtures/claude-usage.json) — nessun dato
  personale nella risposta, salvata as-is.
- [fixtures/codex-usage.json](../fixtures/codex-usage.json) — `user_id`,
  `account_id`, `email` anonimizzati.
- [fixtures/copilot-usage.json](../fixtures/copilot-usage.json) — `login`,
  `analytics_tracking_id` anonimizzati.

Blocco iniziale su Claude: la voce Keychain `Claude Code-credentials`
conteneva solo token OAuth di plugin MCP (`mcpOAuth`), non
`claudeAiOauth.accessToken`. Causa reale: **due voci Keychain con lo stesso
nome servizio** (una vecchia, una nuova dopo un `claude login` pulito).
`security find-generic-password -s "Claude Code-credentials"` senza `-a`
ne pescava una a caso. Risolto rimuovendo la voce vecchia da Keychain Access.
Se ricapita, disambiguare con `-a <account>`.
