/**
 * Rec Corder — Janela de Configurações
 * Gerencia o painel de configurações avançadas em uma janela separada.
 */

import * as Recorder from './recorder.js';

/**
 * @typedef {Object} SettingsState
 * @property {Object|null} config - Objeto de configuração atual.
 * @property {Array} monitors - Monitores detectados.
 * @property {Array} mics - Microfones detectados.
 * @property {Array} audioOutputs - Saídas de áudio detectadas.
 * @property {Array} cameras - Câmeras detectadas.
 * @property {boolean} isDirty - Indica se houve alterações não salvas.
 */

/** @type {SettingsState} */
const state = {
  config: null,
  monitors: [],
  mics: [],
  audioOutputs: [],
  cameras: [],
  isDirty: false,
};

// Elementos DOM
const elements = {
  // Abas
  navBtns: document.querySelectorAll('.nav-btn'),
  tabContents: document.querySelectorAll('.tab-content'),

  // Vídeo
  monitorSelect: document.getElementById('monitorSelect'),
  fpsBtns: document.querySelectorAll('.fps-btn'),
  scaleSlider: document.getElementById('scaleSlider'),
  scaleValue: document.getElementById('scaleValue'),

  // Webcam
  webcamToggle: document.getElementById('webcamToggleSettings'),
  cameraSelect: document.getElementById('cameraSelect'),
  positionBtns: document.querySelectorAll('.pos-btn'),
  webcamSizeSlider: document.getElementById('webcamSizeSlider'),
  webcamSizeValue: document.getElementById('webcamSizeValue'),
  webcamDetails: document.querySelectorAll('.webcam-detail'),

  // Áudio
  micSelect: document.getElementById('micSelect'),
  audioOutputSelect: document.getElementById('audioOutputSelect'),
  sysAudioToggle: document.getElementById('sysAudioToggle'),
  micVolumeSlider: document.getElementById('micVolumeSlider'),
  micVolumeValue: document.getElementById('micVolumeValue'),

  // Saída
  outputPath: document.getElementById('outputPath'),

  // Ações
  saveBtn: document.getElementById('saveBtn'),
  closeBtn: document.getElementById('closeBtn'),
  resetBtn: document.getElementById('resetBtn'),

  // Status
  statusIndicator: document.getElementById('statusIndicator'),
  statusMessage: document.getElementById('statusMessage'),
};

/**
 * Inicializa a janela de configurações.
 * @returns {Promise<void>}
 */
async function init() {
  try {
    setStatusLoading('Carregando...');

    // Carrega dados e configurações
    await Promise.all([
      loadConfig(),
      loadMonitors(),
      loadMics(),
      loadAudioOutputs(),
      loadOutputDir(),
      loadCameras(),
    ]);

    // Configura os ouvintes de evento
    setupEventListeners();

    // Escuta as atualizações de outras janelas
    try {
      await Recorder.listen('config-updated', (event) => {
        console.log('Configurações: configuração atualizada pelo backend:', event.payload);
        state.config = event.payload;
        updateUIFromConfig();
      });
    } catch (e) {
      console.warn('Falha ao configurar o listener config-updated nas configurações:', e);
    }

    setStatusSuccess('Pronto');
  } catch (error) {
    console.error('Erro ao inicializar as configurações:', error);
    setStatusError('Erro');
  }
}

/**
 * Carrega a configuração atual do backend.
 * @returns {Promise<void>}
 */
async function loadConfig() {
  try {
    state.config = await Recorder.getConfig();
    updateUIFromConfig();
  } catch (error) {
    console.error('Erro ao carregar a configuração:', error);
    throw error;
  }
}

/**
 * Atualiza a interface com base na configuração carregada.
 */
function updateUIFromConfig() {
  if (!state.config) return;

  // Configurações de vídeo
  elements.monitorSelect.value = state.config.selected_monitor || 0;

  // Define o botão ativo de FPS
  const currentFps = state.config.fps || 60;
  elements.fpsBtns.forEach((btn) => {
    btn.classList.remove('fps-btn--active');
    if (parseInt(btn.dataset.fps) === currentFps) {
      btn.classList.add('fps-btn--active');
    }
  });

  // Slider de escala
  const scale = state.config.scale || 100;
  elements.scaleSlider.value = scale;
  updateScaleDisplay(scale);

  // Configurações de áudio
  elements.micSelect.value = state.config.selected_mic || '';
  elements.audioOutputSelect.value = state.config.selected_audio_output || 'default';

  // Toggle do áudio de sistema
  const sysAudioEnabled = state.config.system_audio_enabled !== false;
  elements.sysAudioToggle.classList.toggle('toggle--active', sysAudioEnabled);
  elements.sysAudioToggle.classList.toggle('brutalist-toggle--active', sysAudioEnabled);
  elements.sysAudioToggle.setAttribute('aria-checked', sysAudioEnabled);

  // Volume do microfone
  const micVolume = state.config.mic_volume || 100;
  elements.micVolumeSlider.value = micVolume;
  updateMicVolumeDisplay(micVolume);

  // Toggle da webcam
  const webcamEnabled = state.config.webcam_enabled === true;
  elements.webcamToggle.classList.toggle('toggle--active', webcamEnabled);
  elements.webcamToggle.classList.toggle('brutalist-toggle--active', webcamEnabled);
  elements.webcamToggle.setAttribute('aria-checked', webcamEnabled);

  // Exibe/oculta cards de detalhes da webcam
  elements.webcamDetails.forEach(card => {
    card.classList.toggle('webcam-detail--hidden', !webcamEnabled);
  });

  // Câmera da webcam
  if (state.config.webcam_device) {
    elements.cameraSelect.value = state.config.webcam_device;
  }

  // Posição da webcam
  const pos = state.config.webcam_position || 'bottom-right';
  elements.positionBtns.forEach(btn => {
    btn.classList.toggle('pos-btn--active', btn.dataset.pos === pos);
  });

  // Tamanho da webcam
  const webcamSize = state.config.webcam_size || 100;
  elements.webcamSizeSlider.value = webcamSize;
  updateWebcamSizeDisplay(webcamSize);

  state.isDirty = false;
}

/**
 * Carrega monitores disponíveis.
 * @returns {Promise<void>}
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
    console.error('Erro ao carregar monitores:', error);
  }
}

/**
 * Carrega microfones disponíveis.
 * @returns {Promise<void>}
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
    console.error('Erro ao carregar microfones:', error);
  }
}

/**
 * Carrega saídas de áudio disponíveis.
 * @returns {Promise<void>}
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
    console.error('Erro ao carregar saídas de áudio:', error);
  }
}

/**
 * Carrega câmeras disponíveis via FFmpeg DirectShow.
 * @returns {Promise<void>}
 */
async function loadCameras() {
  try {
    state.cameras = await Recorder.listCameras();
    elements.cameraSelect.innerHTML = '';
    if (state.cameras.length === 0) {
      elements.cameraSelect.innerHTML = '<option value="">Nenhuma câmera encontrada</option>';
      elements.webcamToggle.style.pointerEvents = 'none';
      elements.webcamToggle.style.opacity = '0.4';
      return;
    }
    state.cameras.forEach(cam => {
      const option = document.createElement('option');
      option.value = cam.id;
      option.textContent = cam.name;
      elements.cameraSelect.appendChild(option);
    });
    // Auto-seleciona câmera guardada na configuração
    if (state.config?.webcam_device) {
      elements.cameraSelect.value = state.config.webcam_device;
    }
  } catch (error) {
    console.error('Erro ao carregar câmeras:', error);
  }
}

/**
 * Carrega o diretório de saída atual.
 * @returns {Promise<void>}
 */
async function loadOutputDir() {
  try {
    const dir = await Recorder.getOutputDir();
    const textEl = elements.outputPath.querySelector('.brutalist-path__text');
    if (textEl) textEl.textContent = dir;
    elements.outputPath.title = dir;
  } catch (error) {
    console.error('Erro ao carregar o diretório de saída:', error);
  }
}

/**
 * Configura os ouvintes de evento da janela.
 */
function setupEventListeners() {
  // Alternar Abas
  elements.navBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      const tabId = btn.dataset.tab;
      
      // Atualiza botões
      elements.navBtns.forEach(b => b.classList.remove('nav-btn--active'));
      btn.classList.add('nav-btn--active');
      
      // Atualiza conteúdo
      elements.tabContents.forEach(content => {
        content.classList.add('hidden');
        if (content.id === `tab-${tabId}`) {
          content.classList.remove('hidden');
        }
      });
    });
  });

  // Seleção de Monitor
  elements.monitorSelect.addEventListener('change', () => {
    state.isDirty = true;
  });

  // Botões de FPS
  elements.fpsBtns.forEach((btn) => {
    btn.addEventListener('click', () => {
      elements.fpsBtns.forEach((b) => b.classList.remove('fps-btn--active'));
      btn.classList.add('fps-btn--active');
      state.isDirty = true;
    });
  });

  // Slider de escala
  elements.scaleSlider.addEventListener('input', (e) => {
    updateScaleDisplay(e.target.value);
    state.isDirty = true;
  });

  // Seleção de microfone
  elements.micSelect.addEventListener('change', () => {
    state.isDirty = true;
  });

  // Seleção de saída de áudio
  elements.audioOutputSelect.addEventListener('change', () => {
    state.isDirty = true;
  });

  // Toggle do áudio de sistema
  elements.sysAudioToggle.addEventListener('click', () => {
    const isActive = !elements.sysAudioToggle.classList.contains('toggle--active');
    elements.sysAudioToggle.classList.toggle('toggle--active', isActive);
    elements.sysAudioToggle.classList.toggle('brutalist-toggle--active', isActive);
    elements.sysAudioToggle.setAttribute('aria-checked', isActive);
    state.isDirty = true;
  });

  // Slider de volume do microfone
  elements.micVolumeSlider.addEventListener('input', (e) => {
    updateMicVolumeDisplay(e.target.value);
    state.isDirty = true;
  });

  // Toggle da webcam
  elements.webcamToggle.addEventListener('click', () => {
    if (state.cameras.length === 0) return;
    const isActive = !elements.webcamToggle.classList.contains('toggle--active');
    elements.webcamToggle.classList.toggle('toggle--active', isActive);
    elements.webcamToggle.classList.toggle('brutalist-toggle--active', isActive);
    elements.webcamToggle.setAttribute('aria-checked', isActive);
    elements.webcamDetails.forEach(card => {
      card.classList.toggle('webcam-detail--hidden', !isActive);
    });
    state.isDirty = true;
  });

  // Seleção de câmera
  elements.cameraSelect.addEventListener('change', () => {
    state.isDirty = true;
  });

  // Botões de posição
  elements.positionBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      elements.positionBtns.forEach(b => b.classList.remove('pos-btn--active'));
      btn.classList.add('pos-btn--active');
      state.isDirty = true;
    });
  });

  // Slider de tamanho da webcam
  elements.webcamSizeSlider.addEventListener('input', (e) => {
    updateWebcamSizeDisplay(e.target.value);
    state.isDirty = true;
  });

  // Botão de diretório de saída
  elements.outputPath.addEventListener('click', () => {
    selectOutputDirectory();
  });

  // Botões de Ação
  elements.saveBtn.addEventListener('click', saveConfig);
  elements.closeBtn.addEventListener('click', closeWindow);
  elements.resetBtn.addEventListener('click', resetToDefaults);
}

/**
 * Atualiza o visualizador da escala.
 * @param {number|string} value - O valor atual da escala.
 */
function updateScaleDisplay(value) {
  elements.scaleValue.textContent = `${value}%`;
  const percent = ((value - 50) / 50) * 100;
  elements.scaleSlider.style.setProperty('--value', `${percent}%`);
}

/**
 * Atualiza o visualizador do volume do microfone.
 * @param {number|string} value - O valor atual do volume do microfone.
 */
function updateMicVolumeDisplay(value) {
  elements.micVolumeValue.textContent = `${value}%`;
  const percent = (value / 150) * 100;
  elements.micVolumeSlider.style.setProperty('--value', `${percent}%`);
}

/**
 * Atualiza o visualizador do tamanho da webcam.
 * @param {number|string} value - O valor atual do tamanho da webcam.
 */
function updateWebcamSizeDisplay(value) {
  elements.webcamSizeValue.textContent = `${value}%`;
  const percent = ((value - 50) / 250) * 100;
  elements.webcamSizeSlider.style.setProperty('--value', `${percent}%`);
}

/**
 * Salva a configuração no backend.
 * @returns {Promise<void>}
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
      mic_enabled: elements.micSelect.value ? state.config.mic_enabled : false,
      sys_audio_enabled: sysAudioEnabled,
      mic_volume: parseInt(elements.micVolumeSlider.value),
      webcam_enabled: elements.webcamToggle.classList.contains('toggle--active'),
      webcam_device: elements.cameraSelect.value || null,
      webcam_position: document.querySelector('.pos-btn--active')?.dataset.pos || 'bottom-right',
      webcam_size: parseInt(elements.webcamSizeSlider.value),
    };

    await Recorder.updateConfig(config);
    state.isDirty = false;
    await closeWindow({ force: true });
  } catch (error) {
    console.error('Erro ao salvar a configuração:', error);
    setStatusError('Erro ao salvar');
  }
}

/**
 * Restaura para as configurações padrão.
 * @returns {Promise<void>}
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
        webcam_enabled: false,
        webcam_device: null,
        webcam_position: 'bottom-right',
        webcam_size: 100,
      };

      await Recorder.updateConfig(defaultConfig);
      state.config = defaultConfig;
      updateUIFromConfig();
      setStatusSuccess('Resetado!');
    } catch (error) {
      console.error('Erro ao resetar configuração:', error);
      setStatusError('Erro no reset');
    }
  }
}

/**
 * Abre a janela de seleção do diretório de saída.
 * @returns {Promise<void>}
 */
async function selectOutputDirectory() {
  try {
    const selected = await Recorder.openDialog({
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
    console.error('Erro ao selecionar o diretório de saída:', error);
    setStatusError('Erro na pasta');
  }
}

/**
 * Fecha a janela de configurações.
 * @param {Object} [options] - Opções de fechamento.
 * @param {boolean} [options.force=false] - Ignora o prompt de alerta ao ter mudanças não salvas.
 * @returns {Promise<void>}
 */
async function closeWindow({ force = false } = {}) {
  if (!force && state.isDirty && !confirm('Sair sem salvar?')) {
    return;
  }

  try {
    await Recorder.hideSettings();
    return;
  } catch (error) {
    console.warn('Falha ao ocultar a janela de configurações usando comando backend:', error);
  }

  try {
    const tauriWindow = Recorder.getCurrentWindow();

    if (tauriWindow?.hide) {
      await tauriWindow.hide();
      return;
    }
  }
  catch (error) {
    console.warn('Falha ao ocultar a janela de configurações via API do Tauri:', error);
  }

  window.close();
}

/**
 * Define o status visual como "Carregando".
 * @param {string} message - A mensagem a ser exibida.
 */
function setStatusLoading(message) {
  elements.statusIndicator.classList.add('status__dot--loading');
  elements.statusMessage.textContent = message;
}

/**
 * Define o status visual como "Sucesso".
 * @param {string} message - A mensagem a ser exibida.
 */
function setStatusSuccess(message) {
  elements.statusIndicator.classList.remove('status__dot--loading', 'status__dot--error');
  elements.statusMessage.textContent = message;
}

/**
 * Define o status visual como "Erro".
 * @param {string} message - A mensagem a ser exibida.
 */
function setStatusError(message) {
  elements.statusIndicator.classList.add('status__dot--error');
  elements.statusIndicator.classList.remove('status__dot--loading');
  elements.statusMessage.textContent = message;
}

// Inicializa no carregamento do DOM
document.addEventListener('DOMContentLoaded', init);