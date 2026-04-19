use crate::errors::RecorderError;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EncoderStrategy {
    NvidiaNvenc,
    AmdAmf,
    IntelQsv,
    SoftwareX264,
}

impl EncoderStrategy {
    pub fn label(self) -> &'static str {
        match self {
            Self::NvidiaNvenc => "h264_nvenc",
            Self::AmdAmf => "h264_amf",
            Self::IntelQsv => "h264_qsv",
            Self::SoftwareX264 => "libx264",
        }
    }

    pub fn from_label(label: &str) -> Self {
        match label {
            "h264_nvenc" => Self::NvidiaNvenc,
            "h264_amf" => Self::AmdAmf,
            "h264_qsv" => Self::IntelQsv,
            _ => Self::SoftwareX264,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureBackend {
    DesktopDuplication,
    GraphicsCapture,
    GdiGrab,
}

impl CaptureBackend {
    pub fn label(self) -> &'static str {
        match self {
            Self::DesktopDuplication => "ddagrab",
            Self::GraphicsCapture => "gfxcapture",
            Self::GdiGrab => "gdigrab",
        }
    }

    pub fn requires_hwdownload(self) -> bool {
        matches!(self, Self::DesktopDuplication | Self::GraphicsCapture)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureInput {
    pub backend: CaptureBackend,
    args: Vec<String>,
}

impl CaptureInput {
    pub fn apply(&self, cmd: &mut Command) {
        cmd.args(&self.args);
    }
}

pub fn candidate_ffmpeg_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // Primeira prioridade: variável de ambiente customizada
    if let Some(explicit_path) = std::env::var_os("REC_CORDER_FFMPEG_PATH") {
        candidates.push(PathBuf::from(explicit_path));
    }

    // Segunda prioridade: pasta de AppData do Rec Corder (instalação automática)
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local_app_data).join("RecCorder\\ffmpeg.exe"));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("ffmpeg-x86_64-pc-windows-msvc.exe")); // Sidecar embutido
            candidates.push(exe_dir.join("ffmpeg.exe"));

            if let Some(target_dir) = exe_dir.parent() {
                candidates.push(target_dir.join("ffmpeg-x86_64-pc-windows-msvc.exe"));
                candidates.push(target_dir.join("ffmpeg.exe"));

                if let Some(project_dir) = target_dir.parent() {
                    candidates.push(
                        project_dir
                            .join("bin")
                            .join("ffmpeg-x86_64-pc-windows-msvc.exe"),
                    );
                    candidates.push(project_dir.join("ffmpeg.exe"));
                }
            }
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("ffmpeg.exe"));
    }

    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ffmpeg.exe"));

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            candidates.push(dir.join("ffmpeg.exe"));
        }
    }

    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local_app_data).join("Microsoft\\WinGet\\Links\\ffmpeg.exe"));
    }

    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let user_profile = PathBuf::from(user_profile);
        candidates.push(user_profile.join("scoop\\shims\\ffmpeg.exe"));
        candidates.push(user_profile.join("ffmpeg\\bin\\ffmpeg.exe"));
    }

    if let Some(choco_root) = std::env::var_os("ChocolateyInstall") {
        candidates.push(PathBuf::from(choco_root).join("bin\\ffmpeg.exe"));
    }

    candidates.push(PathBuf::from(r"C:\ffmpeg\bin\ffmpeg.exe"));
    candidates.push(PathBuf::from(r"C:\ffmpeg\ffmpeg.exe"));
    candidates.push(PathBuf::from(r"C:\Program Files\ffmpeg\bin\ffmpeg.exe"));
    candidates.push(PathBuf::from(
        r"C:\Program Files (x86)\ffmpeg\bin\ffmpeg.exe",
    ));

    candidates
}

pub fn resolve_ffmpeg_path() -> Result<PathBuf, RecorderError> {
    let mut searched = Vec::new();

    for candidate in candidate_ffmpeg_paths() {
        let candidate_str = candidate.to_string_lossy().to_string();
        if searched.iter().any(|existing| existing == &candidate_str) {
            continue;
        }

        if candidate.is_file() {
            println!("FFmpeg encontrado em {:?}", candidate);
            return Ok(candidate);
        }

        searched.push(candidate_str);
    }

    Err(RecorderError::CaptureInit(format!(
        "FFmpeg nao foi encontrado. Coloque-o em C:\\Users\\<nome_usuario>\\AppData\\Local\\RecCorder\\ffmpeg.exe ou adicione-o ao PATH. Você também pode definir REC_CORDER_FFMPEG_PATH. Locais verificados: {}",
        searched.join(" | ")
    )))
}

pub fn build_video_input(
    monitor_handle: Option<isize>,
    monitor_index: usize,
    window_handle: Option<isize>,
    fps: u32,
) -> String {
    // Prefer capturing the whole monitor (hmonitor) when available. Capturing a
    // specific window handle can fail for exclusive fullscreen DirectX games,
    // so prefer the monitor capture for better compatibility with fullscreen apps.
    if let Some(hmonitor) = monitor_handle {
        format!(
            "gfxcapture=hmonitor={hmonitor}:max_framerate={fps}:capture_cursor=1:display_border=0"
        )
    } else if let Some(hwnd) = window_handle {
        format!(
            "gfxcapture=window_handle={:#x}:max_framerate={fps}:capture_cursor=1:display_border=0",
            hwnd
        )
    } else {
        format!("gfxcapture=monitor_idx={monitor_index}:max_framerate={fps}:capture_cursor=1:display_border=0")
    }
}

pub fn build_gfxcapture_input(
    monitor_handle: Option<isize>,
    monitor_index: usize,
    window_handle: Option<isize>,
    fps: u32,
) -> CaptureInput {
    CaptureInput {
        backend: CaptureBackend::GraphicsCapture,
        args: vec![
            "-f".into(),
            "lavfi".into(),
            "-i".into(),
            build_video_input(monitor_handle, monitor_index, window_handle, fps),
        ],
    }
}

pub fn build_ddagrab_input(output_idx: usize, fps: u32) -> CaptureInput {
    CaptureInput {
        backend: CaptureBackend::DesktopDuplication,
        args: vec![
            "-f".into(),
            "lavfi".into(),
            "-i".into(),
            format!("ddagrab=output_idx={output_idx}:framerate={fps}:draw_mouse=1"),
        ],
    }
}

pub fn build_gdigrab_input(left: i32, top: i32, width: i32, height: i32, fps: u32) -> CaptureInput {
    CaptureInput {
        backend: CaptureBackend::GdiGrab,
        args: vec![
            "-f".into(),
            "gdigrab".into(),
            "-framerate".into(),
            fps.to_string(),
            "-offset_x".into(),
            left.to_string(),
            "-offset_y".into(),
            top.to_string(),
            "-video_size".into(),
            format!("{width}x{height}"),
            "-draw_mouse".into(),
            "1".into(),
            "-i".into(),
            "desktop".into(),
        ],
    }
}

pub fn append_video_input(cmd: &mut Command, input: &CaptureInput) {
    cmd.args(["-hide_banner", "-loglevel", "error"]);
    input.apply(cmd);
}

pub fn build_capture_filter(
    scale_factor: u32,
    fps: u32,
    pixel_format: &str,
    capture_backend: CaptureBackend,
) -> String {
    let mut filters = Vec::new();

    if capture_backend.requires_hwdownload() {
        filters.push("hwdownload".to_string());
        filters.push("format=bgra".to_string());
    }

    filters.push(format!("fps={fps}"));

    if scale_factor < 100 {
        let f = scale_factor as f32 / 100.0;
        filters.push(format!("scale=trunc(iw*{f}/2)*2:trunc(ih*{f}/2)*2"));
    }

    filters.push(format!("format={pixel_format}"));
    filters.join(",")
}

pub fn append_encoder_args(
    cmd: &mut Command,
    strategy: EncoderStrategy,
    fps: u32,
    scale_factor: u32,
    capture_backend: CaptureBackend,
    enable_faststart: bool,
) {
    let vf_amf = build_capture_filter(scale_factor, fps, "nv12", capture_backend);
    let vf_x264 = build_capture_filter(scale_factor, fps, "yuv420p", capture_backend);

    match strategy {
        EncoderStrategy::AmdAmf => {
            cmd.args([
                "-vf",
                &vf_amf,
                "-c:v",
                "h264_amf",
                "-usage",
                "lowlatency",
                "-quality",
                "speed",
                "-rc",
                "cbr",
                "-b:v",
                "5M",
                "-pix_fmt",
                "nv12",
            ]);
        }
        EncoderStrategy::NvidiaNvenc => {
            cmd.args([
                "-vf",
                &vf_x264,
                "-c:v",
                "h264_nvenc",
                "-preset",
                "p4",
                "-tune",
                "ull",
                "-rc",
                "vbr",
                "-cq",
                "23",
                "-pix_fmt",
                "yuv420p",
            ]);
        }
        EncoderStrategy::IntelQsv => {
            cmd.args([
                "-vf",
                &vf_x264,
                "-c:v",
                "h264_qsv",
                "-preset",
                "veryfast",
                "-global_quality",
                "23",
                "-pix_fmt",
                "nv12", // Intel QSV often likes nv12
            ]);
        }
        EncoderStrategy::SoftwareX264 => {
            cmd.args([
                "-vf",
                &vf_x264,
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-crf",
                "23",
                "-pix_fmt",
                "yuv420p",
            ]);
        }
    }

    if enable_faststart {
        cmd.args(["-movflags", "+faststart"]);
    }
}

/// Add a DirectShow webcam as the second video input to the FFmpeg command.
pub fn append_webcam_input(cmd: &mut Command, device_name: &str) {
    cmd.args([
        "-f",
        "dshow",
        "-video_size",
        "640x480",
        "-framerate",
        "30",
        "-i",
    ]);
    cmd.arg(format!("video={}", device_name));
}

/// Build the video filter string with webcam overlay composited onto the screen capture.
///
/// - `base_vf`: The existing video filter string (e.g. "hwdownload,format=bgra,fps=60,format=yuv420p")
/// - `position`: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center"
/// - `size_percent`: Percentage of base size (200px). Range 50–300.
///
/// Returns the full `-filter_complex` value to replace the simple `-vf`.
pub fn build_webcam_overlay_filter(base_vf: &str, position: &str, size_percent: u32) -> String {
    let overlay_width = 200 * size_percent / 100;
    let overlay_height = overlay_width * 3 / 4; // 4:3 aspect ratio

    let position_expr = match position {
        "top-left" => "0:0".to_string(),
        "top-right" => format!("W-{overlay_width}:0"),
        "bottom-left" => format!("0:H-{overlay_height}"),
        "center" => format!("(W-{overlay_width})/2:(H-{overlay_height})/2"),
        _ => format!("W-{overlay_width}:H-{overlay_height}"), // default: bottom-right
    };

    format!(
        "[0:v]{base_vf}[vid];[1:v]scale={overlay_width}:{overlay_height}[cam];[vid][cam]overlay={position_expr}[out]"
    )
}

pub fn test_environment() -> String {
    let ffmpeg_path = match resolve_ffmpeg_path() {
        Ok(path) => path,
        Err(_) => return EncoderStrategy::SoftwareX264.label().to_string(),
    };

    let strategies = [
        EncoderStrategy::NvidiaNvenc,
        EncoderStrategy::AmdAmf,
        EncoderStrategy::IntelQsv,
    ];

    for strategy in strategies {
        let mut cmd = Command::new(&ffmpeg_path);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.args([
            "-f",
            "lavfi",
            "-i",
            "nullsrc=s=128x128:d=0.1",
            "-c:v",
            strategy.label(),
            "-f",
            "null",
            "-",
        ]);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        if let Ok(mut child) = cmd.spawn() {
            if let Ok(status) = child.wait() {
                if status.success() {
                    println!("Hardware test successful for {}", strategy.label());
                    return strategy.label().to_string();
                }
            }
        }
    }

    println!("Hardware tests failed, falling back to libx264");
    EncoderStrategy::SoftwareX264.label().to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        build_capture_filter, build_ddagrab_input, build_gdigrab_input, build_video_input,
        CaptureBackend,
    };

    #[test]
    fn build_video_input_uses_supported_gfxcapture_options() {
        assert_eq!(
            build_video_input(None, 0, None, 60),
            "gfxcapture=monitor_idx=0:max_framerate=60:capture_cursor=1:display_border=0"
        );
        assert_eq!(
            build_video_input(None, 0, Some(0x1234), 60),
            "gfxcapture=window_handle=0x1234:max_framerate=60:capture_cursor=1:display_border=0"
        );
        assert_eq!(
            build_video_input(Some(456), 0, None, 60),
            "gfxcapture=hmonitor=456:max_framerate=60:capture_cursor=1:display_border=0"
        );
    }

    #[test]
    fn build_capture_filter_forces_constant_fps_before_encoding() {
        assert_eq!(
            build_capture_filter(100, 60, "nv12", CaptureBackend::GraphicsCapture),
            "hwdownload,format=bgra,fps=60,format=nv12"
        );
    }

    #[test]
    fn build_capture_filter_skips_hwdownload_for_gdigrab() {
        assert_eq!(
            build_capture_filter(80, 60, "yuv420p", CaptureBackend::GdiGrab),
            "fps=60,scale=trunc(iw*0.8/2)*2:trunc(ih*0.8/2)*2,format=yuv420p"
        );
    }

    #[test]
    fn build_ddagrab_input_draws_mouse() {
        let input = build_ddagrab_input(2, 60);
        let debug = format!("{input:?}");
        assert!(debug.contains("DesktopDuplication"));
        assert!(debug.contains("ddagrab=output_idx=2:framerate=60:draw_mouse=1"));
    }

    #[test]
    fn build_gdigrab_input_uses_monitor_region() {
        let input = build_gdigrab_input(10, 20, 1920, 1080, 30);
        let debug = format!("{input:?}");
        assert!(debug.contains("GdiGrab"));
        assert!(debug.contains("1920x1080"));
        assert!(debug.contains("desktop"));
    }
}
