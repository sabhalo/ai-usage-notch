//! Posizionamento accanto alla notch fisica su macOS (plan/step-5.3.md).
//! `NSScreen.safeAreaInsets`/`auxiliaryTopLeftArea` non sono esposte da
//! Tauri: binding diretti via objc2-app-kit (già nel grafo delle dipendenze
//! transitivamente tramite tao/wry, promossi a dipendenza diretta).

#[cfg(target_os = "macos")]
pub fn left_edge_x() -> Option<f64> {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    let mtm = MainThreadMarker::new()?;
    let screen = NSScreen::mainScreen(mtm)?;

    // safeAreaInsets.top > 0 solo sui Mac con notch fisica (MacBook Pro
    // 14"/16" 2021+) quando il display integrato è davvero attivo — in
    // clamshell mode (coperchio chiuso, solo monitor esterni) lo schermo
    // con la notch non compare affatto in NSScreen.screens() e questa
    // funzione ritorna None per costruzione, non per un bug: verificato
    // qui il 2026-08-30 su un MacBook Pro M4 Max in clamshell, dove tutti
    // gli schermi elencati (due monitor esterni) avevano correttamente
    // insets a zero. Da riverificare con il display integrato attivo.
    let insets = screen.safeAreaInsets();
    #[cfg(debug_assertions)]
    eprintln!("[notch] safeAreaInsets su schermo principale: {insets:?}");
    if insets.top <= 0.0 {
        return None;
    }

    let left_area = screen.auxiliaryTopLeftArea();
    if left_area.size.width <= 0.0 {
        return None;
    }
    Some(left_area.origin.x + left_area.size.width)
}

#[cfg(not(target_os = "macos"))]
pub fn left_edge_x() -> Option<f64> {
    None
}
