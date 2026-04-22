/**
 * Rec Corder — Gerenciador de Preferências
 * Lida com o carregamento e migração das configurações do usuário.
 */

import * as recorder from './recorder.js';

/** @type {string} A chave usada para buscar as preferências antigas no LocalStorage. */
const PREFS_KEY = 'rec_corder_prefs';

/**
 * Carrega a configuração do backend e migra as preferências locais herdadas, se necessário.
 * @returns {Promise<Object|null>} A configuração carregada ou null em caso de falha.
 */
export async function loadAndMigratePrefs() {
  try {
    let config = await recorder.getConfig();
    const saved = localStorage.getItem(PREFS_KEY);
    if (saved) {
      console.log('Migrando as preferências antigas para o backend...');
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
        console.warn('Erro ao converter preferências antigas:', e);
      }
    }
    return config;
  } catch (e) {
    console.warn('Falha ao carregar/migrar as preferências do backend:', e);
    return null;
  }
}

/**
 * Salva o objeto de configuração fornecido no backend.
 * @param {Object} config - O objeto de configuração a ser salvo.
 * @returns {Promise<void>}
 */
export async function savePrefs(config) {
  try {
    await recorder.updateConfig(config);
  } catch (e) {
    console.warn('Falha ao salvar a configuração no backend:', e);
  }
}