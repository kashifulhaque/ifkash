// The one place the renderer configures model loading. Every GLB that `lf-tools assets` writes
// uses meshopt compression, and a textured one stores its textures as KTX2 (ADR 0010), so
// every loader needs both decoders. Call `createGltfLoader` instead of `new GLTFLoader()`.
//
// Three.js expects a single `KTX2Loader`: each one owns a pool of transcoder workers. This
// module keeps one per renderer. The loader fetches the Basis transcoder
// (`static/latticefall/basis/`, about 585 KB) only when a file with KTX2 textures is parsed,
// so models without textures cost nothing extra.

import type { WebGLRenderer } from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { KTX2Loader } from 'three/examples/jsm/loaders/KTX2Loader.js';
import { MeshoptDecoder } from 'three/examples/jsm/libs/meshopt_decoder.module.js';

/** Where the Basis transcoder is served from. `lf-tools assets` copies it from `three`. */
export const BASIS_PATH = '/latticefall/basis/';

const ktx2Loaders = new WeakMap<WebGLRenderer, KTX2Loader>();

/** The renderer that last asked for a loader. Level models load through it. */
let current: WebGLRenderer | null = null;

/** The renderer's KTX2 loader, set up for the compressed formats its GPU supports. */
export function sharedKtx2Loader(renderer: WebGLRenderer): KTX2Loader {
  current = renderer;
  let ktx2 = ktx2Loaders.get(renderer);
  if (!ktx2) {
    ktx2 = new KTX2Loader().setTranscoderPath(BASIS_PATH).detectSupport(renderer);
    ktx2Loaders.set(renderer, ktx2);
  }
  return ktx2;
}

/**
 * A GLTF loader that reads meshopt-compressed geometry and KTX2 textures. Without a
 * `renderer`, it uses the one that last asked for a loader (the scene registers its renderer
 * when it starts). If there is none, the loader reads meshopt only.
 */
export function createGltfLoader(renderer: WebGLRenderer | null = current): GLTFLoader {
  const loader = new GLTFLoader().setMeshoptDecoder(MeshoptDecoder);
  return renderer ? loader.setKTX2Loader(sharedKtx2Loader(renderer)) : loader;
}

/** Stops the renderer's transcoder workers. Call it when the renderer is disposed. */
export function disposeGltfLoaders(renderer: WebGLRenderer): void {
  ktx2Loaders.get(renderer)?.dispose();
  ktx2Loaders.delete(renderer);
  if (current === renderer) current = null;
}
