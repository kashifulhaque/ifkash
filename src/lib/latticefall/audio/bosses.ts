// Boss music: an intro swell, a loop stem with the boss's beat and bass drone, and a phase-2
// stem (an arpeggio) layered over the loop. Most bosses schedule their beats on the audio clock.
// The Bellwright (`fromEvents`) plays each beat when the core reports it (`BELL_BEAT`), so the
// music can't drift from the fight. Everything is synthesized; there are no samples.

import { bellBeat, midiHz, noiseBurst, tone } from './synth';
import { BOSS_VOICES, type BeatStyle, type BossKey, type BossVoice } from './voices';

export type BossMusicState = 'off' | 'intro' | 'loop' | 'phase2';

const INTRO_SECS = 3;
const LOOKAHEAD = 0.2;

export class BossMusic {
  private out: GainNode;
  private loopStem: GainNode;
  private phase2Stem: GainNode;
  private voice: BossVoice = BOSS_VOICES.default;
  private state: BossMusicState = 'off';
  /** The state to enter once the intro's swell has played out. */
  private queued: BossMusicState | null = null;
  private introEnd = 0;
  private timer: ReturnType<typeof setInterval> | null = null;
  private nextBeat = 0;
  private beatIndex = 0;
  private eventBeats = 0;

  constructor(
    private ctx: AudioContext,
    music: AudioNode,
    private noise: AudioBuffer
  ) {
    this.out = ctx.createGain();
    this.out.gain.value = 0;
    this.out.connect(music);
    this.loopStem = ctx.createGain();
    this.loopStem.gain.value = 0;
    this.loopStem.connect(this.out);
    this.phase2Stem = ctx.createGain();
    this.phase2Stem.gain.value = 0;
    this.phase2Stem.connect(this.out);
  }

  get current(): BossMusicState {
    return this.state;
  }

  /** Picks the boss's music. Call it while the music is off, before the intro. */
  setVoice(key: BossKey): void {
    this.voice = BOSS_VOICES[key];
  }

  set(state: BossMusicState): void {
    const t = this.ctx.currentTime;
    if (this.state === 'intro' && state !== 'off' && t < this.introEnd) {
      // The swell plays out first; the loop or phase 2 takes over after it.
      this.queued = state;
      return;
    }
    this.queued = null;
    if (state === this.state) return;
    this.apply(state, t);
  }

  private apply(state: BossMusicState, t: number): void {
    const prev = this.state;
    this.state = state;
    const ramp = (p: AudioParam, v: number, secs: number) => {
      p.cancelScheduledValues(t);
      p.setValueAtTime(p.value, t);
      p.linearRampToValueAtTime(v, t + secs);
    };
    if (state === 'off') {
      ramp(this.out.gain, 0, 2);
      ramp(this.loopStem.gain, 0, 2);
      ramp(this.phase2Stem.gain, 0, 2);
      setTimeout(() => {
        if (this.state === 'off') this.stopTimer();
      }, 2200);
      return;
    }
    ramp(this.out.gain, 0.9, 0.3);
    if (state === 'intro') {
      // A rising swell over the drone; the loop starts when it ends.
      const d = this.voice.drone;
      tone(this.ctx, this.out, t, { type: 'sawtooth', freq: midiHz(d - 7), to: midiHz(d + 5), dur: INTRO_SECS + 0.5, gain: 0.15, attack: 2.5 });
      tone(this.ctx, this.out, t, { type: 'sine', freq: midiHz(d), dur: INTRO_SECS + 0.5, gain: 0.25, attack: 1.5 });
      ramp(this.loopStem.gain, 0, 0.1);
      ramp(this.phase2Stem.gain, 0, 0.1);
      this.introEnd = t + INTRO_SECS;
      this.startTimer();
      return;
    }
    // Entering the loop from nothing or from the intro starts the beat grid on this moment.
    if (prev === 'off' || prev === 'intro') {
      this.nextBeat = t;
      this.beatIndex = 0;
      this.eventBeats = 0;
    }
    ramp(this.loopStem.gain, 1, prev === 'intro' ? 0.5 : 1);
    ramp(this.phase2Stem.gain, state === 'phase2' ? 1 : 0, 2);
    this.startTimer();
  }

  /** Plays the Bellwright's beat `index` (0 to 3; 0 is the toll's beat). Ignored by other bosses. */
  onBellBeat(index: number): void {
    if (!this.voice.fromEvents || (this.state !== 'loop' && this.state !== 'phase2')) return;
    this.beat(this.ctx.currentTime, index % 4, this.eventBeats++, this.secondsPerBeat());
  }

  private secondsPerBeat(): number {
    const speed = this.state === 'phase2' ? this.voice.phase2Speed : 1;
    return 60 / (this.voice.bpm * speed);
  }

  private startTimer(): void {
    if (!this.timer) this.timer = setInterval(() => this.schedule(), 50);
  }

  private stopTimer(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
  }

  private schedule(): void {
    const now = this.ctx.currentTime;
    if (this.state === 'intro' && now >= this.introEnd) {
      const next = this.queued ?? 'loop';
      this.queued = null;
      this.apply(next, this.introEnd);
    }
    if ((this.state !== 'loop' && this.state !== 'phase2') || this.voice.fromEvents) return;
    const spb = this.secondsPerBeat();
    while (this.nextBeat < now + LOOKAHEAD) {
      const t = Math.max(now, this.nextBeat);
      this.beat(t, this.beatIndex % 4, this.beatIndex, spb);
      this.nextBeat += spb;
      this.beatIndex++;
    }
  }

  /** One beat: the bar's percussion and drone on the loop stem, and the arpeggio in phase 2. */
  private beat(t: number, bar: number, count: number, spb: number): void {
    const v = this.voice;
    percussion(this.ctx, this.loopStem, this.noise, t, v.beat, bar, spb, v.drone);
    if (bar === 0) tone(this.ctx, this.loopStem, t, { type: 'triangle', freq: midiHz(v.drone), dur: spb * 3.6, gain: 0.2, attack: 0.1 });
    if (this.state !== 'phase2') return;
    const note = (k: number) => midiHz(v.drone + 24 + v.lead[k % v.lead.length]);
    tone(this.ctx, this.phase2Stem, t, { type: v.leadWave, freq: note(count), dur: spb * 0.45, gain: 0.07 });
    // The off-beat note sits an octave above the arpeggio's chord tone two steps on.
    tone(this.ctx, this.phase2Stem, t + spb / 2, { type: v.leadWave, freq: 2 * note(count + 2), dur: spb * 0.4, gain: 0.05 });
  }

  dispose(): void {
    this.stopTimer();
    this.queued = null;
    this.out.disconnect();
  }
}

function kick(ctx: BaseAudioContext, dest: AudioNode, t: number, gain: number): void {
  tone(ctx, dest, t, { freq: 70, to: 38, dur: 0.3, gain });
}

function snare(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number): void {
  noiseBurst(ctx, dest, noise, t, { dur: 0.12, gain: 0.25, type: 'bandpass', freq: 1800, q: 1 });
}

function hat(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number): void {
  noiseBurst(ctx, dest, noise, t, { dur: 0.03, gain: 0.08, type: 'highpass', freq: 6000, q: 0.7 });
}

/** The percussion for one beat of a boss's style. `bar` is the beat in the bar, 0 to 3. */
function percussion(
  ctx: BaseAudioContext,
  dest: AudioNode,
  noise: AudioBuffer,
  t: number,
  style: BeatStyle,
  bar: number,
  spb: number,
  drone: number
): void {
  switch (style) {
    case 'heavy':
      kick(ctx, dest, t, bar === 0 ? 0.7 : 0.45);
      if (bar === 2) snare(ctx, dest, noise, t);
      break;
    case 'skitter':
      hat(ctx, dest, noise, t);
      hat(ctx, dest, noise, t + spb / 2);
      if (bar === 0) kick(ctx, dest, t, 0.5);
      if (bar === 1 || bar === 3) snare(ctx, dest, noise, t);
      break;
    case 'chant':
      if (bar === 0 || bar === 2) {
        kick(ctx, dest, t, 0.5);
        tone(ctx, dest, t, { type: 'triangle', freq: midiHz(drone + 12), dur: spb * 1.8, gain: 0.12, attack: 0.3 });
      }
      break;
    case 'drip': {
      const steps = [0, 7, 3, 10];
      tone(ctx, dest, t + spb / 2, { type: 'sine', freq: midiHz(drone + 24 + steps[bar]), dur: 0.4, gain: 0.09 });
      break;
    }
    case 'clatter':
      if (bar === 0 || bar === 3) noiseBurst(ctx, dest, noise, t, { dur: 0.04, gain: 0.3, type: 'bandpass', freq: 2500, q: 3 });
      if (Math.random() < 0.3) tone(ctx, dest, t + spb / 2, { type: 'sawtooth', freq: 200, to: 90, dur: 0.08, gain: 0.12 });
      break;
    case 'pulse':
      tone(ctx, dest, t, { type: 'square', freq: midiHz(drone + 12), dur: spb * 0.35, gain: 0.06 });
      tone(ctx, dest, t + spb / 2, { type: 'square', freq: midiHz(drone + 12), dur: spb * 0.35, gain: 0.05 });
      break;
    case 'toll':
      bellBeat(ctx, dest, t, bar === 0);
      break;
    case 'swell':
      break;
  }
}
