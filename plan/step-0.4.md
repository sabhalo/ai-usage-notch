# Step 0.4 — docs/endpoints.md e fallback 404/401

**Fase:** 0 — Verifica degli endpoint (bloccante)
**Stato:** COMPLETATO

## Task originale

Se un endpoint risponde 404/401 anche con credenziali valide, annotarlo in
`docs/endpoints.md` e valutare il fallback: per Copilot esiste un SDK
ufficiale (`account.get_quota`) che è la via documentata e va preferita se
il percorso interno si rivela fragile.

## Cosa è stato fatto

Nessuno dei tre endpoint ha risposto 404/401 con credenziali valide — tutti
e tre hanno dato 200 con dati reali una volta risolto il problema della
voce Keychain duplicata per Claude (vedi step-0.2.md). Il fallback SDK
ufficiale per Copilot non è stato necessario.

[docs/endpoints.md](../docs/endpoints.md) contiene, per ciascun provider:
data dell'ultima verifica, shape reale osservata, differenze rispetto alle
ipotesi originali del piano. Da tenere aggiornato quando qualcosa si rompe.

## Criterio di uscita della Fase 0 — verificato

- [x] `fixtures/claude-usage.json`, `fixtures/codex-usage.json`,
  `fixtures/copilot-usage.json` esistono e sono reali.
- [x] Tre parser che li leggono correttamente in un test unitario offline
  (`cargo test`, 4 test verdi, nessuna rete).
