# AI Usage Notch

Widget flottante sempre visibile in cima allo schermo che mostra il consumo
di Claude e Codex, letto dalle credenziali locali di `claude` (Claude Code CLI)
e `codex` (Codex CLI) — nessuna API key da configurare a mano.

Costruito con **Tauri v2** (Rust + webview di sistema, niente Node/npm richiesto
per il frontend, che è HTML/CSS/JS statico). Pensato per essere portato su
macOS in un secondo momento — vedi in fondo.

## Prerequisiti (Windows)

1. [Rust](https://rustup.rs) (`rustup-init.exe`, poi riavvia il terminale)
2. [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   con il carico di lavoro "Sviluppo di applicazioni desktop con C++"
   (richiesto dal linker MSVC — se hai già Visual Studio con quel workload, salta)
3. [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
   — su Windows 10/11 recenti è già preinstallato
4. Tauri CLI: `cargo install tauri-cli --version "^2"`
5. Claude Code CLI già loggato (`claude login`) e/o Codex CLI già loggato
   (`codex login`) — il widget legge i loro file di credenziali locali

## Avvio in sviluppo

```powershell
cd ai-usage-notch
cargo tauri dev
```

La prima compilazione scarica ed è più lenta (qualche minuto); le successive
sono incrementali.

## Build eseguibile

```powershell
cargo tauri build
```

Produce un installer `.exe` (NSIS) in `target/release/bundle/nsis/`.

## ⚠️ La parte da verificare al primo avvio

Claude e Codex non pubblicano un'API ufficiale per leggere la percentuale di
utilizzo del proprio abbonamento — questo widget usa gli stessi due endpoint
non documentati che usano internamente le rispettive CLI (`/usage` in Claude
Code, `/status` in Codex):

- Claude: `GET https://api.anthropic.com/api/oauth/usage`, autenticato col
  token OAuth che `claude login` salva in `%USERPROFILE%\.claude\.credentials.json`
- Codex: `GET https://chatgpt.com/backend-api/wham/usage`, autenticato con
  `access_token` + `account_id` da `%USERPROFILE%\.codex\auth.json`

Il **nome dei campi nella risposta JSON** (`five_hour`, `seven_day`,
`utilization`, `resets_in_seconds` per Claude; `rate_limits.primary/secondary`,
`used_percent` per Codex) è la mia migliore stima in base a tool community
equivalenti, ma non l'ho potuto verificare contro una risposta reale. Se al
primo avvio i pallini restano su "—" o "!":

1. Lancia `cargo tauri dev` (build di debug: stampa il JSON grezzo su stderr)
2. Guarda la console per le righe `[claude usage raw]` / `[codex usage raw]`
3. Aggiorna i nomi dei campi in `src/main.rs` (funzioni `get_claude_usage` /
   `get_codex_usage`) di conseguenza — sono isolati in poche righe

Questi endpoint possono anche cambiare in futuro senza preavviso: se smettono
di funzionare del tutto, lo stesso approccio (leggere il token locale +
richiamare l'endpoint di stato) resta valido, va solo riverificato.

I token restano sempre e solo in locale (letti dal filesystem, mai loggati
salvo la stampa di debug sopra, mai inviati altrove che ai due endpoint
ufficiali di Anthropic/OpenAI).

## Struttura

```
src/main.rs          logica Rust: lettura credenziali + chiamate HTTP
dist/                frontend statico (nessuna build step)
tauri.conf.json       finestra trasparente, senza bordi, sempre in primo piano
capabilities/          permessi minimi (resize/posizione finestra)
icons/                 icona placeholder generica — sostituiscila quando vuoi
```

## Verso il porting su macOS

L'app è già pensata cross-platform (Rust + Tauri girano nativamente anche su
macOS). Per il porting:

- aggiungi `"app"` e `"dmg"` a `bundle.targets` in `tauri.conf.json`
- genera l'icona `.icns` mancante — con Tauri CLI installato:
  `cargo tauri icon icons/icon.png` rigenera l'intero set per tutte le piattaforme
- il posizionamento "in cima allo schermo" nel `setup()` di `main.rs` va bene
  così com'è (usa le API cross-platform di Tauri), ma su macOS con notch fisico
  potresti voler agganciare il widget accanto alla notch reale invece che al
  centro — è un piccolo aggiustamento di `x`/`y`
- `~/.claude/.credentials.json` e `~/.codex/auth.json` sono gli stessi path
  relativi su macOS (cambia solo la home directory, già gestita da `dirs::home_dir()`)
