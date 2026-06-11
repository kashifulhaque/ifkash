// Procedural low-poly village houses — no GLB assets exist for buildings, so
// these are unit-sized box walls + prism roofs scaled per slot. Geometry and
// materials are shared across every house.
import * as THREE from 'three';
import type { HouseSlot } from './terrain';

const WALL_COLORS = [0xd7c4a3, 0xc9b08a, 0xb5a489, 0xddd2b8];
const ROOF_COLORS = [0xa0522d, 0x8d4a2a, 0x7a5c43, 0x95573a];

const wallMats = WALL_COLORS.map(
  (c) => new THREE.MeshLambertMaterial({ color: c, flatShading: true })
);
const roofMats = ROOF_COLORS.map(
  (c) => new THREE.MeshLambertMaterial({ color: c, flatShading: true })
);
const doorMat = new THREE.MeshLambertMaterial({ color: 0x4e342e, flatShading: true });
const windowMat = new THREE.MeshLambertMaterial({
  color: 0x90caf9,
  flatShading: true,
  emissive: 0x90caf9,
  emissiveIntensity: 0.15
});

const wallGeo = new THREE.BoxGeometry(1, 1, 1);
const slabGeo = new THREE.BoxGeometry(1, 1, 0.06);

// Unit roof prism: ridge along the x axis, base 1x1 at y=0, peak at y=1.
function makeRoofGeo(): THREE.BufferGeometry {
  const geo = new THREE.BufferGeometry();
  // prettier-ignore
  const v = [
    // slope +z
    -0.5, 0, 0.5,   0.5, 0, 0.5,   0.5, 1, 0,
    -0.5, 0, 0.5,   0.5, 1, 0,    -0.5, 1, 0,
    // slope -z
    0.5, 0, -0.5,  -0.5, 0, -0.5, -0.5, 1, 0,
    0.5, 0, -0.5,  -0.5, 1, 0,    0.5, 1, 0,
    // gable +x
    0.5, 0, 0.5,   0.5, 0, -0.5,  0.5, 1, 0,
    // gable -x
    -0.5, 0, -0.5, -0.5, 0, 0.5,  -0.5, 1, 0
  ];
  geo.setAttribute('position', new THREE.Float32BufferAttribute(v, 3));
  geo.computeVertexNormals();
  return geo;
}
const roofGeo = makeRoofGeo();

export function makeHouse(slot: HouseSlot): THREE.Group {
  const group = new THREE.Group();
  const { width, depth, height, variant, stories } = slot;

  const walls = new THREE.Mesh(wallGeo, wallMats[variant % wallMats.length]);
  walls.scale.set(width, height, depth);
  walls.position.y = height / 2;
  group.add(walls);

  // Two-story town buildings stack a slightly inset upper floor
  let roofBase = height;
  let roofW = width;
  let roofD = depth;
  if (stories > 1) {
    const upper = new THREE.Mesh(wallGeo, wallMats[(variant + 2) % wallMats.length]);
    const uw = width * 0.82;
    const ud = depth * 0.82;
    const uh = height * 0.85;
    upper.scale.set(uw, uh, ud);
    upper.position.y = height + uh / 2;
    group.add(upper);
    roofBase = height + uh;
    roofW = uw;
    roofD = ud;
  }

  const roof = new THREE.Mesh(roofGeo, roofMats[(variant + 1) % roofMats.length]);
  roof.scale.set(roofW * 1.15, height * 0.55, roofD * 1.15);
  roof.position.y = roofBase;
  group.add(roof);

  // Door on the +z face (which faces the village center via slot.rot)
  const door = new THREE.Mesh(slabGeo, doorMat);
  door.scale.set(0.8, 1.5, 1);
  door.position.set(0, 0.75, depth / 2 + 0.02);
  group.add(door);

  // A window either side of the door
  for (const side of [-1, 1]) {
    const win = new THREE.Mesh(slabGeo, windowMat);
    win.scale.set(0.6, 0.6, 1);
    win.position.set((side * width) / 3.2, height * 0.55, depth / 2 + 0.02);
    group.add(win);
  }

  group.rotation.y = slot.rot;
  return group;
}
