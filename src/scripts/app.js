/**
 * Rec Corder - Main application logic
 * Handles UI state, user interactions, and timer updates.
 */

import { formatDuration, truncatePath } from './utils.js';
import * as recorder from './recorder.js';

let isRecording = false;
let isProcessing = false;
let timerInterval = null;
let elapsedSecs = 0;
let micEnabled = false;
let selectedMonitor = 0;
let currentFps = 60;
let currentScale = 100;
let sysAudioEnabled = true;
let selectedAudioOutput = null;
let selectedMicId = null;
const PREFS_KEY = 'rec_corder_prefs';

function resolveSelectedMonitor(monitors) {
  if (!Array.isArray(monitors) || monitors.length === 0) {
    return 0;
  }

  const savedMonitor = monitors.find((monitor) => monitor.index === selectedMonitor);
  if (savedMonitor) {
    return savedMonitor.index;
  }

  const primaryMonitor = monitors.find((monitor) => monitor.is_primary);
  return (primaryMonitor || monitors[0]).index;
}

async function savePrefs() {
  try {
    const config = await recorder.getConfig();
    config.mic_enabled = micEnabled;
    config.sys_audio_enabled = sysAudioEnabled;
    config.system_audio_enabled = sysAudioEnabled; // Sincroniza ambos os campos
    config.selected_monitor = selectedMonitor;
    config.fps = currentFps;
    config.scale = currentScale;
    config.selected_mic = selectedMicId;
    config.selected_audio_output = selectedAudioOutput;
    
    // Safety check for outputPath
    if (dom.outputPath && dom.outputPath.title) {
      config.output_dir = dom.outputPath.title;
    }
    
    await recorder.updateConfig(config);
  } catch (e) {
    console.warn('Failed to save config to backend:', e);
  }
}

async function loadAndMigratePrefs() {
  try {
    // 1. Get backend config
    let config = await recorder.getConfig();
    
    // 2. Check for legacy localStorage prefs
    const saved = localStorage.getItem(PREFS_KEY);
    if (saved) {
      console.log('Migrating legacy prefs to backend...');
      try {
        const prefs = JSON.parse(saved);
        
        // Update config with legacy values
        config.mic_enabled = !!prefs.micEnabled;
        config.sys_audio_enabled = prefs.sys_audio_enabled !== undefined ? !!prefs.sysAudioEnabled : true;
        config.selected_monitor = parseInt(prefs.selectedMonitor || 0, 10);
        config.fps = parseInt(prefs.currentFps || 60, 10);
        config.scale = parseInt(prefs.currentScale || 100, 10);
        config.selected_audio_output = prefs.selectedAudioOutput || null;
        if (prefs.outputDir) config.output_dir = prefs.outputDir;
        
        // Save to backend and clear local
        await recorder.updateConfig(config);
        localStorage.removeItem(PREFS_KEY);
      } catch (e) {
        console.warn('Error parsing legacy prefs:', e);
      }
    }
    
    // 3. Apply config to local state
    micEnabled = config.mic_enabled;
    sysAudioEnabled = config.sys_audio_enabled;
    selectedMonitor = config.selected_monitor;
    currentFps = config.fps;
    currentScale = config.scale;
    selectedAudioOutput = config.selected_audio_output;
    selectedMicId = config.selected_mic;

  } catch (e) {
    console.warn('Failed to load/migrate prefs from backend:', e);
  }
}

const $ = (id) => document.getElementById(id);

const dom = {
  timerDisplay: $('timerDisplay'),
  timerLabel: $('timerLabel'),
  btnRecord: $('btnRecord'),
  recIndicator: $('recIndicator'),
  monitorSelect: $('monitorSelect'),
  micToggle: $('micToggle'),
  micName: $('micName'),
  outputPath: $('outputPath'),
  statusDot: $('statusDot'),
  statusText: $('statusText'),
  crashBanner: $('crashBanner'),
  dismissCrash: $('dismissCrash'),
  videoConfigBtn: $('videoConfigBtn'),
  sysAudioToggle: $('sysAudioToggle'),
  audioOutputSelect: $('audioOutputSelect'),
  audioOutputContainer: $('audioOutputContainer'),
  processingIndicator: $('processingIndicator'),
  processingText: $('processingText'),
};

document.addEventListener('DOMContentLoaded', init);

// Security: Disable DevTools shortcuts and context menu
document.addEventListener('contextmenu', e => e.preventDefault());
document.addEventListener('keydown', e => {
  // F12
  if (e.key === 'F12') e.preventDefault();
  // Ctrl+Shift+I, J, C and Ctrl+U
  if (e.ctrlKey && (e.shiftKey && (e.key === 'I' || e.key === 'J' || e.key === 'C') || e.key === 'u')) {
    e.preventDefault();
  }
});

async function init() {
  await loadAndMigratePrefs();

  // Listen for config updates from other windows (e.g. Settings)
  try {
    const { listen } = window.__TAURI__.event;
    await listen('config-updated', (event) => {
      console.log('Config updated from backend:', event.payload);
      const config = event.payload;
      
      // Update local state
      micEnabled = config.mic_enabled;
      sysAudioEnabled = config.sys_audio_enabled || config.system_audio_enabled;
      selectedMonitor = config.selected_monitor;
      currentFps = config.fps;
      currentScale = config.scale;
      selectedAudioOutput = config.selected_audio_output;
      selectedMicId = config.selected_mic;

      // Update UI
      if (dom.micToggle) {
        dom.micToggle.classList.toggle('toggle--active', micEnabled);
        dom.micToggle.setAttribute('aria-checked', micEnabled);
      }
      if (dom.sysAudioToggle) {
        dom.sysAudioToggle.classList.toggle('toggle--active', sysAudioEnabled);
        dom.sysAudioToggle.setAttribute('aria-checked', sysAudioEnabled);
      }
      if (dom.videoConfigBtn) {
        dom.videoConfigBtn.textContent = `MP4 · H.264 · ${currentFps}fps · ${currentScale}%`;
      }
      if (dom.outputPath && config.output_dir) {
        dom.outputPath.textContent = truncatePath(config.output_dir);
        dom.outputPath.title = config.output_dir;
      }
      
      // Refresh audio outputs if needed
      loadAudioOutputs();
      if (micEnabled) loadMics();
    });
  } catch (e) {
    console.warn('Failed to setup config-updated listener:', e);
  }

  if (dom.monitorSelect) {
    try {
      const monitors = await recorder.listMonitors();
      selectedMonitor = resolveSelectedMonitor(monitors);
      dom.monitorSelect.innerHTML = monitors
        .map((monitor) => {
          const selected = monitor.index === selectedMonitor ? ' selected' : '';
          return `<option value="${monitor.index}"${selected}>${monitor.name}</option>`;
        })
        .join('');
    } catch (_) {
      dom.monitorSelect.innerHTML = '<option value="0">Monitor Principal</option>';
      selectedMonitor = 0;
    }
  }

  if (dom.outputPath) {
    try {
      const dir = await recorder.getOutputDir();
      dom.outputPath.textContent = truncatePath(dir);
      dom.outputPath.title = dir;
    } catch (_) {
      dom.outputPath.textContent = '...';
    }
  }

  await loadAudioOutputs();

  try {
    const crashed = await recorder.check_crash_recovery();
    if (crashed && dom.crashBanner) {
      dom.crashBanner.classList.add('crash-banner--visible');
    }
  } catch (_) {}

  try {
    const version = await recorder.getAppVersion();
    const versionElements = document.querySelectorAll('.header__version');
    versionElements.forEach(el => el.textContent = `v${version}`);
  } catch (e) {
    console.warn('Failed to set version:', e);
  }

  // Apply visual state from loaded prefs
  if (dom.micToggle) {
    dom.micToggle.classList.toggle('toggle--active', micEnabled);
    dom.micToggle.setAttribute('aria-checked', micEnabled);
  }
  
  if (dom.sysAudioToggle) {
    dom.sysAudioToggle.classList.toggle('toggle--active', sysAudioEnabled);
    dom.sysAudioToggle.setAttribute('aria-checked', sysAudioEnabled);
  }
  
  if (dom.audioOutputContainer) {
    dom.audioOutputContainer.style.opacity = sysAudioEnabled ? '1' : '0.5';
  }
  
  if (dom.audioOutputSelect) {
    dom.audioOutputSelect.disabled = !sysAudioEnabled;
  }

  if (micEnabled) {
    await loadMics();
  }

  // Update status bar text
  if (dom.videoConfigBtn) {
    dom.videoConfigBtn.textContent = `MP4 · H.264 · ${currentFps}fps · ${currentScale}%`;
  }

  bindEvents();
  updateUI();
}

function bindEvents() {
  dom.btnRecord?.addEventListener('click', handleRecordToggle);

  dom.monitorSelect?.addEventListener('change', async (event) => {
    selectedMonitor = parseInt(event.target.value, 10);
    await savePrefs();
  });

  dom.micToggle?.addEventListener('click', handleMicToggle);
  dom.micToggle?.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleMicToggle();
    }
  });

  dom.outputPath?.addEventListener('click', handleChangeOutputDir);

  dom.dismissCrash?.addEventListener('click', () => {
    dom.crashBanner?.classList.remove('crash-banner--visible');
  });

  dom.audioOutputSelect?.addEventListener('change', async (event) => {
    selectedAudioOutput = event.target.value || null;
    await savePrefs();
  });

  dom.sysAudioToggle?.addEventListener('click', handleSysAudioToggle);
  dom.sysAudioToggle?.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleSysAudioToggle();
    }
  });

  dom.videoConfigBtn?.addEventListener('click', openVideoConfigModal);
}

async function loadAudioOutputs() {
  try {
    if (!dom.audioOutputSelect) return;
    
    const outputs = await recorder.listAudioOutputs();

    if (outputs.length === 0) {
      dom.audioOutputSelect.innerHTML = '<option value="">Nenhuma saida encontrada</option>';
      selectedAudioOutput = null;
      return;
    }

    const activeId = selectedAudioOutput || (outputs.find((output) => output.is_default) || outputs[0]).id;
    selectedAudioOutput = activeId;

    dom.audioOutputSelect.innerHTML = outputs
      .map((output) => {
        const label = output.is_default
          ? `${output.name} (Padrao do Windows)`
          : output.name;
        const selected = output.id === activeId ? ' selected' : '';
        return `<option value="${output.id}"${selected}>${label}</option>`;
      })
      .join('');
  } catch (error) {
    console.error('Failed to load audio outputs:', error);
    if (dom.audioOutputSelect) {
      dom.audioOutputSelect.innerHTML = `<option value="">Erro: ${String(error)}</option>`;
    }
    selectedAudioOutput = null;
  }
}

function openVideoConfigModal() {
  if (isRecording) return;
  openSettingsWindow();
}

/**
 * Open the advanced settings window
 */
async function openSettingsWindow() {
  try {
    await recorder.showSettings();
  } catch (error) {
    console.error('Error opening settings window:', error);
    alert('Erro ao abrir configurações: ' + String(error));
  }
}

async function handleRecordToggle() {
  if (isRecording) {
    await handleStop();
  } else {
    await handleStart();
  }
}

async function handleStart() {
  try {
    if (dom.btnRecord) dom.btnRecord.disabled = true;
    if (dom.statusText) dom.statusText.textContent = 'Iniciando...';
    
    isProcessing = true;
    if (dom.processingText) dom.processingText.textContent = 'Iniciando...';
    updateUI();

    const result = await recorder.startRecording({
      monitorIndex: selectedMonitor,
      micName: micEnabled ? selectedMicId : null,
      systemAudioDevice: sysAudioEnabled ? selectedAudioOutput : null,
      fps: currentFps,
      scaleFactor: currentScale,
    });

    isRecording = true;
    elapsedSecs = 0;
    isProcessing = false;
    updateUI();
    startTimer();

    if (dom.statusText) dom.statusText.textContent = 'Gravando';
    if (dom.timerLabel) dom.timerLabel.textContent = truncatePath(result.file_path, 2);
  } catch (error) {
    isProcessing = false;
    updateUI();
    if (dom.statusText) dom.statusText.textContent = 'Erro: ' + String(error);
    console.error('Start error:', error);
    alert('Capture Error: ' + String(error));
  } finally {
    if (dom.btnRecord) dom.btnRecord.disabled = false;
  }
}

async function handleStop() {
  if (isProcessing) return; // Prevent double clicks
  isProcessing = true;
  if (dom.processingText) dom.processingText.textContent = 'Finalizando...';
  // Para o timer IMEDIATAMENTE, evitando que conte enquanto o arquivo é gravado no disco
  stopTimer();
  updateUI();

  try {
    if (dom.btnRecord) dom.btnRecord.disabled = true;
    if (dom.statusText) dom.statusText.textContent = 'Processando vídeo...';

    await recorder.stopRecording();

    isRecording = false;
    isProcessing = false;
    updateUI();

    if (dom.statusText) dom.statusText.textContent = 'Salvo com sucesso';
    if (dom.timerLabel) dom.timerLabel.textContent = 'Pronto para gravar';

    setTimeout(() => {
      if (!isRecording && !isProcessing && dom.statusText) {
        dom.statusText.textContent = 'Idle';
      }
    }, 3000);
  } catch (error) {
    isProcessing = false;
    updateUI();
    if (dom.statusText) dom.statusText.textContent = 'Erro: ' + String(error);
    console.error('Stop error:', error);
    alert('Stop Error: ' + String(error));
  } finally {
    if (dom.btnRecord) dom.btnRecord.disabled = false;
  }
}

function startTimer() {
  stopTimer();
  timerInterval = setInterval(() => {
    elapsedSecs += 1;
    if (dom.timerDisplay) dom.timerDisplay.textContent = formatDuration(elapsedSecs);
  }, 1000);
}

function stopTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
}

function updateUI() {
  if (isProcessing) {
    dom.btnRecord?.classList.add('hidden');
    dom.processingIndicator?.classList.remove('hidden');
    
    dom.timerDisplay?.classList.remove('timer__time--recording');
    dom.recIndicator?.classList.remove('rec-indicator--active');
    dom.statusDot?.classList.remove('status-bar__dot--idle');
    
    if (dom.monitorSelect) dom.monitorSelect.disabled = true;
    if (dom.micToggle) dom.micToggle.style.pointerEvents = 'none';
    if (dom.outputPath) dom.outputPath.style.pointerEvents = 'none';
    if (dom.videoConfigBtn) {
      dom.videoConfigBtn.style.pointerEvents = 'none';
      dom.videoConfigBtn.style.opacity = '0.5';
    }
    if (dom.sysAudioToggle) dom.sysAudioToggle.style.pointerEvents = 'none';
    
    if (dom.audioOutputSelect) {
      dom.audioOutputSelect.disabled = true;
    }
    return;
  }
  
  dom.btnRecord?.classList.remove('hidden');
  dom.processingIndicator?.classList.add('hidden');

  if (isRecording) {
    dom.btnRecord?.classList.add('btn--record--recording');
    dom.btnRecord?.setAttribute('aria-label', 'Parar gravacao');
  } else {
    dom.btnRecord?.classList.remove('btn--record--recording');
    dom.btnRecord?.setAttribute('aria-label', 'Iniciar gravacao');
    if (dom.timerDisplay) dom.timerDisplay.textContent = '00:00:00';
  }

  dom.timerDisplay?.classList.toggle('timer__time--recording', isRecording);
  dom.recIndicator?.classList.toggle('rec-indicator--active', isRecording);
  dom.statusDot?.classList.toggle('status-bar__dot--idle', !isRecording);

  if (dom.monitorSelect) dom.monitorSelect.disabled = isRecording;
  if (dom.micToggle) dom.micToggle.style.pointerEvents = isRecording ? 'none' : 'auto';
  if (dom.outputPath) dom.outputPath.style.pointerEvents = isRecording ? 'none' : 'auto';
  if (dom.videoConfigBtn) {
    dom.videoConfigBtn.style.pointerEvents = isRecording ? 'none' : 'auto';
    dom.videoConfigBtn.style.opacity = isRecording ? '0.5' : '1';
  }
  if (dom.sysAudioToggle) dom.sysAudioToggle.style.pointerEvents = isRecording ? 'none' : 'auto';
  
  if (dom.audioOutputSelect) {
    dom.audioOutputSelect.disabled = isRecording || !sysAudioEnabled;
  }
}

async function handleMicToggle() {
  if (isRecording) return;

  micEnabled = !micEnabled;
  if (dom.micToggle) {
    dom.micToggle.classList.toggle('toggle--active', micEnabled);
    dom.micToggle.setAttribute('aria-checked', micEnabled);
  }

  if (micEnabled) {
    await loadMics();
  } else {
    selectedMicId = null;
    if (dom.micName) dom.micName.textContent = 'Desativado';
  }
  await savePrefs();
}

async function loadMics() {
  try {
    const mics = await recorder.listMics();
    if (mics.length > 0) {
      const defaultMic = mics.find((mic) => mic.is_default) ?? mics[0];
      selectedMicId = defaultMic.id;
      if (dom.micName) {
        dom.micName.textContent = defaultMic.is_default
          ? `${defaultMic.name} (Padrao)`
          : defaultMic.name;
      }
      return;
    }

    selectedMicId = null;
    if (dom.micName) dom.micName.textContent = 'Nenhum mic encontrado';
    micEnabled = false;
    dom.micToggle?.classList.remove('toggle--active');
  } catch (error) {
    selectedMicId = null;
    if (dom.micName) dom.micName.textContent = 'Erro: ' + String(error);
  }
}

async function handleSysAudioToggle() {
  if (isRecording) return;

  sysAudioEnabled = !sysAudioEnabled;
  if (dom.sysAudioToggle) {
    dom.sysAudioToggle.classList.toggle('toggle--active', sysAudioEnabled);
    dom.sysAudioToggle.setAttribute('aria-checked', sysAudioEnabled);
  }
  
  if (dom.audioOutputContainer) {
    dom.audioOutputContainer.style.opacity = sysAudioEnabled ? '1' : '0.5';
  }
  
  if (dom.audioOutputSelect) {
    dom.audioOutputSelect.disabled = !sysAudioEnabled;
  }
  
  await savePrefs();
}

async function handleChangeOutputDir() {
  if (isRecording) return;

  try {
    const { open } = window.__TAURI__.dialog;
    const selected = await open({
      directory: true,
      title: 'Selecionar pasta de destino',
    });

    if (selected) {
      await recorder.setOutputDir(selected);
      if (dom.outputPath) {
        dom.outputPath.textContent = truncatePath(selected);
        dom.outputPath.title = selected;
      }
      await savePrefs();
    }
  } catch (error) {
    console.error('Folder pick error:', error);
  }
}
