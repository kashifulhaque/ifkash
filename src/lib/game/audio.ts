// Procedural soundscape: a generative soundtrack of glassy pads and soft mallet
// tones (one mood per biome), weather beds, footsteps, a hop blip, and a chime
// when a wonder is found. Everything is synthesised, so there are no audio assets.
//
// The soundtrack runs on its own timer so it keeps playing while the render
// loop is paused. `setMood` crossfades the pad and swaps the note pool that the
// mallet scheduler draws from; `setWeather` fades a rain or wind bed in or out.

import type { BiomeId } from './biomes';

const MUTE_KEY = 'planet_muted';

export type WeatherSound = 'rain' | 'wind' | 'ember' | null;

/** Musical character of a biome: a pad chord plus a scale for the mallet line. */
type Mood = {
  /** MIDI note of the scale root, around the fourth octave. */
  root: number;
  /** Scale degrees in semitones from the root; mallet notes are drawn from these. */
  scale: number[];
  /** Pad chord as semitone offsets from `root`, typically two octaves down. */
  chord: number[];
  /** Seconds between mallet slots. */
  beat: number;
  /** Probability that a slot plays a note. */
  density: number;
  /** Octave range of the mallet line, relative to `root`. */
  octaves: [number, number];
  /** Low-pass cutoff of the pad in Hz: lower reads darker. */
  padTone: number;
};

const MOODS: Record<BiomeId, Mood> = {
  // Fernwood: warm D major, unhurried.
  forest: { root: 62, scale: [0, 2, 4, 7, 9], chord: [-24, -17, -12, -8, -5], beat: 0.72, density: 0.42, octaves: [0, 2], padTone: 900 },
  // Clover Fields: bright G major with a lilt.
  farm: { root: 67, scale: [0, 2, 4, 7, 9], chord: [-24, -20, -17, -12, -8], beat: 0.6, density: 0.5, octaves: [0, 2], padTone: 1100 },
  // Sunmark Shore: airy A with a major seventh in the pad.
  shore: { root: 69, scale: [0, 2, 4, 7, 9, 11], chord: [-24, -17, -13, -12, -8], beat: 0.8, density: 0.4, octaves: [0, 2], padTone: 1000 },
  // Ember Heights: D minor, slow and low.
  ember: { root: 62, scale: [0, 3, 5, 7, 10], chord: [-24, -17, -12, -9, -5], beat: 1.0, density: 0.32, octaves: [-1, 1], padTone: 600 },
  // Northlight: suspended F#, sparse and high.
  arctic: { root: 66, scale: [0, 2, 5, 7, 9], chord: [-24, -19, -12, -7, -5], beat: 0.95, density: 0.3, octaves: [1, 2], padTone: 1300 },
  // The Shallows: floating E pentatonic.
  ocean: { root: 64, scale: [0, 2, 4, 7, 9], chord: [-24, -17, -12, -10, -5], beat: 0.85, density: 0.36, octaves: [0, 1], padTone: 800 }
};

const PAD_LEVEL = 0.05;
const MALLET_LEVEL = 0.1;
/** Crossfade time between pad chords, in seconds. */
const PAD_FADE = 4;
/** How far ahead of the clock the scheduler books notes. */
const LOOKAHEAD = 0.35;

function midiToHz(m: number): number {
  return 440 * Math.pow(2, (m - 69) / 12);
}

/** One sounding pad chord: oscillators through a breathing low-pass into a gain we can fade. */
type Pad = { gain: GainNode; stop: () => void };

export class Ambience {
  muted = false;
  private ctx: AudioContext | null = null;
  private master: GainNode | null = null;
  /** Dry music bus and the reverb send, both into `master`. */
  private music: GainNode | null = null;
  private reverbSend: GainNode | null = null;
  private started = false;
  private lastStep = 0;

  private mood: Mood | null = null;
  private pendingMood: BiomeId = 'forest';
  private pad: Pad | null = null;
  private timer: ReturnType<typeof setInterval> | null = null;
  private nextSlot = 0;
  private lastDegree = -1;

  private weather: WeatherSound = null;
  private rainGain: GainNode | null = null;
  private windGain: GainNode | null = null;
  private emberGain: GainNode | null = null;
  private nextCrackle = 0;

  constructor() {
    try {
      this.muted = localStorage.getItem(MUTE_KEY) === '1';
    } catch {
      /* storage unavailable */
    }
  }

  /** Create the context on the first user gesture (browsers require it). */
  start(): void {
    if (this.started) {
      this.ctx?.resume();
      return;
    }
    this.started = true;
    try {
      const Ctx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
      this.ctx = new Ctx();
    } catch {
      this.ctx = null;
      return;
    }
    const ctx = this.ctx;
    this.master = ctx.createGain();
    this.master.gain.value = this.muted ? 0 : 1;
    this.master.connect(ctx.destination);

    // Music bus: dry signal plus a long, soft reverb.
    this.music = ctx.createGain();
    this.music.gain.value = 0.9;
    this.music.connect(this.master);
    this.reverbSend = ctx.createGain();
    this.reverbSend.gain.value = 0.75;
    const softener = ctx.createBiquadFilter();
    softener.type = 'lowpass';
    softener.frequency.value = 2600;
    const reverb = ctx.createConvolver();
    reverb.buffer = this.impulse(3.2, 2.6);
    this.reverbSend.connect(softener).connect(reverb).connect(this.master);

    this.buildWeatherBeds();
    this.setWeather(this.weather);
    this.setMood(this.pendingMood);
    this.nextSlot = ctx.currentTime + 0.8;
    this.timer = setInterval(this.tick, 120);
  }

  toggle(): boolean {
    this.muted = !this.muted;
    try {
      localStorage.setItem(MUTE_KEY, this.muted ? '1' : '0');
    } catch {
      /* ignore */
    }
    if (this.master && this.ctx) {
      this.master.gain.setTargetAtTime(this.muted ? 0 : 1, this.ctx.currentTime, 0.05);
    }
    return this.muted;
  }

  // ---------------------------------------------------------------- soundtrack

  /** Switch the soundtrack to a biome's mood. Safe to call before `start`. */
  setMood(biome: BiomeId): void {
    this.pendingMood = biome;
    const mood = MOODS[biome];
    const ctx = this.ctx;
    if (!ctx || !this.music || mood === this.mood) return;
    this.mood = mood;
    this.lastDegree = -1;
    const now = ctx.currentTime;
    if (this.pad) {
      const old = this.pad;
      old.gain.gain.cancelScheduledValues(now);
      old.gain.gain.setValueAtTime(old.gain.gain.value, now);
      old.gain.gain.linearRampToValueAtTime(0, now + PAD_FADE);
      setTimeout(old.stop, PAD_FADE * 1000 + 100);
    }
    this.pad = this.buildPad(mood, now);
  }

  private buildPad(mood: Mood, now: number): Pad {
    const ctx = this.ctx!;
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(PAD_LEVEL, now + PAD_FADE);
    const filter = ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.value = mood.padTone;
    filter.Q.value = 0.4;
    const lfo = ctx.createOscillator();
    lfo.frequency.value = 0.05;
    const lfoGain = ctx.createGain();
    lfoGain.gain.value = mood.padTone * 0.35;
    lfo.connect(lfoGain).connect(filter.frequency);
    lfo.start(now);
    const oscs: OscillatorNode[] = [lfo];
    mood.chord.forEach((semis, i) => {
      const hz = midiToHz(mood.root + semis);
      // A sine and a slightly detuned triangle per note gives a glassy, slowly beating tone.
      for (const [type, detune] of [
        ['sine', 0],
        ['triangle', i % 2 === 0 ? 6 : -6]
      ] as const) {
        const osc = ctx.createOscillator();
        osc.type = type;
        osc.frequency.value = hz;
        osc.detune.value = detune;
        const g = ctx.createGain();
        g.gain.value = type === 'sine' ? 1 : 0.35;
        osc.connect(g).connect(filter);
        osc.start(now);
        oscs.push(osc);
      }
    });
    filter.connect(gain);
    gain.connect(this.music!);
    gain.connect(this.reverbSend!);
    return {
      gain,
      stop: () => {
        for (const o of oscs) {
          try {
            o.stop();
          } catch {
            /* already stopped */
          }
        }
        gain.disconnect();
      }
    };
  }

  /** Books mallet notes a little ahead of the clock. */
  private tick = () => {
    const ctx = this.ctx;
    if (!ctx || ctx.state !== 'running') return;
    this.scheduleCrackle(ctx);
    const mood = this.mood;
    if (!mood) return;
    const horizon = ctx.currentTime + LOOKAHEAD;
    while (this.nextSlot < horizon) {
      if (this.nextSlot >= ctx.currentTime && Math.random() < mood.density) this.playSlot(mood, this.nextSlot);
      // A little swing keeps the line from sounding like a metronome.
      this.nextSlot += mood.beat * (0.85 + Math.random() * 0.3);
      // Occasionally rest for a bar so the music breathes.
      if (Math.random() < 0.08) this.nextSlot += mood.beat * 3;
    }
  };

  private playSlot(mood: Mood, t: number): void {
    // Prefer small steps along the scale, with the odd leap.
    const n = mood.scale.length;
    let degree: number;
    if (this.lastDegree < 0 || Math.random() < 0.25) degree = Math.floor(Math.random() * n);
    else degree = (this.lastDegree + (Math.random() < 0.5 ? -1 : 1) + n) % n;
    this.lastDegree = degree;
    const octave = mood.octaves[0] + Math.floor(Math.random() * (mood.octaves[1] - mood.octaves[0] + 1));
    const midi = mood.root + mood.scale[degree] + octave * 12;
    this.mallet(midiToHz(midi), t, 0.6 + Math.random() * 0.4);
    // Now and then a soft dyad a scale-third above.
    if (Math.random() < 0.18) {
      const m2 = mood.root + mood.scale[(degree + 2) % n] + octave * 12 + (degree + 2 >= n ? 12 : 0);
      this.mallet(midiToHz(m2), t + 0.02, 0.35);
    }
  }

  /** A glassy mallet tone: sine fundamental, a quick bell partial, and a tiny strike. */
  private mallet(hz: number, t: number, vel: number): void {
    const ctx = this.ctx!;
    const out = ctx.createGain();
    const decay = 1.6 + Math.random() * 1.2 + (hz < 300 ? 1 : 0);
    out.gain.setValueAtTime(0, t);
    out.gain.linearRampToValueAtTime(MALLET_LEVEL * vel, t + 0.006);
    out.gain.exponentialRampToValueAtTime(0.0005, t + decay);

    const fund = ctx.createOscillator();
    fund.type = 'sine';
    fund.frequency.value = hz;
    fund.connect(out);

    const partial = ctx.createOscillator();
    partial.type = 'sine';
    partial.frequency.value = hz * 2.98;
    const pg = ctx.createGain();
    pg.gain.setValueAtTime(0.22, t);
    pg.gain.exponentialRampToValueAtTime(0.0005, t + 0.35);
    partial.connect(pg).connect(out);

    const strike = ctx.createOscillator();
    strike.type = 'triangle';
    strike.frequency.value = hz * 5.4;
    const sg = ctx.createGain();
    sg.gain.setValueAtTime(0.08, t);
    sg.gain.exponentialRampToValueAtTime(0.0005, t + 0.05);
    strike.connect(sg).connect(out);

    let sink: AudioNode = out;
    if (typeof ctx.createStereoPanner === 'function') {
      const pan = ctx.createStereoPanner();
      pan.pan.value = (Math.random() * 2 - 1) * 0.6;
      out.connect(pan);
      sink = pan;
    }
    sink.connect(this.music!);
    sink.connect(this.reverbSend!);
    for (const o of [fund, partial, strike]) {
      o.start(t);
      o.stop(t + decay + 0.05);
    }
  }

  /** Exponentially decaying noise: a soft hall for the convolver. */
  private impulse(seconds: number, falloff: number): AudioBuffer {
    const ctx = this.ctx!;
    const len = Math.floor(ctx.sampleRate * seconds);
    const buf = ctx.createBuffer(2, len, ctx.sampleRate);
    for (let c = 0; c < 2; c++) {
      const data = buf.getChannelData(c);
      for (let i = 0; i < len; i++) {
        const t = i / len;
        data[i] = (Math.random() * 2 - 1) * Math.pow(1 - t, falloff);
      }
    }
    return buf;
  }

  // ---------------------------------------------------------------- weather

  /** Fade a rain patter, wind bed, or ember hush in or out. Safe to call before `start`. */
  setWeather(kind: WeatherSound): void {
    this.weather = kind;
    const ctx = this.ctx;
    if (!ctx || !this.rainGain || !this.windGain || !this.emberGain) return;
    const now = ctx.currentTime;
    this.rainGain.gain.setTargetAtTime(kind === 'rain' ? 0.05 : 0, now, 0.8);
    this.windGain.gain.setTargetAtTime(kind === 'wind' ? 0.045 : 0, now, 0.8);
    this.emberGain.gain.setTargetAtTime(kind === 'ember' ? 0.05 : 0, now, 1.2);
    if (kind === 'ember' && this.nextCrackle < now) this.nextCrackle = now + 0.4;
  }

  /**
   * Book the next few ember pops. Crackles are sparse and irregular, so the
   * gaps are drawn fresh each time rather than run off a beat.
   */
  private scheduleCrackle(ctx: AudioContext): void {
    if (this.weather !== 'ember') return;
    const horizon = ctx.currentTime + LOOKAHEAD;
    if (this.nextCrackle < ctx.currentTime) this.nextCrackle = ctx.currentTime + 0.1;
    while (this.nextCrackle < horizon) {
      this.crackle(this.nextCrackle);
      this.nextCrackle += 0.12 + Math.random() * Math.random() * 1.4;
    }
  }

  /** One tiny pop of a burning ember: a filtered noise tick with a wooden knock. */
  private crackle(t: number): void {
    const ctx = this.ctx!;
    if (!this.master) return;
    const len = 0.05;
    const buf = ctx.createBuffer(1, Math.ceil(ctx.sampleRate * len), ctx.sampleRate);
    const data = buf.getChannelData(0);
    for (let i = 0; i < data.length; i++) {
      const u = i / data.length;
      data[i] = (Math.random() * 2 - 1) * Math.pow(1 - u, 6);
    }
    const src = ctx.createBufferSource();
    src.buffer = buf;
    const band = ctx.createBiquadFilter();
    band.type = 'bandpass';
    band.frequency.value = 900 + Math.random() * 2200;
    band.Q.value = 1.6;
    const g = ctx.createGain();
    g.gain.value = 0.02 + Math.random() * 0.05;
    let sink: AudioNode = g;
    src.connect(band).connect(g);
    if (typeof ctx.createStereoPanner === 'function') {
      const pan = ctx.createStereoPanner();
      pan.pan.value = (Math.random() * 2 - 1) * 0.7;
      g.connect(pan);
      sink = pan;
    }
    sink.connect(this.master);
    src.start(t);
  }

  private buildWeatherBeds(): void {
    const ctx = this.ctx!;
    const noise = ctx.createBuffer(1, ctx.sampleRate * 2, ctx.sampleRate);
    const data = noise.getChannelData(0);
    for (let i = 0; i < data.length; i++) data[i] = Math.random() * 2 - 1;

    // Rain: bright band-passed noise with a slow shimmer.
    const rainSrc = ctx.createBufferSource();
    rainSrc.buffer = noise;
    rainSrc.loop = true;
    const rainBand = ctx.createBiquadFilter();
    rainBand.type = 'bandpass';
    rainBand.frequency.value = 3200;
    rainBand.Q.value = 0.5;
    const rainHigh = ctx.createBiquadFilter();
    rainHigh.type = 'highpass';
    rainHigh.frequency.value = 900;
    this.rainGain = ctx.createGain();
    this.rainGain.gain.value = 0;
    rainSrc.connect(rainBand).connect(rainHigh).connect(this.rainGain).connect(this.master!);
    rainSrc.start();

    // Wind: low-passed noise whose cutoff and level swell with a slow LFO.
    const windSrc = ctx.createBufferSource();
    windSrc.buffer = noise;
    windSrc.loop = true;
    windSrc.playbackRate.value = 0.7;
    const windLow = ctx.createBiquadFilter();
    windLow.type = 'lowpass';
    windLow.frequency.value = 420;
    windLow.Q.value = 1.2;
    const lfo = ctx.createOscillator();
    lfo.frequency.value = 0.13;
    const lfoGain = ctx.createGain();
    lfoGain.gain.value = 220;
    lfo.connect(lfoGain).connect(windLow.frequency);
    lfo.start();
    this.windGain = ctx.createGain();
    this.windGain.gain.value = 0;
    windSrc.connect(windLow).connect(this.windGain).connect(this.master!);
    windSrc.start();

    // Embers: a low hush under the crackles, like heat moving through a vent.
    const emberSrc = ctx.createBufferSource();
    emberSrc.buffer = noise;
    emberSrc.loop = true;
    emberSrc.playbackRate.value = 0.45;
    const emberLow = ctx.createBiquadFilter();
    emberLow.type = 'lowpass';
    emberLow.frequency.value = 260;
    const emberLfo = ctx.createOscillator();
    emberLfo.frequency.value = 0.08;
    const emberLfoGain = ctx.createGain();
    emberLfoGain.gain.value = 120;
    emberLfo.connect(emberLfoGain).connect(emberLow.frequency);
    emberLfo.start();
    this.emberGain = ctx.createGain();
    this.emberGain.gain.value = 0;
    emberSrc.connect(emberLow).connect(this.emberGain).connect(this.master!);
    emberSrc.start();
  }

  // ---------------------------------------------------------------- effects

  /** Soft footstep; rate-limited so running does not machine-gun. */
  step(running: boolean): void {
    const ctx = this.ctx;
    if (!ctx || !this.master || this.muted) return;
    const now = ctx.currentTime;
    if (now - this.lastStep < (running ? 0.22 : 0.34)) return;
    this.lastStep = now;
    const len = 0.08;
    const buf = ctx.createBuffer(1, Math.floor(ctx.sampleRate * len), ctx.sampleRate);
    const data = buf.getChannelData(0);
    for (let i = 0; i < data.length; i++) {
      const t = i / data.length;
      data[i] = (Math.random() * 2 - 1) * (1 - t) * (1 - t);
    }
    const src = ctx.createBufferSource();
    src.buffer = buf;
    const filter = ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.value = 600 + Math.random() * 250;
    const g = ctx.createGain();
    g.gain.value = 0.07;
    src.connect(filter).connect(g).connect(this.master);
    src.start(now);
  }

  /** Four-note chime for a found wonder, in the key of the current mood. */
  chime(): void {
    const ctx = this.ctx;
    if (!ctx || !this.master || this.muted) return;
    const now = ctx.currentTime;
    const root = this.mood ? this.mood.root : 60;
    const notes = [0, 4, 7, 12].map((s) => midiToHz(root + s + 12));
    notes.forEach((freq, i) => {
      const osc = ctx.createOscillator();
      osc.type = 'sine';
      osc.frequency.value = freq;
      const g = ctx.createGain();
      const t0 = now + i * 0.12;
      g.gain.setValueAtTime(0, t0);
      g.gain.linearRampToValueAtTime(0.11, t0 + 0.02);
      g.gain.exponentialRampToValueAtTime(0.0008, t0 + 1.4);
      osc.connect(g).connect(this.master!);
      if (this.reverbSend) g.connect(this.reverbSend);
      osc.start(t0);
      osc.stop(t0 + 1.5);
    });
  }

  /** Warm two-note chirp when an animal is petted. */
  pet(): void {
    const ctx = this.ctx;
    if (!ctx || !this.master || this.muted) return;
    const now = ctx.currentTime;
    const root = this.mood ? this.mood.root : 60;
    [0, 7].forEach((semis, i) => {
      const osc = ctx.createOscillator();
      osc.type = 'triangle';
      osc.frequency.value = midiToHz(root + semis + 12);
      const g = ctx.createGain();
      const t0 = now + i * 0.09;
      g.gain.setValueAtTime(0, t0);
      g.gain.linearRampToValueAtTime(0.07, t0 + 0.015);
      g.gain.exponentialRampToValueAtTime(0.0008, t0 + 0.5);
      osc.connect(g).connect(this.master!);
      if (this.reverbSend) g.connect(this.reverbSend);
      osc.start(t0);
      osc.stop(t0 + 0.55);
    });
  }

  /** Short hop blip. */
  hop(): void {
    const ctx = this.ctx;
    if (!ctx || !this.master || this.muted) return;
    const now = ctx.currentTime;
    const osc = ctx.createOscillator();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(320, now);
    osc.frequency.exponentialRampToValueAtTime(620, now + 0.12);
    const g = ctx.createGain();
    g.gain.setValueAtTime(0.05, now);
    g.gain.exponentialRampToValueAtTime(0.001, now + 0.18);
    osc.connect(g).connect(this.master);
    osc.start(now);
    osc.stop(now + 0.2);
  }

  dispose(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
    this.pad?.stop();
    this.pad = null;
    this.mood = null;
    this.ctx?.close().catch(() => {});
    this.ctx = null;
    this.master = null;
    this.music = null;
    this.reverbSend = null;
    this.rainGain = null;
    this.windGain = null;
    this.emberGain = null;
    this.started = false;
  }
}
