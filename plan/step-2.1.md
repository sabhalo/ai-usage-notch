# Step 2.1 — Token GitHub per Copilot

**Fase:** 2 — Provider GitHub Copilot
**Stato:** IN GRAN PARTE GIÀ FATTO (in Fase 0, dentro `main.rs`) — da migrare
**Dipende da:** Fase 1 chiusa (architettura a trait/credentials.rs)

## Task originale

Reperire il token GitHub. Ordine di preferenza:
1. `gh auth token` via `std::process::Command` — il percorso più pulito. Se
   manca lo scope, l'errore va tradotto in un suggerimento esplicito:
   `gh auth refresh -h github.com -s user`.
2. Variabile d'ambiente `GITHUB_TOKEN` / `GH_TOKEN`.
3. File di VS Code (`%LOCALAPPDATA%\github-copilot\apps.json`) — path e
   shape non confermati.

## Cosa è già stato fatto (Fase 0)

`copilot_token()` in `main.rs` implementa già i punti 1 e 2, funzionante e
verificato con token reale (`docs/endpoints.md`). **Scoperta non prevista
dal piano originale**: il token `gh auth token` usato in verifica aveva
scope `admin:public_key, gist, read:org, repo` — **senza** `user` — e ha
funzionato comunque. Lo scope minimo reale per questo endpoint non è `user`
come ipotizzato. Il fallback `gh auth refresh -s user` non è quindi
necessario nel caso comune; tenerlo solo come messaggio suggerito per un
401 esplicito, non come assunzione.

Punto 3 (VS Code apps.json) **non implementato**, resta da fare solo se in
pratica servisse (nessuna macchina di test lo ha ancora richiesto).

## Cosa resta da fare in questo step

Spostare `copilot_token()` in `src/credentials.rs` (vedi step-1.2.md) invece
di lasciarla in `main.rs`. Nessuna nuova logica, solo il move.

## Criterio di uscita

`src/credentials.rs` espone la risoluzione del token Copilot con lo stesso
ordine di preferenza, usata da `src/providers/copilot.rs`.
