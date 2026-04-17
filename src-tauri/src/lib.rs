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
use tauri::Manager;

#[cfg(all(debug_assertions, target_os = "linux"))]
fn setup_dev_desktop_file() {
    use std::fs;
    use std::path::PathBuf;
    
    if let Ok(home) = std::env::var("HOME") {
        let apps_dir = PathBuf::from(home).join(".local/share/applications");
        let _ = fs::create_dir_all(&apps_dir);
        
        if let Ok(current_dir) = std::env::current_dir() {
            let icon_path = current_dir.join("icons/icon.png");
            let exec_path = std::env::current_exe().unwrap_or_default();
            
            let content = format!(
                "[Desktop Entry]\n\
                Name=Rec Corder (Dev)\n\
                Exec=\"{}\"\n\
                Icon={}\n\
                Type=Application\n\
                Terminal=false\n",
                exec_path.display(),
                icon_path.display()
            );
            
            let _ = fs::write(apps_dir.join("rec-corder.desktop"), &content);
            let _ = fs::write(apps_dir.join("com.reccorder.app.desktop"), &content);
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            #[cfg(all(debug_assertions, target_os = "linux"))]
            setup_dev_desktop_file();
            
            Ok(())
        })
        .manage(AppState::new())
        .manage::<SessionHandle>(Mutex::new(None))
        .manage(PendingUpdate(Mutex::new(None)))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "settings" {
                    #[cfg(target_os = "windows")]
                    {
                        let _ = window.hide();
                        api.prevent_close();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            recorder::get_config,
            recorder::update_config,
            recorder::get_status,
            recorder::start_recording,
            recorder::stop_recording,
            recorder::force_exit,
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
            recorder::list_cameras,
            ffmpeg::check_ffmpeg,
            updater::check_for_updates,
            updater::show_updater,
            updater::get_release_notes,
            updater::show_release_notes,
            updater::install_update,
            updater::open_link,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
