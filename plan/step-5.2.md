# Step 5.2 — Generare icone e bundle app/dmg

**Fase:** 5 — Porting macOS
**Stato:** DA FARE
**Dipende da:** step-4.5.md (icona sostituita — non ha senso generare
`.icns` dal placeholder se poi va rifatto)

## Task originale

`cargo tauri icon icons/icon.png` per generare il set completo, incluso
`.icns`; aggiungere `"app"` e `"dmg"` a `bundle.targets`.

## Contesto

Fare questo step **dopo** step-4.5.md (icona reale, non placeholder) per
non rigenerare il set icone due volte. `bundle.targets` va aggiunto in
[tauri.conf.json](../tauri.conf.json).

## Criterio di uscita

`cargo tauri build` produce un `.app` e un `.dmg` con l'icona corretta,
verificato aprendo il `.dmg` risultante.
