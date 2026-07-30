// Tunable constants for the whole game. Balance lives here.

export const ARENA = {
  // Radius of the circular play field, in pixels. The tower sits at its center.
  radius: 260,
  // Enemies spawn on this circle (slightly outside the arena edge so they slide in).
  spawnRadius: 275,
};

export const TOWER = {
  maxHp: 10,
  damage: 5,
  fireRate: 1, // shots per second
  size: 28, // side length of the drawn square
  radius: 16, // collision radius (enemies reaching this deal damage)
};

// Base per-enemy stats. Round scaling (hp & damage) is applied on top (see spawner).
export const ENEMIES = {
  slime: {
    hp: 10,
    damage: 1,
    speed: 10, // pixels per second
    radius: 12,
    color: "#a05fd0", // fallback tint (the sprite itself is purple)
    attackInterval: 1.0, // seconds between attacks once in contact with the tower
    stepPeriod: 0.6, // seconds per eased "hop" — see slimePattern (avg speed unchanged)
  },
  // Green slime: the same sprite recoloured green (see sprites.js), with 2x the HP.
  greenslime: {
    hp: 20, // twice the normal slime
    damage: 1,
    speed: 10,
    radius: 12,
    color: "#5fbf5f",
    attackInterval: 1.0,
    stepPeriod: 0.6,
    tint: "green", // recolour key for the slime sprite
  },
  // Large slimes: drawn 2x the size with 2x the HP of their small variety; on death
  // they split into two normal slimes of the same variety (see `splits`).
  largeslime: {
    hp: 20, // twice the normal slime
    damage: 1,
    speed: 10,
    radius: 24, // 2x radius => drawn twice as large
    color: "#a05fd0",
    attackInterval: 1.0,
    stepPeriod: 0.6,
    splits: { type: "slime", count: 2 },
  },
  largegreenslime: {
    hp: 40, // twice the green slime
    damage: 1,
    speed: 10,
    radius: 24,
    color: "#5fbf5f",
    attackInterval: 1.0,
    stepPeriod: 0.6,
    tint: "green",
    splits: { type: "greenslime", count: 2 },
  },
  // Goblin (rounds 21+): a fast steady walker — quicker and harder-hitting than a
  // slime, but with less HP than the round's slimes. Uses its own sheet sprite.
  goblin: {
    hp: 15,
    damage: 2,
    speed: 25,
    radius: 10,
    color: "#5f9e4a",
    attackInterval: 0.8,
    sprite: "goblin", // 1st character on the sheet, before the slime (sprites.js)
  },
};

export const PROJECTILE = {
  speed: 340, // pixels per second
  radius: 4,
  life: 2.0, // seconds before a projectile is culled
};

// How much each base upgrade improves the tower when chosen.
export const UPGRADE_STEP = {
  maxHp: 10,
  damage: 3,
  fireRate: 0.3,
  // Max fire-rate gain from a SINGLE pick — caps the golden 3x (0.9 is overkill).
  fireRateStepCap: 0.6,
};

// Reward-offer tuning. Trait offers are chance-based with a pity guarantee: every
// round-end rolls `traitChance` to offer traits instead of upgrades (while any
// remain), and if a 5-round block would otherwise end without one, its last round
// forces a trait offer. Regular upgrade cards are Silver; each rolls a small
// chance to come up Golden and apply `goldenMult` times the normal step. Golden
// has its own per-block pity: if none was picked, the block's last upgrade-offer
// round turns every card golden (see game.endRound).
export const REWARDS = {
  traitChance: 0.25,
  goldenChance: 0.1,
  goldenMult: 3,
};

// Multi-shot trait tuning.
export const MULTISHOT = {
  baseChance: 0.1, // 10% chance to proc
  baseCount: 1, // extra arrows on proc
  baseDamageMul: 0.5, // extra arrows deal 50% of tower damage
  chanceStep: 0.1,
  countStep: 1,
  damageMulStep: 0.15,
};

// Frost-arrow trait: a proc slows the struck enemy for a short time.
export const FROST = {
  baseChance: 0.1, // 10% chance to proc
  baseSlowPct: 0.1, // -10% move speed on proc
  baseDuration: 2.0, // seconds the slow lasts
  chanceStep: 0.1,
  slowPctStep: 0.1,
  durationStep: 0.5,
};

// Fire-arrow trait: a proc sets the struck enemy alight (damage over time).
// Burn DPS is granted as a SNAPSHOT: picking the trait (or a Burn DPS upgrade)
// converts a percentage of the tower's arrow damage AT THAT MOMENT into flat
// burn DPS. It does not ride along as arrow damage grows afterwards.
export const FIRE = {
  baseChance: 0.1, // 10% chance to proc
  baseDpsPct: 0.6, // trait pick grants 60% of current arrow damage as burn DPS
  baseDuration: 2.0, // seconds the burn lasts
  chanceStep: 0.1,
  dpsPctStep: 0.2, // each Burn DPS upgrade adds 20% of current arrow damage
  durationStep: 1.0,
};

// Ceilings on trait sub-upgrades (enforced via upgrade eligibility + apply clamps).
export const TRAIT_CAPS = {
  procChance: 0.5, // every trait's proc chance tops out at 50%
  frostSlowPct: 0.7, // frost slow can't exceed 70%
  multishotArrows: 5, // at most 5 extra arrows
  fireBurnDuration: 10, // burn duration tops out at 10s
};

// Fixed-timestep length for the simulation (seconds).
export const DT = 1 / 60;
