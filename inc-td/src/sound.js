// Audio: looping background music + one-shot SFX (arrow launch, enemy hit).
//
// Short SFX go through the Web Audio API so many can overlap without cutting each
// other off (arrows/hits fire rapidly). The music is a streamed <audio> element on
// loop. Everything is gated behind a user gesture — browsers block autoplay until
// the player first interacts, so call sound.unlock() from a click/keydown handler.
//
// The module is import-safe in a DOM-less environment (e.g. the headless test
// harness): if there's no AudioContext, every method is a silent no-op.

const A = 'assets/';
const SFX = { arrow: A + 'arrow.mp4', hit: A + 'hit.mp4' };
const MUSIC = A + '8-loop.mp4';

const AudioCtx = typeof window !== 'undefined' && (window.AudioContext || window.webkitAudioContext);

class Sound {
  constructor() {
    this.enabled = !!AudioCtx;
    this.muted = false;
    this.unlocked = false;
    this.buffers = {}; // name -> decoded AudioBuffer
    this.ctx = null;
    this.sfxGain = null;
    this.music = null; // HTMLAudioElement
  }

  // Kick off decoding of the SFX buffers and prepare the music element. Safe to
  // call before any user gesture; playback itself waits for unlock().
  load() {
    if (!this.enabled || this.ctx) return;
    this.ctx = new AudioCtx();
    this.sfxGain = this.ctx.createGain();
    this.sfxGain.gain.value = 0.6;
    this.sfxGain.connect(this.ctx.destination);
    for (const [name, url] of Object.entries(SFX)) {
      fetch(url)
        .then((r) => r.arrayBuffer())
        .then((buf) => this.ctx.decodeAudioData(buf))
        .then((decoded) => { this.buffers[name] = decoded; })
        .catch(() => {}); // a missing/undecodable clip just stays silent
    }
    this.music = new Audio(MUSIC);
    this.music.loop = true;
    this.music.volume = 0.35;
  }

  // Install one-shot, capture-phase listeners so the FIRST user interaction
  // anywhere on the page (button, key, background — not just the canvas) unlocks
  // audio. Capture phase means nothing downstream can swallow the gesture first.
  arm() {
    if (!this.enabled) return;
    const events = ['pointerdown', 'mousedown', 'touchstart', 'keydown', 'click'];
    const fire = () => {
      this.unlock();
      for (const ev of events) window.removeEventListener(ev, fire, true);
    };
    for (const ev of events) window.addEventListener(ev, fire, { capture: true });
  }

  // Resume the (initially suspended) context and start the music. Must be driven
  // by a real user gesture. Idempotent.
  unlock() {
    if (!this.enabled) return;
    this.load();
    if (this.ctx.state === 'suspended') this.ctx.resume();
    if (this.unlocked) return;
    this.unlocked = true;
    if (!this.muted) this.music.play().catch(() => {});
  }

  play(name) {
    if (!this.enabled || this.muted || !this.unlocked) return;
    const buf = this.buffers[name];
    if (!buf) return;
    const src = this.ctx.createBufferSource();
    src.buffer = buf;
    src.connect(this.sfxGain);
    src.start(); // a fresh source each time, so overlapping plays don't cut off
  }

  arrow() { this.play('arrow'); }
  hit() { this.play('hit'); }

  // Returns the new muted state so callers can update their button.
  toggleMute() {
    this.muted = !this.muted;
    if (this.music) {
      if (this.muted) this.music.pause();
      else if (this.unlocked) this.music.play().catch(() => {});
    }
    return this.muted;
  }
}

export const sound = new Sound();
