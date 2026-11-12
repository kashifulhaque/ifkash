// The Three.js scene: renderer, follow camera, a gridded ground plane, one capsule per actor.

import {
  AmbientLight,
  BufferGeometry,
  CapsuleGeometry,
  Color,
  DirectionalLight,
  GridHelper,
  Material,
  Mesh,
  MeshStandardMaterial,
  PerspectiveCamera,
  PlaneGeometry,
  Scene,
  SphereGeometry,
  Vector3,
  WebGLRenderer
} from 'three';
import { ActorRenderer } from '../render/actors';
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

const CAPSULE_RADIUS = 0.35;
const CAPSULE_LENGTH = 1.0;
const CAPSULE_HEIGHT = CAPSULE_LENGTH + CAPSULE_RADIUS * 2;

/** Camera distance from the followed actor, and its resting elevation angle (6 up, 8 back). */
const CAM_DISTANCE = 10;
const CAM_BASE_PITCH = Math.atan2(6, 8);

export class SceneView {
  readonly renderer: WebGLRenderer;
  private scene = new Scene();
  private camera = new PerspectiveCamera(55, 1, 0.1, 400);
  private meshes = new Map<number, Mesh>();
  private disposables: Array<BufferGeometry | Material> = [];
  private camTarget = new Vector3();
  private camInit = false;
  private camYaw = 0;
  private camPitch = 0;
  private previewOffset = new Vector3();
  private canvas: HTMLCanvasElement;
  private resizeObserver: ResizeObserver;
  /** The Wayfarer model. Capsules draw until it loads (or if it fails to). */
  private actors: ActorRenderer;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.renderer = new WebGLRenderer({ canvas, antialias: true });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

    const bg = token('--bg', '#f4f1ea');
    const line = token('--line-strong', '#bfb6a6');
    const surface = token('--surface-2', '#ebe6dc');
    const fjord = token('--fjord', '#2f5d74');
    this.scene.background = bg;

    const ground = new Mesh(new PlaneGeometry(200, 200), new MeshStandardMaterial({ color: surface }));
    ground.rotation.x = -Math.PI / 2;
    this.scene.add(ground);
    this.disposables.push(ground.geometry, ground.material as Material);

    // 2 m cells; sits a hair above the plane to avoid z-fighting.
    const grid = new GridHelper(200, 100, line, line);
    grid.position.y = 0.01;
    this.scene.add(grid);
    this.disposables.push(grid.geometry, grid.material as Material);

    this.scene.add(new AmbientLight(0xffffff, 1.4));
    const sun = new DirectionalLight(0xffffff, 2.2);
    sun.position.set(-6, 12, 8);
    this.scene.add(sun);

    this.capsuleGeometry = new CapsuleGeometry(CAPSULE_RADIUS, CAPSULE_LENGTH, 6, 16);
    this.noseGeometry = new SphereGeometry(0.12, 12, 8);
    this.bodyMaterial = new MeshStandardMaterial({ color: fjord, roughness: 0.7 });
    this.noseMaterial = new MeshStandardMaterial({ color: token('--clay', '#a4502f'), roughness: 0.6 });
    this.disposables.push(this.capsuleGeometry, this.noseGeometry, this.bodyMaterial, this.noseMaterial);

    this.actors = new ActorRenderer(this.scene, this.renderer);
    this.actors.load().catch((err) => console.error('Latticefall actor assets failed to load; keeping capsules', err));

    this.resizeObserver = new ResizeObserver(() => this.resize());
    this.resizeObserver.observe(canvas);
    this.resize();
  }

  private capsuleGeometry: CapsuleGeometry;
  private noseGeometry: SphereGeometry;
  private bodyMaterial: MeshStandardMaterial;
  private noseMaterial: MeshStandardMaterial;

  resize(): void {
    const w = this.canvas.clientWidth || 1;
    const h = this.canvas.clientHeight || 1;
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
  }

  /** Orbits the camera around the followed actor. Yaw 0 puts it on the +Z side. */
  setCamera(yaw: number, pitch: number): void {
    this.camYaw = yaw;
    this.camPitch = pitch;
  }

  /** Draws the given actors; the first one is followed by the camera. */
  render(actors: DrawActor[], frameMs: number): void {
    const live = new Set<number>();
    const modelsReady = this.actors.ready;
    for (const a of actors) {
      live.add(a.id);
      if (modelsReady && this.actors.update(a, frameMs)) continue;
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
    if (modelsReady) this.actors.prune(live);
    for (const [id, mesh] of this.meshes) {
      if (!live.has(id) || modelsReady) {
        this.scene.remove(mesh);
        this.meshes.delete(id);
      }
    }

    const focus = actors[0];
    if (focus) {
      const goal = new Vector3(focus.x, focus.y, focus.z);
      // Frame-rate independent smoothing of the followed point.
      const k = this.camInit ? 1 - Math.exp(-frameMs / 120) : 1;
      this.camTarget.lerp(goal, k);
      this.camInit = true;
      // The dev animation preview (?animcam=) pins the camera; otherwise orbit by input yaw and pitch.
      const off = this.actors.previewCameraOffset();
      if (off) {
        this.camera.position.copy(this.camTarget).add(this.previewOffset.set(...off));
      } else {
        const p = CAM_BASE_PITCH + this.camPitch;
        this.camera.position.copy(this.camTarget).add(
          new Vector3(Math.sin(this.camYaw) * Math.cos(p), Math.sin(p), Math.cos(this.camYaw) * Math.cos(p)).multiplyScalar(
            CAM_DISTANCE
          )
        );
      }
      this.camera.lookAt(this.camTarget.x, this.camTarget.y + 0.8, this.camTarget.z);
    }
    this.renderer.render(this.scene, this.camera);
  }

  dispose(): void {
    this.resizeObserver.disconnect();
    for (const mesh of this.meshes.values()) this.scene.remove(mesh);
    this.meshes.clear();
    this.actors.dispose();
    for (const d of this.disposables) d.dispose();
    this.disposables = [];
    this.renderer.dispose();
  }
}
