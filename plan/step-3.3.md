# Step 3.3 — Distinguere gli errori in UI

**Fase:** 3 — Robustezza e comportamento runtime
**Stato:** DA FARE
**Dipende da:** step-1.3.md (ProviderError tipizzato)

## Task originale

Distinguere in UI "offline / errore di rete" da "non loggato" da "endpoint
cambiato". Il terzo caso deve dire esplicitamente di rilanciare lo script
di probe.

## Contesto

Questo step è il motivo per cui `ProviderError` è stato tipizzato in
step-1.3.md invece di restare una stringa libera — senza quella base
questo step non è fattibile in modo pulito. Il caso "endpoint cambiato"
corrisponde a `ProviderError::UnexpectedShape` (200 ricevuto ma i campi
attesi non ci sono): il messaggio mostrato in UI per questo caso specifico
deve dire letteralmente di rilanciare `scripts/probe.sh` (o `.ps1` su
Windows) e controllare `docs/endpoints.md`.

## Criterio di uscita

Tre messaggi visivamente/testualmente diversi in UI per i tre casi,
verificato forzando ciascuno (rete disconnessa, token invalido, e — se
possibile simulare — un campo mancante nella risposta).
