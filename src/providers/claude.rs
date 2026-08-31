use super::{ProviderError, UsageProvider, UsageResult, UsageWindow, resets_in_from_iso, too_many_requests, FetchError};
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
}
