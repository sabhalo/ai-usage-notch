# Step 3.7 — alwaysOnTop non deve rubare focus

**Fase:** 3 — Robustezza e comportamento runtime
**Stato:** DA FARE — richiede test manuale
**Dipende da:** Fase 2 chiusa

## Task originale

Verificare che la finestra `alwaysOnTop` non copra elementi di sistema e
non rubi il focus. Su Windows valutare `WS_EX_NOACTIVATE` se la finestra
ruba il focus al click.

## Contesto

Su macOS l'equivalente concettuale di `WS_EX_NOACTIVATE` è impostare il
window level giusto e/o `ignore_cursor_events` sulle aree non interattive —
verificare l'API Tauri v2 disponibile invece di assumere che sia identica a
Windows. Questo step si sovrappone parzialmente con lo step-5.4.md (window
level su macOS in fullscreen) — leggere entrambi prima di implementare, per
non risolvere la stessa cosa due volte in due fasi diverse.

## Criterio di uscita

Cliccando sulla pill mentre un'altra app è in primo piano, quell'altra app
non perde il focus in modo indesiderato (verificato a mano, non
automatizzabile facilmente).
