use crate::models::AppState;
use std::sync::Arc;
use tauri::App;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
pub fn register_shortcuts(
    app: &mut App,
    state: Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager;
    let panic_shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyL);

    let _ = app
        .global_shortcut()
        .on_shortcut(panic_shortcut, move |app, shortcut, event| {
            if event.state() == ShortcutState::Pressed && shortcut == &panic_shortcut {
                let mut unlocked = state.is_unlocked.lock().unwrap();
                *unlocked = false;
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
                println!("[Panic] Security Lockdown Triggered.");
            }
        });

    let _ = app.global_shortcut().register(panic_shortcut);
    Ok(())
}
