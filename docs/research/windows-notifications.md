# Notifiche toast su Windows — dev vs installer NSIS

Ricerca per l'issue [#17](https://github.com/sabhalo/ai-usage-notch/issues/17). Data: 2026-09-01.
Nessuna modifica al codice applicativo in questo ticket.

Versioni su cui è verificata l'analisi (da `Cargo.lock` di questo repo):
`tauri-plugin-notification` **2.3.3**, `tauri-winrt-notification` **0.7.3**.
Ultima versione pubblicata del plugin al 2026-09-01: **2.4.0** — il changelog di
2.4.0 contiene solo fix Android/mobile, quindi nulla di quanto segue cambia
aggiornando ([CHANGELOG.md](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/CHANGELOG.md)).

---

## Risposte in breve

| Domanda | Risposta |
| --- | --- |
| Le toast arrivano da `cargo tauri dev`? | **Sì**, ma marchiate "Windows PowerShell" (nome e icona), non con nome/icona dell'app. Il meccanismo nativo funziona. |
| Le toast arrivano dall'installer NSIS? | **Sì**, con nome e icona corretti. L'installer Tauri crea lo shortcut nel menu Start e ci scrive sopra l'AppUserModelID — automaticamente, senza configurazione. |
| Serve un permesso esplicito come su macOS? | **No** lato sistema. Windows non ha un prompt di permesso per le toast dei desktop app classici; il backend Rust del plugin ritorna sempre `Granted`. |
| Allora è un falso bug? | **No — c'è un bug vero, ma non è quello sospettato.** Vedi § "Il vero blocco". |

**Il sospetto dell'issue #17 (AUMID mancante in dev → fallimento silenzioso) è
infondato**: il plugin ha un fallback esplicito proprio per questo caso. Il
motivo per cui le notifiche di questa app *non* partiranno su Windows è un
altro, ed è nel gate di permesso lato frontend.

---

## 1. Come il plugin decide l'AppUserModelID

`plugins/notification/src/desktop.rs`, funzione `show()`
([sorgente, branch v2](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/src/desktop.rs)):

```rust
#[cfg(windows)]
{
    let exe = tauri::utils::platform::current_exe()?;
    let exe_dir = exe.parent().expect("failed to get exe directory");
    let curr_dir = exe_dir.display().to_string();
    // set the notification's System.AppUserModel.ID only when running the installed app
    if !(curr_dir.ends_with(format!("{SEP}target{SEP}debug").as_str())
        || curr_dir.ends_with(format!("{SEP}target{SEP}release").as_str()))
    {
        notification.app_id(&self.identifier);
    }
}
```

Quindi:

- **exe in `…\target\debug` o `…\target\release`** (cioè `cargo tauri dev` e
  `cargo build`): `app_id` **non** viene impostato.
- **exe altrove** (app installata in `C:\Program Files\…`): `app_id` =
  `self.identifier`, che è `config().identifier`, cioè `dev.davide.aiusagenotch`
  del nostro `tauri.conf.json`.

Quando `app_id` non è impostato, `notify-rust` usa la costante di fallback
([`notify-rust/src/windows.rs`](https://github.com/hoodie/notify-rust/blob/main/src/windows.rs)):

```rust
let app_id = notification.app_id.as_deref().unwrap_or(Toast::POWERSHELL_APP_ID);
```

e `POWERSHELL_APP_ID` è
([`tauri-winrt-notification/src/lib.rs`](https://github.com/tauri-apps/winrt-notification/blob/dev/src/lib.rs)):

```rust
/// This can be used if you do not have a AppUserModelID.
///
/// However, the toast will erroneously report its origin as powershell.
pub const POWERSHELL_APP_ID: &'static str = "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\
                                             \\WindowsPowerShell\\v1.0\\powershell.exe";
```

Questo AUMID è registrato da Windows stesso. Verificato sulla macchina di
destinazione (Windows 11 Home 10.0.26200) con `Get-StartApps`:

```
Windows PowerShell    {1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe
```

Lo shortcut corrispondente esiste in
`%APPDATA%\Microsoft\Windows\Start Menu\Programs\Windows PowerShell\Windows PowerShell.lnk`.
Quindi **il fallback in dev è valido e la toast viene consegnata**, solo con
branding sbagliato.

Lo conferma la documentazione ufficiale del plugin, che alla voce Windows dice:
*"Only works for installed apps. Shows powershell name & icon in development."*
([v2.tauri.app/plugin/notification](https://v2.tauri.app/plugin/notification/)).

### Quando il fallback si rompe

L'euristica è basata sul *percorso*, non su `tauri::is_dev()`. Si rompe se
l'eseguibile non sta in `…\target\debug`:

- `CARGO_TARGET_DIR` o `[build] target-dir` puntati altrove — è esattamente il
  caso descritto in
  [plugins-workspace PR #1502](https://github.com/tauri-apps/plugins-workspace/pull/1502)
  ("The old logic failed because exe_dir was `A:\_target\debug` on my system");
- build portable / exe copiato fuori dalla cartella —
  [tauri#11757](https://github.com/tauri-apps/tauri/issues/11757), dove il
  maintainer FabianLars risponde: *"The notification plugin only really works if
  the app was actually installed. In v1 it would show the powershell icon
  instead but in v2 it either fails silently or sometimes still shows the
  powershell icon."*

In quei casi `app_id` viene impostato a `dev.davide.aiusagenotch`, che senza
shortcut nel menu Start **non è un AUMID valido** → fallimento silenzioso.

**Per questo repo il rischio è nullo**: `Cargo.toml` è alla radice, non esiste
`.cargo/config.toml` né a livello progetto né utente, e `CARGO_TARGET_DIR` non è
impostata. `cargo tauri dev` produce `<repo>\target\debug\ai-usage-notch.exe` →
euristica soddisfatta.

---

## 2. Perché serve un AUMID registrato (fonte Microsoft)

La regola è di Windows, non di Tauri. Da
[*How to enable desktop toast notifications through an AppUserModelID*](https://learn.microsoft.com/en-us/windows/win32/shell/enable-desktop-toast-with-appusermodelid):

> "We strongly recommend that you do this in the Windows Installer rather than
> in your app's code. Without a valid shortcut installed in the Start screen or
> in **All Programs**, you cannot raise a toast notification from a desktop app."

E da [*Application User Model IDs (AppUserModelIDs)*](https://learn.microsoft.com/en-us/windows/win32/shell/appids),
sezione "Where to Assign an AppUserModelID", il primo posto in cui va scritto è:

> "In the System.AppUserModel.ID property of the application's shortcut file."

con la nota:

> "The System.AppUserModel.ID property should be applied to a shortcut when that
> shortcut is created."

Il maintainer Tauri amrbashir arriva alla stessa conclusione empiricamente in
[PR #1502](https://github.com/tauri-apps/plugins-workspace/pull/1502):

> "`AppUserModelId` can be set using `SetCurrentProcessExplicitAppUserModelID`
> but it doesn't take effect unless there is a shortcut in Start Menu (doesn't
> need to be pinned)"

Vincoli sul formato AUMID (stessa pagina Microsoft): massimo 128 caratteri, niente
spazi. `dev.davide.aiusagenotch` (23 caratteri, senza spazi) è conforme.

---

## 3. L'installer NSIS di Tauri fa tutto da solo — sì

Nel template ufficiale
[`crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi`](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi),
`Function CreateOrUpdateStartMenuShortcut`:

```nsis
!if "${STARTMENUFOLDER}" != ""
  CreateDirectory "$SMPROGRAMS\$AppStartMenuFolder"
  CreateShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
  !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
!else
  CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
  !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
!endif
```

La stessa macro è applicata anche allo shortcut sul desktop
(`CreateOrUpdateDesktopShortcut`). E la macro, in
[`nsis/utils.nsh`](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/utils.nsh),
scrive proprio `PKEY_AppUserModel_ID` con il valore `${BUNDLEID}`:

```nsis
!macro SetLnkAppUserModelId shortcut
  ...
  System::Call 'Oleaut32::SysAllocString(w "${BUNDLEID}") i.r3'
  System::Call '*${SYSSTRUCT_PROPERTYKEY}(${PKEY_AppUserModel_ID})p.r4'
  System::Call '*${SYSSTRUCT_PROPVARIANT}(${VT_BSTR},,&i4 $3)p.r5'
  ${IPropertyStore::SetValue} $2 '($4,$5)'
  ...
!macroend
```

`${BUNDLEID}` è `{{bundle_id}}`, cioè l'`identifier` di `tauri.conf.json`. È lo
**stesso identificatore** che `desktop.rs` passa a `notification.app_id(...)`.
I due combaciano: **niente da configurare, l'installer NSIS di Tauri è già a posto.**

### Due modi per rompere l'installazione

Nella stessa funzione, prima della creazione:

```nsis
${If} $WixMode = 0
  ${If} $UpdateMode = 1
  ${OrIf} $NoShortcutMode = 1
    Return
  ${EndIf}
${EndIf}
```

- installer lanciato con il flag **`/NS`** (no shortcut) → nessuno shortcut →
  nessun AUMID → **toast silenziosamente assenti**;
- in modalità update lo shortcut non viene ricreato (assunto già presente).

Per la prova end-to-end: installare normalmente, senza flag.

---

## 4. Permessi: Windows non ne ha

`desktop.rs` è esplicito — su tutti i desktop, non solo Windows:

```rust
pub fn request_permission(&self) -> crate::Result<PermissionState> {
    Ok(PermissionState::Granted)
}

pub fn permission_state(&self) -> crate::Result<PermissionState> {
    Ok(PermissionState::Granted)
}
```

Non esiste una richiesta di permesso da fare, né un prompt di sistema da
accettare come su macOS. `notification:default` in `capabilities/default.json`
(già presente) è l'unica autorizzazione necessaria, ed è quella di Tauri, non
di Windows.

---

## 5. Il vero blocco: `Notification.permission` è `"denied"` su Windows

Questo è il risultato che cambia le cose. **Non c'entra l'AUMID.**

`src/lib.rs` del plugin inietta uno script all'avvio del webview, sostituendo un
placeholder in base alla piattaforma:

```rust
.js_init_script(include_str!("init-iife.js").replace(
    "__TEMPLATE_windows__",
    if cfg!(windows) { "true" } else { "false" },
))
```

Lo script ([`guest-js/init.ts`](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/guest-js/init.ts))
contiene:

```ts
async function isPermissionGranted(): Promise<boolean> {
  if (window.Notification.permission !== 'default' || __TEMPLATE_windows__) {
    return await Promise.resolve(window.Notification.permission === 'granted')
  }
  return await invoke('plugin:notification|is_permission_granted')
}
...
void isPermissionGranted().then(function (response) {
  if (response === null) setNotificationPermission('default')
  else setNotificationPermission(response ? 'granted' : 'denied')
})
```

Su Windows il placeholder diventa il letterale `true`, quindi:

1. la condizione è **sempre** vera (`|| true`) e il ramo che interroga il backend
   è codice morto;
2. si valuta `window.Notification.permission === 'granted'` contro il valore
   iniziale, che a quel punto è ancora `'default'` → `false`;
3. `setNotificationPermission(false ? 'granted' : 'denied')` → **`'denied'`**.

Su macOS il placeholder è `false`, si interroga il backend, che ritorna
`Granted` → `'granted'`. **Ecco perché su macOS funziona e su Windows no.**

Questo è tracciato come
[plugins-workspace #3557](https://github.com/tauri-apps/plugins-workspace/issues/3557)
("Notification.permission is always \"denied\" on Windows because the startup
check short-circuits before querying the backend"), con misure via CDP su un
build reale Tauri v2 / plugin 2.3.3:

```
Notification.permission                => "denied"
raw backend: is_permission_granted     => true
raw backend: request_permission        => "granted"
await Notification.requestPermission() => "granted"
AFTER: Notification.permission         => "granted"
```

L'issue risulta chiusa (2026-08-28) ma **il codice sul branch `v2` è ancora
quello buggato** — verificato leggendo `guest-js/init.ts` e `src/init-iife.js`
il 2026-09-01 — e 2.4.0 non contiene alcun fix in merito.

### Impatto su questa app

`dist/main.js` usa la Web API `Notification`, che il plugin sovrascrive
(`window.Notification = function (title, options) { ... }` in `init.ts`), quindi
la chiamata *arriva* al plugin. Ma il gate è questo:

```js
if (typeof Notification === "undefined") return;
if (Notification.permission === "default") await Notification.requestPermission();
if (Notification.permission === "granted") {
  new Notification(...)
}
```

Su Windows `Notification.permission` vale `"denied"`, non `"default"` →
`requestPermission()` **non viene mai chiamata** → il secondo `if` è falso →
**nessuna notifica, mai, né in dev né dall'installer.** Nessun errore in
console: `sendThresholdNotification` non lancia, e `notifiedThisCycle[provider]`
viene comunque messo a `true` dal chiamante, quindi il ciclo non riprova.

### Correzione consigliata (fuori dallo scope di questo ticket)

Una riga in `dist/main.js`: sostituire il gate `=== "default"` con `!== "granted"`.

```js
if (Notification.permission !== "granted") await Notification.requestPermission();
```

`requestPermission()` fa il round-trip al backend, che ritorna `Granted`, e
`setNotificationPermission` corregge il valore in cache a `"granted"`. È lo
stesso workaround suggerito in #3557 ("Call `await Notification.requestPermission()`
on Windows before reading `Notification.permission`"). Su macOS il comportamento
non cambia: lì `permission` è già `"granted"` e la condizione non scatta.

In alternativa, dato che `withGlobalTauri: true` è già impostato in
`tauri.conf.json`, si può bypassare del tutto il gate chiamando
`window.__TAURI__.notification.sendNotification({ title, body })`.

Da fare dietro un ticket separato: qui non si tocca codice applicativo.

---

## 6. Focus assist / Do not disturb

Sopprime, sì, e in silenzio. Da
[*Notifications and Do Not Disturb in Windows*](https://support.microsoft.com/en-us/windows/notifications-and-do-not-disturb-in-windows-feeca47f-0baf-5680-16f0-8801db1a8466):

> "With *do not disturb* on, you will only receive banners for alarms,
> reminders, and apps of your choice."

Le altre notifiche finiscono direttamente nel centro notifiche. Si attiva da
sola: in una fascia oraria configurata, quando si duplica il display, durante un
gioco, con un'app a schermo intero, e per la prima ora dopo un aggiornamento di
Windows. **Durante la prova end-to-end conviene verificare che il DND sia
spento**, o cercare la toast nel centro notifiche invece che a schermo.

Il plugin non ha modo di accorgersene: in `desktop.rs` l'esito è scartato.

```rust
tauri::async_runtime::spawn(async move {
    let _ = notification.show();
});
```

Il `let _ =` in un task spawnato significa che **nessun errore nativo può mai
risalire fino a JS**. Qualunque fallimento — AUMID non valido, DND, toggle
globale spento — si presenta identico: silenzio.

### Precondizione trovata sulla macchina di destinazione

Lettura del registro il 2026-09-01, Windows 11 Home 10.0.26200:

```
HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\PushNotifications
    ToastEnabled = 0
```

È l'interruttore generale delle notifiche (Impostazioni → Sistema → Notifiche).
Con quel valore a `0` **nessuna app riceve toast**, installata o meno, e la
prova end-to-end fallirebbe anche a codice corretto. Va acceso prima di provare.
(Osservazione sulla macchina, non fonte primaria: la conferma sta in
Impostazioni → Sistema → Notifiche, il percorso indicato dalla pagina di
supporto Microsoft citata sopra.)

Nota di contorno: l'app non passa un `icon`, quindi il plugin chiama
`auto_icon()`. Su Windows `notify-rust` popola `icon`, ma il costruttore della
toast usa solo `path_to_image` — l'icona mostrata viene dallo shortcut associato
all'AUMID, non dalla notifica. Nessun rischio di fallimento da lì.

---

## Checklist per la prova end-to-end

1. Accendere le notifiche di sistema (`ToastEnabled` è a `0` su questa macchina).
2. Verificare che il Do Not Disturb sia spento.
3. Applicare il fix del gate di permesso in `dist/main.js` (ticket separato) —
   **senza, non arriva nulla su Windows**.
4. In `cargo tauri dev` aspettarsi una toast marchiata "Windows PowerShell":
   è corretto, non è un bug.
5. Per nome e icona giusti serve l'installer NSIS, installato senza `/NS`.
   Nessuna configurazione aggiuntiva richiesta.

## Fonti

Tutte primarie: sorgente e documentazione ufficiale, tracker upstream, docs Microsoft.

- [`tauri-plugin-notification` — `src/desktop.rs`](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/src/desktop.rs)
- [`tauri-plugin-notification` — `src/lib.rs`](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/src/lib.rs)
- [`tauri-plugin-notification` — `guest-js/init.ts`](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/guest-js/init.ts)
- [`tauri-plugin-notification` — CHANGELOG](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/CHANGELOG.md)
- [Docs Tauri v2 — Notification plugin](https://v2.tauri.app/plugin/notification/)
- [`notify-rust` — `src/windows.rs`](https://github.com/hoodie/notify-rust/blob/main/src/windows.rs)
- [`tauri-winrt-notification` — `src/lib.rs`](https://github.com/tauri-apps/winrt-notification/blob/dev/src/lib.rs)
- [`tauri-bundler` — `nsis/installer.nsi`](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi)
- [`tauri-bundler` — `nsis/utils.nsh`](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/utils.nsh)
- [plugins-workspace #3557 — `Notification.permission` sempre "denied" su Windows](https://github.com/tauri-apps/plugins-workspace/issues/3557)
- [plugins-workspace PR #1502 — is-installed logic](https://github.com/tauri-apps/plugins-workspace/pull/1502)
- [tauri #11757 — notifica silenziosa fuori dalla release folder](https://github.com/tauri-apps/tauri/issues/11757)
- [Microsoft — How to enable desktop toast notifications through an AppUserModelID](https://learn.microsoft.com/en-us/windows/win32/shell/enable-desktop-toast-with-appusermodelid)
- [Microsoft — Application User Model IDs (AppUserModelIDs)](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft — Notifications and Do Not Disturb in Windows](https://support.microsoft.com/en-us/windows/notifications-and-do-not-disturb-in-windows-feeca47f-0baf-5680-16f0-8801db1a8466)
