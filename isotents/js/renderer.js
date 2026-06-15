import { toScreen, TILE_W, TILE_H, SPRITE_H } from "./iso.js";
import { getSprite, baseOffsetY } from "./assets.js";

/**
 * Render the ground-level grid (grass tiles under the puzzle).
 * ground[row][col] = sprite key (e.g. "grass").
 */
export function renderGroundMap(ctx, ground) {
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
    }
  }
}

// ── Puzzle rendering ─────────────────────────────────────────

/** Draw a single object sprite anchored on tile (row, col). */
function drawSprite(ctx, name, row, col) {
  const img = getSprite(name);
  if (!img) return;
  const { x: sx, y: sy } = toScreen(col, row);
  const drawX = Math.round(sx - img.width / 2);
  const drawY = Math.round(sy - img.height + baseOffsetY(name));
  ctx.drawImage(img, drawX, drawY);
}

/** Trees are fixed; treeSprites[row][col] picks the variant. */
export function renderTrees(ctx, board, treeSprites) {
  const { size, trees } = board;
  for (let row = 0; row < size; row++) {
    for (let col = 0; col < size; col++) {
      if (trees[row][col]) drawSprite(ctx, treeSprites[row][col], row, col);
    }
  }
}

/** Houses placed by the player (placement[r][c] === HOUSE). */
export function renderHouses(ctx, placement, houseSprites, HOUSE) {
  for (let row = 0; row < placement.length; row++) {
    for (let col = 0; col < placement[row].length; col++) {
      if (placement[row][col] === HOUSE) {
        drawSprite(ctx, houseSprites[row][col], row, col);
      }
    }
  }
}

/** Small "×" on cells the player flagged as definitely empty. */
export function renderMarkers(ctx, placement, MARKED) {
  ctx.save();
  ctx.strokeStyle = "rgba(40,40,40,0.7)";
  ctx.lineWidth = 2.5;
  ctx.lineCap = "round";
  const s = 7;
  for (let row = 0; row < placement.length; row++) {
    for (let col = 0; col < placement[row].length; col++) {
      if (placement[row][col] !== MARKED) continue;
      const { x, y } = toScreen(col, row);
      ctx.beginPath();
      ctx.moveTo(x - s, y - s);
      ctx.lineTo(x + s, y + s);
      ctx.moveTo(x + s, y - s);
      ctx.lineTo(x - s, y + s);
      ctx.stroke();
    }
  }
  ctx.restore();
}

/** Red diamond under any house that touches another house (rule 1 violation). */
export function renderViolations(ctx, adjacent) {
  for (let row = 0; row < adjacent.length; row++) {
    for (let col = 0; col < adjacent[row].length; col++) {
      if (adjacent[row][col]) {
        highlightTile(ctx, col, row, "rgba(230,40,40,0.45)");
      }
    }
  }
}

const CLUE_COLORS = { under: "#fff", exact: "#5d5", over: "#f55" };

/**
 * Draw the row/column clue numbers along the two upper iso edges.
 * Column clues sit north of each column (row = -1, upper-right edge);
 * row clues sit west of each row (col = -1, upper-left edge).
 */
export function renderClues(ctx, board, validation) {
  const { size, rowClues, colClues } = board;
  ctx.save();
  ctx.font = "bold 16px monospace";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";

  for (let c = 0; c < size; c++) {
    const { x, y } = toScreen(c, -1);
    drawClue(ctx, x, y, colClues[c], CLUE_COLORS[validation.colStatus[c]]);
  }
  for (let r = 0; r < size; r++) {
    const { x, y } = toScreen(-1, r);
    drawClue(ctx, x, y, rowClues[r], CLUE_COLORS[validation.rowStatus[r]]);
  }
  ctx.restore();
}

function drawClue(ctx, x, y, value, color) {
  ctx.fillStyle = "rgba(0,0,0,0.55)";
  ctx.beginPath();
  ctx.arc(x, y, 13, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = color;
  ctx.fillText(String(value), x, y + 1);
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
