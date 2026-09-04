import * as THREE from 'three';
import { Input } from './input';
import { Ambience } from './audio';
import { biomeAt, biomeById, isOcean, layoutBiomes } from './biomes';
import { setNoiseSeed } from './noise';
import { dailySeedLabel, hashSeed } from './seed';
import { PLANET_RADIUS, SEA_LEVEL, buildGround, buildStars, buildWater, tangentBasis, walkRadius, offsetDir } from './planet';
import { DayNight } from './daynight';
import { BOAT_SEAT, LAUNCH_RANGE, Wake, buildBoat, findLaunchPoint } from './boat';
import { buildFlame, buildWindmillBlades, buildWorldProps } from './props';
import { Character, buildBear, buildFox, buildSheep } from './character';
import { Critter } from './critters';
import { Clouds } from './clouds';
import { Aurora } from './aurora';
import { ColliderGrid } from './collision';
import { WONDERS, loadFound, saveFound, wonderDir, type Wonder } from './wonders';
import type { BiomeCaption, ClockState } from './store';

/** Interaction card. `kicker` overrides the wonder wording, for example for the boat. */
export type Prompt = { id: string; action: string; found: boolean; kicker?: string };

export type GameOptions = {
  /** World seed label. Defaults to today's planet. See `seed.ts`. */
  seed?: string;
};

export type GameCallbacks = {
  onBiome: (b: BiomeCaption | null) => void;
  onPrompt: (p: Prompt | null) => void;
  onFound: (ids: string[]) => void;
  onOpenWonder: (w: Wonder) => void;
  onGlobe: (globe: boolean) => void;
  onClock: (clock: ClockState) => void;
  onIntroEnd: () => void;
  onHelp: () => void;
  onEscape: () => void;
};

const WALK_SPEED = 5.4;
const RUN_SPEED = 9.8;
const TURN_RATE = 11;
const GRAVITY = 24;
const HOP_VELOCITY = 7.5;
const INTERACT_RANGE = 3.6;
/** Radius of the explorer's footprint, in surface units. */
const PLAYER_RADIUS = 0.35;
/** Seconds of no progress toward a click target before the walk is cancelled. */
const STUCK_TIMEOUT = 0.6;
const CAM_MIN = 5;
const CAM_MAX = 28;
const GLOBE_MARGIN = 10;
/** Boat cruising speed and the multiplier while "running". */
const BOAT_SPEED = 7;
const BOAT_RUN = 1.5;
/** Boat heading lerp rate, well below TURN_RATE so it swings wide. */
const BOAT_TURN = 2.2;
/** Per-second approach rates toward the target speed: throttle up, then coast down. */
const BOAT_ACCEL = 1.4;
const BOAT_DRAG = 0.9;

const _e = new THREE.Euler();

const _v1 = new THREE.Vector3();
const _v2 = new THREE.Vector3();
const _v3 = new THREE.Vector3();
const _m = new THREE.Matrix4();
const _q = new THREE.Quaternion();
const _size = new THREE.Vector2();

type Gem = { wonder: Wonder; dir: THREE.Vector3; mesh: THREE.Mesh; ring: THREE.Mesh; frameQ: THREE.Quaternion; base: THREE.Vector3; up: THREE.Vector3 };

export class Game {
  readonly input: Input;
  readonly audio = new Ambience();
  /** Label of the seed this world was generated from. */
  readonly seed: string;
  found: string[];

  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera: THREE.PerspectiveCamera;
  private sun: THREE.DirectionalLight;
  private hemi: THREE.HemisphereLight;
  private ground: THREE.Mesh;
  private water: THREE.Mesh;
  private stars: THREE.Points;
  private glowMat: THREE.MeshBasicMaterial | null = null;
  private nightMesh: THREE.Mesh | null = null;
  private dayNight = new DayNight();
  private clockNight = false;
  private raycaster = new THREE.Raycaster();
  private callbacks: GameCallbacks;
  private canvas: HTMLCanvasElement;

  // Player: a unit direction on the sphere plus a tangent heading.
  private dir = new THREE.Vector3();
  private heading = new THREE.Vector3();
  private hopH = 0;
  private hopV = 0;
  private airborne = false;
  private animSpeed = 0;
  private character = new Character();
  private walkTarget: THREE.Vector3 | null = null;
  private stuckTime = 0;
  private targetRing: THREE.Mesh;
  private colliders: ColliderGrid;

  // Boat: one rowboat that is launched from a shore, sailed, and docked where it lands.
  private boat: THREE.Group;
  private boatDir = new THREE.Vector3();
  private boatHeading = new THREE.Vector3();
  private boatSpeed = 0;
  private boatPlaced = false;
  private boating = false;
  private wake = new Wake();
  /** Nearest launchable water within reach, refreshed on the biome cadence. */
  private launchDir: THREE.Vector3 | null = null;

  // Camera: a tangent "forward" carried along with the player, pitch, distance.
  private camForward = new THREE.Vector3();
  private camPitch = 0.92;
  private camDist = 15;
  private camPos = new THREE.Vector3();
  private camLook = new THREE.Vector3();
  private camUp = new THREE.Vector3(0, 1, 0);
  private globe = true;
  private intro = true;
  private globeYaw = 0;
  private globePitch = 0.35;
  private globeZoom = 1;
  private transition = 0;
  private dragThisFrame = false;

  // Scenery that moves.
  private blades: THREE.Group;
  private flame: THREE.Group;
  private flameLight: THREE.PointLight;
  private lampLight: THREE.PointLight;
  private lampOn = false;
  private critters: Critter[] = [];
  private clouds: Clouds;
  private aurora: Aurora;
  private gems: Gem[] = [];

  private promptId: string | null = null;
  private biomeId: string | null = null;
  private biomeTimer = 0;
  private overlayOpen = false;
  private raf = 0;
  private last = 0;
  private time = 0;
  private disposed = false;

  constructor(canvas: HTMLCanvasElement, callbacks: GameCallbacks, options: GameOptions = {}) {
    this.canvas = canvas;
    this.callbacks = callbacks;
    this.found = loadFound();

    // Seed the generators before anything reads noise or biome centres.
    this.seed = options.seed ?? dailySeedLabel();
    const seed = hashSeed(this.seed);
    setNoiseSeed(seed);
    layoutBiomes(seed);
    this.clouds = new Clouds(seed);

    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.15;
    this.scene.background = new THREE.Color(0x0d1c28);

    this.camera = new THREE.PerspectiveCamera(42, 1, 0.1, 1400);

    this.hemi = new THREE.HemisphereLight(0xdcefff, 0x6b7a63, 1.35);
    this.scene.add(this.hemi);
    this.sun = new THREE.DirectionalLight(0xfff3dc, 2.1);
    this.sun.castShadow = true;
    this.sun.shadow.mapSize.set(2048, 2048);
    this.sun.shadow.camera.near = 1;
    this.sun.shadow.camera.far = 320;
    this.sun.shadow.bias = -0.0005;
    this.sun.shadow.normalBias = 0.03;
    this.setShadowBounds(PLANET_RADIUS + 14);
    this.scene.add(this.sun, this.sun.target);

    this.ground = buildGround();
    this.water = buildWater();
    this.stars = buildStars(seed);
    this.scene.add(this.ground, this.water, this.stars);

    const props = buildWorldProps(seed);
    if (props.solid) this.scene.add(props.solid);
    if (props.glow) {
      this.scene.add(props.glow);
      this.glowMat = props.glow.material as THREE.MeshBasicMaterial;
    }
    if (props.night) {
      this.scene.add(props.night);
      this.nightMesh = props.night;
    }
    this.colliders = new ColliderGrid(props.colliders);

    // Windmill rotor.
    const hubPivot = new THREE.Group();
    props.windmillHub.decompose(hubPivot.position, hubPivot.quaternion, _v1);
    this.blades = buildWindmillBlades();
    hubPivot.add(this.blades);
    this.scene.add(hubPivot);

    // Campfire flame and its light.
    const firePivot = new THREE.Group();
    props.fire.decompose(firePivot.position, firePivot.quaternion, _v1);
    this.flame = buildFlame();
    this.flame.position.y = 0.15;
    this.flameLight = new THREE.PointLight(0xff9040, 28, 9, 2);
    this.flameLight.position.y = 1.0;
    firePivot.add(this.flame, this.flameLight);
    this.scene.add(firePivot);

    // Lighthouse lamp, dark until the blog wonder is found.
    this.lampLight = new THREE.PointLight(0xffe6a3, 0, 26, 2);
    this.lampLight.position.setFromMatrixPosition(props.lamp);
    this.scene.add(this.lampLight);

    // Wonder markers.
    for (const w of WONDERS) {
      const frame = props.wonderFrames.get(w.id)!;
      const found = this.found.includes(w.id);
      const gemMat = new THREE.MeshStandardMaterial({
        color: found ? 0xffd166 : 0x9fe8ff,
        emissive: found ? 0xffb703 : 0x5fd3ff,
        emissiveIntensity: 0.9,
        roughness: 0.3,
        flatShading: true
      });
      const mesh = new THREE.Mesh(new THREE.OctahedronGeometry(0.38, 0), gemMat);
      const ring = new THREE.Mesh(
        new THREE.RingGeometry(1.05, 1.28, 28),
        new THREE.MeshBasicMaterial({ color: found ? 0xffd166 : 0xbdf0ff, transparent: true, opacity: 0.45, side: THREE.DoubleSide })
      );
      ring.matrixAutoUpdate = false;
      ring.matrix.copy(frame).multiply(_m.makeTranslation(0, 0.08, 0)).multiply(new THREE.Matrix4().makeRotationX(-Math.PI / 2));
      const up = new THREE.Vector3().setFromMatrixColumn(frame, 1).normalize();
      const base = new THREE.Vector3().setFromMatrixPosition(frame);
      const frameQ = new THREE.Quaternion().setFromRotationMatrix(frame);
      this.scene.add(mesh, ring);
      this.gems.push({ wonder: w, dir: wonderDir(w), mesh, ring, frameQ, base, up });
    }

    // Click-to-walk marker.
    this.targetRing = new THREE.Mesh(
      new THREE.RingGeometry(0.45, 0.6, 24),
      new THREE.MeshBasicMaterial({ color: 0xfff2c4, transparent: true, opacity: 0.8, side: THREE.DoubleSide })
    );
    this.targetRing.visible = false;
    this.scene.add(this.targetRing);

    // The boat waits offstage until it is launched from a shore.
    this.boat = buildBoat();
    this.boat.visible = false;
    this.scene.add(this.boat, this.wake.group);

    // Critters.
    const spawn = (model: THREE.Group, biome: 'farm' | 'arctic' | 'forest' | 'shore', e: number, n: number, roam: number, speed: number, seed: number) => {
      const home = offsetDir(biomeById(biome).center.clone(), e, n);
      const c = new Critter(model, home, roam, speed, seed, this.colliders);
      c.group.traverse((o) => {
        if (o instanceof THREE.Mesh) o.castShadow = true;
      });
      this.critters.push(c);
      this.scene.add(c.group);
    };
    spawn(buildSheep(), 'farm', -2.5, -3.5, 3.5, 1.1, 1);
    spawn(buildSheep(), 'farm', -1.0, -4.5, 3.5, 1.0, 2);
    spawn(buildSheep(), 'farm', 1.5, -5.5, 3.0, 1.2, 3);
    spawn(buildSheep(), 'farm', 6, 3, 3.0, 0.9, 4);
    spawn(buildBear(), 'arctic', -4, 5, 5, 1.3, 5);
    spawn(buildBear(), 'arctic', 5, -5, 5, 1.1, 6);
    spawn(buildBear(), 'arctic', -6, -4, 4, 1.0, 7);
    spawn(buildFox(), 'forest', 4, 3, 5, 2.2, 8);
    spawn(buildFox(), 'shore', -4, 4, 6, 2.4, 9);

    this.scene.add(this.clouds.group);
    this.aurora = new Aurora(biomeById('arctic').center, PLANET_RADIUS);
    this.scene.add(this.aurora.group);
    if (this.found.includes('education')) this.aurora.setActive(true);
    if (this.found.includes('blog')) this.lampOn = true;

    // Player starts a few steps from the campfire, facing it.
    const fire = wonderDir(WONDERS[0]);
    this.dir.copy(fire);
    offsetDir(this.dir, 1.4, -3.6);
    // Some seeds put that spot in the sea; step back toward the fire until it is dry land.
    for (let i = 0; i < 12 && isOcean(this.dir); i++) offsetDir(this.dir, -0.14, 0.36);
    this.colliders.resolve(this.dir, PLAYER_RADIUS);
    this.heading.copy(fire).addScaledVector(this.dir, -fire.dot(this.dir)).normalize();
    this.camForward.copy(this.heading);
    this.character.group.traverse((o) => {
      if (o instanceof THREE.Mesh) o.castShadow = true;
    });
    this.scene.add(this.character.group);
    this.placeCharacter();

    // Globe view first: the camera hangs above the player's hemisphere.
    this.globePitch = Math.asin(THREE.MathUtils.clamp(this.dir.y, -0.9, 0.9)) + 0.25;
    this.globeYaw = Math.atan2(this.dir.x, this.dir.z);
    this.resize();
    this.camPos.copy(this.globeDirection()).multiplyScalar(this.globeDistance());
    this.camLook.set(0, 0, 0);

    this.input = new Input(canvas);
    this.input.onFirstGesture(() => this.exitIntro());
    this.applyLighting();

    this.resize();
    this.last = performance.now();
    this.raf = requestAnimationFrame(this.frame);
  }

  // ---------------------------------------------------------------- public

  setOverlayOpen(open: boolean): void {
    this.overlayOpen = open;
    this.input.enabled = !open;
    if (open) {
      this.walkTarget = null;
      this.targetRing.visible = false;
      this.input.touchMove.x = 0;
      this.input.touchMove.y = 0;
    }
  }

  toggleGlobe(): void {
    this.setGlobe(!this.globe);
  }

  toggleMute(): boolean {
    this.audio.start();
    return this.audio.toggle();
  }

  /** Pause or resume the day-night clock. Returns true when paused. */
  toggleClock(): boolean {
    this.dayNight.paused = !this.dayNight.paused;
    this.emitClock();
    return this.dayNight.paused;
  }

  /** Fraction of the day in [0, 1): midnight is 0, noon is 0.5. */
  get timeOfDay(): number {
    return this.dayNight.time;
  }

  /** Jump the clock, for tuning from the console. */
  setTimeOfDay(t: number): void {
    this.dayNight.set(t);
    this.applyLighting();
  }

  startAudio(): void {
    this.audio.start();
  }

  dispose(): void {
    this.disposed = true;
    cancelAnimationFrame(this.raf);
    this.input.dispose();
    this.audio.dispose();
    this.scene.traverse((o) => {
      if (o instanceof THREE.Mesh) {
        o.geometry.dispose();
        const mats = Array.isArray(o.material) ? o.material : [o.material];
        for (const m of mats) m.dispose();
      }
    });
    this.renderer.dispose();
  }

  // ---------------------------------------------------------------- loop

  private frame = (now: number) => {
    if (this.disposed) return;
    this.raf = requestAnimationFrame(this.frame);
    const dt = Math.min(0.05, (now - this.last) / 1000);
    this.last = now;
    this.time += dt;
    this.resize();
    this.handleInput(dt);
    this.updatePlayer(dt);
    this.updateCamera(dt);
    this.updateWorld(dt);
    this.updatePrompt();
    this.renderer.render(this.scene, this.camera);
  };

  private resize(): void {
    const w = this.canvas.clientWidth;
    const h = this.canvas.clientHeight;
    if (w === 0 || h === 0) return;
    const size = this.renderer.getSize(_size);
    if (size.x !== w || size.y !== h) {
      this.renderer.setSize(w, h, false);
      this.camera.aspect = w / h;
      this.camera.updateProjectionMatrix();
    }
  }

  private exitIntro(): void {
    if (!this.intro) return;
    this.intro = false;
    this.callbacks.onIntroEnd();
    this.setGlobe(false);
  }

  private setGlobe(on: boolean): void {
    if (this.globe === on) return;
    this.globe = on;
    this.transition = 1.6;
    if (on) {
      // Start the orbit from wherever the camera is so the pull-back is smooth.
      const d = _v1.copy(this.camPos).normalize();
      this.globePitch = Math.asin(THREE.MathUtils.clamp(d.y, -0.95, 0.95));
      this.globeYaw = Math.atan2(d.x, d.z);
      this.walkTarget = null;
      this.targetRing.visible = false;
    }
    this.setShadowBounds(on ? PLANET_RADIUS + 14 : 30);
    this.callbacks.onGlobe(on);
  }

  private setShadowBounds(extent: number): void {
    const cam = this.sun.shadow.camera;
    cam.left = -extent;
    cam.right = extent;
    cam.top = extent;
    cam.bottom = -extent;
    cam.updateProjectionMatrix();
  }

  /** Camera distance at which the whole planet fits the viewport, in either orientation. */
  private globeDistance(): number {
    const vfov = THREE.MathUtils.degToRad(this.camera.fov);
    const hfov = 2 * Math.atan(Math.tan(vfov / 2) * this.camera.aspect);
    const half = Math.min(vfov, hfov) / 2;
    return ((PLANET_RADIUS + GLOBE_MARGIN) / Math.sin(half)) * this.globeZoom;
  }

  private globeDirection(): THREE.Vector3 {
    const cp = Math.cos(this.globePitch);
    return _v2.set(cp * Math.sin(this.globeYaw), Math.sin(this.globePitch), cp * Math.cos(this.globeYaw));
  }

  // ---------------------------------------------------------------- input

  private handleInput(dt: number): void {
    const input = this.input;
    if (input.consumeEscape()) this.callbacks.onEscape();
    if (input.consumeHelp()) this.callbacks.onHelp();
    if (input.consumeClock() && !this.overlayOpen) this.toggleClock();
    if (input.consumeGlobe() && !this.overlayOpen) {
      this.exitIntro();
      this.toggleGlobe();
    }

    const orbit = input.consumeOrbit();
    this.dragThisFrame = orbit.dx !== 0 || orbit.dy !== 0;
    if (this.globe) {
      this.globeYaw -= orbit.dx * 0.006;
      this.globePitch = THREE.MathUtils.clamp(this.globePitch + orbit.dy * 0.005, -1.2, 1.2);
    } else {
      if (orbit.dx !== 0) {
        _q.setFromAxisAngle(this.dir, -orbit.dx * 0.006);
        this.camForward.applyQuaternion(_q);
      }
      this.camPitch = THREE.MathUtils.clamp(this.camPitch + orbit.dy * 0.005, 0.22, 1.35);
    }

    const zoom = input.consumeZoom();
    if (zoom !== 0) {
      if (this.globe) this.globeZoom = THREE.MathUtils.clamp(this.globeZoom * Math.exp(zoom * 0.1), 0.65, 1.7);
      else this.camDist = THREE.MathUtils.clamp(this.camDist * Math.exp(zoom * 0.12), CAM_MIN, CAM_MAX);
    }

    const click = input.consumeClick();
    if (click && !this.overlayOpen) {
      this.raycaster.setFromCamera(new THREE.Vector2(click.x, click.y), this.camera);
      const hits = this.raycaster.intersectObjects([this.ground, this.water], false);
      if (hits.length > 0) {
        this.audio.start();
        this.walkTarget = hits[0].point.clone().normalize();
        this.targetRing.visible = true;
        const r = walkRadius(this.walkTarget) + 0.1;
        this.targetRing.position.copy(this.walkTarget).multiplyScalar(r);
        tangentBasis(this.walkTarget, 0, _v1, _v3);
        _m.makeBasis(_v1, this.walkTarget, _v3);
        this.targetRing.quaternion.setFromRotationMatrix(_m);
        this.targetRing.rotateX(-Math.PI / 2);
        if (this.globe) this.setGlobe(false);
      }
    }

    if (input.consumeHop() && !this.overlayOpen) {
      this.audio.start();
      if (this.globe) this.setGlobe(false);
      if (!this.airborne && !this.boating) {
        this.hopV = HOP_VELOCITY;
        this.airborne = true;
        this.audio.hop();
      }
    }

    if (input.consumeInteract() && !this.overlayOpen && !this.globe) {
      const gem = this.nearestGem();
      if (gem) this.findWonder(gem);
      else if (!this.boating && this.launchTarget()) this.launchBoat();
    }

    if (this.globe && !this.intro && (input.moveX !== 0 || input.moveY !== 0)) this.setGlobe(false);
    void dt;
  }

  // ---------------------------------------------------------------- player

  private updatePlayer(dt: number): void {
    if (this.boating) {
      this.updateBoat(dt);
      this.updateBiome(dt);
      return;
    }
    const up = this.dir;
    let moving = false;
    let speed = WALK_SPEED;
    const desired = _v1;

    const mx = this.overlayOpen ? 0 : this.input.moveX;
    const my = this.overlayOpen ? 0 : this.input.moveY;
    if (mx !== 0 || my !== 0) {
      this.walkTarget = null;
      this.targetRing.visible = false;
      _v2.crossVectors(this.camForward, up).normalize(); // camera right
      desired.copy(this.camForward).multiplyScalar(my).addScaledVector(_v2, mx);
      desired.addScaledVector(up, -desired.dot(up));
      if (desired.lengthSq() > 1e-6) {
        desired.normalize();
        moving = true;
        speed = this.input.run ? RUN_SPEED : WALK_SPEED;
      }
    } else if (this.walkTarget) {
      desired.copy(this.walkTarget).addScaledVector(up, -this.walkTarget.dot(up));
      const remaining = up.angleTo(this.walkTarget) * PLANET_RADIUS;
      if (remaining < 0.3 || desired.lengthSq() < 1e-8) {
        this.walkTarget = null;
        this.targetRing.visible = false;
      } else {
        desired.normalize();
        moving = true;
        speed = remaining > 7 ? RUN_SPEED : WALK_SPEED;
      }
    }

    if (moving && !this.globe) {
      this.heading.lerp(desired, Math.min(1, dt * TURN_RATE));
      this.heading.addScaledVector(up, -this.heading.dot(up)).normalize();
      const ang = (speed * dt) / PLANET_RADIUS;
      const cos = Math.cos(ang);
      const sin = Math.sin(ang);
      const before = this.walkTarget ? up.angleTo(this.walkTarget) : 0;
      _v2.copy(up).multiplyScalar(cos).addScaledVector(this.heading, sin).normalize();
      if (isOcean(_v2)) {
        // The shore is the edge of the walkable world; the boat crosses the water.
        moving = false;
        this.stuckTime += dt;
      } else {
        this.colliders.resolve(_v2, PLAYER_RADIUS);
        up.copy(_v2);
        this.heading.addScaledVector(up, -this.heading.dot(up)).normalize();
        this.camForward.addScaledVector(up, -this.camForward.dot(up)).normalize();
        if (!this.airborne) this.audio.step(speed > WALK_SPEED + 0.1);
        if (this.walkTarget) {
          // Give up on a click target that an obstacle keeps us from reaching.
          const progress = before - up.angleTo(this.walkTarget);
          this.stuckTime = progress > ang * 0.2 ? 0 : this.stuckTime + dt;
        }
      }
      if (this.walkTarget && this.stuckTime > STUCK_TIMEOUT) {
        this.walkTarget = null;
        this.targetRing.visible = false;
        this.stuckTime = 0;
      }
    } else {
      moving = false;
      this.stuckTime = 0;
    }

    const targetAnim = moving ? speed / RUN_SPEED : 0;
    this.animSpeed += (targetAnim - this.animSpeed) * Math.min(1, dt * 10);

    if (this.airborne) {
      this.hopH += this.hopV * dt;
      this.hopV -= GRAVITY * dt;
      if (this.hopH <= 0) {
        this.hopH = 0;
        this.hopV = 0;
        this.airborne = false;
      }
    }

    this.character.update(dt, this.animSpeed, this.airborne, this.time);
    this.placeCharacter();
    this.updateBiome(dt);
  }

  /** Biome caption and shore scan, on a quarter-second cadence. */
  private updateBiome(dt: number): void {
    this.biomeTimer -= dt;
    if (this.biomeTimer > 0) return;
    this.biomeTimer = 0.25;
    const b = biomeAt(this.dir);
    if (b.id !== this.biomeId) {
      this.biomeId = b.id;
      this.callbacks.onBiome({ name: b.name, kind: b.kind, index: b.index, tagline: b.tagline });
    }
    this.launchDir = this.boating || this.globe ? null : findLaunchPoint(this.dir);
  }

  private placeCharacter(): void {
    const g = this.character.group;
    if (this.boating) {
      g.position.copy(this.boat.position).add(_v3.copy(BOAT_SEAT).applyQuaternion(this.boat.quaternion));
      g.quaternion.copy(this.boat.quaternion);
      return;
    }
    const r = walkRadius(this.dir) + this.hopH;
    g.position.copy(this.dir).multiplyScalar(r);
    _v2.crossVectors(this.dir, this.heading).normalize();
    _m.makeBasis(_v2, this.dir, this.heading);
    g.quaternion.setFromRotationMatrix(_m);
  }

  // ---------------------------------------------------------------- boat

  /** Water the player could launch into right now: the docked boat if it is close, else open shore. */
  private launchTarget(): THREE.Vector3 | null {
    if (this.boatPlaced && this.boatDir.angleTo(this.dir) * PLANET_RADIUS < LAUNCH_RANGE + 1.5) return this.boatDir;
    return this.launchDir;
  }

  private launchBoat(): void {
    const target = this.launchTarget();
    if (!target) return;
    this.audio.start();
    this.exitIntro();
    if (this.globe) this.setGlobe(false);
    // Face away from the shore we are leaving.
    this.boatHeading.copy(target).sub(this.dir);
    this.boatDir.copy(target);
    this.boatHeading.addScaledVector(this.boatDir, -this.boatHeading.dot(this.boatDir));
    if (this.boatHeading.lengthSq() < 1e-6) this.boatHeading.copy(this.heading);
    this.boatHeading.addScaledVector(this.boatDir, -this.boatHeading.dot(this.boatDir)).normalize();
    this.dir.copy(this.boatDir);
    this.heading.copy(this.boatHeading);
    this.camForward.addScaledVector(this.dir, -this.camForward.dot(this.dir)).normalize();
    this.boatSpeed = 0;
    this.boatPlaced = true;
    this.boating = true;
    this.boat.visible = true;
    this.hopH = 0;
    this.hopV = 0;
    this.airborne = false;
    this.walkTarget = null;
    this.targetRing.visible = false;
    this.launchDir = null;
    this.promptId = null;
    this.audio.hop();
    this.placeBoat();
    this.placeCharacter();
  }

  /** Step ashore at `land`, leaving the boat afloat where it is. */
  private dock(land: THREE.Vector3): void {
    this.dir.copy(land);
    this.colliders.resolve(this.dir, PLAYER_RADIUS);
    this.heading.copy(this.boatHeading).addScaledVector(this.dir, -this.boatHeading.dot(this.dir)).normalize();
    this.camForward.addScaledVector(this.dir, -this.camForward.dot(this.dir)).normalize();
    this.boating = false;
    this.boatSpeed = 0;
    this.stuckTime = 0;
    this.promptId = null;
    this.biomeTimer = 0;
    this.audio.hop();
  }

  private updateBoat(dt: number): void {
    const up = this.dir;
    const desired = _v1;
    let steering = false;
    let run = false;

    const mx = this.overlayOpen ? 0 : this.input.moveX;
    const my = this.overlayOpen ? 0 : this.input.moveY;
    if (mx !== 0 || my !== 0) {
      this.walkTarget = null;
      this.targetRing.visible = false;
      _v2.crossVectors(this.camForward, up).normalize();
      desired.copy(this.camForward).multiplyScalar(my).addScaledVector(_v2, mx);
      desired.addScaledVector(up, -desired.dot(up));
      if (desired.lengthSq() > 1e-6) {
        desired.normalize();
        steering = true;
        run = this.input.run;
      }
    } else if (this.walkTarget) {
      desired.copy(this.walkTarget).addScaledVector(up, -this.walkTarget.dot(up));
      const remaining = up.angleTo(this.walkTarget) * PLANET_RADIUS;
      if (remaining < 0.8 || desired.lengthSq() < 1e-8) {
        this.walkTarget = null;
        this.targetRing.visible = false;
      } else {
        desired.normalize();
        steering = true;
        run = remaining > 12;
      }
    }

    const targetSpeed = steering && !this.globe ? BOAT_SPEED * (run ? BOAT_RUN : 1) : 0;
    this.boatSpeed += (targetSpeed - this.boatSpeed) * Math.min(1, dt * (steering ? BOAT_ACCEL : BOAT_DRAG));
    if (steering && !this.globe) {
      // Only a moving hull answers the rudder.
      const bite = 0.35 + 0.65 * Math.min(1, this.boatSpeed / BOAT_SPEED);
      this.boatHeading.lerp(desired, Math.min(1, dt * BOAT_TURN * bite));
      this.boatHeading.addScaledVector(up, -this.boatHeading.dot(up)).normalize();
    }

    if (this.boatSpeed > 0.05) {
      const ang = (this.boatSpeed * dt) / PLANET_RADIUS;
      _v2.copy(up).multiplyScalar(Math.cos(ang)).addScaledVector(this.boatHeading, Math.sin(ang)).normalize();
      if (!isOcean(_v2)) {
        this.dock(_v2);
        this.character.update(dt, 0, false, this.time);
        this.placeCharacter();
        return;
      }
      up.copy(_v2);
      this.boatDir.copy(up);
      this.boatHeading.addScaledVector(up, -this.boatHeading.dot(up)).normalize();
      this.camForward.addScaledVector(up, -this.camForward.dot(up)).normalize();
    }

    this.animSpeed += (0 - this.animSpeed) * Math.min(1, dt * 10);
    this.character.update(dt, 0, false, this.time, true);
    this.placeBoat();
    this.placeCharacter();
  }

  /** Sit the boat on the water at `boatDir`, bobbing gently, whether crewed or docked. */
  private placeBoat(): void {
    const t = this.time;
    const g = this.boat;
    const way = Math.min(1, this.boatSpeed / BOAT_SPEED);
    g.position.copy(this.boatDir).multiplyScalar(SEA_LEVEL - 0.02 + Math.sin(t * 1.7) * 0.04);
    _v2.crossVectors(this.boatDir, this.boatHeading).normalize();
    _m.makeBasis(_v2, this.boatDir, this.boatHeading);
    g.quaternion.setFromRotationMatrix(_m);
    _e.set(Math.sin(t * 1.3) * 0.025 - way * 0.04, 0, Math.sin(t * 1.9) * 0.05);
    g.quaternion.multiply(_q.setFromEuler(_e));
  }

  // ---------------------------------------------------------------- camera

  private updateCamera(dt: number): void {
    const desiredPos = _v1;
    const desiredLook = _v2;
    const desiredUp = _v3;

    if (this.globe) {
      if (!this.dragThisFrame) this.globeYaw += dt * 0.05;
      desiredPos.copy(this.globeDirection()).multiplyScalar(this.globeDistance());
      desiredLook.set(0, 0, 0);
      desiredUp.set(0, 1, 0);
      if (Math.abs(this.globePitch) > 1.0) desiredUp.set(Math.sin(this.globeYaw), 0, Math.cos(this.globeYaw)).multiplyScalar(-Math.sign(this.globePitch));
    } else {
      const up = this.dir;
      const playerPos = this.character.group.position;
      desiredPos
        .copy(playerPos)
        .addScaledVector(up, Math.sin(this.camPitch) * this.camDist + 0.9)
        .addScaledVector(this.camForward, -Math.cos(this.camPitch) * this.camDist);
      desiredLook.copy(playerPos).addScaledVector(up, 1.3);
      desiredUp.copy(up);
    }

    this.transition = Math.max(0, this.transition - dt);
    const rate = this.transition > 0 ? 3.2 : 11;
    const k = 1 - Math.exp(-dt * rate);
    this.camPos.lerp(desiredPos, k);
    this.camLook.lerp(desiredLook, k);
    this.camUp.lerp(desiredUp, k).normalize();
    this.camera.position.copy(this.camPos);
    this.camera.up.copy(this.camUp);
    this.camera.lookAt(this.camLook);
    this.camera.updateMatrixWorld();

    // The sun rides over the camera's left shoulder so the visible face of
    // the planet is always lit, and shadows fall consistently.
    const right = _v1.setFromMatrixColumn(this.camera.matrixWorld, 0);
    const camUp = _v2.setFromMatrixColumn(this.camera.matrixWorld, 1);
    const back = _v3.setFromMatrixColumn(this.camera.matrixWorld, 2);
    const lift = 0.3 + this.dayNight.light.sunHeight * 0.65; // low sun at dawn and dusk, long shadows
    const sunDir = right.multiplyScalar(-1.15).addScaledVector(camUp, lift).addScaledVector(back, 0.6).normalize();
    const focus = this.globe ? this.sun.target.position.set(0, 0, 0) : this.sun.target.position.copy(this.character.group.position);
    this.sun.position.copy(focus).addScaledVector(sunDir, 140);
    this.sun.target.updateMatrixWorld();
  }

  // ---------------------------------------------------------------- world

  private updateWorld(dt: number): void {
    const t = this.time;
    this.dayNight.update(dt);
    this.applyLighting();
    const night = this.dayNight.light.night;

    this.blades.rotation.z += dt * 1.1;
    this.flame.scale.set(1 + Math.sin(t * 13) * 0.1, 1 + Math.sin(t * 17.3) * 0.18 + Math.sin(t * 5) * 0.06, 1 + Math.cos(t * 11) * 0.1);
    this.flame.rotation.y = t * 0.8;
    this.flameLight.intensity = (26 + Math.sin(t * 11) * 5 + Math.sin(t * 23) * 3) * (1 + night * 0.9);
    const lampTarget = this.lampOn ? 70 * (0.55 + night * 0.6) : 0;
    this.lampLight.intensity += (lampTarget - this.lampLight.intensity) * Math.min(1, dt * 2);

    for (const c of this.critters) c.update(dt, t);
    this.clouds.update(dt);
    this.aurora.night = night;
    this.aurora.update(dt, t);
    if (this.boatPlaced) {
      if (!this.boating) this.placeBoat();
      this.wake.update(dt, this.boatDir, this.boatHeading, this.boating ? this.boatSpeed : 0);
    }

    for (let i = 0; i < this.gems.length; i++) {
      const g = this.gems[i];
      const found = this.found.includes(g.wonder.id);
      const bob = Math.sin(t * 1.6 + i) * 0.18;
      g.mesh.position.copy(g.base).addScaledVector(g.up, 2.5 + bob);
      _q.setFromAxisAngle(g.up, t * 0.9 + i);
      g.mesh.quaternion.copy(_q).multiply(g.frameQ);
      const pulse = found ? 0.6 : 0.8 + Math.sin(t * 3 + i) * 0.35;
      (g.mesh.material as THREE.MeshStandardMaterial).emissiveIntensity = pulse * (1 + night * 1.2);
      (g.ring.material as THREE.MeshBasicMaterial).opacity = found ? 0.3 : 0.35 + Math.sin(t * 3 + i) * 0.15;
    }

    if (this.targetRing.visible) {
      const s = 1 + Math.sin(t * 6) * 0.12;
      this.targetRing.scale.set(s, s, s);
    }
  }

  /** Copy the sampled time-of-day lighting onto the sky, lights, stars, and water. */
  private applyLighting(): void {
    const L = this.dayNight.light;
    (this.scene.background as THREE.Color).copy(L.sky);
    this.hemi.color.copy(L.hemiSky);
    this.hemi.groundColor.copy(L.hemiGround);
    this.hemi.intensity = L.hemiIntensity;
    this.sun.color.copy(L.sun);
    this.sun.intensity = L.sunIntensity;
    (this.stars.material as THREE.PointsMaterial).opacity = L.starOpacity;
    (this.water.material as THREE.MeshStandardMaterial).color.copy(L.water);
    // Lava, embers, and crystals burn brighter against a dark sky.
    if (this.glowMat) this.glowMat.color.setScalar(0.9 + L.night * 0.5);
    if (this.nightMesh) {
      const on = L.night > 0.02;
      this.nightMesh.visible = on;
      if (on) (this.nightMesh.material as THREE.MeshBasicMaterial).opacity = Math.min(1, L.night * 1.3);
    }
    const night = L.night > 0.5;
    if (night !== this.clockNight) {
      this.clockNight = night;
      this.emitClock();
    }
  }

  private emitClock(): void {
    this.callbacks.onClock({ paused: this.dayNight.paused, night: this.clockNight });
  }

  // ---------------------------------------------------------------- wonders

  private nearestGem(): Gem | null {
    let best: Gem | null = null;
    let bestDist = INTERACT_RANGE;
    for (const g of this.gems) {
      const d = g.dir.angleTo(this.dir) * PLANET_RADIUS;
      if (d < bestDist) {
        bestDist = d;
        best = g;
      }
    }
    return best;
  }

  private updatePrompt(): void {
    const idle = this.globe || this.overlayOpen;
    const gem = idle ? null : this.nearestGem();
    let prompt: Prompt | null = null;
    if (gem) prompt = { id: gem.wonder.id, action: gem.wonder.action, found: this.found.includes(gem.wonder.id) };
    else if (!idle && !this.boating && this.launchTarget()) {
      const reuse = this.launchTarget() === this.boatDir;
      prompt = { id: reuse ? 'boat-board' : 'boat-launch', action: reuse ? 'Board the boat' : 'Launch a boat', found: false, kicker: 'The shallows' };
    }
    const id = prompt?.id ?? null;
    if (id === this.promptId) return;
    this.promptId = id;
    this.callbacks.onPrompt(prompt);
  }

  private findWonder(gem: Gem): void {
    const w = gem.wonder;
    this.audio.start();
    if (!this.found.includes(w.id)) {
      this.found = [...this.found, w.id];
      saveFound(this.found);
      this.callbacks.onFound(this.found);
      this.audio.chime();
      const mat = gem.mesh.material as THREE.MeshStandardMaterial;
      mat.color.setHex(0xffd166);
      mat.emissive.setHex(0xffb703);
      (gem.ring.material as THREE.MeshBasicMaterial).color.setHex(0xffd166);
      this.promptId = null; // re-emit the prompt with found = true
    }
    if (w.id === 'education') this.aurora.setActive(true);
    if (w.id === 'blog') this.lampOn = true;
    // Face the wonder.
    this.heading.copy(gem.dir).addScaledVector(this.dir, -gem.dir.dot(this.dir));
    if (this.heading.lengthSq() > 1e-6) this.heading.normalize();
    this.callbacks.onOpenWonder(w);
  }
}
