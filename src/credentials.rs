//! Risoluzione delle credenziali per tutti e tre i provider, per piattaforma.
//! Nessuna funzione qui deve mai loggare o restituire il token dentro un
//! messaggio di errore (vedi nota di sicurezza in plan/step-1.3.md).

use crate::providers::ProviderError;
use std::fs;
use std::path::PathBuf;

#[cfg(not(target_os = "macos"))]
fn claude_creds_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join(".credentials.json"))
}

fn codex_auth_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".codex").join("auth.json"))
}

/// Estrae `claudeAiOauth.accessToken` (o `accessToken` alla radice) da un
/// payload di credenziali. Una stringa vuota conta come assente: un token
/// svuotato nel Keychain/file (visto in produzione) va trattato come
/// NotLoggedIn, non passato come Bearer vuoto ad Anthropic (che risponde 429
/// invece di 401 e maschera il problema reale dietro un finto rate limit).
fn oauth_access_token(json: &serde_json::Value) -> Option<String> {
    json.get("claudeAiOauth")
        .and_then(|v| v.get("accessToken"))
        .or_else(|| json.get("accessToken"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
}

/// Legge il token OAuth salvato da `claude login` dal Keychain (voce
/// "Claude Code-credentials"). Bug reale già incontrato: possono esistere
/// più voci Keychain omonime (una vecchia con solo `mcpOAuth`, una nuova
/// con `claudeAiOauth`) — `security find-generic-password -s ...` senza
/// `-a <account>` ne prende una a caso, silenziosamente. Se capita, il
/// campo `claudeAiOauth` semplicemente manca nella voce presa: lo
/// trattiamo come NotLoggedIn con un messaggio che indica il sospetto
/// principale, invece di un errore muto.
#[cfg(target_os = "macos")]
pub fn claude_token() -> Result<String, ProviderError> {
    let output = std::process::Command::new("security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .map_err(|e| ProviderError::NotLoggedIn(format!("impossibile eseguire 'security': {e}")))?;

    if !output.status.success() {
        return Err(ProviderError::NotLoggedIn(
            "voce 'Claude Code-credentials' non trovata in Keychain. Esegui 'claude login'."
                .to_string(),
        ));
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
        oauth_access_token(&json).ok_or_else(|| {
            ProviderError::NotLoggedIn(
                "Token OAuth assente o vuoto nel Keychain. Esegui 'claude login' da terminale; \
                 se esistono voci 'Claude Code-credentials' duplicate rimuovi la vecchia da \
                 Keychain Access."
                    .to_string(),
            )
        })
    } else if !raw.is_empty() {
        Ok(raw)
    } else {
        Err(ProviderError::NotLoggedIn("Token OAuth vuoto in Keychain".to_string()))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn claude_token() -> Result<String, ProviderError> {
    let path = claude_creds_path()
        .ok_or_else(|| ProviderError::NotLoggedIn("Home directory non trovata".to_string()))?;
    let raw = fs::read_to_string(&path).map_err(|_| {
        ProviderError::NotLoggedIn(format!(
            "Credenziali non trovate ({}). Esegui 'claude login'.",
            path.display()
        ))
    })?;
    let json: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| ProviderError::NotLoggedIn(format!("Credenziali illeggibili: {e}")))?;
    oauth_access_token(&json).ok_or_else(|| {
        ProviderError::NotLoggedIn(
            "Token OAuth assente o vuoto in .credentials.json. Esegui 'claude login'.".to_string(),
        )
    })
}

pub struct CodexCreds {
    pub access_token: String,
    pub account_id: String,
}

pub fn codex_creds() -> Result<CodexCreds, ProviderError> {
    let path = codex_auth_path()
        .ok_or_else(|| ProviderError::NotLoggedIn("Home directory non trovata".to_string()))?;
    let raw = fs::read_to_string(&path).map_err(|_| {
        ProviderError::NotLoggedIn(format!(
            "Credenziali non trovate ({}). Esegui 'codex login'.",
            path.display()
        ))
    })?;
    let json: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| ProviderError::NotLoggedIn(format!("Credenziali illeggibili: {e}")))?;

    let access_token = json
        .get("tokens")
        .and_then(|t| t.get("access_token"))
        .and_then(|v| v.as_str());
    let account_id = json
        .get("tokens")
        .and_then(|t| t.get("account_id"))
        .and_then(|v| v.as_str());

    match (access_token, account_id) {
        (Some(a), Some(b)) => Ok(CodexCreds {
            access_token: a.to_string(),
            account_id: b.to_string(),
        }),
        _ => Err(ProviderError::NotLoggedIn(
            "Token o account id mancanti in auth.json".to_string(),
        )),
    }
}

/// Ordine di preferenza: `gh auth token` (rispetta il login già fatto), poi
/// $GITHUB_TOKEN/$GH_TOKEN. Lo scope minimo reale non richiede `user` (vedi
/// step-2.1.md) — nessun controllo di scope qui, un eventuale 401 è gestito
/// dal provider a valle.
pub fn copilot_token() -> Result<String, ProviderError> {
    // ponytail: un .app lanciato da Finder/Dock/LaunchAgent eredita il PATH
    // minimo di macOS (/usr/bin:/bin:/usr/sbin:/sbin), non quello della shell
    // interattiva — `gh` installato via Homebrew non ci sta. "gh" nudo resta
    // primo (copre dev da terminale dove il PATH è già giusto), poi i path
    // assoluti noti. Niente `.env("PATH", ...)`: su Unix non è garantito che
    // influenzi la risoluzione del nome del programma passato a Command::new.
    for candidate in ["gh", "/opt/homebrew/bin/gh", "/usr/local/bin/gh"] {
        if let Ok(out) = std::process::Command::new(candidate).args(["auth", "token"]).output() {
            if out.status.success() {
                let t = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !t.is_empty() {
                    return Ok(t);
                }
            }
        }
    }
    std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GH_TOKEN"))
        .ok()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| {
            ProviderError::NotLoggedIn(
                "Nessun token GitHub trovato. Esegui 'gh auth login', o imposta \
                 GITHUB_TOKEN/GH_TOKEN."
                    .to_string(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_token_is_extracted() {
        let json = serde_json::json!({"claudeAiOauth": {"accessToken": "sk-ant-oat01-real"}});
        assert_eq!(oauth_access_token(&json), Some("sk-ant-oat01-real".to_string()));
    }

    #[test]
    fn empty_token_is_rejected() {
        let json = serde_json::json!({"claudeAiOauth": {"accessToken": ""}});
        assert_eq!(oauth_access_token(&json), None);
    }

    #[test]
    fn missing_field_is_rejected() {
        let json = serde_json::json!({"claudeAiOauth": {}});
        assert_eq!(oauth_access_token(&json), None);
    }
}
