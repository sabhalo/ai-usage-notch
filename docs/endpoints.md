# Endpoint di usage — stato verificato

Ultima verifica: 2026-08-28, via `scripts/probe.sh` su macOS.

## Claude — `GET https://api.anthropic.com/api/oauth/usage`

**Verificato**, risposta reale in [`fixtures/claude-usage.json`](../fixtures/claude-usage.json).

Causa del blocco iniziale: sulla macchina di sviluppo esistevano **due** voci
Keychain chiamate entrambe `Claude Code-credentials` (una vecchia con solo
`mcpOAuth`, plugin MCP; una nuova con `claudeAiOauth` dopo un `claude login`
pulito). `security find-generic-password -s "Claude Code-credentials"` senza
`-a <account>` ne prende una arbitrariamente. Rimossa la voce vecchia da
Keychain Access, `probe.sh` ha funzionato. Se il problema si ripresenta,
occorre disambiguare per `-a <account>`.

Shape reale (`five_hour`, `seven_day`, molti altri campi con nomi in codice
interno/nulli che qui non servono):

```
five_hour.utilization    (float, 0-100)
five_hour.resets_at      (stringa ISO 8601 UTC, non secondi)
seven_day.utilization    (float, 0-100)
seven_day.resets_at      (stringa ISO 8601 UTC)
```

Shape ipotizzata in origine (`session`/`week`, `resets_in_seconds`,
`used_percent`) era **sbagliata** su entrambi i fronti: i nomi delle chiavi
top-level e la forma del reset (data assoluta, non secondi relativi).
`src/main.rs` converte `resets_at` in una durata con un piccolo parser ISO 8601
scritto a mano (nessuna dipendenza aggiunta: formato fisso, calcolo civile di
Hinnant).

## Codex — `GET https://chatgpt.com/backend-api/wham/usage`

**Verificato**, risposta reale in [`fixtures/codex-usage.json`](../fixtures/codex-usage.json).

Lo shape ipotizzato in origine (`rate_limits.primary/secondary` con
`used_percent`/`resets_in_seconds`) era **sbagliato**. Shape reale:

```
rate_limit.primary_window.used_percent          (int, 0-100)
rate_limit.primary_window.reset_after_seconds   (int, secondi)
rate_limit.secondary_window                     (null se assente, altrimenti stessa forma)
```

`rate_limit` è singolare, non `rate_limits`. Non c'è `utilization`, c'è
`used_percent` diretto. `src/main.rs` è stato allineato a questo shape.

## Copilot — `GET https://api.github.com/copilot_internal/user`

**Verificato**, risposta reale in [`fixtures/copilot-usage.json`](../fixtures/copilot-usage.json).

Shape confermata quasi identica a quella ipotizzata nel piano:
`copilot_plan`, `quota_reset_date`, `quota_snapshots.premium_interactions.{entitlement,remaining,percent_remaining,unlimited,overage_permitted}`.
In più rispetto al piano: `quota_id`, `overage_count`, `credits_used`,
`token_based_billing`, `timestamp_utc`. Nessuna sorpresa strutturale.

Token ottenuto con `gh auth token` — ha funzionato nonostante gli scope del
token (`admin:public_key, gist, read:org, repo`) non includano `user`; lo
scope minimo per questo endpoint non è quindi `user` come ipotizzato in Fase
2.1. Non serve il fallback `gh auth refresh -s user` in questo caso.
