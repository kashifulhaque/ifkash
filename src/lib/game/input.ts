// Unified input state shared by desktop listeners and mobile touch UI.
export type InputState = {
  moveX: number; // -1..1 strafe
  moveY: number; // -1..1 forward
  lookDX: number; // accumulated, consumed each frame
  lookDY: number;
  fireQueued: boolean;
  interactQueued: boolean;
  jumpQueued: boolean;
  reloadQueued: boolean;
  run: boolean;
  crouch: boolean;
  aim: boolean; // hold right mouse button (or touch toggle) to aim down sights
};

export class InputManager {
  state: InputState = {
    moveX: 0,
    moveY: 0,
    lookDX: 0,
    lookDY: 0,
    fireQueued: false,
    interactQueued: false,
    jumpQueued: false,
    reloadQueued: false,
    run: false,
    crouch: false,
    aim: false
  };

  // Touch joystick writes here; merged with keys in read()
  touchMove = { x: 0, y: 0 };
  touchCrouch = false;
  touchAim = false;

  private keys = new Set<string>();
  private canvas: HTMLCanvasElement;
  private onLockChange: (locked: boolean) => void;
  enabled = true;

  constructor(canvas: HTMLCanvasElement, onLockChange: (locked: boolean) => void) {
    this.canvas = canvas;
    this.onLockChange = onLockChange;
    document.addEventListener('keydown', this.onKeyDown);
    document.addEventListener('keyup', this.onKeyUp);
    document.addEventListener('mousemove', this.onMouseMove);
    document.addEventListener('mousedown', this.onMouseDown);
    document.addEventListener('mouseup', this.onMouseUp);
    document.addEventListener('contextmenu', this.onContextMenu);
    document.addEventListener('pointerlockchange', this.onPointerLockChange);
  }

  get locked(): boolean {
    return document.pointerLockElement === this.canvas;
  }

  requestLock() {
    this.canvas.requestPointerLock?.();
  }

  exitLock() {
    if (this.locked) document.exitPointerLock();
  }

  // Touch UI hooks
  addLook(dx: number, dy: number) {
    this.state.lookDX += dx;
    this.state.lookDY += dy;
  }
  queueFire() {
    if (this.enabled) this.state.fireQueued = true;
  }
  queueInteract() {
    this.state.interactQueued = true;
  }
  queueJump() {
    this.state.jumpQueued = true;
  }
  queueReload() {
    this.state.reloadQueued = true;
  }

  private onKeyDown = (e: KeyboardEvent) => {
    if (e.repeat) return;
    this.keys.add(e.code);
    if (e.code === 'KeyE') this.state.interactQueued = true;
    if (e.code === 'Space') this.state.jumpQueued = true;
    if (e.code === 'KeyR') this.state.reloadQueued = true;
  };
  private onKeyUp = (e: KeyboardEvent) => {
    this.keys.delete(e.code);
  };
  private onMouseMove = (e: MouseEvent) => {
    if (!this.locked) return;
    this.state.lookDX += e.movementX;
    this.state.lookDY += e.movementY;
  };
  private onMouseDown = (e: MouseEvent) => {
    if (!this.locked || !this.enabled) return;
    if (e.button === 0) this.state.fireQueued = true;
    if (e.button === 2) this.state.aim = true;
  };
  private onMouseUp = (e: MouseEvent) => {
    if (e.button === 2) this.state.aim = false;
  };
  private onContextMenu = (e: Event) => {
    if (this.locked) e.preventDefault();
  };
  private onPointerLockChange = () => {
    this.onLockChange(this.locked);
  };

  // Snapshot movement from keys + touch, return state; caller zeroes deltas after use.
  read(): InputState {
    let kx = 0;
    let ky = 0;
    if (this.keys.has('KeyW') || this.keys.has('ArrowUp')) ky += 1;
    if (this.keys.has('KeyS') || this.keys.has('ArrowDown')) ky -= 1;
    if (this.keys.has('KeyD') || this.keys.has('ArrowRight')) kx += 1;
    if (this.keys.has('KeyA') || this.keys.has('ArrowLeft')) kx -= 1;
    this.state.moveX = Math.max(-1, Math.min(1, kx + this.touchMove.x));
    this.state.moveY = Math.max(-1, Math.min(1, ky + this.touchMove.y));
    this.state.run =
      this.keys.has('ShiftLeft') ||
      this.keys.has('ShiftRight') ||
      // Touch: pushing the stick to its edge sprints
      Math.hypot(this.touchMove.x, this.touchMove.y) > 0.95;
    this.state.crouch =
      this.keys.has('KeyC') || this.keys.has('ControlLeft') || this.touchCrouch;
    if (this.touchAim) this.state.aim = true;
    else if (!this.locked) this.state.aim = false;
    return this.state;
  }

  consumeFrame() {
    this.state.lookDX = 0;
    this.state.lookDY = 0;
    this.state.fireQueued = false;
    this.state.interactQueued = false;
    this.state.jumpQueued = false;
    this.state.reloadQueued = false;
  }

  dispose() {
    document.removeEventListener('keydown', this.onKeyDown);
    document.removeEventListener('keyup', this.onKeyUp);
    document.removeEventListener('mousemove', this.onMouseMove);
    document.removeEventListener('mousedown', this.onMouseDown);
    document.removeEventListener('mouseup', this.onMouseUp);
    document.removeEventListener('contextmenu', this.onContextMenu);
    document.removeEventListener('pointerlockchange', this.onPointerLockChange);
    this.exitLock();
  }
}
