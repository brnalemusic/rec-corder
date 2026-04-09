export const $ = (id) => document.getElementById(id);

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

export function setupSecurity() {
  document.addEventListener('contextmenu', e => e.preventDefault());
  document.addEventListener('keydown', e => {
    if (e.key === 'F12') e.preventDefault();
    if (e.ctrlKey && (e.shiftKey && (e.key === 'I' || e.key === 'J' || e.key === 'C') || e.key === 'u')) {
      e.preventDefault();
    }
  });
}
