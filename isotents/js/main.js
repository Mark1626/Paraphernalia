import { loadAssets } from "./assets.js";
import { Camera } from "./camera.js";
import {
  renderGroundMap,
  renderTrees,
  renderHouses,
  renderMarkers,
  renderViolations,
  renderClues,
  highlightTile,
} from "./renderer.js";
import { toGrid, toScreen, TILE_W, TILE_H } from "./iso.js";
import { LEVELS } from "./generator.js";
import {
  newPuzzle,
  reset,
  getState,
  validate,
  toggleHouse,
  mark,
  markLine,
  HOUSE,
  MARKED,
} from "./puzzle.js";
import { renderHUD } from "./hud.js";

// ── Canvas setup ──────────────────────────────────────────────
const canvas = document.getElementById("game");
const ctx = canvas.getContext("2d");

function resize() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
}
window.addEventListener("resize", resize);
resize();

// ── Camera (editorMode = only middle-mouse pans, freeing L/R for play) ──
const camera = new Camera(canvas);
camera.editorMode = true;

// ── Puzzle state ──────────────────────────────────────────────
let groundGrid = [];     // N×N grass tiles for the ground pass
let treeCount = 0;       // total houses required (== number of trees)

const config = { difficulty: "Medium", reveal: false };

function levelByName(name) {
  return LEVELS.find((l) => l.name === name) ?? LEVELS[1];
}

function startPuzzle(name) {
  const lvl = levelByName(name);
  const st = newPuzzle(lvl.size, lvl.density);
  const N = lvl.size;
  groundGrid = Array.from({ length: N }, () => new Array(N).fill("grass"));
  treeCount = 0;
  for (let r = 0; r < N; r++) {
    for (let c = 0; c < N; c++) if (st.board.trees[r][c]) treeCount++;
  }
  fitCamera(N);
}

/** Centre and zoom the camera so the whole board (plus clue edge) fits. */
function fitCamera(N) {
  let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
  for (let r = -1; r < N; r++) {
    for (let c = -1; c < N; c++) {
      const { x, y } = toScreen(c, r);
      minX = Math.min(minX, x); maxX = Math.max(maxX, x);
      minY = Math.min(minY, y); maxY = Math.max(maxY, y);
    }
  }
  const padX = TILE_W, padTop = 130, padBot = TILE_H * 2; // headroom for tall trees
  const bboxW = maxX - minX + padX;
  const bboxH = maxY - minY + padTop + padBot;
  const zoom = Math.max(
    0.25,
    Math.min(1.3, canvas.width * 0.9 / bboxW, canvas.height * 0.85 / bboxH),
  );
  const cx = (minX + maxX) / 2;
  const cy = (minY + maxY) / 2;
  camera.zoom = zoom;
  camera.x = canvas.width / 2 - cx * zoom;
  camera.y = canvas.height / 2 - cy * zoom;
}

// ── Input ─────────────────────────────────────────────────────
let hoverTile = null;
let rightDragging = false;
let rightStart = null;     // { row, col }
let rightMoved = false;

function tileAt(clientX, clientY) {
  const world = camera.screenToWorld(clientX, clientY);
  const g = toGrid(world.x, world.y);
  const N = getState().board.size;
  if (g.row >= 0 && g.row < N && g.col >= 0 && g.col < N) return g;
  return null;
}

canvas.addEventListener("mousemove", (e) => {
  hoverTile = tileAt(e.clientX, e.clientY);
  if (rightDragging && rightStart && hoverTile) {
    if (hoverTile.row !== rightStart.row || hoverTile.col !== rightStart.col) {
      rightMoved = true;
    }
    if (rightMoved) {
      markLine(rightStart.row, rightStart.col, hoverTile.row, hoverTile.col);
    }
  }
});

canvas.addEventListener("mousedown", (e) => {
  if (e.button === 0) {
    const t = tileAt(e.clientX, e.clientY);
    if (t) toggleHouse(t.row, t.col);
  } else if (e.button === 2) {
    const t = tileAt(e.clientX, e.clientY);
    if (t) {
      rightDragging = true;
      rightStart = t;
      rightMoved = false;
    }
  }
});

canvas.addEventListener("mouseup", (e) => {
  if (e.button === 2 && rightDragging) {
    if (!rightMoved && rightStart) mark(rightStart.row, rightStart.col);
    rightDragging = false;
    rightStart = null;
    rightMoved = false;
  }
});

canvas.addEventListener("mouseleave", () => {
  rightDragging = false;
  rightStart = null;
});

// Right-click is gameplay — suppress the context menu.
canvas.addEventListener("contextmenu", (e) => e.preventDefault());

// Wheel = zoom toward the cursor.
canvas.addEventListener("wheel", (e) => {
  e.preventDefault();
  const factor = e.deltaY < 0 ? 1.1 : 0.9;
  const newZoom = Math.min(camera.maxZoom, Math.max(camera.minZoom, camera.zoom * factor));
  const wx = (e.clientX - camera.x) / camera.zoom;
  const wy = (e.clientY - camera.y) / camera.zoom;
  camera.zoom = newZoom;
  camera.x = e.clientX - wx * camera.zoom;
  camera.y = e.clientY - wy * camera.zoom;
}, { passive: false });

window.addEventListener("keydown", (e) => {
  if (isMenuOpen() || !getState()) return; // hotkeys are in-game only
  if (e.key === "n" || e.key === "N") startPuzzle(config.difficulty);
  else if (e.key === "r" || e.key === "R") reset();
});

// ── Menu + background music ──────────────────────────────────
const menuEl = document.getElementById("menu");
const topbarEl = document.getElementById("topbar");
const levelListEl = document.getElementById("levelList");
const revealToggle = document.getElementById("revealToggle");
const bgm = document.getElementById("bgm");

const isMenuOpen = () => !menuEl.classList.contains("hidden");

// Build a button per difficulty level.
for (const lvl of LEVELS) {
  const btn = document.createElement("button");
  btn.type = "button";
  btn.innerHTML = `<span>${lvl.name}</span><span class="size">${lvl.size}×${lvl.size}</span>`;
  btn.addEventListener("click", () => startGame(lvl.name));
  levelListEl.appendChild(btn);
}

revealToggle.addEventListener("change", () => {
  config.reveal = revealToggle.checked;
});

function startGame(name) {
  config.difficulty = name;
  startPuzzle(name);
  menuEl.classList.add("hidden");
  topbarEl.classList.remove("hidden");
}

function showMenu() {
  // Offer "Resume" only when there's a puzzle in progress to return to.
  resumeBtn.classList.toggle("hidden", !getState());
  menuEl.classList.remove("hidden");
  topbarEl.classList.add("hidden");
}

function resumeGame() {
  if (!getState()) return;
  menuEl.classList.add("hidden");
  topbarEl.classList.remove("hidden");
}

const resumeBtn = document.getElementById("resumeBtn");
resumeBtn.addEventListener("click", resumeGame);
document.getElementById("menuBtn").addEventListener("click", showMenu);
// Click the backdrop (outside the card) to resume an in-progress puzzle.
menuEl.addEventListener("click", (e) => {
  if (e.target === menuEl) resumeGame();
});

bgm.volume = 0.4;
function playMusic() {
  bgm.play().catch(() => {});
}

// Start music as soon as the page loads. Browsers usually block audio until
// the first user gesture, so retry on the first interaction as a fallback.
playMusic();
const startMusicOnce = () => {
  playMusic();
  window.removeEventListener("pointerdown", startMusicOnce);
  window.removeEventListener("keydown", startMusicOnce);
};
window.addEventListener("pointerdown", startMusicOnce);
window.addEventListener("keydown", startMusicOnce);

const muteBtn = document.getElementById("muteBtn");
muteBtn.addEventListener("click", () => {
  bgm.muted = !bgm.muted;
  muteBtn.textContent = bgm.muted ? "🔇" : "♪";
});

// ── Click sound effect ───────────────────────────────────────
// Plays on every left/right click anywhere (canvas placements, menu buttons).
// Clone the node each time so rapid clicks can overlap.
const sfx = document.getElementById("sfx");
function playClick() {
  if (bgm.muted) return; // the mute button silences SFX too
  const s = sfx.cloneNode(true);
  s.volume = 0.6;
  s.play().catch(() => {});
}
document.addEventListener("mousedown", (e) => {
  if (e.button === 0 || e.button === 2) playClick();
});

// ── Game loop ─────────────────────────────────────────────────
function frame() {
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  const st = getState();
  if (!st) {
    // No puzzle yet (menu screen) — nothing to draw on the canvas.
    requestAnimationFrame(frame);
    return;
  }

  camera.apply(ctx);
  const v = validate();
  const { board, placement, treeSprites, houseSprites } = st;

  renderGroundMap(ctx, groundGrid);
  renderTrees(ctx, board, treeSprites);
  renderHouses(ctx, placement, houseSprites, HOUSE);
  renderViolations(ctx, v.adjacent);
  renderMarkers(ctx, placement, MARKED);

  if (config.reveal) {
    for (let r = 0; r < board.size; r++) {
      for (let c = 0; c < board.size; c++) {
        if (board.solution[r][c]) highlightTile(ctx, c, r, "rgba(80,220,80,0.30)");
      }
    }
  }

  if (hoverTile && !v.won) {
    highlightTile(ctx, hoverTile.col, hoverTile.row, "rgba(255,255,255,0.22)");
  }

  renderClues(ctx, board, v);

  renderHUD(ctx, canvas.width, canvas.height, {
    levelName: config.difficulty,
    houseCount: v.houseCount,
    treeCount,
    won: v.won,
  });

  requestAnimationFrame(frame);
}

// ── Boot ──────────────────────────────────────────────────────
loadAssets()
  .then(() => {
    // Start on the menu; a puzzle begins when the player picks a difficulty.
    showMenu();
    requestAnimationFrame(frame);
  })
  .catch((err) => {
    console.error(err);
    document.body.style.color = "white";
    document.body.innerText = `Failed to load assets: ${err.message}`;
  });
