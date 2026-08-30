# Step 2.3 — Casi limite: unlimited e overage

**Fase:** 2 — Provider GitHub Copilot
**Stato:** DA FARE (backend parziale, UI non iniziata)
**Dipende da:** step-2.2.md

## Task originale

Gestire i due casi limite: `unlimited: true` (mostrare "∞" invece di una
percentuale, non un anello a 0%) e `overage_permitted: true` con quota
esaurita (mostrare che sei in overage — è il caso che costa denaro, deve
essere visivamente distinto dal semplice 100%).

## Contesto

Nella fixture reale ([fixtures/copilot-usage.json](../fixtures/copilot-usage.json))
`chat` e `completions` hanno `unlimited: true`, `premium_interactions` ha
`unlimited: false, overage_permitted: true`. Il backend
(`parse_copilot_usage` in `main.rs`, Fase 0) oggi espone **solo**
`premium_interactions` come finestra — `chat`/`completions` unlimited non
sono ancora surfaced verso la UI. Decidere in questo step se vanno
mostrati (e come, dato che sono "∞" quasi sempre) o restano ignorati di
proposito.

Il campo `overage_count` (visto nella fixture reale, valore 0 in quel
momento) è disponibile nel body raw ma non ancora esposto in `UsageResult`
— va aggiunto se la UI deve distinguere "in overage" da "al 100% ma senza
overage".

Questo è **lavoro sia backend (esporre i campi mancanti) che frontend**
(rendering "∞", colore/badge diverso per overage) — non solo parsing.

## Criterio di uscita

Quota illimitata mostra "∞" nell'anello, non uno 0% ambiguo. Overage attivo
è visivamente distinto dal 100% normale (colore diverso, badge, o simile —
decidere in fase di implementazione).
