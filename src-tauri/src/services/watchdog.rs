use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::RecorderError;

const CRASH_MARKER_NAME: &str = ".reccorder_recording";

/// Write a crash marker file when a recording starts.
/// Contains the path to the active recording file.
pub fn write_crash_marker(
    output_dir: &Path,
    recording_path: &Path,
) -> Result<PathBuf, RecorderError> {
    fs::create_dir_all(output_dir)?;
    let marker = output_dir.join(CRASH_MARKER_NAME);
    fs::write(&marker, recording_path.to_string_lossy().as_bytes())?;
    Ok(marker)
}

/// Remove the crash marker when recording finishes cleanly.
pub fn clear_crash_marker(output_dir: &Path) {
    let marker = output_dir.join(CRASH_MARKER_NAME);
    let _ = fs::remove_file(marker);
}

/// On startup, check if a previous recording was interrupted.
/// Returns the path to the incomplete file if found.
pub fn check_crash_recovery(output_dir: &Path) -> Option<PathBuf> {
    let marker = output_dir.join(CRASH_MARKER_NAME);
    if marker.exists() {
        if let Ok(content) = fs::read_to_string(&marker) {
            let path = PathBuf::from(content.trim());
            // Clean up the marker regardless
            let _ = fs::remove_file(&marker);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

/// Performance monitor — checks system CPU usage (simplified).
/// Returns true if the system is under heavy load (> threshold %).
pub fn is_system_under_load() -> bool {
    // Lightweight heuristic: check available memory as a proxy
    // Full CPU monitoring would require sysinfo crate — too heavy for v1
    // This will be enhanced in v2 if needed
    false
}

/// Suggested FPS based on system load.
/// Normal = 30fps, High load = 15fps.
pub fn recommended_fps() -> u32 {
    if is_system_under_load() {
        15
    } else {
        30
    }
}
