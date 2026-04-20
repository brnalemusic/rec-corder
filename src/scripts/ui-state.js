/**
 * Rec Corder — Estado e UI
 * Gerencia o estado compartilhado da aplicação e atualizações de interface.
 */

import { dom } from './dom.js';
import * as recorder from './recorder.js';
import { savePrefs } from './prefs.js';
import { truncatePath } from './utils.js';

/**
 * @typedef {Object} AppState
 * @property {boolean} isRecording - Se está atualmente gravando.
 * @property {boolean} isProcessing - Se está processando o início/parada da gravação.
 * @property {boolean} micEnabled - Se o microfone está habilitado.
 * @property {number} selectedMonitor - O índice do monitor selecionado.
 * @property {number} currentFps - Os frames por segundo atuais.
 * @property {number} currentScale - A escala da gravação atual.
 * @property {boolean} sysAudioEnabled - Se o áudio do sistema está habilitado.
 * @property {string|null} selectedAudioOutput - ID da saída de áudio selecionada.
 * @property {string|null} selectedMicId - ID do microfone selecionado.
 * @property {boolean} webcamEnabled - Se a webcam está habilitada.
 * @property {boolean} webcamAvailable - Se existem câmeras disponíveis.
 */

/** @type {AppState} */
export const state = {
  isRecording: false,
  isProcessing: false,
  micEnabled: false,
  selectedMonitor: 0,
  currentFps: 60,
  currentScale: 100,
  sysAudioEnabled: true,
  selectedAudioOutput: null,
  selectedMicId: null,
  webcamEnabled: false,
  webcamAvailable: false
};

/**
 * Sincroniza o estado em memória com as preferências salvas no backend.
 * @returns {Promise<void>}
 */
export async function syncPrefs() {
  try {
    const config = await recorder.getConfig();
    config.mic_enabled = state.micEnabled;
    config.sys_audio_enabled = state.sysAudioEnabled;
    config.system_audio_enabled = state.sysAudioEnabled;
    config.selected_monitor = state.selectedMonitor;
    config.fps = state.currentFps;
    config.scale = state.currentScale;
    config.selected_mic = state.selectedMicId;
    config.selected_audio_output = state.selectedAudioOutput;
    config.webcam_enabled = state.webcamEnabled;
    
    if (dom.outputPath && dom.outputPath.title) {
      config.output_dir = dom.outputPath.title;
    }
    await savePrefs(config);
  } catch (e) {
    console.warn('Falha ao sincronizar preferências:', e);
  }
}

/**
 * Atualiza todos os elementos visuais da interface dependendo do estado atual.
 */
export function updateUI() {
  if (state.isProcessing) {
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
    if (dom.webcamToggle) dom.webcamToggle.style.pointerEvents = 'none';
    
    if (dom.audioOutputSelect) {
      dom.audioOutputSelect.disabled = true;
    }
    return;
  }
  
  dom.btnRecord?.classList.remove('hidden');
  dom.processingIndicator?.classList.add('hidden');

  if (state.isRecording) {
    dom.btnRecord?.classList.add('btn--record--recording');
    dom.btnRecord?.setAttribute('aria-label', 'Parar gravação');
  } else {
    dom.btnRecord?.classList.remove('btn--record--recording');
    dom.btnRecord?.setAttribute('aria-label', 'Iniciar gravação');
    if (dom.timerDisplay) dom.timerDisplay.textContent = '00:00:00';
  }

  dom.timerDisplay?.classList.toggle('timer__time--recording', state.isRecording);
  dom.recIndicator?.classList.toggle('rec-indicator--active', state.isRecording);
  dom.statusDot?.classList.toggle('status-bar__dot--idle', !state.isRecording);

  if (dom.monitorSelect) dom.monitorSelect.disabled = state.isRecording;
  if (dom.micToggle) dom.micToggle.style.pointerEvents = state.isRecording ? 'none' : 'auto';
  if (dom.outputPath) dom.outputPath.style.pointerEvents = state.isRecording ? 'none' : 'auto';
  if (dom.videoConfigBtn) {
    dom.videoConfigBtn.style.pointerEvents = state.isRecording ? 'none' : 'auto';
    dom.videoConfigBtn.style.opacity = state.isRecording ? '0.5' : '1';
  }
  if (dom.sysAudioToggle) dom.sysAudioToggle.style.pointerEvents = state.isRecording ? 'none' : 'auto';
  if (dom.webcamToggle) dom.webcamToggle.style.pointerEvents = state.isRecording ? 'none' : 'auto';
  
  if (dom.audioOutputSelect) {
    dom.audioOutputSelect.disabled = state.isRecording || !state.sysAudioEnabled;
  }
}

/**
 * Carrega a lista de dispositivos de saída de áudio no select.
 * @returns {Promise<void>}
 */
export async function loadAudioOutputs() {
  try {
    if (!dom.audioOutputSelect) return;
    
    const outputs = await recorder.listAudioOutputs();
    if (outputs.length === 0) {
      dom.audioOutputSelect.innerHTML = '<option value="">Nenhuma saída encontrada</option>';
      state.selectedAudioOutput = null;
      return;
    }

    const activeId = state.selectedAudioOutput || (outputs.find((output) => output.is_default) || outputs[0]).id;
    state.selectedAudioOutput = activeId;

    dom.audioOutputSelect.innerHTML = outputs
      .map((output) => {
        const label = output.is_default
          ? `${output.name} (Padrão do Windows)`
          : output.name;
        const selected = output.id === activeId ? ' selected' : '';
        return `<option value="${output.id}"${selected}>${label}</option>`;
      })
      .join('');
  } catch (error) {
    console.error('Falha ao carregar as saídas de áudio:', error);
    if (dom.audioOutputSelect) {
      dom.audioOutputSelect.innerHTML = `<option value="">Erro: ${String(error)}</option>`;
    }
    state.selectedAudioOutput = null;
  }
}

/**
 * Carrega a lista de microfones.
 * @returns {Promise<void>}
 */
export async function loadMics() {
  try {
    const mics = await recorder.listMics();
    if (mics.length > 0) {
      const exists = state.selectedMicId && mics.some(m => m.id === state.selectedMicId);
      if (!exists) {
        const defaultMic = mics.find((mic) => mic.is_default) ?? mics[0];
        state.selectedMicId = defaultMic.id;
      }
      return;
    }

    state.selectedMicId = null;
    state.micEnabled = false;
    dom.micToggle?.classList.remove('toggle--active');
  } catch (error) {
    state.selectedMicId = null;
  }
}

/**
 * Resolve o índice do monitor a ser selecionado inicialmente.
 * @param {Array<Object>} monitors - Lista de monitores.
 * @returns {number} O índice do monitor selecionado.
 */
export function resolveSelectedMonitor(monitors) {
  if (!Array.isArray(monitors) || monitors.length === 0) {
    return 0;
  }
  const savedMonitor = monitors.find((monitor) => monitor.index === state.selectedMonitor);
  if (savedMonitor) return savedMonitor.index;
  const primaryMonitor = monitors.find((monitor) => monitor.is_primary);
  return (primaryMonitor || monitors[0]).index;
}