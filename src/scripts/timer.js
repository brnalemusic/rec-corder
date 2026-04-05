import { formatDuration } from './utils.js';

let timerInterval = null;
let elapsedSecs = 0;

export function startTimer(displayElement) {
  stopTimer();
  elapsedSecs = 0;
  if (displayElement) displayElement.textContent = formatDuration(elapsedSecs);
  
  timerInterval = setInterval(() => {
    elapsedSecs += 1;
    if (displayElement) displayElement.textContent = formatDuration(elapsedSecs);
  }, 1000);
}

export function stopTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
}
