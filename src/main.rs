// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod credentials;
mod notch;
mod providers;
mod settings;
mod tray;

use providers::{
    ClaudeProvider, CodexProvider, CopilotProvider, FetchError, UsageProvider, UsageReport,
    UsageResult,
};
use tauri_plugin_autostart::MacosLauncher;

/// Ultimo dato noto su disco, per dipingere subito la pill all'avvio invece
/// di "—" mentre parte la prima fetch di rete (vedi plan/step-3.1.md).
#[tauri::command]
fn get_cached_usage() -> Option<cache::CachedUsage> {
    cache::load()
}

#[tauri::command]
fn get_settings() -> settings::Settings {
    settings::load()
}

fn sync_pill_visibility(app: &tauri::AppHandle, show_pill: bool) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        let _ = if show_pill {
            window.show()
        } else {
            window.hide()
        };
    }
}

#[tauri::command]
fn save_settings(settings: settings::Settings, app: tauri::AppHandle) {
    settings::save(&settings);
    sync_pill_visibility(&app, settings.show_pill);
    let reports = cache::load()
        .map(|cached| cached.reports)
        .unwrap_or_default();
    if let Err(error) = tray::sync(&app, &settings, &reports) {
        eprintln!("[tray] impossibile aggiornare le icone: {error}");
    }
}

/// Chiamato dal frontend quando l'utente finisce di trascinare la pill
/// (plan/step-4.1.md). Coordinate fisiche, le stesse usate per riposizionare
/// la finestra all'avvio in `setup()`.
#[tauri::command]
fn save_window_position(x: i32, y: i32) {
    let mut s = settings::load();
    s.window_position = Some((x, y));
    settings::save(&s);
}

/// Centra la finestra sul bordo superiore del monitor primario. Estratta per
/// essere riusata sia dal ramo di fallback in `setup()` (prima che il
/// frontend misuri il DOM) sia dal comando `center_if_unpositioned`.
fn center_on_primary_top(window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let screen_size = monitor.size();
        let scale = monitor.scale_factor();
        if let Ok(win_size) = window.outer_size() {
            let x = (screen_size.width as f64 / scale - win_size.width as f64 / scale) / 2.0;
            let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(
                x, 0.0,
            )));
        }
    }
}

/// Ricentra la Pill dopo il primo `applyLayout()` del frontend, ma solo se
/// l'utente non l'ha mai trascinata: a quel punto la larghezza della finestra
/// riflette la scala scelta (`pill_scale`), cosa che il centraggio nel
/// `setup()` — fatto sui 330x56 dichiarati — non può sapere. Con scale grandi
/// senza questo la Pill nascerebbe vistosamente scentrata.
#[tauri::command]
fn center_if_unpositioned(window: tauri::WebviewWindow) {
    if settings::load().window_position.is_none() {
        center_on_primary_top(&window);
    }
}

/// Interroga i provider in parallelo invece di tre round-trip separati dal
/// frontend (vedi plan/step-1.4.md). `skip` sono gli id dei provider che il
/// frontend ha già messo in backoff (plan/step-3.2.md): non vengono
/// ricontattati, e restano fuori dalla risposta — il chiamante conserva il
/// loro ultimo stato noto lato UI. La cache su disco viene comunque
/// aggiornata con l'unione tra i risultati freschi e quelli già in cache
/// per i provider saltati, così non perde mai dati per un provider
/// temporaneamente in backoff.
#[tauri::command]
async fn get_all_usage(skip: Vec<String>, app: tauri::AppHandle) -> Vec<UsageReport> {
    let want = |id: &str| !skip.iter().any(|s| s == id);

    let (claude, codex, copilot) = tokio::join!(
        maybe_fetch(want("claude"), ClaudeProvider),
        maybe_fetch(want("codex"), CodexProvider),
        maybe_fetch(want("copilot"), CopilotProvider),
    );

    let fresh: Vec<UsageReport> = [claude, codex, copilot].into_iter().flatten().collect();

    let mut merged = cache::load().map(|c| c.reports).unwrap_or_default();
    for report in &fresh {
        merged.retain(|r| r.provider != report.provider);
        merged.push(report.clone());
    }
    cache::save(&merged);
    if let Err(error) = tray::sync(&app, &settings::load(), &merged) {
        eprintln!("[tray] impossibile aggiornare i consumi: {error}");
    }

    fresh
}

async fn maybe_fetch<P: UsageProvider>(want: bool, provider: P) -> Option<UsageReport> {
    if !want {
        return None;
    }
    Some(to_report(provider.id(), provider.fetch().await))
}

fn to_report(id: &str, result: Result<UsageResult, FetchError>) -> UsageReport {
    match result {
        Ok(r) => UsageReport::ok(r),
        Err(e) => UsageReport::err(id, e),
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .on_menu_event(|app, event| {
            use tauri::Manager;
            if event.id().as_ref() == "open-settings" {
                if let Some(window) = app.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .setup(|app| {
            use tauri::Manager;
            let settings = settings::load();
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(debug_assertions)]
                window.open_devtools();
                let saved_position = settings.window_position;
                if let Some((x, y)) = saved_position {
                    let _ = window.set_position(tauri::Position::Physical(
                        tauri::PhysicalPosition::new(x, y),
                    ));
                } else if let Some(notch_x) = notch::left_edge_x() {
                    #[cfg(debug_assertions)]
                    eprintln!("[notch] Mac con notch fisica rilevata, x={notch_x}");
                    let _ = window.set_position(tauri::Position::Logical(
                        tauri::LogicalPosition::new(notch_x, 0.0),
                    ));
                } else {
                    center_on_primary_top(&window);
                }
                sync_pill_visibility(app.handle(), settings.show_pill);
            }

            // Il tasto rosso di chiusura, senza questo intercettore, distrugge
            // la finestra: da quel momento `getByLabel("settings")` in JS
            // torna sempre None e il gear smette di funzionare per il resto
            // della sessione (issue #10). La nascondiamo invece di chiuderla,
            // così resta riutilizzabile da `openSettingsWindow()`.
            if let Some(settings_window) = app.get_webview_window("settings") {
                use tauri::Emitter;
                let hide_target = settings_window.clone();
                let app_handle = app.handle().clone();
                settings_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = hide_target.hide();
                        // Chiudere le Impostazioni senza salvare fa rileggere
                        // alla Pill il valore persistito, annullando
                        // l'anteprima live della scala. Innocuo se l'utente
                        // aveva già salvato.
                        let _ = app_handle.emit("settings-changed", ());
                    }
                });
            }

            let reports = cache::load()
                .map(|cached| cached.reports)
                .unwrap_or_default();
            tray::sync(app.handle(), &settings, &reports)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_usage,
            get_cached_usage,
            get_settings,
            save_settings,
            save_window_position,
            center_if_unpositioned
        ])
        .run(tauri::generate_context!())
        .expect("errore durante l'avvio dell'applicazione Tauri");
}
