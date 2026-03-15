/**
 * Isometric coordinate helpers.
 * TILE_W  = horizontal span of one diamond   (90)
 * TILE_H  = vertical span of the diamond top  (52)
 */

export const TILE_W = 90;
export const TILE_H = 52;

// Full sprite dimensions (for drawing offset calculations)
export const SPRITE_W = 95;
export const SPRITE_H = 97;

/**
 * Convert grid (col, row) → screen (px, py).
 * The result is the position where the *center-top* of the diamond sits.
 */
export function toScreen(col, row, offsetX = 0, offsetY = 0) {
  return {
    x: Math.round((col - row) * (TILE_W / 2) + offsetX),
    y: Math.round((col + row) * (TILE_H / 2) + offsetY),
  };
}

/**
 * Convert screen (px, py) → grid (col, row).
 * Inverse of toScreen — useful for mouse picking.
 */
export function toGrid(px, py) {
  const col = (px / (TILE_W / 2) + py / (TILE_H / 2)) / 2;
  const row = (py / (TILE_H / 2) - px / (TILE_W / 2)) / 2;
  return { col: Math.round(col), row: Math.round(row) };
}
