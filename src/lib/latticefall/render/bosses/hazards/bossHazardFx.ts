// The boss hazards the core already simulates but the renderer didn't draw (WP-100), under one
// roof so the engine wires a single object:
//
//   - B08's creeping lava (`LavaFx`)
//   - B10's toll shockwave and ice (`TollFx`)
//   - B09's lit lamp flames (`LampFx`)
//   - B04's weak point, lit only while it is open (the model reads `setWeakPointOpen`)
//   - B02's intro camera move (`IntroCamera`)
//
// (B02's bramble patches and B12's loom lines have their own effects, `BriarFx` and `LoomFx`.)
//
// Read-only: it takes the decoded events and the drawn actors and draws. The game clock it follows is
// `world.tick_count() + alpha`, so a paused game holds every effect where it is. The events carry no
// positions for these hazards, so the arenas' fixed ones live in `arenas.ts`.

import type { PerspectiveCamera, Scene } from 'three';
import {
  BOSS_DEFEATED,
  BOSS_FIGHT_STARTED,
  BOSS_RESET,
  BOSS_SEALED,
  type GameEvent
} from '../../../engine/events';
import { FLAG_DEAD, type DrawActor } from '../../../engine/snapshot';
import { B02_KIND } from '../b02/briarStag';
import { IntroCamera } from '../b02/introCamera';
import { setWeakPointOpen } from '../b04/mossgutBrood';
import { B02, B04, B08, B10, MECH, TICK_HZ, type Vec3 } from './arenas';
import { LampFx } from './lampFx';
import { LavaFx } from './lavaFx';
import { parseHazardPreview, staged12Loom, stagedEvents, type HazardPreview } from './preview';
import { TollFx } from './tollFx';

/** Registry numbers of the bodies the hazards read (`data/registry.ron`). */
const B08_BODY = 37011;
const B10_BODY = 39011;

export interface HazardOptions {
  /** The scenario being played: the intro camera is B02's, so it belongs to the Verge. */
  scenario: string;
  /** True when the camera shake refuses motion (reduced motion). The intro move is a cut then. */
  reducedMotion?: () => boolean;
  /** Where the dev preview sends B12's loom events (the engine's `LoomFx`). */
  loomSink?: (events: GameEvent[]) => void;
}

export class BossHazardFx {
  private readonly lava: LavaFx;
  private readonly toll: TollFx;
  private readonly lamps: LampFx;
  private readonly intro: IntroCamera;
  private actors: DrawActor[] = [];
  private now = 0;
  private weakFromEvent = false;
  private weakOpen = false;
  private readonly preview: HazardPreview | null;
  /** The dev preview's held clock in ticks, or null for a real clock. */
  private held: number | null = null;

  constructor(
    scene: Scene,
    private readonly opts: HazardOptions
  ) {
    this.lava = new LavaFx(scene);
    this.toll = new TollFx(scene);
    this.lamps = new LampFx(scene);
    this.intro = new IntroCamera(opts.reducedMotion ?? (() => false));
    this.preview = typeof window === 'undefined' ? null : parseHazardPreview(window.location.search);
    if (this.preview) this.stagePreview(this.preview);
  }

  /** Reacts to a tick's events. `now` is the game tick they came from. */
  handleEvents(events: GameEvent[], now: number): void {
    for (const e of events) {
      if (e.kind === 'mechanic') this.mechanic(e.what, e.index, now);
      else if (e.kind === 'boss') this.bossEvent(e.what, e.boss, now);
    }
  }

  private mechanic(what: number, index: number, now: number): void {
    switch (what) {
      case MECH.WEAK_POINT_EXPOSED:
        this.weakFromEvent = true;
        break;
      case MECH.LAVA_STARTED:
        this.lava.start(now);
        break;
      case MECH.LAVA_BURNED:
        this.lava.burned(now);
        break;
      case MECH.LAMP_LIT:
        this.lamps.lit(index, now);
        break;
      case MECH.LAMP_STUN:
        this.lamps.stunned(index, now);
        break;
      case MECH.LAMPS_DOUSED:
        this.lamps.douse();
        break;
      case MECH.BELL_BEAT:
        this.toll.beat(now);
        break;
      case MECH.BELL_TOLL_WARNED:
        this.toll.warned(now, this.bodyPos(B10_BODY));
        break;
      case MECH.BELL_TOLLED:
        this.toll.tolled(now, this.bodyPos(B10_BODY));
        break;
      case MECH.ICE_CRACKED:
        this.toll.ice(index, 1, now);
        break;
      case MECH.ICE_BROKE:
        this.toll.ice(index, 2, now);
        break;
      case MECH.ICE_WATER_BURNED:
        this.toll.burned(index, now);
        break;
    }
  }

  private bossEvent(what: number, boss: number, now: number): void {
    if (what === BOSS_SEALED && this.opts.scenario === 'verge' && boss === B02.boss) {
      const stag = this.actors.find((a) => a.kind === B02_KIND);
      this.intro.begin(now, stag ? [stag.x, stag.y, stag.z] : B02.home, stag ? stag.yaw : -Math.PI / 2);
    } else if (what === BOSS_FIGHT_STARTED) {
      this.intro.end(now);
    } else if (what === BOSS_DEFEATED || what === BOSS_RESET) {
      // A fight ended: the lava dries, the ice and the lamps go, the weak point shuts.
      if (what === BOSS_RESET) this.intro.end(now);
      this.lava.stop();
      this.toll.clear();
      this.lamps.douse();
      this.weakFromEvent = false;
    }
  }

  private bodyPos(kind: number): Vec3 | null {
    const a = this.actors.find((x) => x.kind === kind);
    return a ? [a.x, a.y, a.z] : null;
  }

  /** Once per rendered frame, before the draw: takes the drawn actors and the game clock in ticks. */
  frame(now: number, actors: DrawActor[]): void {
    this.actors = actors;
    this.now = this.held ?? now;
    const t = this.now;

    // B08: the lava dries when the Colossus is down or has been rebuilt (a reset gives back its HP).
    if (this.lava.active && !this.preview) {
      const body = actors.find((a) => a.kind === B08_BODY);
      if (body && ((body.flags & FLAG_DEAD) !== 0 || body.hpFrac > B08.phase2Frac + 0.05)) this.lava.stop();
    }

    // B04: the weak point opens when every egg sac is down. The sacs are actors, so when all four are
    // there the answer is theirs (a reset revives them and shuts it); if the snapshot drops the dead
    // ones, the core's own "exposed" event stands in until the fight resets.
    const sacs = actors.filter((a) => B04.sacs.includes(a.kind));
    const open = sacs.length === B04.sacs.length ? sacs.every((a) => (a.flags & FLAG_DEAD) !== 0) : this.weakFromEvent;
    const mother = actors.find((a) => a.kind === B04.body);
    const motherDown = !!mother && (mother.flags & FLAG_DEAD) !== 0;
    this.weakOpen = this.preview?.name === 'b04' || (open && !motherDown);
    setWeakPointOpen(this.weakOpen);

    this.lava.update(t);
    this.toll.update(t);
    this.lamps.update(t);
  }

  /** Whether the weak point is drawn open (for tests and the dev readout). */
  get weakPointOpen(): boolean {
    return this.weakOpen;
  }

  /** From a frame hook, after the rig has placed the camera: B02's intro move overrides it. */
  applyCamera(camera: PerspectiveCamera): void {
    this.intro.apply(camera, this.now);
  }

  /** Dev preview: stages the hazard `?hazard=` asks for, at a held clock. */
  private stagePreview(p: HazardPreview): void {
    const ticks = Math.round(p.seconds * TICK_HZ);
    // The clock is held at a point well after the start, so a staged "N ticks ago" is never negative.
    const clock = 1000 + ticks;
    this.held = clock;
    this.now = clock;
    for (const s of stagedEvents(p.name, ticks)) this.handleEvents([s.event], clock - s.before);
    if (p.name === 'b12') this.opts.loomSink?.(staged12Loom());
    if (p.name === 'b02') {
      // The intro needs the Stag's place before the first frame; frame() has no actors yet.
      this.intro.begin(clock - ticks, B02.home, -Math.PI / 2);
    }
  }

  dispose(): void {
    this.lava.dispose();
    this.toll.dispose();
    this.lamps.dispose();
    this.intro.dispose();
    setWeakPointOpen(false);
  }
}
