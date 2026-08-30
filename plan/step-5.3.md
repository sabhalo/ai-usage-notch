# Step 5.3 — Posizionamento accanto alla notch fisica

**Fase:** 5 — Porting macOS
**Stato:** DA FARE — richiede hardware con notch per verifica reale
**Dipende da:** Fase 4 chiusa

## Task originale

Posizionamento accanto alla notch fisica invece che al centro assoluto,
quando presente. `NSScreen.safeAreaInsets` è l'informazione giusta; su Mac
senza notch il fallback resta il centro.

## Contesto

`NSScreen.safeAreaInsets` è un'API Cocoa/AppKit — Tauri non la espone
direttamente, serve probabilmente codice `objc2`/`cocoa` a basso livello
(il progetto ha già `objc2-*` come dipendenze transitive di `tao`/`wry`,
verificare se sono usabili direttamente o vanno aggiunte esplicitamente).
Il fallback al centro (comportamento attuale, `main.rs` `setup()`) va
mantenuto per i Mac senza notch, non sostituito.

## Criterio di uscita

Testato su un Mac con notch fisica reale (MacBook Pro 14"/16" 2021+): la
pill si posiziona accanto alla notch, non sopra o al centro. Su un Mac
senza notch, resta il comportamento attuale (centro).
