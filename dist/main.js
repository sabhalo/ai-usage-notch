const { invoke } = window.__TAURI__.core;
const { getCurrentWindow, LogicalSize } = window.__TAURI__.window;

const win = getCurrentWindow();

const REFRESH_MS = 90_000; // 90s: gli endpoint sono non ufficiali, non conviene martellarli
const COLLAPSED_WIDTH = 220;
const COLLAPSED_HEIGHT = 46;
const EXPANDED_EXTRA_HEIGHT = 150; // deve restare coerente con .detail nel CSS

const state = { claude: null, codex: null };
let openProvider = null;

function pctColor(p) {
  if (p >= 90) return "#ef4444";
  if (p >= 70) return "#f59e0b";
  return "#22c55e";
}

function setRing(circleEl, pct) {
  const r = 15.5;
  const c = 2 * Math.PI * r;
  const clamped = Math.max(0, Math.min(100, pct));
  circleEl.style.strokeDasharray = `${c}`;
  circleEl.style.strokeDashoffset = `${c - (clamped / 100) * c}`;
  circleEl.style.stroke = pctColor(clamped);
}

async function refreshProvider(provider, pctElId, ringFgId) {
  try {
    const result = await invoke(
      provider === "claude" ? "get_claude_usage" : "get_codex_usage"
    );
    state[provider] = result;
    const primary = result.windows && result.windows[0];
    const pct = primary ? Math.round(primary.used_percent) : null;
    document.getElementById(pctElId).textContent = pct === null ? "—" : `${pct}%`;
    setRing(document.getElementById(ringFgId), pct ?? 0);
  } catch (e) {
    state[provider] = { provider, windows: [], error: String(e) };
    document.getElementById(pctElId).textContent = "!";
    setRing(document.getElementById(ringFgId), 0);
  }
}

function renderDetail(provider) {
  const data = state[provider];
  const title = document.getElementById("detail-title");
  const rows = document.getElementById("detail-rows");
  const errBox = document.getElementById("detail-error");

  title.textContent = provider === "claude" ? "Claude" : "Codex";
  rows.innerHTML = "";
  errBox.hidden = true;

  if (!data || data.error) {
    errBox.hidden = false;
    errBox.textContent = (data && data.error) || "Nessun dato disponibile";
    return;
  }

  for (const w of data.windows) {
    const row = document.createElement("div");
    row.className = "detail-row";
    row.innerHTML = `
      <div class="detail-row-top">
        <span>${w.label}</span>
        <span>Reset: ${w.resets_in}</span>
      </div>
      <div class="bar"><div class="bar-fill" style="width:${Math.min(
        100,
        w.used_percent
      )}%; background:${pctColor(w.used_percent)}"></div></div>
      <div class="detail-row-pct">${Math.round(w.used_percent)}% usato</div>
    `;
    rows.appendChild(row);
  }
}

async function setExpanded(expanded) {
  const detail = document.getElementById("detail");
  detail.hidden = !expanded;
  const h = expanded ? COLLAPSED_HEIGHT + EXPANDED_EXTRA_HEIGHT : COLLAPSED_HEIGHT;
  try {
    await win.setSize(new LogicalSize(COLLAPSED_WIDTH, h));
  } catch (e) {
    // Se l'API finestra non è disponibile (vedi README), il pannello si apre
    // comunque: resta solo il piccolo overflow visivo da correggere a mano.
    console.warn("setSize non disponibile:", e);
  }
}

function toggleProvider(provider) {
  if (openProvider === provider) {
    openProvider = null;
    setExpanded(false);
    return;
  }
  openProvider = provider;
  renderDetail(provider);
  setExpanded(true);
}

document
  .getElementById("claude-btn")
  .addEventListener("click", () => toggleProvider("claude"));
document
  .getElementById("codex-btn")
  .addEventListener("click", () => toggleProvider("codex"));

async function refreshAll() {
  await Promise.all([
    refreshProvider("claude", "claude-pct", "claude-ring-fg"),
    refreshProvider("codex", "codex-pct", "codex-ring-fg"),
  ]);
  if (openProvider) renderDetail(openProvider);
}

refreshAll();
setInterval(refreshAll, REFRESH_MS);
