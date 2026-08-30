# Step 5.1 — Lettura token Claude dal Keychain

**Fase:** 5 — Porting macOS
**Stato:** GIÀ FATTO in Fase 0/1 — verificare solo il nome servizio
**Dipende da:** Fase 4 chiusa

## Task originale

Lettura del token Claude dal Keychain (`security find-generic-password -s
"Claude Code-credentials" -w`). Il nome esatto del servizio va confermato
in Keychain Access.

## Cosa è già stato fatto (Fase 0/1)

Il nome servizio **è già confermato**: `Claude Code-credentials`, campo
`claudeAiOauth.accessToken` nel JSON restituito. Implementato e verificato
con token reale su questa macchina (vedi `docs/endpoints.md`,
step-1.2.md).

## Attenzione — problema reale già incontrato, non ipotetico

Possono esistere **più voci Keychain con lo stesso nome servizio** (è
successo qui: una vecchia con solo token MCP, una nuova con il token di
sessione vero dopo un `claude login` pulito).
`security find-generic-password -s "Claude Code-credentials" -w` senza
`-a <account>` ne prende una a caso — può restituire la voce sbagliata
silenziosamente (nessun errore, solo JSON senza `claudeAiOauth`). Se questo
step riappare come "da verificare", la causa più probabile è questa,
controllare prima da Keychain Access se ci sono duplicati prima di
sospettare un cambio di endpoint.

## Criterio di uscita

Già soddisfatto — nessuna azione necessaria a meno che il problema delle
voci duplicate non si ripresenti su un'altra macchina/utente.
