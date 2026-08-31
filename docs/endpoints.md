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
`src/providers/claude.rs` converte `resets_at` in una durata con un piccolo
parser ISO 8601 scritto a mano (nessuna dipendenza aggiunta: formato fisso,
calcolo civile di Hinnant).

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
`used_percent` diretto. `src/providers/codex.rs` è allineato a questo shape.

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

## Gemini — `POST http://127.0.0.1:<porta>/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary`

**Verificato**, risposta reale (nomi/email rimossi, non presenti in questo
endpoint) in [`fixtures/gemini-usage.json`](../fixtures/gemini-usage.json).

Non è un endpoint remoto: vive solo dentro un processo `agy` (Antigravity
CLI) già in esecuzione, su una porta TCP loopback effimera scelta a ogni
sessione. `agy` **non** è un demone — `agy --print "..."` apre ed esce dal
language server nello stesso comando; solo una sessione interattiva
(`agy`, richiede un TTY reale: verificato che fallisce con
`bubbletea: could not open TTY` sotto redirezione pipe) lo tiene su per
tutta la sua durata. Il notch non avvia/gestisce `agy`: si limita a
scoprire una sessione già in corso con `pgrep -x agy` +
`lsof -nP -a -p <pid> -iTCP -sTCP:LISTEN`, e prova ogni porta trovata
finché una risponde (ogni sessione apre due porte, una HTTPS e una HTTP in
chiaro — solo la seconda serve l'RPC).

**Nessun token richiesto.** Ipotesi originale (Bearer da Keychain/file,
come per gli altri tre) **falsificata**: la chiamata POST funziona senza
alcun header di autenticazione — fiducia implicita sul loopback, non un
bearer token applicativo. `src/credentials.rs` non ha quindi bisogno di
alcuna funzione per Gemini.

Shape reale:

```
response.groups[].displayName            (stringa, es. "Gemini Models")
response.groups[].buckets[].bucketId      (stringa: "gemini-weekly" | "3p-weekly")
response.groups[].buckets[].remainingFraction  (float, 0-1)
response.groups[].buckets[].resetTime     (stringa ISO 8601 UTC)
```

Due gruppi, quota settimanale condivisa all'interno di ciascuno: `gemini-weekly`
copre tutti i modelli Gemini (guida l'anello, `windows[0]`), `3p-weekly`
copre Claude e GPT-OSS instradati via Antigravity (informativo, solo nel
pannello dettaglio). L'endpoint alternativo `GetUserStatus` espone lo stesso
dato ripetuto per ogni modello nel piano (14 voci per questo account, tutte
con lo stesso `remainingFraction` all'interno del loro gruppo) — usato per
la sola perlustrazione, `RetrieveUserQuotaSummary` è la fonte scelta perché
già aggregata per gruppo e senza dati personali nel corpo della risposta.

Se nessuna sessione `agy` è attiva: `ProviderError::NotLoggedIn`, distinto
da `UsageWindow.unlimited` (mai `true` per questo provider — la quota
Gemini/Antigravity non è mai illimitata, "non misurabile" e "illimitato"
restano quindi già separati dai due meccanismi esistenti, nessun nuovo
stato UI necessario).
