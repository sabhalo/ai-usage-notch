# Step 3.1 — Cache su disco

**Fase:** 3 — Robustezza e comportamento runtime
**Stato:** DA FARE
**Dipende da:** Fase 2 chiusa

## Task originale

Cache su disco dell'ultimo dato valido
(`%APPDATA%\ai-usage-notch\cache.json`) con timestamp. All'avvio mostra
subito il valore cached invece di "—", marcato come stale se più vecchio di
~10 min.

## Contesto

Path Windows-centrico nel testo originale (`%APPDATA%`) — su macOS
l'equivalente idiomatico è `~/Library/Application Support/ai-usage-notch/`
(via crate `dirs`, già dipendenza del progetto:
`dirs::data_dir()` o `dirs::config_dir()`). Decidere e usare quello, non il
path Windows letterale su Mac.

## Nota di sicurezza (dal piano originale, non derogabile)

La cache contiene solo `UsageResult` (percentuali, label, timestamp) —
**mai** token o credenziali. Verificare che qualunque struct serializzata
su disco non porti dentro per errore il token (es. non salvare l'intero
body raw della risposta se contiene dati sensibili, solo i campi già
parsati).

## Criterio di uscita

All'avvio dell'app con rete assente, si vede l'ultimo valore noto invece di
"—", con un indicatore visivo di "dato non fresco" se più vecchio di 10
minuti.
