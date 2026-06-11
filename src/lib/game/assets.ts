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

// Several outfits per portfolio section (Modular Men + Modular Women packs —
// identical rig, so the procedural animation works on all of them).
export const NPC_MODELS: Record<string, string[]> = {
  work: ['/models/npc/Suit.glb', '/models/npc/W_Formal.glb', '/models/npc/King.glb'],
  education: ['/models/npc/Adventurer.glb', '/models/npc/W_Adventurer.glb'],
  projects: ['/models/npc/Worker.glb', '/models/npc/W_Worker.glb', '/models/npc/Swat.glb'],
  resume: ['/models/npc/Casual.glb', '/models/npc/Casual2.glb', '/models/npc/W_Casual.glb'],
  blog: ['/models/npc/Punk.glb', '/models/npc/W_Punk.glb', '/models/npc/W_Witch.glb'],
  contact: ['/models/npc/Farmer.glb', '/models/npc/Beach.glb', '/models/npc/W_Medieval.glb']
};

export const NATURE_MODELS = {
  trees: [
    '/models/nature/CommonTree_1.glb',
    '/models/nature/CommonTree_2.glb',
    '/models/nature/CommonTree_3.glb',
    '/models/nature/CommonTree_4.glb',
    '/models/nature/CommonTree_5.glb',
    '/models/nature/CommonTree_Dead_1.glb',
    '/models/nature/PineTree_1.glb',
    '/models/nature/PineTree_2.glb',
    '/models/nature/PineTree_3.glb',
    '/models/nature/PineTree_5.glb',
    '/models/nature/BirchTree_1.glb',
    '/models/nature/BirchTree_2.glb',
    '/models/nature/Willow_1.glb'
  ],
  // Waterside picks, biased onto riverbanks
  riverTrees: [
    '/models/nature/Willow_1.glb',
    '/models/nature/Willow_2.glb',
    '/models/nature/PalmTree_1.glb'
  ],
  rocks: [
    '/models/nature/Rock_Moss_1.glb',
    '/models/nature/Rock_Moss_4.glb',
    '/models/nature/Rock_Moss_5.glb',
    '/models/nature/Rock_1.glb',
    '/models/nature/Rock_2.glb',
    '/models/nature/Rock_5.glb',
    '/models/nature/Rock_7.glb'
  ],
  // Chunky silhouettes that read well scaled up on mountain faces
  cliffs: [
    '/models/nature/Rock_3.glb',
    '/models/nature/Rock_6.glb',
    '/models/nature/Rock_Moss_6.glb'
  ],
  smalls: [
    '/models/nature/Bush_1.glb',
    '/models/nature/Bush_2.glb',
    '/models/nature/BushBerries_1.glb',
    '/models/nature/Grass_2.glb',
    '/models/nature/Grass_Short.glb',
    '/models/nature/Grass.glb',
    '/models/nature/Flowers.glb',
    '/models/nature/Plant_1.glb',
    '/models/nature/Plant_2.glb',
    '/models/nature/Wheat.glb',
    '/models/nature/TreeStump_Moss.glb',
    '/models/nature/WoodLog_Moss.glb'
  ],
  lilypad: '/models/nature/Lilypad.glb'
};

export const VEHICLE_MODELS = [
  '/models/vehicles/sedan.glb',
  '/models/vehicles/suv.glb',
  '/models/vehicles/hatchback-sports.glb',
  '/models/vehicles/truck.glb',
  '/models/vehicles/van.glb',
  '/models/vehicles/taxi.glb'
];

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
    ...Object.values(NPC_MODELS).flat(),
    ...NATURE_MODELS.trees,
    ...NATURE_MODELS.riverTrees,
    ...NATURE_MODELS.rocks,
    ...NATURE_MODELS.cliffs,
    ...NATURE_MODELS.smalls,
    NATURE_MODELS.lilypad,
    ...COVER_MODELS,
    ...VEHICLE_MODELS
  ];
  await Promise.all(urls.map(loadModel));
}
