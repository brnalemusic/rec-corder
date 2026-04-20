/**
 * Rec Corder — Eventos de DOM
 * Gerencia a vinculação de eventos de interação do usuário para botões, toggles e selects.
 */

import { dom } from './dom.js';
import { state, updateUI, syncPrefs, loadMics } from './ui-state.js';
import * as recorder from './recorder.js';
import { truncatePath } from './utils.js';
import { startTimer, stopTimer } from './timer.js';

/**
 * Vincula todos os eventos iniciais aos elementos de interface do usuário.
 */
export function bindEvents() {
  dom.btnRecord?.addEventListener('click', handleRecordToggle);

  dom.monitorSelect?.addEventListener('change', async (event) => {
    state.selectedMonitor = parseInt(event.target.value, 10);
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
    state.selectedAudioOutput = event.target.value || null;
    await syncPrefs();
  });

  dom.sysAudioToggle?.addEventListener('click', handleSysAudioToggle);
  dom.sysAudioToggle?.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleSysAudioToggle();
    }
  });

  dom.webcamToggle?.addEventListener('click', handleWebcamToggle);
  dom.webcamToggle?.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleWebcamToggle();
    }
  });

  dom.videoConfigBtn?.addEventListener('click', openVideoConfigModal);

  dom.versionDisplay?.addEventListener('click', handleVersionClick);

  // Configura janela de confirmação de saída
  try {
    const appWindow = recorder.getCurrentWindow();
    appWindow.onCloseRequested(async (event) => {
      event.preventDefault();
      if (state.isRecording) {
        if (dom.exitModalBackdrop) {
          dom.exitModalBackdrop.classList.remove('hidden');
        }
      } else {
        // Sai imediatamente caso não esteja gravando
        await recorder.forceExit();
      }
    });
    
    dom.confirmExitBtn?.addEventListener('click', async () => {
      try {
        await recorder.forceExit();
      } catch (error) {
        console.error('Falha ao encerrar o app:', error);
      }
    });
  } catch (error) {
    console.error('Falha ao configurar onCloseRequested:', error);
  }

  const hideExitModal = () => {
    dom.exitModalBackdrop?.classList.add('hidden');
  };

  dom.closeExitModal?.addEventListener('click', hideExitModal);
  dom.cancelExitBtn?.addEventListener('click', hideExitModal);
}

/**
 * Lida com o clique para abrir as notas de lançamento.
 * @returns {Promise<void>}
 */
async function handleVersionClick() {
  try {
    dom.versionDisplay?.blur();
    await recorder.showReleaseNotes();
  } catch (error) {
    console.error('Erro ao abrir as notas de lançamento:', error);
  }
}

/**
 * Abre o modal de configurações de vídeo se não estiver gravando.
 */
function openVideoConfigModal() {
  if (state.isRecording) return;
  openSettingsWindow();
}

/**
 * Abre a janela de configurações de aplicativo.
 * @returns {Promise<void>}
 */
async function openSettingsWindow() {
  try {
    await recorder.showSettings();
  } catch (error) {
    console.error('Erro ao abrir a janela de configurações:', error);
    alert('Erro ao abrir configurações: ' + String(error));
  }
}

/**
 * Alterna entre iniciar ou parar a gravação de acordo com o estado atual.
 * @returns {Promise<void>}
 */
async function handleRecordToggle() {
  if (state.isRecording) {
    await handleStop();
  } else {
    await handleStart();
  }
}

/**
 * Inicia o processo de gravação.
 * @returns {Promise<void>}
 */
async function handleStart() {
  try {
    if (dom.btnRecord) dom.btnRecord.disabled = true;
    if (dom.statusText) dom.statusText.textContent = 'Iniciando...';
    
    state.isProcessing = true;
    if (dom.processingText) dom.processingText.textContent = 'Iniciando...';
    updateUI();

    const result = await recorder.startRecording({
      monitorIndex: state.selectedMonitor,
      micName: state.micEnabled ? state.selectedMicId : null,
      systemAudioDevice: state.sysAudioEnabled ? state.selectedAudioOutput : null,
      fps: state.currentFps,
      scaleFactor: state.currentScale,
    });

    state.isRecording = true;
    state.isProcessing = false;
    updateUI();
    startTimer(dom.timerDisplay);

    if (dom.statusText) dom.statusText.textContent = 'Gravando';
    if (dom.timerLabel) dom.timerLabel.textContent = truncatePath(result.file_path, 2);
  } catch (error) {
    state.isProcessing = false;
    updateUI();
    if (dom.statusText) dom.statusText.textContent = 'Erro: ' + String(error);
    console.error('Erro de inicialização:', error);
    alert('Erro de Captura: ' + String(error));
  } finally {
    if (dom.btnRecord) dom.btnRecord.disabled = false;
  }
}

/**
 * Para o processo de gravação atual.
 * @returns {Promise<void>}
 */
async function handleStop() {
  if (state.isProcessing) return;
  state.isProcessing = true;
  if (dom.processingText) dom.processingText.textContent = 'Finalizando...';
  stopTimer();
  updateUI();

  try {
    if (dom.btnRecord) dom.btnRecord.disabled = true;
    if (dom.statusText) dom.statusText.textContent = 'Processando vídeo...';

    await recorder.stopRecording();

    state.isRecording = false;
    state.isProcessing = false;
    updateUI();

    if (dom.statusText) dom.statusText.textContent = 'Salvo com sucesso';
    if (dom.timerLabel) dom.timerLabel.textContent = 'Pronto para gravar';

    setTimeout(() => {
      if (!state.isRecording && !state.isProcessing && dom.statusText) {
        dom.statusText.textContent = 'Idle';
      }
    }, 3000);
  } catch (error) {
    state.isProcessing = false;
    updateUI();
    if (dom.statusText) dom.statusText.textContent = 'Erro: ' + String(error);
    console.error('Erro de parada:', error);
    alert('Erro de Parada: ' + String(error));
  } finally {
    if (dom.btnRecord) dom.btnRecord.disabled = false;
  }
}

/**
 * Lida com o clique no toggle do microfone.
 * @returns {Promise<void>}
 */
async function handleMicToggle() {
  if (state.isRecording) return;

  state.micEnabled = !state.micEnabled;
  if (dom.micToggle) {
    dom.micToggle.classList.toggle('toggle--active', state.micEnabled);
    dom.micToggle.setAttribute('aria-checked', state.micEnabled);
    dom.micToggle.closest('.setting')?.classList.toggle('setting--disabled', !state.micEnabled);
  }

  if (state.micEnabled) {
    if (!state.selectedMicId) {
      await loadMics();
    }
  }
  await syncPrefs();
}

/**
 * Lida com o clique no toggle da webcam.
 * @returns {Promise<void>}
 */
async function handleWebcamToggle() {
  if (state.isRecording || !state.webcamAvailable) return;
  state.webcamEnabled = !state.webcamEnabled;
  if (dom.webcamToggle) {
    dom.webcamToggle.classList.toggle('toggle--active', state.webcamEnabled);
    dom.webcamToggle.setAttribute('aria-checked', state.webcamEnabled);
    dom.webcamToggle.closest('.setting')?.classList.toggle('setting--disabled', !state.webcamEnabled);
  }
  await syncPrefs();
}

/**
 * Lida com o clique no toggle de áudio do sistema.
 * @returns {Promise<void>}
 */
async function handleSysAudioToggle() {
  if (state.isRecording) return;

  state.sysAudioEnabled = !state.sysAudioEnabled;
  if (dom.sysAudioToggle) {
    dom.sysAudioToggle.classList.toggle('toggle--active', state.sysAudioEnabled);
    dom.sysAudioToggle.setAttribute('aria-checked', state.sysAudioEnabled);
    dom.sysAudioToggle.closest('.setting')?.classList.toggle('setting--disabled', !state.sysAudioEnabled);
  }
  
  if (dom.audioOutputContainer) {
    dom.audioOutputContainer.style.opacity = state.sysAudioEnabled ? '1' : '0.5';
  }
  
  if (dom.audioOutputSelect) {
    dom.audioOutputSelect.disabled = !state.sysAudioEnabled;
  }
  
  await syncPrefs();
}

/**
 * Lida com a mudança de diretório de destino.
 * @returns {Promise<void>}
 */
async function handleChangeOutputDir() {
  if (state.isRecording) return;

  try {
    const selected = await recorder.openDialog({
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
    console.error('Erro ao escolher a pasta:', error);
  }
}