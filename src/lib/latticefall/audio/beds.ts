// Procedural region beds: the ambience layers (wind, water, drips, fire) on the ambience bus,
// and the calm exploration music on the music bus: a pad under a plucked loop. The numbers for
// each region live in `voices.ts`. All of it is synthesized, with no assets.

import { midiHz, noiseBurst, tone } from './synth';
import { regionVoice, type BedMix, type RegionVoice } from './voices';

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

/** Region ambience: wind, water, cave drips, and fire, plus the calm exploration music. */
export class AmbienceBed {
  private wind: NoiseLayer;
  private water: NoiseLayer;
  private fire: NoiseLayer;
  private padOut: GainNode | null = null;
  private padOscs: OscillatorNode[] = [];
  private voice: RegionVoice | null = null;
  private mix: BedMix = { wind: 0, water: 0, drips: 0, fire: 0 };
  private timer: ReturnType<typeof setInterval> | null = null;
  private nextDrip = 0;
  private nextCrackle = 0;
  private nextPluck = 0;
  private pluckStep = 0;
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
    const v = regionVoice(id);
    const t = this.ctx.currentTime;
    this.voice = v;
    this.mix = v.mix;
    this.nextPluck = t;
    this.pluckStep = 0;
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
    this.schedulePluck(now);
  }

  /** The exploration loop: one step per eighth note, looping its bar. Rests and empty bars are silent. */
  private schedulePluck(now: number): void {
    const v = this.voice;
    if (!v || v.pluck.length === 0) return;
    const step = 60 / v.bpm / 2;
    if (this.nextPluck < now) this.nextPluck = now;
    while (this.nextPluck < now + LOOKAHEAD) {
      const note = v.pluck[this.pluckStep % v.pluck.length];
      if (note >= 0) {
        tone(this.ctx, this.music, this.nextPluck, {
          type: 'triangle',
          freq: midiHz(v.root + 12 + note),
          dur: 0.9,
          gain: 0.05,
          attack: 0.01
        });
      }
      this.pluckStep++;
      this.nextPluck += step;
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
