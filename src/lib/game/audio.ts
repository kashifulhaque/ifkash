// All audio is synthesized with WebAudio — no asset downloads.
// A laid-back chiptune loop plus gunshot / hit / hurt effects.

const STORAGE_KEY = 'ifkash-game-muted';

export class GameAudio {
  private ctx: AudioContext | null = null;
  private musicGain: GainNode | null = null;
  private sfxGain: GainNode | null = null;
  private nextBarTime = 0;
  private bar = 0;
  private schedulerId: ReturnType<typeof setInterval> | null = null;
  muted = false;

  constructor() {
    try {
      this.muted = localStorage.getItem(STORAGE_KEY) === '1';
    } catch {
      /* private mode */
    }
  }

  // Must be called from a user gesture.
  start() {
    if (this.ctx) {
      this.ctx.resume();
      return;
    }
    this.ctx = new AudioContext();
    this.musicGain = this.ctx.createGain();
    this.musicGain.gain.value = this.muted ? 0 : 0.16;
    this.musicGain.connect(this.ctx.destination);
    this.sfxGain = this.ctx.createGain();
    this.sfxGain.gain.value = this.muted ? 0 : 0.5;
    this.sfxGain.connect(this.ctx.destination);

    this.nextBarTime = this.ctx.currentTime + 0.1;
    this.schedulerId = setInterval(() => this.schedule(), 200);
  }

  toggleMute(): boolean {
    this.muted = !this.muted;
    if (this.musicGain) this.musicGain.gain.value = this.muted ? 0 : 0.16;
    if (this.sfxGain) this.sfxGain.gain.value = this.muted ? 0 : 0.5;
    try {
      localStorage.setItem(STORAGE_KEY, this.muted ? '1' : '0');
    } catch {
      /* ignore */
    }
    return this.muted;
  }

  // ── Music: 2-bar lookahead scheduler, I–VI–IV–V progression ──
  private schedule() {
    if (!this.ctx || !this.musicGain) return;
    const BPM = 96;
    const barLen = (60 / BPM) * 4;
    while (this.nextBarTime < this.ctx.currentTime + barLen * 1.5) {
      this.scheduleBar(this.nextBarTime, this.bar);
      this.nextBarTime += barLen;
      this.bar++;
    }
  }

  private note(
    freq: number,
    time: number,
    dur: number,
    type: OscillatorType,
    gain: number,
    dest: AudioNode
  ) {
    const ctx = this.ctx!;
    const osc = ctx.createOscillator();
    const g = ctx.createGain();
    osc.type = type;
    osc.frequency.value = freq;
    g.gain.setValueAtTime(0, time);
    g.gain.linearRampToValueAtTime(gain, time + 0.02);
    g.gain.exponentialRampToValueAtTime(0.001, time + dur);
    osc.connect(g).connect(dest);
    osc.start(time);
    osc.stop(time + dur + 0.05);
  }

  private scheduleBar(t0: number, bar: number) {
    const dest = this.musicGain!;
    const beat = 60 / 96;
    // C maj pentatonic-ish progression: C, Am, F, G (roots in Hz)
    const roots = [130.81, 110.0, 87.31, 98.0];
    const root = roots[bar % 4];
    // Bass: root on 1 and 3
    this.note(root, t0, beat * 0.9, 'triangle', 0.5, dest);
    this.note(root, t0 + beat * 2, beat * 0.9, 'triangle', 0.4, dest);
    // Pad: fifth + octave, soft
    this.note(root * 1.5, t0, beat * 4, 'sine', 0.12, dest);
    this.note(root * 2, t0, beat * 4, 'sine', 0.1, dest);
    // Lead: sparse pentatonic noodling, deterministic per bar
    const penta = [1, 9 / 8, 5 / 4, 3 / 2, 5 / 3, 2];
    for (let i = 0; i < 4; i++) {
      const step = (bar * 7 + i * 3) % penta.length;
      if ((bar + i) % 3 === 0) continue; // leave gaps
      this.note(root * 2 * penta[step], t0 + beat * i, beat * 0.5, 'square', 0.07, dest);
    }
  }

  // ── SFX ──
  private noiseBurst(dur: number, filterFreq: number, gain: number) {
    if (!this.ctx || !this.sfxGain) return;
    const ctx = this.ctx;
    const len = Math.floor(ctx.sampleRate * dur);
    const buffer = ctx.createBuffer(1, len, ctx.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < len; i++) data[i] = (Math.random() * 2 - 1) * (1 - i / len);
    const src = ctx.createBufferSource();
    src.buffer = buffer;
    const filter = ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.value = filterFreq;
    const g = ctx.createGain();
    g.gain.value = gain;
    src.connect(filter).connect(g).connect(this.sfxGain);
    src.start();
  }

  gunshot() {
    this.noiseBurst(0.12, 1800, 0.7);
  }

  enemyShot() {
    this.noiseBurst(0.1, 900, 0.4);
  }

  hitConfirm() {
    if (!this.ctx || !this.sfxGain) return;
    this.note(880, this.ctx.currentTime, 0.08, 'square', 0.25, this.sfxGain);
    this.note(1320, this.ctx.currentTime + 0.06, 0.1, 'square', 0.2, this.sfxGain);
  }

  headshotConfirm() {
    if (!this.ctx || !this.sfxGain) return;
    this.note(1320, this.ctx.currentTime, 0.06, 'square', 0.25, this.sfxGain);
    this.note(1760, this.ctx.currentTime + 0.05, 0.1, 'square', 0.22, this.sfxGain);
  }

  reload() {
    if (!this.ctx || !this.sfxGain) return;
    const t = this.ctx.currentTime;
    // mag out, mag in, bolt
    this.note(220, t, 0.05, 'square', 0.18, this.sfxGain);
    this.note(180, t + 0.45, 0.05, 'square', 0.2, this.sfxGain);
    this.note(330, t + 0.95, 0.04, 'square', 0.22, this.sfxGain);
  }

  dryFire() {
    if (!this.ctx || !this.sfxGain) return;
    this.note(700, this.ctx.currentTime, 0.03, 'square', 0.15, this.sfxGain);
  }

  footstep(running: boolean) {
    this.noiseBurst(0.04, running ? 500 : 350, running ? 0.16 : 0.1);
  }

  streak(count: number) {
    if (!this.ctx || !this.sfxGain) return;
    const base = 440 * Math.pow(1.12, Math.min(count, 10));
    this.note(base, this.ctx.currentTime, 0.1, 'triangle', 0.25, this.sfxGain);
    this.note(base * 1.5, this.ctx.currentTime + 0.08, 0.12, 'triangle', 0.22, this.sfxGain);
  }

  playerHurt() {
    if (!this.ctx || !this.sfxGain) return;
    this.note(160, this.ctx.currentTime, 0.18, 'sawtooth', 0.3, this.sfxGain);
  }

  crateOpen() {
    if (!this.ctx || !this.sfxGain) return;
    const t = this.ctx.currentTime;
    [523, 659, 784, 1046].forEach((f, i) => this.note(f, t + i * 0.07, 0.18, 'triangle', 0.25, this.sfxGain!));
  }

  dispose() {
    if (this.schedulerId) clearInterval(this.schedulerId);
    this.ctx?.close();
    this.ctx = null;
  }
}
