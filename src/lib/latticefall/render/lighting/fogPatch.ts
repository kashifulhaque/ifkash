// Height fog on top of Three's distance fog (docs/game/07-presentation.md, "Art direction":
// height fog and distance fog in every outdoor region). `applyHeightFog` patches a standard
// material so fog thickens below a world height. One set of uniforms is shared by every
// patched material; `LevelLighting` updates them each frame.

import { Matrix4, Vector3, type Material, type WebGLProgramParametersWithUniforms } from 'three';

export const heightFogUniforms = {
  /** The camera's world matrix, to recover world Y from the view-space position. */
  lfViewInv: { value: new Matrix4() },
  /** x: height at which the extra fog is full, y: height at which it is gone, z: its strength 0..1. */
  lfFog: { value: new Vector3(0, 14, 0.4) }
};

const KEY = 'lf-height-fog';

export function applyHeightFog(material: Material): void {
  material.onBeforeCompile = (shader: WebGLProgramParametersWithUniforms) => {
    shader.uniforms.lfViewInv = heightFogUniforms.lfViewInv;
    shader.uniforms.lfFog = heightFogUniforms.lfFog;
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', '#include <common>\nvarying float vLfY;\nuniform mat4 lfViewInv;')
      .replace('#include <fog_vertex>', '#include <fog_vertex>\nvLfY = (lfViewInv * mvPosition).y;');
    shader.fragmentShader = shader.fragmentShader
      .replace('#include <common>', '#include <common>\nvarying float vLfY;\nuniform vec3 lfFog;')
      .replace(
        '#include <fog_fragment>',
        `#ifdef USE_FOG
  float lfFactor = smoothstep(fogNear, fogFar, vFogDepth);
  float lfLow = 1.0 - smoothstep(lfFog.x, lfFog.y, vLfY);
  lfFactor = clamp(lfFactor + lfLow * lfFog.z * smoothstep(0.0, fogFar, vFogDepth), 0.0, 1.0);
  gl_FragColor.rgb = mix(gl_FragColor.rgb, fogColor, lfFactor);
#endif`
      );
  };
  material.customProgramCacheKey = () => KEY;
}
