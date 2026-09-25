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
    UsageResult, SUPPORTED_PROVIDER_IDS,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
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

// Il webview Hidden può sospendere i timer JS. In quello stato il backend
// continua ad aggiornare cache e menu tray usando la stessa fetch della Pill.
async fn poll_while_hidden(app: tauri::AppHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut next_at = HashMap::new();
    let mut failures = HashMap::<String, usize>::new();

    loop {
        ticker.tick().await;
        let settings = settings::load();
        if settings.show_pill {
            next_at.clear();
            failures.clear();
            continue;
        }

        let now = Instant::now();
        let skip = SUPPORTED_PROVIDER_IDS
            .iter()
            .filter(|&&provider| {
                settings.active_providers.get(provider) == Some(&false)
                    || next_at.get(provider).is_some_and(|&due| now < due)
            })
            .map(|provider| (*provider).to_string())
            .collect::<Vec<_>>();
        if skip.len() == SUPPORTED_PROVIDER_IDS.len() {
            continue;
        }

        for report in get_all_usage(skip, app.clone()).await {
            let count = failures.entry(report.provider.clone()).or_default();
            let delay = next_hidden_poll_delay(&report, count, settings.refresh_interval_s);
            next_at.insert(report.provider, Instant::now() + delay);
        }
    }
}

fn next_hidden_poll_delay(
    report: &UsageReport,
    failures: &mut usize,
    refresh_interval_s: u64,
) -> Duration {
    let seconds = if report.error.is_none() {
        *failures = 0;
        refresh_interval_s.max(30)
    } else if let Some(retry_after) = report.retry_after_s {
        retry_after
    } else {
        *failures += 1;
        [30, 300, 900][(*failures - 1).min(2)]
    };
    Duration::from_secs(seconds)
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
        .on_tray_icon_event(|app, event| {
            use tauri::{tray::TrayIconEvent, Emitter};
            let TrayIconEvent::Click {
                id,
                button,
                button_state,
                ..
            } = event
            else {
                return;
            };
            if !tray::should_refresh_on_click(id.as_ref(), button, button_state) {
                return;
            }

            let settings = settings::load();
            if settings.show_pill {
                if let Err(error) = app.emit("refresh-now", ()) {
                    eprintln!("[tray] impossibile richiedere il refresh: {error}");
                }
            } else {
                let skip = SUPPORTED_PROVIDER_IDS
                    .iter()
                    .filter(|&&provider| settings.active_providers.get(provider) == Some(&false))
                    .map(|provider| (*provider).to_string())
                    .collect();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    get_all_usage(skip, app).await;
                });
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
            tauri::async_runtime::spawn(poll_while_hidden(app.handle().clone()));

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

#[cfg(test)]
mod hidden_poll_tests {
    use super::*;

    #[test]
    fn respects_refresh_retry_and_backoff() {
        let mut failures = 0;
        let mut report = UsageReport::ok(UsageResult {
            provider: "codex".into(),
            windows: vec![],
        });
        assert_eq!(
            next_hidden_poll_delay(&report, &mut failures, 60).as_secs(),
            60
        );
        report.error = Some(providers::ProviderError::RateLimited("wait".into()));
        report.retry_after_s = Some(120);
        assert_eq!(
            next_hidden_poll_delay(&report, &mut failures, 60).as_secs(),
            120
        );
        assert_eq!(failures, 0);
        report.retry_after_s = None;
        assert_eq!(
            next_hidden_poll_delay(&report, &mut failures, 60).as_secs(),
            30
        );
        assert_eq!(
            next_hidden_poll_delay(&report, &mut failures, 60).as_secs(),
            300
        );
        assert_eq!(
            next_hidden_poll_delay(&report, &mut failures, 60).as_secs(),
            900
        );
        report.error = None;
        assert_eq!(
            next_hidden_poll_delay(&report, &mut failures, 60).as_secs(),
            60
        );
        assert_eq!(failures, 0);
    }
}
