// The Three.js scene: renderer, the third-person camera (see render/camera), a gridded ground plane, one capsule per actor.

import {
  AmbientLight,
  BufferGeometry,
  CapsuleGeometry,
  Color,
  CylinderGeometry,
  DirectionalLight,
  GridHelper,
  Material,
  Mesh,
  MeshStandardMaterial,
  Object3D,
  PerspectiveCamera,
  PlaneGeometry,
  Scene,
  SphereGeometry,
  Vector3,
  WebGLRenderer
} from 'three';
import { ActorRenderer } from '../render/actors';
import { digestMoveResolver } from '../render/actors/moveTable';
import { CombatVfx } from '../render/vfx';
import { LevelRenderer } from '../render/terrain';
import { PropsRenderer } from '../render/props';
import { VossArena } from '../render/bosses/b07/arena';
import { getQuality, pixelRatioFor, subscribeQuality } from '../render/lighting';
import {
  CameraRig,
  TEST_FIELD_BOUNDS,
  buildLevelProxy,
  exposeDebug,
  loadCameraSettings,
  scriptPose,
  type DevCameraOptions,
  type RaycastFn
} from '../render/camera';
import type { Equipped } from '../render/equipment/playerGear';
import type { DrawActor } from './snapshot';

/** Reads a CSS custom property from `<html>` and falls back if it is missing or unparsable. */
function token(name: string, fallback: string): Color {
  try {
    const raw = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    if (raw) return new Color(raw);
  } catch {
    // Fall through to the fallback.
  }
  return new Color(fallback);
}

const ORIGIN = { x: 0, y: 0, z: 0 };
const CAPSULE_RADIUS = 0.35;
const CAPSULE_LENGTH = 1.0;
const CAPSULE_HEIGHT = CAPSULE_LENGTH + CAPSULE_RADIUS * 2;

export class SceneView {
  readonly renderer: WebGLRenderer;
  /** Exposed for the dev tools' debug draw; nothing else adds to it from outside. */
  readonly scene = new Scene();
  private camera = new PerspectiveCamera(60, 1, 0.1, 400);
  private rig: CameraRig;
  private dev: DevCameraOptions | null;
  private scriptT = 0;
  private scriptStage = -1;
  private lastActors: DrawActor[] = [];
  private meshes = new Map<number, Mesh>();
  private disposables: Array<BufferGeometry | Material> = [];
  /** The level renderer, once `loadLevel` finds a manifest. Null for the flat scenarios. */
  private level: LevelRenderer | null = null;
  /** Ladders, lifts, doors, levers, pickups and the other world mechanics (WP-042). */
  readonly props = new PropsRenderer(this.scene);
  /** B07's chandeliers and, with `&dev=1`, its counts readout (WP-075). */
  readonly voss: VossArena;
  /** The stand-in ground, proxy and lights that the level renderer replaces. */
  private standIns: Object3D[] = [];
  private offQuality: () => void;
  private camTarget = new Vector3();
  private camInit = false;
  private camYaw = 0;
  private camPitch = 0;
  /** The yaw the camera actually used last frame. Differs from the input yaw while locked on. */
  cameraYaw = 0;
  /** True while the camera is driving yaw (lock-on), so input should follow `cameraYaw`. */
  yawDriven = false;
  private previewOffset = new Vector3();
  /** Photo mode's free camera (WP-088); null while the game camera drives. */
  private photoPose: { x: number; y: number; z: number; yaw: number; pitch: number } | null = null;
  private canvas: HTMLCanvasElement;
  private resizeObserver: ResizeObserver;
  /** The Wayfarer model. Capsules draw until it loads (or if it fails to). */
  private actors: ActorRenderer;
  /** Dev scenes (the armory) draw once per frame, right before the scene renders. */
  readonly frameHooks: Array<(dt: number, camera: PerspectiveCamera, viewportPx: number) => void> = [];
  /** Trails, sparks, hitstop, camera shake and auras. The engine feeds it the core's events. */
  readonly vfx: CombatVfx;

  constructor(canvas: HTMLCanvasElement, raycast: RaycastFn, dev: DevCameraOptions | null = null) {
    this.canvas = canvas;
    this.dev = dev;
    this.rig = new CameraRig(this.camera, raycast, loadCameraSettings());
    if (dev?.fakeLock) this.rig.fakeTarget = dev.fakeLock;
    this.renderer = new WebGLRenderer({ canvas, antialias: true });
    this.renderer.setPixelRatio(pixelRatioFor(getQuality(), window.devicePixelRatio));
    this.offQuality = subscribeQuality((p) => {
      this.renderer.setPixelRatio(pixelRatioFor(p, window.devicePixelRatio));
      this.resize();
    });

    const bg = token('--bg', '#f4f1ea');
    const line = token('--line-strong', '#bfb6a6');
    const surface = token('--surface-2', '#ebe6dc');
    const fjord = token('--fjord', '#2f5d74');
    this.scene.background = bg;

    const ground = new Mesh(new PlaneGeometry(200, 200), new MeshStandardMaterial({ color: surface }));
    ground.rotation.x = -Math.PI / 2;
    this.scene.add(ground);
    this.standIns.push(ground);
    this.disposables.push(ground.geometry, ground.material as Material);

    // 2 m cells; sits a hair above the plane to avoid z-fighting.
    const grid = new GridHelper(200, 100, line, line);
    grid.position.y = 0.01;
    this.scene.add(grid);
    this.standIns.push(grid);
    this.disposables.push(grid.geometry, grid.material as Material);

    // Dev camera runs on the test field draw a raycast-built copy of it instead of the flat plane.
    if (dev?.active && dev.scenario === 'test_field') {
      const proxy = buildLevelProxy(
        raycast,
        TEST_FIELD_BOUNDS,
        token('--moss', '#6b7f5e').lerp(surface, 0.55),
        token('--clay', '#a4502f')
      );
      this.scene.add(proxy);
      this.standIns.push(proxy);
      this.disposables.push(proxy.geometry, proxy.material as Material);
      ground.visible = false;
      grid.visible = false;
    }
    if (dev?.fakeLock) {
      // A stand-in for the locked actor: a post the size the camera assumes for a target.
      const post = new Mesh(new CylinderGeometry(0.8, 0.8, 2.2, 20), new MeshStandardMaterial({ color: token('--falu', '#8b2e1f') }));
      post.position.set(dev.fakeLock.x, dev.fakeLock.y + 1.1, dev.fakeLock.z);
      this.scene.add(post);
      this.disposables.push(post.geometry, post.material as Material);
    }
    if (dev?.active) exposeDebug(this.rig, () => this.lastActors);

    const ambient = new AmbientLight(0xffffff, 1.4);
    const sun = new DirectionalLight(0xffffff, 2.2);
    sun.position.set(-6, 12, 8);
    this.scene.add(ambient, sun);
    this.standIns.push(ambient, sun);

    this.capsuleGeometry = new CapsuleGeometry(CAPSULE_RADIUS, CAPSULE_LENGTH, 6, 16);
    this.noseGeometry = new SphereGeometry(0.12, 12, 8);
    this.bodyMaterial = new MeshStandardMaterial({ color: fjord, roughness: 0.7 });
    this.noseMaterial = new MeshStandardMaterial({ color: token('--clay', '#a4502f'), roughness: 0.6 });
    this.disposables.push(this.capsuleGeometry, this.noseGeometry, this.bodyMaterial, this.noseMaterial);

    const resolveMove = digestMoveResolver();
    this.actors = new ActorRenderer(this.scene, this.renderer, { resolveMove });
    this.vfx = new CombatVfx(this.scene, this.actors, { resolveMove });
    this.voss = new VossArena(this.scene, canvas.parentElement);
    this.actors.load().catch((err) => console.error('Latticefall actor assets failed to load; keeping capsules', err));

    this.resizeObserver = new ResizeObserver(() => this.resize());
    this.resizeObserver.observe(canvas);
    this.resize();
  }

  private capsuleGeometry: CapsuleGeometry;
  private noseGeometry: SphereGeometry;
  private bodyMaterial: MeshStandardMaterial;
  private noseMaterial: MeshStandardMaterial;

  /**
   * Draws `scenario`'s level from its render manifest (static/latticefall/levels/NAME.json).
   * Areas without a manifest keep the stand-in ground. Safe to call once, after construction.
   */
  async loadLevel(scenario: string): Promise<void> {
    this.props.load(scenario).catch((err) => console.error('Latticefall props failed to load', err));
    const level = new LevelRenderer(this.scene, this.renderer);
    try {
      if (!(await level.load(scenario))) return level.dispose();
    } catch (err) {
      console.error('Latticefall level failed to load; keeping the stand-in ground', err);
      return level.dispose();
    }
    this.level = level;
    for (const o of this.standIns) o.visible = false;
  }

  resize(): void {
    const w = this.canvas.clientWidth || 1;
    const h = this.canvas.clientHeight || 1;
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.rig.applyFov();
  }

  /** Where a world point lands on the canvas in CSS pixels, or null behind the camera (for the HUD). */
  project(x: number, y: number, z: number): { x: number; y: number } | null {
    this.projected.set(x, y, z).project(this.camera);
    if (this.projected.z > 1) return null;
    const w = this.canvas.clientWidth || 1;
    const h = this.canvas.clientHeight || 1;
    return { x: (this.projected.x * 0.5 + 0.5) * w, y: (-this.projected.y * 0.5 + 0.5) * h };
  }

  private projected = new Vector3();

  /** Sets the look input: yaw 0 puts the camera on the +Z side; pitch is elevation above the horizon. */
  /** Sets what the player wears: registry numbers from the menu state (WP-058). */
  setEquipment(eq: Equipped): void {
    this.actors.setEquipment(eq);
  }

  /** Draws the player's gesture (WP-088): the core's gesture number, or null for none. */
  setGesture(gesture: number | null): void {
    this.actors.setGesture(gesture);
  }

  /**
   * Puts the camera at a fixed pose for photo mode (WP-088), or with null hands it back to the
   * game camera. Yaw 0 looks along +Z; positive pitch looks down.
   */
  setPhotoPose(pose: { x: number; y: number; z: number; yaw: number; pitch: number } | null): void {
    this.photoPose = pose;
    if (!pose) this.rig.reset();
  }

  /** Where the camera is and which way it looks, in the same terms as `setPhotoPose`. */
  cameraPose(): { x: number; y: number; z: number; yaw: number; pitch: number } {
    const d = this.camera.getWorldDirection(new Vector3());
    return {
      x: this.camera.position.x,
      y: this.camera.position.y,
      z: this.camera.position.z,
      yaw: Math.atan2(d.x, d.z),
      pitch: -Math.asin(Math.max(-1, Math.min(1, d.y)))
    };
  }

  setCamera(yaw: number, pitch: number): void {
    this.camYaw = yaw;
    this.camPitch = pitch;
  }

  /**
   * Draws the given actors; the first one is the player, whom the camera follows. While
   * `simPaused` the VFX hold their clock (sparks stay where they are) but still redraw.
   */
  render(actors: DrawActor[], frameMs: number, simPaused = false): void {
    const live = new Set<number>();
    const modelsReady = this.actors.ready;
    for (const a of actors) {
      live.add(a.id);
      if (this.actors.update(a, frameMs)) continue;
      let mesh = this.meshes.get(a.id);
      if (!mesh) {
        mesh = new Mesh(this.capsuleGeometry, this.bodyMaterial);
        // A nose on the +Z side (yaw 0 faces +Z) so facing is visible.
        const nose = new Mesh(this.noseGeometry, this.noseMaterial);
        nose.position.set(0, 0.45, CAPSULE_RADIUS);
        mesh.add(nose);
        this.scene.add(mesh);
        this.meshes.set(a.id, mesh);
      }
      mesh.position.set(a.x, a.y + CAPSULE_HEIGHT / 2, a.z);
      mesh.rotation.y = a.yaw;
    }
    this.actors.playerId = actors[0]?.id ?? -1;
    this.actors.prune(live);
    this.actors.tickPreview(frameMs);
    for (const [id, mesh] of this.meshes) {
      if (!live.has(id) || modelsReady) {
        this.scene.remove(mesh);
        this.meshes.delete(id);
      }
    }

    this.lastActors = actors;
    const focus = actors[0];
    if (this.photoPose) {
      // Photo mode (WP-088): the free camera, wherever the player put it.
      const { x, y, z, yaw, pitch } = this.photoPose;
      const cp = Math.cos(pitch);
      this.camera.position.set(x, y, z);
      this.camera.lookAt(x + Math.sin(yaw) * cp, y - Math.sin(pitch), z + Math.cos(yaw) * cp);
      this.yawDriven = false;
    } else if (this.dev?.fixed) {
      // `&cam=fixed:...`: the camera stays put and ignores the player and the look input.
      const { pos, yaw, pitch } = this.dev.fixed;
      const cp = Math.cos(pitch);
      this.camera.position.set(pos[0], pos[1], pos[2]);
      this.camera.lookAt(pos[0] + Math.sin(yaw) * cp, pos[1] - Math.sin(pitch), pos[2] + Math.cos(yaw) * cp);
      this.yawDriven = false;
    } else if (focus) {
      // The dev animation preview (?animcam=) pins the camera to the actor.
      const off = this.actors.previewCameraOffset();
      if (off) {
        const goal = this.previewOffset.set(focus.x, focus.y, focus.z);
        // Frame-rate independent smoothing of the followed point.
        const k = this.camInit ? 1 - Math.exp(-frameMs / 120) : 1;
        this.camTarget.lerp(goal, k);
        this.camInit = true;
        this.camera.position.set(this.camTarget.x + off[0], this.camTarget.y + off[1], this.camTarget.z + off[2]);
        this.camera.lookAt(this.camTarget.x, this.camTarget.y + 0.8, this.camTarget.z);
        this.yawDriven = false;
      } else if (this.dev?.script) {
        // The dev orbit script ignores input and the player; it sweeps pivots around the ledge.
        this.scriptT = this.dev.scriptAt ?? this.scriptT + frameMs / 1000;
        const s = scriptPose(this.scriptT);
        if (s.stage !== this.scriptStage) {
          this.scriptStage = s.stage;
          this.rig.reset(); // The pivots are far apart; don't glide between them.
        }
        this.rig.update(actors, s.yaw, s.pitch, frameMs, s.pivot);
        this.yawDriven = false;
      } else {
        const pose = this.rig.update(actors, this.camYaw, this.camPitch, frameMs);
        this.cameraYaw = pose.yaw;
        this.yawDriven = pose.locked;
      }
    }
    if (this.level) {
      this.level.setVisibility(this.props.visibility);
      this.level.update(this.camera, focus ?? ORIGIN, frameMs);
    }
    if (this.props.active) this.props.update(simPaused ? 0 : frameMs);
    this.vfx.update(actors, simPaused ? 0 : Math.min(frameMs, 100) / 1000, this.camera, this.canvas.clientHeight || 1);
    this.voss.update(simPaused ? 0 : Math.min(frameMs, 100), actors);
    this.vfx.applyCamera(this.camera);
    const dtSeconds = simPaused ? 0 : Math.min(frameMs, 100) / 1000;
    this.actors.updateFx(dtSeconds, this.camera, this.canvas.clientHeight || 1);
    for (const hook of this.frameHooks) hook(Math.min(frameMs, 100) / 1000, this.camera, this.canvas.clientHeight || 1);
    this.renderer.render(this.scene, this.camera);
  }

  dispose(): void {
    this.resizeObserver.disconnect();
    for (const mesh of this.meshes.values()) this.scene.remove(mesh);
    this.meshes.clear();
    this.offQuality();
    this.level?.dispose();
    this.level = null;
    this.props.dispose();
    this.vfx.dispose();
    this.voss.dispose();
    this.actors.dispose();
    for (const d of this.disposables) d.dispose();
    this.disposables = [];
    this.renderer.dispose();
  }
}
