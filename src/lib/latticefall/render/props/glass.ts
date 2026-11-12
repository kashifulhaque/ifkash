// Glasswing Grotto's props (WP-063): beam emitters, crystal mirrors, receptor sockets, and the
// light beams between them. Greybox, drawn from the level manifest's `BeamSource`, `Mirror`, and
// `Receptor` entities and the core's state words (each mirror's facing, each receptor's lit flag, and
// each beam's legs, in `lf_core::world_state::mechanics::light_beam::render_words`). The beam's
// path is the core's: nothing here traces a ray. Read-only: nothing here writes simulation state.
// The art pass swaps the geometry, not the plumbing, the same as the other props.

import {
  AdditiveBlending,
  BoxGeometry,
  Color,
  CylinderGeometry,
  Group,
  Mesh,
  MeshBasicMaterial,
  MeshStandardMaterial,
  OctahedronGeometry,
  type BufferGeometry,
  type Material,
  type Object3D
} from 'three';

type V3 = [number, number, number];

/** Must match `light_beam::MAX_SEGMENTS`: the legs each beam sends. */
export const MAX_BEAM_LEGS = 8;
/** Words per beam: the leg count, then `MAX_BEAM_LEGS` legs of `x0, z0, x1, z1`. */
const BEAM_WORDS = 1 + 4 * MAX_BEAM_LEGS;
/** The beam's thickness in meters, and the lit receptor's glow color. */
const BEAM_WIDTH = 0.12;
const LIT = '#d8fff0';
const DARK = '#3d5a70';
const BEAM = '#7fe8ff';
const LIT_COLOR = new Color(LIT);
const DARK_COLOR = new Color(DARK);
const BEAM_COLOR = new Color(BEAM);
/** How fast a mirror turns to its facing, and the glow eases (milliseconds, as a time constant). */
const TURN_MS = 90;

interface Mirror {
  pivot: Group;
  /** The yaw it shows now, and the one it is turning to. */
  yaw: number;
}

interface Socket {
  crystal: Mesh;
  material: MeshStandardMaterial;
  glow: number;
}

interface Beam {
  y: number;
  legs: Mesh[];
}

/** The yaw (from +Z toward +X) of a mirror's face for a facing 0..3: the normals (1,1), (1,-1), (-1,-1), (-1,1). */
export function facingYaw(facing: number): number {
  const f = ((Math.round(facing) % 4) + 4) % 4;
  return [Math.PI / 4, (3 * Math.PI) / 4, (-3 * Math.PI) / 4, -Math.PI / 4][f];
}

function ease(cur: number, goal: number, dt: number, ms: number): number {
  return cur + (goal - cur) * (1 - Math.exp(-dt / ms));
}

/** The shortest signed turn from `a` to `b`, in radians. */
function turnBetween(a: number, b: number): number {
  let d = (b - a) % (2 * Math.PI);
  if (d > Math.PI) d -= 2 * Math.PI;
  if (d < -Math.PI) d += 2 * Math.PI;
  return d;
}

export class GlassProps {
  private geometries: BufferGeometry[] = [];
  private materials: Material[] = [];
  private mirrors: Mirror[] = [];
  private sockets: Socket[] = [];
  private beams: Beam[] = [];
  readonly group = new Group();

  constructor(parent: Object3D) {
    this.group.name = 'latticefall-glass';
    parent.add(this.group);
  }

  get mirrorCount(): number {
    return this.mirrors.length;
  }

  get socketCount(): number {
    return this.sockets.length;
  }

  get beamCount(): number {
    return this.beams.length;
  }

  add(kind: 'BeamSource' | 'Mirror' | 'Receptor', d: Record<string, unknown>): void {
    if (kind === 'BeamSource') this.source(d);
    else if (kind === 'Mirror') this.mirror(d);
    else this.receptor(d);
  }

  /**
   * Moves the props to the state words. `mirrorAt`, `receptorAt`, and `beamAt` are the offsets of the first
   * mirror word, the first receptor word, and the first beam's words in `state`.
   */
  update(dtMs: number, state: ArrayLike<number>, mirrorAt: number, receptorAt: number, beamAt: number): void {
    this.mirrors.forEach((m, i) => {
      const word = state[mirrorAt + i];
      if (word === undefined) return;
      const goal = facingYaw(word);
      m.yaw += turnBetween(m.yaw, goal) * (1 - Math.exp(-dtMs / TURN_MS));
      m.pivot.rotation.y = m.yaw;
    });
    this.sockets.forEach((s, i) => {
      const word = state[receptorAt + i];
      if (word === undefined) return;
      s.glow = ease(s.glow, word > 0.5 ? 1 : 0, dtMs, 160);
      s.material.color.copy(DARK_COLOR).lerp(LIT_COLOR, s.glow);
      s.material.emissive.copy(BEAM_COLOR).multiplyScalar(0.15 + 0.85 * s.glow);
      s.crystal.rotation.y += dtMs * 0.0008 * (0.2 + s.glow);
    });
    this.beams.forEach((b, i) => {
      const at = beamAt + i * BEAM_WORDS;
      const count = state[at];
      if (count === undefined) return;
      b.legs.forEach((leg, k) => {
        if (k >= count) {
          leg.visible = false;
          return;
        }
        const x0 = state[at + 1 + 4 * k];
        const z0 = state[at + 2 + 4 * k];
        const x1 = state[at + 3 + 4 * k];
        const z1 = state[at + 4 + 4 * k];
        const len = Math.hypot(x1 - x0, z1 - z0);
        if (len < 0.01) {
          leg.visible = false;
          return;
        }
        leg.visible = true;
        leg.position.set((x0 + x1) / 2, b.y, (z0 + z1) / 2);
        leg.rotation.y = Math.atan2(x1 - x0, z1 - z0);
        leg.scale.set(1, 1, len);
      });
    });
  }

  dispose(): void {
    this.group.parent?.remove(this.group);
    for (const g of this.geometries) g.dispose();
    for (const m of this.materials) m.dispose();
    this.geometries = [];
    this.materials = [];
    this.mirrors = [];
    this.sockets = [];
    this.beams = [];
  }

  // -------------------------------------------------------------------------------------

  private geo<T extends BufferGeometry>(g: T): T {
    this.geometries.push(g);
    return g;
  }

  private mat<T extends Material>(m: T): T {
    this.materials.push(m);
    return m;
  }

  private stone(): MeshStandardMaterial {
    return this.mat(new MeshStandardMaterial({ color: new Color('#46505e'), roughness: 0.9, metalness: 0.05 }));
  }

  /** An emitter: a crystal on a short plinth, and the pool of beam segments it shines. */
  private source(d: Record<string, unknown>): void {
    const pos = d.pos as V3;
    const g = new Group();
    g.position.set(pos[0], pos[1], pos[2]);
    const post = new Mesh(this.geo(new CylinderGeometry(0.22, 0.3, pos[1], 8)), this.stone());
    post.position.y = -pos[1] / 2;
    const glass = this.mat(
      new MeshStandardMaterial({
        color: new Color(BEAM),
        emissive: new Color(BEAM),
        emissiveIntensity: 1.1,
        roughness: 0.2,
        metalness: 0.1
      })
    );
    const crystal = new Mesh(this.geo(new OctahedronGeometry(0.34)), glass);
    crystal.scale.y = 1.5;
    g.add(post, crystal);
    this.group.add(g);
    // One thin additive box per leg; the update moves, turns, and stretches them.
    const beamMat = this.mat(
      new MeshBasicMaterial({
        color: new Color(BEAM),
        transparent: true,
        opacity: 0.8,
        blending: AdditiveBlending,
        depthWrite: false
      })
    );
    const unit = this.geo(new BoxGeometry(BEAM_WIDTH, BEAM_WIDTH, 1));
    const legs: Mesh[] = [];
    for (let k = 0; k < MAX_BEAM_LEGS; k++) {
      const leg = new Mesh(unit, beamMat);
      leg.visible = false;
      legs.push(leg);
      this.group.add(leg);
    }
    this.beams.push({ y: pos[1], legs });
  }

  /** A mirror: a post and a thin crystal slab, whose face is the local +Z. */
  private mirror(d: Record<string, unknown>): void {
    const pos = d.pos as V3;
    const g = new Group();
    g.position.set(pos[0], pos[1], pos[2]);
    const post = new Mesh(this.geo(new CylinderGeometry(0.16, 0.24, 0.5, 8)), this.stone());
    post.position.y = 0.25;
    const pivot = new Group();
    pivot.position.y = 1.2;
    const glass = this.mat(
      new MeshStandardMaterial({
        color: new Color('#bfeaff'),
        emissive: new Color('#2a6f86'),
        emissiveIntensity: 0.6,
        roughness: 0.05,
        metalness: 0.7
      })
    );
    const slab = new Mesh(this.geo(new BoxGeometry(1.5, 1.5, 0.1)), glass);
    // The slab's back: a darker plate behind the face, so a player can tell which side reflects.
    const back = new Mesh(this.geo(new BoxGeometry(1.46, 1.46, 0.06)), this.stone());
    back.position.z = -0.08;
    pivot.add(slab, back);
    const facing = Number(d.facing ?? 0);
    const yaw = facingYaw(facing);
    pivot.rotation.y = yaw;
    g.add(post, pivot);
    this.group.add(g);
    this.mirrors.push({ pivot, yaw });
  }

  /** A receptor: a socket crystal on a post at the beam's height, glowing once lit. */
  private receptor(d: Record<string, unknown>): void {
    const pos = d.pos as V3;
    const g = new Group();
    g.position.set(pos[0], pos[1], pos[2]);
    const post = new Mesh(this.geo(new CylinderGeometry(0.2, 0.28, pos[1], 8)), this.stone());
    post.position.y = -pos[1] / 2;
    const material = this.mat(
      new MeshStandardMaterial({
        color: new Color(DARK),
        emissive: new Color(BEAM).multiplyScalar(0.15),
        roughness: 0.3,
        metalness: 0.1
      })
    );
    const crystal = new Mesh(this.geo(new OctahedronGeometry(0.42)), material);
    g.add(post, crystal);
    this.group.add(g);
    this.sockets.push({ crystal, material, glow: 0 });
  }
}
