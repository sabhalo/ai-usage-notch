# Step 5.5 — Nota su macOSPrivateApi

**Fase:** 5 — Porting macOS
**Stato:** GIÀ FATTO in Fase 0 — è solo una nota, non un'azione
**Dipende da:** —

## Task originale

Nota: `transparent: true` su macOS richiede `macOSPrivateApi`, già attivo
in config — questo preclude la distribuzione su Mac App Store. Per uso
personale è irrilevante, va solo saputo.

## Cosa è già stato fatto (Fase 0)

Durante la Fase 0, `cargo test`/`cargo clippy` fallivano perché la feature
`macos-private-api` **non** era abilitata in [Cargo.toml](../Cargo.toml)
nonostante `tauri.conf.json` la richiedesse per `transparent: true` — build
error bloccante, non solo una nota teorica. Corretto aggiungendo
`features = ["macos-private-api"]` alla dipendenza `tauri`.

## Da tenere a mente (non un'azione, solo memoria)

Questo esclude la distribuzione via Mac App Store. Per uso personale (unico
uso previsto, vedi piano generale) è irrilevante — non serve fare nulla qui,
solo non dimenticarlo se in futuro si valutasse l'App Store.
