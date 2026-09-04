// Two passes over the lit materials that no amount of extra geometry would buy.
//
// Weather that lands on the world instead of falling through it: snow that
// settles on upward-facing surfaces, and rain that darkens the ground and puts
// a sheen on it. And a rim of sky light along every silhouette, which is what
// separates one dark shape from the dark shape behind it.
//
// Both are one small patch to the lit materials rather than extra geometry. The
// effect is strongest at the player and falls off with distance, so a snowfall
// dusts the roofs around you without whitening the far side of the planet.

import * as THREE from 'three';

/** Handle on the patched materials. The Game drives it every frame. */
export type SurfaceFx = {
  /** Snow settled on upward faces, 0 to 1. */
  setSnow(v: number): void;
  /** Wet, darkened, shiny ground, 0 to 1. */
  setWet(v: number): void;
  /** World point the effect is centred on: the player. */
  setCenter(p: THREE.Vector3): void;
};

/** Surface units from the centre at which the effect has faded out. */
const FALLOFF = 34;

/**
 * Patch every lit material that makes up the world so it can be snowed on and
 * rained on. All the materials share one set of uniforms, so one call drives
 * the ground and the props together.
 */
export function patchSurface(materials: THREE.Material[], rim: THREE.Color): SurfaceFx {
  const uniforms = {
    uFxCenter: { value: new THREE.Vector3(0, 1e6, 0) },
    uFxRadius: { value: FALLOFF },
    uFxSnow: { value: 0 },
    uFxWet: { value: 0 },
    uFxRim: { value: rim.clone() }
  };

  for (const material of materials) {
    material.onBeforeCompile = (shader) => {
      Object.assign(shader.uniforms, uniforms);
      shader.vertexShader =
        'varying vec3 vFxPos;\nvarying vec3 vFxNrm;\n' +
        shader.vertexShader.replace(
          '#include <begin_vertex>',
          `#include <begin_vertex>
  vFxPos = (modelMatrix * vec4(transformed, 1.0)).xyz;
  vFxNrm = normalize(mat3(modelMatrix) * objectNormal);`
        );
      shader.fragmentShader =
        `uniform vec3 uFxCenter;
uniform float uFxRadius;
uniform float uFxSnow;
uniform float uFxWet;
uniform vec3 uFxRim;
varying vec3 vFxPos;
varying vec3 vFxNrm;
float fxWet;
float fxSky;
` +
        shader.fragmentShader
          .replace(
            '#include <color_fragment>',
            `#include <color_fragment>
  // Near the player, and only where the surface faces the sky.
  float fxNear = 1.0 - smoothstep(uFxRadius * 0.55, uFxRadius, distance(vFxPos, uFxCenter));
  float fxUp = smoothstep(0.15, 0.70, dot(normalize(vFxNrm), normalize(vFxPos)));
  fxSky = dot(normalize(vFxNrm), normalize(vFxPos));
  float fxSnow = uFxSnow * fxNear * fxUp;
  fxWet = uFxWet * fxNear * (0.4 + 0.6 * fxUp);
  diffuseColor.rgb = mix(diffuseColor.rgb, vec3(0.92, 0.95, 0.99), fxSnow);
  diffuseColor.rgb *= 1.0 - fxWet * 0.28;`
          )
          .replace(
            '#include <roughnessmap_fragment>',
            `#include <roughnessmap_fragment>
  roughnessFactor = mix(roughnessFactor, 0.12, fxWet * 0.85);`
          )
          .replace(
            '#include <opaque_fragment>',
            `// Sky bounce along the silhouette. Under one dim key light every
  // unlit shape flattens into the same near-black; a cool rim where a
  // surface turns away from the camera gives each one its edge back.
  {
    vec3 fxView = normalize(cameraPosition - vFxPos);
    float fxRim = pow(1.0 - clamp(dot(normalize(vFxNrm), fxView), 0.0, 1.0), 3.0);
    outgoingLight += uFxRim * fxRim * (0.35 + 0.35 * clamp(fxSky, 0.0, 1.0));
  }
#include <opaque_fragment>`
          );
    };
    // Without a distinct cache key three can hand this material a program
    // compiled for an unpatched one with the same settings.
    material.customProgramCacheKey = () => 'surface-fx';
    material.needsUpdate = true;
  }

  return {
    setSnow: (v) => (uniforms.uFxSnow.value = v),
    setWet: (v) => (uniforms.uFxWet.value = v),
    setCenter: (p) => uniforms.uFxCenter.value.copy(p)
  };
}
