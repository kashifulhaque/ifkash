// Procedural placeholder sounds. Everything is synthesized from oscillators and filtered
// noise, so there are no audio assets. Each function schedules one sound on `dest` at
// time `t` and cleans itself up when it ends.

export type Surface = 'stone' | 'grass' | 'wood' | 'sand' | 'water';

export function noiseBuffer(ctx: BaseAudioContext, seconds = 2, seed = 1): AudioBuffer {
  const len = Math.max(1, Math.floor(ctx.sampleRate * seconds));
  const buf = ctx.createBuffer(1, len, ctx.sampleRate);
  const data = buf.getChannelData(0);
  let s = seed >>> 0 || 1;
  for (let i = 0; i < len; i++) {
    s = (Math.imul(s, 1664525) + 1013904223) >>> 0; // A small LCG: noise needs no true randomness.
    data[i] = (s / 0x80000000 - 1) * 0.9;
  }
  return buf;
}

/** A burst of filtered noise with an exponential decay. */
export function noiseBurst(
  ctx: BaseAudioContext,
  dest: AudioNode,
  noise: AudioBuffer,
  t: number,
  o: { dur: number; gain: number; type?: BiquadFilterType; freq: number; q?: number; sweepTo?: number }
): void {
  const src = ctx.createBufferSource();
  src.buffer = noise;
  const f = ctx.createBiquadFilter();
  f.type = o.type ?? 'bandpass';
  f.frequency.setValueAtTime(o.freq, t);
  if (o.sweepTo) f.frequency.exponentialRampToValueAtTime(o.sweepTo, t + o.dur);
  f.Q.value = o.q ?? 1;
  const g = ctx.createGain();
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(o.gain, t + 0.004);
  g.gain.exponentialRampToValueAtTime(0.0001, t + o.dur);
  src.connect(f).connect(g).connect(dest);
  src.start(t, Math.random() * 1.5);
  src.stop(t + o.dur + 0.02);
}

/** A pitched blip: an oscillator with an optional pitch glide and exponential decay. */
export function tone(
  ctx: BaseAudioContext,
  dest: AudioNode,
  t: number,
  o: { type?: OscillatorType; freq: number; to?: number; dur: number; gain: number; attack?: number }
): void {
  const osc = ctx.createOscillator();
  osc.type = o.type ?? 'sine';
  osc.frequency.setValueAtTime(o.freq, t);
  if (o.to) osc.frequency.exponentialRampToValueAtTime(o.to, t + o.dur);
  const g = ctx.createGain();
  const a = o.attack ?? 0.004;
  g.gain.setValueAtTime(0.0001, t);
  g.gain.exponentialRampToValueAtTime(o.gain, t + a);
  g.gain.exponentialRampToValueAtTime(0.0001, t + o.dur);
  osc.connect(g).connect(dest);
  osc.start(t);
  osc.stop(t + o.dur + 0.02);
}

export function footstep(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number, surface: Surface): void {
  const jitter = 0.85 + Math.random() * 0.3;
  switch (surface) {
    case 'grass':
      noiseBurst(ctx, dest, noise, t, { dur: 0.12, gain: 0.35, type: 'bandpass', freq: 2200 * jitter, q: 0.7 });
      break;
    case 'sand':
      noiseBurst(ctx, dest, noise, t, { dur: 0.16, gain: 0.3, type: 'lowpass', freq: 1400 * jitter, q: 0.5 });
      break;
    case 'wood':
      noiseBurst(ctx, dest, noise, t, { dur: 0.07, gain: 0.3, type: 'bandpass', freq: 700 * jitter, q: 3 });
      tone(ctx, dest, t, { freq: 180 * jitter, to: 110, dur: 0.09, gain: 0.35 });
      break;
    case 'water':
      noiseBurst(ctx, dest, noise, t, { dur: 0.22, gain: 0.35, type: 'bandpass', freq: 900 * jitter, q: 1.2, sweepTo: 2600 });
      break;
    default:
      noiseBurst(ctx, dest, noise, t, { dur: 0.06, gain: 0.35, type: 'highpass', freq: 1800 * jitter, q: 0.8 });
      tone(ctx, dest, t, { freq: 130 * jitter, to: 70, dur: 0.08, gain: 0.3 });
  }
}

export type HitKind = 'flesh' | 'guard' | 'guardBreak' | 'kill';

export function hit(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number, kind: HitKind, weight: number): void {
  const w = Math.min(1, Math.max(0, weight));
  switch (kind) {
    case 'guard':
      tone(ctx, dest, t, { type: 'square', freq: 1150, to: 900, dur: 0.14, gain: 0.12 });
      tone(ctx, dest, t, { type: 'triangle', freq: 1720, dur: 0.22, gain: 0.1 });
      noiseBurst(ctx, dest, noise, t, { dur: 0.05, gain: 0.35, type: 'highpass', freq: 3000 });
      break;
    case 'guardBreak':
      tone(ctx, dest, t, { type: 'sawtooth', freq: 600, to: 90, dur: 0.4, gain: 0.2 });
      noiseBurst(ctx, dest, noise, t, { dur: 0.3, gain: 0.5, type: 'bandpass', freq: 2400, q: 0.5, sweepTo: 500 });
      break;
    case 'kill':
      tone(ctx, dest, t, { freq: 90, to: 38, dur: 0.5, gain: 0.7 });
      noiseBurst(ctx, dest, noise, t, { dur: 0.25, gain: 0.5, type: 'lowpass', freq: 1500, sweepTo: 200 });
      break;
    default:
      tone(ctx, dest, t, { freq: 160 - 60 * w, to: 50, dur: 0.12 + 0.1 * w, gain: 0.4 + 0.3 * w });
      noiseBurst(ctx, dest, noise, t, { dur: 0.08 + 0.06 * w, gain: 0.3 + 0.3 * w, type: 'bandpass', freq: 1400 - 600 * w, q: 0.8 });
  }
}

export function uiClick(ctx: BaseAudioContext, dest: AudioNode, t: number): void {
  tone(ctx, dest, t, { type: 'triangle', freq: 880, to: 660, dur: 0.06, gain: 0.25 });
}

export function pickup(ctx: BaseAudioContext, dest: AudioNode, t: number): void {
  tone(ctx, dest, t, { freq: 784, dur: 0.18, gain: 0.25 });
  tone(ctx, dest, t + 0.07, { freq: 1175, dur: 0.3, gain: 0.25 });
}

export function lanternIgnite(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number): void {
  noiseBurst(ctx, dest, noise, t, { dur: 0.5, gain: 0.35, type: 'bandpass', freq: 600, q: 1, sweepTo: 3000 });
  tone(ctx, dest, t + 0.1, { freq: 523, dur: 0.9, gain: 0.2, attack: 0.08 });
  tone(ctx, dest, t + 0.1, { freq: 784, dur: 0.9, gain: 0.14, attack: 0.08 });
}

export function swing(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number, heavy: boolean): void {
  noiseBurst(ctx, dest, noise, t, {
    dur: heavy ? 0.32 : 0.18,
    gain: heavy ? 0.4 : 0.28,
    type: 'bandpass',
    freq: heavy ? 500 : 900,
    q: 0.9,
    sweepTo: heavy ? 1800 : 2600
  });
}

export function midiHz(note: number): number {
  return 440 * 2 ** ((note - 69) / 12);
}

/** A struck bell: a sine fundamental with the inharmonic partials of a bronze bell. */
export function bell(ctx: BaseAudioContext, dest: AudioNode, t: number, freq: number, gain: number, dur: number): void {
  tone(ctx, dest, t, { type: 'sine', freq, dur, gain, attack: 0.002 });
  tone(ctx, dest, t, { type: 'sine', freq: freq * 2.76, dur: dur * 0.5, gain: gain * 0.35, attack: 0.002 });
  tone(ctx, dest, t, { type: 'sine', freq: freq * 5.4, dur: dur * 0.25, gain: gain * 0.15, attack: 0.002 });
}

/** The Bellwright's beat. The bar's first beat (`accent`) rings deeper than the rest. */
export function bellBeat(ctx: BaseAudioContext, dest: AudioNode, t: number, accent: boolean): void {
  if (accent) bell(ctx, dest, t, 220, 0.16, 1.4);
  else bell(ctx, dest, t, 330, 0.08, 0.6);
}

/** The bell tolls: a deep ring with a low thud under it. */
export function bellToll(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number): void {
  bell(ctx, dest, t, 110, 0.3, 2.6);
  noiseBurst(ctx, dest, noise, t, { dur: 0.35, gain: 0.45, type: 'lowpass', freq: 500, sweepTo: 120 });
}

/** The bell winds up before a toll: a rising tone that swells toward it. */
export function bellWindup(ctx: BaseAudioContext, dest: AudioNode, t: number): void {
  tone(ctx, dest, t, { type: 'sine', freq: 180, to: 420, dur: 0.6, gain: 0.08, attack: 0.3 });
}

/** The toll's shockwave reaches the player. */
export function tollHit(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number): void {
  tone(ctx, dest, t, { freq: 90, to: 40, dur: 0.45, gain: 0.6 });
  noiseBurst(ctx, dest, noise, t, { dur: 0.3, gain: 0.4, type: 'lowpass', freq: 900, sweepTo: 150 });
}

/** An ice patch cracks: a short, bright crackle. */
export function iceCrack(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number): void {
  noiseBurst(ctx, dest, noise, t, { dur: 0.22, gain: 0.5, type: 'highpass', freq: 3500, sweepTo: 1200 });
}

/** An ice patch breaks into water: a long splash that sinks. */
export function iceBreak(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number): void {
  noiseBurst(ctx, dest, noise, t, { dur: 0.7, gain: 0.5, type: 'bandpass', freq: 1400, q: 0.6, sweepTo: 250 });
  tone(ctx, dest, t, { type: 'sine', freq: 260, to: 110, dur: 0.6, gain: 0.25 });
}

/** The chain snaps and the bell becomes a flail. */
export function chainSnap(ctx: BaseAudioContext, dest: AudioNode, noise: AudioBuffer, t: number): void {
  noiseBurst(ctx, dest, noise, t, { dur: 0.3, gain: 0.6, type: 'bandpass', freq: 2200, q: 0.8, sweepTo: 300 });
  tone(ctx, dest, t, { type: 'sawtooth', freq: 900, to: 180, dur: 0.35, gain: 0.2 });
}

/** The boss's defeat: a rising chord that rings out. */
export function bossSting(ctx: BaseAudioContext, dest: AudioNode, t: number): void {
  const notes = [50, 57, 62, 65, 69];
  notes.forEach((note, i) => tone(ctx, dest, t + i * 0.09, { type: 'triangle', freq: midiHz(note), dur: 2.6, gain: 0.12, attack: 0.05 }));
}
