use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use parking_lot::Mutex as ParkingMutex;
use serde::Serialize;
use tauri::{AppHandle, Manager, State, Emitter};

use crate::services::capture::{
    self, AudioOutputInfo, CaptureSession, MicInfo, MonitorInfo,
    ffmpeg::resolve_ffmpeg_path,
    session::WebcamOverlayConfig,
};
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
    pub runtime_error: Option<String>,
}



#[derive(Serialize)]
pub struct StartResult {
    pub file_path: String,
}

#[derive(Serialize)]
pub struct CameraInfo {
    pub name: String,
    pub id: String,
}

#[derive(Serialize)]
pub struct AppInfo {
    pub version: String,
}

#[tauri::command]
pub fn get_status(
    state: State<'_, AppState>,
    session_handle: State<'_, SessionHandle>,
) -> RecordingStatus {
    let file = state
        .current_file
        .lock()
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());
    let runtime_error = session_handle.lock().as_mut().and_then(|active| {
        active
            .session
            .poll_runtime_error()
            .unwrap_or_else(|err| Some(err.to_string()))
    });

    RecordingStatus {
        is_recording: state.recording(),
        elapsed_secs: state.elapsed_secs(),
        output_file: file,
        runtime_error,
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
        .inner_size(930.0, 750.0)
        .min_inner_size(800.0, 650.0)
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
        version: env!("CARGO_PKG_VERSION").to_string(),
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

    let webcam_config = if config.webcam_enabled {
        config.webcam_device.as_ref().map(|name| {
            WebcamOverlayConfig {
                device_name: name.clone(),
                position: config.webcam_position.clone(),
                size_percent: config.webcam_size,
            }
        })
    } else {
        None
    };

    let session = match CaptureSession::start(file_path.clone(), monitor, mic_name.or(config.selected_mic.clone()), system_audio_device.or(config.selected_audio_output.clone()), configured_fps, scale, &encoder, webcam_config) {
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

    // Reset state regardless of outcome to allow new recordings
    state.set_recording(false);
    *state.recording_start.lock() = None;
    *state.current_file.lock() = None;
    *state.crash_marker.lock() = None;

    match stop_result {
        Ok(_) => {
            watchdog::clear_crash_marker(&output_dir);
            Ok(file_path)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn force_exit(session_handle: State<'_, SessionHandle>) {
    if let Some(mut active) = session_handle.lock().take() {
        active.session.kill();
    }
    std::process::exit(0);
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

#[tauri::command]
pub async fn list_cameras() -> Result<Vec<CameraInfo>, String> {
    use std::process::{Command, Stdio};
    use std::os::windows::process::CommandExt;
    use super::super::services::capture::ffmpeg::CREATE_NO_WINDOW;

    let mut cameras = Vec::new();

    // Tenta usar FFmpeg primeiro
    if let Ok(ffmpeg_path) = resolve_ffmpeg_path() {
        if let Ok(output) = Command::new(ffmpeg_path)
            .creation_flags(CREATE_NO_WINDOW)
            .args(["-list_devices", "true", "-f", "dshow", "-i", "dummy"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            let combined_output = format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
            let mut in_video_section = false;

            for line in combined_output.lines() {
                let line_lower = line.to_lowercase();
                if line_lower.contains("directshow video devices") {
                    in_video_section = true;
                    continue;
                }
                if line_lower.contains("directshow audio devices") {
                    break;
                }
                if in_video_section {
                    // Device names appear between quotes: "Device Name"
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start + 1..].find('"') {
                            let name = line[start + 1..start + 1 + end].to_string();
                            // Skip "alternative name" lines
                            if !line_lower.contains("alternative name") {
                                cameras.push(CameraInfo {
                                    id: name.clone(),
                                    name,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: PowerShell if ffmpeg fails or finds no cameras
    if cameras.is_empty() {
        if let Ok(output) = Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args([
                "-NoProfile",
                "-Command",
                "Get-PnpDevice -PresentOnly | Where-Object { $_.PNPClass -eq 'Camera' -or $_.PNPClass -eq 'Image' } | Select-Object -ExpandProperty FriendlyName"
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let name = line.trim().to_string();
                if !name.is_empty() {
                    cameras.push(CameraInfo {
                        id: name.clone(),
                        name,
                    });
                }
            }
        }
    }

    Ok(cameras)
}
