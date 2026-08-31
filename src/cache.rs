//! Cache su disco dell'ultimo dato valido, per mostrare subito qualcosa
//! all'avvio invece di "—" mentre parte la prima fetch di rete.
//!
//! Contiene solo `UsageReport` (percentuali, label, errori tipizzati senza
//! messaggio col token dentro) — mai credenziali. Vedi plan/step-3.1.md.

use crate::providers::{UsageReport, SUPPORTED_PROVIDER_IDS};
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
    let mut cached: CachedUsage = serde_json::from_str(&raw).ok()?;
    cached.reports = supported_reports(&cached.reports);
    Some(cached)
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
        reports: supported_reports(reports),
    };
    if let Ok(json) = serde_json::to_string(&cached) {
        let _ = std::fs::write(path, json);
    }
}

fn supported_reports(reports: &[UsageReport]) -> Vec<UsageReport> {
    reports
        .iter()
        .filter(|report| SUPPORTED_PROVIDER_IDS.contains(&report.provider.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(provider: &str) -> UsageReport {
        UsageReport {
            provider: provider.to_string(),
            windows: vec![],
            error: None,
            retry_after_s: None,
        }
    }

    #[test]
    fn keeps_supported_report() {
        assert_eq!(supported_reports(&[report("claude")]).len(), 1);
    }

    #[test]
    fn removes_unsupported_report() {
        assert!(supported_reports(&[report("removed-provider")]).is_empty());
    }

    #[test]
    fn filters_mixed_reports() {
        let reports = supported_reports(&[report("claude"), report("removed-provider")]);

        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].provider, "claude");
    }

    #[test]
    fn keeps_all_supported_reports() {
        let reports = supported_reports(&[report("claude"), report("codex"), report("copilot")]);

        assert_eq!(reports.len(), 3);
    }
}
