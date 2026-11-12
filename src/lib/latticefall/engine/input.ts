// Minimal keyboard input: WASD or arrow keys become a 4-byte ActionFrame
// ([move_x, move_z, buttons_lo, buttons_hi], see lf-core `ActionFrame`).
// WP-011 replaces this with camera-relative movement and the full button set.
//
// Axes are world axes. The camera sits behind the player on the +Z side looking toward -Z,
// so W (up) is move_z = -1 and D (right) is move_x = +1. Shift runs (magnitude 127);
// otherwise the stick is at 60, a walk.

const WALK = 60;
const RUN = 127;

export class KeyboardInput {
  private down = new Set<string>();

  attach(): void {
    window.addEventListener('keydown', this.onDown);
    window.addEventListener('keyup', this.onUp);
    window.addEventListener('blur', this.onBlur);
  }

  detach(): void {
    window.removeEventListener('keydown', this.onDown);
    window.removeEventListener('keyup', this.onUp);
    window.removeEventListener('blur', this.onBlur);
    this.down.clear();
  }

  /** The ActionFrame for this tick. */
  frame(): Uint8Array {
    const k = this.down;
    let x = 0;
    let z = 0;
    if (k.has('KeyD') || k.has('ArrowRight')) x += 1;
    if (k.has('KeyA') || k.has('ArrowLeft')) x -= 1;
    if (k.has('KeyS') || k.has('ArrowDown')) z += 1;
    if (k.has('KeyW') || k.has('ArrowUp')) z -= 1;
    const out = new Uint8Array(4);
    const len = Math.hypot(x, z);
    if (len > 0) {
      const mag = k.has('ShiftLeft') || k.has('ShiftRight') ? RUN : WALK;
      // Two's complement bytes for i8: Uint8Array wraps negatives for us.
      out[0] = Math.round((x / len) * mag);
      out[1] = Math.round((z / len) * mag);
    }
    return out;
  }

  private static isTyping(e: KeyboardEvent): boolean {
    const t = e.target;
    return t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName));
  }

  private onDown = (e: KeyboardEvent): void => {
    if (KeyboardInput.isTyping(e) || e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.code.startsWith('Arrow')) e.preventDefault();
    this.down.add(e.code);
  };

  private onUp = (e: KeyboardEvent): void => {
    this.down.delete(e.code);
  };

  private onBlur = (): void => {
    this.down.clear();
  };
}
