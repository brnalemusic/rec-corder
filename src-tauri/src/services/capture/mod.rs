pub mod ffmpeg;
pub mod linux;
pub mod session;
pub mod windows;

pub use ffmpeg::test_environment;
pub use session::CaptureSession;

use serde::Serialize;

/// Informações sobre uma câmera de vídeo.
#[derive(Serialize, Clone, Debug)]
pub struct CameraInfo {
    /// Nome amigável da câmera.
    pub name: String,
    /// Identificador único da câmera (ex: caminho no Linux, nome do dispositivo no Windows).
    pub id: String,
}

#[cfg(target_os = "windows")]
pub use windows::{
    list_audio_outputs, list_cameras, list_mic_devices, list_monitors, resolve_monitor_index,
    AudioOutputInfo, MicInfo, MonitorInfo,
};

// [IMPORTANTE] A exportação no Linux também expõe os tipos que existem primariamente no arquivo windows.rs 
// para manter a consistência de API entre sistemas, mesmo quando algumas funções não estão disponíveis.
#[cfg(not(target_os = "windows"))]
pub use linux::list_cameras;

#[cfg(not(target_os = "windows"))]
pub use windows::{
    list_audio_outputs, list_mic_devices, list_monitors, resolve_monitor_index, AudioOutputInfo,
    MicInfo, MonitorInfo,
};
