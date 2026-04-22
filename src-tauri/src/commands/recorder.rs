use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex as ParkingMutex;
use serde::Serialize;
use tauri::{AppHandle, Manager, State, Emitter};

use crate::services::capture::{
    self, AudioOutputInfo, CaptureSession, MicInfo, MonitorInfo, CameraInfo,
    ffmpeg::resolve_ffmpeg_path,
    session::WebcamOverlayConfig,
};
use crate::services::watchdog;
use crate::state::AppState;

/// Referência thread-safe para a sessão de captura atual (estado protegido).
/// // [IMPORTANTE] Garante segurança ao compartilhar o processo de gravação do Tauri.
pub type SessionHandle = ParkingMutex<Option<ActiveSession>>;

/// Estrutura que mantém o processo FFmpeg em execução e um flag atômico para interrupções.
pub struct ActiveSession {
    /// A sessão de captura responsável por gerenciar o processo do FFmpeg e arquivos locais.
    pub session: CaptureSession,
    /// Sinalizador de parada thread-safe, permitindo interrupções assíncronas suaves.
    pub stop_flag: Arc<AtomicBool>,
}

/// Representa o status atual da gravação (serializado para o frontend).
#[derive(Serialize)]
pub struct RecordingStatus {
    /// Verdadeiro se a gravação está ocorrendo no momento.
    pub is_recording: bool,
    /// Quantidade de tempo decorrido em segundos.
    pub elapsed_secs: u64,
    /// Arquivo de saída atual, se a gravação estiver ativa.
    pub output_file: Option<String>,
}

/// Resultado do comando de início da gravação, contendo o caminho do arquivo gerado.
#[derive(Serialize)]
pub struct StartResult {
    /// Caminho gerado do arquivo destino.
    pub file_path: String,
}

/// Estrutura contendo as informações e versão do aplicativo.
#[derive(Serialize)]
pub struct AppInfo {
    /// Versão compilada a partir do Cargo.toml.
    pub version: String,
}

/// Retorna o status contínuo da gravação e informações parciais.
#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> RecordingStatus {
    let file = state
        .current_file
        .lock()
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned());

    RecordingStatus {
        is_recording: state.recording(),
        elapsed_secs: state.elapsed_secs(),
        output_file: file,
    }
}

/// Retorna uma lista com informações sobre os monitores ativos conectados ao sistema.
#[tauri::command]
pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    capture::list_monitors().map_err(|e| e.to_string())
}

/// Lista assincronamente os microfones (dispositivos de áudio de captura) disponíveis.
#[tauri::command]
pub async fn list_mics() -> Result<Vec<MicInfo>, String> {
    capture::list_mic_devices().map_err(|e| e.to_string())
}

/// Lista assincronamente as saídas de áudio do sistema ativas (loopback de sistema).
#[tauri::command]
pub async fn list_audio_outputs() -> Result<Vec<AudioOutputInfo>, String> {
    capture::list_audio_outputs().map_err(|e| e.to_string())
}

/// Lista assincronamente as câmeras disponíveis de acordo com a plataforma (Linux / Windows).
#[tauri::command]
pub async fn list_cameras() -> Result<Vec<CameraInfo>, String> {
    capture::list_cameras().map_err(|e| e.to_string())
}

/// Lê a configuração salva no arquivo local sincronizado via Mutex de leitura.
#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> crate::config::AppConfig {
    state.config.lock().clone()
}

/// Atualiza e persiste a configuração do App no disco e notifica a GUI Tauri.
/// // [IMPORTANTE] Escopo do lock reduzido para garantir que o save() do I/O 
/// não afete a estabilidade geral ou bloqueie renderizações de interface.
#[tauri::command]
pub fn update_config(app: AppHandle, state: State<'_, AppState>, config: crate::config::AppConfig) -> Result<(), String> {
    {
        let mut current = state.config.lock();
        *current = config.clone();
        
        // Sincroniza o diretório legado temporário, mantendo coerência
        *state.output_dir.lock() = current.output_dir.clone();
        
        current.save()?;
    }
    
    // Dispara o evento atualizado pro JS da Interface Gráfica
    let _ = app.emit("config-updated", config);
    
    Ok(())
}

/// Testa o ambiente detectando o encoder ideal para captura de hardware atual.
#[tauri::command]
pub fn test_environment(app: AppHandle, state: State<'_, AppState>) -> String {
    // Reduz escopo do Mutex limitando o escopo
    let cached_encoder = {
        let cfg = state.config.lock();
        if cfg.encoder != "libx264" && !cfg.encoder.is_empty() {
            Some(cfg.encoder.clone())
        } else {
            None
        }
    };
    
    if let Some(enc) = cached_encoder {
        println!("Encoder carregado do cache: {}", enc);
        return enc;
    }

    // Identifica e varre drivers da GPU sem bloqueio de trava
    let encoder = capture::test_environment();
    
    let config = {
        let mut cfg = state.config.lock();
        cfg.encoder = encoder.clone();
        let _ = cfg.save();
        cfg.clone()
    };
    
    let _ = app.emit("config-updated", config);
    
    println!("Novo encoder detectado e salvo: {}", encoder);
    encoder
}

/// Exibe a janela gráfica secundária de configurações do Tauri (Preferences/Config).
#[tauri::command]
pub async fn show_settings(app: AppHandle) -> Result<(), String> {
    if let Some(settings_window) = app.get_webview_window("settings") {
        settings_window.show().map_err(|e| e.to_string())?;
        settings_window.unminimize().map_err(|e| e.to_string())?;
        settings_window.set_focus().map_err(|e| e.to_string())?;
    } else {
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

/// Oculta a janela secundária de configurações.
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

/// Substitui a Splash Screen inicial e exibe a janela primária principal.
#[tauri::command]
pub fn finish_splash(app: AppHandle) -> Result<(), String> {
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.show().map_err(|e| e.to_string())?;
        main_window.set_focus().map_err(|e| e.to_string())?;

        // Workaround for Linux Wayland/GTK where window decorations (X button) 
        // can lose hover/click after .show() if window was created hidden.
        #[cfg(target_os = "linux")]
        {
            let _ = main_window.set_resizable(false);
            let _ = main_window.set_resizable(true);
            if let Ok(size) = main_window.outer_size() {
                let _ = main_window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: size.width, height: size.height + 1 }));
                let _ = main_window.set_size(tauri::Size::Physical(size));
            }
        }
    } else {
        let main_window = tauri::WebviewWindowBuilder::new(
            &app,
            "main",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("Rec Corder")
        .inner_size(380.0, 730.0)
        .min_inner_size(360.0, 640.0)
        .resizable(true)
        .decorations(true)
        .transparent(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;

        // Apply the same hack if the window is created dynamically here
        #[cfg(target_os = "linux")]
        {
            let _ = main_window.set_resizable(false);
            let _ = main_window.set_resizable(true);
            if let Ok(size) = main_window.outer_size() {
                let _ = main_window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: size.width, height: size.height + 1 }));
                let _ = main_window.set_size(tauri::Size::Physical(size));
            }
        }
    }
    if let Some(splash_window) = app.get_webview_window("splash") {
        splash_window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Retorna metadados do binário, atualmente versionamento via manifest.
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Dispara o começo de uma gravação.
/// // [IMPORTANTE] Reduzimos os scopes de Lock (Mutex) e tratamos referências pra garantir
/// ausência de engasgos durante I/O das streams de sistema (System Loopback ou Microfone).
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
    // Validação de dependências (Hard Block no Linux)
    #[cfg(target_os = "linux")]
    {
        use crate::services::capture::linux::validate_linux_system_deps;
        if let Err(missing) = validate_linux_system_deps() {
            return Err(format!(
                "Não é possível gravar: dependências do sistema ausentes ({}). Por favor, instale-as para continuar.",
                missing.join(", ")
            ));
        }
    }

    // Sincronização e verificação de corridas (Race Condition guard)
    // Usamos compare_exchange para \"reservar\" o estado de gravação atomicamente, fechando a janela TOCTOU.
    if state.is_recording.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        println!("Erro: ja existe uma gravacao em andamento (flag atômica)");
        return Err("Ja existe uma gravacao em andamento".into());
    }

    {
        let handle_guard = session_handle.lock();
        if handle_guard.is_some() {
            println!("Erro: inconsistência detectada, session_handle ocupado");
            state.set_recording(false); // Rollback da reserva
            return Err("Ja existe uma gravacao em andamento".into());
        }
    }

    // Duplicar os valores e dropar o Lock de ConfigURAÇÕES imediatamente
    let (config_monitor, config_fps, config_scale, config_encoder, output_dir_base, config_selected_mic, config_selected_audio, webcam_cfg) = {
        let cfg = state.config.lock();
        let webcam = if cfg.webcam_enabled {
            cfg.webcam_device.as_ref().map(|name| {
                WebcamOverlayConfig {
                    device_name: name.clone(),
                    position: cfg.webcam_position.clone(),
                    size_percent: cfg.webcam_size,
                }
            })
        } else {
            None
        };
        (
            cfg.selected_monitor, 
            cfg.fps, 
            cfg.scale, 
            cfg.encoder.clone(), 
            cfg.output_dir.clone(),
            cfg.selected_mic.clone(),
            cfg.selected_audio_output.clone(),
            webcam
        )
    };
    
    let requested_monitor = monitor_index.unwrap_or(config_monitor);
    let monitor = capture::resolve_monitor_index(requested_monitor)
        .unwrap_or(requested_monitor);
    let configured_fps = fps.unwrap_or(config_fps);
    let scale = scale_factor.unwrap_or(config_scale);
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    
    let file_path = output_dir_base.join(format!("RecCorder_{timestamp}.mp4"));

    let marker =
        watchdog::write_crash_marker(&output_dir_base, &file_path).map_err(|e| {
            state.set_recording(false);
            e.to_string()
        })?;
    let stop_flag = Arc::new(AtomicBool::new(false));

    let encoder = config_encoder;

    let session = match CaptureSession::start(file_path.clone(), monitor, mic_name.or(config_selected_mic), system_audio_device.or(config_selected_audio), configured_fps, scale, &encoder, webcam_cfg) {
        Ok(session) => session,
        Err(err) => {
            state.set_recording(false);
            watchdog::clear_crash_marker(&output_dir_base);
            let err_msg = err.to_string();
            println!("Erro critico no CaptureSession: {}", err_msg);
            return Err(err_msg);
        }
    };

    // Assumindo propriedade na estrutura sem Lock global excessivo
    *state.recording_start.lock() = Some(std::time::Instant::now());
    *state.current_file.lock() = Some(file_path.clone());
    *state.crash_marker.lock() = Some(marker);

    let mut guard = session_handle.lock();
    *guard = Some(ActiveSession { session, stop_flag });

    Ok(StartResult {
        file_path: file_path.to_string_lossy().into_owned(),
    })
}

/// Encerra a gravação da tela que está operando no processo global assincronamente.
/// // [IMPORTANTE] Evita o bloqueio da Thead principal do Tauri (UI) mandando o shutdown process pra um Worker isolado do Tokio.
#[tauri::command]
pub async fn stop_recording(
    state: State<'_, AppState>,
    session_handle: State<'_, SessionHandle>,
) -> Result<String, String> {
    if !state.recording() {
        return Err("Nenhuma gravacao em andamento".into());
    }

    // Retira a sessão do estado para processamento background de forma segura e limpa (sem panics de unwraps).
    let mut active = match session_handle.lock().take() {
        Some(s) => s,
        None => {
            state.set_recording(false);
            *state.recording_start.lock() = None;
            *state.current_file.lock() = None;
            return Err("A sessao de gravacao falhou por motivos externos (Nao encontrada no mutex)".into());
        }
    };
    
    // Sinaliza parada imediata das threads nativas de loopback C++ ou Alsa/PulseAudio
    active.stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);

    let stop_result = tokio::task::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_millis(200));
        active.session.stop()
    }).await.map_err(|e| format!("Erro interno no pool de finalizacao: {e}"))?;

    let file_path = state
        .current_file
        .lock()
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
    // Executa a finalização pesada do muxing (FFmpeg IO) em uma thead em blocking
    let stop_result = tokio::task::spawn_blocking(move || {

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

/// Realiza interrupção brusca (Force Exit) nos processos dependentes da gravação matando as instâncias via sinal KILL.
#[tauri::command]
pub fn force_exit(session_handle: State<'_, SessionHandle>) {
    if let Some(mut active) = session_handle.lock().take() {
        active.session.kill();
    }
    std::process::exit(0);
}

/// Recupera diretório em String para o path no front-end JS.
#[tauri::command]
pub fn get_output_dir(state: State<'_, AppState>) -> String {
    state.output_dir.lock().to_string_lossy().into_owned()
}

/// Modifica o path default global persistido.
#[tauri::command]
pub fn set_output_dir(app: AppHandle, state: State<'_, AppState>, path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        std::fs::create_dir_all(&p).map_err(|e| format!("Erro ao criar diretorio de gravação atual: {e}"))?;
    }
    
    let config = {
        let mut cfg = state.config.lock();
        cfg.output_dir = p.clone();
        let _ = cfg.save();
        cfg.clone()
    };
    
    *state.output_dir.lock() = p;
    
    let _ = app.emit("config-updated", config);
    
    Ok(())
}

/// Verifica se no disco sobrou algum rastro de gravacoes interrompidas forçadamente.
#[tauri::command]
pub fn check_crash_recovery(state: State<'_, AppState>) -> Option<String> {
    let output_dir = state.output_dir.lock().clone();
    watchdog::check_crash_recovery(&output_dir).map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn check_linux_deps() -> Result<Vec<String>, String> {
    #[cfg(target_os = "linux")]
    {
        use crate::services::capture::linux::validate_linux_system_deps;
        match validate_linux_system_deps() {
            Ok(_) => Ok(vec![]),
            Err(missing) => Ok(missing),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(vec![])
    }
}

#[tauri::command]
pub fn install_linux_deps(app_handle: AppHandle) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        use std::io::Write;
        
        let exe_path = std::env::current_exe().unwrap_or_default();
        let exe_path_str = exe_path.to_string_lossy().to_string();
        
        let script_content = format!(
r#"#!/bin/bash
echo "============================================="
echo " Instalando dependências do RecCorder... "
echo "============================================="
echo ""
echo "O aplicativo precisa de permissão para instalar pacotes essenciais de gravação."
echo ""

if command -v apt-get &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y ffmpeg x11-xserver-utils wireplumber pulseaudio-utils gawk
elif command -v dnf &> /dev/null; then
    sudo dnf install -y ffmpeg xrandr wireplumber pulseaudio-utils gawk
elif command -v pacman &> /dev/null; then
    sudo pacman -Sy --noconfirm ffmpeg xorg-xrandr wireplumber libpulse gawk
elif command -v zypper &> /dev/null; then
    sudo zypper install -y ffmpeg xrandr wireplumber pulseaudio-utils gawk
else
    echo "Gerenciador de pacotes não suportado. Instale manualmente: ffmpeg, xrandr, wpctl, pactl, awk."
    read -p "Pressione ENTER para sair..."
    exit 1
fi

echo ""
echo "Instalação concluída! Reiniciando o RecCorder..."
sleep 2

nohup "{}" > /dev/null 2>&1 &
exit 0
"#,
            exe_path_str
        );

        let script_path = "/tmp/reccorder_install_deps.sh";
        if let Ok(mut file) = std::fs::File::create(script_path) {
            let _ = file.write_all(script_content.as_bytes());
            let _ = Command::new("chmod").arg("+x").arg(script_path).status();
        }

        // Tenta abrir diferentes emuladores de terminal
        let terminals = ["x-terminal-emulator", "gnome-terminal", "konsole", "xfce4-terminal", "alacritty", "kitty", "xterm"];
        let mut spawned = false;
        
        for term in terminals {
            if term == "gnome-terminal" {
                let cmd = Command::new(term);
                let _ = cmd.arg("--").arg(script_path).spawn();
            } else {
                let cmd = Command::new(term);
                let _ = cmd.arg("-e").arg(script_path).spawn();
            }
        }
        app_handle.exit(0);
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = app_handle;
        Err("Comando válido apenas no Linux.".into())
    }
}
