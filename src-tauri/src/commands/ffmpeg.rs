use crate::services::capture::ffmpeg::candidate_ffmpeg_paths;
use serde::Serialize;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize)]
pub struct FfmpegStatus {
    pub found: bool,
    pub path: Option<String>,
}

fn check_capture_backends(ffmpeg_path: &PathBuf) -> bool {
    let mut filter_cmd = Command::new(ffmpeg_path);
    filter_cmd.creation_flags(CREATE_NO_WINDOW);
    filter_cmd.args(["-hide_banner", "-filters"]);

    if let Ok(output) = filter_cmd.output() {
        let combined_output = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if combined_output.contains("ddagrab") || combined_output.contains("gfxcapture") {
            return true;
        }
    }

    let mut device_cmd = Command::new(ffmpeg_path);
    device_cmd.creation_flags(CREATE_NO_WINDOW);
    device_cmd.args(["-hide_banner", "-devices"]);

    if let Ok(output) = device_cmd.output() {
        let combined_output = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        combined_output.contains("gdigrab")
    } else {
        false
    }
}

/// Verifica se o FFmpeg esta disponivel no sistema e possui um backend de captura compativel.
#[tauri::command]
pub fn check_ffmpeg() -> FfmpegStatus {
    for candidate in candidate_ffmpeg_paths() {
        if candidate.is_file() && check_capture_backends(&candidate) {
            return FfmpegStatus {
                found: true,
                path: Some(candidate.to_string_lossy().to_string()),
            };
        }
    }

    FfmpegStatus {
        found: false,
        path: None,
    }
}
