use crate::{
    providers::{UsageReport, SUPPORTED_PROVIDER_IDS},
    settings::Settings,
};
use tauri::{
    image::Image,
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder},
    AppHandle, Runtime,
};

const TRAY_ICONS: [(&str, &str, &[u8]); 3] = [
    (
        "claude",
        "Claude",
        include_bytes!("../dist/icons/tray/claude.png"),
    ),
    (
        "codex",
        "Codex",
        include_bytes!("../dist/icons/tray/codex.png"),
    ),
    (
        "copilot",
        "GitHub Copilot",
        include_bytes!("../dist/icons/tray/copilot.png"),
    ),
];

pub fn sync<R: Runtime>(
    app: &AppHandle<R>,
    settings: &Settings,
    reports: &[UsageReport],
) -> tauri::Result<()> {
    for provider in SUPPORTED_PROVIDER_IDS {
        let tray_id = format!("usage-{provider}");
        if !should_show_tray(provider, settings) {
            app.remove_tray_by_id(&tray_id);
            continue;
        }

        let mut menu = MenuBuilder::new(app);
        if settings.active_providers.get(provider) != Some(&false) {
            for (index, line) in usage_lines(provider, reports).into_iter().enumerate() {
                menu = menu.text(format!("{tray_id}-{index}"), line);
            }
            menu = menu.separator();
        }
        menu = menu.text("open-settings", "Impostazioni…");
        let menu = menu.build()?;

        if let Some(tray) = app.tray_by_id(&tray_id) {
            tray.set_menu(Some(menu))?;
            continue;
        }

        let (_, title, bytes) = TRAY_ICONS
            .iter()
            .find(|(id, _, _)| *id == provider)
            .expect("ogni provider supportato deve avere un'icona tray");
        TrayIconBuilder::with_id(tray_id)
            .icon(Image::from_bytes(bytes)?)
            .icon_as_template(true)
            .tooltip(title)
            .menu(&menu)
            .show_menu_on_left_click(false)
            .build(app)?;
    }

    Ok(())
}

fn should_show_tray(provider: &str, settings: &Settings) -> bool {
    settings.active_providers.get(provider) != Some(&false)
        || (!settings.show_pill
            && !settings.active_providers.values().any(|active| *active)
            && provider == SUPPORTED_PROVIDER_IDS[0])
}

pub fn should_refresh_on_click(id: &str, button: MouseButton, state: MouseButtonState) -> bool {
    button == MouseButton::Left
        && state == MouseButtonState::Down
        && id
            .strip_prefix("usage-")
            .is_some_and(|provider| SUPPORTED_PROVIDER_IDS.contains(&provider))
}

fn usage_lines(provider: &str, reports: &[UsageReport]) -> Vec<String> {
    let Some(report) = reports.iter().find(|report| report.provider == provider) else {
        return vec!["In attesa dei dati…".into()];
    };
    if report.error.is_some() || report.windows.is_empty() {
        return vec!["Consumo non disponibile".into()];
    }

    report
        .windows
        .iter()
        .map(|window| {
            let usage = if window.unlimited {
                "illimitato".into()
            } else {
                format!("{:.0}% usato", window.used_percent)
            };
            if window.resets_in.is_empty() {
                format!("{}: {usage}", window.label)
            } else {
                format!("{}: {usage} · {}", window.label, window.resets_in)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::UsageWindow;

    #[test]
    fn formats_all_usage_windows() {
        let reports = [UsageReport {
            provider: "codex".into(),
            windows: vec![
                UsageWindow {
                    label: "5 ore".into(),
                    used_percent: 42.4,
                    resets_in: "reset tra 2h".into(),
                    ..Default::default()
                },
                UsageWindow {
                    label: "Crediti".into(),
                    unlimited: true,
                    resets_in: "".into(),
                    ..Default::default()
                },
            ],
            error: None,
            retry_after_s: None,
        }];

        assert_eq!(
            usage_lines("codex", &reports),
            ["5 ore: 42% usato · reset tra 2h", "Crediti: illimitato"]
        );
    }

    #[test]
    fn tray_icons_have_transparency() {
        for (provider, _, bytes) in TRAY_ICONS {
            let image = Image::from_bytes(bytes).unwrap();
            assert!(
                image
                    .rgba()
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|pixel| pixel[3] == 0),
                "{provider}"
            );
        }
    }

    #[test]
    fn keeps_settings_access_when_pill_and_providers_are_hidden() {
        let mut settings = Settings {
            show_pill: false,
            ..Default::default()
        };
        settings
            .active_providers
            .values_mut()
            .for_each(|active| *active = false);

        assert!(should_show_tray(SUPPORTED_PROVIDER_IDS[0], &settings));
        assert!(!should_show_tray(SUPPORTED_PROVIDER_IDS[1], &settings));
    }

    #[test]
    fn refreshes_once_per_click_on_a_provider_icon() {
        assert!(should_refresh_on_click(
            "usage-codex",
            MouseButton::Left,
            MouseButtonState::Down
        ));
        assert!(!should_refresh_on_click(
            "usage-codex",
            MouseButton::Left,
            MouseButtonState::Up
        ));
        assert!(!should_refresh_on_click(
            "usage-codex",
            MouseButton::Right,
            MouseButtonState::Down
        ));
        assert!(!should_refresh_on_click(
            "other",
            MouseButton::Left,
            MouseButtonState::Down
        ));
    }
}
