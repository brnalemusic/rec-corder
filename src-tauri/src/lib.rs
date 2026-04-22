mod commands;
mod config;
mod errors;
#[cfg(feature = "python")]
mod python_api;
mod services;
mod state;

use commands::recorder::{self, SessionHandle};
use commands::ffmpeg;
use parking_lot::Mutex;
use state::AppState;
use commands::updater::{self, PendingUpdate};

/// Função de entrada principal que inicializa o contexto e os plugins do Tauri.
/// // [IMPORTANTE] Configura o AppState e o gerador de eventos globais.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            // Validação de dependências no Linux (não-bloqueante no setup)
            #[cfg(target_os = "linux")]
            {
                use crate::services::capture::linux::validate_linux_system_deps;
                if let Err(missing) = validate_linux_system_deps() {
                    eprintln!("[AVISO] Dependências do sistema ausentes: {:?}. A gravação será bloqueada.", missing);
                } else {
                    println!("[INFO] Todas as dependências do sistema Linux foram encontradas.");
                }
            }

            Ok(())
        })
        .manage(AppState::new())
        .manage::<SessionHandle>(Mutex::new(None))
        .manage(PendingUpdate(Mutex::new(None)))
        .on_window_event(|window, event| {
            // Comportamento customizado ao fechar as janelas
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = api;
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
            recorder::check_linux_deps,
            recorder::install_linux_deps,
        ])
        .run(tauri::generate_context!())
        .expect("erro enquanto rodava a aplicacao tauri");
}
