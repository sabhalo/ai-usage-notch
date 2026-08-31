const { invoke } = window.__TAURI__.core;
const { getCurrentWindow, LogicalSize } = window.__TAURI__.window;
const { WebviewWindow } = window.__TAURI__.webviewWindow;

const win = getCurrentWindow();

let refreshMs = 300_000; // intervallo normale a provider sano; sovrascritto da settings.json (step-4.6.md)
const TICK_MS = 30_000; // granularità del loop di controllo backoff (= primo gradino)
const BACKOFF_STEPS_S = [30, 300, 900]; // 30s -> 5min -> 15min (cap), per provider
const CACHE_STALE_MS = 10 * 60 * 1000;
// Due timer distinti che collassavano sotto lo stesso nome prima di issue #8:
// questo chiude il Panel (dettaglio provider) dopo inattività.
const PANEL_AUTO_CLOSE_MS = 7000;

// Unica fonte per l'elenco provider: state/everSucceeded/backoff/activeProviders
// (issue #7) erano quattro liste hardcoded da tenere allineate a mano, un
// rischio già con tre provider e peggiore con quattro.
const PROVIDER_IDS = ["claude", "codex", "copilot", "gemini"];
const PROVIDER_TITLES = { claude: "Claude", codex: "Codex", copilot: "Copilot", gemini: "Gemini" };
const UNLIMITED_COLOR = "#8b5cf6"; // viola: distinto dalla scala verde/ambra/rosso, "non applicabile"
const ERROR_COLOR = "#ff4d4d";

const RING_IDS = {
  claude: { pctElId: "claude-pct", ringFgId: "claude-ring-fg", btnId: "claude-btn" },
  codex: { pctElId: "codex-pct", ringFgId: "codex-ring-fg", btnId: "codex-btn" },
  copilot: { pctElId: "copilot-pct", ringFgId: "copilot-ring-fg", btnId: "copilot-btn" },
  gemini: { pctElId: "gemini-pct", ringFgId: "gemini-ring-fg", btnId: "gemini-btn" },
};

const state = Object.fromEntries(PROVIDER_IDS.map((p) => [p, null]));
// true solo dopo la prima fetch riuscita: distingue "non ancora configurato"
// (grigio, mai partito) da "ha funzionato e ora fallisce" (rosso, allarme
// vero) — vedi plan/step-3.5.md.
const everSucceeded = Object.fromEntries(PROVIDER_IDS.map((p) => [p, false]));
const backoff = Object.fromEntries(PROVIDER_IDS.map((p) => [p, { failures: 0, nextAt: 0 }]));
// Per il pulsare sopra soglia (step-4.3.md): ultima percentuale vista (per
// riconoscere un reset di finestra: pct che scende) e se è già partita una
// notifica per il ciclo corrente.
const lastPct = {};
const notifiedThisCycle = {};

let openProvider = null;
let activeProviders = Object.fromEntries(PROVIDER_IDS.map((p) => [p, true]));
let alertThresholdPct = 80;
let panelCloseTimer = null;

// Visibilità della Pill (issue #8): "always" la tiene sempre Visible,
// "auto_collapse" la fa Collassare dopo pillCollapseDelayMs di inattività.
// Vedi CONTEXT.md per i termini Visible/Collapsed/Panel.
let pillVisibilityMode = "always";
let pillCollapseDelayMs = 3000;
let pillCollapsed = false;
let pillCollapseTimer = null;

function pctColor(p) {
  if (p >= 90) return "#ff4d4d";
  if (p >= 70) return "#ffab40";
  return "#3ecf5a";
}

function setRingColor(circleEl, color, pct) {
  const r = 15;
  const c = 2 * Math.PI * r;
  const clamped = Math.max(0, Math.min(100, pct));
  circleEl.style.strokeDasharray = `${c}`;
  circleEl.style.strokeDashoffset = `${c - (clamped / 100) * c}`;
  circleEl.style.stroke = color;
}

function renderRing(provider, report, stale = false) {
  const ids = RING_IDS[provider];
  if (!ids) return;
  const btn = document.getElementById(ids.btnId);
  const pctEl = document.getElementById(ids.pctElId);
  const ringEl = document.getElementById(ids.ringFgId);

  // Un 429 è transitorio (il server ci ha detto di aspettare), non un
  // guasto del provider: se abbiamo già mostrato un dato valido in
  // precedenza, resta in stile "stale" con l'ultima percentuale nota
  // invece del "!" rosso da allarme vero.
  const rateLimited = !!(report && report.error && report.error.kind === "RateLimited");
  const notConfigured = !!(report && report.error && !rateLimited && !everSucceeded[provider]);
  const primary = report && !report.error && report.windows[0];
  const pct = primary ? Math.round(primary.used_percent) : null;
  const unlimited = !!(primary && primary.unlimited);
  const inOverage = !!(primary && primary.in_overage);
  const showStale = stale || (rateLimited && everSucceeded[provider]);

  btn.classList.toggle("overage", inOverage);
  btn.classList.toggle("not-configured", notConfigured);
  btn.classList.toggle("stale", showStale);
  btn.title = (report && report.error && report.error.message) || PROVIDER_TITLES[provider] || provider;

  if (notConfigured) {
    pctEl.textContent = "–";
    btn.classList.remove("alert");
    return;
  }

  if (rateLimited && everSucceeded[provider] && lastPct[provider] !== undefined) {
    const p = Math.round(lastPct[provider]);
    pctEl.textContent = `${p}%`;
    setRingColor(ringEl, pctColor(p), p);
    return;
  }

  pctEl.textContent = report && report.error
    ? "!"
    : unlimited
    ? "∞"
    : pct === null
    ? "—"
    : `${pct}%`;

  if (report && report.error) {
    setRingColor(ringEl, ERROR_COLOR, 100);
  } else if (unlimited) {
    setRingColor(ringEl, UNLIMITED_COLOR, 100);
  } else {
    setRingColor(ringEl, pctColor(pct ?? 0), pct ?? 0);
  }
}

// Pulsazione sopra soglia + una sola notifica di sistema per ciclo (plan/step-4.3.md).
// Il "ciclo" riparte quando la percentuale scende rispetto all'ultima vista:
// è il segnale che la finestra (5h/7g/mensile) si è resettata, senza bisogno
// di un timestamp di reset esposto dal backend.
function checkAlert(provider, report) {
  const btn = document.getElementById(RING_IDS[provider].btnId);
  const primary = report && !report.error && report.windows[0];
  if (!primary || primary.unlimited) {
    btn.classList.remove("alert");
    return;
  }
  const pct = primary.used_percent;
  const prev = lastPct[provider];
  if (prev !== undefined && pct < prev - 1) {
    notifiedThisCycle[provider] = false;
  }
  lastPct[provider] = pct;

  const above = pct >= alertThresholdPct;
  btn.classList.toggle("alert", above);

  if (above && !notifiedThisCycle[provider]) {
    notifiedThisCycle[provider] = true;
    sendThresholdNotification(provider, pct);
  }
}

async function sendThresholdNotification(provider, pct) {
  try {
    if (typeof Notification === "undefined") return;
    if (Notification.permission === "default") await Notification.requestPermission();
    if (Notification.permission === "granted") {
      new Notification(`${PROVIDER_TITLES[provider] || provider} sopra soglia`, {
        body: `Uso al ${Math.round(pct)}% (soglia ${alertThresholdPct}%).`,
      });
    }
  } catch (e) {
    console.warn("notifica fallita:", e);
  }
}

async function loadRuntimeSettings() {
  try {
    const s = await invoke("get_settings");
    alertThresholdPct = s.alert_threshold_pct;
    activeProviders = { ...activeProviders, ...s.active_providers };
    refreshMs = Math.max(30, s.refresh_interval_s) * 1000;
    pillVisibilityMode = s.pill_visibility_mode || "always";
    pillCollapseDelayMs = Math.max(1, s.pill_collapse_delay_s || 3) * 1000;
  } catch (e) {
    console.warn("impostazioni non disponibili, uso i default:", e);
  }
  for (const [provider, active] of Object.entries(activeProviders)) {
    const ids = RING_IDS[provider];
    if (ids) document.getElementById(ids.btnId).hidden = !active;
  }
}

async function loadCachedUsage() {
  let cached;
  try {
    cached = await invoke("get_cached_usage");
  } catch (e) {
    console.warn("cache non disponibile:", e);
    return;
  }
  if (!cached) return;
  const stale = Date.now() - cached.timestamp * 1000 > CACHE_STALE_MS;
  for (const report of cached.reports) {
    state[report.provider] = report;
    if (!report.error) everSucceeded[report.provider] = true;
    renderRing(report.provider, report, stale);
  }
}

function dueProviders(now) {
  return Object.keys(backoff).filter((p) => activeProviders[p] && now >= backoff[p].nextAt);
}

// Un giro di poll: interroga solo i provider "dovuti" (non in backoff, non
// disattivati nelle impostazioni), aggiorna il backoff per provider in base
// al risultato. `force` bypassa il backoff e interroga tutti gli attivi
// (refresh manuale, step-3.4.md).
async function tick(force) {
  const now = Date.now();
  const active = Object.keys(backoff).filter((p) => activeProviders[p]);
  const due = force ? active : dueProviders(now);
  if (due.length === 0) return;
  const skip = Object.keys(backoff).filter((p) => !due.includes(p));

  let reports;
  try {
    reports = await invoke("get_all_usage", { skip });
  } catch (e) {
    console.warn("get_all_usage fallita:", e);
    return;
  }

  for (const report of reports) {
    const p = report.provider;
    state[p] = report;
    if (report.error) {
      if (report.retry_after_s) {
        // Un rate limit non è un guasto del provider: onora il tempo detto
        // dal server e non consuma un gradino di backoff.
        backoff[p].nextAt = now + report.retry_after_s * 1000;
      } else {
        backoff[p].failures += 1;
        const step = BACKOFF_STEPS_S[Math.min(backoff[p].failures - 1, BACKOFF_STEPS_S.length - 1)];
        backoff[p].nextAt = now + step * 1000;
      }
    } else {
      backoff[p].failures = 0;
      backoff[p].nextAt = now + refreshMs;
      everSucceeded[p] = true;
    }
    renderRing(p, report);
    checkAlert(p, report);
  }
  if (openProvider) renderDetail(openProvider);
}

function forceRefreshNow() {
  for (const p of Object.keys(backoff)) backoff[p].nextAt = 0;
  tick(true);
}

function renderDetail(provider) {
  const data = state[provider];
  const title = document.getElementById("detail-title");
  const rows = document.getElementById("detail-rows");
  const errBox = document.getElementById("detail-error");

  title.textContent = PROVIDER_TITLES[provider] || provider;
  rows.innerHTML = "";
  errBox.hidden = true;
  delete errBox.dataset.kind;

  if (!data || data.error) {
    errBox.hidden = false;
    errBox.textContent = (data && data.error && data.error.message) || "Nessun dato disponibile";
    if (data && data.error) errBox.dataset.kind = data.error.kind;
    return;
  }

  for (const w of data.windows) {
    const row = document.createElement("div");
    row.className = "detail-row";
    const barWidth = w.unlimited ? 100 : Math.min(100, w.used_percent);
    const barColor = w.unlimited ? UNLIMITED_COLOR : pctColor(w.used_percent);
    const pctLabel = w.unlimited ? "illimitato" : `${Math.round(w.used_percent)}% usato`;
    const overageBadge = w.in_overage ? '<span class="badge-overage">overage</span>' : "";
    row.innerHTML = `
      <div class="detail-row-top">
        <span>${w.label} ${overageBadge}</span>
        <span>Reset: ${w.resets_in}</span>
      </div>
      <div class="bar"><div class="bar-fill" style="width:${barWidth}%; background:${barColor}"></div></div>
      <div class="detail-row-pct">${pctLabel}</div>
    `;
    rows.appendChild(row);
  }
}

// Unico punto che traduce i due assi indipendenti (Panel open/closed, Pill
// visible/collapsed — vedi CONTEXT.md) in dimensioni reali della finestra.
// Il Panel aperto forza sempre la Pill Visible: non esiste uno stato
// Collapsed+Panel aperto.
// La geometria (mock Claude Design) non è più hardcoded: si misura il DOM
// dopo l'applicazione delle classi, invece di duplicare a mano le dimensioni
// di .pill/.detail qui e in tauri.conf.json.
async function applyLayout() {
  const panelOpen = !!openProvider;
  const collapsed = pillCollapsed && !panelOpen;
  document.getElementById("detail").hidden = !panelOpen;
  document.querySelector(".pill").classList.toggle("pill-collapsed", collapsed);

  await new Promise(requestAnimationFrame);
  const box = document.querySelector(".notch");
  try {
    await win.setSize(new LogicalSize(Math.ceil(box.offsetWidth), Math.ceil(box.offsetHeight)));
  } catch (e) {
    // Se l'API finestra non è disponibile (vedi README), il pannello si apre
    // comunque: resta solo il piccolo overflow visivo da correggere a mano.
    console.warn("setSize non disponibile:", e);
  }
}

function closeProvider() {
  if (!openProvider) return;
  document.getElementById(RING_IDS[openProvider].btnId).classList.remove("open");
  openProvider = null;
}

function schedulePanelAutoClose() {
  clearTimeout(panelCloseTimer);
  if (!openProvider) return;
  panelCloseTimer = setTimeout(() => {
    closeProvider();
    applyLayout();
    schedulePillCollapse();
  }, PANEL_AUTO_CLOSE_MS);
}

// Sospeso mentre il Panel è aperto (issue #8, punto 4): niente collasso della
// Pill sotto un dettaglio che l'utente sta guardando.
function schedulePillCollapse() {
  clearTimeout(pillCollapseTimer);
  if (pillVisibilityMode !== "auto_collapse" || openProvider) return;
  pillCollapseTimer = setTimeout(() => {
    pillCollapsed = true;
    applyLayout();
  }, pillCollapseDelayMs);
}

function wakePill() {
  if (pillCollapsed) {
    pillCollapsed = false;
    applyLayout();
  }
  schedulePillCollapse();
}

function toggleProvider(provider) {
  const wasOpen = openProvider === provider;
  closeProvider();
  if (wasOpen) {
    clearTimeout(panelCloseTimer);
    applyLayout();
    schedulePillCollapse();
    return;
  }
  openProvider = provider;
  document.getElementById(RING_IDS[provider].btnId).classList.add("open");
  renderDetail(provider);
  clearTimeout(pillCollapseTimer);
  applyLayout();
  schedulePanelAutoClose();
}

async function openSettingsWindow() {
  try {
    const settingsWin = await WebviewWindow.getByLabel("settings");
    if (settingsWin) {
      await settingsWin.show();
      await settingsWin.setFocus();
    }
  } catch (e) {
    console.warn("impossibile aprire le impostazioni:", e);
  }
}

document
  .getElementById("claude-btn")
  .addEventListener("click", () => toggleProvider("claude"));
document
  .getElementById("codex-btn")
  .addEventListener("click", () => toggleProvider("codex"));
document
  .getElementById("copilot-btn")
  .addEventListener("click", () => toggleProvider("copilot"));
document
  .getElementById("gemini-btn")
  .addEventListener("click", () => toggleProvider("gemini"));
document
  .getElementById("settings-btn")
  .addEventListener("click", (e) => {
    e.stopPropagation();
    openSettingsWindow();
  });

document.getElementById("detail").addEventListener("pointermove", schedulePanelAutoClose);
document.getElementById("detail").addEventListener("click", schedulePanelAutoClose);
win.onFocusChanged(({ payload: focused }) => {
  if (!focused && openProvider) {
    closeProvider();
    clearTimeout(panelCloseTimer);
    applyLayout();
    schedulePillCollapse();
  }
});

// Refresh manuale: click destro sulla pill forza subito tutti i provider,
// bypassando il backoff (plan/step-3.4.md). Scelto un listener JS invece di
// un vero menu contestuale nativo Tauri: stesso risultato, zero codice Rust
// in più.
const pillEl = document.querySelector(".pill");
pillEl.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  forceRefreshNow();
});

// Sveglia la Pill (torna Visible) al passaggio del mouse; se la modalità è
// "auto_collapse" riarma anche il timer di collasso (issue #8).
pillEl.addEventListener("mouseenter", wakePill);
pillEl.addEventListener("pointermove", wakePill);

// Trascinamento nativo via data-tauri-drag-region sulla pill (index.html);
// qui persistiamo solo la posizione finale, con un debounce per non scrivere
// su disco a ogni pixel di movimento (plan/step-4.1.md).
let dragSaveTimer = null;
win.onMoved(({ payload }) => {
  clearTimeout(dragSaveTimer);
  dragSaveTimer = setTimeout(() => {
    invoke("save_window_position", { x: Math.round(payload.x), y: Math.round(payload.y) }).catch(
      (e) => console.warn("impossibile salvare la posizione:", e)
    );
  }, 400);
});

// La modalità di visibilità va commutabile dalle impostazioni senza
// riavviare (issue #8): riletta a ogni tick invece di un canale di eventi
// dedicato, riusando il polling che il resto dell'app già fa ogni TICK_MS.
async function refreshVisibilitySettings() {
  const prevMode = pillVisibilityMode;
  await loadRuntimeSettings();
  if (pillVisibilityMode === prevMode) return;
  if (pillVisibilityMode !== "auto_collapse") {
    clearTimeout(pillCollapseTimer);
    if (pillCollapsed) {
      pillCollapsed = false;
      applyLayout();
    }
  } else {
    schedulePillCollapse();
  }
}

async function boot() {
  await loadRuntimeSettings();
  await loadCachedUsage();
  await tick(true); // il primo giro è sempre forzato, non aspetta il primo tick
  setInterval(() => {
    refreshVisibilitySettings();
    tick(false);
  }, TICK_MS);
  schedulePillCollapse();
}

boot();
