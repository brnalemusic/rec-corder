use std::path::PathBuf;
use std::process::Command;
use serde::Serialize;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use crate::services::capture::ffmpeg::candidate_ffmpeg_paths;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize)]
pub struct FfmpegStatus {
    pub found: bool,
    pub path: Option<String>,
}

fn verify_ffmpeg_binary(ffmpeg_path: &PathBuf) -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new(ffmpeg_path);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.args([
            "-f", "lavfi",
            "-i", "gfxcapture=list_sources=true",
            "-vframes", "1",
            "-f", "null",
            "-"
        ]);
        if let Ok(output) = cmd.output() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            !stderr.contains("No such filter")
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new(ffmpeg_path);
        cmd.arg("-version");
        if let Ok(output) = cmd.output() {
            output.status.success()
        } else {
            false
        }
    }
}

/// Verifica se o FFmpeg está disponível no sistema e possui os filtros necessários
#[tauri::command]
pub fn check_ffmpeg() -> FfmpegStatus {
    for candidate in candidate_ffmpeg_paths() {
        if candidate.is_file() && verify_ffmpeg_binary(&candidate) {
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
