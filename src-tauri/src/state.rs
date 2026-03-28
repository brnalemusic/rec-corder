use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use parking_lot::Mutex;

/// Central application state managed by Tauri.
/// Uses atomics for lock-free status checks from the frontend.
pub struct AppState {
    pub is_recording: AtomicBool,
    pub recording_start: Mutex<Option<Instant>>,
    pub output_dir: Mutex<PathBuf>,
    pub current_file: Mutex<Option<PathBuf>>,

    /// Persisted flag for crash recovery
    pub crash_marker: Mutex<Option<PathBuf>>,
    pub config: Mutex<crate::config::AppConfig>,
}

impl AppState {
    pub fn new() -> Self {
        let config = crate::config::AppConfig::load();
        let output = config.output_dir.clone();

        Self {
            is_recording: AtomicBool::new(false),
            recording_start: Mutex::new(None),
            output_dir: Mutex::new(output),
            current_file: Mutex::new(None),
            crash_marker: Mutex::new(None),
            config: Mutex::new(config),
        }
    }

    pub fn recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    pub fn set_recording(&self, val: bool) {
        self.is_recording.store(val, Ordering::Relaxed);
    }
    pub fn elapsed_secs(&self) -> u64 {
        self.recording_start
            .lock()
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
