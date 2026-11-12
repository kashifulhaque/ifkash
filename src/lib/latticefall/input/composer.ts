// Turns device state into one ActionFrame per tick. No DOM access, so it runs under Node.
//
// Devices call `press`, `release`, and `pulse` as events arrive. `compose` runs once per
// tick and reads the state. A press that starts and ends between two ticks is latched, so a
// fast tap is never lost.

import {
  ACTION_BIT,
  BTN,
  BUTTON_ACTIONS,
  type Action,
  type ButtonAction,
  type InputOptions,
  type UiAction
} from './bindings';
import { cameraRelative, packFrame } from './frame';

/** Ticks a one-button attack must be held before it counts as heavy. 10 ticks is 0.33 s. */
export const HEAVY_HOLD_TICKS = 10;

export class FrameComposer {
  /** Input sources currently holding each action down, such as `KeyW` or `Pad5`. */
  private held = new Map<Action, Set<string>>();
  /** Rising edges since the last `compose`. */
  private rises = new Map<Action, number>();
  private toggled = new Set<ButtonAction>();
  private attackActive = false;
  private attackTicks = 0;
  private padStick: [number, number] = [0, 0];
  private uiQueue: UiAction[] = [];

  /** Marks `source` as holding `action`. Repeated presses from the same source are ignored. */
  press(action: Action, source: string): void {
    let set = this.held.get(action);
    if (!set) this.held.set(action, (set = new Set()));
    if (set.has(source)) return;
    if (set.size === 0) this.rise(action);
    set.add(source);
  }

  release(action: Action, source: string): void {
    this.held.get(action)?.delete(source);
  }

  /** A press and release in one step, such as a mouse wheel notch. */
  pulse(action: Action): void {
    this.rise(action);
  }

  /** Releases everything, for example when the window loses focus. */
  releaseAll(): void {
    this.held.clear();
    this.padStick = [0, 0];
  }

  /** Gamepad left stick after the deadzone: x is right, y is forward, each in [-1, 1]. */
  setPadStick(x: number, y: number): void {
    this.padStick = [x, y];
  }

  /** UI actions (map, menu) pressed since the last call, in order. */
  drainUi(): UiAction[] {
    const q = this.uiQueue;
    this.uiQueue = [];
    return q;
  }

  isHeld(action: Action): boolean {
    return (this.held.get(action)?.size ?? 0) > 0;
  }

  private rise(action: Action): void {
    this.rises.set(action, (this.rises.get(action) ?? 0) + 1);
    if (action === 'map' || action === 'menu') this.uiQueue.push(action);
  }

  /**
   * The ActionFrame for the next tick. `cameraYaw` turns the camera-relative stick into
   * world-space `move_x` and `move_z`.
   */
  compose(cameraYaw: number, options: InputOptions): Uint8Array {
    // Movement. The keyboard is a full-tilt stick; the gamepad stick keeps its magnitude.
    let right = 0;
    let forward = 0;
    if (this.isHeld('moveRight')) right += 1;
    if (this.isHeld('moveLeft')) right -= 1;
    if (this.isHeld('moveForward')) forward += 1;
    if (this.isHeld('moveBack')) forward -= 1;
    const klen = Math.hypot(right, forward);
    if (klen > 0) {
      right /= klen;
      forward /= klen;
    }
    if (Math.hypot(this.padStick[0], this.padStick[1]) > Math.hypot(right, forward)) {
      right = this.padStick[0];
      forward = this.padStick[1];
    }
    const [mx, mz] = cameraRelative(right, forward, cameraYaw);

    // Buttons.
    let buttons = 0;
    for (const a of BUTTON_ACTIONS) {
      if (options.oneButtonAttack && (a === 'light' || a === 'heavy')) continue;
      const rises = this.rises.get(a) ?? 0;
      let on: boolean;
      if (options.toggle[a]) {
        if (rises % 2 === 1) {
          if (this.toggled.has(a)) this.toggled.delete(a);
          else this.toggled.add(a);
        }
        on = this.toggled.has(a);
      } else {
        this.toggled.delete(a);
        on = this.isHeld(a) || rises > 0;
      }
      if (on) buttons |= ACTION_BIT[a];
    }
    if (options.oneButtonAttack) buttons |= this.oneButtonAttack();

    this.rises.clear();
    return packFrame(mx, mz, buttons);
  }

  /** Tap is light (sent on release), hold is heavy (sent from the threshold until release). */
  private oneButtonAttack(): number {
    const down = this.isHeld('light') || this.isHeld('heavy');
    const rose = (this.rises.get('light') ?? 0) + (this.rises.get('heavy') ?? 0) > 0;
    let out = 0;
    if (!this.attackActive && (down || rose)) {
      this.attackActive = true;
      this.attackTicks = 0;
    }
    if (this.attackActive) {
      if (down) {
        this.attackTicks++;
        if (this.attackTicks >= HEAVY_HOLD_TICKS) out |= BTN.HEAVY;
      } else {
        if (this.attackTicks < HEAVY_HOLD_TICKS) out |= BTN.LIGHT;
        this.attackActive = false;
      }
    }
    return out;
  }
}
