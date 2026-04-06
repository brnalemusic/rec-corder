pub mod ffmpeg;
pub mod session;
pub mod windows;

pub use ffmpeg::test_environment;
pub use session::CaptureSession;
pub use windows::{list_monitors, list_mic_devices, list_audio_outputs, resolve_monitor_index, MonitorInfo, MicInfo, AudioOutputInfo};
