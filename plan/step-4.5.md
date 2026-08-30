# Step 4.5 — Sostituire l'icona placeholder

**Fase:** 4 — Interazione e rifinitura UI
**Stato:** DA FARE
**Dipende da:** Fase 3 chiusa

## Task originale

Sostituire l'icona placeholder. Nota: **non** usare i logo di Anthropic,
OpenAI o GitHub — sono marchi registrati. Usare glifi neutri o iniziali.

## Contesto

Vincolo legale esplicito dal piano originale, non derogabile: nessun logo
di terze parti, nemmeno stilizzato/ispirato in modo riconoscibile. Iniziali
o glifi geometrici neutri (es. lettera "C" per Claude, "X" per Codex, cerchio
per Copilot, o uno stile unificato astratto) sono la via sicura.

Le icone attuali sono in [icons/](../icons/) (`icon.png`, `.ico`,
`128x128.png`, `128x128@2x.png`, `32x32.png`) — sostituire i sorgenti e
rigenerare il set con `cargo tauri icon` (vedi anche step-5.2.md, che fa la
stessa cosa per macOS/`.icns` — coordinare per non rigenerare due volte).

## Criterio di uscita

Nessun asset nella cartella `icons/` assomiglia a un logo registrato di
Anthropic/OpenAI/GitHub.
