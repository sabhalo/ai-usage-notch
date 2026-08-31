# Gemini CLI — ricerca su quota lato server

Ricerca svolta il **2026-08-30** da sorgenti primarie (codice sorgente + docs
ufficiali).

> **AGGIORNAMENTO 2026-08-30, dopo la stesura — la probe live è stata fatta.**
> Il documento sotto è stato scritto quando `gemini` non era ancora installato.
> Nel frattempo l'utente ha installato la CLI (v0.57.0) e completato il login,
> e le due chiamate sono state eseguite davvero. **Leggere prima la sezione 0**:
> l'endpoint esiste ed è raggiungibile come descritto qui, ma per questo account
> risponde **403**. Le sezioni 1-7 restano valide sul *meccanismo*; ogni frase
> che dice "nessuna probe live" o "la CLI non è installata" è superata.

Clone di riferimento: `google-gemini/gemini-cli` @
**`0bd1d439751478771c45d3d0895a6a9760554bf4`** (clonato il 2026-08-30, versione
in `package.json`: `0.59.0-nightly.20260825.g812f7a2bc`). I permalink in questo
documento puntano a quello sha. L'ultima release stabile al momento della
ricerca è **v0.57.0** (2026-08-25) e contiene già tutto quanto descritto qui
(verificato via `gh api .../contents/...?ref=v0.57.0`).

---

## 0. Probe live — l'endpoint esiste, ma questo account riceve 403

Eseguita il 2026-08-30 con l'`access_token` reale da `~/.gemini/oauth_creds.json`
(token valido, 52 min alla scadenza). Nessun valore di token letto o registrato.

### Chiamata 1 — `POST v1internal:loadCodeAssist` → **HTTP 200**

L'endpoint risponde e la meccanica descritta in questo documento è confermata.
Ma il corpo dice che il tier gratuito non è più servito:

```json
{
  "allowedTiers": [
    { "id": "standard-tier", "name": "Gemini Code Assist",
      "userDefinedCloudaicompanionProject": true, "usesGcpTos": true, "isDefault": true }
  ],
  "ineligibleTiers": [
    { "tierId": "free-tier", "tierName": "Gemini Code Assist for individuals",
      "reasonCode": "UNSUPPORTED_CLIENT",
      "reasonMessage": "This client is no longer supported ... please migrate to the Antigravity suite of products" }
  ]
}
```

**Nessun `cloudaicompanionProject` nella risposta.** È il campo che la chiamata 2
richiede, e la sua assenza qui è il vero punto di rottura: senza tier ammesso il
server non assegna nessun progetto.

### Chiamata 2 — `POST v1internal:retrieveUserQuota` → **HTTP 403**

```json
{ "error": { "code": 403, "status": "PERMISSION_DENIED",
    "message": "You do not have a valid license of this product. ... (#3501)",
    "details": [ { "reason": "SUBSCRIPTION_REQUIRED",
                   "domain": "cloudaicompanion.googleapis.com" } ] } }
```

### Cosa cambia rispetto al resto del documento

| affermazione | esito della probe |
|---|---|
| l'endpoint di quota esiste | **confermata** |
| le credenziali stanno in `~/.gemini/oauth_creds.json` | **confermata** — scritte dal login della CLI, non da Antigravity |
| servono due chiamate, `loadCodeAssist` poi `retrieveUserQuota` | **confermata** |
| lo shape della risposta di quota (`buckets[].remainingFraction`) | **ancora non osservato**: il 403 arriva prima. Resta inferito dai tipi TypeScript |

### Conseguenza pratica

L'unico tier ammesso è **Code Assist Standard**, che con
`userDefinedCloudaicompanionProject: true` e `usesGcpTos: true` richiede un
progetto GCP e una licenza assegnata — **non** è un abbonamento consumer tipo
Google AI Pro. Il tier gratuito è stato spostato su Antigravity.

Quindi: l'implementazione descritta in questo documento è corretta e
riutilizzabile *se e quando* l'account avrà una licenza Code Assist. Per un
account personale senza licenza non c'è nessuna quota da leggere qui.

---

## 1. Risposta in due righe

**Sì, esiste un endpoint di quota lato server**:
`POST https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuota`,
autenticato con il Bearer OAuth che la CLI scrive in `~/.gemini/oauth_creds.json`,
che restituisce per ogni modello una **frazione di quota rimanente** (`remainingFraction`,
0-1) e un **istante di reset** (`resetTime`, ISO 8601 UTC) — cioè esattamente le
due grandezze che servono a `UsageWindow`.
Vale però **solo per l'auth mode "Login with Google"** (Code Assist): con API key
Gemini o Vertex AI questo endpoint non è nel percorso di codice e non esiste
equivalente.

---

## 2. Credenziali

### Modalità di auth — non sono equivalenti

`AuthType` in `packages/core/src/core/contentGenerator.ts`; il factory che decide
è `createContentGenerator` (righe ~270-395).

| Modalità | Credenziale | Endpoint quota disponibile |
| :-- | :-- | :-- |
| `LOGIN_WITH_GOOGLE` (account Google personale/Workspace) | OAuth2 su file/keychain | **Sì**, `retrieveUserQuota` |
| `COMPUTE_ADC` (ADC su GCE/Cloud Shell) | ADC | **Sì**, stesso percorso Code Assist |
| `USE_GEMINI` (API key) | `GEMINI_API_KEY` env | **No** — va diretto a `generativelanguage`, nessuna chiamata Code Assist |
| `USE_VERTEX_AI` | API key / ADC Vertex | **No** |

Solo `LOGIN_WITH_GOOGLE` e `COMPUTE_ADC` costruiscono un `CodeAssistServer`, ed è
lì che vive `retrieveUserQuota`:
[`codeAssist.ts:15-40`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/codeAssist.ts#L15-L40).
Per il nostro caso d'uso (notch, utente che fa `gemini` login normale) la
modalità rilevante è `LOGIN_WITH_GOOGLE`.

### Dove finisce il token — attenzione, il default NON è il Keychain

Ci sono **due** backend, scelti da una env var:

```
GEMINI_FORCE_ENCRYPTED_FILE_STORAGE=true  → Keychain macOS
(non impostata, DEFAULT)                  → file ~/.gemini/oauth_creds.json
```

Il flag è letto da `getUseEncryptedStorageFlag()`
([`oauth2.ts:112-114`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/oauth2.ts#L112-L114)),
la costante è
[`token-storage/index.ts:13-14`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/mcp/token-storage/index.ts#L13-L14).
Il ramo di scrittura è nel listener `client.on('tokens', ...)`
([`oauth2.ts:143-151`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/oauth2.ts#L143-L151)):
se il flag è off chiama `cacheCredentials(tokens)`, che è
[`oauth2.ts:799-810`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/oauth2.ts#L799-L810):

```ts
async function cacheCredentials(credentials: Credentials) {
  const filePath = Storage.getOAuthCredsPath();
  await fs.mkdir(path.dirname(filePath), { recursive: true });
  const credString = JSON.stringify(credentials, null, 2);
  await fs.writeFile(filePath, credString, { mode: 0o600 });
  ...
}
```

Percorso esatto: `Storage.getOAuthCredsPath()` = `~/.gemini/oauth_creds.json`
(`GEMINI_DIR = '.gemini'` in
[`utils/paths.ts:13`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/utils/paths.ts#L13),
`OAUTH_FILE = 'oauth_creds.json'` in
[`config/storage.ts:22`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/storage.ts#L22),
join in
[`storage.ts:206-208`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/storage.ts#L206-L208)).

Il contenuto è serializzato tal quale dall'oggetto `Credentials` di
`google-auth-library`, quindi le chiavi sono **snake_case**:

```json
{
  "access_token":  "ya29....",
  "refresh_token": "1//0....",
  "scope":         "https://www.googleapis.com/auth/cloud-platform ...",
  "token_type":    "Bearer",
  "id_token":      "eyJ...",
  "expiry_date":   1756557060000
}
```

Il campo da usare come Bearer è **`access_token`**; `expiry_date` è un epoch in
**millisecondi**.

> Riscontro locale (corroborante, **non** attribuibile alla gemini-cli): su
> questa macchina esiste `~/.gemini/oauth_creds.json` con permessi `0600` e le
> chiavi top-level, lette con `jq 'keys'` senza guardare i valori, sono
> esattamente `access_token, expiry_date, id_token, refresh_token, scope,
> token_type`. Il file è però scritto da Antigravity (in `~/.gemini` c'è una
> dir `antigravity/`), che condivide la stessa home dir: conferma lo shape,
> non prova che sia stata la gemini-cli a scriverlo.

Il ramo Keychain (solo con la env var attiva) usa service `gemini-cli-oauth`
account `main-account`, e il valore è un JSON **diverso**, camelCase e annidato
(`{serverName, token:{accessToken, refreshToken, expiresAt, tokenType, scope}, updatedAt}`):
[`oauth-credential-storage.ts:16-22, 64-87`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/oauth-credential-storage.ts#L16-L22)
e
[`token-storage/types.ts:10-28`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/mcp/token-storage/types.ts#L10-L28).
Un client Rust dovrebbe leggere il file e, se assente, provare il Keychain.

### Refresh del token

`access_token` Google dura ~1h, quindi un poller deve saper rinfrescare. La CLI
usa il classico installed-app flow con client id/secret **hardcoded in chiaro nel
sorgente** (gli autori lo dichiarano esplicitamente lecito nel commento),
[`oauth2.ts:75-92`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/oauth2.ts#L75-L92):

```
client_id     = <redatto, vedi sorgente linkato sopra>
client_secret = <redatto, vedi sorgente linkato sopra>
scopes        = cloud-platform, userinfo.email, userinfo.profile
```

Il refresh è quello standard di `google-auth-library`
(`POST https://oauth2.googleapis.com/token`, `grant_type=refresh_token`) —
**inferito**, non ho letto la riga che compone quella richiesta perché sta dentro
la dipendenza, non nel repo.

### Altri file di identità

- `~/.gemini/google_accounts.json` — email dell'account (`{active, old}`).
  Costante in [`utils/paths.ts:14`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/utils/paths.ts#L14),
  path in [`storage.ts:86-88`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/storage.ts#L86-L88).
  Serve solo per la label, non per l'auth.
- `~/.gemini/installation_id` — UUID anonimo per la telemetria
  ([`storage.ts:82-84`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/storage.ts#L82-L84)).

---

## 3. Endpoint di quota

### Spec

```
POST https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuota
Authorization: Bearer <oauth_creds.json .access_token>
Content-Type: application/json
User-Agent: GeminiCLI/<ver>/<model> (<platform>; <arch>; <surface>)   # opzionale, vedi sotto

{ "project": "<cloudaicompanionProject>" }
```

- Base URL: `CODE_ASSIST_ENDPOINT` + `/` + `CODE_ASSIST_API_VERSION`, override
  possibile via env `CODE_ASSIST_ENDPOINT` / `CODE_ASSIST_API_VERSION`
  ([`server.ts:73-74`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/server.ts#L73-L74),
  [`server.ts:524-534`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/server.ts#L524-L534)).
  Il metodo si concatena con **due punti**: `${base}:${method}`.
- Il metodo è `retrieveUserQuota`
  ([`server.ts:367-374`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/server.ts#L367-L374)).
- L'`Authorization` non è messo a mano: lo inietta `AuthClient.request()` di
  `google-auth-library`. Gli unici header espliciti sono `Content-Type` e quelli
  di `httpOptions.headers`, che contengono un `User-Agent`
  ([`contentGenerator.ts:280-283`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/core/contentGenerator.ts#L280-L283)).
  Non ho verificato se il server lo pretenda.

### Il campo `project`

È il **`cloudaicompanionProject`** dell'utente, non il nome di un progetto GCP
scelto da noi. Per l'utente free-tier è un progetto gestito da Google, assegnato
al primo login. Ordine di risoluzione
([`setup.ts:124-305`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/setup.ts#L124-L305)):

1. env `GOOGLE_CLOUD_PROJECT` / `GOOGLE_CLOUD_PROJECT_ID`, se impostate;
2. altrimenti `POST /v1internal:loadCodeAssist` con body
   `{ metadata: { ideType: 'IDE_UNSPECIFIED', platform: 'PLATFORM_UNSPECIFIED', pluginType: 'GEMINI' } }`
   e si legge `cloudaicompanionProject` dalla risposta;
3. se manca del tutto, si passa da `onboardUser` (LRO con polling a 5s).

**Questo id non viene mai scritto su disco.** La cache di `setupUser` è una
`WeakMap` in memoria con TTL 30s
([`setup.ts:81-86, 138-147`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/setup.ts#L81-L86)),
e ho cercato senza trovare alcun percorso che lo persista. Conseguenza pratica
per noi: **il provider Rust deve fare due chiamate**, `loadCodeAssist` e poi
`retrieveUserQuota`.

Nota di dettaglio: in produzione `config.ts` passa l'id **nudo**
(`project: codeAssistServer.projectId`,
[`config.ts:2313-2315`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/config.ts#L2313-L2315)),
mentre il test unitario usa la forma `"projects/my-cloudcode-project"`. Prevale
il codice di produzione: id nudo.

### Shape della risposta

Definizione TypeScript,
[`code_assist/types.ts:250-265`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/types.ts#L250-L265):

```ts
export interface RetrieveUserQuotaRequest {
  project: string;
  userAgent?: string;
}

export interface BucketInfo {
  remainingAmount?: string;    // int64 come stringa
  remainingFraction?: number;  // 0.0 - 1.0
  resetTime?: string;          // ISO 8601 UTC
  tokenType?: string;
  modelId?: string;
}

export interface RetrieveUserQuotaResponse {
  buckets?: BucketInfo[];
}
```

Tutti i campi sono opzionali. L'unico esempio concreto di valori è il fixture nel
test upstream
([`server.test.ts:688-712`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/server.test.ts#L688-L712)) —
è un mock scritto a mano dai manutentori, **non** una risposta reale:

```json
{ "buckets": [ { "modelId": "gemini-2.5-pro",
                 "tokenType": "REQUESTS",
                 "remainingFraction": 0.75,
                 "resetTime": "2025-10-22T16:01:15Z" } ] }
```

`tokenType: "REQUESTS"` è coerente con i limiti documentati (§5), che sono
conteggi di richieste, non di token.

Il codice difensivo di `refreshUserQuota`
([`config.ts:2307-2371`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/config.ts#L2307-L2371))
ci dice cosa il server può realisticamente omettere: scarta i bucket senza
`modelId` o senza `remainingFraction`, e gestisce esplicitamente il caso
"`remainingFraction` presente ma `remainingAmount` assente" normalizzando su
scala 100. Quindi **`remainingFraction` è il campo affidabile**, `remainingAmount`
no.

### Chi lo chiama nella CLI

Non è codice morto:

- all'inizializzazione del content generator, se c'è un projectId
  ([`config.ts:1619-1621`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/config.ts#L1619-L1621));
- dopo **ogni** risposta del modello, con debounce a 30s via
  `refreshUserQuotaIfStale()`
  ([`loggingContentGenerator.ts:420-423` e `571-574`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/core/loggingContentGenerator.ts#L420-L423));
- da `/stats`
  ([`statsCommand.ts:65-79`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/cli/src/ui/commands/statsCommand.ts#L65-L79));
- da `/model`
  ([`modelCommand.ts:57`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/cli/src/ui/commands/modelCommand.ts#L57)).

Il rendering è `ModelQuotaDisplay.tsx`, che è letteralmente la stessa idea del
nostro anello
([`ModelQuotaDisplay.tsx:173-183`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/cli/src/ui/components/ModelQuotaDisplay.tsx#L173-L183)):

```ts
const usedFraction = 1 - data.remainingFraction;
const usedPercentage = usedFraction * 100;
```

con soglie colore a 80% (warning) e 100% (error)
([righe 76-81](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/cli/src/ui/components/ModelQuotaDisplay.tsx#L76-L81)),
e i bucket raggruppati per **tier di modello** (`pro` / `flash` / `flash-lite`),
tenendo per ogni tier il bucket con `remainingFraction` più bassa
([righe 144-171](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/cli/src/ui/components/ModelQuotaDisplay.tsx#L144-L171)).

### Mappatura su `UsageWindow`

Una `UsageWindow` per tier di modello, replicando il raggruppamento della CLI:

| Campo | Sorgente |
| :-- | :-- |
| `label` | nome del tier: `"Pro (giornaliera)"`, `"Flash (giornaliera)"` |
| `used_percent` | `(1.0 - bucket.remainingFraction) * 100.0` |
| `resets_in` | `bucket.resetTime` (ISO 8601 UTC) → durata, **stesso parser di `src/providers/claude.rs`** (anche Claude usa `resets_at` assoluto) |
| `unlimited` | `false` sempre — nessun campo lo segnala. Bucket assente = *ignoto*, non illimitato |
| `in_overage` | `false` — `retrieveUserQuota` non lo espone (vedi sotto) |

### `in_overage`: c'è un segnale, ma è altrove

L'analogo dell'overage Copilot esistono ma vive in un'API diversa: i **Google One
AI credits**. `LoadCodeAssistResponse.paidTier.availableCredits[]`
(`{creditType: 'GOOGLE_ONE_AI', creditAmount: "<int64 string>"}`,
[`types.ts:46-65, 93-105`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/types.ts#L46-L65)),
e in streaming il server rimanda `consumedCredits` / `remainingCredits` dentro le
risposte di `streamGenerateContent`
([`server.ts:163-179`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/server.ts#L163-L179)).
Un saldo crediti è ottenibile a costo zero dalla stessa `loadCodeAssist` che
dobbiamo già fare per il projectId, ed è un **numero assoluto**, non una
percentuale: si potrebbe mostrare come testo, non come anello. Il consumo di
crediti scatta quando la quota è esaurita, quindi
`creditBalance < saldo_iniziale` è un proxy ragionevole di "in overage" —
ma è una **inferenza mia**, non c'è un flag `in_overage` da nessuna parte.

### Gestione errori → enum tipizzato

`requestPost` ritenta 3 volte su `429`, `499`, `5xx`
([`server.ts:431-440`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/server.ts#L431-L440)).
Mappatura consigliata: file/keychain assente o `refresh_token` mancante →
`NotLoggedIn`; `401`/`403` → `Unauthorized`; `429` → `RateLimited`;
`buckets` assente o vuoto → `UnexpectedShape`. Nota che `loadCodeAssist` può
tirare `403 PERMISSION_DENIED` e un errore `VALIDATION_REQUIRED` per account che
richiedono verifica
([`setup.ts:328-352`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/setup.ts#L328-L352)) —
entrambi vanno a `Unauthorized`.

### Cosa c'è *oltre* all'endpoint: fallback reattivo

Esiste anche, in parallelo, la gestione **reattiva** che il brief ipotizzava
essere l'unica cosa presente. C'è, ma non è l'unica:
`packages/core/src/utils/googleQuotaErrors.ts` classifica i `429/499/503` in
`TerminalQuotaError` / `RetryableQuotaError` leggendo `error.details[].reason`
(`QUOTA_EXHAUSTED`, `RATE_LIMIT_EXCEEDED`, `MODEL_CAPACITY_EXHAUSTED`,
`INSUFFICIENT_G1_CREDITS_BALANCE`), e `packages/core/src/fallback/handler.ts` +
`packages/core/src/availability/` fanno il downgrade Pro→Flash. Questo percorso
**non produce percentuali**: serve solo a decidere il fallback. Le percentuali
vengono unicamente da `retrieveUserQuota`.

Da notare: `Config.setQuota()`
([`config.ts:2036-2053`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/config.ts#L2036-L2053))
è **senza chiamanti** in tutto il repo (cercato con grep su `packages/`,
escludendo i test). Quindi non esiste una seconda sorgente di quota, tipo header
HTTP di risposta: `retrieveUserQuota` è l'unica.

---

## 4. Cosa persiste in locale

| Percorso | Formato | Attivo di default | Utile per contare? |
| :-- | :-- | :-- | :-- |
| `~/.gemini/oauth_creds.json` | JSON, `0600` | **sì** (default) | no, solo credenziale |
| Keychain `gemini-cli-oauth` / `main-account` | JSON camelCase | **no**, solo con `GEMINI_FORCE_ENCRYPTED_FILE_STORAGE=true` | no |
| `~/.gemini/google_accounts.json` | `{active, old}` | sì | no |
| `~/.gemini/installation_id` | UUID | sì | no |
| `~/.gemini/settings.json` | JSON | sì | no |
| `~/.gemini/projects.json` | registry slug→path progetto | sì | no |
| `~/.gemini/tmp/<slug>/chats/session-<ts>-<id>.jsonl` | JSONL, un record per messaggio | **sì, sempre** | **parzialmente** |
| `~/.gemini/tmp/<slug>/logs.json` | array di `LogEntry` | sì | solo prompt utente |
| `~/.gemini/tmp/<slug>/shell_history` | testo | sì | no |
| `~/.gemini/history/<slug>/` | — | sì | no |

### `chats/*.jsonl` — l'unico artefatto con numeri d'uso

Registrato senza condizioni: `ChatRecordingService` è istanziato nel costruttore
di `GeminiChat`
([`geminiChat.ts:424`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/core/geminiChat.ts#L424)),
non dietro un setting. Path costruito in
[`chatRecordingService.ts:479-519`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/services/chatRecordingService.ts#L479-L519).
Ogni messaggio di tipo `gemini` porta
([`chatRecordingTypes.ts:19-26, 70-80`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/services/chatRecordingTypes.ts#L19-L26)):

```ts
tokens?: { input, output, cached, thoughts?, tool?, total }  // conteggi token
model?: string
timestamp: string
```

Un contatore costruito su questi file catturerebbe: numero di risposte del
modello per progetto, per modello, nel tempo, e i token. **Non** catturerebbe:
il denominatore (la quota assegnata), la finestra di reset, le richieste fatte
da altri client Code Assist con lo stesso account (IDE, Antigravity, web), né
la nozione di "model request" usata dal server per il conteggio quota — che non
coincide necessariamente con un record `gemini` nel JSONL, visto che la doc
avverte che *"one prompt might result in multiple model requests"*. Un contatore
locale sarebbe quindi sistematicamente sfasato. Dato che l'endpoint server
esiste, non ha senso costruirlo.

### `/stats` — da dove prende davvero i dati

Due sorgenti distinte, e vale la pena separarle:

- **Statistiche di sessione** (durata, token della sessione): stato React in
  memoria, `SessionStatsProvider` /
  `useState<SessionStatsState>({sessionStartTime: new Date(), ...})`
  ([`SessionContext.tsx:197-236`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/cli/src/ui/contexts/SessionContext.tsx#L197-L236)).
  Muore col processo, non è su disco.
- **Percentuali di quota**: chiamata di rete `refreshUserQuota()` fatta al
  momento
  ([`statsCommand.ts:65-79`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/cli/src/ui/commands/statsCommand.ts#L65-L79)).

Quindi le percentuali di `/stats` **non sono calcolate localmente**: sono
rilette dal server ad ogni invocazione. È la conferma più diretta che la strada
giusta è l'endpoint.

### Telemetria OTEL e Clearcut

- OTEL: `telemetry` in `settings.json` ha `default: undefined`
  ([`settingsSchema.ts:982-991`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/cli/src/config/settingsSchema.ts#L982-L991)),
  cioè **spenta di default**; scrive su file solo se si imposta
  `telemetry.target: 'file'` + `telemetry.outfile`
  (`TelemetrySettings` in
  [`config.ts:215-225`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/config.ts#L215-L225)).
  Inutilizzabile come sorgente affidabile.
- Clearcut: `usageStatisticsEnabled` ha default **`true`**
  ([`config.ts:1097`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/config.ts#L1097)),
  ma spedisce a Google e **non lascia nulla sul disco**. Zero valore per noi.

---

## 5. Finestre di quota sensate

### Limiti documentati (oggi)

Doc ufficiale in-repo,
[`docs/resources/quota-and-pricing.md`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/docs/resources/quota-and-pricing.md):

| Auth | Tier | Max richieste / utente / **giorno** |
| :-- | :-- | :-- |
| Google account | Code Assist (Individual) | 1.000 |
| | Google AI Pro | 1.500 |
| | Google AI Ultra | 2.000 |
| Gemini API key | Free | 250 |
| | Pay-as-you-go | variabile |
| Vertex AI | Express / PAYG | variabile |
| Workspace | Code Assist Standard | 1.500 |
| | Code Assist Enterprise | 2.000 |
| | Workspace AI Ultra | 2.000 |

Sul **per-minuto**: il valore "60 req/min" citato spesso **non compare più**.
La doc dice solo, senza numero: *"Requests are limited per user per minute and
are subject to the availability of the service in times of high demand"*
(riga 38 del file sopra). Stessa formulazione, sempre senza numero, sulla pagina
first-party Google Cloud (che oggi è
`https://docs.cloud.google.com/gemini/docs/quotas` — il vecchio
`developers.google.com/gemini-code-assist/resources/quotas` fa `301` lì), la
quale pubblica solo:

| Quota | Edition | Value |
| :-- | :-- | :-- |
| Maximum requests per user per day | Standard | 1500 |
| Maximum requests per user per day | Enterprise | 2000 |

E `https://ai.google.dev/gemini-api/docs/rate-limits` oggi **non pubblica più
numeri**: dice che i limiti dipendono dal tier e rimanda alla dashboard di AI
Studio, elencando solo le dimensioni (RPM, TPM, RPD). Nessuna delle tre pagine
documenta un endpoint per interrogare la quota residua: `retrieveUserQuota` resta
non documentato, esattamente come i tre endpoint già implementati nel repo.

### Cosa mettere nell'anello

Le finestre sensate sono **quelle che il server stesso decide**, non quelle che
scegliamo noi. `retrieveUserQuota` restituisce un bucket per modello con il suo
`resetTime`: non serve indovinare se la finestra sia oraria o giornaliera, la si
legge. Dai numeri documentati la finestra effettiva è **giornaliera**.

Proposta concreta: **due `UsageWindow`**, `Pro` e `Flash`, raggruppate per tier
di modello come fa la CLI. Sono due finestre semanticamente diverse (Pro si
esaurisce prima e fa cadere in fallback su Flash), quindi mostrarle entrambe è
informativo — non è ridondanza.

Il limite per-minuto **non va rappresentato**: non ha un numero pubblicato e non
risulta esposto come bucket separato. Al massimo si manifesta come `429`
`RATE_LIMIT_EXCEEDED` transitorio.

### Sul mismatch percentuale vs conteggio

Il brief lo poneva come rischio principale: gli altri tre provider riportano una
**percentuale di finestra**, mentre un limite a conteggio di richieste è un'altra
cosa che richiederebbe un contatore locale. **Il rischio non si materializza.**
Il server fa già la divisione e restituisce `remainingFraction` come float 0-1
già normalizzato; `remainingAmount` (il conteggio grezzo) è opzionale e la CLI
stessa lo tratta come inaffidabile. Quindi Gemini si allinea al modello degli
altri tre senza bisogno di contatori locali. Se mai un giorno il server mandasse
solo `remainingAmount` senza `remainingFraction`, la CLI scarta il bucket
([`config.ts:2322-2324`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/config/config.ts#L2322-L2324));
faremmo bene a fare lo stesso e restituire `UnexpectedShape`.

Per un account a pagamento l'anello mostrerebbe le stesse percentuali (i tier
Pro/Ultra hanno solo un denominatore più alto), più eventualmente il saldo crediti
Google One come testo accanto.

### Costo di un poll

Due POST per refresh (`loadCodeAssist` + `retrieveUserQuota`), contro il singolo
GET degli altri tre provider. `loadCodeAssist` supporta
`mode: 'HEALTH_CHECK'`, che la CLI usa proprio per i refresh leggeri
([`server.ts:296-313`](https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/packages/core/src/code_assist/server.ts#L296-L313));
in alternativa il projectId si può cachare lato notch, dato che non cambia.
Non ho verificato se `retrieveUserQuota` consumi quota — **presumo di no** (la
CLI lo chiama dopo ogni risposta, con debounce a soli 30s, cosa che sarebbe
assurda se costasse una richiesta).

---

## 6. Verificato vs inferito

### Verificato (letto nel sorgente al commit citato)

- Esiste il metodo `retrieveUserQuota` su `CodeAssistServer`; URL costruito come
  `https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuota`.
- Request `{project, userAgent?}`, response `{buckets: BucketInfo[]}` con
  `remainingAmount` / `remainingFraction` / `resetTime` / `tokenType` / `modelId`,
  tutti opzionali.
- L'endpoint è realmente chiamato: init, dopo ogni risposta del modello (debounce
  30s), `/stats`, `/model`.
- La UI calcola `usedPercentage = (1 - remainingFraction) * 100`.
- `remainingFraction` è il campo su cui la CLI si basa; i bucket che ne sono privi
  vengono scartati.
- `Config.setQuota()` non ha chiamanti: `retrieveUserQuota` è l'unica sorgente di
  quota.
- Default di storage credenziali = **file** `~/.gemini/oauth_creds.json`, `0600`;
  Keychain (`gemini-cli-oauth` / `main-account`) solo con
  `GEMINI_FORCE_ENCRYPTED_FILE_STORAGE=true`.
- Il file è `JSON.stringify` di `Credentials` di `google-auth-library` → chiavi
  snake_case, token in `access_token`, scadenza in `expiry_date` (ms).
- OAuth client id e secret sono hardcoded in `oauth2.ts`, scope
  `cloud-platform` + `userinfo.email` + `userinfo.profile`.
- `cloudaicompanionProject` non è mai persistito su disco; va ottenuto da
  `loadCodeAssist`.
- L'endpoint esiste solo per `LOGIN_WITH_GOOGLE` / `COMPUTE_ADC`, non per API key
  né Vertex.
- La registrazione chat in `~/.gemini/tmp/<slug>/chats/*.jsonl` è incondizionata e
  include `tokens` e `model` per messaggio.
- Telemetria OTEL spenta di default; Clearcut acceso di default ma non scrive su
  disco.
- Le percentuali di `/stats` vengono da una chiamata di rete, non da stato locale;
  le stats di sessione sono stato React in memoria.
- Numeri di quota giornalieri documentati (1.000 / 1.500 / 2.000 / 250) e assenza
  di un numero pubblicato per il limite al minuto.
- La feature è presente anche nell'ultima stabile v0.57.0, non solo in nightly.
- Sul filesystem locale esiste un `~/.gemini/oauth_creds.json` `0600` con
  esattamente le sei chiavi previste — coerente con il sorgente, ma scritto da
  Antigravity, non dalla gemini-cli.

### Inferito / non verificato

- **Non ho mai visto una risposta reale di `retrieveUserQuota`.** L'unico esempio
  di valori è un mock nei test upstream. Nessuna fixture, per costruzione:
  probe live impossibile.
- Non so quanti bucket ritorni in pratica, né se `resetTime` sia sempre presente
  o quale sia la sua granularità reale.
- Non so se il server richieda l'header `User-Agent` o rifiuti un client
  arbitrario.
- Non so se `retrieveUserQuota` consuma quota (presumo di no, per il pattern di
  chiamata della CLI).
- Il meccanismo di refresh del token è quello standard `oauth2.googleapis.com/token`:
  assunto, la riga sta nella dipendenza `google-auth-library`, non nel repo.
- Non ho verificato se il client id/secret embedded siano utilizzabili da un
  processo terzo per il refresh, o se Google leghi il refresh token al client.
- `in_overage` non ha corrispettivo: la proposta basata sul saldo crediti Google
  One è una mia inferenza.
- Non ho stabilito in quale versione esatta sia comparso `retrieveUserQuota`
  (l'issue #13415 "Show model usage limits on /stats" è chiusa il 2025-12-02, il
  che lo colloca intorno a lì, ma non l'ho confermato sul changelog).
- Non ho verificato il comportamento con account Workspace, né con
  `GOOGLE_CLOUD_PROJECT` impostata a mano.

---

## 7. Fonti

### Repo clonato

`google-gemini/gemini-cli` @ `0bd1d439751478771c45d3d0895a6a9760554bf4`, clonato
2026-08-30 in scratchpad (non nel repo). Permalink base:
`https://github.com/google-gemini/gemini-cli/blob/0bd1d439751478771c45d3d0895a6a9760554bf4/`

- `packages/core/src/code_assist/server.ts` — endpoint, `retrieveUserQuota`, retry, crediti
- `packages/core/src/code_assist/types.ts` — `BucketInfo`, `RetrieveUserQuota*`, `LoadCodeAssistResponse`, `GeminiUserTier`
- `packages/core/src/code_assist/server.test.ts` — fixture mock del response
- `packages/core/src/code_assist/setup.ts` — risoluzione `cloudaicompanionProject`, onboarding, errori
- `packages/core/src/code_assist/codeAssist.ts` — quali auth mode arrivano a Code Assist
- `packages/core/src/code_assist/oauth2.ts` — client id/secret/scope, `cacheCredentials`, scelta del backend
- `packages/core/src/code_assist/oauth-credential-storage.ts` — ramo Keychain
- `packages/core/src/mcp/token-storage/{index,types}.ts` — env var, shape camelCase
- `packages/core/src/config/storage.ts` — tutti i path sotto `~/.gemini`
- `packages/core/src/utils/paths.ts` — `GEMINI_DIR`, `GOOGLE_ACCOUNTS_FILENAME`
- `packages/core/src/config/config.ts` — `refreshUserQuota`, `getQuota*`, `setQuota`, `TelemetrySettings`, `usageStatisticsEnabled`
- `packages/core/src/core/loggingContentGenerator.ts` — refresh quota post-risposta
- `packages/core/src/core/contentGenerator.ts` — header `User-Agent`, auth mode
- `packages/core/src/services/chatRecording{Service,Types}.ts` — JSONL sessioni, `TokensSummary`
- `packages/core/src/core/logger.ts` — `logs.json`, `LogEntry`
- `packages/core/src/utils/googleQuotaErrors.ts` — classificazione 429 reattiva
- `packages/cli/src/ui/commands/statsCommand.ts`, `modelCommand.ts`
- `packages/cli/src/ui/components/ModelQuotaDisplay.tsx` — calcolo percentuale, soglie
- `packages/cli/src/ui/utils/formatters.ts` — `formatResetTime`
- `packages/cli/src/ui/contexts/SessionContext.tsx` — stats di sessione in memoria
- `packages/cli/src/config/settingsSchema.ts` — default telemetria
- `docs/resources/quota-and-pricing.md` — tabella limiti

### Web

- <https://docs.cloud.google.com/gemini/docs/quotas> — pagina first-party Google Cloud (il vecchio `https://developers.google.com/gemini-code-assist/resources/quotas` reindirizza `301` qui). Consultata 2026-08-30.
- <https://ai.google.dev/gemini-api/docs/rate-limits> — consultata 2026-08-30: nessun numero pubblicato, rimanda ad AI Studio.
- <https://github.com/google-gemini/gemini-cli/issues/13415> — "Show model usage limits on /stats", chiusa 2025-12-02.
- Lista release: `gh release list --repo google-gemini/gemini-cli` — ultima stabile v0.57.0 (2026-08-25).
- Verifica presenza in stabile: `gh api repos/google-gemini/gemini-cli/contents/packages/core/src/code_assist/{server,types}.ts?ref=v0.57.0`.

### Locale (corroborante, non attribuibile alla gemini-cli)

- `~/.gemini/` su questa macchina: `oauth_creds.json` (`0600`, chiavi lette con
  `jq 'keys'` senza accedere ai valori), `google_accounts.json`,
  `installation_id`, `projects.json`, `tmp/<slug>/{chats/session-*.jsonl,logs.json}`.
  Scritti da Antigravity, che condivide la dir. La gemini-cli non è installata
  (`which gemini` → nulla).
