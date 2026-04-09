/**
 * Rec Corder — Tauri command bridge
 * Typed wrappers for every Rust command. 
 * Single source of truth for the frontend ↔ backend contract.
 */

const { invoke } = window.__TAURI__.core;

/**
 * @typedef {{ is_recording: boolean, elapsed_secs: number, output_file: string|null }} RecordingStatus
 * @typedef {{ file_path: string }} StartResult
 * @typedef {{ index: number, name: string, is_primary: boolean }} MonitorInfo
 * @typedef {{ id: string, name: string, is_default: boolean }} MicInfo
 * @typedef {{ id: string, name: string, is_default: boolean }} AudioOutputInfo
 */

/** Get current recording status. */
export async function getStatus() {
  return invoke('get_status');
}

/** Get the central configuration. */
export async function getConfig() {
  return invoke('get_config');
}

/** 
 * Update the central configuration.
 * @param {object} config 
 */
export async function updateConfig(config) {
  return invoke('update_config', { config });
}

/** List available monitors. */
export async function listMonitors() {
  return invoke('list_monitors');
}

/** List available microphones. */
export async function listMics() {
  return invoke('list_mics');
}

/**
 * Start recording.
 * @param {object} options
 * @param {number} [options.monitorIndex=0]
 * @param {string|null} [options.micName=null]
 * @param {string|null} [options.systemAudioDevice=null]
 * @param {number} [options.fps=60]
 * @param {number} [options.scaleFactor=100]
 * @returns {Promise<StartResult>}
 */
export async function startRecording({ 
  monitorIndex = 0, 
  micName = null, 
  systemAudioDevice = null, 
  fps = 60, 
  scaleFactor = 100 
} = {}) {
  return invoke('start_recording', { monitorIndex, micName, systemAudioDevice, fps, scaleFactor });
}

/** List available audio outputs (wasapi render devices). */
export async function listAudioOutputs() {
  return invoke('list_audio_outputs');
}

/** Stop recording. Returns the output file path. */
export async function stopRecording() {
  return invoke('stop_recording');
}

/** Get the current output directory. */
export async function getOutputDir() {
  return invoke('get_output_dir');
}

/**
 * Set a new output directory.
 * @param {string} path
 */
export async function setOutputDir(path) {
  return invoke('set_output_dir', { path });
}

/** Show the settings window. */
export async function showSettings() {
  return invoke('show_settings');
}

/** Hide the settings window without destroying the webview. */
export async function hideSettings() {
  return invoke('hide_settings');
}

/** Check for incomplete recordings from a crash. */
export async function check_crash_recovery() {
  return invoke('check_crash_recovery');
}

/** Get application info and first-run status. */
export async function getAppInfo() {
  return invoke('get_app_info');
}

/** Mark welcome popup as acknowledged. */
export async function acknowledgeWelcome() {
  return invoke('acknowledge_welcome');
}

/** Get the application version from Tauri. */
export async function getAppVersion() {
  try {
    return await window.__TAURI__.app.getVersion();
  } catch (e) {
    console.warn('Failed to get version from Tauri API, falling back to backend:', e);
    const info = await getAppInfo();
    return info.version;
  }
}


/** Check for updates via backend (returns version string or null). */
export async function checkForUpdates() {
  return invoke('check_for_updates');
}

/** Show the updater window. */
export async function showUpdater(version, body = null) {
  return invoke('show_updater', { version, body });
}

/** Get release notes for a specific version. */
export async function getReleaseNotes(version) {
  return invoke('get_release_notes', { version });
}

/** Show the release notes window. */
export async function showReleaseNotes() {
  return invoke('show_release_notes');
}

/** Install the pending update (handled by backend). */
export async function installUpdate() {
  return invoke('install_update');
}

/** @typedef {{ name: string, id: string }} CameraInfo */

/** List available cameras (DirectShow video devices). */
export async function listCameras() {
  return invoke('list_cameras');
}
