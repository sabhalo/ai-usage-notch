const { invoke } = window.__TAURI__.core;

function updateCollapseDelayVisibility() {
  const auto = document.getElementById("pill-visibility-mode").value === "auto_collapse";
  document.getElementById("pill-collapse-delay-row").hidden = !auto;
}

async function load() {
  const settings = await invoke("get_settings");
  document.getElementById("active-claude").checked = settings.active_providers.claude !== false;
  document.getElementById("active-codex").checked = settings.active_providers.codex !== false;
  document.getElementById("active-copilot").checked = settings.active_providers.copilot !== false;
  document.getElementById("active-gemini").checked = settings.active_providers.gemini !== false;
  document.getElementById("refresh-interval").value = settings.refresh_interval_s;
  document.getElementById("alert-threshold").value = settings.alert_threshold_pct;
  document.getElementById("pill-visibility-mode").value = settings.pill_visibility_mode || "always";
  document.getElementById("pill-collapse-delay").value = settings.pill_collapse_delay_s;
  updateCollapseDelayVisibility();
  try {
    document.getElementById("autostart").checked = await invoke("plugin:autostart|is_enabled");
  } catch (e) {
    console.warn("autostart non disponibile:", e);
  }
}

async function save() {
  const settings = await invoke("get_settings");
  settings.active_providers = {
    claude: document.getElementById("active-claude").checked,
    codex: document.getElementById("active-codex").checked,
    copilot: document.getElementById("active-copilot").checked,
    gemini: document.getElementById("active-gemini").checked,
  };
  settings.refresh_interval_s = Number(document.getElementById("refresh-interval").value) || 90;
  settings.alert_threshold_pct = Number(document.getElementById("alert-threshold").value) || 80;
  settings.pill_visibility_mode = document.getElementById("pill-visibility-mode").value;
  settings.pill_collapse_delay_s = Number(document.getElementById("pill-collapse-delay").value) || 3;
  await invoke("save_settings", { settings });

  const wantAutostart = document.getElementById("autostart").checked;
  try {
    await invoke(wantAutostart ? "plugin:autostart|enable" : "plugin:autostart|disable");
  } catch (e) {
    console.warn("impossibile aggiornare l'autostart:", e);
  }

  const hint = document.getElementById("hint");
  hint.hidden = false;
  setTimeout(() => {
    hint.hidden = true;
  }, 2500);
}

document.getElementById("save-btn").addEventListener("click", save);
load();
