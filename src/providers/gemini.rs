use super::{FetchError, ProviderError, UsageProvider, UsageResult, UsageWindow, resets_in_from_iso};

/// Antigravity, via il language server locale della CLI `agy` — diverso dagli
/// altri tre: niente file/Keychain di credenziali, l'endpoint esiste solo
/// mentre una sessione `agy` gira (verificato: `agy` in modalità `--print`
/// esce subito, il language server resta su solo dentro una sessione
/// interattiva). Verificato anche che l'RPC locale non richiede alcun token
/// (fiducia sul loopback) — il campo Bearer ipotizzato in origine per questo
/// provider non esiste. Shape reale in fixtures/gemini-usage.json e
/// docs/endpoints.md.
pub struct GeminiProvider;

impl UsageProvider for GeminiProvider {
    fn id(&self) -> &'static str {
        "gemini"
    }

    async fn fetch(&self) -> Result<UsageResult, FetchError> {
        let ports = discover_agy_ports();
        if ports.is_empty() {
            return Err(ProviderError::NotLoggedIn(
                "Antigravity CLI ('agy') non risulta in esecuzione. Avvia una sessione 'agy' per \
                 leggere la quota Gemini."
                    .to_string(),
            )
            .into());
        }

        let client = reqwest::Client::new();
        let mut last_err = None;
        for port in ports {
            let url = format!(
                "http://127.0.0.1:{port}/exa.language_server_pb.LanguageServerService/RetrieveUserQuotaSummary"
            );
            let resp = match client
                .post(&url)
                .header("Content-Type", "application/json")
                .body("{}")
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            };
            if !resp.status().is_success() {
                last_err = Some(resp.status().to_string());
                continue;
            }

            let body: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| ProviderError::UnexpectedShape(format!("Risposta illeggibile: {e}")))?;

            #[cfg(debug_assertions)]
            eprintln!("[gemini usage raw] {:#}", body);

            return parse_gemini_usage(&body).map_err(Into::into);
        }

        Err(ProviderError::Network(format!(
            "Nessuna porta locale di 'agy' ha risposto correttamente ({})",
            last_err.unwrap_or_else(|| "nessun dettaglio".to_string())
        ))
        .into())
    }
}

/// `agy` non è un demone: ogni sessione interattiva apre due porte TCP
/// effimere su loopback (una HTTPS, una HTTP — solo quest'ultima serve
/// l'RPC in chiaro). Il notch non avvia/gestisce `agy` (richiederebbe un
/// vero TTY, verificato con bubbletea), si limita a scoprire una sessione
/// già in corso: `pgrep` per il processo, `lsof` per le sue porte in
/// ascolto. Prova ogni porta trovata finché una risponde.
fn discover_agy_ports() -> Vec<u16> {
    let Ok(pids) = std::process::Command::new("pgrep").args(["-x", "agy"]).output() else {
        return vec![];
    };
    if !pids.status.success() {
        return vec![];
    }

    let mut ports = vec![];
    for pid in String::from_utf8_lossy(&pids.stdout).lines() {
        let pid = pid.trim();
        if pid.is_empty() {
            continue;
        }
        let Ok(lsof) = std::process::Command::new("lsof")
            .args(["-nP", "-a", "-p", pid, "-iTCP", "-sTCP:LISTEN"])
            .output()
        else {
            continue;
        };
        for line in String::from_utf8_lossy(&lsof.stdout).lines() {
            if let Some(port) = line
                .rsplit(':')
                .next()
                .and_then(|tail| tail.split_whitespace().next())
                .and_then(|p| p.parse::<u16>().ok())
            {
                ports.push(port);
            }
        }
    }
    ports
}

/// `RetrieveUserQuotaSummary` raggruppa i modelli in due pool con quota
/// settimanale condivisa (verificato su risposta reale): `gemini-weekly`
/// (i modelli Gemini, quello che guida l'anello) e `3p-weekly` (Claude+GPT
/// via Antigravity, informativo). `windows[0]` è sempre il pool Gemini
/// così la UI esistente (che legge `windows[0]` per l'anello) mostra la
/// quota giusta senza modifiche.
fn parse_gemini_usage(body: &serde_json::Value) -> Result<UsageResult, ProviderError> {
    let groups = body
        .get("response")
        .and_then(|r| r.get("groups"))
        .and_then(|g| g.as_array())
        .filter(|g| !g.is_empty())
        .ok_or_else(|| {
            ProviderError::UnexpectedShape(
                "Risposta ricevuta ma 'response.groups' assente o vuoto — rilancia \
                 scripts/probe.sh e controlla docs/endpoints.md."
                    .to_string(),
            )
        })?;

    let mut windows: Vec<(bool, UsageWindow)> = groups
        .iter()
        .filter_map(|g| {
            let bucket = g.get("buckets")?.as_array()?.first()?;
            let remaining = bucket.get("remainingFraction").and_then(|v| v.as_f64())?;
            let bucket_id = bucket.get("bucketId").and_then(|v| v.as_str()).unwrap_or("");
            let (is_gemini, label) = match bucket_id {
                "gemini-weekly" => (true, "Gemini (settimanale)".to_string()),
                "3p-weekly" => (false, "Claude + GPT (settimanale)".to_string()),
                _ => (
                    false,
                    g.get("displayName")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Gruppo modelli")
                        .to_string(),
                ),
            };
            let resets = bucket.get("resetTime").and_then(|v| v.as_str());
            Some((
                is_gemini,
                UsageWindow {
                    label,
                    used_percent: (1.0 - remaining) * 100.0,
                    resets_in: resets_in_from_iso(resets),
                    ..Default::default()
                },
            ))
        })
        .collect();

    if windows.is_empty() {
        return Err(ProviderError::UnexpectedShape(
            "Risposta ricevuta ma nessun bucket riconosciuto — rilancia scripts/probe.sh e \
             controlla docs/endpoints.md."
                .to_string(),
        ));
    }

    windows.sort_by_key(|(is_gemini, _)| !is_gemini);

    Ok(UsageResult {
        provider: "gemini".into(),
        windows: windows.into_iter().map(|(_, w)| w).collect(),
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
    fn parses_real_gemini_fixture() {
        let body = fixture("gemini-usage.json");
        let result = parse_gemini_usage(&body).unwrap();
        assert_eq!(result.windows.len(), 2);
        assert_eq!(result.windows[0].label, "Gemini (settimanale)");
        assert!((result.windows[0].used_percent - 1.2576).abs() < 0.001);
        assert_eq!(result.windows[1].label, "Claude + GPT (settimanale)");
        assert_eq!(result.windows[1].used_percent, 0.0);
    }

    #[test]
    fn gemini_group_always_first_regardless_of_api_order() {
        let mut body = fixture("gemini-usage.json");
        let groups = body["response"]["groups"].as_array_mut().unwrap();
        groups.reverse();
        let result = parse_gemini_usage(&body).unwrap();
        assert_eq!(result.windows[0].label, "Gemini (settimanale)");
    }
}
