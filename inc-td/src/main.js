// Bootstrap: grab the canvas, build the game, start the loop.

import { Game } from './game.js';
import { initDebug } from './debug.js';
import { sound } from './sound.js';

const canvas = document.getElementById('game');
const ctx = canvas.getContext('2d');

const game = new Game(canvas, ctx);
initDebug(game); // press ` to open the debug/sandbox panel
window.game = game; // handy for tinkering from the console

// Audio can't autoplay — start it on the player's first interaction anywhere.
sound.load();
sound.arm();

// Mute toggle (music + SFX).
const muteBtn = document.getElementById('mute-toggle');
muteBtn.addEventListener('click', () => {
  const muted = sound.toggleMute();
  muteBtn.textContent = muted ? '🔇 Muted' : '🔊 Sound';
  muteBtn.classList.toggle('muted', muted);
});

// Toggle 2x simulation speed.
const speedBtn = document.getElementById('speed-toggle');
speedBtn.addEventListener('click', () => {
  game.speed = game.speed === 1 ? 2 : 1;
  speedBtn.textContent = `Speed ${game.speed}×`;
  speedBtn.classList.toggle('fast', game.speed === 2);
});

game.start();
