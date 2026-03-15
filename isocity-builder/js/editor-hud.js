import { TILE_CATEGORIES, getSprite } from "./assets.js";
import { getEditorState } from "./editor.js";

/**
 * Editor HUD: tile palette panel + toolbar drawn on canvas.
 *
 * Layout:
 *   Left panel (240px wide): scrollable tile palette grouped by category
 *   Top bar: mode label, map size, export/import buttons
 */

const PANEL_W = 260;
const TILE_THUMB = 48;      // thumbnail size for palette tiles
const TILE_PAD = 6;
const COLS = 4;              // tiles per row in palette
const HEADER_H = 24;         // category header height
const TOP_BAR_H = 40;
const BTN_H = 28;
const BTN_PAD = 8;

// Scroll state for the palette
let scrollY = 0;
let maxScrollY = 0;

// Precomputed layout (built once after assets load)
let layoutItems = [];   // { type: "header"|"tile", y, x, w, h, label?, key?, cat? }
let layoutDirty = true;

function buildLayout() {
  layoutItems = [];
  let y = TOP_BAR_H + 8;

  for (const [cat, keys] of Object.entries(TILE_CATEGORIES)) {
    // Category header
    layoutItems.push({ type: "header", y, x: 8, w: PANEL_W - 16, h: HEADER_H, label: cat });
    y += HEADER_H + 4;

    // Tile grid
    for (let i = 0; i < keys.length; i++) {
      const col = i % COLS;
      const row = Math.floor(i / COLS);
      const tx = 8 + col * (TILE_THUMB + TILE_PAD);
      const ty = y + row * (TILE_THUMB + TILE_PAD);
      layoutItems.push({ type: "tile", y: ty, x: tx, w: TILE_THUMB, h: TILE_THUMB, key: keys[i], cat });
    }
    const rows = Math.ceil(keys.length / COLS);
    y += rows * (TILE_THUMB + TILE_PAD) + 8;
  }

  maxScrollY = Math.max(0, y - 600);
  layoutDirty = false;
}

// ── Buttons in top bar ───────────────────────────────────────
const buttons = [];

function buildButtons(canvasW) {
  buttons.length = 0;
  // Top bar buttons (right-aligned)
  let bx = PANEL_W + 8;
  buttons.push({ id: "export", x: bx, y: 6, w: 80, h: BTN_H, label: "Export" });
  bx += 88;
  buttons.push({ id: "import", x: bx, y: 6, w: 80, h: BTN_H, label: "Import" });
  bx += 88;
  buttons.push({ id: "clear", x: bx, y: 6, w: 70, h: BTN_H, label: "Clear" });
  bx += 78;
  buttons.push({ id: "size-", x: bx, y: 6, w: 30, h: BTN_H, label: "-" });
  bx += 34;
  buttons.push({ id: "size_label", x: bx, y: 6, w: 60, h: BTN_H, label: "" }); // drawn dynamically
  bx += 64;
  buttons.push({ id: "size+", x: bx, y: 6, w: 30, h: BTN_H, label: "+" });
}

// ── Rendering ────────────────────────────────────────────────

export function renderEditorHUD(ctx, canvasW, canvasH) {
  if (layoutDirty) buildLayout();
  buildButtons(canvasW);

  const es = getEditorState();

  ctx.save();
  ctx.setTransform(1, 0, 0, 1, 0, 0);

  // ── Top bar ────────────────────────────────────────────────
  ctx.fillStyle = "rgba(0,0,0,0.75)";
  ctx.fillRect(0, 0, canvasW, TOP_BAR_H);

  ctx.font = "bold 14px monospace";
  ctx.textBaseline = "middle";
  ctx.textAlign = "left";
  ctx.fillStyle = "#0f0";
  ctx.fillText("MAP EDITOR", 10, TOP_BAR_H / 2);

  // Buttons
  for (const btn of buttons) {
    if (btn.id === "size_label") {
      // Dynamic size label
      ctx.fillStyle = "rgba(255,255,255,0.15)";
      ctx.fillRect(btn.x, btn.y, btn.w, btn.h);
      ctx.fillStyle = "#fff";
      ctx.font = "bold 13px monospace";
      ctx.textAlign = "center";
      ctx.fillText(`${es.mapSize}x${es.mapSize}`, btn.x + btn.w / 2, btn.y + btn.h / 2);
      continue;
    }
    // Button bg
    ctx.fillStyle = "rgba(255,255,255,0.15)";
    ctx.fillRect(btn.x, btn.y, btn.w, btn.h);
    ctx.strokeStyle = "rgba(255,255,255,0.3)";
    ctx.strokeRect(btn.x, btn.y, btn.w, btn.h);
    ctx.fillStyle = "#fff";
    ctx.font = "bold 12px monospace";
    ctx.textAlign = "center";
    ctx.fillText(btn.label, btn.x + btn.w / 2, btn.y + btn.h / 2);
  }

  // ── Left palette panel ────────────────────────────────────
  ctx.fillStyle = "rgba(0,0,0,0.8)";
  ctx.fillRect(0, TOP_BAR_H, PANEL_W, canvasH - TOP_BAR_H);

  // Clip to panel area
  ctx.save();
  ctx.beginPath();
  ctx.rect(0, TOP_BAR_H, PANEL_W, canvasH - TOP_BAR_H);
  ctx.clip();

  for (const item of layoutItems) {
    const drawY = item.y - scrollY;

    // Skip items outside visible area
    if (drawY + item.h < TOP_BAR_H || drawY > canvasH) continue;

    if (item.type === "header") {
      ctx.fillStyle = "rgba(255,255,255,0.1)";
      ctx.fillRect(item.x, drawY, item.w, item.h);
      ctx.fillStyle = "#aaa";
      ctx.font = "bold 11px monospace";
      ctx.textAlign = "left";
      ctx.textBaseline = "middle";
      ctx.fillText(item.label, item.x + 6, drawY + item.h / 2);
    } else if (item.type === "tile") {
      const selected = es.selectedTile === item.key;

      // Tile background
      ctx.fillStyle = selected ? "rgba(0,255,0,0.3)" : "rgba(255,255,255,0.08)";
      ctx.fillRect(item.x, drawY, item.w, item.h);

      if (selected) {
        ctx.strokeStyle = "#0f0";
        ctx.lineWidth = 2;
        ctx.strokeRect(item.x, drawY, item.w, item.h);
        ctx.lineWidth = 1;
      }

      // Draw sprite thumbnail
      const img = getSprite(item.key);
      if (img) {
        const scale = Math.min((item.w - 4) / img.width, (item.h - 4) / img.height);
        const sw = img.width * scale;
        const sh = img.height * scale;
        ctx.drawImage(
          img,
          item.x + (item.w - sw) / 2,
          drawY + (item.h - sh) / 2,
          sw, sh,
        );
      }
    }
  }

  ctx.restore(); // unclip

  // ── Selected tile name (bottom of palette) ────────────────
  ctx.fillStyle = "rgba(0,0,0,0.8)";
  ctx.fillRect(0, canvasH - 28, PANEL_W, 28);
  ctx.fillStyle = "#0f0";
  ctx.font = "bold 11px monospace";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(es.selectedTile || "none", PANEL_W / 2, canvasH - 14);

  // ── Hint text ─────────────────────────────────────────────
  const hint = "Left-click: paint  |  Right-click: erase  |  [Tab] toggle editor";
  ctx.fillStyle = "rgba(0,0,0,0.6)";
  const hw = ctx.measureText(hint).width + 32;
  ctx.fillRect(Math.round(canvasW / 2 - hw / 2), canvasH - 36, hw, 24);
  ctx.fillStyle = "#ff0";
  ctx.font = "bold 12px monospace";
  ctx.textAlign = "center";
  ctx.fillText(hint, canvasW / 2, canvasH - 24);

  ctx.restore();
}

// ── Hit testing ──────────────────────────────────────────────

/**
 * Check if a screen click hits a palette tile or button.
 * Returns { type: "tile", key } | { type: "button", id } | null
 */
export function editorHitTest(sx, sy) {
  // Check buttons first
  for (const btn of buttons) {
    if (btn.id === "size_label") continue;
    if (sx >= btn.x && sx <= btn.x + btn.w && sy >= btn.y && sy <= btn.y + btn.h) {
      return { type: "button", id: btn.id };
    }
  }

  // Check palette tiles
  if (sx < PANEL_W && sy > TOP_BAR_H) {
    for (const item of layoutItems) {
      if (item.type !== "tile") continue;
      const drawY = item.y - scrollY;
      if (sx >= item.x && sx <= item.x + item.w && sy >= drawY && sy <= drawY + item.h) {
        return { type: "tile", key: item.key };
      }
    }
  }

  return null;
}

/** Is the given screen point inside the editor panel (not the map)? */
export function isInEditorPanel(sx, sy) {
  return sx < PANEL_W || sy < TOP_BAR_H;
}

/** Scroll the palette. */
export function scrollPalette(deltaY) {
  scrollY = Math.max(0, Math.min(maxScrollY, scrollY + deltaY));
}

export function getPanelWidth() {
  return PANEL_W;
}
