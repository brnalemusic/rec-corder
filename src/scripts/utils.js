/**
 * Rec Corder — Utilidades
 * Funções utilitárias de formatação de tempo e geração de nomes de arquivos.
 */

/**
 * Formata os segundos em uma string no formato HH:MM:SS.
 * @param {number} totalSecs - O número total de segundos.
 * @returns {string} A string de tempo formatada.
 */
export function formatDuration(totalSecs) {
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs % 3600) / 60);
  const s = totalSecs % 60;
  return [h, m, s].map((v) => String(v).padStart(2, '0')).join(':');
}

/**
 * Trunca um caminho de arquivo para exibição, mostrando apenas os últimos N segmentos.
 * @param {string} fullPath - O caminho completo do arquivo.
 * @param {number} [segments=3] - O número de segmentos para manter.
 * @returns {string} O caminho truncado.
 */
export function truncatePath(fullPath, segments = 3) {
  if (!fullPath) return '...';
  const parts = fullPath.replace(/\\/g, '/').split('/');
  if (parts.length <= segments) return fullPath;
  return '...' + parts.slice(-segments).join('/');
}