/**
 * Rec Corder — Utility functions
 * Timer formatting and file name generation.
 */

/**
 * Format seconds into HH:MM:SS string.
 * @param {number} totalSecs
 * @returns {string}
 */
export function formatDuration(totalSecs) {
  const h = Math.floor(totalSecs / 3600);
  const m = Math.floor((totalSecs % 3600) / 60);
  const s = totalSecs % 60;
  return [h, m, s].map((v) => String(v).padStart(2, '0')).join(':');
}

/**
 * Truncate a file path for display, showing only the last N segments.
 * @param {string} fullPath
 * @param {number} segments
 * @returns {string}
 */
export function truncatePath(fullPath, segments = 3) {
  if (!fullPath) return '...';
  const parts = fullPath.replace(/\\/g, '/').split('/');
  if (parts.length <= segments) return fullPath;
  return '...' + parts.slice(-segments).join('/');
}
