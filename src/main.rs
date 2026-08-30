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

#[cfg(not(target_os = "macos"))]
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

fn fmt_window(seconds: Option<i64>) -> String {
    match seconds {
        Some(s) if s >= 86400 => format!("{}g", s / 86400),
        Some(s) if s > 0 => format!("{}h", s / 3600),
        _ => String::new(),
    }
}

fn err(provider: &str, msg: impl Into<String>) -> UsageResult {
    UsageResult {
        provider: provider.into(),
        windows: vec![],
        error: Some(msg.into()),
    }
}

/// Legge il token OAuth salvato da `claude login` (Claude Code CLI): Keychain
/// su macOS (voce "Claude Code-credentials"), file su Linux/Windows.
/// Shape della risposta verificata su dati reali, vedi fixtures/claude-usage.json
/// e docs/endpoints.md.
#[cfg(target_os = "macos")]
fn claude_token() -> Result<String, String> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .map_err(|e| format!("impossibile eseguire 'security': {e}"))?;

    if !output.status.success() {
        return Err(
            "voce 'Claude Code-credentials' non trovata in Keychain. Esegui 'claude login'."
                .to_string(),
        );
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
        json.get("claudeAiOauth")
            .and_then(|v| v.get("accessToken"))
            .or_else(|| json.get("accessToken"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Token OAuth non trovato nella voce Keychain".to_string())
    } else if !raw.is_empty() {
        Ok(raw)
    } else {
        Err("Token OAuth vuoto in Keychain".to_string())
    }
}

#[cfg(not(target_os = "macos"))]
fn claude_token() -> Result<String, String> {
    let path = claude_creds_path().ok_or("Home directory non trovata")?;
    let raw = fs::read_to_string(&path).map_err(|_| {
        format!(
            "Credenziali non trovate ({}). Esegui 'claude login'.",
            path.display()
        )
    })?;
    let json: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("Credenziali illeggibili: {e}"))?;
    json.get("claudeAiOauth")
        .and_then(|v| v.get("accessToken"))
        .or_else(|| json.get("accessToken"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Token OAuth non trovato in .credentials.json".to_string())
}

#[tauri::command]
async fn get_claude_usage() -> UsageResult {
    let token = match claude_token() {
        Ok(t) => t,
        Err(e) => return err("claude", e),
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

    parse_claude_usage(&body)
}

/// Converte una data ISO 8601 UTC ("2026-08-28T15:19:59.85+00:00") in secondi
/// dall'epoch, senza dipendenze esterne (algoritmo civile di Howard Hinnant).
fn parse_iso_utc_epoch(s: &str) -> Option<i64> {
    let s = s.get(0..19)?; // "YYYY-MM-DDTHH:MM:SS", ignora frazioni/offset
    let (date, time) = s.split_once('T')?;
    let mut d = date.split('-');
    let y: i64 = d.next()?.parse().ok()?;
    let m: i64 = d.next()?.parse().ok()?;
    let day: i64 = d.next()?.parse().ok()?;
    let mut t = time.split(':');
    let h: i64 = t.next()?.parse().ok()?;
    let mi: i64 = t.next()?.parse().ok()?;
    let se: i64 = t.next()?.parse().ok()?;

    let y2 = if m <= 2 { y - 1 } else { y };
    let era = (if y2 >= 0 { y2 } else { y2 - 399 }) / 400;
    let yoe = y2 - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;

    Some(days * 86400 + h * 3600 + mi * 60 + se)
}

fn resets_in_from_iso(resets_at: Option<&str>) -> String {
    let target = match resets_at.and_then(parse_iso_utc_epoch) {
        Some(t) => t,
        None => return "—".to_string(),
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    fmt_reset(Some(target - now))
}

fn parse_claude_usage(body: &serde_json::Value) -> UsageResult {
    let mut windows = vec![];
    if let Some(session) = body.get("five_hour") {
        let pct = session.get("utilization").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let resets = session.get("resets_at").and_then(|v| v.as_str());
        windows.push(UsageWindow {
            label: "Sessione (5h)".into(),
            used_percent: pct,
            resets_in: resets_in_from_iso(resets),
        });
    }
    if let Some(week) = body.get("seven_day") {
        let pct = week.get("utilization").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let resets = week.get("resets_at").and_then(|v| v.as_str());
        windows.push(UsageWindow {
            label: "Tutti i modelli (7g)".into(),
            used_percent: pct,
            resets_in: resets_in_from_iso(resets),
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
/// documentato pubblicamente. Shape verificata su risposta reale, vedi
/// fixtures/codex-usage.json e docs/endpoints.md.
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

    parse_codex_usage(&body)
}

fn parse_codex_usage(body: &serde_json::Value) -> UsageResult {
    let mut windows = vec![];
    if let Some(rl) = body.get("rate_limit") {
        if let Some(primary) = rl.get("primary_window") {
            let pct = primary
                .get("used_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let resets = primary.get("reset_after_seconds").and_then(|v| v.as_i64());
            let window = fmt_window(primary.get("limit_window_seconds").and_then(|v| v.as_i64()));
            windows.push(UsageWindow {
                label: format!("Finestra primaria ({window})"),
                used_percent: pct,
                resets_in: fmt_reset(resets),
            });
        }
        if let Some(secondary) = rl.get("secondary_window").filter(|v| !v.is_null()) {
            let pct = secondary
                .get("used_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let resets = secondary.get("reset_after_seconds").and_then(|v| v.as_i64());
            let window = fmt_window(secondary.get("limit_window_seconds").and_then(|v| v.as_i64()));
            windows.push(UsageWindow {
                label: format!("Finestra secondaria ({window})"),
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

/// GitHub Copilot: quota mensile di "premium request", non finestre a
/// scorrimento. Shape verificata su risposta reale, vedi
/// fixtures/copilot-usage.json e docs/endpoints.md. La UI compatta a soglie
/// (illimitato → "∞", overage evidenziato) è lavoro di Fase 2/4, non qui.
#[tauri::command]
async fn get_copilot_usage() -> UsageResult {
    let token = match copilot_token() {
        Some(t) => t,
        None => {
            return err(
                "copilot",
                "Nessun token GitHub trovato. Esegui 'gh auth login', o imposta GITHUB_TOKEN/GH_TOKEN.",
            )
        }
    };

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/copilot_internal/user")
        .header("Authorization", format!("token {token}"))
        .header("User-Agent", "GitHubCopilotChat/0.26.7")
        .header("Editor-Version", "vscode/1.96.2")
        .send()
        .await;

    let resp = match resp {
        Ok(r) => r,
        Err(e) => return err("copilot", format!("Richiesta fallita: {e}")),
    };

    if !resp.status().is_success() {
        return err("copilot", format!("GitHub ha risposto {}", resp.status()));
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(e) => return err("copilot", format!("Risposta illeggibile: {e}")),
    };

    #[cfg(debug_assertions)]
    eprintln!("[copilot usage raw] {:#}", body);

    parse_copilot_usage(&body)
}

/// Ordine di preferenza: `gh auth token` (rispetta il login già fatto),
/// poi $GITHUB_TOKEN/$GH_TOKEN. Il fallback su apps.json di VS Code (Fase 2.1)
/// non è ancora implementato: non ne serve, `gh auth token` ha funzionato
/// anche senza lo scope `user` ipotizzato nel piano.
fn copilot_token() -> Option<String> {
    if let Ok(out) = std::process::Command::new("gh").args(["auth", "token"]).output() {
        if out.status.success() {
            let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GH_TOKEN"))
        .ok()
        .filter(|t| !t.is_empty())
}

fn parse_copilot_usage(body: &serde_json::Value) -> UsageResult {
    let premium = match body
        .get("quota_snapshots")
        .and_then(|q| q.get("premium_interactions"))
    {
        Some(p) => p,
        None => {
            return err(
                "copilot",
                "Risposta ricevuta ma 'premium_interactions' assente — controlla il log di debug",
            )
        }
    };

    let reset_date = body
        .get("quota_reset_date")
        .and_then(|v| v.as_str())
        .unwrap_or("—");
    let unlimited = premium
        .get("unlimited")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let (used_percent, label) = if unlimited {
        (0.0, "Premium requests (illimitate)".to_string())
    } else {
        let remaining_pct = premium
            .get("percent_remaining")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let remaining = premium.get("remaining").and_then(|v| v.as_i64()).unwrap_or(0);
        let entitlement = premium.get("entitlement").and_then(|v| v.as_i64()).unwrap_or(0);
        (
            100.0 - remaining_pct,
            format!("Premium requests ({remaining}/{entitlement})"),
        )
    };

    UsageResult {
        provider: "copilot".into(),
        windows: vec![UsageWindow {
            label,
            used_percent,
            resets_in: reset_date.to_string(),
        }],
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
        .invoke_handler(tauri::generate_handler![
            get_claude_usage,
            get_codex_usage,
            get_copilot_usage
        ])
        .run(tauri::generate_context!())
        .expect("errore durante l'avvio dell'applicazione Tauri");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_claude_fixture() {
        let raw = fs::read_to_string("fixtures/claude-usage.json").unwrap();
        let body: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let result = parse_claude_usage(&body);
        assert!(result.error.is_none());
        assert_eq!(result.windows.len(), 2);
        assert_eq!(result.windows[0].used_percent, 48.0);
        assert_eq!(result.windows[1].used_percent, 12.0);
    }

    #[test]
    fn parses_iso_utc_epoch_correctly() {
        assert_eq!(parse_iso_utc_epoch("2026-08-28T15:19:59+00:00"), Some(1787930399));
        assert_eq!(parse_iso_utc_epoch("not a date"), None);
    }

    #[test]
    fn parses_real_codex_fixture() {
        let raw = fs::read_to_string("fixtures/codex-usage.json").unwrap();
        let body: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let result = parse_codex_usage(&body);
        assert!(result.error.is_none());
        assert_eq!(result.windows.len(), 1);
        assert_eq!(result.windows[0].used_percent, 19.0);
    }

    #[test]
    fn parses_real_copilot_fixture() {
        let raw = fs::read_to_string("fixtures/copilot-usage.json").unwrap();
        let body: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let result = parse_copilot_usage(&body);
        assert!(result.error.is_none());
        assert_eq!(result.windows.len(), 1);
        assert_eq!(result.windows[0].label, "Premium requests (216/15000)");
        assert!((result.windows[0].used_percent - 98.6).abs() < 0.01);
    }
}
