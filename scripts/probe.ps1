# Fase 0.1 — chiama i tre endpoint di usage con le credenziali locali e
# stampa il JSON grezzo indentato. Non stampa mai un token.

function Invoke-Probe {
    param([string]$Name, [string]$Url, [hashtable]$Headers)
    Write-Host "== $Name =="
    try {
        $resp = Invoke-WebRequest -Uri $Url -Headers $Headers -Method Get -ErrorAction Stop
        Write-Host "HTTP $($resp.StatusCode)"
        try {
            $obj = $resp.Content | ConvertFrom-Json
            $obj | ConvertTo-Json -Depth 10
        } catch {
            Write-Host $resp.Content
        }
    } catch [System.Net.WebException] {
        $r = $_.Exception.Response
        if ($r) {
            Write-Host "HTTP $([int]$r.StatusCode)"
            $stream = New-Object System.IO.StreamReader($r.GetResponseStream())
            Write-Host $stream.ReadToEnd()
        } else {
            Write-Host "richiesta fallita: $($_.Exception.Message)"
        }
    } catch {
        Write-Host "richiesta fallita: $($_.Exception.Message)"
    }
    Write-Host ""
}

# --- Claude: %USERPROFILE%\.claude\.credentials.json ---
$claudeCreds = Join-Path $env:USERPROFILE ".claude\.credentials.json"
if (Test-Path $claudeCreds) {
    $json = Get-Content $claudeCreds -Raw | ConvertFrom-Json
    $token = $json.claudeAiOauth.accessToken
    if (-not $token) { $token = $json.accessToken }
    if ($token) {
        Invoke-Probe -Name "claude" -Url "https://api.anthropic.com/api/oauth/usage" -Headers @{
            "Authorization"  = "Bearer $token"
            "anthropic-beta" = "oauth-2025-04-20"
            "User-Agent"     = "claude-code"
        }
    } else {
        Write-Host "== claude =="
        Write-Host "token non trovato in $claudeCreds"
        Write-Host ""
    }
} else {
    Write-Host "== claude =="
    Write-Host "$claudeCreds non trovato. Esegui 'claude login'."
    Write-Host ""
}

# --- Codex: %USERPROFILE%\.codex\auth.json ---
$codexAuth = Join-Path $env:USERPROFILE ".codex\auth.json"
if (Test-Path $codexAuth) {
    $json = Get-Content $codexAuth -Raw | ConvertFrom-Json
    $token = $json.tokens.access_token
    $account = $json.tokens.account_id
    if ($token -and $account) {
        Invoke-Probe -Name "codex" -Url "https://chatgpt.com/backend-api/wham/usage" -Headers @{
            "Authorization"       = "Bearer $token"
            "ChatGPT-Account-ID"  = $account
        }
    } else {
        Write-Host "== codex =="
        Write-Host "token o account_id mancanti in $codexAuth"
        Write-Host ""
    }
} else {
    Write-Host "== codex =="
    Write-Host "$codexAuth non trovato. Esegui 'codex login'."
    Write-Host ""
}

# --- Copilot: gh auth token > $env:GITHUB_TOKEN/$env:GH_TOKEN > apps.json (VS Code) ---
function Get-CopilotToken {
    $gh = Get-Command gh -ErrorAction SilentlyContinue
    if ($gh) {
        $t = & gh auth token 2>$null
        if ($LASTEXITCODE -eq 0 -and $t) { return $t.Trim() }
    }
    if ($env:GITHUB_TOKEN) { return $env:GITHUB_TOKEN }
    if ($env:GH_TOKEN) { return $env:GH_TOKEN }
    # Fallback non verificato (Fase 0.4): shape del file da confermare.
    $appsJson = Join-Path $env:LOCALAPPDATA "github-copilot\apps.json"
    if (Test-Path $appsJson) {
        try {
            $apps = Get-Content $appsJson -Raw | ConvertFrom-Json
            $first = $apps.PSObject.Properties | Select-Object -First 1
            if ($first) { return $first.Value.oauth_token }
        } catch { return $null }
    }
    return $null
}

$copilotToken = Get-CopilotToken
if ($copilotToken) {
    Invoke-Probe -Name "copilot" -Url "https://api.github.com/copilot_internal/user" -Headers @{
        "Authorization"  = "token $copilotToken"
        "User-Agent"     = "GitHubCopilotChat/0.26.7"
        "Editor-Version" = "vscode/1.96.2"
    }
} else {
    Write-Host "== copilot =="
    Write-Host "nessun token trovato. Esegui 'gh auth login', o imposta GITHUB_TOKEN/GH_TOKEN."
    Write-Host "Se il token esiste ma manca lo scope, prova: gh auth refresh -h github.com -s user"
    Write-Host ""
}
