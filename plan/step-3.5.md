# Step 3.5 — Provider non configurato non deve rompere la UI

**Fase:** 3 — Robustezza e comportamento runtime
**Stato:** DA FARE
**Dipende da:** step-1.3.md, step-1.4.md

## Task originale

Il provider non configurato (es. Codex non installato) non deve mostrare
un anello rotto — nasconderlo, o mostrarlo grigio con tooltip esplicativo.
Il widget deve essere utile anche con un solo provider attivo.

## Contesto

Corrisponde principalmente a `ProviderError::NotLoggedIn` (vedi
step-1.3.md) quando è il **primo** tentativo (non un fallimento dopo aver
funzionato) — distinguere questo caso da un provider che ha funzionato e
poi ha iniziato a fallire (quello è lo step-3.3.md, mostra errore; questo è
"non configurato", mostra grigio/nascosto senza allarmare).

## Criterio di uscita

Disconnettendo/rimuovendo le credenziali di un solo provider (es.
rinominando temporaneamente `~/.codex/auth.json`), quell'anello appare
grigio con tooltip, non rotto/vuoto, e gli altri due continuano a
funzionare normalmente.
