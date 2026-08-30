# Step 4.3 — Soglie di allerta e notifica

**Fase:** 4 — Interazione e rifinitura UI
**Stato:** DA FARE
**Dipende da:** Fase 3 chiusa

## Task originale

Pulsazione dell'anello sopra una soglia configurabile (default 80%), con
notifica di sistema al superamento. Una sola notifica per finestra, non
ripetuta a ogni poll.

## Contesto

"Una sola notifica per finestra" richiede uno stato in memoria (o su disco,
per sopravvivere a un restart) del tipo "già notificato per questo ciclo di
reset" — altrimenti ogni poll sopra soglia rinotifica. Il reset della
finestra (5h/7g per Claude e Codex, mensile per Copilot) è il momento
naturale per far ripartire il flag. Verificare la Notification API di
Tauri v2 (permessi macOS da richiedere all'utente al primo uso).

## Criterio di uscita

Superata la soglia una volta, arriva una notifica di sistema; restando
sopra soglia nei poll successivi, non arrivano altre notifiche fino al
prossimo reset della finestra.
