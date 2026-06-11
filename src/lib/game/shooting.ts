import * as THREE from 'three';
import type { Npc } from './npcs';

const PARTICLE_COUNT = 24;
const GRAVITY = -14;
const TRACER_POOL = 6;

export type FireResult = { npc: Npc; killed: boolean; headshot: boolean };

type Particle = { mesh: THREE.Mesh; vel: THREE.Vector3; life: number };
type Tracer = { line: THREE.Line; life: number };

export class Shooter {
  private raycaster = new THREE.Raycaster();
  private particles: Particle[] = [];
  private pool: THREE.Mesh[] = [];
  private tracers: Tracer[] = [];

  constructor(scene: THREE.Scene) {
    for (let i = 0; i < TRACER_POOL; i++) {
      const geo = new THREE.BufferGeometry().setFromPoints([
        new THREE.Vector3(),
        new THREE.Vector3()
      ]);
      const line = new THREE.Line(
        geo,
        new THREE.LineBasicMaterial({ color: 0xfff2a8, transparent: true, opacity: 0.9 })
      );
      line.visible = false;
      line.frustumCulled = false;
      scene.add(line);
      this.tracers.push({ line, life: 0 });
    }

    for (let i = 0; i < PARTICLE_COUNT; i++) {
      const m = new THREE.Mesh(
        new THREE.BoxGeometry(0.09, 0.09, 0.09),
        new THREE.MeshBasicMaterial({ color: 0xffffff })
      );
      m.visible = false;
      scene.add(m);
      this.pool.push(m);
    }
  }

  private showTracer(from: THREE.Vector3, to: THREE.Vector3, color: number) {
    const t = this.tracers.find((t) => !t.line.visible) ?? this.tracers[0];
    const pos = t.line.geometry.attributes.position as THREE.BufferAttribute;
    pos.setXYZ(0, from.x, from.y, from.z);
    pos.setXYZ(1, to.x, to.y, to.z);
    pos.needsUpdate = true;
    (t.line.material as THREE.LineBasicMaterial).color.set(color);
    t.line.visible = true;
    t.life = 0.07;
  }

  // Player hitscan from the screen-center crosshair. Tracer starts at the
  // gun's real muzzle when provided.
  fire(
    camera: THREE.Camera,
    hittables: THREE.Object3D[],
    muzzlePos?: THREE.Vector3
  ): FireResult | null {
    this.raycaster.setFromCamera(new THREE.Vector2(0, 0), camera);
    const hits = this.raycaster.intersectObjects(hittables, false);

    const origin = new THREE.Vector3();
    camera.getWorldPosition(origin);
    const dir = this.raycaster.ray.direction;
    const muzzle =
      muzzlePos ??
      origin
        .clone()
        .add(dir.clone().multiplyScalar(0.8))
        .add(new THREE.Vector3(0, -0.15, 0));

    let result: FireResult | null = null;
    let end: THREE.Vector3;
    if (hits.length > 0) {
      end = hits[0].point;
      const npc = (hits[0].object.userData.npc as Npc) ?? null;
      this.burst(end, npc ? npc.section.color : 0xffffff);
      if (npc) {
        const headshot = npc.isHeadshot(end);
        const killed = npc.hit(headshot ? 2 : 1);
        result = { npc, killed, headshot };
      }
    } else {
      end = origin.clone().add(dir.clone().multiplyScalar(60));
    }

    this.showTracer(muzzle, end, 0xfff2a8);
    return result;
  }

  // Enemy fire visual (red tracer + impact sparks on hit).
  enemyShot(from: THREE.Vector3, to: THREE.Vector3, hit: boolean) {
    this.showTracer(from, to, 0xff5544);
    if (hit) this.burst(to, 0xff5544);
  }

  private burst(at: THREE.Vector3, color: number) {
    let used = 0;
    for (const mesh of this.pool) {
      if (mesh.visible || used >= 10) continue;
      used++;
      mesh.visible = true;
      mesh.position.copy(at);
      (mesh.material as THREE.MeshBasicMaterial).color.set(color);
      this.particles.push({
        mesh,
        vel: new THREE.Vector3(
          (Math.random() - 0.5) * 6,
          Math.random() * 5 + 1,
          (Math.random() - 0.5) * 6
        ),
        life: 0.6 + Math.random() * 0.3
      });
    }
  }

  update(dt: number) {
    for (const t of this.tracers) {
      if (t.life > 0) {
        t.life -= dt;
        if (t.life <= 0) t.line.visible = false;
      }
    }
    for (let i = this.particles.length - 1; i >= 0; i--) {
      const p = this.particles[i];
      p.life -= dt;
      if (p.life <= 0) {
        p.mesh.visible = false;
        this.particles.splice(i, 1);
        continue;
      }
      p.vel.y += GRAVITY * dt;
      p.mesh.position.addScaledVector(p.vel, dt);
      p.mesh.rotation.x += dt * 8;
      p.mesh.rotation.y += dt * 6;
    }
  }
}
