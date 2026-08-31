# AI Usage Notch

Widget flottante sempre visibile in cima allo schermo che mostra il consumo
di **Claude**, **Codex** e **GitHub Copilot**, letto dalle credenziali
locali già presenti sulla macchina (Keychain per Claude su macOS, file di
credenziali per Codex, `gh auth token` per Copilot) — nessuna API key da
configurare a mano.

Costruito con **Tauri v2** (Rust + webview di sistema, niente Node/npm
richiesto per il frontend, che è HTML/CSS/JS statico).

## Prerequisiti

1. [Rust](https://rustup.rs)
2. Tauri CLI: `cargo install tauri-cli --version "^2"`
3. Solo Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   (workload "Sviluppo di applicazioni desktop con C++", richiesto dal
   linker MSVC) e [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
   (già preinstallato su Windows 10/11 recenti)
4. Almeno uno tra: Claude Code CLI loggato (`claude login`), Codex CLI
   loggato (`codex login`), GitHub CLI loggato (`gh auth login`) — il widget
   legge le credenziali che trova, un provider non configurato appare grigio
   invece di rompere l'interfaccia (vedi "Comportamento" sotto)

## Avvio in sviluppo

```sh
cd ai-usage-notch
cargo tauri dev
```

## Build eseguibile

```sh
cargo tauri build
```

- Windows: installer `.exe` (NSIS) in `target/release/bundle/nsis/`
- macOS: `.app` in `target/release/bundle/macos/` e `.dmg` in `target/release/bundle/dmg/`

## Comportamento

- **Click sinistro** su un anello: apre/chiude il dettaglio di quel provider
  (finestre di utilizzo, reset, eventuale badge overage).
- **Click destro** sulla pill: forza un refresh immediato di tutti i
  provider, bypassando il backoff.
- **Doppio click** sullo sfondo della pill (non su un anello): alterna
  modalità estesa/compatta (solo anelli, percentuale all'hover).
- **Trascinamento**: la pill si sposta e ricorda la posizione tra un riavvio
  e l'altro.
- **Icona ingranaggio**: apre la finestra impostazioni (provider attivi,
  intervallo di refresh, soglia di allerta, avvio al login).
- Un provider senza credenziali configurate appare **grigio** con un
  tooltip esplicativo invece di un anello rotto; un provider che ha
  funzionato e poi ha iniziato a fallire mostra invece un allarme rosso.
- Sopra la soglia di allerta (80% di default) l'anello pulsa e arriva una
  notifica di sistema, una sola volta per ciclo (fino al prossimo reset
  della finestra).

## Struttura

```
src/main.rs           comandi Tauri, setup finestra, posizionamento notch
src/providers/         un modulo per provider (claude/codex/copilot), trait UsageProvider comune
src/credentials.rs      risoluzione token per piattaforma (Keychain/file/gh)
src/cache.rs            ultimo dato valido su disco, mostrato subito all'avvio
src/settings.rs         provider attivi, intervallo refresh, soglia, posizione finestra
src/notch.rs            posizionamento accanto alla notch fisica su macOS (via objc2-app-kit)
dist/                   frontend statico (pill + pannello dettagli + finestra impostazioni)
tauri.conf.json         due finestre: pill trasparente sempre in primo piano + impostazioni
capabilities/           permessi minimi (resize/posizione finestra, notifiche, autostart)
docs/endpoints.md        shape reali degli endpoint non ufficiali, verificate con dati veri
fixtures/                risposte reali salvate, usate dai test (nessuna rete nei test)
scripts/probe.*          script per riverificare a mano gli endpoint se smettono di funzionare
```

## Se un endpoint cambia

Claude, Codex e Copilot non pubblicano API ufficiali per la percentuale di
utilizzo — questo widget usa gli stessi endpoint non documentati delle
rispettive CLI/estensioni. Se un provider inizia a mostrare l'errore
"risposta ricevuta ma nessun campo riconosciuto":

1. Rilancia `scripts/probe.sh` (o `.ps1` su Windows) per catturare una
   risposta reale aggiornata
2. Confrontala con `docs/endpoints.md`
3. Aggiorna il parsing nel modulo del provider interessato sotto
   `src/providers/`

I token restano sempre e solo in locale; in build di debug il body grezzo
della risposta viene stampato su stderr per diagnosticare, mai altrove.

## Distribuzione — installer Windows non firmato

L'installer NSIS prodotto da `cargo tauri build` **non è firmato digitalmente**:
Windows SmartScreen mostrerà un avviso ("Windows ha protetto il PC") al primo
avvio. La firma del codice richiede un certificato a pagamento (rinnovo
annuale) che non ha senso per un progetto a uso personale — l'avviso va
accettato consapevolmente ("Ulteriori informazioni" → "Esegui comunque"),
non è un segnale di un problema nel software. Da rivalutare solo se questo
progetto smettesse di essere per uso personale.

## Nota — Mac App Store

`transparent: true` su macOS richiede la feature `macos-private-api`, già
abilitata: questo esclude la distribuzione via Mac App Store. Irrilevante
per uso personale.
