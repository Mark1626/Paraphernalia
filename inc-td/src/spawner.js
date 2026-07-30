// Round definitions and enemy spawning at the arena edge.

import { ARENA, ENEMIES } from "./config.js";
import { Enemy, slimePattern, walkPattern } from "./entities.js";

const PATTERNS = {
  slime: slimePattern,
  greenslime: slimePattern, // moves like a normal slime, just tankier
  largeslime: slimePattern, // ditto, just bigger (splits on death)
  largegreenslime: slimePattern,
  goblin: walkPattern, // fast steady march, no hop
};

// Data-driven round schedule. `duration` is optional and reserved for future
// time-boxed rounds; today every round is set-boxed (ends when enemies clear).
// Rounds 1-4 are plain slimes only; greens and the first large slime join at round 5.
export const ROUNDS = [
  { enemies: [{ type: "slime", count: 2 }], spawnInterval: 1.2 },
  { enemies: [{ type: "slime", count: 3 }], spawnInterval: 1.0 },
  { enemies: [{ type: "slime", count: 6 }], spawnInterval: 0.9 },
  { enemies: [{ type: "slime", count: 9 }], spawnInterval: 0.8 },
  {
    enemies: [
      { type: "slime", count: 10 },
      { type: "greenslime", count: 2 },
      { type: "largeslime", count: 1 },
    ],
    spawnInterval: 0.7,
  },
];

// Per-round stat scaling so enemies stay a threat after upgrades. HP and damage
// step up every round in three tiers: +50% of base per round through round 5, a
// gentle +15% per round through the mid-game, then +30% per round past round 10
// once the player has traits and golden upgrades online. Speed is left at base.
function scaleFor(roundNumber) {
  const r = Math.max(1, roundNumber);
  const early = Math.min(r, 5) - 1; // rounds 1-5: +50% of base each
  const mid = Math.max(0, Math.min(r, 10) - 5); // rounds 6-10: +15% of base each
  const late = Math.max(0, r - 10); // rounds 11+: +30% of base each
  const s = 1 + early * 0.5 + mid * 0.15 + late * 0.3;
  return { hp: s, damage: s };
}

// Rounds beyond the authored list are generated procedurally. Growth is kept
// gentle (see scaleFor's eased stat curve too) so rounds 6+ ramp instead of spike.
function roundDef(roundNumber) {
  if (roundNumber <= ROUNDS.length) return ROUNDS[roundNumber - 1];
  const extra = roundNumber - ROUNDS.length;
  const enemies = [];

  // Purple category: smalls + the odd large until round 10; from round 11 ONLY
  // large slimes spawn (their splits keep small slimes on the field), with the
  // small-slime budget folded in at one large ≈ four smalls' worth of HP.
  const smalls = 10 + Math.floor(extra * 1.5);
  const larges = 1 + Math.floor(extra / 3);
  if (roundNumber <= 10) {
    enemies.push({ type: "slime", count: smalls });
    enemies.push({ type: "largeslime", count: larges });
  } else {
    enemies.push({ type: "largeslime", count: larges + Math.ceil(smalls / 4) });
  }

  // Green category: the same switch, at round 20 (large greens join from round 9).
  const greens = 2 + Math.floor(extra / 3);
  const largeGreens = Math.floor(extra / 4);
  if (roundNumber <= 20) {
    enemies.push({ type: "greenslime", count: greens });
    if (largeGreens > 0) enemies.push({ type: "largegreenslime", count: largeGreens });
  } else {
    enemies.push({ type: "largegreenslime", count: largeGreens + Math.ceil(greens / 4) });
  }

  // Goblins join beyond round 20: a couple at first, one more each round.
  if (roundNumber > 20) {
    enemies.push({ type: "goblin", count: 2 + (roundNumber - 21) });
  }

  const spawnInterval = Math.max(0.45, 0.7 - extra * 0.015);
  return { enemies, spawnInterval };
}

// Builds a flat queue of enemy types for a round, round-robining the groups so
// different enemy types are interleaved rather than arriving in separate blocks.
export function buildSpawnQueue(roundNumber) {
  const def = roundDef(roundNumber);
  const remaining = def.enemies.map((g) => ({ type: g.type, count: g.count }));
  const queue = [];
  let placed = true;
  while (placed) {
    placed = false;
    for (const g of remaining) {
      if (g.count > 0) {
        queue.push(g.type);
        g.count--;
        placed = true;
      }
    }
  }
  return { queue, spawnInterval: def.spawnInterval };
}

// Creates one enemy of `type` at an exact position, stat-scaled for `roundNumber`.
// Used for edge spawns and for large-slime splits (children appear where the parent fell).
export function spawnEnemyAt(type, roundNumber, x, y) {
  const base = ENEMIES[type];
  const scale = scaleFor(roundNumber);
  const stats = {
    hp: Math.round(base.hp * scale.hp),
    damage: base.damage * scale.damage,
    speed: base.speed, // speed no longer scales — only HP & damage do
    radius: base.radius,
    color: base.color,
    attackInterval: base.attackInterval,
    stepPeriod: base.stepPeriod,
    tint: base.tint,
    sprite: base.sprite,
    splits: base.splits,
    round: roundNumber,
  };
  return new Enemy(type, x, y, stats, PATTERNS[type]);
}

// Creates one enemy of `type` at a random point on the spawn circle.
export function spawnEnemy(type, roundNumber, cx, cy, rng = Math.random) {
  const angle = rng() * Math.PI * 2;
  const x = cx + Math.cos(angle) * ARENA.spawnRadius;
  const y = cy + Math.sin(angle) * ARENA.spawnRadius;
  return spawnEnemyAt(type, roundNumber, x, y);
}
