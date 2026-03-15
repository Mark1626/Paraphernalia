const ASSET_BASE = "assets/IsometricMedievalPack/Sprites";

// Asset manifest — keys become the sprite names used in tile maps
const MANIFEST = {
  // Environment - Spring
  grass: `${ASSET_BASE}/Enviroument/Spring/grass.png`,
  ground1: `${ASSET_BASE}/Enviroument/Spring/ground(1).png`,
  ground2: `${ASSET_BASE}/Enviroument/Spring/ground(2).png`,
  grass_detail1: `${ASSET_BASE}/Enviroument/Spring/grass_detail(1).png`,
  grass_detail2: `${ASSET_BASE}/Enviroument/Spring/grass_detail(2).png`,
  grass_detail3: `${ASSET_BASE}/Enviroument/Spring/grass_detail(3).png`,
  grass_detail4: `${ASSET_BASE}/Enviroument/Spring/grass_detail(4).png`,
  grass_detail5: `${ASSET_BASE}/Enviroument/Spring/grass_detail(5).png`,
  grass_detail6: `${ASSET_BASE}/Enviroument/Spring/grass_detail(6).png`,
  grass_detail7: `${ASSET_BASE}/Enviroument/Spring/grass_detail(7).png`,
  grass_detail8: `${ASSET_BASE}/Enviroument/Spring/grass_detail(8).png`,
  grass_detail9: `${ASSET_BASE}/Enviroument/Spring/grass_detail(9).png`,
  tree1: `${ASSET_BASE}/Enviroument/Spring/trees/tree(1).png`,
  tree2: `${ASSET_BASE}/Enviroument/Spring/trees/tree(2).png`,
  tree3: `${ASSET_BASE}/Enviroument/Spring/trees/tree(3).png`,
  tree4: `${ASSET_BASE}/Enviroument/Spring/trees/tree(4).png`,
  stone1: `${ASSET_BASE}/Enviroument/Spring/stones/stone(1).png`,
  stone2: `${ASSET_BASE}/Enviroument/Spring/stones/stone(2).png`,
  stone3: `${ASSET_BASE}/Enviroument/Spring/stones/stone(3).png`,
  stone4: `${ASSET_BASE}/Enviroument/Spring/stones/stone(4).png`,

  // Roads - Spring
  road_4_n_e_s_w: `${ASSET_BASE}/Roads/Spring/road(1).png`,
  road_2_e_w: `${ASSET_BASE}/Roads/Spring/road(2).png`,
  road_2_n_s: `${ASSET_BASE}/Roads/Spring/road(3).png`,
  road_3_e_s_w: `${ASSET_BASE}/Roads/Spring/road(4).png`,
  road_3_n_e_s: `${ASSET_BASE}/Roads/Spring/road(5).png`,
  road_3_n_e_w: `${ASSET_BASE}/Roads/Spring/road(6).png`,
  road_3_n_s_w: `${ASSET_BASE}/Roads/Spring/road(7).png`,
  road_2_s_w: `${ASSET_BASE}/Roads/Spring/road(8).png`,
  road_2_n_e: `${ASSET_BASE}/Roads/Spring/road(9).png`,
  road_2_n_w: `${ASSET_BASE}/Roads/Spring/road(10).png`,
  road_2_e_s: `${ASSET_BASE}/Roads/Spring/road(11).png`,
  road_1_e: `${ASSET_BASE}/Roads/Spring/road(12).png`,
  road_1_s: `${ASSET_BASE}/Roads/Spring/road(13).png`,
  road_1_n: `${ASSET_BASE}/Roads/Spring/road(14).png`,
  road_1_w: `${ASSET_BASE}/Roads/Spring/road(15).png`,

  // Rivers - Spring
  river_4_n_e_s_w: `${ASSET_BASE}/Rivers/Spring/river(1).png`,
  river_2_n_s: `${ASSET_BASE}/Rivers/Spring/river(2).png`,
  river_2_e_w: `${ASSET_BASE}/Rivers/Spring/river(3).png`,
  river_3_e_s_w: `${ASSET_BASE}/Rivers/Spring/river(4).png`,
  river_3_n_e_s: `${ASSET_BASE}/Rivers/Spring/river(5).png`,
  river_3_n_e_w: `${ASSET_BASE}/Rivers/Spring/river(6).png`,
  river_3_n_s_w: `${ASSET_BASE}/Rivers/Spring/river(7).png`,
  river_2_s_w: `${ASSET_BASE}/Rivers/Spring/river(8).png`,
  river_2_n_e: `${ASSET_BASE}/Rivers/Spring/river(9).png`,
  river_2_n_w: `${ASSET_BASE}/Rivers/Spring/river(10).png`,
  river_2_e_s: `${ASSET_BASE}/Rivers/Spring/river(11).png`,
  river_1_e: `${ASSET_BASE}/Rivers/Spring/river(12).png`,
  river_1_s: `${ASSET_BASE}/Rivers/Spring/river(13).png`,
  river_1_n: `${ASSET_BASE}/Rivers/Spring/river(14).png`,
  river_1_w: `${ASSET_BASE}/Rivers/Spring/river(15).png`,
  river_road_2_n_s: `${ASSET_BASE}/Rivers/Spring/river(16).png`,
  river_road_2_e_w: `${ASSET_BASE}/Rivers/Spring/river(17).png`,

  // Buildings - Houses (White/Blue)
  white_blue_house1: `${ASSET_BASE}/Buildings/house/White houses/Blue house/white_blue_house(1).png`,
  white_blue_house2: `${ASSET_BASE}/Buildings/house/White houses/Blue house/white_blue_house(2).png`,
  white_blue_house3: `${ASSET_BASE}/Buildings/house/White houses/Blue house/white_blue_house(3).png`,
  white_blue_house4: `${ASSET_BASE}/Buildings/house/White houses/Blue house/white_blue_house(4).png`,

  // Buildings - Houses (White/Red)
  white_red_house1: `${ASSET_BASE}/Buildings/house/White houses/Red house/white_red_house(1).png`,
  white_red_house2: `${ASSET_BASE}/Buildings/house/White houses/Red house/white_red_house(2).png`,
  white_red_house3: `${ASSET_BASE}/Buildings/house/White houses/Red house/white_red_house(3).png`,
  white_red_house4: `${ASSET_BASE}/Buildings/house/White houses/Red house/white_red_house(4).png`,

  // Buildings - Houses (White/Green)
  white_green_house1: `${ASSET_BASE}/Buildings/house/White houses/Green house/white_green_house(1).png`,
  white_green_house2: `${ASSET_BASE}/Buildings/house/White houses/Green house/white_green_house(2).png`,
  white_green_house3: `${ASSET_BASE}/Buildings/house/White houses/Green house/white_green_house(3).png`,
  white_green_house4: `${ASSET_BASE}/Buildings/house/White houses/Green house/white_green_house(4).png`,

  // Buildings - Houses (Wood/Blue)
  wood_blue_house1: `${ASSET_BASE}/Buildings/house/Wood houses/Blue house/wood_blue_house(1).png`,
  wood_blue_house2: `${ASSET_BASE}/Buildings/house/Wood houses/Blue house/wood_blue_house(2).png`,
  wood_blue_house3: `${ASSET_BASE}/Buildings/house/Wood houses/Blue house/wood_blue_house(3).png`,
  wood_blue_house4: `${ASSET_BASE}/Buildings/house/Wood houses/Blue house/wood_blue_house(4).png`,

  // Buildings - Blacksmith
  workshop_blue1: `${ASSET_BASE}/Buildings/house/blacksmith_blue(1).png`,
  workshop_blue2: `${ASSET_BASE}/Buildings/house/blacksmith_blue(2).png`,

  // Farm - Corn
  corn1: `${ASSET_BASE}/Buildings/corn/corn(1).png`,
  corn2: `${ASSET_BASE}/Buildings/corn/corn(2).png`,

  // Farm - Wheat
  wheat1: `${ASSET_BASE}/Buildings/wheat/wheat(1).png`,
  wheat2: `${ASSET_BASE}/Buildings/wheat/wheat(2).png`,

  // Farm - Mill detail (detail layer)
  mill_detail_1: `${ASSET_BASE}/Buildings/mill/mill(1).png`,
  mill_detail_2: `${ASSET_BASE}/Buildings/mill/mill(2).png`,
  mill_detail_3: `${ASSET_BASE}/Buildings/mill/mill(3).png`,
  mill_detail_4: `${ASSET_BASE}/Buildings/mill/mill(4).png`,
  mill_blue: `${ASSET_BASE}/Buildings/mill/mill_blue.png`,
  mill_green: `${ASSET_BASE}/Buildings/mill/mill_green.png`,
  mill_red: `${ASSET_BASE}/Buildings/mill/mill_red.png`,
  mill_wood: `${ASSET_BASE}/Buildings/mill/mill_wood.png`,

  // Castle walls
  castle_wall1: `${ASSET_BASE}/Castle walls/castle_wall(1).png`,
  castle_wall2: `${ASSET_BASE}/Castle walls/castle_wall(2).png`,
  castle_wall3: `${ASSET_BASE}/Castle walls/castle_wall(3).png`,
  castle_wall4: `${ASSET_BASE}/Castle walls/castle_wall(4).png`,
  castle_wall5: `${ASSET_BASE}/Castle walls/castle_wall(5).png`,
  castle_wall6: `${ASSET_BASE}/Castle walls/castle_wall(6).png`,
  castle_wall7: `${ASSET_BASE}/Castle walls/castle_wall(7).png`,
  castle_wall8: `${ASSET_BASE}/Castle walls/castle_wall(8).png`,
  castle_wall9: `${ASSET_BASE}/Castle walls/castle_wall(9).png`,
  castle_wall10: `${ASSET_BASE}/Castle walls/castle_wall(10).png`,
};

/** Grouped tile keys for the editor palette. */
export const TILE_CATEGORIES = {
  "Ground": ["grass", "ground1", "ground2"],
  "Grass Detail": ["grass_detail1", "grass_detail2", "grass_detail3", "grass_detail4", "grass_detail5", "grass_detail6", "grass_detail7", "grass_detail8", "grass_detail9"],
  "Trees": ["tree1", "tree2", "tree3", "tree4"],
  "Stones": ["stone1", "stone2", "stone3", "stone4"],
  "Roads": ["road_4_n_e_s_w", "road_2_e_w", "road_2_n_s", "road_3_e_s_w", "road_3_n_e_s", "road_3_n_e_w", "road_3_n_s_w", "road_2_s_w", "road_2_n_e", "road_2_n_w", "road_2_e_s", "road_1_e", "road_1_s", "road_1_n", "road_1_w"],
  "Rivers": ["river_4_n_e_s_w", "river_2_n_s", "river_2_e_w", "river_3_e_s_w", "river_3_n_e_s", "river_3_n_e_w", "river_3_n_s_w", "river_2_s_w", "river_2_n_e", "river_2_n_w", "river_2_e_s", "river_1_e", "river_1_s", "river_1_n", "river_1_w", "river_road_2_n_s", "river_road_2_e_w"],
  "Houses": ["white_blue_house1", "white_blue_house2", "white_blue_house3", "white_blue_house4", "white_red_house1", "white_red_house2", "white_red_house3", "white_red_house4", "white_green_house1", "white_green_house2", "white_green_house3", "white_green_house4", "wood_blue_house1", "wood_blue_house2", "wood_blue_house3", "wood_blue_house4"],
  "Workshops": ["workshop_blue1", "workshop_blue2"],
  "Farm": ["corn1", "corn2", "wheat1", "wheat2", "mill_detail_1", "mill_detail_2", "mill_detail_3", "mill_detail_4", "mill_blue", "mill_green", "mill_red", "mill_wood"],
  "Castle": ["castle_wall1", "castle_wall2", "castle_wall3", "castle_wall4", "castle_wall5", "castle_wall6", "castle_wall7", "castle_wall8", "castle_wall9", "castle_wall10"],
};

// ── Three-layer tile classification ──────────────────────────
// Ground: flat terrain tiles rendered first
const GROUND_TILES = new Set([
  "grass", "ground1", "ground2",
  ...Object.keys(MANIFEST).filter(k => k.startsWith("road_")),
  ...Object.keys(MANIFEST).filter(k => k.startsWith("river_")),
  ...Object.keys(MANIFEST).filter(k => k.startsWith("corn")),
  ...Object.keys(MANIFEST).filter(k => k.startsWith("wheat")),
]);

// Detail: decorative overlays rendered last (on top of base)
const DETAIL_TILES = new Set([
  ...Object.keys(MANIFEST).filter(k => k.startsWith("grass_detail")),
  ...Object.keys(MANIFEST).filter(k => k.startsWith("mill_detail")),
]);

// Base: everything else (trees, stones, houses, workshops, castle walls)
// If not ground and not detail, it's base.

export function tileLayer(key) {
  if (GROUND_TILES.has(key)) return "ground";
  if (DETAIL_TILES.has(key)) return "detail";
  return "base";
}

export function isGroundTile(key) {
  return GROUND_TILES.has(key);
}

/** Per-tile base Y offset for base-layer object rendering. */
export function baseOffsetY(key) {
  if (key.startsWith("castle_wall")) return 30;
  if (key.startsWith("white_") || key.startsWith("wood_") || key.startsWith("workshop_")) return 20;
  return 15;
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
