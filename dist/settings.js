const { invoke } = window.__TAURI__.core;
const { getCurrentWindow, LogicalSize } = window.__TAURI__.window;

// Auto-fit all'altezza reale del contenuto: le metriche dei font e lo scaling
// display (125%/150% su Windows) cambiano l'altezza renderizzata, e la riga
// "Collassa dopo" appare/scompare a seconda della modalità. Stesso pattern di
// main.js: applyLayout — si misura il DOM invece di duplicare le dimensioni tra
// CSS e tauri.conf.json. `setSize` in Tauri v2 imposta la inner size, quindi la
// titlebar Windows resta fuori dal conto.
async function fitWindowToContent() {
  await new Promise(requestAnimationFrame);
  const height = document.documentElement.scrollHeight;
  try {
    await getCurrentWindow().setSize(new LogicalSize(340, Math.ceil(height)));
  } catch (e) {
    // Se l'API finestra non è disponibile, resta solo la scrollbar come
    // degradazione graziosa (coerente con la scelta fatta per la Pill).
    console.warn("setSize non disponibile:", e);
  }
}

function updateCollapseDelayVisibility() {
  const auto = document.getElementById("pill-visibility-mode").value === "auto_collapse";
  document.getElementById("pill-collapse-delay-row").hidden = !auto;
  fitWindowToContent();
}

// Anteprima live: a ogni spostamento dello slider aggiorna l'etichetta ed
// emette "pill-scale-preview" verso la Pill. Nessuna scrittura su disco: la
// persistenza è solo al Salva (save()).
const pillScaleInput = document.getElementById("pill-scale");
const pillScaleOut = document.getElementById("pill-scale-out");
pillScaleInput.addEventListener("input", () => {
  pillScaleOut.textContent = `${pillScaleInput.value}%`;
  window.__TAURI__.event
    .emit("pill-scale-preview", Number(pillScaleInput.value) / 100)
    .catch((e) => console.warn("impossibile inviare l'anteprima della scala:", e));
});

async function load() {
  const settings = await invoke("get_settings");
  document.getElementById("active-claude").checked = settings.active_providers.claude !== false;
  document.getElementById("active-codex").checked = settings.active_providers.codex !== false;
  document.getElementById("active-copilot").checked = settings.active_providers.copilot !== false;
  document.getElementById("refresh-interval").value = settings.refresh_interval_s;
  document.getElementById("alert-threshold").value = settings.alert_threshold_pct;
  document.getElementById("pill-visibility-mode").value = settings.pill_visibility_mode || "always";
  document.getElementById("pill-collapse-delay").value = settings.pill_collapse_delay_s;
  const pct = Math.round((settings.pill_scale ?? 1) * 100);
  pillScaleInput.value = pct;
  pillScaleOut.textContent = `${pct}%`;
  updateCollapseDelayVisibility();
  try {
    document.getElementById("autostart").checked = await invoke("plugin:autostart|is_enabled");
  } catch (e) {
    console.warn("autostart non disponibile:", e);
  }
  fitWindowToContent();
}

async function save() {
  const settings = await invoke("get_settings");
  settings.active_providers = {
    claude: document.getElementById("active-claude").checked,
    codex: document.getElementById("active-codex").checked,
    copilot: document.getElementById("active-copilot").checked,
  };
  settings.refresh_interval_s = Number(document.getElementById("refresh-interval").value) || 90;
  settings.alert_threshold_pct = Number(document.getElementById("alert-threshold").value) || 80;
  settings.pill_visibility_mode = document.getElementById("pill-visibility-mode").value;
  settings.pill_collapse_delay_s = Number(document.getElementById("pill-collapse-delay").value) || 3;
  settings.pill_scale = Number(pillScaleInput.value) / 100 || 1;
  await invoke("save_settings", { settings });

  const wantAutostart = document.getElementById("autostart").checked;
  try {
    await invoke(wantAutostart ? "plugin:autostart|enable" : "plugin:autostart|disable");
  } catch (e) {
    console.warn("impossibile aggiornare l'autostart:", e);
  }

  // La Pill applica subito la nuova configurazione invece di aspettare il
  // prossimo giro di polling (main.js: applySettingsNow). Canale già coperto
  // da core:event:default in capabilities/default.json.
  try {
    await window.__TAURI__.event.emit("settings-changed");
  } catch (e) {
    console.warn("impossibile notificare la modifica:", e);
  }

  const hint = document.getElementById("hint");
  hint.hidden = false;
  setTimeout(() => {
    hint.hidden = true;
  }, 2500);
}

document.getElementById("save-btn").addEventListener("click", save);
load();
