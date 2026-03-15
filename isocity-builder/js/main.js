import { loadAssets } from "./assets.js";
import { Camera } from "./camera.js";
import {
  renderGroundMap,
  renderObjectMap,
  renderDetailMap,
  highlightTile,
  renderCompass,
} from "./renderer.js";
import { toGrid } from "./iso.js";
import { createDemoMap } from "./world.js";
import {
  canPlace,
  placeBuilding,
  endTurn,
  getState,
  currentPlaceable,
} from "./game.js";
import { renderHUD } from "./hud.js";
import {
  getEditorState,
  setEditorActive,
  setSelectedTile,
  setMapSize,
  paintTile,
  eraseTile,
  exportMap,
  importMap,
  createBlankMap,
} from "./editor.js";
import {
  renderEditorHUD,
  editorHitTest,
  isInEditorPanel,
  scrollPalette,
  getPanelWidth,
} from "./editor-hud.js";

// ── Canvas setup ──────────────────────────────────────────────
const canvas = document.getElementById("game");
const ctx = canvas.getContext("2d");

function resize() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
}
window.addEventListener("resize", resize);
resize();

// ── Camera ────────────────────────────────────────────────────
const camera = new Camera(canvas);
camera.x = canvas.width / 2;
camera.y = 100;

// ── dat.gui controls ─────────────────────────────────────────
const config = {
  groundDebug: false,
  objectDebug: false,
  objectOffsetX: 0,
  objectOffsetY: 0,
};
const gui = new dat.GUI({ width: 280 });

const renderFolder = gui.addFolder("Rendering");
renderFolder.add(config, "groundDebug").name("Ground Debug");
renderFolder.add(config, "objectDebug").name("Object Debug");
renderFolder.add(config, "objectOffsetX", -100, 100, 1).name("Object Offset X");
renderFolder.add(config, "objectOffsetY", -100, 100, 1).name("Object Offset Y");

const gameState = getState();
const stateFolder = gui.addFolder("Game State");
stateFolder.add(gameState, "turn", 1, 100, 1).name("Turn");
stateFolder.add(gameState, "ap", 0, 10, 1).name("AP");
stateFolder
  .add(gameState, "phase", ["place_house", "play", "turn_over"])
  .name("Phase")
  .listen();

const resFolder = stateFolder.addFolder("Resources");
resFolder.add(gameState.resources, "food", 0, 999, 1).name("Food");
resFolder.add(gameState.resources, "wood", 0, 999, 1).name("Wood");
resFolder.add(gameState.resources, "water", 0, 999, 1).name("Water");
resFolder.add(gameState.resources, "stone", 0, 999, 1).name("Stone");
resFolder.open();
stateFolder.open();

// ── Map state ────────────────────────────────────────────────
let MAP_SIZE = 10;
let { ground, objects, detail } = createDemoMap(MAP_SIZE);

// ── Helper to replace the map ────────────────────────────────
function replaceMap(newGround, newObjects, newDetail, newSize) {
  ground = newGround;
  objects = newObjects;
  detail = newDetail;
  MAP_SIZE = newSize;
  setMapSize(newSize);
}

// ── Mouse state ──────────────────────────────────────────────
let hoverTile = null;
let mouseDown = false;        // for editor paint-drag
let mouseButton = 0;          // 0=left, 2=right
let dragStartPos = null;      // to distinguish drag from click

canvas.addEventListener("mousemove", (e) => {
  const world = camera.screenToWorld(e.clientX, e.clientY);
  const grid = toGrid(world.x, world.y);
  if (
    grid.row >= 0 &&
    grid.row < MAP_SIZE &&
    grid.col >= 0 &&
    grid.col < MAP_SIZE
  ) {
    hoverTile = grid;
  } else {
    hoverTile = null;
  }

  // Editor: paint while dragging
  const es = getEditorState();
  if (es.active && mouseDown && hoverTile && !isInEditorPanel(e.clientX, e.clientY)) {
    if (mouseButton === 0) {
      paintTile(ground, objects, detail, hoverTile.row, hoverTile.col);
    } else if (mouseButton === 2) {
      eraseTile(ground, objects, detail, hoverTile.row, hoverTile.col);
    }
  }
});

canvas.addEventListener("mousedown", (e) => {
  mouseDown = true;
  mouseButton = e.button;
  dragStartPos = { x: e.clientX, y: e.clientY };
});

canvas.addEventListener("mouseup", (e) => {
  const es = getEditorState();
  const wasDrag = dragStartPos &&
    (Math.abs(e.clientX - dragStartPos.x) > 3 || Math.abs(e.clientY - dragStartPos.y) > 3);

  if (es.active && !wasDrag) {
    // Check if click is on editor UI
    const hit = editorHitTest(e.clientX, e.clientY);
    if (hit) {
      handleEditorHit(hit);
    } else if (hoverTile && !isInEditorPanel(e.clientX, e.clientY)) {
      // Paint/erase on map
      if (e.button === 0) {
        paintTile(ground, objects, detail, hoverTile.row, hoverTile.col);
      } else if (e.button === 2) {
        eraseTile(ground, objects, detail, hoverTile.row, hoverTile.col);
      }
    }
  } else if (!es.active && !wasDrag) {
    // Game mode click
    if (hoverTile && e.button === 0) {
      const { row, col } = hoverTile;
      if (canPlace(ground, objects, row, col)) {
        placeBuilding(objects, row, col);
      }
    }
  }

  mouseDown = false;
  dragStartPos = null;
});

// Prevent context menu in editor mode
canvas.addEventListener("contextmenu", (e) => {
  if (getEditorState().active) e.preventDefault();
});

// Editor palette scroll
canvas.addEventListener("wheel", (e) => {
  if (getEditorState().active && e.clientX < getPanelWidth()) {
    e.preventDefault();
    scrollPalette(e.deltaY);
  }
}, { passive: false });

// ── Editor button handlers ───────────────────────────────────
function handleEditorHit(hit) {
  if (hit.type === "tile") {
    setSelectedTile(hit.key);
    return;
  }
  if (hit.type === "button") {
    switch (hit.id) {
      case "export": {
        const json = exportMap(ground, objects, detail);
        const blob = new Blob([json], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `isocity-map-${MAP_SIZE}x${MAP_SIZE}.json`;
        a.click();
        URL.revokeObjectURL(url);
        break;
      }
      case "import": {
        const input = document.createElement("input");
        input.type = "file";
        input.accept = ".json";
        input.onchange = (e) => {
          const file = e.target.files[0];
          if (!file) return;
          const reader = new FileReader();
          reader.onload = (ev) => {
            try {
              const { ground: g, objects: o, detail: d, size } = importMap(ev.target.result);
              replaceMap(g, o, d, size);
            } catch (err) {
              console.error("Import failed:", err);
              alert("Failed to import map: " + err.message);
            }
          };
          reader.readAsText(file);
        };
        input.click();
        break;
      }
      case "clear": {
        const { ground: g, objects: o, detail: d } = createBlankMap(MAP_SIZE);
        replaceMap(g, o, d, MAP_SIZE);
        break;
      }
      case "size+": {
        const newSize = Math.min(30, MAP_SIZE + 1);
        if (newSize !== MAP_SIZE) resizeMap(newSize);
        break;
      }
      case "size-": {
        const newSize = Math.max(3, MAP_SIZE - 1);
        if (newSize !== MAP_SIZE) resizeMap(newSize);
        break;
      }
    }
  }
}

/** Resize the map, preserving existing tiles where possible. */
function resizeMap(newSize) {
  const newGround = [];
  const newObjects = [];
  const newDetail = [];
  for (let row = 0; row < newSize; row++) {
    newGround[row] = [];
    newObjects[row] = [];
    newDetail[row] = [];
    for (let col = 0; col < newSize; col++) {
      const exists = row < MAP_SIZE && col < MAP_SIZE;
      newGround[row][col] = exists ? ground[row][col] : "grass";
      newObjects[row][col] = exists ? objects[row][col] : null;
      newDetail[row][col] = exists ? detail[row][col] : null;
    }
  }
  replaceMap(newGround, newObjects, newDetail, newSize);
}

// ── Keyboard ─────────────────────────────────────────────────
window.addEventListener("keydown", (e) => {
  // Tab toggles editor mode
  if (e.key === "Tab") {
    e.preventDefault();
    const es = getEditorState();
    setEditorActive(!es.active);
    camera.editorMode = !es.active;
    if (es.active) {
      gui.show();
    } else {
      gui.hide();
    }
    return;
  }

  // Game-mode keys
  if (!getEditorState().active) {
    if (e.key === "Enter") {
      const s = getState();
      if (s.phase === "play" || s.phase === "turn_over") {
        endTurn();
      }
    }
  }
});

// ── Game loop ─────────────────────────────────────────────────
function frame() {
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  camera.apply(ctx);
  renderGroundMap(ctx, ground, config.groundDebug);
  renderObjectMap(
    ctx,
    objects,
    config.objectDebug,
    config.objectOffsetX,
    config.objectOffsetY,
  );
  renderDetailMap(ctx, detail, config.objectDebug, config.objectOffsetX, config.objectOffsetY);
  renderCompass(ctx);

  const es = getEditorState();

  if (hoverTile) {
    if (es.active) {
      highlightTile(
        ctx,
        hoverTile.col,
        hoverTile.row,
        "rgba(0,200,255,0.35)",
      );
    } else {
      const placeable = currentPlaceable();
      const valid =
        placeable && canPlace(ground, objects, hoverTile.row, hoverTile.col);
      highlightTile(
        ctx,
        hoverTile.col,
        hoverTile.row,
        valid ? "rgba(0,255,0,0.3)" : "rgba(255,255,255,0.3)",
      );
    }
  }

  // HUD
  if (es.active) {
    renderEditorHUD(ctx, canvas.width, canvas.height);
  } else {
    renderHUD(ctx, canvas.width, canvas.height);
  }

  requestAnimationFrame(frame);
}

// ── Boot ──────────────────────────────────────────────────────
loadAssets()
  .then(() => {
    console.log("Assets loaded — starting game loop");
    requestAnimationFrame(frame);
  })
  .catch((err) => {
    console.error(err);
    document.body.style.color = "white";
    document.body.innerText = `Failed to load assets: ${err.message}`;
  });
