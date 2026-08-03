import * as THREE from 'three';
import { GLTFLoader, type GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { clone as cloneSkeleton } from 'three/examples/jsm/utils/SkeletonUtils.js';

const draco = new DRACOLoader();
// threejsassets GLBs are Draco-compressed; pin a versioned decoder CDN.
draco.setDecoderPath('https://www.gstatic.com/draco/versioned/decoders/1.5.7/');

const loader = new GLTFLoader();
loader.setDRACOLoader(draco);

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
    '/models/nature/apple-tree.glb',
    '/models/nature/tree-oak-01.glb',
    '/models/nature/lineside-oak.glb',
    '/models/nature/lineside-pine.glb',
    '/models/nature/street-tree-01.glb',
    '/models/nature/metropolis-street-tree-01.glb',
    '/models/nature/coconut-palm.glb',
    '/models/nature/royal-palm.glb',
    '/models/nature/date-palm.glb'
  ],
  // Waterside picks, biased onto riverbanks
  riverTrees: [
    '/models/nature/coconut-palm.glb',
    '/models/nature/royal-palm.glb',
    '/models/nature/papyrus-reed.glb',
    '/models/nature/date-palm.glb'
  ],
  // Biome-flavored tree picks (subsets of trees/riverTrees, all preloaded)
  autumnTrees: [
    '/models/nature/apple-tree.glb',
    '/models/nature/tree-oak-01.glb',
    '/models/nature/lineside-oak.glb',
    '/models/nature/lineside-pine.glb'
  ],
  dryTrees: [
    '/models/nature/date-palm.glb',
    '/models/nature/coconut-palm.glb',
    '/models/nature/royal-palm.glb'
  ],
  rocks: [
    '/models/nature/railway-boulder.glb',
    '/models/nature/railway-rock-cluster.glb',
    '/models/nature/sandstone-boulder.glb',
    '/models/nature/rockfall-debris.glb',
    '/models/nature/rubble-scatter.glb',
    '/models/nature/shell-pebble-scatter.glb'
  ],
  // Chunky silhouettes that read well scaled up on mountain faces
  cliffs: [
    '/models/nature/cutting-rock-face.glb',
    '/models/nature/cutting-wall-corner.glb',
    '/models/nature/retaining-wall-run.glb',
    '/models/nature/rockfall-debris.glb'
  ],
  smalls: [
    '/models/nature/bush-round-01.glb',
    '/models/nature/lineside-shrub.glb',
    '/models/nature/desert-scrub.glb',
    '/models/nature/dune-grass-tuft.glb',
    '/models/nature/grass-tuft-scatter.glb',
    '/models/nature/wheat-cluster.glb',
    '/models/nature/flower-planter-01.glb',
    '/models/nature/plaza-planter-01.glb',
    '/models/nature/papyrus-reed.glb',
    '/models/nature/hedgerow-run.glb'
  ],
  // Quaternius lilypad — no free threejsassets equivalent
  lilypad: '/models/nature/Lilypad.glb'
};

export const VEHICLE_MODELS = [
  '/models/vehicles/car-sedan-01.glb',
  '/models/vehicles/sedan-01.glb',
  '/models/vehicles/taxi-01.glb',
  '/models/vehicles/metropolis-taxi-01.glb',
  '/models/vehicles/pastel-sedan.glb',
  '/models/vehicles/white-sports-convertible.glb'
];

export const COVER_MODELS = [
  '/models/props/crate-01.glb',
  '/models/props/barrel-01.glb',
  '/models/props/oil-drum-stack.glb',
  '/models/props/hay-bale-square.glb',
  '/models/props/basket-set.glb',
  '/models/props/bazaar-stall.glb',
  '/models/props/bench-01.glb',
  '/models/props/plaza-bench-01.glb',
  '/models/props/terrazzo-bench.glb',
  '/models/props/cafe-table-chairs.glb',
  '/models/props/picket-fence-01.glb',
  '/models/props/farm-gate.glb',
  '/models/props/wheelbarrow.glb',
  '/models/props/scarecrow.glb',
  '/models/props/amphora.glb',
  '/models/props/streetlamp-01.glb',
  '/models/props/trash-bin.glb',
  '/models/props/recycling-bin-01.glb',
  '/models/props/fire-brazier.glb',
  '/models/props/wall-torch.glb',
  '/models/props/beach-umbrella.glb',
  '/models/props/sun-lounger.glb',
  '/models/nature/drystone-wall.glb',
  '/models/nature/lineside-fence-run.glb'
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
  await Promise.all([...new Set(urls)].map(loadModel));
}
