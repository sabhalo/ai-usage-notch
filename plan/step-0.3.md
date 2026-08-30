# Step 0.3 — Allineare i parser alle fixture reali

**Fase:** 0 — Verifica degli endpoint (bloccante)
**Stato:** COMPLETATO

## Task originale

Allineare i parser in `src/main.rs` alle fixture reali. Per Claude e Codex è
probabile che i nomi dei campi attualmente ipotizzati vadano corretti.

## Cosa è stato fatto

Tutti e tre i parser sono stati estratti come funzioni pure
(`parse_claude_usage`, `parse_codex_usage`, `parse_copilot_usage` in
[src/main.rs](../src/main.rs)) che prendono un `&serde_json::Value` e non
toccano la rete — testabili offline con le fixture. `cargo test` copre tutti
e tre più un test dedicato sul parser ISO 8601 scritto a mano per Claude.

Correzioni rispetto alle ipotesi originali:

- **Codex**: `rate_limits.primary/secondary` era sbagliato → reale
  `rate_limit.primary_window`/`secondary_window` (singolare), campo
  `reset_after_seconds` non `resets_in_seconds`.
- **Claude**: chiavi top-level giuste per caso (`five_hour`/`seven_day`), ma
  il reset è una data ISO 8601 assoluta (`resets_at`) non secondi relativi
  (`resets_in_seconds`) — serviva un parser di date. Scritto a mano
  (algoritmo civile di Hinnant), nessuna dipendenza nuova aggiunta.
- **Copilot**: shape quasi identica a quella ipotizzata nel piano originale,
  nessuna sorpresa strutturale grossa.

Bonus non pianificato qui ma emerso naturalmente: `get_copilot_usage` è già
stato scritto come comando Tauri funzionante in `main.rs` (non solo il
parser), perché i dati reali erano già disponibili. La Fase 2 lo migrerà
nell'architettura a trait invece di scriverlo da zero.

Nota per la Fase 1.2: `get_claude_usage` ora legge il token da Keychain su
macOS (prima leggeva solo da file, quindi non avrebbe mai funzionato su
Mac) — pezzo di credenziali multi-piattaforma anticipato qui, da spostare in
`src/credentials.rs` quando si fa il refactor.
