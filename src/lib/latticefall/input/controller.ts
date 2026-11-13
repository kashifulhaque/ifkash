// Browser input devices: keyboard (KeyboardEvent.code), mouse (pointer lock) and gamepad
// (navigator.getGamepads(), polled). They feed a FrameComposer, which makes the per-tick
// ActionFrame. Browser only.

import {
  loadSettings,
  parseToken,
  resolveActions,
  saveSettings,
  type Action,
  type InputSettings,
  type UiAction
} from './bindings';
import { FrameComposer } from './composer';
import { PITCH_DEFAULT, PITCH_MAX, PITCH_MIN } from '../render/camera/limits';
import { radialDeadzone } from './frame';
import { createPadReader, padActive, STICK_DEADZONE } from './gamepad';
import { lastDevice } from './device';
import type { TouchSink } from './touch/sink';

const STICK_CAMERA_RAD_PER_S = 2.6;
const FLICK_ON = 0.85;
const FLICK_OFF = 0.5;

export class InputController {
  readonly settings: InputSettings;
  /** Camera yaw in radians. See `cameraRelative` for the convention. */
  cameraYaw = 0;
  /** Camera elevation in radians above the pivot's horizon; positive looks down from higher up. */
  cameraPitch = PITCH_DEFAULT;
  /**
   * Decides what gamepad X does: true means Interact, false means Use item. The engine sets
   * it to "the core shows an interact prompt"; the default always interacts.
   */
  nearInteractable: () => boolean = () => true;

  private composer = new FrameComposer();
  private canvas: HTMLCanvasElement | null = null;
  /** Which actions each held keyboard or mouse input pressed, so release matches the press. */
  private pressed = new Map<string, Action[]>();
  private padPressed = new Map<number, Action[]>();
  private padLast: boolean[] = [];
  private flick = 0;
  private skipNextClick = false;
  private wasLocked = false;
  private lookDx = 0;
  private lookDy = 0;
  private padLook: [number, number] = [0, 0];
  /** Reads the most recently used pad in standard layout (see `gamepad.ts`). */
  private readPad = createPadReader();

  constructor(settings: InputSettings = loadSettings()) {
    this.settings = settings;
  }

  attach(canvas: HTMLCanvasElement): void {
    this.canvas = canvas;
    window.addEventListener('keydown', this.onKeyDown);
    window.addEventListener('keyup', this.onKeyUp);
    window.addEventListener('blur', this.onBlur);
    window.addEventListener('mousedown', this.onMouseDown);
    window.addEventListener('mouseup', this.onMouseUp);
    window.addEventListener('mousemove', this.onMouseMove);
    window.addEventListener('wheel', this.onWheel, { passive: false });
    canvas.addEventListener('contextmenu', this.onContextMenu);
    document.addEventListener('pointerlockchange', this.onLockChange);
    window.addEventListener('gamepaddisconnected', this.onPadDisconnected);
  }

  detach(): void {
    window.removeEventListener('keydown', this.onKeyDown);
    window.removeEventListener('keyup', this.onKeyUp);
    window.removeEventListener('blur', this.onBlur);
    window.removeEventListener('mousedown', this.onMouseDown);
    window.removeEventListener('mouseup', this.onMouseUp);
    window.removeEventListener('mousemove', this.onMouseMove);
    window.removeEventListener('wheel', this.onWheel);
    this.canvas?.removeEventListener('contextmenu', this.onContextMenu);
    document.removeEventListener('pointerlockchange', this.onLockChange);
    window.removeEventListener('gamepaddisconnected', this.onPadDisconnected);
    if (this.locked) document.exitPointerLock();
    this.canvas = null;
    this.composer.releaseAll();
    this.pressed.clear();
    this.padPressed.clear();
  }

  get locked(): boolean {
    return !!this.canvas && document.pointerLockElement === this.canvas;
  }

  save(): void {
    saveSettings(this.settings);
  }

  private suspended = false;

  /**
   * While a menu is open the game takes no input: held keys and buttons are released, and new
   * presses are ignored (the menus read the keyboard and gamepad themselves). Pad buttons still
   * held when the game resumes stay ignored until they are let go, so the press that closed a
   * menu doesn't also jump.
   */
  setSuspended(on: boolean): void {
    if (on === this.suspended) return;
    this.suspended = on;
    this.composer.releaseAll();
    this.pressed.clear();
    this.padPressed.clear();
    this.skipNextClick = false;
    if (!on) {
      const pad = this.readPad();
      this.padLast = pad ? [...pad.pressed] : [];
    }
  }

  /**
   * Polls the gamepad. Call once per rendered frame, before the ticks it feeds. Any pad works:
   * `gamepad.ts` turns wired, Bluetooth, and unknown layouts into the standard one.
   */
  poll(): void {
    const pad = this.readPad();
    if (!pad) {
      if (this.padLast.length > 0) this.releasePad();
      this.composer.setPadStick(0, 0);
      this.padLook = [0, 0];
      return;
    }
    if (padActive(pad)) lastDevice.set('pad');
    // Left stick: x right, y up. The Gamepad API reports y positive downward.
    const [lx, ly] = radialDeadzone(pad.axes[0], pad.axes[1], STICK_DEADZONE);
    this.composer.setPadStick(lx, -ly);
    const [rx, ry] = radialDeadzone(pad.axes[2], pad.axes[3], STICK_DEADZONE);
    this.padLook = [rx, ry];

    const n = Math.max(pad.pressed.length, this.padLast.length);
    for (let i = 0; i < n; i++) {
      const down = !!pad.pressed[i];
      if (down === !!this.padLast[i]) continue;
      this.padLast[i] = down;
      if (down) this.padDown(i);
      else this.padUp(i);
    }

    // Right stick flick switches target. A flick is a fast move to the edge of the range.
    const f = Math.abs(pad.axes[2]) > FLICK_ON && Math.abs(pad.axes[3]) < 0.5 ? Math.sign(pad.axes[2]) : 0;
    if (f !== 0 && this.flick === 0 && !this.suspended) this.composer.pulse(f < 0 ? 'switchL' : 'switchR');
    if (f !== 0) this.flick = f;
    else if (Math.abs(pad.axes[2]) < FLICK_OFF) this.flick = 0;
  }

  /** Applies mouse and right-stick look to the camera. Call once per rendered frame. */
  updateLook(frameMs: number): void {
    const { mouseSensitivity, invertY } = this.settings.options;
    const dy = invertY ? -1 : 1;
    this.cameraYaw -= this.lookDx * mouseSensitivity + this.padLook[0] * STICK_CAMERA_RAD_PER_S * (frameMs / 1000);
    this.cameraPitch += dy * (this.lookDy * mouseSensitivity + this.padLook[1] * STICK_CAMERA_RAD_PER_S * (frameMs / 1000));
    this.cameraPitch = Math.max(PITCH_MIN, Math.min(PITCH_MAX, this.cameraPitch));
    // Keep yaw in (-pi, pi] so it never loses float precision.
    this.cameraYaw = Math.atan2(Math.sin(this.cameraYaw), Math.cos(this.cameraYaw));
    this.lookDx = 0;
    this.lookDy = 0;
  }

  /** The ActionFrame for the next tick, in world space. */
  frame(): Uint8Array {
    return this.composer.compose(this.cameraYaw, this.settings.options);
  }

  /**
   * The touch surface's way in (WP-083). It feeds the same composer as the keyboard and the
   * gamepad, so touch makes the same ActionFrame. Ignored while the menus are open.
   */
  touchSink(): TouchSink {
    return {
      press: (a, src) => {
        if (this.suspended) return;
        lastDevice.set('touch');
        this.composer.press(a, src);
      },
      release: (a, src) => this.composer.release(a, src),
      pulse: (a) => !this.suspended && this.composer.pulse(a),
      setStick: (x, y) => this.composer.setTouchStick(x, y),
      look: (dx, dy) => {
        if (this.suspended) return;
        this.lookDx += dx;
        this.lookDy += dy;
      }
    };
  }

  /** Map and menu presses since the last call. */
  drainUi(): UiAction[] {
    return this.composer.drainUi();
  }

  private padDown(i: number): void {
    let actions = resolveActions(this.settings.bindings, `Pad${i}`, false);
    // X is Interact near an interactable, Use item otherwise.
    if (actions.includes('interact') && actions.includes('useItem')) {
      const drop: Action = this.nearInteractable() ? 'useItem' : 'interact';
      actions = actions.filter((a) => a !== drop);
    }
    this.padPressed.set(i, actions);
    for (const a of actions) this.composer.press(a, `Pad${i}`);
  }

  private padUp(i: number): void {
    for (const a of this.padPressed.get(i) ?? []) this.composer.release(a, `Pad${i}`);
    this.padPressed.delete(i);
  }

  private releasePad(): void {
    for (const i of [...this.padPressed.keys()]) this.padUp(i);
    this.padLast = [];
    this.flick = 0;
  }

  private static isTyping(e: Event): boolean {
    const t = e.target;
    return t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName));
  }

  private down(token: string, shift: boolean): void {
    if (this.pressed.has(token)) return;
    lastDevice.set('keyboard');
    const actions = resolveActions(this.settings.bindings, token, shift);
    this.pressed.set(token, actions);
    for (const a of actions) this.composer.press(a, token);
  }

  private up(token: string): void {
    for (const a of this.pressed.get(token) ?? []) this.composer.release(a, token);
    this.pressed.delete(token);
  }

  /** True if some binding uses this key, so the page should not also act on it. */
  private isBound(base: string): boolean {
    for (const tokens of Object.values(this.settings.bindings)) {
      if (tokens.some((t) => parseToken(t).base === base)) return true;
    }
    return false;
  }

  private onKeyDown = (e: KeyboardEvent): void => {
    if (this.suspended || InputController.isTyping(e) || e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.repeat) {
      if (this.isBound(e.code)) e.preventDefault();
      return;
    }
    if (this.isBound(e.code)) {
      if (e.code === 'Space' || e.code.startsWith('Arrow')) e.preventDefault();
      this.down(e.code, e.shiftKey);
    }
  };

  private onKeyUp = (e: KeyboardEvent): void => {
    this.up(e.code);
  };

  /** A pad that drops (cable out, Bluetooth asleep) lets go of everything it held. */
  private onPadDisconnected = (): void => {
    this.releasePad();
    this.composer.setPadStick(0, 0);
    this.padLook = [0, 0];
  };

  private onBlur = (): void => {
    this.composer.releaseAll();
    this.pressed.clear();
  };

  private onMouseDown = (e: MouseEvent): void => {
    if (this.suspended || !this.canvas || InputController.isTyping(e)) return;
    if (!this.locked) {
      // The first click on the canvas captures the mouse and is not an attack.
      if (e.target === this.canvas && e.button === 0) {
        this.skipNextClick = true;
        void this.canvas.requestPointerLock?.();
      }
      return;
    }
    e.preventDefault();
    this.down(`Mouse${e.button}`, e.shiftKey);
  };

  private onMouseUp = (e: MouseEvent): void => {
    if (this.skipNextClick && e.button === 0) {
      this.skipNextClick = false;
      return;
    }
    this.up(`Mouse${e.button}`);
  };

  private onMouseMove = (e: MouseEvent): void => {
    if (!this.locked) return;
    this.lookDx += e.movementX;
    this.lookDy += e.movementY;
  };

  private onWheel = (e: WheelEvent): void => {
    if (this.suspended || !this.locked || e.deltaY === 0) return;
    e.preventDefault();
    const base = e.deltaY < 0 ? 'WheelUp' : 'WheelDown';
    for (const a of resolveActions(this.settings.bindings, base, e.shiftKey)) this.composer.pulse(a);
  };

  private onContextMenu = (e: Event): void => {
    e.preventDefault();
  };

  private onLockChange = (): void => {
    const locked = this.locked;
    if (!locked) {
      // Leaving pointer lock (Escape in most browsers) opens the menu and drops mouse holds.
      for (const t of [...this.pressed.keys()]) if (t.startsWith('Mouse')) this.up(t);
      if (this.wasLocked) this.composer.pulse('menu');
    }
    this.wasLocked = locked;
  };
}
