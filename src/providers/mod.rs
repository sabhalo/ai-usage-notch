pub mod claude;
pub mod codex;
pub mod copilot;
pub mod gemini;

pub use claude::ClaudeProvider;
pub use codex::CodexProvider;
pub use copilot::CopilotProvider;
pub use gemini::GeminiProvider;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct UsageWindow {
    pub label: String,
    pub used_percent: f64,
    pub resets_in: String,
    /// Quota illimitata: la UI deve mostrare "∞", non uno 0% ambiguo
    /// (vedi plan/step-2.3.md).
    pub unlimited: bool,
    /// `true` quando il provider sta effettivamente sforando la quota
    /// (costa denaro) — distinto dal semplice 100% raggiunto.
    pub in_overage: bool,
}

/// Dato "pulito" restituito da un provider quando il fetch riesce.
#[derive(Serialize, Clone)]
pub struct UsageResult {
    pub provider: String,
    pub windows: Vec<UsageWindow>,
}

/// Non deve mai contenere il token: solo messaggi descrittivi per la UI.
/// Il tag `kind` è quello su cui il frontend distingue i casi
/// (vedi step-3.3.md): NotLoggedIn / Unauthorized / Network hanno un
/// messaggio azionabile diretto, UnexpectedShape dice esplicitamente di
/// rilanciare scripts/probe.sh, RateLimited è transitorio (vedi
/// `UsageReport::retry_after_s`).
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", content = "message")]
pub enum ProviderError {
    NotLoggedIn(String),
    Unauthorized(String),
    Network(String),
    UnexpectedShape(String),
    RateLimited(String),
}

/// Errore di fetch più l'eventuale hint del server su quanto attendere
/// (429 + header `Retry-After`). Il numero di secondi resta un campo Rust a
/// parte invece che dentro `ProviderError::RateLimited`: quest'ultimo deve
/// restare un newtype-variant a un solo campo `String`, da cui dipende la
/// forma JSON `{"kind":"RateLimited","message":"..."}` che il frontend legge
/// come stringa pura (`report.error.message`) — un secondo campo nella
/// variante lo trasformerebbe in un array e romperebbe quel contratto.
pub struct FetchError {
    pub error: ProviderError,
    pub retry_after_s: Option<u64>,
}

impl From<ProviderError> for FetchError {
    fn from(error: ProviderError) -> Self {
        FetchError { error, retry_after_s: None }
    }
}

/// Legge l'header `retry-after` (intero, secondi) e costruisce l'errore 429
/// uniforme per i tre provider — duplicato altrimenti in claude.rs/codex.rs/
/// copilot.rs. Fallback a 900s se l'header manca o non è un intero valido.
pub(crate) fn too_many_requests(provider_label: &str, headers: &reqwest::header::HeaderMap) -> FetchError {
    let secs = parse_retry_after_header(headers).unwrap_or(900);
    FetchError {
        error: ProviderError::RateLimited(format!(
            "Limite di richieste {provider_label} raggiunto, riprovo tra {}",
            fmt_reset(Some(secs as i64))
        )),
        retry_after_s: Some(secs),
    }
}

fn parse_retry_after_header(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Shape restituita al frontend da `get_all_usage`: un `UsageResult` più un
/// errore tipizzato invece del vecchio campo libero `error: Option<String>`.
#[derive(Serialize, Deserialize, Clone)]
pub struct UsageReport {
    pub provider: String,
    pub windows: Vec<UsageWindow>,
    pub error: Option<ProviderError>,
    /// Solo per `ProviderError::RateLimited`: secondi suggeriti dal server
    /// prima di ritentare. `None` per tutti gli altri esiti (successo o
    /// altri tipi di errore) — `#[serde(default)]` per restare compatibile
    /// con la cache su disco scritta prima di questo campo.
    #[serde(default)]
    pub retry_after_s: Option<u64>,
}

impl UsageReport {
    pub fn ok(result: UsageResult) -> Self {
        UsageReport {
            provider: result.provider,
            windows: result.windows,
            error: None,
            retry_after_s: None,
        }
    }

    pub fn err(provider: &str, error: FetchError) -> Self {
        UsageReport {
            provider: provider.into(),
            windows: vec![],
            error: Some(error.error),
            retry_after_s: error.retry_after_s,
        }
    }
}

pub trait UsageProvider {
    fn id(&self) -> &'static str;
    async fn fetch(&self) -> Result<UsageResult, FetchError>;
}

pub(crate) fn fmt_reset(seconds: Option<i64>) -> String {
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

pub(crate) fn fmt_window(seconds: Option<i64>) -> String {
    match seconds {
        Some(s) if s >= 86400 => format!("{}g", s / 86400),
        Some(s) if s > 0 => format!("{}h", s / 3600),
        _ => String::new(),
    }
}

/// Converte una data ISO 8601 UTC ("2026-08-28T15:19:59.85+00:00") in secondi
/// dall'epoch, senza dipendenze esterne (algoritmo civile di Howard Hinnant).
/// Condiviso da claude.rs (`resets_at`) e gemini.rs (`resetTime`) — stesso
/// formato, fonti diverse.
pub(crate) fn parse_iso_utc_epoch(s: &str) -> Option<i64> {
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

pub(crate) fn resets_in_from_iso(resets_at: Option<&str>) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_iso_utc_epoch_correctly() {
        assert_eq!(parse_iso_utc_epoch("2026-08-28T15:19:59+00:00"), Some(1787930399));
        assert_eq!(parse_iso_utc_epoch("not a date"), None);
    }

    #[test]
    fn parses_retry_after_seconds() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, "418".parse().unwrap());
        assert_eq!(parse_retry_after_header(&headers), Some(418));
    }

    #[test]
    fn falls_back_to_900_when_header_missing() {
        let fe = too_many_requests("Anthropic", &reqwest::header::HeaderMap::new());
        assert_eq!(fe.retry_after_s, Some(900));
    }

    #[test]
    fn falls_back_to_900_when_header_unparseable() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, "Wed, 21 Oct".parse().unwrap());
        let fe = too_many_requests("Anthropic", &headers);
        assert_eq!(fe.retry_after_s, Some(900));
    }
}
