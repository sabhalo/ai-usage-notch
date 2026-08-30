# Step 4.7 — Avvio automatico al login

**Fase:** 4 — Interazione e rifinitura UI
**Stato:** DA FARE
**Dipende da:** Fase 3 chiusa

## Task originale

Avvio automatico al login (`tauri-plugin-autostart`).

## Contesto

Plugin ufficiale Tauri, non richiede logica custom — verificare la versione
compatibile con Tauri v2 (già in uso in questo progetto, vedi
[Cargo.toml](../Cargo.toml)) prima di aggiungerlo.

## Criterio di uscita

Attivando l'opzione (probabilmente nella finestra impostazioni,
step-4.6.md), l'app si avvia da sola al login del sistema; disattivandola,
smette.
