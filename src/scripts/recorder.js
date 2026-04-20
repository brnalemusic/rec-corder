/**
 * Rec Corder — Ponte de comandos do Tauri
 * Wrappers tipados para cada comando do Rust.
 * Única fonte de verdade para o contrato frontend ↔ backend.
 */

const { invoke } = window.__TAURI__.core;

/**
 * @typedef {{ is_recording: boolean, elapsed_secs: number, output_file: string|null }} RecordingStatus
 * @typedef {{ file_path: string }} StartResult
 * @typedef {{ index: number, name: string, is_primary: boolean }} MonitorInfo
 * @typedef {{ id: string, name: string, is_default: boolean }} MicInfo
 * @typedef {{ id: string, name: string, is_default: boolean }} AudioOutputInfo
 * @typedef {{ name: string, id: string }} CameraInfo
 */

/** 
 * Obtém o status atual da gravação. 
 * @returns {Promise<RecordingStatus>}
 */
export async function getStatus() {
  return invoke('get_status');
}

/** 
 * Obtém a configuração central. 
 * @returns {Promise<Object>}
 */
export async function getConfig() {
  return invoke('get_config');
}

/** 
 * Atualiza a configuração central.
 * @param {Object} config - O novo objeto de configuração.
 * @returns {Promise<void>}
 */
export async function updateConfig(config) {
  return invoke('update_config', { config });
}

/** 
 * Lista os monitores disponíveis. 
 * @returns {Promise<MonitorInfo[]>}
 */
export async function listMonitors() {
  return invoke('list_monitors');
}

/** 
 * Lista os microfones disponíveis. 
 * @returns {Promise<MicInfo[]>}
 */
export async function listMics() {
  return invoke('list_mics');
}

/**
 * Inicia a gravação da tela.
 * @param {Object} options - Opções de gravação.
 * @param {number} [options.monitorIndex=0] - O índice do monitor a gravar.
 * @param {string|null} [options.micName=null] - O ID do microfone (opcional).
 * @param {string|null} [options.systemAudioDevice=null] - O ID do dispositivo de áudio do sistema (opcional).
 * @param {number} [options.fps=60] - Frames por segundo da gravação.
 * @param {number} [options.scaleFactor=100] - Fator de escala da gravação.
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

/** 
 * Lista as saídas de áudio disponíveis (dispositivos de renderização WASAPI). 
 * @returns {Promise<AudioOutputInfo[]>}
 */
export async function listAudioOutputs() {
  return invoke('list_audio_outputs');
}

/** 
 * Para a gravação atual. Retorna o caminho do arquivo de saída. 
 * @returns {Promise<string>}
 */
export async function stopRecording() {
  return invoke('stop_recording');
}

/** 
 * Obtém o diretório de saída atual. 
 * @returns {Promise<string>}
 */
export async function getOutputDir() {
  return invoke('get_output_dir');
}

/**
 * Define um novo diretório de saída.
 * @param {string} path - O novo caminho de pasta para as gravações.
 * @returns {Promise<void>}
 */
export async function setOutputDir(path) {
  return invoke('set_output_dir', { path });
}

/** 
 * Mostra a janela de configurações. 
 * @returns {Promise<void>}
 */
export async function showSettings() {
  return invoke('show_settings');
}

/** 
 * Oculta a janela de configurações sem destruir o webview. 
 * @returns {Promise<void>}
 */
export async function hideSettings() {
  return invoke('hide_settings');
}

/** 
 * Verifica se existem gravações incompletas derivadas de uma falha (crash). 
 * @returns {Promise<boolean>}
 */
export async function check_crash_recovery() {
  return invoke('check_crash_recovery');
}

/** 
 * Obtém informações do aplicativo e status de primeira execução. 
 * @returns {Promise<{version: string, is_first_run: boolean}>}
 */
export async function getAppInfo() {
  return invoke('get_app_info');
}

/** 
 * Marca o pop-up de boas-vindas como visualizado/reconhecido. 
 * @returns {Promise<void>}
 */
export async function acknowledgeWelcome() {
  return invoke('acknowledge_welcome');
}

/** 
 * Obtém a versão do aplicativo a partir da API do Tauri. 
 * @returns {Promise<string>}
 */
export async function getAppVersion() {
  try {
    return await window.__TAURI__.app.getVersion();
  } catch (e) {
    console.warn('Falha ao obter versão pela API do Tauri, usando fallback para o backend:', e);
    const info = await getAppInfo();
    return info.version;
  }
}


/** 
 * Verifica se há atualizações através do backend. 
 * @returns {Promise<[string, string]|null>} Uma tupla contendo a versão e as notas (ou null se não houver atualização).
 */
export async function checkForUpdates() {
  return invoke('check_for_updates');
}

/** 
 * Mostra a janela de atualização do aplicativo. 
 * @param {string} version - A nova versão disponível.
 * @param {string|null} [body=null] - O conteúdo (notas de lançamento).
 * @returns {Promise<void>}
 */
export async function showUpdater(version, body = null) {
  return invoke('show_updater', { version, body });
}

/** 
 * Obtém as notas de lançamento para uma versão específica. 
 * @param {string} version - A versão desejada.
 * @returns {Promise<string>}
 */
export async function getReleaseNotes(version) {
  return invoke('get_release_notes', { version });
}

/** 
 * Mostra a janela com as notas de lançamento. 
 * @returns {Promise<void>}
 */
export async function showReleaseNotes() {
  return invoke('show_release_notes');
}

/** 
 * Instala a atualização pendente (controlado pelo backend). 
 * @returns {Promise<void>}
 */
export async function installUpdate() {
  return invoke('install_update');
}

/** 
 * Lista as câmeras disponíveis (dispositivos de vídeo DirectShow). 
 * @returns {Promise<CameraInfo[]>}
 */
export async function listCameras() {
  return invoke('list_cameras');
}