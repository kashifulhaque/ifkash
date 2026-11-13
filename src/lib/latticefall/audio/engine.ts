// The audio engine: five buses (master, music, effects, ambience, UI) on one AudioContext,
// positional effects through `PannerNode`s, footsteps by surface, a procedural ambient bed
// and exploration music per region, and boss music stems. Sounds are driven from the core's
// decoded events and the HUD's boss-bar state, so the engine only listens: it never writes
// simulation state.
//
// Browsers keep an AudioContext suspended until a user gesture, so the context is created
// on construction but `resume()` runs from the first pointer or key event (`unlockOn`).

import type { GameEvent } from '../engine/events';
import { HIT_GUARDED, HIT_GUARD_BROKEN } from '../engine/events';
import { AmbienceBed } from './beds';
import { BossMusic, type BossMusicState } from './bosses';
import { BUSES, audioSettings, type AudioSettings, type Bus } from './settings';
import * as synth from './synth';
import type { Surface } from './synth';
import { bossKeyForName } from './voices';

/** Where the listener is: the player's position and the camera's yaw. */
export interface ListenerPose {
  x: number;
  y: number;
  z: number;
  yaw: number;
}

/** The HUD's boss bar: the boss's name, its HP fraction, and its phase (from 1). */
export interface BossBarState {
  name: string;
  frac: number;
  phase: number;
}

export type EffectName =
  | 'uiClick'
  | 'pickup'
  | 'lanternIgnite'
  | 'swingLight'
  | 'swingHeavy'
  | 'bellWindup'
  | 'bellToll'
  | 'tollHit'
  | 'iceCrack'
  | 'iceBreak'
  | 'chainSnap'
  | 'bossSting';

/** Looks up the surface under a world position. Defaults to stone until the world has materials. */
export type SurfaceResolver = (x: number, y: number, z: number) => Surface;

const MAX_VOICES = 24;
/** Meters between footsteps while walking. */
const STRIDE = 1.7;
/** A boss bar that clears at or below this HP fraction counts as a defeat. */
const DEFEAT_FRAC = 0.02;

// The Bellwright's mechanic sub-kinds (WASM event kind 14, see `game/crates/core/src/events.rs`).
const MECH_BELL_BEAT = 190;
const MECH_BELL_TOLL_WARNED = 191;
const MECH_BELL_TOLLED = 192;
const MECH_BELL_TOLL_HIT = 193;
const MECH_ICE_CRACKED = 194;
const MECH_ICE_BROKE = 195;
const MECH_BELL_CHAIN_SNAPPED = 197;

export class AudioEngine {
  readonly ctx: AudioContext;
  private buses = {} as Record<Bus, GainNode>;
  private noise: AudioBuffer;
  private ambience: AmbienceBed;
  private boss: BossMusic;
  private unsub: () => void;
  private voices = 0;
  private disposed = false;
  private unlockCleanup: (() => void) | null = null;
  private lastPos: { x: number; y: number; z: number } | null = null;
  private stride = 0;
  /** The boss fight the HUD shows now, with its last HP fraction, or null. */
  private fight: { name: string; frac: number } | null = null;
  surfaceAt: SurfaceResolver = () => 'stone';

  constructor() {
    const Ctor = window.AudioContext ?? (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    this.ctx = new Ctor({ latencyHint: 'interactive' });
    // Master feeds the speakers; the other four feed master.
    this.buses.master = this.ctx.createGain();
    this.buses.master.connect(this.ctx.destination);
    for (const bus of BUSES) {
      if (bus === 'master') continue;
      this.buses[bus] = this.ctx.createGain();
      this.buses[bus].connect(this.buses.master);
    }
    this.noise = synth.noiseBuffer(this.ctx, 2);
    this.ambience = new AmbienceBed(this.ctx, this.buses.ambience, this.buses.music, this.noise);
    this.boss = new BossMusic(this.ctx, this.buses.music, this.noise);
    this.unsub = audioSettings.subscribe((s) => this.applySettings(s));
    document.addEventListener('visibilitychange', this.onVisibility);
  }

  private applySettings(s: AudioSettings): void {
    if (this.disposed) return;
    const t = this.ctx.currentTime;
    for (const bus of BUSES) {
      // Muting zeroes the master bus only, so the per-bus volumes survive a mute.
      const v = bus === 'master' && s.muted ? 0 : s.volumes[bus];
      const p = this.buses[bus].gain;
      p.cancelScheduledValues(t);
      p.setTargetAtTime(v, t, 0.03);
    }
  }

  private onVisibility = (): void => {
    // A hidden tab goes quiet; coming back resumes only if a gesture already unlocked audio.
    if (document.hidden) void this.ctx.suspend();
    else if (this.unlocked) void this.ctx.resume();
  };

  private unlocked = false;

  /** Resumes the context on the first pointer or key event on `target`. Call once. */
  unlockOn(target: EventTarget): void {
    const go = () => {
      this.unlocked = true;
      if (!document.hidden) void this.ctx.resume();
      this.unlockCleanup?.();
    };
    const events = ['pointerdown', 'keydown', 'touchend'];
    for (const e of events) target.addEventListener(e, go, { once: false, passive: true });
    this.unlockCleanup = () => {
      for (const e of events) target.removeEventListener(e, go);
      this.unlockCleanup = null;
    };
  }

  /** Moves the listener to the player and turns it to face where the camera looks. */
  setListener(p: ListenerPose): void {
    const l = this.ctx.listener;
    const t = this.ctx.currentTime;
    // The camera sits on the +Z side at yaw 0 and looks toward -Z.
    const fx = -Math.sin(p.yaw);
    const fz = -Math.cos(p.yaw);
    if (l.positionX) {
      l.positionX.setValueAtTime(p.x, t);
      l.positionY.setValueAtTime(p.y, t);
      l.positionZ.setValueAtTime(p.z, t);
      l.forwardX.setValueAtTime(fx, t);
      l.forwardY.setValueAtTime(0, t);
      l.forwardZ.setValueAtTime(fz, t);
      l.upX.setValueAtTime(0, t);
      l.upY.setValueAtTime(1, t);
      l.upZ.setValueAtTime(0, t);
    } else {
      l.setPosition(p.x, p.y, p.z);
      l.setOrientation(fx, 0, fz, 0, 1, 0);
    }
  }

  /** Runs `play` on a panner at a world position, on the effects bus, within the voice cap. */
  private positional(at: [number, number, number], play: (dest: AudioNode, t: number) => void): void {
    if (this.disposed || this.ctx.state !== 'running' || this.voices >= MAX_VOICES) return;
    const panner = this.ctx.createPanner();
    panner.panningModel = 'HRTF';
    panner.distanceModel = 'inverse';
    panner.refDistance = 2;
    panner.rolloffFactor = 1.2;
    panner.maxDistance = 60;
    panner.positionX.value = at[0];
    panner.positionY.value = at[1];
    panner.positionZ.value = at[2];
    panner.connect(this.buses.effects);
    this.voices++;
    play(panner, this.ctx.currentTime);
    // Sounds here are all under 1.5 s; free the panner and the voice slot after that.
    setTimeout(() => {
      panner.disconnect();
      this.voices--;
    }, 1600);
  }

  /** Plays sounds for the events the core produced this tick. */
  handleEvents(events: readonly GameEvent[]): void {
    for (const e of events) {
      if (e.kind === 'mechanic') this.mechanicEvent(e.what, e.index);
      if (e.kind !== 'hit') continue;
      const killed = (e.flags & 2) !== 0;
      const kind: synth.HitKind =
        e.result === HIT_GUARD_BROKEN ? 'guardBreak' : e.result === HIT_GUARDED ? 'guard' : killed ? 'kill' : 'flesh';
      // Motion value is around 0.3 to 2.5; map it to a 0-1 weight.
      const weight = Math.min(1, e.motionValue / 2.5);
      this.positional(e.point, (dest, t) => synth.hit(this.ctx, dest, this.noise, t, kind, weight));
    }
  }

  /**
   * The Bellwright's events drive his beat and his bell. The beat follows the core's events,
   * never the audio clock, so the music can't drift from the fight.
   */
  private mechanicEvent(what: number, index: number): void {
    switch (what) {
      case MECH_BELL_BEAT:
        this.boss.onBellBeat(index);
        break;
      case MECH_BELL_TOLL_WARNED:
        this.play('bellWindup');
        break;
      case MECH_BELL_TOLLED:
        this.play('bellToll');
        break;
      case MECH_BELL_TOLL_HIT:
        this.play('tollHit');
        break;
      case MECH_ICE_CRACKED:
        this.play('iceCrack');
        break;
      case MECH_ICE_BROKE:
        this.play('iceBreak');
        break;
      case MECH_BELL_CHAIN_SNAPPED:
        this.play('chainSnap');
        break;
    }
  }

  /**
   * Call every frame with the player's position. Plays a footstep each time the player
   * has walked a stride along the ground; jumps and teleports reset the count.
   */
  updatePlayer(pose: ListenerPose): void {
    this.setListener(pose);
    const last = this.lastPos;
    this.lastPos = { x: pose.x, y: pose.y, z: pose.z };
    if (!last) return;
    const dx = pose.x - last.x;
    const dz = pose.z - last.z;
    const step = Math.hypot(dx, dz);
    const airborne = Math.abs(pose.y - last.y) > 0.08;
    if (step > 3 || airborne) {
      this.stride = 0;
      return;
    }
    this.stride += step;
    if (this.stride >= STRIDE) {
      this.stride -= STRIDE;
      const surface = this.surfaceAt(pose.x, pose.y, pose.z);
      this.positional([pose.x, pose.y, pose.z], (dest, t) => synth.footstep(this.ctx, dest, this.noise, t, surface));
    }
  }

  /** Plays a non-positional effect: UI sounds on the UI bus, the rest on effects. */
  play(name: EffectName): void {
    if (this.disposed || this.ctx.state !== 'running') return;
    const t = this.ctx.currentTime;
    const fx = this.buses.effects;
    switch (name) {
      case 'uiClick':
        synth.uiClick(this.ctx, this.buses.ui, t);
        break;
      case 'pickup':
        synth.pickup(this.ctx, fx, t);
        break;
      case 'lanternIgnite':
        synth.lanternIgnite(this.ctx, fx, this.noise, t);
        break;
      case 'swingLight':
      case 'swingHeavy':
        synth.swing(this.ctx, fx, this.noise, t, name === 'swingHeavy');
        break;
      case 'bellWindup':
        synth.bellWindup(this.ctx, fx, t);
        break;
      case 'bellToll':
        synth.bellToll(this.ctx, fx, this.noise, t);
        break;
      case 'tollHit':
        synth.tollHit(this.ctx, fx, this.noise, t);
        break;
      case 'iceCrack':
        synth.iceCrack(this.ctx, fx, this.noise, t);
        break;
      case 'iceBreak':
        synth.iceBreak(this.ctx, fx, this.noise, t);
        break;
      case 'chainSnap':
        synth.chainSnap(this.ctx, fx, this.noise, t);
        break;
      case 'bossSting':
        synth.bossSting(this.ctx, fx, t);
        break;
    }
  }

  /** Crossfades the ambient bed and exploration music to an area (`waking_hollow`, `verge`, ...). */
  setRegion(id: string): void {
    this.ambience.setRegion(id);
  }

  /**
   * Follows the HUD's boss bar, once per tick: a new name starts that boss's intro, a phase
   * of 2 or more layers in the phase-2 stem, and a bar that clears ends the fight. A bar that
   * clears at no HP plays the defeat sting; anything else (a reset or a death) just goes quiet.
   */
  setBossBar(bar: BossBarState | null): void {
    if (this.disposed) return;
    if (!bar) {
      if (this.fight) {
        if (this.fight.frac <= DEFEAT_FRAC) this.play('bossSting');
        this.boss.set('off');
        this.fight = null;
      }
      return;
    }
    if (!this.fight || this.fight.name !== bar.name) {
      this.boss.set('off');
      this.boss.setVoice(bossKeyForName(bar.name));
      this.boss.set('intro');
    }
    this.fight = { name: bar.name, frac: bar.frac };
    this.boss.set(bar.phase >= 2 ? 'phase2' : 'loop');
  }

  /** Sets the boss music directly: 'intro' swells in, 'loop' pulses, 'phase2' layers the second stem. */
  setBossMusic(state: BossMusicState): void {
    this.boss.set(state);
  }

  get bossMusic(): BossMusicState {
    return this.boss.current;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.unsub();
    this.unlockCleanup?.();
    document.removeEventListener('visibilitychange', this.onVisibility);
    this.ambience.dispose();
    this.boss.dispose();
    void this.ctx.close();
  }
}
