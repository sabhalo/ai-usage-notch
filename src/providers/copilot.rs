use super::{ProviderError, UsageProvider, UsageResult, UsageWindow, too_many_requests, FetchError};
use crate::credentials;

/// GitHub Copilot: quota mensile di "premium request", non finestre a
/// scorrimento. Shape verificata su risposta reale, vedi
/// fixtures/copilot-usage.json e docs/endpoints.md.
pub struct CopilotProvider;

impl UsageProvider for CopilotProvider {
    fn id(&self) -> &'static str {
        "copilot"
    }

    async fn fetch(&self) -> Result<UsageResult, FetchError> {
        let token = credentials::copilot_token()?;

        let client = reqwest::Client::new();
        let resp = client
            .get("https://api.github.com/copilot_internal/user")
            .header("Authorization", format!("token {token}"))
            .header("User-Agent", "GitHubCopilotChat/0.26.7")
            .header("Editor-Version", "vscode/1.96.2")
            .send()
            .await
            .map_err(|e| ProviderError::Network(format!("Richiesta fallita: {e}")))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Unauthorized(
                "GitHub ha risposto 401: token scaduto o senza gli scope necessari. Prova 'gh \
                 auth refresh -h github.com -s user'."
                    .to_string(),
            )
            .into());
        }
        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(too_many_requests("GitHub", resp.headers()));
        }
        if !resp.status().is_success() {
            return Err(
                ProviderError::Network(format!("GitHub ha risposto {}", resp.status())).into(),
            );
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::UnexpectedShape(format!("Risposta illeggibile: {e}")))?;

        #[cfg(debug_assertions)]
        eprintln!("[copilot usage raw] {:#}", body);

        parse_copilot_usage(&body).map_err(Into::into)
    }
}

/// `chat` e `completions` sono `unlimited: true` in pratica su ogni piano
/// osservato finora (vedi fixtures/copilot-usage.json) — mostrarli come
/// finestre a sé aggiungerebbe due anelli "∞" quasi sempre non azionabili.
/// Scelta deliberata (step-2.3.md): restano ignorati, solo
/// `premium_interactions` (quella con un costo reale in overage) è esposta.
fn parse_copilot_usage(body: &serde_json::Value) -> Result<UsageResult, ProviderError> {
    let premium = body
        .get("quota_snapshots")
        .and_then(|q| q.get("premium_interactions"))
        .ok_or_else(|| {
            ProviderError::UnexpectedShape(
                "Risposta ricevuta ma 'premium_interactions' assente — rilancia \
                 scripts/probe.sh e controlla docs/endpoints.md."
                    .to_string(),
            )
        })?;

    let reset_date = body
        .get("quota_reset_date")
        .and_then(|v| v.as_str())
        .unwrap_or("—");
    let unlimited = premium
        .get("unlimited")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let overage_count = premium.get("overage_count").and_then(|v| v.as_i64()).unwrap_or(0);

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

    Ok(UsageResult {
        provider: "copilot".into(),
        windows: vec![UsageWindow {
            label,
            used_percent,
            resets_in: reset_date.to_string(),
            unlimited,
            in_overage: overage_count > 0,
        }],
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
    fn parses_real_copilot_fixture() {
        let body = fixture("copilot-usage.json");
        let result = parse_copilot_usage(&body).unwrap();
        assert_eq!(result.windows.len(), 1);
        assert_eq!(result.windows[0].label, "Premium requests (216/15000)");
        assert!((result.windows[0].used_percent - 98.6).abs() < 0.01);
    }
}
