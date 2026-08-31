//! Cache su disco dell'ultimo dato valido, per mostrare subito qualcosa
//! all'avvio invece di "—" mentre parte la prima fetch di rete.
//!
//! Contiene solo `UsageReport` (percentuali, label, errori tipizzati senza
//! messaggio col token dentro) — mai credenziali. Vedi plan/step-3.1.md.

use crate::providers::UsageReport;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct CachedUsage {
    pub timestamp: i64,
    pub reports: Vec<UsageReport>,
}

fn cache_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("ai-usage-notch").join("cache.json"))
}

pub fn load() -> Option<CachedUsage> {
    let path = cache_path()?;
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Best-effort: un fallimento nello scrivere la cache non deve mai far
/// fallire una fetch riuscita.
pub fn save(reports: &[UsageReport]) {
    let Some(path) = cache_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let cached = CachedUsage {
        timestamp,
        reports: reports.to_vec(),
    };
    if let Ok(json) = serde_json::to_string(&cached) {
        let _ = std::fs::write(path, json);
    }
}
