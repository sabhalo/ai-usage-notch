# Step 2.5 — Terzo anello nella pill

**Fase:** 2 — Provider GitHub Copilot
**Stato:** DA FARE
**Dipende da:** step-1.4.md (get_all_usage), step-2.2.md

## Task originale

Aggiungere il terzo anello nella pill. A 3 provider la larghezza fissa di
220px va rivista — valutare 300px, o anelli senza etichetta numerica con la
percentuale solo all'hover.

## Contesto

Lavoro puramente frontend, su [dist/index.html](../dist/index.html) /
[dist/style.css](../dist/style.css) / [dist/main.js](../dist/main.js). Va
fatto **dopo** che `get_all_usage()` (step-1.4.md) ritorna anche il dato
Copilot, altrimenti non c'è nulla da renderizzare nel terzo anello.

Nessuna decisione presa finora su 300px vs "solo hover" — è una scelta
visiva da verificare con `cargo tauri dev` e uno screenshot reale, non da
decidere solo leggendo il codice.

## Criterio di uscita

Tre anelli visibili e leggibili nella pill, verificato visivamente (non
solo che il codice compili) con `cargo tauri dev` — vedi anche il criterio
di uscita generale della Fase 2 sotto.

## Criterio di uscita della Fase 2 (tutta)

Tre anelli live, quota Copilot corretta confrontata a mano con
`github.com/settings/billing`.
