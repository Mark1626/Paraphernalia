// Sprite-sheet loading + source rectangles for drawn entities.
//
// Guarded so this module is import-safe without a DOM (the headless test harness
// has no `Image`): there, sheets are null and render.js falls back to shapes.

const HAS_IMAGE = typeof Image !== 'undefined';

function loadImage(src) {
  if (!HAS_IMAGE) return null;
  const img = new Image();
  img.src = src;
  return img;
}

// Loaded sheet images, keyed by name.
export const SHEETS = {
  towers: loadImage('assets/towers.png'),
  characters: loadImage('assets/characters.png'),
  tiles: loadImage('assets/tileset1.png'),
};

// Ground-tile rects from tileset1.png, all indexing the FIRST of its four 96px-wide
// recoloured themes (green grass / tan dirt). `grass` is the solid base swatch from
// the strip at the sheet's bottom; the rest are pre-composed dirt-on-grass shapes.
export const TILES = {
  grass: { sx: 32, sy: 208, sw: 32, sh: 16 },
  arenaCircle: { sx: 1, sy: 1, sw: 46, sh: 46 }, // big filled dirt circle
  donut: { sx: 20, sy: 147, sw: 24, sh: 25 },
  patch: { sx: 68, sy: 148, sw: 24, sh: 24 },
  stripH: { sx: 2, sy: 179, sw: 42, sh: 11 },
  stripV: { sx: 51, sy: 148, sw: 10, sh: 42 },
};

// Source rect (which sheet + sx,sy,sw,sh) for each entity sprite. Coordinates were
// measured off the sheets: the tower is the 1st sprite of row 2 in towers.png; the
// characters.png sheet holds five 15px-pitch characters — goblin, slime, knight,
// archer, wizard, in that order.
export const SPRITES = {
  tower: { sheet: 'towers', sx: 0, sy: 56, sw: 26, sh: 24 },
  // The goblin is the 1st character, right before the slime.
  goblin: { sheet: 'characters', sx: 0, sy: 0, sw: 14, sh: 16 },
  slime: { sheet: 'characters', sx: 15, sy: 10, sw: 12, sh: 6 },
  // The archer (4th of the five characters) rides on top of the tower.
  archer: { sheet: 'characters', sx: 46, sy: 0, sw: 14, sh: 16 },
};

// A sheet is drawable once the browser has decoded it.
export function imgReady(img) {
  return !!img && img.complete && img.naturalWidth > 0;
}

// --- Slime recolouring --------------------------------------------------------
//
// The slime sprite is purple; variants (e.g. the green slime) are produced once by
// replacing the hue of every opaque pixel and caching the result as an offscreen
// canvas. Returns a ready-to-draw source { img, sx, sy, sw, sh } — the untouched
// sheet for the default slime, or a cached recoloured canvas for a tint. Returns
// null if the sheet hasn't decoded yet (caller falls back to a shape).

const TINT_HUE = { green: 120 }; // target hue (degrees) per tint key
const tintCache = {}; // tint key -> drawable source

export function slimeSpriteFor(tint) {
  const spr = SPRITES.slime;
  const img = SHEETS[spr.sheet];
  if (!imgReady(img)) return null;
  if (!tint || !(tint in TINT_HUE)) {
    return { img, sx: spr.sx, sy: spr.sy, sw: spr.sw, sh: spr.sh };
  }
  if (!tintCache[tint]) {
    try {
      tintCache[tint] = makeTintedSlime(TINT_HUE[tint]);
    } catch {
      // getImageData can throw on a tainted canvas (e.g. file://) — fall back.
      tintCache[tint] = { img, sx: spr.sx, sy: spr.sy, sw: spr.sw, sh: spr.sh };
    }
  }
  return tintCache[tint];
}

function makeTintedSlime(hueDeg) {
  const spr = SPRITES.slime;
  const img = SHEETS[spr.sheet];
  const cv = document.createElement('canvas');
  cv.width = spr.sw;
  cv.height = spr.sh;
  const c = cv.getContext('2d');
  c.imageSmoothingEnabled = false;
  c.drawImage(img, spr.sx, spr.sy, spr.sw, spr.sh, 0, 0, spr.sw, spr.sh);
  const data = c.getImageData(0, 0, spr.sw, spr.sh);
  const p = data.data;
  const h = hueDeg / 360;
  for (let i = 0; i < p.length; i += 4) {
    if (p[i + 3] === 0) continue; // skip transparent
    const [, s, l] = rgbToHsl(p[i], p[i + 1], p[i + 2]);
    const [r, g, b] = hslToRgb(h, s, l); // keep saturation/lightness, swap hue
    p[i] = r;
    p[i + 1] = g;
    p[i + 2] = b;
  }
  c.putImageData(data, 0, 0);
  return { img: cv, sx: 0, sy: 0, sw: spr.sw, sh: spr.sh };
}

function rgbToHsl(r, g, b) {
  r /= 255;
  g /= 255;
  b /= 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  let h = 0;
  let s = 0;
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r) h = (g - b) / d + (g < b ? 6 : 0);
    else if (max === g) h = (b - r) / d + 2;
    else h = (r - g) / d + 4;
    h /= 6;
  }
  return [h, s, l];
}

function hslToRgb(h, s, l) {
  let r;
  let g;
  let b;
  if (s === 0) {
    r = g = b = l;
  } else {
    const hue2rgb = (p, q, t) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1 / 6) return p + (q - p) * 6 * t;
      if (t < 1 / 2) return q;
      if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
      return p;
    };
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1 / 3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1 / 3);
  }
  return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}
