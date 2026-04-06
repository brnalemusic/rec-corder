/**
 * Rec Corder - Main application logic
 * Handles UI state, user interactions, and timer updates.
 */

import { truncatePath } from './utils.js';
import * as recorder from './recorder.js';
import { dom, setupSecurity } from './dom.js';
import { loadAndMigratePrefs, savePrefs } from './prefs.js';
import { startTimer, stopTimer } from './timer.js';

let isRecording = false;
let isProcessing = false;
let micEnabled = false;
let selectedMonitor = 0;
let currentFps = 60;
let currentScale = 100;
let sysAudioEnabled = true;
let selectedAudioOutput = null;
let selectedMicId = null;

function resolveSelectedMonitor(monitors) {
  if (!Array.isArray(monitors) || monitors.length === 0) {
    return 0;
  }
  const savedMonitor = monitors.find((monitor) => monitor.index === selectedMonitor);
  if (savedMonitor) return savedMonitor.index;
  const primaryMonitor = monitors.find((monitor) => monitor.is_primary);
  return (primaryMonitor || monitors[0]).index;
}

async function syncPrefs() {
  try {
    const config = await recorder.getConfig();
    config.mic_enabled = micEnabled;
    config.sys_audio_enabled = sysAudioEnabled;
    config.system_audio_enabled = sysAudioEnabled;
    config.selected_monitor = selectedMonitor;
    config.fps = currentFps;
    config.scale = currentScale;
    config.selected_mic = selectedMicId;
    config.selected_audio_output = selectedAudioOutput;
    
    if (dom.outputPath && dom.outputPath.title) {
      config.output_dir = dom.outputPath.title;
    }
    await savePrefs(config);
  } catch (e) {
    console.warn('Failed to sync prefs:', e);
  }
}

document.addEventListener('DOMContentLoaded', init);
setupSecurity();

async function init() {
  const config = await loadAndMigratePrefs();
  if (config) {
    micEnabled = config.mic_enabled;
    sysAudioEnabled = config.sys_audio_enabled;
    selectedMonitor = config.selected_monitor;
    currentFps = config.fps;
    currentScale = config.scale;
    selectedAudioOutput = config.selected_audio_output;
    selectedMicId = config.selected_mic;
  }

  // Listen for config updates from other windows (e.g. Settings)
  try {
    const { listen } = window.__TAURI__.event;
    await listen('config-updated', (event) => {
      console.log('Config updated from backend:', event.payload);
      const updatedConfig = event.payload;
      
      micEnabled = updatedConfig.mic_enabled;
      sysAudioEnabled = updatedConfig.sys_audio_enabled || updatedConfig.system_audio_enabled;
      selectedMonitor = updatedConfig.selected_monitor;
      currentFps = updatedConfig.fps;
      currentScale = updatedConfig.scale;
      selectedAudioOutput = updatedConfig.selected_audio_output;
      selectedMicId = updatedConfig.selected_mic;

      if (dom.micToggle) {
        dom.micToggle.classList.toggle('toggle--active', micEnabled);
        dom.micToggle.setAttribute('aria-checked', micEnabled);
        dom.micToggle.closest('.setting')?.classList.toggle('setting--disabled', !micEnabled);
      }
      if (dom.sysAudioToggle) {
        dom.sysAudioToggle.classList.toggle('toggle--active', sysAudioEnabled);
        dom.sysAudioToggle.setAttribute('aria-checked', sysAudioEnabled);
        dom.sysAudioToggle.closest('.setting')?.classList.toggle('setting--disabled', !sysAudioEnabled);
      }
      if (dom.videoConfigBtn) {
        dom.videoConfigBtn.textContent = `MP4 · H.264 · ${currentFps}fps · ${currentScale}%`;
      }
      if (dom.outputPath && updatedConfig.output_dir) {
        dom.outputPath.textContent = truncatePath(updatedConfig.output_dir);
        dom.outputPath.title = updatedConfig.output_dir;
      }
      
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
    dom.micToggle.closest('.setting')?.classList.toggle('setting--disabled', !micEnabled);
  }
  
  if (dom.sysAudioToggle) {
    dom.sysAudioToggle.classList.toggle('toggle--active', sysAudioEnabled);
    dom.sysAudioToggle.setAttribute('aria-checked', sysAudioEnabled);
    dom.sysAudioToggle.closest('.setting')?.classList.toggle('setting--disabled', !sysAudioEnabled);
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

  if (dom.videoConfigBtn) {
    dom.videoConfigBtn.textContent = `MP4 · H.264 · ${currentFps}fps · ${currentScale}%`;
  }

  bindEvents();
  updateUI();

  // Check for updates after app fully loaded
  try {
    const update = await recorder.checkForUpdates();
    if (update) {
      const [updateVersion, changelog] = update;
      await recorder.showUpdater(updateVersion, changelog);
    }
  } catch (e) {
    console.error('Failed to check for updates:', e);
  }
}

function bindEvents() {
  dom.btnRecord?.addEventListener('click', handleRecordToggle);

  dom.monitorSelect?.addEventListener('change', async (event) => {
    selectedMonitor = parseInt(event.target.value, 10);
    await syncPrefs();
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
    await syncPrefs();
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
    isProcessing = false;
    updateUI();
    startTimer(dom.timerDisplay);

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
  if (isProcessing) return;
  isProcessing = true;
  if (dom.processingText) dom.processingText.textContent = 'Finalizando...';
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
    dom.micToggle.closest('.setting')?.classList.toggle('setting--disabled', !micEnabled);
  }

  if (micEnabled) {
    if (!selectedMicId) {
      await loadMics();
    }
  }
  await syncPrefs();
}

async function loadMics() {
  try {
    const mics = await recorder.listMics();
    if (mics.length > 0) {
      const exists = selectedMicId && mics.some(m => m.id === selectedMicId);
      if (!exists) {
        const defaultMic = mics.find((mic) => mic.is_default) ?? mics[0];
        selectedMicId = defaultMic.id;
      }
      return;
    }

    selectedMicId = null;
    micEnabled = false;
    dom.micToggle?.classList.remove('toggle--active');
  } catch (error) {
    selectedMicId = null;
  }
}

async function handleSysAudioToggle() {
  if (isRecording) return;

  sysAudioEnabled = !sysAudioEnabled;
  if (dom.sysAudioToggle) {
    dom.sysAudioToggle.classList.toggle('toggle--active', sysAudioEnabled);
    dom.sysAudioToggle.setAttribute('aria-checked', sysAudioEnabled);
    dom.sysAudioToggle.closest('.setting')?.classList.toggle('setting--disabled', !sysAudioEnabled);
  }
  
  if (dom.audioOutputContainer) {
    dom.audioOutputContainer.style.opacity = sysAudioEnabled ? '1' : '0.5';
  }
  
  if (dom.audioOutputSelect) {
    dom.audioOutputSelect.disabled = !sysAudioEnabled;
  }
  
  await syncPrefs();
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
      await syncPrefs();
    }
  } catch (error) {
    console.error('Folder pick error:', error);
  }
}
