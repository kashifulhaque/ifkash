// The planet's one and only lighting mood. There is no day-night cycle: the
// world sits under a permanent, moonlit night, bright enough overhead to read
// the stars, the constellations, the moon, and the planets against, and bright
// enough underfoot to walk by. Everything warmer than this comes from the light
// sources scattered over the surface; see `lights.ts`.

import * as THREE from 'three';

export type NightLighting = {
  /** Scene background: a deep navy the star field still reads against. */
  sky: THREE.Color;
  /** Hemisphere light: moonlit blue from above, cold ground bounce below. */
  hemiSky: THREE.Color;
  hemiGround: THREE.Color;
  hemiIntensity: number;
  /** The key light. Pale and blue, standing in for moonlight. */
  moon: THREE.Color;
  moonIntensity: number;
  /** Elevation of the key light, 0 at the horizon and 1 overhead. Sets the shadow angle. */
  moonHeight: number;
  water: THREE.Color;
};

/**
 * Tuned so the sky stays legible and the ground stays walkable. Raising
 * `hemiIntensity` flattens the world; raising `moonIntensity` deepens the
 * shadows. `sky` is the one to lift if the stars start to feel lost.
 */
export const NIGHT: NightLighting = {
  sky: new THREE.Color(0x16243f),
  hemiSky: new THREE.Color(0x4a6c9e),
  hemiGround: new THREE.Color(0x1e2a34),
  hemiIntensity: 0.86,
  moon: new THREE.Color(0xaecbf5),
  moonIntensity: 1.15,
  moonHeight: 0.62,
  water: new THREE.Color(0x256a8c)
};
