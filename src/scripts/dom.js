/**
 * Rec Corder — Módulo DOM
 * Fornece referências cacheadas para elementos da interface do usuário e configurações de segurança do DOM.
 */

/**
 * Atalho para buscar elementos por ID.
 * @param {string} id - O ID do elemento DOM.
 * @returns {HTMLElement | null} O elemento encontrado ou null.
 */
export const $ = (id) => document.getElementById(id);

/**
 * @type {Object<string, HTMLElement | null>}
 * Dicionário contendo referências cacheadas para elementos cruciais da interface.
 */
export const dom = {
  timerDisplay: $('timerDisplay'),
  timerLabel: $('timerLabel'),
  btnRecord: $('btnRecord'),
  recIndicator: $('recIndicator'),
  monitorSelect: $('monitorSelect'),
  micToggle: $('micToggle'),
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
  webcamToggle: $('webcamToggle'),
  webcamSetting: $('webcamSetting'),
  versionDisplay: $('versionDisplay'),
  exitModalBackdrop: $('exitModalBackdrop'),
  closeExitModal: $('closeExitModal'),
  cancelExitBtn: $('cancelExitBtn'),
  confirmExitBtn: $('confirmExitBtn'),
};

/**
 * Configura as restrições de segurança na interface, desativando menu de contexto e atalhos de ferramentas de desenvolvedor.
 */
export function setupSecurity() {
  document.addEventListener('contextmenu', e => e.preventDefault());
  document.addEventListener('keydown', e => {
    if (e.key === 'F12') e.preventDefault();
    if (e.ctrlKey && (e.shiftKey && (e.key === 'I' || e.key === 'J' || e.key === 'C') || e.key === 'u')) {
      e.preventDefault();
    }
  });
}