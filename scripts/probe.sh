#!/usr/bin/env bash
# Fase 0.1 — chiama i tre endpoint di usage con le credenziali locali e
# stampa il JSON grezzo indentato. Non stampa mai un token.
set -uo pipefail

need() { command -v "$1" >/dev/null 2>&1 || { echo "manca '$1' nel PATH" >&2; exit 1; }; }
need curl
need jq

probe() {
    local name="$1" url="$2"; shift 2
    local tmp status
    tmp=$(mktemp)
    status=$(curl -sS -o "$tmp" -w '%{http_code}' "$url" "$@" 2>/dev/null)
    local rc=$?
    echo "== $name =="
    if [[ $rc -ne 0 ]]; then
        echo "richiesta fallita (curl exit $rc)"
    else
        echo "HTTP $status"
        if jq -e . "$tmp" >/dev/null 2>&1; then
            jq '.' "$tmp"
        else
            cat "$tmp"
        fi
    fi
    echo
    rm -f "$tmp"
}

# --- Claude: Keychain su macOS, file su Linux ---
claude_token() {
    if [[ "$(uname)" == "Darwin" ]]; then
        local raw
        raw=$(security find-generic-password -s "Claude Code-credentials" -w 2>/dev/null) || return 1
        if jq -e . >/dev/null 2>&1 <<<"$raw"; then
            jq -r '.claudeAiOauth.accessToken // .accessToken // empty' <<<"$raw"
        else
            printf '%s' "$raw"
        fi
    else
        local creds="$HOME/.claude/.credentials.json"
        [[ -f "$creds" ]] || return 1
        jq -r '.claudeAiOauth.accessToken // .accessToken // empty' "$creds"
    fi
}

CLAUDE_TOKEN=$(claude_token)
if [[ -n "${CLAUDE_TOKEN:-}" ]]; then
    probe "claude" "https://api.anthropic.com/api/oauth/usage" \
        -H "Authorization: Bearer $CLAUDE_TOKEN" \
        -H "anthropic-beta: oauth-2025-04-20" \
        -H "User-Agent: claude-code"
else
    echo "== claude =="
    echo "token non trovato (Keychain 'Claude Code-credentials' su macOS, ~/.claude/.credentials.json altrove). Esegui 'claude login'."
    echo
fi
unset CLAUDE_TOKEN

# --- Codex: ~/.codex/auth.json ---
CODEX_AUTH="$HOME/.codex/auth.json"
if [[ -f "$CODEX_AUTH" ]]; then
    CODEX_TOKEN=$(jq -r '.tokens.access_token // empty' "$CODEX_AUTH")
    CODEX_ACCOUNT=$(jq -r '.tokens.account_id // empty' "$CODEX_AUTH")
    if [[ -n "$CODEX_TOKEN" && -n "$CODEX_ACCOUNT" ]]; then
        probe "codex" "https://chatgpt.com/backend-api/wham/usage" \
            -H "Authorization: Bearer $CODEX_TOKEN" \
            -H "ChatGPT-Account-ID: $CODEX_ACCOUNT"
    else
        echo "== codex =="
        echo "token o account_id mancanti in $CODEX_AUTH"
        echo
    fi
    unset CODEX_TOKEN CODEX_ACCOUNT
else
    echo "== codex =="
    echo "$CODEX_AUTH non trovato. Esegui 'codex login'."
    echo
fi

# --- Copilot: gh auth token > $GITHUB_TOKEN / $GH_TOKEN ---
copilot_token() {
    if command -v gh >/dev/null 2>&1; then
        local t
        t=$(gh auth token 2>/dev/null) && [[ -n "$t" ]] && { printf '%s' "$t"; return 0; }
    fi
    [[ -n "${GITHUB_TOKEN:-}" ]] && { printf '%s' "$GITHUB_TOKEN"; return 0; }
    [[ -n "${GH_TOKEN:-}" ]] && { printf '%s' "$GH_TOKEN"; return 0; }
    return 1
}

COPILOT_TOKEN=$(copilot_token)
if [[ -n "${COPILOT_TOKEN:-}" ]]; then
    probe "copilot" "https://api.github.com/copilot_internal/user" \
        -H "Authorization: token $COPILOT_TOKEN" \
        -H "User-Agent: GitHubCopilotChat/0.26.7" \
        -H "Editor-Version: vscode/1.96.2"
else
    echo "== copilot =="
    echo "nessun token trovato. Esegui 'gh auth login', o imposta GITHUB_TOKEN/GH_TOKEN."
    echo "Se il token esiste ma manca lo scope, prova: gh auth refresh -h github.com -s user"
    echo
fi
unset COPILOT_TOKEN
