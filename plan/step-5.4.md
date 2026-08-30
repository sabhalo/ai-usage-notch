# Step 5.4 — visibleOnAllWorkspaces e window level in fullscreen

**Fase:** 5 — Porting macOS
**Stato:** DA FARE
**Dipende da:** Fase 4 chiusa

## Task originale

Valutare `visibleOnAllWorkspaces` e il window level: su macOS un
`alwaysOnTop` normale sparisce in fullscreen.

## Contesto

Si sovrappone parzialmente con lo step-3.7.md (alwaysOnTop non ruba
focus) — leggere quello step prima di implementare, per risolvere insieme
le due questioni invece di toccare lo stesso codice due volte in fasi
diverse.

## Criterio di uscita

Con un'altra app in fullscreen (es. browser), la pill resta visibile,
verificato a mano.
