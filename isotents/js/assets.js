const ASSET_BASE = "assets/IsometricMedievalPack/Sprites";

// Asset manifest — keys become the sprite names used by the puzzle.
// Only the sprites the game actually draws are listed (see TREE_SPRITES /
// HOUSE_SPRITES in puzzle.js); the medieval pack ships many more on disk.
const MANIFEST = {
  // Ground
  grass: `${ASSET_BASE}/Enviroument/Spring/grass.png`,

  // Trees (fixed board obstacles)
  tree1: `${ASSET_BASE}/Enviroument/Spring/trees/tree(1).png`,
  tree2: `${ASSET_BASE}/Enviroument/Spring/trees/tree(2).png`,
  tree3: `${ASSET_BASE}/Enviroument/Spring/trees/tree(3).png`,
  tree4: `${ASSET_BASE}/Enviroument/Spring/trees/tree(4).png`,

  // Houses (player placements)
  white_blue_house1: `${ASSET_BASE}/Buildings/house/White houses/Blue house/white_blue_house(1).png`,
  white_red_house1: `${ASSET_BASE}/Buildings/house/White houses/Red house/white_red_house(1).png`,
  white_green_house1: `${ASSET_BASE}/Buildings/house/White houses/Green house/white_green_house(1).png`,
  wood_blue_house1: `${ASSET_BASE}/Buildings/house/Wood houses/Blue house/wood_blue_house(1).png`,
};

/** Per-sprite vertical anchor offset for base-layer object rendering. */
export function baseOffsetY(key) {
  if (key.startsWith("white_") || key.startsWith("wood_")) return 20; // houses
  return 15; // trees
}

/** @type {Map<string, HTMLImageElement>} */
const sprites = new Map();

/**
 * Load all assets from the manifest. Returns a promise that resolves
 * when every image has loaded (or rejects on first failure).
 */
export function loadAssets() {
  const entries = Object.entries(MANIFEST);
  let loaded = 0;

  return new Promise((resolve, reject) => {
    for (const [name, src] of entries) {
      const img = new Image();
      img.src = src;
      img.onload = () => {
        sprites.set(name, img);
        loaded++;
        if (loaded === entries.length) resolve(sprites);
      };
      img.onerror = () => reject(new Error(`Failed to load: ${src}`));
    }
  });
}

/**
 * Get a loaded sprite by name.
 * @param {string} name
 * @returns {HTMLImageElement}
 */
export function getSprite(name) {
  return sprites.get(name);
}
