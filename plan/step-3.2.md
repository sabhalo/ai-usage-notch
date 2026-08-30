# Step 3.2 — Backoff esponenziale sui fallimenti

**Fase:** 3 — Robustezza e comportamento runtime
**Stato:** DA FARE
**Dipende da:** Fase 2 chiusa

## Task originale

Backoff esponenziale sui fallimenti (30s → 5min → 15min, cap) invece del
polling fisso a 90s. Gli endpoint sono non ufficiali: martellarli dopo un
401 è il modo migliore per farsi notare.

## Contesto

Verificare dove vive oggi il polling fisso (probabilmente nel frontend,
`dist/main.js`, con un `setInterval`) prima di decidere se il backoff va
implementato lato frontend (più semplice, stato in memoria JS) o lato Rust
(persistente tra restart, più complesso). Il piano non lo specifica — è una
scelta di design da fare qui, non assunta.

Il backoff dovrebbe essere **per provider**, non globale: se Claude dà 401
ma Codex risponde bene, non ha senso rallentare anche Codex.

## Criterio di uscita

Un fallimento ripetuto (es. 401 simulato disconnettendo il token) allunga
l'intervallo di retry fino al cap di 15 minuti invece di martellare ogni
90s: verificabile nei log/devtools della finestra, non solo a leggere il
codice.
