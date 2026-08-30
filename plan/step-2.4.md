# Step 2.4 — Dettaglio remaining/entitlement e reset date

**Fase:** 2 — Provider GitHub Copilot
**Stato:** BACKEND GIÀ FATTO (Fase 0), FRONTEND DA FARE
**Dipende da:** step-2.2.md

## Task originale

Nel dettaglio mostrare `remaining/entitlement` (es. "211/300 premium") e la
data di reset, non solo la percentuale: per una quota mensile il numero
assoluto è più azionabile.

## Cosa è già stato fatto (Fase 0)

`parse_copilot_usage()` in `main.rs` costruisce già un `label` del tipo
`"Premium requests (216/15000)"` usando `remaining`/`entitlement` dal body
reale, e mette `quota_reset_date` (stringa `"2026-09-01"`, non un timestamp)
in `resets_in`. Verificato contro dati reali.

## Cosa resta da fare in questo step

Solo lavoro **frontend**: verificare che [dist/main.js](../dist/main.js) /
[dist/index.html](../dist/index.html) mostrino davvero questo dettaglio nel
pannello espandibile, invece di limitarsi alla percentuale nell'anello
principale (che è già così per Claude/Codex, verificare come sono resi
oggi prima di aggiungere Copilot allo stesso pattern).

## Criterio di uscita

Aprendo il pannello dettagli di Copilot si vede "211/300 premium" (o
equivalente) e la data di reset, non solo una percentuale.
