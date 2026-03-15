import { toScreen, TILE_W, TILE_H, SPRITE_H } from "./iso.js";
import { getSprite, baseOffsetY } from "./assets.js";

/**
 * Render the ground-level grid (grass, ground, rivers).
 * ground[row][col] = "grass" | "ground1" | "river3" | …
 */
export function renderGroundMap(ctx, ground, debug = false) {
  const rows = ground.length;
  const cols = ground[0].length;

  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const spriteName = ground[row][col];
      if (!spriteName) continue;

      const img = getSprite(spriteName);
      if (!img) continue;

      const { x: sx, y: sy } = toScreen(col, row);
      const drawX = Math.round(sx - img.width / 2);
      const drawY = Math.round(sy - img.height + SPRITE_H / 2 + 24);

      ctx.drawImage(img, drawX, drawY);
      if (debug) {
        drawLabel(ctx, sx, sy, spriteName);
        drawLabel(ctx, sx, sy + 12, `(${row},${col})`, "#aaf");
      }
    }
  }
}

/**
 * Render the above-ground object grid (trees, stones, houses).
 * objects[row][col] = "tree1" | "stone2" | null
 * Drawn in a second pass so all objects sit visually on top of all ground tiles.
 */
export function renderObjectMap(
  ctx,
  objects,
  debug = false,
  offsetX = 0,
  offsetY = 0,
) {
  const rows = objects.length;
  const cols = objects[0].length;

  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const spriteName = objects[row][col];
      if (!spriteName) continue;

      const img = getSprite(spriteName);
      if (!img) continue;

      const tileOffY = baseOffsetY(spriteName);
      const { x: sx, y: sy } = toScreen(col, row);
      const drawX = Math.round(sx - img.width / 2 + offsetX);
      // Base of sprite sits on the tile surface; per-tile base + dat.gui adjustment
      const drawY = Math.round(sy - img.height + tileOffY + offsetY);

      ctx.drawImage(img, drawX, drawY);
      if (debug) {
        drawLabel(ctx, sx + offsetX, sy + tileOffY + offsetY, spriteName);
      }
    }
  }
}

/**
 * Render the detail-level grid (grass_detail overlays).
 * detail[row][col] = "grass_detail1" | null
 * Drawn in a third pass on top of ground and base layers.
 */
export function renderDetailMap(ctx, detail, debug = false, offsetX = 0, offsetY = 0) {
  const rows = detail.length;
  const cols = detail[0].length;

  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const spriteName = detail[row][col];
      if (!spriteName) continue;

      const img = getSprite(spriteName);
      if (!img) continue;

      const { x: sx, y: sy } = toScreen(col, row);
      const drawX = Math.round(sx - img.width / 2 + offsetX);
      const drawY = Math.round(sy - img.height + 15 + offsetY);

      ctx.drawImage(img, drawX, drawY);
      if (debug) {
        drawLabel(ctx, sx + offsetX, sy + 15 + offsetY, spriteName, "#ff0");
      }
    }
  }
}

function drawLabel(ctx, x, y, label, color = "#fff") {
  ctx.font = "bold 9px monospace";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillStyle = "rgba(0,0,0,0.6)";
  const w = ctx.measureText(label).width + 6;
  ctx.fillRect(Math.round(x - w / 2), y - 6, w, 12);
  ctx.fillStyle = color;
  ctx.fillText(label, x, y);
}

/**
 * Render a highlight on a specific tile (e.g. for hover / selection).
 */
export function highlightTile(ctx, col, row, color = "rgba(255,255,255,0.3)") {
  const { x: sx, y: sy } = toScreen(col, row);
  const hw = TILE_W / 2;
  const hh = 24; // half the diamond height

  ctx.beginPath();
  ctx.moveTo(sx, sy - hh); // top
  ctx.lineTo(sx + hw, sy); // right
  ctx.lineTo(sx, sy + hh); // bottom
  ctx.lineTo(sx - hw, sy); // left
  ctx.closePath();
  ctx.fillStyle = color;
  ctx.fill();
}

/**
 * Draw a compass rose above the grid showing N/S/E/W in isometric directions.
 * N = row-1 (upper-right), S = row+1 (lower-left),
 * E = col+1 (lower-right), W = col-1 (upper-left).
 */
export function renderCompass(ctx) {
  // Position above the top corner of the grid (col=0, row=0)
  const { x: ox, y: oy } = toScreen(0, 0);
  const cx = ox;
  const cy = oy - 80;
  const len = 40;

  // Isometric axis directions (unit vectors scaled by len)
  // N (row-1): screen direction is (+TILE_W/2, -TILE_H/2) normalized
  // S (row+1): opposite of N
  // E (col+1): screen direction is (+TILE_W/2, +TILE_H/2) normalized
  // W (col-1): opposite of E
  const nx = TILE_W / 2,
    ny = -TILE_H / 2;
  const ex = TILE_W / 2,
    ey = TILE_H / 2;
  const mag = Math.sqrt(nx * nx + ny * ny);

  const dirs = [
    { label: "N", dx: (nx / mag) * len, dy: (ny / mag) * len },
    { label: "S", dx: (-nx / mag) * len, dy: (-ny / mag) * len },
    { label: "E", dx: (ex / mag) * len, dy: (ey / mag) * len },
    { label: "W", dx: (-ex / mag) * len, dy: (-ey / mag) * len },
  ];

  for (const { label, dx, dy } of dirs) {
    const tx = cx + dx;
    const ty = cy + dy;

    // Arrow line
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(tx, ty);
    ctx.strokeStyle = "#fff";
    ctx.lineWidth = 2;
    ctx.stroke();

    // Arrowhead
    const angle = Math.atan2(dy, dx);
    const headLen = 8;
    ctx.beginPath();
    ctx.moveTo(tx, ty);
    ctx.lineTo(
      tx - headLen * Math.cos(angle - 0.4),
      ty - headLen * Math.sin(angle - 0.4),
    );
    ctx.moveTo(tx, ty);
    ctx.lineTo(
      tx - headLen * Math.cos(angle + 0.4),
      ty - headLen * Math.sin(angle + 0.4),
    );
    ctx.stroke();

    // Label
    const labelDist = len + 14;
    const lx = cx + (dx / len) * labelDist;
    const ly = cy + (dy / len) * labelDist;
    ctx.font = "bold 13px monospace";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillStyle = "#fff";
    ctx.fillText(label, lx, ly);
  }
}
