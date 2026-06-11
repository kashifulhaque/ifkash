import * as THREE from 'three';
import type { AABB } from './collision';
import { chunkRng } from './rng';
import { loadModel, cloneStatic, NATURE_MODELS, COVER_MODELS } from './assets';
import {
  terrainHeight,
  roadFactor,
  riverFactor,
  riverCarveAt,
  waterLevel,
  slopeAt,
  villageContaining,
  villagePlan,
  REGION
} from './terrain';
import { makeHouse } from './buildings';

export const CHUNK_SIZE = 24;
const VIEW_CHUNKS = 3; // load radius in chunks (7x7 grid)

const GROUND_COLORS = [0x8bc34a, 0x7cb342, 0x9ccc65];
const ROAD_COLORS = [0xa1887f, 0x8d6e63, 0x97806f];
const VILLAGE_COLORS = [0xa5b85b, 0x9aad55, 0xb0c266];
const ROCK_COLORS = [0x8d8d85, 0x7f7f78, 0x96968c];
const SNOW_COLORS = [0xe8e8ec, 0xdfe2e8, 0xf2f2f6];
const RIVERBED_COLORS = [0x8a7a5c, 0x817152, 0x938265]; // wet mud under the water
const BANK_COLORS = [0xc2b280, 0xb8a96f, 0xccbd8d]; // sandy banks

// Scale a model so its bounding-box height matches target.
export function normalizeHeight(obj: THREE.Object3D, target: number) {
  const box = new THREE.Box3().setFromObject(obj);
  const h = box.max.y - box.min.y;
  if (h > 0.001) obj.scale.multiplyScalar(target / h);
}

type Chunk = {
  group: THREE.Group;
  colliders: AABB[];
};

export class ChunkManager {
  private scene: THREE.Scene;
  private chunks = new Map<string, Chunk>();
  private groundMat: THREE.MeshLambertMaterial;
  private waterMat: THREE.MeshLambertMaterial;

  constructor(scene: THREE.Scene) {
    this.scene = scene;
    this.groundMat = new THREE.MeshLambertMaterial({ vertexColors: true, flatShading: true });
    this.waterMat = new THREE.MeshLambertMaterial({
      color: 0x3f7fae,
      transparent: true,
      opacity: 0.78,
      flatShading: true
    });
  }

  update(px: number, pz: number) {
    const ccx = Math.floor(px / CHUNK_SIZE);
    const ccz = Math.floor(pz / CHUNK_SIZE);

    for (let dx = -VIEW_CHUNKS; dx <= VIEW_CHUNKS; dx++) {
      for (let dz = -VIEW_CHUNKS; dz <= VIEW_CHUNKS; dz++) {
        const key = `${ccx + dx},${ccz + dz}`;
        if (!this.chunks.has(key)) this.createChunk(ccx + dx, ccz + dz, key);
      }
    }

    for (const [key, chunk] of this.chunks) {
      const [cx, cz] = key.split(',').map(Number);
      if (Math.max(Math.abs(cx - ccx), Math.abs(cz - ccz)) > VIEW_CHUNKS + 1) {
        this.disposeChunk(chunk);
        this.chunks.delete(key);
      }
    }
  }

  // Colliders from the 3x3 chunk neighborhood around a point.
  collidersNear(x: number, z: number): AABB[] {
    const ccx = Math.floor(x / CHUNK_SIZE);
    const ccz = Math.floor(z / CHUNK_SIZE);
    const out: AABB[] = [];
    for (let dx = -1; dx <= 1; dx++) {
      for (let dz = -1; dz <= 1; dz++) {
        const chunk = this.chunks.get(`${ccx + dx},${ccz + dz}`);
        if (chunk) out.push(...chunk.colliders);
      }
    }
    return out;
  }

  // Is this spot free of prop colliders (for NPC/crate placement)?
  isFree(x: number, z: number, r = 1): boolean {
    for (const b of this.collidersNear(x, z)) {
      if (x > b.minX - r && x < b.maxX + r && z > b.minZ - r && z < b.maxZ + r) return false;
    }
    return true;
  }

  private createChunk(cx: number, cz: number, key: string) {
    const rng = chunkRng(cx, cz);
    const group = new THREE.Group();
    const colliders: AABB[] = [];
    const ox = cx * CHUNK_SIZE;
    const oz = cz * CHUNK_SIZE;

    // Ground tile: world-space terrain heights (seams match by construction)
    // colored grass / village clearing / dirt road per vertex.
    const geo = new THREE.PlaneGeometry(CHUNK_SIZE, CHUNK_SIZE, 16, 16);
    geo.rotateX(-Math.PI / 2);
    const pos = geo.attributes.position;
    const colors: number[] = [];
    const palette = GROUND_COLORS.map((c) => new THREE.Color(c));
    const roadPalette = ROAD_COLORS.map((c) => new THREE.Color(c));
    const villagePalette = VILLAGE_COLORS.map((c) => new THREE.Color(c));
    const rockPalette = ROCK_COLORS.map((c) => new THREE.Color(c));
    const snowPalette = SNOW_COLORS.map((c) => new THREE.Color(c));
    const bedPalette = RIVERBED_COLORS.map((c) => new THREE.Color(c));
    const bankPalette = BANK_COLORS.map((c) => new THREE.Color(c));
    let maxCarve = 0; // does a river run through this chunk?
    for (let i = 0; i < pos.count; i++) {
      const wx = pos.getX(i) + ox + CHUNK_SIZE / 2;
      const wz = pos.getZ(i) + oz + CHUNK_SIZE / 2;
      const h = terrainHeight(wx, wz);
      pos.setY(i, h);
      maxCarve = Math.max(maxCarve, riverCarveAt(wx, wz));
      const pick = Math.floor(Math.abs(Math.sin(wx * 12.9898 + wz * 78.233) * 43758.5453) % 1 * 3) % 3;
      let c = palette[pick];
      const rf = riverFactor(wx, wz);
      if (roadFactor(wx, wz) > 0.5) c = roadPalette[pick];
      else if (rf > 0.55) c = bedPalette[pick];
      else if (rf > 0.2) c = bankPalette[pick];
      else if (villageContaining(wx, wz)) c = villagePalette[pick];
      else {
        // Mountains: bare rock on steep faces, snow on the high peaks
        const slope =
          Math.abs(terrainHeight(wx + 1, wz) - h) + Math.abs(terrainHeight(wx, wz + 1) - h);
        if (h > 12) c = snowPalette[pick];
        else if (slope > 1.2) c = rockPalette[pick];
      }
      colors.push(c.r, c.g, c.b);
    }
    geo.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    geo.computeVertexNormals();
    const ground = new THREE.Mesh(geo, this.groundMat);
    ground.position.set(ox + CHUNK_SIZE / 2, 0, oz + CHUNK_SIZE / 2);
    group.add(ground);

    // Water surface where a river crosses this chunk. The surface follows the
    // un-carved bank line minus a bit, so it pokes above the riverbed but
    // dives underground at the banks — no hard shoreline geometry needed.
    if (maxCarve > 0.6) {
      const wgeo = new THREE.PlaneGeometry(CHUNK_SIZE, CHUNK_SIZE, 8, 8);
      wgeo.rotateX(-Math.PI / 2);
      const wpos = wgeo.attributes.position;
      for (let i = 0; i < wpos.count; i++) {
        const wx = wpos.getX(i) + ox + CHUNK_SIZE / 2;
        const wz = wpos.getZ(i) + oz + CHUNK_SIZE / 2;
        wpos.setY(i, waterLevel(wx, wz));
      }
      wgeo.computeVertexNormals();
      const water = new THREE.Mesh(wgeo, this.waterMat);
      water.position.set(ox + CHUNK_SIZE / 2, 0, oz + CHUNK_SIZE / 2);
      group.add(water);
    }

    const place = (
      url: string,
      x: number,
      z: number,
      height: number,
      rotY: number,
      colliderRadius: number | null
    ) => {
      const y = terrainHeight(x, z);
      loadModel(url).then((gltf) => {
        const obj = cloneStatic(gltf);
        normalizeHeight(obj, height);
        obj.position.set(x, y, z);
        obj.rotation.y = rotY;
        group.add(obj);
      });
      if (colliderRadius) {
        colliders.push({
          minX: x - colliderRadius,
          maxX: x + colliderRadius,
          minZ: z - colliderRadius,
          maxZ: z + colliderRadius
        });
      }
    };

    const spot = () => [ox + 1.5 + rng() * (CHUNK_SIZE - 3), oz + 1.5 + rng() * (CHUNK_SIZE - 3)];
    // Keep wild props off roads, out of village clearings, out of the water
    // and off steep faces (props sit at their center height; cliffs would
    // expose floating bases)
    const blocked = (x: number, z: number) => {
      if (roadFactor(x, z) > 0.1 || riverFactor(x, z) > 0.2 || villageContaining(x, z)) return true;
      const h = terrainHeight(x, z);
      return Math.abs(terrainHeight(x + 1, z) - h) + Math.abs(terrainHeight(x, z + 1) - h) > 1.5;
    };

    // Trees: wide size spread, with the occasional giant; willows and palms
    // crowd the riverbanks
    const nTrees = 2 + Math.floor(rng() * 4);
    for (let i = 0; i < nTrees; i++) {
      const [x, z] = spot();
      const rf = riverFactor(x, z);
      const pool = rf > 0.02 && rf <= 0.2 ? NATURE_MODELS.riverTrees : NATURE_MODELS.trees;
      const url = pool[Math.floor(rng() * pool.length)];
      const giant = rng() < 0.1;
      const h = giant ? 10 + rng() * 4 : 4 + rng() * 4.5;
      const rot = rng() * Math.PI * 2;
      if (!blocked(x, z)) place(url, x, z, h, rot, giant ? 0.95 : 0.55);
    }

    // Cliff outcrops on steep mountain faces: big scaled-up rocks sunk into
    // the slope, facing downhill
    for (let i = 0; i < 2; i++) {
      const [x, z] = spot();
      if (slopeAt(x, z) > 0.7 && riverFactor(x, z) < 0.2 && !villageContaining(x, z)) {
        const url = NATURE_MODELS.cliffs[Math.floor(rng() * NATURE_MODELS.cliffs.length)];
        const h = 6 + rng() * 4;
        const downhill = Math.atan2(
          terrainHeight(x - 1, z) - terrainHeight(x + 1, z),
          terrainHeight(x, z - 1) - terrainHeight(x, z + 1)
        );
        const y = terrainHeight(x, z) - 0.5;
        loadModel(url).then((gltf) => {
          const obj = cloneStatic(gltf);
          normalizeHeight(obj, h);
          obj.position.set(x, y, z);
          obj.rotation.y = downhill;
          group.add(obj);
        });
        colliders.push({ minX: x - 2, maxX: x + 2, minZ: z - 2, maxZ: z + 2 });
      }
    }

    // Lilypads drifting on river water
    for (let i = 0; i < 3; i++) {
      const [x, z] = spot();
      if (riverFactor(x, z) > 0.7) {
        const y = waterLevel(x, z) + 0.02;
        const rot = rng() * Math.PI * 2;
        loadModel(NATURE_MODELS.lilypad).then((gltf) => {
          const obj = cloneStatic(gltf);
          obj.position.set(x, y, z);
          obj.rotation.y = rot;
          group.add(obj);
        });
      }
    }
    // Rocks
    const nRocks = 1 + Math.floor(rng() * 3);
    for (let i = 0; i < nRocks; i++) {
      const [x, z] = spot();
      const url = NATURE_MODELS.rocks[Math.floor(rng() * NATURE_MODELS.rocks.length)];
      const h = 0.7 + rng() * 1.2;
      const rot = rng() * Math.PI * 2;
      if (!blocked(x, z)) place(url, x, z, h, rot, h * 0.8);
    }
    // Small decor (no collision)
    const nSmall = 4 + Math.floor(rng() * 6);
    for (let i = 0; i < nSmall; i++) {
      const [x, z] = spot();
      const url = NATURE_MODELS.smalls[Math.floor(rng() * NATURE_MODELS.smalls.length)];
      const h = 0.3 + rng() * 0.6;
      const rot = rng() * Math.PI * 2;
      if (!blocked(x, z)) place(url, x, z, h, rot, null);
    }
    // Cover cluster in ~40% of chunks: crates/barrels/carts to duck behind
    if (rng() < 0.4) {
      const [bx, bz] = spot();
      const n = 2 + Math.floor(rng() * 3);
      for (let i = 0; i < n; i++) {
        const x = bx + (rng() - 0.5) * 5;
        const z = bz + (rng() - 0.5) * 5;
        const url = COVER_MODELS[Math.floor(rng() * COVER_MODELS.length)];
        const h = 1.0 + rng() * 0.5;
        const rot = rng() * Math.PI * 2;
        if (!blocked(x, z)) place(url, x, z, h, rot, 0.8);
      }
    }

    // Village content. The plan comes from region RNG (never the chunk rng),
    // so every overlapping chunk derives the same layout; each chunk only
    // instantiates the slots that fall inside its own bounds.
    const rx = Math.floor((ox + CHUNK_SIZE / 2) / REGION);
    const rz = Math.floor((oz + CHUNK_SIZE / 2) / REGION);
    const inChunk = (x: number, z: number) =>
      x >= ox && x < ox + CHUNK_SIZE && z >= oz && z < oz + CHUNK_SIZE;
    for (let drx = -1; drx <= 1; drx++) {
      for (let drz = -1; drz <= 1; drz++) {
        const plan = villagePlan(rx + drx, rz + drz);
        if (!plan) continue;
        for (const slot of plan.houses) {
          if (!inChunk(slot.x, slot.z)) continue;
          const house = makeHouse(slot);
          house.position.set(slot.x, terrainHeight(slot.x, slot.z), slot.z);
          group.add(house);
          const r = Math.max(slot.width, slot.depth) / 2 + 0.15;
          colliders.push({ minX: slot.x - r, maxX: slot.x + r, minZ: slot.z - r, maxZ: slot.z + r });
        }
        for (const prop of plan.props) {
          if (!inChunk(prop.x, prop.z)) continue;
          const url = COVER_MODELS[prop.modelIndex % COVER_MODELS.length];
          place(url, prop.x, prop.z, prop.height, prop.rot, 0.8);
        }
      }
    }

    group.userData.chunkKey = key;
    this.scene.add(group);
    this.chunks.set(key, { group, colliders });
  }

  private disposeChunk(chunk: Chunk) {
    this.scene.remove(chunk.group);
    chunk.group.traverse((obj) => {
      const mesh = obj as THREE.Mesh;
      // Only dispose the per-chunk ground geometry; prop geometries are shared via the GLTF cache.
      if (mesh.isMesh && (mesh.material === this.groundMat || mesh.material === this.waterMat))
        mesh.geometry.dispose();
    });
  }

  dispose() {
    for (const chunk of this.chunks.values()) this.disposeChunk(chunk);
    this.chunks.clear();
    this.groundMat.dispose();
    this.waterMat.dispose();
  }
}
