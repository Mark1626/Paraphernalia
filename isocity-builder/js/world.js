// ── River generation ───────────────────────────────────────────
// Sprite mapping by connection directions (sorted alphabetically)
// N = row-1, S = row+1, E = col+1, W = col-1
// Asset naming uses rotated compass: asset N=our W, asset E=our N, asset S=our E, asset W=our S
const RIVER_SPRITES = {
  // Straights
  "N,S": "river_2_n_s",   // straight along row axis
  "E,W": "river_2_e_w",   // straight along col axis
  // Corners
  "E,N": "river_2_n_e",   // corner: N + E
  "N,W": "river_2_n_w",   // corner: N + W
  "E,S": "river_2_e_s",   // corner: S + E
  "S,W": "river_2_s_w",   // corner: S + W
  // T-junctions
  "E,S,W": "river_3_e_s_w",  // T: E + S + W
  "E,N,S": "river_3_n_e_s",  // T: N + E + S
  "E,N,W": "river_3_n_e_w",  // T: N + E + W
  "N,S,W": "river_3_n_s_w",  // T: N + S + W
  // Cross
  "E,N,S,W": "river_4_n_e_s_w",
  // End pieces
  "N": "river_1_n",
  "S": "river_1_s",
  "E": "river_1_e",
  "W": "river_1_w",
};

export function createDemoMap(size) {
  const ground = [];
  const objects = [];
  const detail = [];

  // Rule 1: By default always place ground(1) or grass sprites
  for (let row = 0; row < size; row++) {
    ground[row] = [];
    objects[row] = [];
    detail[row] = [];
    for (let col = 0; col < size; col++) {
      ground[row][col] = "grass";
      objects[row][col] = null;
      detail[row][col] = null;
    }
  }

  // Rule 2: Rivers replace ground/grass from one side of the map to another
  placeRiver(ground, size);

  // Rule 3: Trees and stones are placed on top of ground tiles
  placeTrees(ground, objects, size);
  placeRocks(ground, objects, size);

  return { ground, objects, detail };
}

function isOpen(ground, objects, row, col, size) {
  return row >= 0 && row < size && col >= 0 && col < size
    && !ground[row][col].startsWith("river") && objects[row][col] === null;
}

/**
 * Place trees in 3x3 subgrid clusters with at least 3 trees per cluster.
 */
function placeTrees(ground, objects, size) {
  const clusterChance = 0.08;

  for (let row = 0; row < size - 2; row++) {
    for (let col = 0; col < size - 2; col++) {
      if (Math.random() > clusterChance) continue;

      // Collect open cells in this 3x3 subgrid
      const candidates = [];
      for (let dr = 0; dr < 3; dr++) {
        for (let dc = 0; dc < 3; dc++) {
          const r = row + dr, c = col + dc;
          if (isOpen(ground, objects, r, c, size)) {
            candidates.push({ r, c });
          }
        }
      }

      // Need at least 3 open cells
      if (candidates.length < 3) continue;

      // Shuffle and pick 3 to 5 trees (at least 3)
      shuffle(candidates);
      const count = 3 + Math.floor(Math.random() * Math.min(3, candidates.length - 2));
      for (let i = 0; i < Math.min(count, candidates.length); i++) {
        const { r, c } = candidates[i];
        objects[r][c] = "tree" + Math.ceil(Math.random() * 4);
      }
    }
  }
}

/**
 * Place rocks as 1x1 or 2x2 blocks.
 */
function placeRocks(ground, objects, size) {
  const rockChance = 0.04;

  for (let row = 0; row < size; row++) {
    for (let col = 0; col < size; col++) {
      if (Math.random() > rockChance) continue;
      if (!isOpen(ground, objects, row, col, size)) continue;

      const use2x2 = Math.random() < 0.3
        && isOpen(ground, objects, row + 1, col, size)
        && isOpen(ground, objects, row, col + 1, size)
        && isOpen(ground, objects, row + 1, col + 1, size);

      if (use2x2) {
        for (let dr = 0; dr < 2; dr++) {
          for (let dc = 0; dc < 2; dc++) {
            objects[row + dr][col + dc] = "stone" + Math.ceil(Math.random() * 4);
          }
        }
      } else {
        objects[row][col] = "stone" + Math.ceil(Math.random() * 4);
      }
    }
  }
}

function shuffle(arr) {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
}

function placeRiver(grid, size) {
  const path = simplifyPath(generateRiverPath(size));

  for (let i = 0; i < path.length; i++) {
    const { row, col } = path[i];
    const prev = i > 0 ? path[i - 1] : null;
    const next = i < path.length - 1 ? path[i + 1] : null;
    grid[row][col] = pickRiverSprite(row, col, prev, next);
  }
}

function generateRiverPath(size) {
  const path = [];
  const vertical = Math.random() < 0.5;
  const center = Math.floor(size / 2);
  // Start near the center of the entry edge (within middle 40%)
  const margin = Math.floor(size * 0.3);

  if (vertical) {
    let col = center - margin + Math.floor(Math.random() * (margin * 2 + 1));
    // Bias col toward center
    col = Math.round((col + center) / 2);
    let sideways = 0; // track consecutive sideways steps for 90° turns

    for (let row = 0; row < size; row++) {
      path.push({ row, col });
      if (row < size - 1) {
        const drift = Math.random();
        // Higher drift chance (0.45 each side) for more turns
        if (drift < 0.45 && col > 0) {
          col--;
          path.push({ row, col });
          sideways++;
          // Allow consecutive sideways steps for 90° turns (up to 2)
          if (sideways < 2 && Math.random() < 0.4 && col > 0) {
            col--;
            path.push({ row, col });
            sideways++;
          }
        } else if (drift > 0.55 && col < size - 1) {
          col++;
          path.push({ row, col });
          sideways++;
          if (sideways < 2 && Math.random() < 0.4 && col < size - 1) {
            col++;
            path.push({ row, col });
            sideways++;
          }
        } else {
          sideways = 0;
        }
        // Pull back toward center to keep river from hugging edges
        if (col < margin) col++;
        else if (col > size - 1 - margin) col--;
      }
    }
  } else {
    let row = center - margin + Math.floor(Math.random() * (margin * 2 + 1));
    row = Math.round((row + center) / 2);
    let sideways = 0;

    for (let col = 0; col < size; col++) {
      path.push({ row, col });
      if (col < size - 1) {
        const drift = Math.random();
        if (drift < 0.45 && row > 0) {
          row--;
          path.push({ row, col });
          sideways++;
          if (sideways < 2 && Math.random() < 0.4 && row > 0) {
            row--;
            path.push({ row, col });
            sideways++;
          }
        } else if (drift > 0.55 && row < size - 1) {
          row++;
          path.push({ row, col });
          sideways++;
          if (sideways < 2 && Math.random() < 0.4 && row < size - 1) {
            row++;
            path.push({ row, col });
            sideways++;
          }
        } else {
          sideways = 0;
        }
        if (row < margin) row++;
        else if (row > size - 1 - margin) row--;
      }
    }
  }

  return path;
}

/**
 * Remove U-turn detours from a river path.
 * e.g. (6,3),(6,4),(5,4),(5,5),(6,5),(6,6) → (6,3),(6,4),(6,5),(6,6)
 * Scans for segments that leave and return to the same row (or col),
 * replacing the detour with a straight connection.
 */
function simplifyPath(path) {
  let changed = true;
  while (changed) {
    changed = false;
    for (let i = 0; i < path.length - 2; i++) {
      // Look ahead for a point that shares the same row or col as path[i]
      // and is only 1 step away on the other axis, meaning we can skip the detour
      for (let j = i + 2; j < path.length; j++) {
        const a = path[i];
        const b = path[j];
        const sameRow = a.row === b.row;
        const sameCol = a.col === b.col;
        if (!sameRow && !sameCol) continue;

        // Build the direct segment from a to b
        const segment = [];
        if (sameRow) {
          const step = b.col > a.col ? 1 : -1;
          for (let c = a.col; c !== b.col; c += step) {
            segment.push({ row: a.row, col: c });
          }
        } else {
          const step = b.row > a.row ? 1 : -1;
          for (let r = a.row; r !== b.row; r += step) {
            segment.push({ row: r, col: a.col });
          }
        }

        // Only simplify if the direct path is shorter than the detour
        const detourLen = j - i;
        if (segment.length < detourLen) {
          path.splice(i, detourLen, ...segment);
          changed = true;
          break;
        }
      }
      if (changed) break;
    }
  }
  return path;
}

function pickRiverSprite(row, col, prev, next) {
  const dirs = new Set();

  if (prev) {
    if (prev.row < row) dirs.add("N");
    if (prev.row > row) dirs.add("S");
    if (prev.col < col) dirs.add("W");
    if (prev.col > col) dirs.add("E");
  }

  if (next) {
    if (next.row < row) dirs.add("N");
    if (next.row > row) dirs.add("S");
    if (next.col < col) dirs.add("W");
    if (next.col > col) dirs.add("E");
  }

  const key = [...dirs].sort().join(",");
  return RIVER_SPRITES[key] || "river_4_n_e_s_w";
}
