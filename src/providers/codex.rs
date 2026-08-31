use super::{ProviderError, UsageProvider, UsageResult, UsageWindow, fmt_reset, fmt_window, too_many_requests, FetchError};
use crate::credentials;

/// Stessa logica per Codex CLI. `~/.codex/auth.json` contiene un access token
/// e un account id; l'endpoint `backend-api/wham/usage` è anch'esso non
/// documentato pubblicamente. Shape verificata su risposta reale, vedi
/// fixtures/codex-usage.json e docs/endpoints.md.
pub struct CodexProvider;

impl UsageProvider for CodexProvider {
    fn id(&self) -> &'static str {
        "codex"
    }

    async fn fetch(&self) -> Result<UsageResult, FetchError> {
        let creds = credentials::codex_creds()?;

        let client = reqwest::Client::new();
        let resp = client
            .get("https://chatgpt.com/backend-api/wham/usage")
            .bearer_auth(&creds.access_token)
            .header("ChatGPT-Account-ID", &creds.account_id)
            .send()
            .await
            .map_err(|e| ProviderError::Network(format!("Richiesta fallita: {e}")))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ProviderError::Unauthorized(
                "OpenAI ha risposto 401: token scaduto o non valido, rifai 'codex login'."
                    .to_string(),
            )
            .into());
        }
        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(too_many_requests("OpenAI", resp.headers()));
        }
        if !resp.status().is_success() {
            return Err(
                ProviderError::Network(format!("OpenAI ha risposto {}", resp.status())).into(),
            );
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::UnexpectedShape(format!("Risposta illeggibile: {e}")))?;

        #[cfg(debug_assertions)]
        eprintln!("[codex usage raw] {:#}", body);

        parse_codex_usage(&body).map_err(Into::into)
    }
}

fn parse_codex_usage(body: &serde_json::Value) -> Result<UsageResult, ProviderError> {
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
                ..Default::default()
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
                ..Default::default()
            });
        }
    }

    if windows.is_empty() {
        return Err(ProviderError::UnexpectedShape(
            "Risposta ricevuta ma nessun campo riconosciuto — rilancia scripts/probe.sh e \
             controlla docs/endpoints.md."
                .to_string(),
        ));
    }

    Ok(UsageResult {
        provider: "codex".into(),
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
    fn parses_real_codex_fixture() {
        let body = fixture("codex-usage.json");
        let result = parse_codex_usage(&body).unwrap();
        assert_eq!(result.windows.len(), 1);
        assert_eq!(result.windows[0].used_percent, 19.0);
    }
}
