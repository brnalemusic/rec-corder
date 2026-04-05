import * as recorder from './recorder.js';

const PREFS_KEY = 'rec_corder_prefs';

export async function loadAndMigratePrefs() {
  try {
    let config = await recorder.getConfig();
    const saved = localStorage.getItem(PREFS_KEY);
    if (saved) {
      console.log('Migrating legacy prefs to backend...');
      try {
        const prefs = JSON.parse(saved);
        config.mic_enabled = !!prefs.micEnabled;
        config.sys_audio_enabled = prefs.sys_audio_enabled !== undefined ? !!prefs.sysAudioEnabled : true;
        config.selected_monitor = parseInt(prefs.selectedMonitor || 0, 10);
        config.fps = parseInt(prefs.currentFps || 60, 10);
        config.scale = parseInt(prefs.currentScale || 100, 10);
        config.selected_audio_output = prefs.selectedAudioOutput || null;
        if (prefs.outputDir) config.output_dir = prefs.outputDir;
        
        await recorder.updateConfig(config);
        localStorage.removeItem(PREFS_KEY);
      } catch (e) {
        console.warn('Error parsing legacy prefs:', e);
      }
    }
    return config;
  } catch (e) {
    console.warn('Failed to load/migrate prefs from backend:', e);
    return null;
  }
}

export async function savePrefs(config) {
  try {
    await recorder.updateConfig(config);
  } catch (e) {
    console.warn('Failed to save config to backend:', e);
  }
}
