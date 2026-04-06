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
