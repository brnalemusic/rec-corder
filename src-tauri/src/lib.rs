mod commands;
mod config;
mod errors;
mod services;
mod state;

use commands::recorder::{self, SessionHandle};
use parking_lot::Mutex;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .manage::<SessionHandle>(Mutex::new(None))
        .invoke_handler(tauri::generate_handler![
            recorder::get_config,
            recorder::update_config,
            recorder::get_status,
            recorder::start_recording,
            recorder::stop_recording,
            recorder::list_monitors,
            recorder::list_mics,
            recorder::list_audio_outputs,
            recorder::get_output_dir,
            recorder::set_output_dir,
            recorder::check_crash_recovery,
            recorder::test_environment,
            recorder::finish_splash,
            recorder::get_app_info,
            recorder::acknowledge_welcome,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
