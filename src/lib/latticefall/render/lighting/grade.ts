// An area's tone mapping and color grade (the art manifest's `grade`). It runs inside every
// material's fragment shader through Three's `CustomToneMapping` hook, so it costs no extra
// full-screen pass: ACES filmic tone mapping, then saturation, a power contrast around mid
// grey, a lift added to the shadows, and a gain on the highlights.
//
// Three caches compiled programs by tone-mapping mode, not by the shader text, so the first
// grade installed is the one every graded area uses for the rest of the page's life. Only
// Waking Hollow has a grade today. Areas with no grade keep `NoToneMapping`, as before.

import { CustomToneMapping, ShaderChunk, type ToneMapping, type WebGLRenderer } from 'three';
import type { ArtGrade } from '../kits/art';

const HOOK = 'vec3 CustomToneMapping( vec3 color ) { return color; }';
let installed = false;

function f(v: number): string {
  return Number.isInteger(v) ? `${v}.0` : String(v);
}

function v3(v: readonly number[]): string {
  return `vec3( ${v.map(f).join(', ')} )`;
}

function install(g: ArtGrade): void {
  if (installed) return;
  if (!ShaderChunk.tonemapping_pars_fragment.includes(HOOK)) {
    console.warn('latticefall: this Three.js has no CustomToneMapping hook; the color grade is off');
    return;
  }
  const body = `vec3 CustomToneMapping( vec3 color ) {
	vec3 c = ACESFilmicToneMapping( color );
	float l = dot( c, vec3( 0.2126, 0.7152, 0.0722 ) );
	c = mix( vec3( l ), c, ${f(g.saturation ?? 1)} );
	c = pow( max( c, vec3( 0.0 ) ) / 0.18, vec3( ${f(g.contrast ?? 1)} ) ) * 0.18;
	c = c * ${v3(g.gain ?? [1, 1, 1])} + ${v3(g.lift ?? [0, 0, 0])} * ( 1.0 - c );
	return clamp( c, 0.0, 1.0 );
}`;
  ShaderChunk.tonemapping_pars_fragment = ShaderChunk.tonemapping_pars_fragment.replace(HOOK, body);
  installed = true;
}

/**
 * Turns on `grade` for the renderer. Returns the function that puts the renderer's tone
 * mapping back the way it was.
 */
export function applyGrade(renderer: WebGLRenderer, grade: ArtGrade): () => void {
  install(grade);
  if (!installed) return () => {};
  const prev: { mapping: ToneMapping; exposure: number } = { mapping: renderer.toneMapping, exposure: renderer.toneMappingExposure };
  renderer.toneMapping = CustomToneMapping;
  renderer.toneMappingExposure = grade.exposure ?? 1;
  return () => {
    renderer.toneMapping = prev.mapping;
    renderer.toneMappingExposure = prev.exposure;
  };
}
