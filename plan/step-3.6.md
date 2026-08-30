# Step 3.6 — Multi-schermo e DPI scaling

**Fase:** 3 — Robustezza e comportamento runtime
**Stato:** DA FARE — richiede hardware reale, non testabile in cloud/headless
**Dipende da:** Fase 2 chiusa

## Task originale

Verificare il comportamento su schermi multipli e con DPI scaling diverso
da 100% — il calcolo della posizione in `setup()` usa `scale_factor()` ma
non l'ho testato su una configurazione reale.

## Contesto

Il codice attuale (`main.rs`, funzione `setup()`) calcola la posizione
usando `window.primary_monitor()` e `monitor.scale_factor()`. Su macOS con
un solo schermo Retina questo probabilmente va bene per costruzione, ma
"probabilmente" non è verificato — questo step è esplicitamente su hardware
reale con più monitor e/o scaling non-100%, non simulabile in un agente
cloud/headless.

## Criterio di uscita

Testato a mano su almeno due configurazioni diverse (es. laptop singolo
schermo + laptop con monitor esterno a risoluzione diversa), la pill si
posiziona in modo sensato in entrambe. Annotare qui il risultato quando
fatto.
