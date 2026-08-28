// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Clone)]
struct UsageWindow {
    label: String,
    used_percent: f64,
    resets_in: String,
}

#[derive(Serialize, Clone)]
struct UsageResult {
    provider: String,
    windows: Vec<UsageWindow>,
    error: Option<String>,
}

fn claude_creds_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join(".credentials.json"))
}

fn codex_auth_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".codex").join("auth.json"))
}

fn fmt_reset(seconds: Option<i64>) -> String {
    match seconds {
        Some(s) if s > 0 => {
            let h = s / 3600;
            let m = (s % 3600) / 60;
            if h > 0 {
                format!("{h}h {m}m")
            } else {
                format!("{m}m")
            }
        }
        _ => "—".to_string(),
    }
}

fn err(provider: &str, msg: impl Into<String>) -> UsageResult {
    UsageResult {
        provider: provider.into(),
        windows: vec![],
        error: Some(msg.into()),
    }
}

/// Legge il token OAuth salvato da `claude login` (Claude Code CLI).
///
/// NOTA: l'endpoint `api/oauth/usage` non è documentato pubblicamente da Anthropic
/// — è lo stesso usato internamente dal comando `/usage` della CLI. Il nome dei
/// campi nella risposta JSON qui sotto (`five_hour`, `seven_day`, `utilization`,
/// `resets_in_seconds`) è la forma più plausibile in base ai tool community che
/// lo usano, ma NON è verificato contro una risposta reale. In debug, il corpo
/// grezzo viene stampato su stderr: eseguendo `cargo tauri dev` la prima volta
/// puoi controllare la console e correggere i nomi dei campi qui sotto in 2 minuti
/// se non corrispondono.
#[tauri::command]
async fn get_claude_usage() -> UsageResult {
    let path = match claude_creds_path() {
        Some(p) => p,
        None => return err("claude", "Home directory non trovata"),
    };

    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(_) => {
            return err(
                "claude",
                format!(
                    "Credenziali non trovate ({}). Esegui 'claude login'.",
                    path.display()
                ),
            )
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(j) => j,
        Err(e) => return err("claude", format!("Credenziali illeggibili: {e}")),
    };

    // Lo shape del file è cambiato nel tempo tra versioni della CLI: proviamo
    // entrambe le forme note.
    let token = json
        .get("claudeAiOauth")
        .and_then(|v| v.get("accessToken"))
        .or_else(|| json.get("accessToken"))
        .and_then(|v| v.as_str());

    let token = match token {
        Some(t) => t,
        None => return err("claude", "Token OAuth non trovato in .credentials.json"),
    };

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", "claude-code")
        .send()
        .await;

    let resp = match resp {
        Ok(r) => r,
        Err(e) => return err("claude", format!("Richiesta fallita: {e}")),
    };

    if !resp.status().is_success() {
        return err("claude", format!("Anthropic ha risposto {}", resp.status()));
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(e) => return err("claude", format!("Risposta illeggibile: {e}")),
    };

    #[cfg(debug_assertions)]
    eprintln!("[claude usage raw] {:#}", body);

    let mut windows = vec![];
    if let Some(session) = body.get("five_hour").or_else(|| body.get("session")) {
        let pct = session
            .get("utilization")
            .and_then(|v| v.as_f64())
            .or_else(|| session.get("used_percent").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);
        let resets = session.get("resets_in_seconds").and_then(|v| v.as_i64());
        windows.push(UsageWindow {
            label: "Sessione (5h)".into(),
            used_percent: pct,
            resets_in: fmt_reset(resets),
        });
    }
    if let Some(week) = body.get("seven_day").or_else(|| body.get("week")) {
        let pct = week
            .get("utilization")
            .and_then(|v| v.as_f64())
            .or_else(|| week.get("used_percent").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);
        let resets = week.get("resets_in_seconds").and_then(|v| v.as_i64());
        windows.push(UsageWindow {
            label: "Tutti i modelli".into(),
            used_percent: pct,
            resets_in: fmt_reset(resets),
        });
    }

    if windows.is_empty() {
        return err(
            "claude",
            "Risposta ricevuta ma nessun campo riconosciuto — controlla il log di debug e aggiorna il parsing in main.rs",
        );
    }

    UsageResult {
        provider: "claude".into(),
        windows,
        error: None,
    }
}

/// Stessa logica per Codex CLI. `~/.codex/auth.json` contiene un access token
/// e un account id; l'endpoint `backend-api/wham/usage` è anch'esso non
/// documentato pubblicamente. Vale la stessa nota di cui sopra sul nome dei campi.
#[tauri::command]
async fn get_codex_usage() -> UsageResult {
    let path = match codex_auth_path() {
        Some(p) => p,
        None => return err("codex", "Home directory non trovata"),
    };

    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(_) => {
            return err(
                "codex",
                format!(
                    "Credenziali non trovate ({}). Esegui 'codex login'.",
                    path.display()
                ),
            )
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(j) => j,
        Err(e) => return err("codex", format!("Credenziali illeggibili: {e}")),
    };

    let access_token = json
        .get("tokens")
        .and_then(|t| t.get("access_token"))
        .and_then(|v| v.as_str());
    let account_id = json
        .get("tokens")
        .and_then(|t| t.get("account_id"))
        .and_then(|v| v.as_str());

    let (access_token, account_id) = match (access_token, account_id) {
        (Some(a), Some(b)) => (a, b),
        _ => return err("codex", "Token o account id mancanti in auth.json"),
    };

    let client = reqwest::Client::new();
    let resp = client
        .get("https://chatgpt.com/backend-api/wham/usage")
        .bearer_auth(access_token)
        .header("ChatGPT-Account-ID", account_id)
        .send()
        .await;

    let resp = match resp {
        Ok(r) => r,
        Err(e) => return err("codex", format!("Richiesta fallita: {e}")),
    };

    if !resp.status().is_success() {
        return err("codex", format!("OpenAI ha risposto {}", resp.status()));
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(e) => return err("codex", format!("Risposta illeggibile: {e}")),
    };

    #[cfg(debug_assertions)]
    eprintln!("[codex usage raw] {:#}", body);

    let mut windows = vec![];
    if let Some(rl) = body.get("rate_limits") {
        if let Some(primary) = rl.get("primary") {
            let pct = primary
                .get("used_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let resets = primary.get("resets_in_seconds").and_then(|v| v.as_i64());
            windows.push(UsageWindow {
                label: "Sessione (5h)".into(),
                used_percent: pct,
                resets_in: fmt_reset(resets),
            });
        }
        if let Some(secondary) = rl.get("secondary") {
            let pct = secondary
                .get("used_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let resets = secondary.get("resets_in_seconds").and_then(|v| v.as_i64());
            windows.push(UsageWindow {
                label: "Settimanale".into(),
                used_percent: pct,
                resets_in: fmt_reset(resets),
            });
        }
    }

    if windows.is_empty() {
        return err(
            "codex",
            "Risposta ricevuta ma nessun campo riconosciuto — controlla il log di debug e aggiorna il parsing in main.rs",
        );
    }

    UsageResult {
        provider: "codex".into(),
        windows,
        error: None,
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let screen_size = monitor.size();
                    let scale = monitor.scale_factor();
                    if let Ok(win_size) = window.outer_size() {
                        let x = (screen_size.width as f64 / scale - win_size.width as f64 / scale)
                            / 2.0;
                        let _ = window.set_position(tauri::Position::Logical(
                            tauri::LogicalPosition::new(x, 0.0),
                        ));
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_claude_usage, get_codex_usage])
        .run(tauri::generate_context!())
        .expect("errore durante l'avvio dell'applicazione Tauri");
}
