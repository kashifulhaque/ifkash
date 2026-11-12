// Procedural ambient beds (one per region, on the ambience bus) and the music stems (on the
// music bus): a calm exploration pad per region, and boss music with an intro, a loop, and
// a phase-2 stem layered in. All placeholders, synthesized with no assets.

import { midiHz, noiseBurst, tone } from './synth';

/** The ambience layers the spec names. Each region mixes them at its own levels (0 to 1). */
export interface BedMix {
  wind: number;
  water: number;
  drips: number;
  fire: number;
}

/** Pad root (MIDI) and darkness for the region's exploration loop. */
export interface RegionVoice {
  mix: BedMix;
  root: number;
  chord: number[];
  tone: number;
}

export const REGION_VOICES: Record<string, RegionVoice> = {
  // Waking Hollow: a sheltered start, light wind and a trickle of water.
  'waking-hollow': { mix: { wind: 0.35, water: 0.3, drips: 0.15, fire: 0 }, root: 50, chord: [0, 7, 12, 16], tone: 900 },
  // The Verge: open and windy.
  verge: { mix: { wind: 0.7, water: 0.1, drips: 0, fire: 0 }, root: 53, chord: [0, 7, 12, 15], tone: 700 },
  // Hollowhearth (WP-068): the safe hub, a crackling hearth and almost no wind.
  hollowhearth: { mix: { wind: 0.05, water: 0, drips: 0.05, fire: 0.6 }, root: 48, chord: [0, 7, 12, 16], tone: 1100 },
  // Used when a region has no voice of its own, and on the test field.
  default: { mix: { wind: 0.3, water: 0, drips: 0, fire: 0 }, root: 50, chord: [0, 7, 12], tone: 800 }
};

const LOOKAHEAD = 0.25;

/** A looping filtered-noise layer with a slow gain wobble. */
class NoiseLayer {
  readonly gain: GainNode;
  private src: AudioBufferSourceNode;
  private lfo: OscillatorNode;

  constructor(ctx: AudioContext, dest: AudioNode, noise: AudioBuffer, o: { type: BiquadFilterType; freq: number; q: number; lfoHz: number; depth: number }) {
    this.src = ctx.createBufferSource();
    this.src.buffer = noise;
    this.src.loop = true;
    const f = ctx.createBiquadFilter();
    f.type = o.type;
    f.frequency.value = o.freq;
    f.Q.value = o.q;
    const wobble = ctx.createGain();
    wobble.gain.value = 1 - o.depth;
    this.lfo = ctx.createOscillator();
    this.lfo.frequency.value = o.lfoHz;
    const depth = ctx.createGain();
    depth.gain.value = o.depth;
    this.lfo.connect(depth).connect(wobble.gain);
    this.gain = ctx.createGain();
    this.gain.gain.value = 0;
    this.src.connect(f).connect(wobble).connect(this.gain).connect(dest);
    this.src.start();
    this.lfo.start();
  }

  fadeTo(ctx: AudioContext, level: number, secs: number): void {
    const p = this.gain.gain;
    p.cancelScheduledValues(ctx.currentTime);
    p.setValueAtTime(p.value, ctx.currentTime);
    p.linearRampToValueAtTime(level, ctx.currentTime + secs);
  }

  stop(): void {
    try {
      this.src.stop();
      this.lfo.stop();
    } catch {
      // Already stopped.
    }
    this.gain.disconnect();
  }
}

/** Region ambience: wind, water, cave drips, and fire, plus the calm exploration pad. */
export class AmbienceBed {
  private wind: NoiseLayer;
  private water: NoiseLayer;
  private fire: NoiseLayer;
  private padOut: GainNode | null = null;
  private padOscs: OscillatorNode[] = [];
  private mix: BedMix = { wind: 0, water: 0, drips: 0, fire: 0 };
  private timer: ReturnType<typeof setInterval> | null = null;
  private nextDrip = 0;
  private nextCrackle = 0;
  region = '';

  constructor(
    private ctx: AudioContext,
    private ambience: AudioNode,
    private music: AudioNode,
    private noise: AudioBuffer
  ) {
    this.wind = new NoiseLayer(ctx, ambience, noise, { type: 'bandpass', freq: 500, q: 0.6, lfoHz: 0.11, depth: 0.5 });
    this.water = new NoiseLayer(ctx, ambience, noise, { type: 'bandpass', freq: 2600, q: 0.4, lfoHz: 0.7, depth: 0.3 });
    this.fire = new NoiseLayer(ctx, ambience, noise, { type: 'lowpass', freq: 260, q: 0.7, lfoHz: 0.4, depth: 0.4 });
    this.timer = setInterval(() => this.schedule(), 100);
  }

  /** Crossfades to a region's bed. Unknown ids use the default voice. */
  setRegion(id: string): void {
    if (id === this.region) return;
    this.region = id;
    const v = REGION_VOICES[id] ?? REGION_VOICES.default;
    const t = this.ctx.currentTime;
    this.mix = v.mix;
    this.wind.fadeTo(this.ctx, v.mix.wind * 0.5, 3);
    this.water.fadeTo(this.ctx, v.mix.water * 0.25, 3);
    this.fire.fadeTo(this.ctx, v.mix.fire * 0.5, 3);
    // Retire the old pad, then start a new one on its own filter and gain.
    const old = this.padOscs;
    if (this.padOut) {
      const g = this.padOut.gain;
      g.cancelScheduledValues(t);
      g.setValueAtTime(g.value, t);
      g.linearRampToValueAtTime(0, t + 3);
    }
    setTimeout(() => {
      for (const o of old) {
        try {
          o.stop();
        } catch {
          // Already stopped.
        }
      }
    }, 3200);
    const filter = this.ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.value = v.tone;
    const out = this.ctx.createGain();
    out.gain.setValueAtTime(0, t);
    out.gain.linearRampToValueAtTime(0.06, t + 3);
    filter.connect(out).connect(this.music);
    this.padOut = out;
    this.padOscs = v.chord.map((semi, i) => {
      const osc = this.ctx.createOscillator();
      osc.type = i % 2 ? 'triangle' : 'sine';
      osc.frequency.value = midiHz(v.root + semi);
      osc.detune.value = (i - 1.5) * 4;
      osc.connect(filter);
      osc.start(t);
      return osc;
    });
  }

  private schedule(): void {
    const now = this.ctx.currentTime;
    if (this.mix.drips > 0 && now + LOOKAHEAD > this.nextDrip) {
      const t = Math.max(now, this.nextDrip);
      tone(this.ctx, this.ambience, t, { freq: 1400 + Math.random() * 900, to: 700, dur: 0.18, gain: 0.08 * this.mix.drips * 4 });
      this.nextDrip = t + 0.6 + Math.random() * 2.4 / Math.max(0.1, this.mix.drips);
    }
    if (this.mix.fire > 0 && now + LOOKAHEAD > this.nextCrackle) {
      const t = Math.max(now, this.nextCrackle);
      noiseBurst(this.ctx, this.ambience, this.noise, t, { dur: 0.03, gain: 0.25 * this.mix.fire, type: 'highpass', freq: 2500 });
      this.nextCrackle = t + 0.05 + Math.random() * 0.35;
    }
  }

  dispose(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
    this.wind.stop();
    this.water.stop();
    this.fire.stop();
    for (const o of this.padOscs) {
      try {
        o.stop();
      } catch {
        // Already stopped.
      }
    }
  }
}

export type BossMusicState = 'off' | 'intro' | 'loop' | 'phase2';

const BOSS_BPM = 84;

/**
 * Boss music: an intro swell, a looping pulse, and a phase-2 stem that fades in over the
 * loop. The pulse is scheduled on the audio clock so the fight's beat stays steady.
 */
export class BossMusic {
  private out: GainNode;
  private loopStem: GainNode;
  private phase2Stem: GainNode;
  private state: BossMusicState = 'off';
  private timer: ReturnType<typeof setInterval> | null = null;
  private nextBeat = 0;
  private beatIndex = 0;

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

  set(state: BossMusicState): void {
    if (state === this.state) return;
    const t = this.ctx.currentTime;
    const ramp = (p: AudioParam, v: number, secs: number) => {
      p.cancelScheduledValues(t);
      p.setValueAtTime(p.value, t);
      p.linearRampToValueAtTime(v, t + secs);
    };
    const prev = this.state;
    this.state = state;
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
      // Intro stem: a rising swell with a low drone, then the loop takes over.
      tone(this.ctx, this.out, t, { type: 'sawtooth', freq: midiHz(31), to: midiHz(43), dur: 3.5, gain: 0.15, attack: 2.5 });
      tone(this.ctx, this.out, t, { type: 'sine', freq: midiHz(38), dur: 3.5, gain: 0.25, attack: 1.5 });
      ramp(this.loopStem.gain, 0, 0.1);
      ramp(this.phase2Stem.gain, 0, 0.1);
      this.startTimer(t + 3);
      return;
    }
    if (prev === 'off') this.startTimer(t);
    ramp(this.loopStem.gain, 1, state === 'loop' && prev === 'intro' ? 0.5 : 1);
    ramp(this.phase2Stem.gain, state === 'phase2' ? 1 : 0, 2);
  }

  private startTimer(firstBeat: number): void {
    this.nextBeat = firstBeat;
    this.beatIndex = 0;
    if (!this.timer) this.timer = setInterval(() => this.schedule(), 50);
  }

  private stopTimer(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
  }

  private schedule(): void {
    const spb = 60 / BOSS_BPM;
    const now = this.ctx.currentTime;
    while (this.nextBeat < now + 0.2) {
      const t = Math.max(now, this.nextBeat);
      const beat = this.beatIndex % 4;
      // Loop stem: a drum on every beat (accented on the bar) and a low bass note.
      tone(this.ctx, this.loopStem, t, { freq: beat === 0 ? 70 : 55, to: 38, dur: 0.3, gain: beat === 0 ? 0.7 : 0.45 });
      if (beat === 2) noiseBurst(this.ctx, this.loopStem, this.noise, t, { dur: 0.12, gain: 0.25, type: 'bandpass', freq: 1800, q: 1 });
      if (beat === 0) tone(this.ctx, this.loopStem, t, { type: 'triangle', freq: midiHz(38), dur: spb * 3.6, gain: 0.2, attack: 0.1 });
      // Phase-2 stem: an eighth-note arpeggio over the loop.
      const arp = [62, 65, 69, 72];
      tone(this.ctx, this.phase2Stem, t, { type: 'square', freq: midiHz(arp[beat]), dur: spb * 0.45, gain: 0.07 });
      tone(this.ctx, this.phase2Stem, t + spb / 2, { type: 'square', freq: midiHz(arp[(beat + 2) % 4] + 12), dur: spb * 0.4, gain: 0.05 });
      this.nextBeat += spb;
      this.beatIndex++;
    }
  }

  dispose(): void {
    this.stopTimer();
    this.out.disconnect();
  }
}
