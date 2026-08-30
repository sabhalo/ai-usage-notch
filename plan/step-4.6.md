# Step 4.6 — Finestra impostazioni minima

**Fase:** 4 — Interazione e rifinitura UI
**Stato:** DA FARE
**Dipende da:** step-3.1.md (persistenza), step-4.3.md (soglia)

## Task originale

Finestra impostazioni minima (provider attivi, intervallo di refresh,
soglia di allerta, posizione) persistita in JSON.

## Contesto

Riusa lo stesso file/meccanismo di persistenza dello step-3.1.md (cache) e
step-4.1.md (posizione) invece di un terzo formato — probabilmente ha
senso un unico `settings.json` con sezioni distinte piuttosto che file
separati per ogni singola impostazione.

## Criterio di uscita

Una finestra (anche minimale) dove cambiare provider attivi, intervallo di
refresh e soglia; le modifiche sopravvivono a un riavvio dell'app.
