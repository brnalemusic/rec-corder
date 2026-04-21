/**
 * Rec Corder — Inicialização e Bootstrap
 * Módulo principal responsável por iniciar a aplicação, interligar estados e eventos, e realizar configurações iniciais.
 */

import { truncatePath } from './utils.js';
import * as recorder from './recorder.js';
import { dom, setupSecurity } from './dom.js';
import { loadAndMigratePrefs } from './prefs.js';
import { state, updateUI, loadAudioOutputs, loadMics, resolveSelectedMonitor } from './ui-state.js';
import { bindEvents } from './events.js';

/**
 * Função de inicialização disparada quando o DOM estiver completamente carregado.
 * Prepara o estado inicial, atualiza a interface e escuta por modificações.
 * @returns {Promise<void>}
 */
async function init() {
  const config = await loadAndMigratePrefs();
  if (config) {
    state.micEnabled = config.mic_enabled;
    state.sysAudioEnabled = config.sys_audio_enabled;
    state.selectedMonitor = config.selected_monitor;
    state.currentFps = config.fps;
    state.currentScale = config.scale;
    state.selectedAudioOutput = config.selected_audio_output;
    state.selectedMicId = config.selected_mic;
    state.webcamEnabled = config.webcam_enabled === true;
  }

  // Escuta as atualizações de configurações de outras janelas (ex. Configurações)
  try {
    await recorder.listen('config-updated', (event) => {
      console.log('Configuração atualizada pelo backend:', event.payload);
      const updatedConfig = event.payload;
      
      state.micEnabled = updatedConfig.mic_enabled;
      state.sysAudioEnabled = updatedConfig.sys_audio_enabled || updatedConfig.system_audio_enabled;
      state.selectedMonitor = updatedConfig.selected_monitor;
      state.currentFps = updatedConfig.fps;
      state.currentScale = updatedConfig.scale;
      state.selectedAudioOutput = updatedConfig.selected_audio_output;
      state.selectedMicId = updatedConfig.selected_mic;

      if (dom.micToggle) {
        dom.micToggle.classList.toggle('toggle--active', state.micEnabled);
        dom.micToggle.setAttribute('aria-checked', state.micEnabled);
        dom.micToggle.closest('.setting')?.classList.toggle('setting--disabled', !state.micEnabled);
      }
      if (dom.sysAudioToggle) {
        dom.sysAudioToggle.classList.toggle('toggle--active', state.sysAudioEnabled);
        dom.sysAudioToggle.setAttribute('aria-checked', state.sysAudioEnabled);
        dom.sysAudioToggle.closest('.setting')?.classList.toggle('setting--disabled', !state.sysAudioEnabled);
      }

      // Sincroniza estado da webcam
      state.webcamEnabled = updatedConfig.webcam_enabled === true;
      if (dom.webcamToggle) {
        dom.webcamToggle.classList.toggle('toggle--active', state.webcamEnabled);
        dom.webcamToggle.setAttribute('aria-checked', state.webcamEnabled);
        dom.webcamToggle.closest('.setting')?.classList.toggle('setting--disabled', !state.webcamEnabled);
      }

      if (dom.videoConfigBtn) {
        dom.videoConfigBtn.textContent = `MP4 · H.264 · ${state.currentFps}fps · ${state.currentScale}%`;
      }
      if (dom.outputPath && updatedConfig.output_dir) {
        dom.outputPath.textContent = truncatePath(updatedConfig.output_dir);
        dom.outputPath.title = updatedConfig.output_dir;
      }
      
      loadAudioOutputs();
      if (state.micEnabled) loadMics();
    });
  } catch (e) {
    console.warn('Falha ao configurar listener config-updated:', e);
  }

  if (dom.monitorSelect) {
    try {
      const monitors = await recorder.listMonitors();
      state.selectedMonitor = resolveSelectedMonitor(monitors);
      dom.monitorSelect.innerHTML = monitors
        .map((monitor) => {
          const selected = monitor.index === state.selectedMonitor ? ' selected' : '';
          return `<option value="${monitor.index}"${selected}>${monitor.name}</option>`;
        })
        .join('');
    } catch (_) {
      dom.monitorSelect.innerHTML = '<option value="0">Monitor Principal</option>';
      state.selectedMonitor = 0;
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
    console.warn('Falha ao obter versão:', e);
  }

  // Aplica o estado visual inicial nas preferências carregadas
  if (dom.micToggle) {
    dom.micToggle.classList.toggle('toggle--active', state.micEnabled);
    dom.micToggle.setAttribute('aria-checked', state.micEnabled);
    dom.micToggle.closest('.setting')?.classList.toggle('setting--disabled', !state.micEnabled);
  }
  
  if (dom.sysAudioToggle) {
    dom.sysAudioToggle.classList.toggle('toggle--active', state.sysAudioEnabled);
    dom.sysAudioToggle.setAttribute('aria-checked', state.sysAudioEnabled);
    dom.sysAudioToggle.closest('.setting')?.classList.toggle('setting--disabled', !state.sysAudioEnabled);
  }

  // Detecta câmeras disponíveis e aplica o estado visual da webcam
  try {
    const cameras = await recorder.listCameras();
    state.webcamAvailable = cameras.length > 0;
    if (!state.webcamAvailable) state.webcamEnabled = false;
  } catch (_) {
    state.webcamAvailable = false;
    state.webcamEnabled = false;
  }

  if (dom.webcamToggle) {
    dom.webcamToggle.classList.toggle('toggle--active', state.webcamEnabled);
    dom.webcamToggle.setAttribute('aria-checked', state.webcamEnabled);
    dom.webcamToggle.closest('.setting')?.classList.toggle('setting--disabled', !state.webcamEnabled);
  }
  if (!state.webcamAvailable && dom.webcamSetting) {
    if (dom.webcamToggle) {
      dom.webcamToggle.style.pointerEvents = 'none';
      dom.webcamToggle.style.opacity = '0.35';
    }
    dom.webcamSetting.classList.add('setting--disabled');
    dom.webcamSetting.title = 'Nenhuma câmera encontrada';
  }
  
  if (dom.audioOutputContainer) {
    dom.audioOutputContainer.style.opacity = state.sysAudioEnabled ? '1' : '0.5';
  }
  
  if (dom.audioOutputSelect) {
    dom.audioOutputSelect.disabled = !state.sysAudioEnabled;
  }

  if (state.micEnabled) {
    await loadMics();
  }

  if (dom.videoConfigBtn) {
    dom.videoConfigBtn.textContent = `MP4 · H.264 · ${state.currentFps}fps · ${state.currentScale}%`;
  }

  bindEvents();
  updateUI();

  // Checa por atualizações após aplicativo carregar por completo
  try {
    const update = await recorder.checkForUpdates();
    if (update) {
      const [updateVersion, changelog] = update;
      await recorder.showUpdater(updateVersion, changelog);
    }
  } catch (e) {
    console.error('Falha ao checar atualizações:', e);
  }

  // Checa por dependências no Linux
  try {
    const missing = await recorder.checkLinuxDeps();
    if (missing && missing.length > 0) {
      console.warn('Dependências ausentes no Linux:', missing);
      if (dom.linuxDepsModalBackdrop) {
        dom.linuxDepsModalBackdrop.classList.remove('hidden');
      }
    }
  } catch (e) {
    // Ignora erros (provavelmente não está no Linux)
  }
}

document.addEventListener('DOMContentLoaded', init);
setupSecurity();