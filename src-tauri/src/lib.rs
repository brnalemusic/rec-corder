mod commands;
mod config;
mod errors;
mod services;
mod state;

use commands::recorder::{self, SessionHandle};
use commands::ffmpeg;
use parking_lot::Mutex;
use state::AppState;
use commands::updater::{self, PendingUpdate};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            Ok(())
        })
        .manage(AppState::new())
        .manage::<SessionHandle>(Mutex::new(None))
        .manage(PendingUpdate(Mutex::new(None)))
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
            recorder::show_settings,
            recorder::hide_settings,
            recorder::check_crash_recovery,
            recorder::test_environment,
            recorder::finish_splash,
            recorder::get_app_info,
            ffmpeg::check_ffmpeg,
            updater::check_for_updates,
            updater::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
