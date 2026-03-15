/**
 * Game state and turn logic.
 *
 * Rules (from PLAN.md):
 * 1. Turn-based game
 * 2. Player has resources: food, wood, water, stone. Starts with 1 AP.
 * 3. Initially the player places a house anywhere on the map.
 * 4. Each turn with an AP the player can place a workshop to harvest resources.
 */

// ── State ────────────────────────────────────────────────────
const state = {
  turn: 1,
  ap: 1,
  resources: { food: 0, wood: 0, water: 0, stone: 0 },
  phase: "place_house", // "place_house" | "play" | "turn_over"
  buildings: [],        // { row, col, type }
};

export function getState() {
  return state;
}

// ── Queries ──────────────────────────────────────────────────

/** Can the player place something on this tile? */
export function canPlace(ground, objects, row, col) {
  if (ground[row][col].startsWith("river")) return false;
  if (objects[row][col] !== null) return false;
  return true;
}

/** Returns what the player is currently allowed to place, or null. */
export function currentPlaceable() {
  if (state.phase === "place_house") return "house";
  if (state.phase === "play" && state.ap > 0) return "workshop";
  return null;
}

// ── Actions ──────────────────────────────────────────────────

/**
 * Place a building on the map. Returns true if successful.
 */
export function placeBuilding(objects, row, col) {
  const type = currentPlaceable();
  if (!type) return false;

  if (type === "house") {
    objects[row][col] = "white_blue_house1";
    state.buildings.push({ row, col, type: "house" });
    state.phase = "play";
    return true;
  }

  if (type === "workshop") {
    objects[row][col] = "workshop_blue1";
    state.buildings.push({ row, col, type: "workshop" });
    state.ap--;
    harvestResources(objects, row, col);
    if (state.ap <= 0) {
      state.phase = "turn_over";
    }
    return true;
  }

  return false;
}

/**
 * Harvest resources based on surrounding tiles.
 * Workshop checks its 8 neighbours for harvestable objects.
 */
function harvestResources(objects, row, col) {
  const size = objects.length;
  for (let dr = -1; dr <= 1; dr++) {
    for (let dc = -1; dc <= 1; dc++) {
      if (dr === 0 && dc === 0) continue;
      const r = row + dr, c = col + dc;
      if (r < 0 || r >= size || c < 0 || c >= size) continue;
      const obj = objects[r][c];
      if (!obj) continue;
      if (obj.startsWith("tree")) state.resources.wood++;
      if (obj.startsWith("stone")) state.resources.stone++;
    }
  }
}

/**
 * End the current turn and start a new one.
 */
export function endTurn() {
  // Each house produces 1 food per turn
  for (const b of state.buildings) {
    if (b.type === "house") state.resources.food++;
  }

  state.turn++;
  state.ap = 1;
  state.phase = "play";
}
