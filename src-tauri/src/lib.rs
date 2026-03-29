mod commands;
mod config;
mod errors;
mod services;
mod state;

use commands::recorder::{self, SessionHandle};
use commands::ffmpeg;
use parking_lot::Mutex;
use state::AppState;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(updater) = handle.updater() {
                    if let Ok(Some(update)) = updater.check().await {
                        let version = update.version.clone();
                        let msg = format!(
                            "Uma nova versão ({}) está disponível! Deseja baixar e atualizar agora?",
                            version
                        );
                        handle
                            .dialog()
                            .message(msg)
                            .title("Atualização Disponível")
                            .kind(MessageDialogKind::Info)
                            .buttons(MessageDialogButtons::OkCancel)
                            .show(move |ok| {
                                if ok {
                                    let handle_clone = handle.clone();
                                    tauri::async_runtime::spawn(async move {
                                        let _ = update.download_and_install(|_, _| {}, || {}).await;
                                        handle_clone.restart();
                                    });
                                }
                            });
                    }
                }
            });
            Ok(())
        })
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
            ffmpeg::check_ffmpeg,
            ffmpeg::download_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
