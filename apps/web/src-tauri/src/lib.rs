//! Tauri shell for the giffstack web dashboard. The SvelteKit app does the actual work;
//! this binary just hosts a webview, exposes the HTTP plugin (for CORS-free GitHub
//! requests), and exposes the OS plugin (so the frontend can detect macOS vs others
//! and render the right window controls).
//!
//! Window chrome strategy:
//!   - macOS: `decorations: true` + `titleBarStyle: "Overlay"` + `hiddenTitle: true`.
//!     Keeps the native traffic lights visible while removing the title bar text and
//!     letting content extend under them — the Linear / Notion look.
//!   - Windows / Linux: decorations stripped at runtime here (those platforms don't have
//!     an equivalent of macOS's overlay style; the frontend draws its own custom min /
//!     maximize / close buttons via WindowControls.svelte).

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            #[cfg(not(target_os = "macos"))]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_decorations(false);
                }
            }
            // Keep `app` referenced on macOS where the cfg block is empty.
            let _ = app;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
