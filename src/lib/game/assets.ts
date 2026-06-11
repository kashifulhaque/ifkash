import * as THREE from 'three';
import { GLTFLoader, type GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { clone as cloneSkeleton } from 'three/examples/jsm/utils/SkeletonUtils.js';

const loader = new GLTFLoader();
const cache = new Map<string, Promise<GLTF>>();

export function loadModel(url: string): Promise<GLTF> {
  let p = cache.get(url);
  if (!p) {
    p = loader.loadAsync(url);
    cache.set(url, p);
  }
  return p;
}

// Static props: plain clone, shared geometry/materials.
export function cloneStatic(gltf: GLTF): THREE.Object3D {
  return gltf.scene.clone(true);
}

// Rigged characters need SkeletonUtils so bones stay wired up.
export function cloneRigged(gltf: GLTF): THREE.Object3D {
  return cloneSkeleton(gltf.scene);
}

export const NPC_MODELS: Record<string, string> = {
  work: '/models/npc/Suit.glb',
  education: '/models/npc/Adventurer.glb',
  projects: '/models/npc/Worker.glb',
  resume: '/models/npc/Casual.glb',
  blog: '/models/npc/Punk.glb',
  contact: '/models/npc/Farmer.glb'
};

export const NATURE_MODELS = {
  trees: [
    '/models/nature/CommonTree_1.glb',
    '/models/nature/CommonTree_3.glb',
    '/models/nature/PineTree_1.glb',
    '/models/nature/PineTree_3.glb',
    '/models/nature/BirchTree_1.glb',
    '/models/nature/Willow_1.glb'
  ],
  rocks: [
    '/models/nature/Rock_Moss_1.glb',
    '/models/nature/Rock_Moss_4.glb',
    '/models/nature/Rock_1.glb',
    '/models/nature/Rock_5.glb'
  ],
  smalls: [
    '/models/nature/Bush_1.glb',
    '/models/nature/BushBerries_1.glb',
    '/models/nature/Grass_2.glb',
    '/models/nature/Grass_Short.glb',
    '/models/nature/Flowers.glb',
    '/models/nature/TreeStump_Moss.glb',
    '/models/nature/WoodLog_Moss.glb'
  ]
};

export const COVER_MODELS = [
  '/models/props/Crate.glb',
  '/models/props/Barrel.glb',
  '/models/props/Fence.glb',
  '/models/props/Cart.glb',
  '/models/props/Hay.glb',
  '/models/props/Bags.glb',
  '/models/props/MarketStand_1.glb',
  '/models/props/Bench_1.glb'
];

export async function preloadAll(): Promise<void> {
  const urls = [
    ...Object.values(NPC_MODELS),
    ...NATURE_MODELS.trees,
    ...NATURE_MODELS.rocks,
    ...NATURE_MODELS.smalls,
    ...COVER_MODELS
  ];
  await Promise.all(urls.map(loadModel));
}
