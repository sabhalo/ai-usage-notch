# Step 2.2 — Mapping premium_interactions

**Fase:** 2 — Provider GitHub Copilot
**Stato:** GIÀ FATTO (in Fase 0, dentro `main.rs`) — da migrare
**Dipende da:** step-2.1.md

## Task originale

Implementare il provider mappando `premium_interactions` su
`used_percent = 100 - percent_remaining`.

## Cosa è già stato fatto (Fase 0)

`parse_copilot_usage()` in `main.rs` fa esattamente questo, verificato
contro [fixtures/copilot-usage.json](../fixtures/copilot-usage.json) (dati
reali, non inventati) con test unitario verde. Gestisce già il caso
`unlimited: true` restituendo `used_percent = 0.0` invece di calcolare su
`percent_remaining` (che per le quote illimitate è sempre 100 e darebbe un
falso "0% used" fuorviante se preso alla lettera — vedi step-2.3.md per la
UI, qui è solo il dato).

## Cosa resta da fare in questo step

Spostare `parse_copilot_usage()` in `src/providers/copilot.rs` (vedi
step-1.1.md). Nessuna nuova logica di mapping da scrivere.

## Criterio di uscita

Il test esistente (`parses_real_copilot_fixture`, spostato per step-1.5.md)
resta verde dopo il move.
