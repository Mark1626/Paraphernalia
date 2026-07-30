// Upgrade & trait pools plus the reward-roll logic.
//
// An upgrade/trait is { id, name, desc, apply(tower) }. An optional `eligible(tower)`
// predicate lets an upgrade drop out of the offer pool once it is maxed (e.g. a proc
// chance at its cap). Traits additionally carry `subUpgrades` that get folded into the
// live upgrade pool once the trait is picked.

import { UPGRADE_STEP, MULTISHOT, FROST, FIRE, TRAIT_CAPS } from './config.js';

// Health gains scale linearly with the round being played, so they keep pace with
// round-scaled enemy damage. gain = base * round (round 1 = base amount).
export function roundStep(base, round = 1) {
  return base * Math.max(1, round);
}

// Damage gains grow with the SQUARE ROOT of the round instead: still bigger later,
// but gently — linear scaling was handing out +70 or more per card by round 24,
// dwarfing the tower's own damage. (r1: +3, r9: +9, r24: +15.)
export function damageStep(round = 1) {
  return Math.round(UPGRADE_STEP.damage * Math.sqrt(Math.max(1, round)));
}

// --- Base upgrades (repeatable, always in the pool) ---------------------------
//
// `apply(tower, round, mult)` receives the current round and a step multiplier
// (1 for Silver picks, REWARDS.goldenMult for Golden ones). Upgrades that scale
// with the round (health, damage) use it, the rest ignore it. `desc` is a
// function of (round, mult) so the card shows the exact amount being offered.

export const BASE_UPGRADES = [
  {
    id: 'health',
    name: 'Health',
    desc: (round, mult = 1) => `+${roundStep(UPGRADE_STEP.maxHp, round) * mult} max HP (and heal)`,
    apply(t, round, mult = 1) {
      const inc = roundStep(UPGRADE_STEP.maxHp, round) * mult;
      t.maxHp += inc;
      t.hp += inc;
    },
  },
  {
    id: 'damage',
    name: 'Damage',
    desc: (round, mult = 1) => `+${damageStep(round) * mult} arrow damage`,
    apply(t, round, mult = 1) {
      t.damage += damageStep(round) * mult;
    },
  },
  {
    id: 'attackspeed',
    name: 'Attack Speed',
    // A single pick's gain is capped (golden 3x would otherwise be 0.9 shots/sec).
    desc: (round, mult = 1) =>
      `+${Math.min(UPGRADE_STEP.fireRate * mult, UPGRADE_STEP.fireRateStepCap).toFixed(1)} shots/sec`,
    apply(t, round, mult = 1) {
      t.fireRate += Math.min(UPGRADE_STEP.fireRate * mult, UPGRADE_STEP.fireRateStepCap);
    },
  },
];

// --- Traits (chance-based offers with a 5-round pity — see game.endRound) -----

// The multishot fire hook: on proc, queue `count` extra arrows at reduced damage,
// each aimed at a DIFFERENT enemy (nearest others first). If there are fewer other
// enemies than arrows, they cycle through whoever is available; with no other enemy
// the extras fall back to the primary target.
function multishotHook(tower, target, spawn, game) {
  const st = tower.traitState.multishot;
  if (Math.random() >= st.chance) return;
  const others = game.otherEnemies(target);
  for (let i = 0; i < st.count; i++) {
    const tgt = others.length ? others[i % others.length] : target;
    spawn(tgt, st.damageMul);
  }
}

const hasMultishot = (t) => !!t.traitState.multishot;

const MULTISHOT_SUBUPGRADES = [
  {
    id: 'ms-chance',
    name: 'Multi-shot: Chance',
    desc: (round, mult = 1) => `+${Math.round(MULTISHOT.chanceStep * 100 * mult)}% proc chance`,
    // Only while owned and below the shared 50% proc-chance ceiling.
    eligible: (t) => hasMultishot(t) && t.traitState.multishot.chance < TRAIT_CAPS.procChance,
    apply(t, round, mult = 1) {
      t.traitState.multishot.chance = Math.min(
        TRAIT_CAPS.procChance,
        t.traitState.multishot.chance + MULTISHOT.chanceStep * mult,
      );
    },
  },
  {
    id: 'ms-count',
    name: 'Multi-shot: Arrows',
    desc: (round, mult = 1) => `+${MULTISHOT.countStep * mult} extra arrow${MULTISHOT.countStep * mult > 1 ? 's' : ''}`,
    // Capped at TRAIT_CAPS.multishotArrows extra arrows.
    eligible: (t) => hasMultishot(t) && t.traitState.multishot.count < TRAIT_CAPS.multishotArrows,
    apply(t, round, mult = 1) {
      t.traitState.multishot.count = Math.min(
        TRAIT_CAPS.multishotArrows,
        t.traitState.multishot.count + MULTISHOT.countStep * mult,
      );
    },
  },
  {
    id: 'ms-damage',
    name: 'Multi-shot: Damage',
    desc: (round, mult = 1) => `+${Math.round(MULTISHOT.damageMulStep * 100 * mult)}% extra-arrow damage`,
    eligible: hasMultishot,
    apply(t, round, mult = 1) {
      t.traitState.multishot.damageMul += MULTISHOT.damageMulStep * mult;
    },
  },
];

// --- Frost arrow: proc slows the struck enemy ---------------------------------

// Frost is an IMBUE (not an extra arrow): on proc it tags the outgoing projectile
// with a slow effect that lands on whatever enemy the arrow hits.
function frostImbue(tower) {
  const st = tower.traitState.frost;
  if (Math.random() >= st.chance) return null;
  return { slow: true, slowMul: 1 - st.slowPct, duration: st.duration };
}

const hasFrost = (t) => !!t.traitState.frost;

const FROST_SUBUPGRADES = [
  {
    id: 'frost-chance',
    name: 'Frost: Chance',
    desc: (round, mult = 1) => `+${Math.round(FROST.chanceStep * 100 * mult)}% proc chance`,
    eligible: (t) => hasFrost(t) && t.traitState.frost.chance < TRAIT_CAPS.procChance,
    apply(t, round, mult = 1) {
      t.traitState.frost.chance = Math.min(TRAIT_CAPS.procChance, t.traitState.frost.chance + FROST.chanceStep * mult);
    },
  },
  {
    id: 'frost-slow',
    name: 'Frost: Slow %',
    desc: (round, mult = 1) => `+${Math.round(FROST.slowPctStep * 100 * mult)}% slow`,
    eligible: (t) => hasFrost(t) && t.traitState.frost.slowPct < TRAIT_CAPS.frostSlowPct,
    apply(t, round, mult = 1) {
      t.traitState.frost.slowPct = Math.min(TRAIT_CAPS.frostSlowPct, t.traitState.frost.slowPct + FROST.slowPctStep * mult);
    },
  },
  {
    id: 'frost-duration',
    name: 'Frost: Duration',
    desc: (round, mult = 1) => `+${FROST.durationStep * mult}s slow duration`,
    eligible: hasFrost,
    apply(t, round, mult = 1) {
      t.traitState.frost.duration += FROST.durationStep * mult;
    },
  },
];

// --- Fire arrow: proc sets the struck enemy alight (burn DoT) ------------------

function fireImbue(tower) {
  const st = tower.traitState.fire;
  if (Math.random() >= st.chance) return null;
  return { burn: true, dps: st.dps, duration: st.duration };
}

const hasFire = (t) => !!t.traitState.fire;

const FIRE_SUBUPGRADES = [
  {
    id: 'fire-chance',
    name: 'Fire: Chance',
    desc: (round, mult = 1) => `+${Math.round(FIRE.chanceStep * 100 * mult)}% proc chance`,
    eligible: (t) => hasFire(t) && t.traitState.fire.chance < TRAIT_CAPS.procChance,
    apply(t, round, mult = 1) {
      t.traitState.fire.chance = Math.min(TRAIT_CAPS.procChance, t.traitState.fire.chance + FIRE.chanceStep * mult);
    },
  },
  {
    id: 'fire-damage',
    name: 'Fire: Burn DPS',
    // Snapshots the tower's arrow damage when picked — the gain is flat afterwards.
    desc: (round, mult = 1) => `+${Math.round(FIRE.dpsPctStep * 100 * mult)}% of current arrow dmg as burn/sec`,
    eligible: hasFire,
    apply(t, round, mult = 1) {
      t.traitState.fire.dps += t.damage * FIRE.dpsPctStep * mult;
    },
  },
  {
    id: 'fire-duration',
    name: 'Fire: Duration',
    desc: (round, mult = 1) => `+${FIRE.durationStep * mult}s burn duration`,
    eligible: (t) => hasFire(t) && t.traitState.fire.duration < TRAIT_CAPS.fireBurnDuration,
    apply(t, round, mult = 1) {
      t.traitState.fire.duration = Math.min(TRAIT_CAPS.fireBurnDuration, t.traitState.fire.duration + FIRE.durationStep * mult);
    },
  },
];

export const TRAITS = [
  {
    id: 'multishot',
    name: 'Multi-shot',
    desc: `${Math.round(MULTISHOT.baseChance * 100)}% chance for +${MULTISHOT.baseCount} arrow at ${Math.round(
      MULTISHOT.baseDamageMul * 100,
    )}% damage`,
    subUpgrades: MULTISHOT_SUBUPGRADES,
    apply(t) {
      t.traits.push('multishot');
      t.traitState.multishot = {
        chance: MULTISHOT.baseChance,
        count: MULTISHOT.baseCount,
        damageMul: MULTISHOT.baseDamageMul,
      };
      t.fireHooks.push(multishotHook);
    },
  },
  {
    id: 'frost',
    name: 'Frost Arrow',
    desc: `${Math.round(FROST.baseChance * 100)}% chance to slow ${Math.round(
      FROST.baseSlowPct * 100,
    )}% for ${FROST.baseDuration}s`,
    subUpgrades: FROST_SUBUPGRADES,
    apply(t) {
      t.traits.push('frost');
      t.traitState.frost = {
        chance: FROST.baseChance,
        slowPct: FROST.baseSlowPct,
        duration: FROST.baseDuration,
      };
      t.imbues.push(frostImbue);
    },
  },
  {
    id: 'fire',
    name: 'Fire Arrow',
    desc: `${Math.round(FIRE.baseChance * 100)}% chance to burn ${Math.round(
      FIRE.baseDpsPct * 100,
    )}% of arrow dmg/s for ${FIRE.baseDuration}s`,
    subUpgrades: FIRE_SUBUPGRADES,
    apply(t) {
      t.traits.push('fire');
      t.traitState.fire = {
        chance: FIRE.baseChance,
        // Snapshot: 60% of arrow damage at pick time, flat from then on.
        dps: t.damage * FIRE.baseDpsPct,
        duration: FIRE.baseDuration,
      };
      t.imbues.push(fireImbue);
    },
  },
];

// --- Rolling rewards ----------------------------------------------------------

// Pick `n` distinct entries from `pool` at random. Falls back to fewer when the
// pool is small.
export function rollRewards(pool, n = 3, rng = Math.random) {
  const copy = pool.slice();
  const picks = [];
  while (picks.length < n && copy.length > 0) {
    const i = Math.floor(rng() * copy.length);
    picks.push(copy.splice(i, 1)[0]);
  }
  return picks;
}
