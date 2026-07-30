// Debug / sandbox panel. Toggle with the backtick (`) key.
//
// Lets you edit tower stats live, grant traits, flip Sandbox (no auto round-end,
// so waves are summoned by hand) and God mode, and spawn/clear waves on demand.
// Entirely self-contained: remove the initDebug() call in main.js to drop it.

const num = (v, fallback) => {
  const n = parseFloat(v);
  return Number.isFinite(n) ? n : fallback;
};

function injectStyles() {
  const css = `
    #debug-panel {
      position: fixed; top: 12px; right: 12px; z-index: 50;
      width: 230px; padding: 12px 14px;
      background: #12181c; color: #d8e6ee;
      border: 1px solid #38505c; border-radius: 8px;
      font: 12px/1.4 ui-monospace, SFMono-Regular, Menlo, monospace;
      box-shadow: 0 6px 24px rgba(0,0,0,0.5);
    }
    #debug-panel.hidden { display: none; }
    #debug-panel h3 { margin: 0 0 8px; font-size: 12px; color: #6fd3ff; letter-spacing: .5px; }
    #debug-panel .sect { margin-top: 10px; border-top: 1px solid #26363f; padding-top: 8px; }
    #debug-panel .sect > span.head { display:block; color:#8aa; margin-bottom:6px; text-transform:uppercase; font-size:10px; letter-spacing:.6px; }
    #debug-panel label { display:flex; align-items:center; justify-content:space-between; gap:8px; margin:4px 0; }
    #debug-panel input[type=number] { width: 80px; background:#0c1114; color:#d8e6ee; border:1px solid #38505c; border-radius:4px; padding:2px 5px; font:inherit; }
    #debug-panel input[type=checkbox] { transform: translateY(1px); }
    #debug-panel .row { display:flex; gap:6px; flex-wrap:wrap; margin-top:6px; }
    #debug-panel button { flex:1 1 auto; background:#1d3a48; color:#d8e6ee; border:1px solid #38505c; border-radius:5px; padding:5px 6px; font:inherit; cursor:pointer; }
    #debug-panel button:hover { background:#275367; }
    #debug-panel .hint { margin-top:10px; color:#5a7581; font-size:10px; }
  `;
  const style = document.createElement('style');
  style.textContent = css;
  document.head.appendChild(style);
}

export function initDebug(game) {
  injectStyles();
  const panel = document.createElement('div');
  panel.id = 'debug-panel';
  panel.className = 'hidden';
  panel.innerHTML = `
    <h3>DEBUG</h3>
    <div class="sect">
      <span class="head">Tower stats</span>
      <label>Max HP <input id="dbg-maxhp" type="number" step="5"></label>
      <label>HP <input id="dbg-hp" type="number" step="5"></label>
      <label>Damage <input id="dbg-damage" type="number" step="1"></label>
      <label>Fire rate <input id="dbg-firerate" type="number" step="0.1"></label>
      <div class="row"><button id="dbg-heal">Heal full</button></div>
    </div>
    <div class="sect">
      <span class="head">Modes & traits</span>
      <label>Sandbox (manual waves) <input id="dbg-sandbox" type="checkbox"></label>
      <label>God mode <input id="dbg-god" type="checkbox"></label>
      <div class="row"><button id="dbg-multishot">Grant Multi-shot</button></div>
      <div class="row">
        <button id="dbg-frost">Grant Frost</button>
        <button id="dbg-fire">Grant Fire</button>
      </div>
    </div>
    <div class="sect">
      <span class="head">Waves</span>
      <label>Round (scaling) <input id="dbg-round" type="number" step="1" min="1"></label>
      <label>Enemy count <input id="dbg-count" type="number" step="1" min="1" value="5"></label>
      <div class="row">
        <button id="dbg-spawn">Spawn wave</button>
        <button id="dbg-spawn-green">Spawn green</button>
      </div>
      <div class="row">
        <button id="dbg-spawn-large">Spawn large</button>
        <button id="dbg-spawn-large-green">Spawn lg green</button>
      </div>
      <div class="row">
        <button id="dbg-spawn-goblin">Spawn goblin</button>
      </div>
      <div class="row">
        <button id="dbg-next">Next round</button>
      </div>
      <div class="row">
        <button id="dbg-clear">Clear</button>
        <button id="dbg-kill">Kill all</button>
        <button id="dbg-restart">Restart</button>
      </div>
    </div>
    <div class="hint">Press \` to toggle this panel</div>
  `;
  document.body.appendChild(panel);
  const q = (sel) => panel.querySelector(sel);

  const t = () => game.tower;
  function sync() {
    q('#dbg-maxhp').value = t().maxHp;
    q('#dbg-hp').value = Math.round(t().hp);
    q('#dbg-damage').value = t().damage;
    q('#dbg-firerate').value = t().fireRate;
    q('#dbg-sandbox').checked = game.sandbox;
    q('#dbg-god').checked = game.godMode;
    q('#dbg-round').value = game.round;
  }

  // Stat edits write straight through to the live tower.
  q('#dbg-maxhp').addEventListener('input', (e) => { t().maxHp = Math.max(1, num(e.target.value, t().maxHp)); });
  q('#dbg-hp').addEventListener('input', (e) => { t().hp = num(e.target.value, t().hp); });
  q('#dbg-damage').addEventListener('input', (e) => { t().damage = Math.max(0, num(e.target.value, t().damage)); });
  q('#dbg-firerate').addEventListener('input', (e) => { t().fireRate = Math.max(0.05, num(e.target.value, t().fireRate)); });

  q('#dbg-heal').addEventListener('click', () => { t().hp = t().maxHp; sync(); });
  q('#dbg-sandbox').addEventListener('change', (e) => { game.setSandbox(e.target.checked); sync(); });
  q('#dbg-god').addEventListener('change', (e) => { game.godMode = e.target.checked; });
  q('#dbg-multishot').addEventListener('click', () => { game.grantTrait('multishot'); sync(); });
  q('#dbg-frost').addEventListener('click', () => { game.grantTrait('frost'); sync(); });
  q('#dbg-fire').addEventListener('click', () => { game.grantTrait('fire'); sync(); });

  q('#dbg-round').addEventListener('input', (e) => { game.round = Math.max(1, Math.round(num(e.target.value, game.round))); });
  const spawn = (type) => {
    const count = Math.max(1, Math.round(num(q('#dbg-count').value, 5)));
    const round = Math.max(1, Math.round(num(q('#dbg-round').value, game.round)));
    game.spawnWave(count, type, round);
  };
  q('#dbg-spawn').addEventListener('click', () => spawn('slime'));
  q('#dbg-spawn-green').addEventListener('click', () => spawn('greenslime'));
  q('#dbg-spawn-large').addEventListener('click', () => spawn('largeslime'));
  q('#dbg-spawn-large-green').addEventListener('click', () => spawn('largegreenslime'));
  q('#dbg-spawn-goblin').addEventListener('click', () => spawn('goblin'));
  q('#dbg-next').addEventListener('click', () => { game.startRound(game.round + 1); sync(); });
  q('#dbg-clear').addEventListener('click', () => game.clearEnemies());
  q('#dbg-kill').addEventListener('click', () => game.killEnemies());
  q('#dbg-restart').addEventListener('click', () => { game.reset(); sync(); });

  window.addEventListener('keydown', (e) => {
    if (e.key !== '`') return;
    const tag = e.target && e.target.tagName;
    if (tag === 'INPUT' || tag === 'SELECT' || tag === 'TEXTAREA') return; // don't steal input focus
    panel.classList.toggle('hidden');
    if (!panel.classList.contains('hidden')) sync();
  });

  sync();
  return { panel, sync };
}
