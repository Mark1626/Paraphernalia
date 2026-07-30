// All canvas drawing lives here (geometric shapes only for now). Swapping in
// sprites later means touching just this file.

import { ARENA, TOWER, PROJECTILE } from './config.js';
import { SHEETS, SPRITES, TILES, imgReady, slimeSpriteFor } from './sprites.js';

const TAU = Math.PI * 2;

// Fixed decorative tiles scattered on the grass outside the arena ring
// (canvas corners / edge midpoints). [tile key, dest x, dest y] at 2x scale.
const GRASS_DECOR = [
  ['donut', 36, 36],
  ['patch', 556, 40],
  ['patch', 40, 550],
  ['donut', 556, 546],
  ['stripH', 278, 14],
  ['stripH', 278, 604],
  ['stripV', 16, 278],
  ['stripV', 604, 278],
];

export function render(ctx, canvas, state) {
  const { tower, enemies, projectiles } = state;
  ctx.imageSmoothingEnabled = false; // crisp pixel-art scaling
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  drawArena(ctx, tower);
  for (const p of projectiles) drawProjectile(ctx, p);
  for (const e of enemies) drawEnemy(ctx, e);
  drawTower(ctx, tower);
}

function drawArena(ctx, tower) {
  ctx.save();
  const tiles = SHEETS.tiles;
  const { width, height } = ctx.canvas;
  if (imgReady(tiles)) {
    // Grass-tile the whole canvas (2x keeps the chunky pixel look), sprinkle the
    // fixed decorations, then lay the big dirt circle down as the arena floor.
    const g = TILES.grass;
    for (let y = 0; y < height; y += g.sh * 2) {
      for (let x = 0; x < width; x += g.sw * 2) {
        ctx.drawImage(tiles, g.sx, g.sy, g.sw, g.sh, x, y, g.sw * 2, g.sh * 2);
      }
    }
    for (const [key, dx, dy] of GRASS_DECOR) {
      const d = TILES[key];
      ctx.drawImage(tiles, d.sx, d.sy, d.sw, d.sh, dx, dy, d.sw * 2, d.sh * 2);
    }
    // Slightly larger than the arena so the dirt's organic fringe pokes past the ring.
    const size = (ARENA.radius + 10) * 2;
    const a = TILES.arenaCircle;
    ctx.drawImage(tiles, a.sx, a.sy, a.sw, a.sh, tower.x - size / 2, tower.y - size / 2, size, size);
  } else {
    // Geometric fallback (before the sheet decodes / in headless).
    ctx.beginPath();
    ctx.arc(tower.x, tower.y, ARENA.radius, 0, TAU);
    ctx.fillStyle = '#243b24';
    ctx.fill();
  }
  ctx.restore();
}

function drawTower(ctx, tower) {
  ctx.save();
  const spr = SPRITES.tower;
  const img = SHEETS[spr.sheet];
  const drawH = 48;
  const drawW = drawH * (spr.sw / spr.sh); // keep the sprite's aspect ratio
  if (imgReady(img)) {
    ctx.drawImage(img, spr.sx, spr.sy, spr.sw, spr.sh, tower.x - drawW / 2, tower.y - drawH / 2, drawW, drawH);
  } else {
    // Geometric fallback (before the sheet decodes / in headless).
    const s = TOWER.size;
    ctx.fillStyle = '#c9b06b';
    ctx.strokeStyle = '#7a6636';
    ctx.lineWidth = 3;
    ctx.fillRect(tower.x - s / 2, tower.y - s / 2, s, s);
    ctx.strokeRect(tower.x - s / 2, tower.y - s / 2, s, s);
  }
  // Archer(s) manning the tower. Multi-shot puts a second archer on the wall.
  const archerTop = drawArchers(ctx, tower);
  // HP bar just above whatever is tallest (the archers, or the tower if unloaded).
  const barY = (archerTop ?? tower.y - drawH / 2) - 8;
  drawBar(ctx, tower.x, barY, drawW, 5, tower.hp / tower.maxHp, '#e05555');
  ctx.restore();
}

// Draw the archer(s) standing on top of the tower. One normally; two side-by-side
// once the multi-shot trait is owned. Returns the top y of the archers (for HP-bar
// placement), or null if the sheet hasn't decoded yet.
function drawArchers(ctx, tower) {
  const spr = SPRITES.archer;
  const img = SHEETS[spr.sheet];
  if (!imgReady(img)) return null;
  const h = 32;
  const w = h * (spr.sw / spr.sh); // keep aspect
  const feetY = tower.y - 18; // stand on the tower's top edge, not inside it
  const topY = feetY - h;
  const dual = !!tower.traitState.multishot;
  const xs = dual ? [tower.x - 9, tower.x + 9] : [tower.x];
  for (const cx of xs) {
    ctx.drawImage(img, spr.sx, spr.sy, spr.sw, spr.sh, cx - w / 2, topY, w, h);
  }
  return topY;
}

function drawEnemy(ctx, e) {
  ctx.save();
  // Slimes share one sheet sprite (recoloured per tint); other enemies (goblin)
  // draw their own sheet rect directly.
  const spr = SPRITES[e.sprite] || SPRITES.slime;
  let src;
  if (e.sprite === 'slime') {
    src = slimeSpriteFor(e.tint); // recoloured source for green slimes
  } else {
    const img = SHEETS[spr.sheet];
    src = imgReady(img) ? { img, sx: spr.sx, sy: spr.sy, sw: spr.sw, sh: spr.sh } : null;
  }
  const drawW = e.radius * 2.6;
  const drawH = drawW * (spr.sh / spr.sw); // keep the sprite's aspect ratio

  // Slowed enemies get an icy aura behind them.
  if (e.slowMul < 1) {
    ctx.beginPath();
    ctx.arc(e.x, e.y, e.radius + 3, 0, TAU);
    ctx.fillStyle = 'rgba(120,200,255,0.28)';
    ctx.fill();
  }
  // Burning enemies get an orange glow.
  if (e.burnTimer > 0) {
    ctx.shadowColor = '#ff6a2a';
    ctx.shadowBlur = 10;
  }

  if (src) {
    ctx.drawImage(src.img, src.sx, src.sy, src.sw, src.sh, e.x - drawW / 2, e.y - drawH / 2, drawW, drawH);
  } else {
    ctx.beginPath();
    ctx.arc(e.x, e.y, e.radius, 0, TAU);
    ctx.fillStyle = e.slowMul < 1 ? tintFrost(e.color) : e.color;
    ctx.fill();
    ctx.lineWidth = 2;
    ctx.strokeStyle = e.slowMul < 1 ? '#9fd8ff' : 'rgba(0,0,0,0.35)';
    ctx.stroke();
  }
  ctx.shadowBlur = 0;
  if (e.hp < e.maxHp) {
    drawBar(ctx, e.x, e.y - drawH / 2 - 6, e.radius * 2, 3, e.hp / e.maxHp, '#e05555');
  }
  ctx.restore();
}

function drawProjectile(ctx, p) {
  const speed = Math.hypot(p.vx, p.vy) || 1;
  const len = 10;
  const tx = p.x - (p.vx / speed) * len;
  const ty = p.y - (p.vy / speed) * len;
  ctx.save();
  ctx.strokeStyle = p.target ? '#ffe08a' : 'rgba(255,224,138,0.5)';
  ctx.lineWidth = PROJECTILE.radius;
  ctx.lineCap = 'round';
  ctx.beginPath();
  ctx.moveTo(tx, ty);
  ctx.lineTo(p.x, p.y);
  ctx.stroke();
  ctx.restore();
}

// Blend an enemy's colour toward icy blue to signal a slow.
function tintFrost(hex) {
  const m = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(hex);
  if (!m) return '#8fd0ff';
  const mix = (c, t) => Math.round(parseInt(c, 16) * 0.5 + t * 0.5);
  const r = mix(m[1], 0x8f), g = mix(m[2], 0xd0), b = mix(m[3], 0xff);
  return `rgb(${r}, ${g}, ${b})`;
}

function drawBar(ctx, cx, y, width, height, frac, color) {
  const x = cx - width / 2;
  frac = Math.max(0, Math.min(1, frac));
  ctx.fillStyle = 'rgba(0,0,0,0.5)';
  ctx.fillRect(x, y, width, height);
  ctx.fillStyle = color;
  ctx.fillRect(x, y, width * frac, height);
}
