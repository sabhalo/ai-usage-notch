//! Un unico `settings.json` per tutte le preferenze persistite dell'utente
//! (posizione finestra, provider attivi, intervallo di refresh, soglia di
//! allerta) invece di un file separato per ognuna — vedi plan/step-4.6.md,
//! che riusa deliberatamente il meccanismo dello step-4.1.md.
//!
//! L'autostart (step-4.7.md) NON è qui: `tauri-plugin-autostart` è già la
//! fonte di verità per quello (registra/rimuove l'app dal login OS), un
//! secondo flag qui rischierebbe solo di disallinearsi.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

fn default_active_providers() -> HashMap<String, bool> {
    ["claude", "codex", "copilot"]
        .into_iter()
        .map(|p| (p.to_string(), true))
        .collect()
}

fn default_refresh_interval_s() -> u64 {
    300
}

fn default_alert_threshold_pct() -> f64 {
    80.0
}

// "always" | "auto_collapse" — vedi issue #8. La pill non collassa mai finché
// l'utente non lo sceglie esplicitamente nelle impostazioni.
fn default_pill_visibility_mode() -> String {
    "always".to_string()
}

fn default_pill_collapse_delay_s() -> u64 {
    3
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default)]
    pub window_position: Option<(i32, i32)>,
    #[serde(default = "default_active_providers")]
    pub active_providers: HashMap<String, bool>,
    #[serde(default = "default_refresh_interval_s")]
    pub refresh_interval_s: u64,
    #[serde(default = "default_alert_threshold_pct")]
    pub alert_threshold_pct: f64,
    #[serde(default = "default_pill_visibility_mode")]
    pub pill_visibility_mode: String,
    #[serde(default = "default_pill_collapse_delay_s")]
    pub pill_collapse_delay_s: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            window_position: None,
            active_providers: default_active_providers(),
            refresh_interval_s: default_refresh_interval_s(),
            alert_threshold_pct: default_alert_threshold_pct(),
            pill_visibility_mode: default_pill_visibility_mode(),
            pill_collapse_delay_s: default_pill_collapse_delay_s(),
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("ai-usage-notch").join("settings.json"))
}

pub fn load() -> Settings {
    settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save(settings: &Settings) {
    let Some(path) = settings_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, json);
    }
}
