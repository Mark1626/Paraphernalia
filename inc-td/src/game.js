// Game state machine + fixed-timestep loop. Owns all entity lists.

import { DT, PROJECTILE, REWARDS } from './config.js';
import { Tower, Projectile } from './entities.js';
import { buildSpawnQueue, spawnEnemy, spawnEnemyAt } from './spawner.js';
import { BASE_UPGRADES, TRAITS, rollRewards } from './upgrades.js';
import { render } from './render.js';
import { updateHud, showRewards, hideRewards, showGameOver } from './ui.js';
import { sound } from './sound.js';

export const State = { PLAYING: 'playing', ROUND_END: 'roundend', GAME_OVER: 'gameover' };

export class Game {
  constructor(canvas, ctx) {
    this.canvas = canvas;
    this.ctx = ctx;
    this.cx = canvas.width / 2;
    this.cy = canvas.height / 2;
    // Debug switches (see debug.js). Sandbox stops the automatic round-end /
    // upgrade flow so waves can be summoned manually; god mode blocks tower damage.
    this.sandbox = false;
    this.godMode = false;
    this.speed = 1; // simulation speed multiplier (see the 2x toggle)
    this.reset();
  }

  reset() {
    this.tower = new Tower(this.cx, this.cy);
    this.enemies = [];
    this.projectiles = [];
    // Fresh, per-run copies so picked traits/sub-upgrades don't leak across runs.
    // Base upgrades and trait sub-upgrades are kept apart: offers are slotted by
    // category (see rollUpgradeOffers).
    this.traitUpgradePool = [];
    this.traitPool = TRAITS.slice();
    this.roundWallTime = 0; // real seconds elapsed this round (updated by the loop)
    this.totalWallTime = 0; // real seconds elapsed across the whole run
    hideRewards();
    this.startRound(1);
  }

  startRound(n) {
    this.round = n;
    // New 5-round block (rounds 1, 6, 11, …): the trait and golden pities reset.
    if ((n - 1) % 5 === 0) {
      this.traitOfferedInBlock = false;
      this.goldenInBlock = false;
    }
    const { queue, spawnInterval } = buildSpawnQueue(n);
    this.spawnQueue = queue;
    this.spawnInterval = spawnInterval;
    this.spawnTimer = 0; // first enemy spawns immediately
    this.state = State.PLAYING;
    this.roundGameTime = 0; // simulated seconds elapsed this round (wall time reset in the loop)
    updateHud(this);
  }

  // One fixed simulation step.
  update(dt) {
    if (this.state !== State.PLAYING) return;
    this.roundGameTime += dt;

    this.spawnTimer -= dt;
    if (this.spawnQueue.length > 0 && this.spawnTimer <= 0) {
      this.enemies.push(spawnEnemy(this.spawnQueue.shift(), this.round, this.cx, this.cy));
      this.spawnTimer = this.spawnInterval;
    }

    for (const e of this.enemies) {
      e.tickEffects(dt); // frost slow / fire burn (burn can kill)
      const d = Math.hypot(e.x - this.cx, e.y - this.cy);
      if (d <= this.tower.radius + e.radius) {
        // In contact: hold position and attack on the enemy's interval. Enemies
        // persist here until the tower kills them.
        e.attacking = true;
        e.attackTimer -= dt;
        if (e.attackTimer <= 0) {
          if (!this.godMode) this.tower.hp -= e.damage;
          e.attackTimer += e.attackInterval;
        }
      } else {
        e.attacking = false;
        e.update(dt, this.cx, this.cy);
      }
    }

    this.tower.cooldown -= dt;
    const target = this.closestEnemy();
    if (target && this.tower.cooldown <= 0) {
      this.fire(target);
      this.tower.cooldown = 1 / this.tower.fireRate;
    }

    for (const p of this.projectiles) {
      p.update(dt);
      // Collision is geometric against ANY enemy in the projectile's path — so an
      // arrow aimed at one enemy but veering (a miss) still damages a different
      // enemy it physically crosses, and a foe in front can block a shot.
      const struck = this.enemyHitBy(p);
      if (struck) {
        struck.hp -= p.damage;
        // Apply any imbued on-hit effects (frost slow / fire burn).
        for (const eff of p.effects) {
          if (eff.slow) struck.applySlow(eff.slowMul, eff.duration);
          if (eff.burn) struck.applyBurn(eff.dps, eff.duration);
        }
        if (struck.hp <= 0) struck.dead = true;
        p.dead = true;
        sound.hit();
      } else if (p.target && p.target.dead) {
        p.target = null; // intended target died mid-flight; render as a stray
      }
    }

    // Dying large slimes split into two of their small variety where they fell,
    // stat-scaled for the round the parent spawned in. Push before the dead are
    // culled so the round-end check below sees the children.
    const born = [];
    for (const e of this.enemies) {
      if (!e.dead || !e.splits) continue;
      for (let i = 0; i < e.splits.count; i++) {
        const a = Math.random() * Math.PI * 2;
        born.push(spawnEnemyAt(e.splits.type, e.round, e.x + Math.cos(a) * e.radius, e.y + Math.sin(a) * e.radius));
      }
      e.splits = null;
    }
    this.enemies.push(...born);

    this.enemies = this.enemies.filter((e) => !e.dead);
    this.projectiles = this.projectiles.filter((p) => !p.dead);

    if (this.tower.hp <= 0) {
      this.tower.hp = 0;
      this.gameOver();
      return;
    }

    // In sandbox mode the round never auto-completes; waves are summoned by hand.
    if (!this.sandbox && this.spawnQueue.length === 0 && this.enemies.length === 0) {
      this.endRound();
      return;
    }

    updateHud(this);
  }

  closestEnemy() {
    let best = null;
    let bestD = Infinity;
    for (const e of this.enemies) {
      if (e.dead) continue;
      const d = (e.x - this.cx) ** 2 + (e.y - this.cy) ** 2;
      if (d < bestD) {
        bestD = d;
        best = e;
      }
    }
    return best;
  }

  // The nearest living enemy the projectile currently overlaps, or null.
  enemyHitBy(p) {
    let best = null;
    let bestD = Infinity;
    for (const e of this.enemies) {
      if (e.dead) continue;
      const reach = e.radius + PROJECTILE.radius;
      const d = Math.hypot(p.x - e.x, p.y - e.y);
      if (d <= reach && d < bestD) {
        bestD = d;
        best = e;
      }
    }
    return best;
  }

  // Fire a base arrow, then let trait hooks queue extras. Hooks receive the game
  // so they can pick their own targets (e.g. multi-shot fires at other enemies).
  fire(target) {
    const spawn = (tgt, damageMul = 1) => {
      if (tgt) this.spawnProjectile(tgt, this.tower.damage * damageMul);
    };
    spawn(target, 1);
    for (const hook of this.tower.fireHooks) hook(this.tower, target, spawn, this);
  }

  // Living enemies other than `exclude`, ordered nearest-first to the tower.
  otherEnemies(exclude) {
    return this.enemies
      .filter((e) => !e.dead && e !== exclude)
      .sort(
        (a, b) =>
          (a.x - this.cx) ** 2 + (a.y - this.cy) ** 2 - ((b.x - this.cx) ** 2 + (b.y - this.cy) ** 2),
      );
  }

  // Arrows are aimed straight at the target. The arrow can still strike whatever
  // enemy it physically flies through first (see the collision loop).
  spawnProjectile(target, damage) {
    const angle = Math.atan2(target.y - this.cy, target.x - this.cx);
    const vx = Math.cos(angle) * PROJECTILE.speed;
    const vy = Math.sin(angle) * PROJECTILE.speed;
    const sx = this.cx + Math.cos(angle) * this.tower.radius;
    const sy = this.cy + Math.sin(angle) * this.tower.radius;
    const proj = new Projectile(sx, sy, vx, vy, damage, target);
    // Tag the arrow with any imbued on-hit effects (frost/fire) that proc.
    for (const imbue of this.tower.imbues) {
      const eff = imbue(this.tower);
      if (eff) proj.effects.push(eff);
    }
    this.projectiles.push(proj);
    sound.arrow();
  }

  endRound() {
    this.state = State.ROUND_END;
    // Trait offers are chance-based, with a pity guarantee: if the current 5-round
    // block hasn't offered a trait yet, its last round (5, 10, …) forces one.
    const canTrait = this.traitPool.length > 0;
    const blockEnd = this.round % 5 === 0;
    const mustTrait = canTrait && !this.traitOfferedInBlock && blockEnd;
    let isTrait = canTrait && (mustTrait || Math.random() < REWARDS.traitChance);
    // Golden pity: each block must also hand the player at least one Golden pick.
    // At block end a still-pending golden outranks a CHANCE trait roll (it can never
    // clash with a forced one: a block that reaches round 5 without a trait offer
    // was all upgrades, so its golden was already forced at round 4 below).
    if (blockEnd && !this.goldenInBlock && !mustTrait) isTrait = false;
    if (isTrait) this.traitOfferedInBlock = true;
    // Force golden on the block's last upgrade-offer round: round 5 normally, or
    // round 4 when a pending trait pity means round 5 is guaranteed to be traits.
    // Forcing turns ALL cards golden so the pick is golden no matter the choice.
    const lastUpgradeRound = blockEnd || (this.round % 5 === 4 && canTrait && !this.traitOfferedInBlock);
    const forceGolden = !isTrait && !this.goldenInBlock && lastUpgradeRound;
    const rewards = isTrait
      ? rollRewards(this.traitPool.filter((r) => !r.eligible || r.eligible(this.tower)), 3)
      : this.rollUpgradeOffers();
    // Each upgrade card independently rolls Golden (3x effect); traits never do.
    const offers = rewards.map((reward) => ({
      reward,
      golden: !isTrait && (forceGolden || Math.random() < REWARDS.goldenChance),
    }));
    // Pass the current round so cards can show round-scaled amounts (health/damage).
    showRewards(offers, isTrait, this.round, (r, golden) => this.pickReward(r, isTrait, golden));
  }

  // Upgrade offers are slotted by category: card 1 is always a base stat (Health /
  // Damage / Attack Speed), card 2 is always a trait sub-upgrade once any are
  // unlocked (and not maxed), and card 3 can come from either category. When the
  // trait slot has nothing to offer (no traits yet, or all capped) it falls back
  // to base stats, so early rounds simply show the three base upgrades.
  rollUpgradeOffers() {
    const ok = (r) => !r.eligible || r.eligible(this.tower);
    const base = BASE_UPGRADES.filter(ok);
    const traitUps = this.traitUpgradePool.filter(ok);
    const picks = [...rollRewards(base, 1), ...rollRewards(traitUps, 1)];
    while (picks.length < 3) {
      const rest = [...base, ...traitUps].filter((r) => !picks.includes(r));
      if (!rest.length) break;
      picks.push(...rollRewards(rest, 1));
    }
    return picks;
  }

  // Apply a reward's effect and do the pool bookkeeping, without advancing rounds.
  // `mult` scales the upgrade's step (Golden picks apply REWARDS.goldenMult).
  applyReward(reward, isTrait, mult = 1) {
    reward.apply(this.tower, this.round, mult);
    if (isTrait) {
      this.traitPool = this.traitPool.filter((t) => t.id !== reward.id);
      if (reward.subUpgrades) this.traitUpgradePool.push(...reward.subUpgrades);
    }
  }

  pickReward(reward, isTrait, golden = false) {
    if (golden) this.goldenInBlock = true;
    this.applyReward(reward, isTrait, golden ? REWARDS.goldenMult : 1);
    hideRewards();
    this.startRound(this.round + 1);
  }

  // --- Debug / sandbox actions (driven by debug.js) --------------------------

  // Immediately add `count` enemies of `type`, scaled as if for `round`.
  spawnWave(count = 5, type = 'slime', round = this.round) {
    for (let i = 0; i < count; i++) {
      this.enemies.push(spawnEnemy(type, round, this.cx, this.cy));
    }
    if (this.state !== State.PLAYING) {
      hideRewards();
      this.state = State.PLAYING;
    }
  }

  clearEnemies() {
    this.enemies = [];
    this.projectiles = [];
  }

  // Mark every enemy dead so the tower's kill logic / round flow treats them as slain.
  // Splits are suppressed so "Kill all" actually clears the field.
  killEnemies() {
    for (const e of this.enemies) {
      e.splits = null;
      e.dead = true;
    }
  }

  // Grant a trait by id (e.g. from the debug panel), if not already taken.
  grantTrait(id) {
    const trait = this.traitPool.find((t) => t.id === id);
    if (trait) this.applyReward(trait, true);
    return !!trait;
  }

  setSandbox(on) {
    this.sandbox = on;
    if (on) {
      // Leave the reward overlay / auto-spawn behind and drop into free play.
      this.spawnQueue = [];
      hideRewards();
      this.state = State.PLAYING;
    }
    updateHud(this);
  }

  gameOver() {
    this.state = State.GAME_OVER;
    updateHud(this);
    showGameOver(this.round, () => this.reset());
  }

  // Accumulator-based fixed update + rAF render.
  start() {
    let last = performance.now();
    let acc = 0;
    let wallRoundStart = last; // real timestamp the current round began
    let prevRound = this.round;
    const loop = (now) => {
      // Real elapsed time is clamped (anti spiral-of-death) then scaled by speed,
      // so 2x runs twice as many fixed steps per frame.
      const real = Math.min(0.25, (now - last) / 1000);
      acc += real * this.speed;
      last = now;
      this.totalWallTime += real; // wall-clock time played across the whole run
      while (acc >= DT) {
        this.update(DT);
        acc -= DT;
      }
      // Reset the wall-clock stopwatch whenever the round changes.
      if (this.round !== prevRound) {
        prevRound = this.round;
        wallRoundStart = now;
      }
      this.roundWallTime = (now - wallRoundStart) / 1000;
      render(this.ctx, this.canvas, this);
      requestAnimationFrame(loop);
    };
    requestAnimationFrame(loop);
  }
}
