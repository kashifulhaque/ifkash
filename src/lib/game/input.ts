// Keyboard, pointer, wheel, and touch input for the planet walker. The Game
// polls this each frame; one-shot actions are queued and consumed once.

export type Click = { x: number; y: number };

const MOVE_KEYS: Record<string, [number, number]> = {
  KeyW: [0, 1],
  ArrowUp: [0, 1],
  KeyS: [0, -1],
  ArrowDown: [0, -1],
  KeyA: [-1, 0],
  ArrowLeft: [-1, 0],
  KeyD: [1, 0],
  ArrowRight: [1, 0]
};

const DRAG_THRESHOLD = 5;

export class Input {
  /** Virtual joystick vector from the touch controls, each axis in [-1, 1]. */
  touchMove = { x: 0, y: 0 };
  touchRun = false;
  /** When false, keys and clicks are ignored (an overlay is open). */
  enabled = true;

  private keys = new Set<string>();
  private hopQueued = false;
  private interactQueued = false;
  private globeQueued = false;
  private helpQueued = false;
  private clockQueued = false;
  private photoQueued = false;
  private escapeQueued = false;
  private orbitDx = 0;
  private orbitDy = 0;
  private zoom = 0;
  private click: Click | null = null;
  private pointers = new Map<number, { x: number; y: number }>();
  private dragStart: { x: number; y: number } | null = null;
  private dragged = false;
  private pinchDist = 0;
  private canvas: HTMLCanvasElement;
  private onAny: (() => void) | null = null;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    window.addEventListener('keydown', this.onKeyDown);
    window.addEventListener('keyup', this.onKeyUp);
    window.addEventListener('blur', this.onBlur);
    canvas.addEventListener('pointerdown', this.onPointerDown);
    canvas.addEventListener('pointermove', this.onPointerMove);
    canvas.addEventListener('pointerup', this.onPointerUp);
    canvas.addEventListener('pointercancel', this.onPointerUp);
    canvas.addEventListener('wheel', this.onWheel, { passive: false });
    canvas.addEventListener('contextmenu', this.onContext);
  }

  /** Called once on the first movement key, hop, or click (used to leave the intro). */
  onFirstGesture(cb: () => void): void {
    this.onAny = cb;
  }

  private fireAny(): void {
    if (this.onAny) {
      const cb = this.onAny;
      this.onAny = null;
      cb();
    }
  }

  get moveX(): number {
    let x = this.touchMove.x;
    if (this.enabled) for (const k of this.keys) x += MOVE_KEYS[k]?.[0] ?? 0;
    return Math.max(-1, Math.min(1, x));
  }

  get moveY(): number {
    let y = this.touchMove.y;
    if (this.enabled) for (const k of this.keys) y += MOVE_KEYS[k]?.[1] ?? 0;
    return Math.max(-1, Math.min(1, y));
  }

  get run(): boolean {
    return this.touchRun || (this.enabled && (this.keys.has('ShiftLeft') || this.keys.has('ShiftRight')));
  }

  get dragging(): boolean {
    return this.dragged;
  }

  queueHop(): void {
    this.hopQueued = true;
    this.fireAny();
  }
  queueInteract(): void {
    this.interactQueued = true;
  }
  queueGlobe(): void {
    this.globeQueued = true;
  }
  queuePhoto(): void {
    this.photoQueued = true;
  }
  /** Touch joystick moved: counts as the first gesture. */
  notifyTouchMove(): void {
    this.fireAny();
  }

  consumeHop(): boolean {
    const v = this.hopQueued;
    this.hopQueued = false;
    return v;
  }
  consumeInteract(): boolean {
    const v = this.interactQueued;
    this.interactQueued = false;
    return v;
  }
  consumeGlobe(): boolean {
    const v = this.globeQueued;
    this.globeQueued = false;
    return v;
  }
  consumePhoto(): boolean {
    const v = this.photoQueued;
    this.photoQueued = false;
    return v;
  }
  consumeClock(): boolean {
    const v = this.clockQueued;
    this.clockQueued = false;
    return v;
  }
  consumeHelp(): boolean {
    const v = this.helpQueued;
    this.helpQueued = false;
    return v;
  }
  consumeEscape(): boolean {
    const v = this.escapeQueued;
    this.escapeQueued = false;
    return v;
  }
  consumeOrbit(): { dx: number; dy: number } {
    const r = { dx: this.orbitDx, dy: this.orbitDy };
    this.orbitDx = 0;
    this.orbitDy = 0;
    return r;
  }
  consumeZoom(): number {
    const z = this.zoom;
    this.zoom = 0;
    return z;
  }
  consumeClick(): Click | null {
    const c = this.click;
    this.click = null;
    return c;
  }

  private onKeyDown = (e: KeyboardEvent) => {
    const target = e.target as HTMLElement | null;
    if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) return;
    if (e.code === 'Escape') {
      this.escapeQueued = true;
      return;
    }
    if (!this.enabled) return;
    if (e.code in MOVE_KEYS) {
      e.preventDefault();
      this.keys.add(e.code);
      this.fireAny();
      return;
    }
    switch (e.code) {
      case 'ShiftLeft':
      case 'ShiftRight':
        this.keys.add(e.code);
        break;
      case 'Space':
        e.preventDefault();
        if (!e.repeat) this.queueHop();
        break;
      case 'KeyE':
      case 'Enter':
        if (!e.repeat) this.interactQueued = true;
        break;
      case 'KeyM':
        if (!e.repeat) this.globeQueued = true;
        break;
      case 'KeyH':
      case 'Slash':
        if (!e.repeat) this.helpQueued = true;
        break;
      case 'KeyT':
        if (!e.repeat) this.clockQueued = true;
        break;
      case 'KeyP':
        if (!e.repeat) this.photoQueued = true;
        break;
    }
  };

  private onKeyUp = (e: KeyboardEvent) => {
    this.keys.delete(e.code);
  };

  private onBlur = () => {
    this.keys.clear();
    this.pointers.clear();
    this.dragStart = null;
    this.dragged = false;
  };

  private onPointerDown = (e: PointerEvent) => {
    this.canvas.setPointerCapture(e.pointerId);
    this.pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (this.pointers.size === 1) {
      this.dragStart = { x: e.clientX, y: e.clientY };
      this.dragged = false;
    } else if (this.pointers.size === 2) {
      this.pinchDist = this.pointerDistance();
      this.dragged = true; // a pinch is never a click
    }
  };

  private onPointerMove = (e: PointerEvent) => {
    const prev = this.pointers.get(e.pointerId);
    if (!prev) return;
    const dx = e.clientX - prev.x;
    const dy = e.clientY - prev.y;
    this.pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });

    if (this.pointers.size === 1) {
      if (this.dragStart && !this.dragged) {
        const mx = e.clientX - this.dragStart.x;
        const my = e.clientY - this.dragStart.y;
        if (Math.hypot(mx, my) > DRAG_THRESHOLD) this.dragged = true;
      }
      if (this.dragged) {
        this.orbitDx += dx;
        this.orbitDy += dy;
      }
    } else if (this.pointers.size === 2) {
      const d = this.pointerDistance();
      if (this.pinchDist > 0) this.zoom += (this.pinchDist - d) * 0.02;
      this.pinchDist = d;
    }
  };

  private onPointerUp = (e: PointerEvent) => {
    const had = this.pointers.delete(e.pointerId);
    if (!had) return;
    if (this.pointers.size === 0) {
      if (!this.dragged && this.dragStart && this.enabled) {
        const rect = this.canvas.getBoundingClientRect();
        this.click = {
          x: ((e.clientX - rect.left) / rect.width) * 2 - 1,
          y: -((e.clientY - rect.top) / rect.height) * 2 + 1
        };
        this.fireAny();
      }
      this.dragStart = null;
      this.dragged = false;
    }
  };

  private onWheel = (e: WheelEvent) => {
    e.preventDefault();
    this.zoom += Math.sign(e.deltaY) * Math.min(1.5, Math.abs(e.deltaY) * 0.01);
  };

  private onContext = (e: Event) => e.preventDefault();

  private pointerDistance(): number {
    const pts = [...this.pointers.values()];
    if (pts.length < 2) return 0;
    return Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
  }

  dispose(): void {
    window.removeEventListener('keydown', this.onKeyDown);
    window.removeEventListener('keyup', this.onKeyUp);
    window.removeEventListener('blur', this.onBlur);
    this.canvas.removeEventListener('pointerdown', this.onPointerDown);
    this.canvas.removeEventListener('pointermove', this.onPointerMove);
    this.canvas.removeEventListener('pointerup', this.onPointerUp);
    this.canvas.removeEventListener('pointercancel', this.onPointerUp);
    this.canvas.removeEventListener('wheel', this.onWheel);
    this.canvas.removeEventListener('contextmenu', this.onContext);
  }
}
