# AI Usage Notch — piano a step

Ogni file `step-X.Y.md` è
autosufficiente — apri una sessione nuova, passa il file, elabora.

Le fasi sono **sequenziali** (non iniziare la Fase N+1 prima che la Fase N sia
chiusa). Dentro una fase i task sono liberi.

## Stato

- [x] **Fase 0 — Verifica endpoint** — CHIUSA. Tre fixture reali in
  `fixtures/`, parser allineati in `src/main.rs`, `docs/endpoints.md`
  aggiornato. Vedi `step-0.1.md`..`step-0.4.md` per il dettaglio di cosa è
  stato trovato (in particolare: bug della voce Keychain duplicata per
  Claude, shape reali diverse dalle ipotesi per Claude e Codex).
- [ ] **Fase 1 — Refactor architetturale** (`step-1.1.md`..`step-1.5.md`)
- [ ] **Fase 2 — Provider Copilot** (`step-2.1.md`..`step-2.5.md`) — nota:
  il backend è già in gran parte scritto dentro `main.rs` durante la Fase 0
  (bonus, avevamo i dati reali sotto mano). Questi step lo migrano
  nell'architettura a trait e completano la UI.
- [ ] **Fase 3 — Robustezza runtime** (`step-3.1.md`..`step-3.7.md`)
- [ ] **Fase 4 — Interazione e rifinitura UI** (`step-4.1.md`..`step-4.7.md`)
- [ ] **Fase 5 — Porting macOS** (`step-5.1.md`..`step-5.5.md`) — nota:
  5.1 e 5.5 in parte già coperti in Fase 0/1, lo step lo dice esplicitamente.
- [ ] **Fase 6 — Distribuzione** (`step-6.1.md`..`step-6.3.md`)

## Ambiente verificato (Fase 0)

- Rust toolchain installato via `rustup` (Homebrew, keg-only): prima di
  `cargo` su questo Mac serve
  `export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"`
  (non è in `.zshrc`, è stato lasciato solo per-sessione di proposito).
- `cargo test` e `cargo clippy --all-targets -- -D warnings` sono verdi allo
  stato attuale (4 test, tutti su fixture offline, nessuna rete).
- Claude su macOS legge il token da Keychain (voce `Claude Code-credentials`,
  campo `claudeAiOauth.accessToken`). Se in futuro torna a fallire, controllare
  prima di tutto se esistono **voci Keychain duplicate omonime** — è già
  successo una volta ed era quello.
