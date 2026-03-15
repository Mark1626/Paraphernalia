import { tileLayer } from "./assets.js";

/**
 * Map editor state and operations.
 * Three-layer system: ground → base → detail
 */

const state = {
  active: false,
  selectedTile: "grass",    // current brush tile key
  mapSize: 10,
};

export function getEditorState() {
  return state;
}

export function setEditorActive(v) {
  state.active = v;
}

export function setSelectedTile(key) {
  state.selectedTile = key;
}

export function setMapSize(size) {
  state.mapSize = size;
}

// ── Grid operations ──────────────────────────────────────────

/** Paint the selected tile onto the correct layer at (row, col). */
export function paintTile(ground, objects, detail, row, col) {
  const key = state.selectedTile;
  if (!key) return;

  const layer = tileLayer(key);
  if (layer === "ground") {
    ground[row][col] = key;
  } else if (layer === "base") {
    objects[row][col] = key;
  } else if (layer === "detail") {
    detail[row][col] = key;
  }
}

/** Erase a tile: reset ground to grass, clear base and detail. */
export function eraseTile(ground, objects, detail, row, col) {
  ground[row][col] = "grass";
  objects[row][col] = null;
  detail[row][col] = null;
}

// ── Export / Import ──────────────────────────────────────────

/** Export the current map as a JSON string. */
export function exportMap(ground, objects, detail) {
  const size = ground.length;
  return JSON.stringify({ size, ground, objects, detail }, null, 2);
}

/** Import a map from a JSON string. Returns { ground, objects, detail, size }. */
export function importMap(json) {
  const data = JSON.parse(json);
  if (!data.ground || !data.objects || !data.size) {
    throw new Error("Invalid map format");
  }
  // Support importing older 2-layer maps (no detail)
  if (!data.detail) {
    const size = data.size;
    data.detail = [];
    for (let row = 0; row < size; row++) {
      data.detail[row] = [];
      for (let col = 0; col < size; col++) {
        data.detail[row][col] = null;
      }
    }
  }
  return { ground: data.ground, objects: data.objects, detail: data.detail, size: data.size };
}

/** Create a blank map filled with grass. */
export function createBlankMap(size) {
  const ground = [];
  const objects = [];
  const detail = [];
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
  return { ground, objects, detail };
}
