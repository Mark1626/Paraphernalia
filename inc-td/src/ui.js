// HUD + overlay DOM. Kept apart from canvas rendering.

import { REWARDS } from './config.js';

const el = (id) => document.getElementById(id);

export function updateHud(state) {
  const t = state.tower;
  el('hud-round').textContent = `Round ${state.round}`;
  el('hud-hp').textContent = `HP ${Math.max(0, Math.ceil(t.hp))}/${t.maxHp}`;
  // Per-round stopwatch: wall-clock vs simulated time (they diverge at 2x speed).
  el('hud-time').textContent = `⏱ ${state.roundWallTime.toFixed(1)}s wall · ${state.roundGameTime.toFixed(1)}s game`;
  // Total wall-clock time played across the whole run (mm:ss).
  el('hud-total').textContent = `🕒 Total ${fmtClock(state.totalWallTime)}`;
  el('stat-damage').textContent = t.damage.toFixed(0);
  el('stat-firerate').textContent = t.fireRate.toFixed(2);

  // Multi-shot readout: proc chance, extra-arrow count, and their damage (each
  // shown separately). Dashes until the trait is owned.
  // Multi-shot: proc / extra-arrow count / extra-arrow damage on one line.
  const ms = t.traitState.multishot;
  el('stat-ms').textContent = ms
    ? `${Math.round(ms.chance * 100)}% / ${ms.count} / ${Math.round(ms.damageMul * 100)}%`
    : '—';

  // Frost / Fire readouts (compact proc/effect/duration), dashes until owned.
  const fr = t.traitState.frost;
  el('stat-frost').textContent = fr
    ? `${Math.round(fr.chance * 100)}% / ${Math.round(fr.slowPct * 100)}% / ${fr.duration.toFixed(1)}s`
    : '—';
  const fi = t.traitState.fire;
  el('stat-fire').textContent = fi
    ? `${Math.round(fi.chance * 100)}% / ${Math.round(fi.dps)} / ${fi.duration.toFixed(1)}s`
    : '—';

  el('stat-traits').textContent = t.traits.length ? t.traits.join(', ') : '—';

  updateProgress(state.round);
}

// Seconds -> m:ss.
function fmtClock(seconds) {
  const s = Math.max(0, Math.floor(seconds));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, '0')}`;
}

// Batch progress bar: the 5 markers show the current block of rounds (e.g. 6–10).
// Passed rounds are filled, the current round is highlighted, the rest are dim.
function updateProgress(round) {
  const batchStart = Math.floor((round - 1) / 5) * 5 + 1; // 1, 6, 11, ...
  const marks = el('progress').children;
  for (let i = 0; i < marks.length; i++) {
    const r = batchStart + i;
    const m = marks[i];
    m.firstElementChild.textContent = r; // the round number label
    m.classList.toggle('done', r < round);
    m.classList.toggle('current', r === round);
  }
}

// Show the reward-choice cards. `offers` is [{ reward, golden }]; `onPick(reward,
// golden)` is called with the chosen entry. `round` lets a reward's `desc` be a
// function of (round, mult) that shows the exact (round-scaled, golden-boosted)
// amount being offered.
export function showRewards(offers, isTrait, round, onPick) {
  const overlay = el('overlay');
  const cards = el('cards');
  el('overlay-title').textContent = isTrait ? 'Choose a Trait' : 'Choose an Upgrade';
  cards.innerHTML = '';
  for (const { reward: r, golden } of offers) {
    const card = document.createElement('button');
    card.className = 'card' + (isTrait ? ' trait' : golden ? ' golden' : ' silver');
    const mult = golden ? REWARDS.goldenMult : 1;
    const desc = typeof r.desc === 'function' ? r.desc(round, mult) : r.desc;
    const badge = isTrait
      ? ''
      : golden
        ? `<span class="badge gold">✦ GOLDEN ×${REWARDS.goldenMult}</span>`
        : '<span class="badge">SILVER</span>';
    card.innerHTML = `${badge}<h3>${r.name}</h3><p>${desc}</p>`;
    card.addEventListener('click', () => onPick(r, golden), { once: true });
    cards.appendChild(card);
  }
  overlay.classList.remove('hidden');
}

export function hideRewards() {
  el('overlay').classList.add('hidden');
}

export function showGameOver(round, onRestart) {
  const overlay = el('overlay');
  el('overlay-title').textContent = `Game Over — reached round ${round}`;
  const cards = el('cards');
  cards.innerHTML = '';
  const btn = document.createElement('button');
  btn.className = 'card';
  btn.innerHTML = '<h3>Restart</h3><p>Start again from round 1</p>';
  btn.addEventListener('click', onRestart, { once: true });
  cards.appendChild(btn);
  overlay.classList.remove('hidden');
}
