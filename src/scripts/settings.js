/**
 * Rec Corder — Settings Window
 * Manages the advanced configuration panel in a separate window.
 */

import * as Recorder from './recorder.js';

// State
const state = {
  config: null,
  monitors: [],
  mics: [],
  audioOutputs: [],
  isDirty: false,
};

// DOM Elements
const elements = {
  // Tabs
  navBtns: document.querySelectorAll('.nav-btn'),
  tabContents: document.querySelectorAll('.tab-content'),

  // Video
  monitorSelect: document.getElementById('monitorSelect'),
  fpsBtns: document.querySelectorAll('.fps-btn'),
  scaleSlider: document.getElementById('scaleSlider'),
  scaleValue: document.getElementById('scaleValue'),

  // Audio
  micSelect: document.getElementById('micSelect'),
  audioOutputSelect: document.getElementById('audioOutputSelect'),
  sysAudioToggle: document.getElementById('sysAudioToggle'),
  micVolumeSlider: document.getElementById('micVolumeSlider'),
  micVolumeValue: document.getElementById('micVolumeValue'),

  // Output
  outputPath: document.getElementById('outputPath'),

  // Actions
  saveBtn: document.getElementById('saveBtn'),
  closeBtn: document.getElementById('closeBtn'),
  resetBtn: document.getElementById('resetBtn'),

  // Status
  statusIndicator: document.getElementById('statusIndicator'),
  statusMessage: document.getElementById('statusMessage'),
};

/**
 * Initialize the settings window
 */
async function init() {
  try {
    setStatusLoading('Carregando...');

    // Load config and data
    await Promise.all([
      loadConfig(),
      loadMonitors(),
      loadMics(),
      loadAudioOutputs(),
      loadOutputDir(),
    ]);

    // Setup event listeners
    setupEventListeners();

    // Listen for updates from other windows
    try {
      const { listen } = window.__TAURI__.event;
      await listen('config-updated', (event) => {
        console.log('Settings: config updated from backend:', event.payload);
        state.config = event.payload;
        updateUIFromConfig();
      });
    } catch (e) {
      console.warn('Failed to setup config-updated listener in settings:', e);
    }

    setStatusSuccess('Pronto');
  } catch (error) {
    console.error('Error initializing settings:', error);
    setStatusError('Erro');
  }
}

/**
 * Load current configuration from backend
 */
async function loadConfig() {
  try {
    state.config = await Recorder.getConfig();
    updateUIFromConfig();
  } catch (error) {
    console.error('Error loading config:', error);
    throw error;
  }
}

/**
 * Update UI based on loaded config
 */
function updateUIFromConfig() {
  if (!state.config) return;

  // Video settings
  elements.monitorSelect.value = state.config.selected_monitor || 0;

  // Set active FPS button
  const currentFps = state.config.fps || 60;
  elements.fpsBtns.forEach((btn) => {
    btn.classList.remove('fps-btn--active');
    if (parseInt(btn.dataset.fps) === currentFps) {
      btn.classList.add('fps-btn--active');
    }
  });

  // Scale slider
  const scale = state.config.scale || 100;
  elements.scaleSlider.value = scale;
  updateScaleDisplay(scale);

  // Audio settings
  elements.micSelect.value = state.config.selected_mic || '';
  elements.audioOutputSelect.value = state.config.selected_audio_output || 'default';

  // System audio toggle
  const sysAudioEnabled = state.config.system_audio_enabled !== false;
  elements.sysAudioToggle.classList.toggle('toggle--active', sysAudioEnabled);
  elements.sysAudioToggle.classList.toggle('brutalist-toggle--active', sysAudioEnabled);
  elements.sysAudioToggle.setAttribute('aria-checked', sysAudioEnabled);

  // Mic volume
  const micVolume = state.config.mic_volume || 100;
  elements.micVolumeSlider.value = micVolume;
  updateMicVolumeDisplay(micVolume);

  state.isDirty = false;
}

/**
 * Load available monitors
 */
async function loadMonitors() {
  try {
    state.monitors = await Recorder.listMonitors();
    elements.monitorSelect.innerHTML = '';
    state.monitors.forEach((monitor) => {
      const option = document.createElement('option');
      option.value = monitor.index;
      option.textContent = `${monitor.name}${monitor.is_primary ? ' (Principal)' : ''}`;
      elements.monitorSelect.appendChild(option);
    });
  } catch (error) {
    console.error('Error loading monitors:', error);
  }
}

/**
 * Load available microphones
 */
async function loadMics() {
  try {
    state.mics = await Recorder.listMics();
    elements.micSelect.innerHTML = '<option value="">Nenhum</option>';
    state.mics.forEach((mic) => {
      const option = document.createElement('option');
      option.value = mic.id;
      option.textContent = `${mic.name}${mic.is_default ? ' (Padrão)' : ''}`;
      elements.micSelect.appendChild(option);
    });
  } catch (error) {
    console.error('Error loading mics:', error);
  }
}

/**
 * Load available audio outputs
 */
async function loadAudioOutputs() {
  try {
    state.audioOutputs = await Recorder.listAudioOutputs();
    elements.audioOutputSelect.innerHTML = '';
    state.audioOutputs.forEach((output) => {
      const option = document.createElement('option');
      option.value = output.id;
      option.textContent = output.name;
      elements.audioOutputSelect.appendChild(option);
    });
  } catch (error) {
    console.error('Error loading audio outputs:', error);
  }
}

/**
 * Load current output directory
 */
async function loadOutputDir() {
  try {
    const dir = await Recorder.getOutputDir();
    const textEl = elements.outputPath.querySelector('.brutalist-path__text');
    if (textEl) textEl.textContent = dir;
    elements.outputPath.title = dir;
  } catch (error) {
    console.error('Error loading output dir:', error);
  }
}

/**
 * Setup event listeners
 */
function setupEventListeners() {
  // Tab Switching
  elements.navBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const tabId = btn.dataset.tab;
      
      // Update buttons
      elements.navBtns.forEach(b => b.classList.remove('nav-btn--active'));
      btn.classList.add('nav-btn--active');
      
      // Update content
      elements.tabContents.forEach(content => {
        content.classList.add('hidden');
        if (content.id === `tab-${tabId}`) {
          content.classList.remove('hidden');
        }
      });
    });
  });

  // Monitor selection
  elements.monitorSelect.addEventListener('change', () => {
    state.isDirty = true;
  });

  // FPS buttons
  elements.fpsBtns.forEach((btn) => {
    btn.addEventListener('click', () => {
      elements.fpsBtns.forEach((b) => b.classList.remove('fps-btn--active'));
      btn.classList.add('fps-btn--active');
      state.isDirty = true;
    });
  });

  // Scale slider
  elements.scaleSlider.addEventListener('input', (e) => {
    updateScaleDisplay(e.target.value);
    state.isDirty = true;
  });

  // Mic selection
  elements.micSelect.addEventListener('change', () => {
    state.isDirty = true;
  });

  // Audio output selection
  elements.audioOutputSelect.addEventListener('change', () => {
    state.isDirty = true;
  });

  // System audio toggle
  elements.sysAudioToggle.addEventListener('click', () => {
    const isActive = !elements.sysAudioToggle.classList.contains('toggle--active');
    elements.sysAudioToggle.classList.toggle('toggle--active', isActive);
    elements.sysAudioToggle.classList.toggle('brutalist-toggle--active', isActive);
    elements.sysAudioToggle.setAttribute('aria-checked', isActive);
    state.isDirty = true;
  });

  // Mic volume slider
  elements.micVolumeSlider.addEventListener('input', (e) => {
    updateMicVolumeDisplay(e.target.value);
    state.isDirty = true;
  });

  // Output path button
  elements.outputPath.addEventListener('click', () => {
    selectOutputDirectory();
  });

  // Action buttons
  elements.saveBtn.addEventListener('click', saveConfig);
  elements.closeBtn.addEventListener('click', closeWindow);
  elements.resetBtn.addEventListener('click', resetToDefaults);
}

/**
 * Update scale display
 */
function updateScaleDisplay(value) {
  elements.scaleValue.textContent = `${value}%`;
  elements.scaleSlider.style.setProperty('--value', `${value}%`);
}

/**
 * Update mic volume display
 */
function updateMicVolumeDisplay(value) {
  elements.micVolumeValue.textContent = `${value}%`;
  const percent = (value / 150) * 100;
  elements.micVolumeSlider.style.setProperty('--value', `${percent}%`);
}

/**
 * Save configuration to backend
 */
async function saveConfig() {
  try {
    setStatusLoading('Salvando...');

    const activeFpsBtn = document.querySelector('.fps-btn--active');
    const fps = parseInt(activeFpsBtn.dataset.fps);
    const sysAudioEnabled = elements.sysAudioToggle.classList.contains('toggle--active');

    const config = {
      encoder: state.config.encoder,
      output_dir: state.config.output_dir,
      selected_monitor: parseInt(elements.monitorSelect.value),
      fps,
      scale: parseInt(elements.scaleSlider.value),
      selected_mic: elements.micSelect.value || null,
      selected_audio_output: elements.audioOutputSelect.value,
      system_audio_enabled: sysAudioEnabled,
      mic_enabled: !!elements.micSelect.value,
      sys_audio_enabled: sysAudioEnabled,
      mic_volume: parseInt(elements.micVolumeSlider.value),
    };

    await Recorder.updateConfig(config);
    state.isDirty = false;
    await closeWindow({ force: true });
  } catch (error) {
    console.error('Error saving config:', error);
    setStatusError('Erro ao salvar');
  }
}

/**
 * Reset to default settings
 */
async function resetToDefaults() {
  if (confirm('Restaurar padrões?')) {
    try {
      setStatusLoading('Resetando...');

      const defaultConfig = {
        encoder: state.config.encoder || 'libx264',
        output_dir: state.config.output_dir,
        selected_monitor: 0,
        fps: 60,
        scale: 100,
        mic_enabled: false,
        sys_audio_enabled: true,
        system_audio_enabled: true,
        selected_mic: null,
        selected_audio_output: null,
        mic_volume: 100,
      };

      await Recorder.updateConfig(defaultConfig);
      state.config = defaultConfig;
      updateUIFromConfig();
      setStatusSuccess('Resetado!');
    } catch (error) {
      console.error('Error resetting config:', error);
      setStatusError('Erro no reset');
    }
  }
}

/**
 * Select output directory
 */
async function selectOutputDirectory() {
  try {
    const { open } = window.__TAURI__.dialog;
    const selected = await open({
      directory: true,
      title: 'Selecionar pasta de destino',
    });

    if (selected) {
      state.isDirty = true;
      state.config.output_dir = selected;
      const textEl = elements.outputPath.querySelector('.brutalist-path__text');
      if (textEl) textEl.textContent = selected;
      elements.outputPath.title = selected;
      setStatusSuccess('Pasta alterada');
    }
  } catch (error) {
    console.error('Error selecting output directory:', error);
    setStatusError('Erro na pasta');
  }
}

/**
 * Close settings window
 */
async function closeWindow({ force = false } = {}) {
  if (!force && state.isDirty && !confirm('Sair sem salvar?')) {
    return;
  }

  try {
    await Recorder.hideSettings();
    return;
  } catch (error) {
    console.warn('Failed to hide settings window via backend command:', error);
  }

  try {
    const tauriWindow =
      window.__TAURI__?.webviewWindow?.getCurrentWebviewWindow?.() ||
      window.__TAURI__?.window?.getCurrentWindow?.();

    if (tauriWindow?.hide) {
      await tauriWindow.hide();
      return;
    }
  }
  catch (error) {
    console.warn('Failed to hide settings window via Tauri API:', error);
  }

  window.close();
}

/**
 * Set status as loading
 */
function setStatusLoading(message) {
  elements.statusIndicator.classList.add('status__dot--loading');
  elements.statusMessage.textContent = message;
}

/**
 * Set status as success
 */
function setStatusSuccess(message) {
  elements.statusIndicator.classList.remove('status__dot--loading', 'status__dot--error');
  elements.statusMessage.textContent = message;
}

/**
 * Set status as error
 */
function setStatusError(message) {
  elements.statusIndicator.classList.add('status__dot--error');
  elements.statusIndicator.classList.remove('status__dot--loading');
  elements.statusMessage.textContent = message;
}

// Initialize on load
document.addEventListener('DOMContentLoaded', init);
