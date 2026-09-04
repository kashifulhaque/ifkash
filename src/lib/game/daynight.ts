// Day and night. A slow clock drives `timeOfDay` through a ring of lighting
// keyframes; the Game copies the sampled values onto the sky, the lights, the
// stars, and the water every frame, and uses `night` to brighten the emissive
// pieces after dark.

import * as THREE from 'three';

/** Seconds for one full day. */
export const DAY_LENGTH = 240;

/** Time of day the planet wakes up at: mid-morning, so first light is friendly. */
export const START_TIME = 0.34;

export type Lighting = {
  sky: THREE.Color;
  hemiSky: THREE.Color;
  hemiGround: THREE.Color;
  hemiIntensity: number;
  sun: THREE.Color;
  sunIntensity: number;
  /** Sun elevation, 0 at the horizon and 1 overhead. Shapes the shadow angle. */
  sunHeight: number;
  starOpacity: number;
  water: THREE.Color;
  /** 0 in full daylight, 1 at midnight. Drives the emissive pieces. */
  night: number;
};

type Key = {
  t: number;
  sky: number;
  hemiSky: number;
  hemiGround: number;
  hemiI: number;
  sun: number;
  sunI: number;
  sunH: number;
  stars: number;
  water: number;
  night: number;
};

// Midnight is 0, noon is 0.5. The last key repeats the first so the ring wraps.
const MIDNIGHT: Key = {
  t: 0,
  sky: 0x060b16,
  hemiSky: 0x24344f,
  hemiGround: 0x0c1218,
  hemiI: 0.5,
  sun: 0x8aa4cf,
  sunI: 0.42,
  sunH: 0.7,
  stars: 0.9,
  water: 0x1a4a66,
  night: 1
};

const KEYS: Key[] = [
  MIDNIGHT,
  { ...MIDNIGHT, t: 0.2, sky: 0x0b1424 },
  { t: 0.28, sky: 0x7a4f5e, hemiSky: 0xffbf94, hemiGround: 0x4a3a3e, hemiI: 0.95, sun: 0xffb26b, sunI: 1.4, sunH: 0.15, stars: 0.25, water: 0x3a7f9c, night: 0.25 },
  { t: 0.36, sky: 0x2f6a9c, hemiSky: 0xe6f2ff, hemiGround: 0x6b7a63, hemiI: 1.3, sun: 0xfff0d4, sunI: 2.0, sunH: 0.6, stars: 0.08, water: 0x3fb3d3, night: 0 },
  { t: 0.5, sky: 0x3778ad, hemiSky: 0xdcefff, hemiGround: 0x6b7a63, hemiI: 1.35, sun: 0xfff3dc, sunI: 2.1, sunH: 1, stars: 0.04, water: 0x3fb3d3, night: 0 },
  { t: 0.64, sky: 0x2f6a9c, hemiSky: 0xe6f2ff, hemiGround: 0x6b7a63, hemiI: 1.3, sun: 0xfff0d4, sunI: 2.0, sunH: 0.6, stars: 0.08, water: 0x3fb3d3, night: 0 },
  { t: 0.73, sky: 0x6b3a4d, hemiSky: 0xff9e6e, hemiGround: 0x3e2c36, hemiI: 0.9, sun: 0xff8f52, sunI: 1.25, sunH: 0.15, stars: 0.3, water: 0x35708f, night: 0.3 },
  { t: 0.82, sky: 0x0f1a30, hemiSky: 0x2c3e60, hemiGround: 0x121820, hemiI: 0.55, sun: 0x8aa4cf, sunI: 0.45, sunH: 0.5, stars: 0.85, water: 0x1d4d6a, night: 0.95 },
  { ...MIDNIGHT, t: 1 }
];

const _c = new THREE.Color();

function mix(a: number, b: number, u: number): number {
  return a + (b - a) * u;
}

export class DayNight {
  /** Fraction of the day, in [0, 1). Midnight is 0 and noon is 0.5. */
  time = START_TIME;
  paused = false;
  readonly light: Lighting = {
    sky: new THREE.Color(),
    hemiSky: new THREE.Color(),
    hemiGround: new THREE.Color(),
    hemiIntensity: 1,
    sun: new THREE.Color(),
    sunIntensity: 1,
    sunHeight: 1,
    starOpacity: 0,
    water: new THREE.Color(),
    night: 0
  };

  constructor() {
    this.sample();
  }

  /** Advance the clock and refresh `light`. */
  update(dt: number): void {
    if (!this.paused) this.time = (this.time + dt / DAY_LENGTH) % 1;
    this.sample();
  }

  /** Jump to a time of day; handy from the console when tuning. */
  set(time: number): void {
    this.time = ((time % 1) + 1) % 1;
    this.sample();
  }

  private sample(): void {
    const t = this.time;
    let i = 0;
    while (i < KEYS.length - 2 && KEYS[i + 1].t <= t) i++;
    const a = KEYS[i];
    const b = KEYS[i + 1];
    const u = THREE.MathUtils.smoothstep((t - a.t) / (b.t - a.t), 0, 1);
    const L = this.light;
    L.sky.setHex(a.sky).lerp(_c.setHex(b.sky), u);
    L.hemiSky.setHex(a.hemiSky).lerp(_c.setHex(b.hemiSky), u);
    L.hemiGround.setHex(a.hemiGround).lerp(_c.setHex(b.hemiGround), u);
    L.sun.setHex(a.sun).lerp(_c.setHex(b.sun), u);
    L.water.setHex(a.water).lerp(_c.setHex(b.water), u);
    L.hemiIntensity = mix(a.hemiI, b.hemiI, u);
    L.sunIntensity = mix(a.sunI, b.sunI, u);
    L.sunHeight = mix(a.sunH, b.sunH, u);
    L.starOpacity = mix(a.stars, b.stars, u);
    L.night = mix(a.night, b.night, u);
  }
}
