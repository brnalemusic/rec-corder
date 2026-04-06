use crate::errors::RecorderError;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::os::windows::process::CommandExt;

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
                    candidates.push(project_dir.join("bin").join("ffmpeg-x86_64-pc-windows-msvc.exe"));
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
    if let Some(hwnd) = window_handle {
        format!("gfxcapture=window_handle={:#x}:max_framerate={fps}:capture_cursor=1:display_border=0", hwnd)
    } else if let Some(hmonitor) = monitor_handle {
        format!("gfxcapture=hmonitor={hmonitor}:max_framerate={fps}:capture_cursor=1:display_border=0")
    } else {
        format!("gfxcapture=monitor_idx={monitor_index}:max_framerate={fps}:capture_cursor=1:display_border=0")
    }
}

pub fn append_common_inputs(
    cmd: &mut Command,
    monitor_handle: Option<isize>,
    monitor_index: usize,
    window_handle: Option<isize>,
    fps: u32,
) {
    let video_input = build_video_input(monitor_handle, monitor_index, window_handle, fps);
    cmd.args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i"]);
    cmd.arg(video_input);
    cmd.args(["-map", "0:v:0"]);
}

pub fn build_capture_filter(scale_factor: u32, fps: u32, pixel_format: &str) -> String {
    if scale_factor >= 100 {
        format!("hwdownload,format=bgra,fps={fps},format={pixel_format}")
    } else {
        let f = scale_factor as f32 / 100.0;
        format!(
            "hwdownload,format=bgra,fps={fps},scale=trunc(iw*{f}/2)*2:trunc(ih*{f}/2)*2,format={pixel_format}"
        )
    }
}

pub fn append_encoder_args(
    cmd: &mut Command,
    strategy: EncoderStrategy,
    fps: u32,
    scale_factor: u32,
    enable_faststart: bool,
) {
    let vf_amf = build_capture_filter(scale_factor, fps, "nv12");
    let vf_x264 = build_capture_filter(scale_factor, fps, "yuv420p");

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
            "-f", "lavfi",
            "-i", "nullsrc=s=128x128:d=0.1",
            "-c:v", strategy.label(),
            "-f", "null",
            "-"
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
    use super::{build_capture_filter, build_video_input};

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
            build_capture_filter(100, 60, "nv12"),
            "hwdownload,format=bgra,fps=60,format=nv12"
        );
    }
}
