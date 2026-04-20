use crate::errors::RecorderError;
use std::path::PathBuf;
use std::process::{Command, Stdio};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
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

    #[cfg(target_os = "linux")]
    {
        candidates.push(PathBuf::from("/usr/bin/ffmpeg"));
        candidates.push(PathBuf::from("/snap/bin/ffmpeg"));
        candidates.push(PathBuf::from("/usr/local/bin/ffmpeg"));
        if let Some(home) = std::env::var_os("HOME") {
            candidates.push(PathBuf::from(home).join(".local/bin/ffmpeg"));
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let prefixes = ["rec-corder-ffmpeg", "ffmpeg"];
            for prefix in prefixes {
                #[cfg(target_os = "windows")]
                {
                    candidates.push(exe_dir.join(format!("{}-x86_64-pc-windows-msvc.exe", prefix)));
                    candidates.push(exe_dir.join(format!("{}.exe", prefix)));
                    candidates.push(exe_dir.join(format!("{}-x86_64-unknown-linux-gnu", prefix)));
                    candidates.push(exe_dir.join(prefix));
                }
                #[cfg(not(target_os = "windows"))]
                {
                    candidates.push(exe_dir.join(format!("{}-x86_64-unknown-linux-gnu", prefix)));
                    candidates.push(exe_dir.join(prefix));
                    candidates.push(exe_dir.join(format!("{}-x86_64-pc-windows-msvc.exe", prefix)));
                    candidates.push(exe_dir.join(format!("{}.exe", prefix)));
                }
            }

            if let Some(target_dir) = exe_dir.parent() {
                for prefix in prefixes {
                    #[cfg(target_os = "windows")]
                    {
                        candidates.push(target_dir.join(format!("{}-x86_64-pc-windows-msvc.exe", prefix)));
                        candidates.push(target_dir.join(format!("{}.exe", prefix)));
                        candidates.push(target_dir.join(format!("{}-x86_64-unknown-linux-gnu", prefix)));
                        candidates.push(target_dir.join(prefix));
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        candidates.push(target_dir.join(format!("{}-x86_64-unknown-linux-gnu", prefix)));
                        candidates.push(target_dir.join(prefix));
                        candidates.push(target_dir.join(format!("{}-x86_64-pc-windows-msvc.exe", prefix)));
                        candidates.push(target_dir.join(format!("{}.exe", prefix)));
                    }
                }

                if let Some(project_dir) = target_dir.parent() {
                    for prefix in prefixes {
                        #[cfg(target_os = "windows")]
                        {
                            candidates.push(project_dir.join("bin").join(format!("{}-x86_64-pc-windows-msvc.exe", prefix)));
                            candidates.push(project_dir.join(format!("{}.exe", prefix)));
                            candidates.push(project_dir.join("bin").join(format!("{}-x86_64-unknown-linux-gnu", prefix)));
                            candidates.push(project_dir.join(prefix));
                        }
                        #[cfg(not(target_os = "windows"))]
                        {
                            candidates.push(project_dir.join("bin").join(format!("{}-x86_64-unknown-linux-gnu", prefix)));
                            candidates.push(project_dir.join(prefix));
                            candidates.push(project_dir.join("bin").join(format!("{}-x86_64-pc-windows-msvc.exe", prefix)));
                            candidates.push(project_dir.join(format!("{}.exe", prefix)));
                        }
                    }
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

    let error_msg = if cfg!(target_os = "windows") {
        format!("FFmpeg nao foi encontrado. Coloque-o em C:\\Users\\<nome_usuario>\\AppData\\Local\\RecCorder\\ffmpeg.exe ou adicione-o ao PATH. Locais verificados: {}", searched.join(" | "))
    } else {
        format!("FFmpeg nao foi encontrado. Instale-o via 'sudo apt install ffmpeg' ou coloque-o no PATH. Locais verificados: {}", searched.join(" | "))
    };

    Err(RecorderError::CaptureInit(error_msg))
}

pub fn build_video_input(
    monitor_handle: Option<isize>,
    monitor_index: usize,
    window_handle: Option<isize>,
    fps: u32,
) -> String {
    #[cfg(target_os = "windows")]
    {
        if let Some(hwnd) = window_handle {
            format!("gfxcapture=window_handle={:#x}:max_framerate={fps}:capture_cursor=1:display_border=0", hwnd)
        } else if let Some(hmonitor) = monitor_handle {
            format!("gfxcapture=hmonitor={hmonitor}:max_framerate={fps}:capture_cursor=1:display_border=0")
        } else {
            format!("gfxcapture=monitor_idx={monitor_index}:max_framerate={fps}:capture_cursor=1:display_border=0")
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = monitor_handle;
        let _ = window_handle;
        let _ = fps;
        
        let mut x = 0;
        let mut y = 0;
        
        if let Ok(monitors) = crate::services::capture::linux::enumerate_linux_monitors() {
            if let Some(mon) = monitors.iter().find(|m| m.index == monitor_index) {
                x = mon.bounds.0;
                y = mon.bounds.1;
            } else if let Some(mon) = monitors.first() {
                x = mon.bounds.0;
                y = mon.bounds.1;
            }
        }
        
        format!(":0.0+{},{}", x, y)
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
    
    #[cfg(target_os = "windows")]
    cmd.args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", &video_input]);

    #[cfg(not(target_os = "windows"))]
    {
        let mut width = 1920;
        let mut height = 1080;
        
        if let Ok(monitors) = crate::services::capture::linux::enumerate_linux_monitors() {
            if let Some(mon) = monitors.iter().find(|m| m.index == monitor_index) {
                width = mon.bounds.2;
                height = mon.bounds.3;
            } else if let Some(mon) = monitors.first() {
                width = mon.bounds.2;
                height = mon.bounds.3;
            }
        }
        
        let video_size = format!("{}x{}", width, height);
        
        // Exemplo: "-video_size 1920x1080 -framerate 30 -i :0.0+0,0"
        cmd.args([
            "-hide_banner", "-loglevel", "error", 
            "-f", "x11grab", 
            "-video_size", &video_size,
            "-framerate", &fps.to_string(), 
            "-i", &video_input
        ]);
    }
}

pub fn build_capture_filter(scale_factor: u32, fps: u32, pixel_format: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        if scale_factor >= 100 {
            format!("hwdownload,format=bgra,fps={fps},format={pixel_format}")
        } else {
            let f = scale_factor as f32 / 100.0;
            format!(
                "hwdownload,format=bgra,fps={fps},scale=trunc(iw*{f}/2)*2:trunc(ih*{f}/2)*2,format={pixel_format}"
            )
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if scale_factor >= 100 {
            format!("fps={fps},format={pixel_format}")
        } else {
            let f = scale_factor as f32 / 100.0;
            format!(
                "fps={fps},scale=trunc(iw*{f}/2)*2:trunc(ih*{f}/2)*2,format={pixel_format}"
            )
        }
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

/// Add a DirectShow webcam as the second video input to the FFmpeg command.
pub fn append_webcam_input(cmd: &mut Command, device_name: &str) {
    #[cfg(target_os = "windows")]
    {
        cmd.args([
            "-f", "dshow",
            "-video_size", "640x480",
            "-framerate", "30",
            "-i",
        ]);
        cmd.arg(format!("video={}", device_name));
    }
    #[cfg(not(target_os = "windows"))]
    {
        cmd.args([
            "-f", "v4l2",
            "-video_size", "640x480",
            "-framerate", "30",
            "-i",
        ]);
        cmd.arg(device_name);
    }
}

/// Build the video filter string with webcam overlay composited onto the screen capture.
///
/// - `base_vf`: The existing video filter string (e.g. "hwdownload,format=bgra,fps=60,format=yuv420p")
/// - `position`: "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center"
/// - `size_percent`: Percentage of base size (200px). Range 50–300.
///
/// Returns the full `-filter_complex` value to replace the simple `-vf`.
pub fn build_webcam_overlay_filter(
    base_vf: &str,
    position: &str,
    size_percent: u32,
) -> String {
    let overlay_width = 200 * size_percent / 100;
    let overlay_height = overlay_width * 3 / 4; // 4:3 aspect ratio

    let position_expr = match position {
        "top-left"     => "0:0".to_string(),
        "top-right"    => format!("W-{overlay_width}:0"),
        "bottom-left"  => format!("0:H-{overlay_height}"),
        "center"       => format!("(W-{overlay_width})/2:(H-{overlay_height})/2"),
        _              => format!("W-{overlay_width}:H-{overlay_height}"), // default: bottom-right
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

    let targets = vec![
        EncoderStrategy::NvidiaNvenc,
        EncoderStrategy::AmdAmf,
        EncoderStrategy::IntelQsv,
    ];

    for target in targets {
        let label = target.label();

        let mut cmd = Command::new(&ffmpeg_path);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        
        cmd.args([
            "-f", "lavfi",
            "-i", "nullsrc=s=128x128:d=0.1",
            "-c:v", label,
            "-f", "null",
            "-"
        ]);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        if let Ok(mut child) = cmd.spawn() {
            if let Ok(status) = child.wait() {
                if status.success() {
                    println!("Hardware test successful for {}", label);
                    return label.to_string();
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
    fn build_video_input_uses_correct_platform_string() {
        #[cfg(target_os = "windows")]
        {
            assert_eq!(
                build_video_input(None, 0, None, 60),
                "gfxcapture=monitor_idx=0:max_framerate=60:capture_cursor=1:display_border=0"
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            let input = build_video_input(None, 0, None, 60);
            assert!(input.starts_with(":0.0+"));
        }
    }

    #[test]
    fn build_capture_filter_forces_constant_fps_before_encoding() {
        #[cfg(target_os = "windows")]
        assert_eq!(
            build_capture_filter(100, 60, "nv12"),
            "hwdownload,format=bgra,fps=60,format=nv12"
        );
        #[cfg(not(target_os = "windows"))]
        assert_eq!(
            build_capture_filter(100, 60, "nv12"),
            "fps=60,format=nv12"
        );
    }
}
