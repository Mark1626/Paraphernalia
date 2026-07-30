// Entity classes: Tower, Enemy, Projectile.
// Movement patterns are plain functions so new enemy behaviours slot in easily.

import { TOWER, PROJECTILE } from './config.js';

export class Tower {
  constructor(x, y) {
    this.x = x;
    this.y = y;
    this.maxHp = TOWER.maxHp;
    this.hp = TOWER.maxHp;
    this.damage = TOWER.damage;
    this.fireRate = TOWER.fireRate; // shots per second
    this.cooldown = 0; // seconds until next shot is allowed
    this.traits = []; // active trait ids, e.g. 'multishot'
    // Fire hooks let traits inject extra projectiles. Each hook is
    // (tower, target, spawn) => void, where spawn(target, damageMul) queues a shot.
    this.fireHooks = [];
    // Imbue hooks tag each outgoing projectile with on-hit effects (frost/fire).
    // Each is (tower) => effect|null, where effect is applied when the arrow strikes.
    this.imbues = [];
    // Per-trait mutable state (e.g. multishot chance/count/damage).
    this.traitState = {};
  }

  get radius() {
    return TOWER.radius;
  }
}

// Quadratic ease-in-out VELOCITY profile over one step. It's a tent that is 0 at
// the ends of a step and peaks (2x) mid-step, and its average over the cycle is
// exactly 1 — so the slime eases into and out of each "hop" while its average speed
// stays equal to `enemy.speed`. (This is the derivative of easeInOutQuad.)
function stepEaseVel(phase) {
  return phase < 0.5 ? 4 * phase : 4 * (1 - phase);
}

// Steady straight-line march toward the arena center (goblins don't hop).
export function walkPattern(enemy, dt, cx, cy) {
  const dx = cx - enemy.x;
  const dy = cy - enemy.y;
  const dist = Math.hypot(dx, dy) || 1;
  const speed = enemy.moveSpeed();
  enemy.x += (dx / dist) * speed * dt;
  enemy.y += (dy / dist) * speed * dt;
}

// Straight line toward the arena center, but advanced with an eased per-step
// velocity so the motion pulses smoothly (accelerate → decelerate) each hop
// instead of sliding at a constant rate.
export function slimePattern(enemy, dt, cx, cy) {
  enemy.stepPhase += dt / enemy.stepPeriod;
  if (enemy.stepPhase >= 1) enemy.stepPhase -= Math.floor(enemy.stepPhase);
  const factor = stepEaseVel(enemy.stepPhase);
  const dx = cx - enemy.x;
  const dy = cy - enemy.y;
  const dist = Math.hypot(dx, dy) || 1;
  const speed = enemy.moveSpeed() * factor;
  enemy.x += (dx / dist) * speed * dt;
  enemy.y += (dy / dist) * speed * dt;
}

export class Enemy {
  constructor(type, x, y, stats, pattern) {
    this.type = type;
    this.x = x;
    this.y = y;
    this.maxHp = stats.hp;
    this.hp = stats.hp;
    this.damage = stats.damage;
    this.speed = stats.speed;
    this.radius = stats.radius;
    this.color = stats.color;
    this.sprite = stats.sprite || 'slime'; // which sheet sprite to draw (see render.js)
    this.tint = stats.tint || null; // sprite recolour key (e.g. 'green'), null = as-is
    this.splits = stats.splits || null; // on death: { type, count } children (large slimes)
    this.round = stats.round || 1; // round this enemy was stat-scaled for (children reuse it)
    this.pattern = pattern;
    this.dead = false;
    // Once in contact the enemy stops and attacks on this interval instead of
    // vanishing. attackTimer starts at 0 so the first hit lands on contact.
    this.attackInterval = stats.attackInterval;
    this.attackTimer = 0;
    this.attacking = false;
    // Eased "hop" motion: phase advances 0→1 each step (randomised so a wave of
    // slimes doesn't hop in lockstep). See slimePattern.
    this.stepPeriod = stats.stepPeriod || 0.6;
    this.stepPhase = Math.random();
    // Status effects (frost slow / fire burn). slowMul multiplies move speed.
    this.slowMul = 1;
    this.slowTimer = 0;
    this.burnDps = 0;
    this.burnTimer = 0;
  }

  update(dt, cx, cy) {
    this.pattern(this, dt, cx, cy);
  }

  // Current move speed after any active slow.
  moveSpeed() {
    return this.speed * this.slowMul;
  }

  // Apply a slow: the strongest slow wins and the timer is refreshed.
  applySlow(mul, duration) {
    this.slowMul = Math.min(this.slowMul, mul);
    this.slowTimer = Math.max(this.slowTimer, duration);
  }

  // Apply a burn: the hottest burn wins and the timer is refreshed.
  applyBurn(dps, duration) {
    this.burnDps = Math.max(this.burnDps, dps);
    this.burnTimer = Math.max(this.burnTimer, duration);
  }

  // Tick status effects once per fixed step (burn can kill).
  tickEffects(dt) {
    if (this.slowTimer > 0) {
      this.slowTimer -= dt;
      if (this.slowTimer <= 0) {
        this.slowTimer = 0;
        this.slowMul = 1;
      }
    }
    if (this.burnTimer > 0) {
      this.burnTimer -= dt;
      this.hp -= this.burnDps * dt;
      if (this.burnTimer <= 0) {
        this.burnTimer = 0;
        this.burnDps = 0;
      }
      if (this.hp <= 0) this.dead = true;
    }
  }
}

export class Projectile {
  // vx/vy are the velocity direction * speed. `target` is the intended enemy
  // (null for a miss). Hits are resolved by proximity to the target.
  constructor(x, y, vx, vy, damage, target) {
    this.x = x;
    this.y = y;
    this.vx = vx;
    this.vy = vy;
    this.damage = damage;
    this.target = target;
    this.life = PROJECTILE.life;
    this.dead = false;
    // On-hit effects (frost/fire) attached at spawn time by tower imbues.
    this.effects = [];
  }

  update(dt) {
    this.x += this.vx * dt;
    this.y += this.vy * dt;
    this.life -= dt;
    if (this.life <= 0) this.dead = true;
  }
}
