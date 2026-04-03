use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use parking_lot::Mutex as ParkingMutex;
use serde::Serialize;
use tauri::{AppHandle, Manager, State, Emitter};

use crate::services::capture::{self, AudioOutputInfo, CaptureSession, MicInfo, MonitorInfo};
use crate::services::watchdog;
use crate::state::AppState;

pub type SessionHandle = ParkingMutex<Option<ActiveSession>>;

pub struct ActiveSession {
    pub session: CaptureSession,
    pub stop_flag: Arc<AtomicBool>,
}

#[derive(Serialize)]
pub struct RecordingStatus {
    pub is_recording: bool,
    pub elapsed_secs: u64,
    pub output_file: Option<String>,
}



#[derive(Serialize)]
pub struct StartResult {
    pub file_path: String,
}

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
}

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> RecordingStatus {
    let file = state
        .current_file
        .lock()
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());

    RecordingStatus {
        is_recording: state.recording(),
        elapsed_secs: state.elapsed_secs(),
        output_file: file,
    }
}

#[tauri::command]
pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    capture::list_monitors().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_mics() -> Result<Vec<MicInfo>, String> {
    capture::list_mic_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_audio_outputs() -> Result<Vec<AudioOutputInfo>, String> {
    capture::list_audio_outputs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> crate::config::AppConfig {
    state.config.lock().clone()
}

#[tauri::command]
pub fn update_config(app: AppHandle, state: State<'_, AppState>, config: crate::config::AppConfig) -> Result<(), String> {
    {
        let mut current = state.config.lock();
        *current = config.clone();
        
        // Sync with legacy output_dir for stability
        *state.output_dir.lock() = current.output_dir.clone();
        
        current.save()?;
    }
    
    // Notificar todas as janelas sobre a mudança
    let _ = app.emit("config-updated", config);
    
    Ok(())
}

#[tauri::command]
pub fn test_environment(app: AppHandle, state: State<'_, AppState>) -> String {
    // Check central config first
    {
        let cfg = state.config.lock();
        if cfg.encoder != "libx264" && !cfg.encoder.is_empty() {
            println!("Encoder carregado do cache: {}", cfg.encoder);
            return cfg.encoder.clone();
        }
    }

    // First run or manual test — detect hardware
    let encoder = capture::test_environment();
    
    // Update and persist
    let config = {
        let mut cfg = state.config.lock();
        cfg.encoder = encoder.clone();
        let _ = cfg.save();
        cfg.clone()
    };
    
    // Notificar sobre a atualização do encoder
    let _ = app.emit("config-updated", config);
    
    println!("Novo encoder detectado e salvo: {}", encoder);
    encoder
}

#[tauri::command]
pub async fn show_settings(app: AppHandle) -> Result<(), String> {
    if let Some(settings_window) = app.get_webview_window("settings") {
        settings_window.show().map_err(|e| e.to_string())?;
        settings_window.unminimize().map_err(|e| e.to_string())?;
        settings_window.set_focus().map_err(|e| e.to_string())?;
    } else {
        // Se a janela foi fechada (destruída), precisamos criá-la novamente
        let _ = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("settings.html".into()),
        )
        .title("Configurações - Rec Corder")
        .inner_size(500.0, 650.0)
        .min_inner_size(450.0, 550.0)
        .resizable(true)
        .decorations(true)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_settings(app: AppHandle) -> Result<(), String> {
    if let Some(settings_window) = app.get_webview_window("settings") {
        settings_window.hide().map_err(|e| e.to_string())?;
    }

    if let Some(main_window) = app.get_webview_window("main") {
        let _ = main_window.unminimize();
        let _ = main_window.set_focus();
    }

    Ok(())
}

#[tauri::command]
pub fn finish_splash(app: AppHandle) -> Result<(), String> {
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.show().map_err(|e| e.to_string())?;
        main_window.set_focus().map_err(|e| e.to_string())?;
    }
    if let Some(splash_window) = app.get_webview_window("splash") {
        splash_window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: "0.2.0".to_string(), // Sincronizado com tauri.conf.json
    }
}

#[tauri::command]
pub fn start_recording(
    state: State<'_, AppState>,
    session_handle: State<'_, SessionHandle>,
    monitor_index: Option<usize>,
    mic_name: Option<String>,
    system_audio_device: Option<String>,
    fps: Option<u32>,
    scale_factor: Option<u32>,
) -> Result<StartResult, String> {
    if state.recording() {
        println!("Erro: ja existe uma gravacao em andamento");
        return Err("Ja existe uma gravacao em andamento".into());
    }

    let config = state.config.lock();
    
    let requested_monitor = monitor_index.unwrap_or(config.selected_monitor);
    let monitor = capture::resolve_monitor_index(requested_monitor)
        .unwrap_or(requested_monitor);
    let configured_fps = fps.unwrap_or(config.fps);
    let scale = scale_factor.unwrap_or(config.scale);
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let output_dir = state.output_dir.lock().clone();
    let file_path = output_dir.join(format!("RecCorder_{timestamp}.mp4"));

    let marker =
        watchdog::write_crash_marker(&output_dir, &file_path).map_err(|e| e.to_string())?;
    let stop_flag = Arc::new(AtomicBool::new(false));

    let encoder = config.encoder.clone();

    let session = match CaptureSession::start(file_path.clone(), monitor, mic_name.or(config.selected_mic.clone()), system_audio_device.or(config.selected_audio_output.clone()), configured_fps, scale, &encoder) {
        Ok(session) => session,
        Err(err) => {
            watchdog::clear_crash_marker(&output_dir);
            let err_msg = err.to_string();
            println!("Erro critico no CaptureSession: {}", err_msg);
            return Err(err_msg);
        }
    };

    state.set_recording(true);
    *state.recording_start.lock() = Some(std::time::Instant::now());
    *state.current_file.lock() = Some(file_path.clone());
    *state.crash_marker.lock() = Some(marker);

    *session_handle.lock() = Some(ActiveSession { session, stop_flag });

    Ok(StartResult {
        file_path: file_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn stop_recording(
    state: State<'_, AppState>,
    session_handle: State<'_, SessionHandle>,
) -> Result<String, String> {
    if !state.recording() {
        return Err("Nenhuma gravacao em andamento".into());
    }

    // Retira a sessão do estado para processamento background
    let mut active = session_handle.lock().take().ok_or("Erro: sessao nao encontrada no estado")?;
    
    // Sinaliza parada imediata das threads de audio
    active.stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);

    // Executa a finalização pesada (FFmpeg, Muxing, IO) em uma thread de bloqueio
    // Isso evita que o executor assíncrono do Tauri trave a comunicação com a UI
    let stop_result = tokio::task::spawn_blocking(move || {
        // Pequena espera para garantir que os buffers de áudio fechem
        std::thread::sleep(std::time::Duration::from_millis(200));
        active.session.stop()
    }).await.map_err(|e| format!("Erro no worker de finalizacao: {e}"))?;

    let file_path = state
        .current_file
        .lock()
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let output_dir = state.output_dir.lock().clone();
    watchdog::clear_crash_marker(&output_dir);

    state.set_recording(false);
    *state.recording_start.lock() = None;
    *state.current_file.lock() = None;
    *state.crash_marker.lock() = None;

    stop_result.map_err(|e| e.to_string())?;

    Ok(file_path)
}

#[tauri::command]
pub fn get_output_dir(state: State<'_, AppState>) -> String {
    state.output_dir.lock().to_string_lossy().to_string()
}

#[tauri::command]
pub fn set_output_dir(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        std::fs::create_dir_all(&p).map_err(|e| format!("Erro ao criar diretorio: {e}"))?;
    }
    let config = {
        let mut cfg = state.config.lock();
        cfg.output_dir = p.clone();
        let _ = cfg.save();
        cfg.clone()
    };
    *state.output_dir.lock() = p;
    
    // Notificar todas as janelas sobre a mudança na pasta (e config)
    let _ = app.emit("config-updated", config);
    
    Ok(())
}

#[tauri::command]
pub fn check_crash_recovery(state: State<'_, AppState>) -> Option<String> {
    let output_dir = state.output_dir.lock().clone();
    watchdog::check_crash_recovery(&output_dir).map(|p| p.to_string_lossy().to_string())
}
