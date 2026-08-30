# Step 4.1 — Pill trascinabile con posizione persistita

**Fase:** 4 — Interazione e rifinitura UI
**Stato:** DA FARE
**Dipende da:** Fase 3 chiusa

## Task originale

Rendere la pill trascinabile (`data-tauri-drag-region`) e persistere la
posizione: l'ancoraggio in alto al centro non va bene per tutti.

## Contesto

La posizione persistita va salvata da qualche parte — riusare lo stesso
meccanismo della cache dello step-3.1.md (stesso file o file adiacente
nella stessa cartella dati) invece di inventare un secondo formato di
persistenza.

## Criterio di uscita

Trascinando la pill e riavviando l'app, riappare nella posizione dove è
stata lasciata, non tornata al centro.
