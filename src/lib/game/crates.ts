import * as THREE from 'three';
import type { Section } from '$lib/content';
import { loadModel, cloneStatic } from './assets';
import { normalizeHeight } from './chunks';
import { terrainHeight } from './terrain';

export const INTERACT_RANGE = 2.8;
export const CRATE_DESPAWN_DIST = 60;

export class LootCrate {
  section: Section;
  group = new THREE.Group();
  private time = 0;
  private baseY = 0;

  constructor(section: Section, x: number, z: number) {
    this.section = section;

    const accent = new THREE.Color(section.color);
    // Proper crate model, topped with a glowing accent lid
    loadModel('/models/props/Crate.glb').then((gltf) => {
      const crate = cloneStatic(gltf);
      normalizeHeight(crate, 0.85);
      this.group.add(crate);
    });
    const lid = new THREE.Mesh(
      new THREE.BoxGeometry(0.7, 0.12, 0.7),
      new THREE.MeshLambertMaterial({ color: accent, flatShading: true, emissive: accent, emissiveIntensity: 0.5 })
    );
    lid.position.y = 0.95;

    // Light beam pillar (PUBG drop style)
    const beam = new THREE.Mesh(
      new THREE.CylinderGeometry(0.32, 0.5, 9, 8, 1, true),
      new THREE.MeshBasicMaterial({
        color: accent,
        transparent: true,
        opacity: 0.22,
        blending: THREE.AdditiveBlending,
        side: THREE.DoubleSide,
        depthWrite: false
      })
    );
    beam.position.y = 4.5;

    this.group.add(lid, beam);
    this.baseY = terrainHeight(x, z);
    this.group.position.set(x, this.baseY, z);
  }

  update(dt: number) {
    this.time += dt;
    this.group.rotation.y += dt * 0.8;
    this.group.position.y = this.baseY + Math.sin(this.time * 2.5) * 0.08;
  }

  distanceTo(x: number, z: number): number {
    return Math.hypot(this.group.position.x - x, this.group.position.z - z);
  }
}
