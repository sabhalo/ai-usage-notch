# Step 3.4 — Refresh manuale

**Fase:** 3 — Robustezza e comportamento runtime
**Stato:** DA FARE
**Dipende da:** Fase 2 chiusa

## Task originale

Refresh manuale: click destro sulla pill, o scorciatoia globale.

## Contesto

Nessuna decisione presa nel piano tra click destro e scorciatoia globale —
implementare quello più semplice con quello che offre Tauri v2 senza
plugin aggiuntivi (menu contestuale nativo sulla finestra) prima di
introdurre `tauri-plugin-global-shortcut` se non strettamente necessario.

## Criterio di uscita

Un'azione manuale (click destro o scorciatoia) forza un refresh immediato
di tutti i provider, bypassando il backoff dello step-3.2.md.
