/**
 * Rec Corder — Gerenciador de cronômetro
 * Lida com o controle e exibição do tempo de gravação.
 */

import { formatDuration } from './utils.js';

/** @type {number|null} Variável interna para armazenar o ID do intervalo do cronômetro. */
let timerInterval = null;

/** @type {number} Segundos decorridos desde o início do cronômetro. */
let elapsedSecs = 0;

/**
 * Inicia o cronômetro e atualiza o elemento de exibição.
 * @param {HTMLElement} displayElement - O elemento DOM onde o tempo será exibido.
 */
export function startTimer(displayElement) {
  stopTimer();
  elapsedSecs = 0;
  if (displayElement) displayElement.textContent = formatDuration(elapsedSecs);
  
  timerInterval = setInterval(() => {
    elapsedSecs += 1;
    if (displayElement) displayElement.textContent = formatDuration(elapsedSecs);
  }, 1000);
}

/**
 * Para o cronômetro atual.
 */
export function stopTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
}