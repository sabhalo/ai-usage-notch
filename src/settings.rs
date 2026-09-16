//! Un unico `settings.json` per tutte le preferenze persistite dell'utente
//! (posizione finestra, provider attivi, intervallo di refresh, soglia di
//! allerta) invece di un file separato per ognuna — vedi plan/step-4.6.md,
//! che riusa deliberatamente il meccanismo dello step-4.1.md.
//!
//! L'autostart (step-4.7.md) NON è qui: `tauri-plugin-autostart` è già la
//! fonte di verità per quello (registra/rimuove l'app dal login OS), un
//! secondo flag qui rischierebbe solo di disallinearsi.

use crate::providers::SUPPORTED_PROVIDER_IDS;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

fn default_active_providers() -> HashMap<String, bool> {
    SUPPORTED_PROVIDER_IDS
        .iter()
        .map(|p| (p.to_string(), true))
        .collect()
}

fn default_refresh_interval_s() -> u64 {
    300
}

fn default_alert_threshold_pct() -> f64 {
    80.0
}

fn default_show_pill() -> bool {
    true
}

// "always" | "auto_collapse" — vedi issue #8. La pill non collassa mai finché
// l'utente non lo sceglie esplicitamente nelle impostazioni.
fn default_pill_visibility_mode() -> String {
    "always".to_string()
}

fn default_pill_collapse_delay_s() -> u64 {
    3
}

// Scala della Pill: moltiplicatore continuo applicato a tutte le misure del
// frontend via --pill-scale (main.js: applyPillScale). Non è una densità
// (vedi CONTEXT.md): agisce identicamente in entrambe le modalità di
// Visibilità, sullo stato Visible come sul Collapsed e sul Panel.
fn default_pill_scale() -> f64 {
    1.0
}

pub const PILL_SCALE_MIN: f64 = 0.7;
pub const PILL_SCALE_MAX: f64 = 2.0;

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
    #[serde(default = "default_show_pill")]
    pub show_pill: bool,
    #[serde(default = "default_pill_visibility_mode")]
    pub pill_visibility_mode: String,
    #[serde(default = "default_pill_collapse_delay_s")]
    pub pill_collapse_delay_s: u64,
    #[serde(default = "default_pill_scale")]
    pub pill_scale: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            window_position: None,
            active_providers: default_active_providers(),
            refresh_interval_s: default_refresh_interval_s(),
            alert_threshold_pct: default_alert_threshold_pct(),
            show_pill: default_show_pill(),
            pill_visibility_mode: default_pill_visibility_mode(),
            pill_collapse_delay_s: default_pill_collapse_delay_s(),
            pill_scale: default_pill_scale(),
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("ai-usage-notch").join("settings.json"))
}

pub fn load() -> Settings {
    let settings: Settings = settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    normalize(settings)
}

pub fn save(settings: &Settings) {
    let Some(path) = settings_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&normalize(settings.clone())) {
        let _ = std::fs::write(path, json);
    }
}

fn normalize(mut settings: Settings) -> Settings {
    settings
        .active_providers
        .retain(|provider, _| SUPPORTED_PROVIDER_IDS.contains(&provider.as_str()));
    for provider in SUPPORTED_PROVIDER_IDS {
        settings
            .active_providers
            .entry(provider.to_string())
            .or_insert(true);
    }
    // Unico punto attraversato sia da load() sia da save(): copre un
    // settings.json modificato a mano e un valore fuori range dal frontend.
    settings.pill_scale = if settings.pill_scale.is_finite() {
        settings.pill_scale.clamp(PILL_SCALE_MIN, PILL_SCALE_MAX)
    } else {
        default_pill_scale()
    };
    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_unknown_provider() {
        let mut settings = Settings::default();
        settings
            .active_providers
            .insert("removed-provider".to_string(), true);

        let normalized = normalize(settings);

        assert!(!normalized.active_providers.contains_key("removed-provider"));
    }

    #[test]
    fn adds_missing_supported_provider_with_default() {
        let mut settings = Settings::default();
        settings.active_providers.remove("codex");

        let normalized = normalize(settings);

        assert_eq!(normalized.active_providers.get("codex"), Some(&true));
    }

    #[test]
    fn preserves_false_for_supported_provider() {
        let mut settings = Settings::default();
        settings
            .active_providers
            .insert("copilot".to_string(), false);

        let normalized = normalize(settings);

        assert_eq!(normalized.active_providers.get("copilot"), Some(&false));
    }

    #[test]
    fn preserves_other_settings_properties() {
        let settings = Settings {
            window_position: Some((12, 34)),
            active_providers: HashMap::new(),
            refresh_interval_s: 42,
            alert_threshold_pct: 73.5,
            show_pill: false,
            pill_visibility_mode: "auto_collapse".to_string(),
            pill_collapse_delay_s: 9,
            pill_scale: 1.4,
        };

        let normalized = normalize(settings);

        assert_eq!(normalized.window_position, Some((12, 34)));
        assert_eq!(normalized.refresh_interval_s, 42);
        assert_eq!(normalized.alert_threshold_pct, 73.5);
        assert!(!normalized.show_pill);
        assert_eq!(normalized.pill_visibility_mode, "auto_collapse");
        assert_eq!(normalized.pill_collapse_delay_s, 9);
        assert_eq!(normalized.pill_scale, 1.4);
    }

    #[test]
    fn clamps_pill_scale_out_of_range() {
        let with_scale = |scale| Settings {
            pill_scale: scale,
            ..Default::default()
        };

        assert_eq!(normalize(with_scale(5.0)).pill_scale, PILL_SCALE_MAX);
        assert_eq!(normalize(with_scale(0.1)).pill_scale, PILL_SCALE_MIN);
        assert_eq!(normalize(with_scale(f64::NAN)).pill_scale, 1.0);
    }
}
