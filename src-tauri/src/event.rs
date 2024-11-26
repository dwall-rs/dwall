use tauri::{AppHandle, RunEvent};

pub fn run_callback(_app_handle: &AppHandle, event: RunEvent) {
    if let tauri::RunEvent::ExitRequested { api, .. } = event {
        api.prevent_exit();
    }
}
