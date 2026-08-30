use super::{ProviderError, UsageProvider, UsageResult, UsageWindow, fmt_reset, too_many_requests, FetchError};
use crate::credentials;

/// Shape della risposta verificata su dati reali, vedi
/// fixtures/claude-usage.json e docs/endpoints.md.
pub struct ClaudeProvider;

impl UsageProvider for ClaudeProvider {
    fn id(&self) -> &'static str {
        "claude"
    }

    async fn fetch(&self) -> Result<UsageResult, FetchError> {
        let token = credentials::claude_token()?;

        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.anthropic.com/api/oauth/usage")
            .bearer_auth(token)
            .header("anthropic-beta", "oauth-2025-04-20")
            .header("User-Agent", "claude-code")
            .send()
            .await
            .map_err(|e| ProviderError::Network(format!("Richiesta fallita: {e}")))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Unauthorized(
                "Anthropic ha risposto 401: token scaduto o non valido, rifai 'claude login'."
                    .to_string(),
            )
            .into());
        }
        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(too_many_requests("Anthropic", resp.headers()));
        }
        if !resp.status().is_success() {
            return Err(ProviderError::Network(format!(
                "Anthropic ha risposto {}",
                resp.status()
            ))
            .into());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::UnexpectedShape(format!("Risposta illeggibile: {e}")))?;

        #[cfg(debug_assertions)]
        eprintln!("[claude usage raw] {:#}", body);

        parse_claude_usage(&body).map_err(Into::into)
    }
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

fn parse_claude_usage(body: &serde_json::Value) -> Result<UsageResult, ProviderError> {
    let mut windows = vec![];
    if let Some(session) = body.get("five_hour") {
        let pct = session.get("utilization").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let resets = session.get("resets_at").and_then(|v| v.as_str());
        windows.push(UsageWindow {
            label: "Sessione (5h)".into(),
            used_percent: pct,
            resets_in: resets_in_from_iso(resets),
            ..Default::default()
        });
    }
    if let Some(week) = body.get("seven_day") {
        let pct = week.get("utilization").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let resets = week.get("resets_at").and_then(|v| v.as_str());
        windows.push(UsageWindow {
            label: "Tutti i modelli (7g)".into(),
            used_percent: pct,
            resets_in: resets_in_from_iso(resets),
            ..Default::default()
        });
    }

    if windows.is_empty() {
        return Err(ProviderError::UnexpectedShape(
            "Risposta ricevuta ma nessun campo riconosciuto — rilancia scripts/probe.sh e \
             controlla docs/endpoints.md."
                .to_string(),
        ));
    }

    Ok(UsageResult {
        provider: "claude".into(),
        windows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture(name: &str) -> serde_json::Value {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name);
        let raw = fs::read_to_string(path).unwrap();
        serde_json::from_str(&raw).unwrap()
    }

    #[test]
    fn parses_real_claude_fixture() {
        let body = fixture("claude-usage.json");
        let result = parse_claude_usage(&body).unwrap();
        assert_eq!(result.windows.len(), 2);
        assert_eq!(result.windows[0].used_percent, 48.0);
        assert_eq!(result.windows[1].used_percent, 12.0);
    }

    #[test]
    fn parses_iso_utc_epoch_correctly() {
        assert_eq!(parse_iso_utc_epoch("2026-08-28T15:19:59+00:00"), Some(1787930399));
        assert_eq!(parse_iso_utc_epoch("not a date"), None);
    }
}
