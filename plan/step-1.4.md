# Step 1.4 — comando Tauri get_all_usage()

**Fase:** 1 — Refactor architetturale
**Stato:** DA FARE
**Dipende da:** step-1.1.md, step-1.3.md

## Task originale

Un unico comando Tauri `get_all_usage()` che interroga i provider in
parallelo (`futures::join!`) e ritorna un vettore — evita tre round-trip dal
frontend.

## Contesto

Oggi (Fase 0) il frontend ([dist/main.js](../dist/main.js)) chiama
presumibilmente tre comandi separati (`get_claude_usage`, `get_codex_usage`,
`get_copilot_usage`, tutti e tre già registrati in
`invoke_handler![...]` in `main.rs`). Verificare come li chiama oggi prima di
cambiare la firma — il frontend andrà aggiornato di conseguenza in questo
stesso step (non lasciarlo rotto a metà).

`futures` non è ancora una dipendenza in [Cargo.toml](../Cargo.toml) — va
aggiunta (oppure usare `tokio::join!` se tauri porta già tokio come
dipendenza transitiva, verificare quale evita di aggiungere un crate in più).

## Criterio di uscita

Un solo comando Tauri, il frontend fa una sola chiamata invece di tre,
`cargo build` passa, l'app mostra ancora tutti e tre gli anelli (verifica
visiva con `cargo tauri dev`, non solo build).
