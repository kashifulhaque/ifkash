// Menu navigation shared by the keyboard and the gamepad. Every control in a menu carries
// `data-nav="<column>"`; the screen's root lists its columns left to right in `data-cols`.
// Up and Down move within a column, Left and Right move between columns (or adjust a slider
// that has focus), and a gamepad's D-pad, A, B, and Start map to the same moves.

/** True while a control is waiting for the next key or button to bind; navigation stands down. */
export const navState = { capturing: false };

export type NavMove = 'up' | 'down' | 'left' | 'right' | 'confirm' | 'back' | 'menu';

function navItems(root: HTMLElement, col?: string): HTMLElement[] {
  const sel = col ? `[data-nav="${col}"]` : '[data-nav]';
  return [...root.querySelectorAll<HTMLElement>(sel)].filter((el) => !el.hasAttribute('disabled') && el.offsetParent !== null);
}

/** The column of the focused control, or null if focus is outside `root`. */
function currentCol(root: HTMLElement): string | null {
  const a = document.activeElement;
  return a instanceof HTMLElement && root.contains(a) ? (a.dataset.nav ?? null) : null;
}

/** Focuses the first control of `col`, or of any column. Returns false if there is none. */
export function focusFirst(root: HTMLElement, col?: string): boolean {
  const el = navItems(root, col)[0] ?? navItems(root)[0];
  el?.focus();
  return !!el;
}

function adjustRange(el: HTMLInputElement, dir: 1 | -1): void {
  const step = Number(el.step) || 1;
  const next = Math.min(Number(el.max), Math.max(Number(el.min), Number(el.value) + dir * step));
  if (next === Number(el.value)) return;
  el.value = String(next);
  el.dispatchEvent(new Event('input', { bubbles: true }));
  el.dispatchEvent(new Event('change', { bubbles: true }));
}

/**
 * Applies a move to the controls under `root`. Returns true if it did something with
 * `up`, `down`, `left`, `right`, or `confirm`; `back` and `menu` are the caller's.
 */
export function navigate(root: HTMLElement, move: NavMove): boolean {
  const active = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const col = currentCol(root);
  if (col === null) return focusFirst(root);
  const cols = (root.dataset.cols ?? '').split(',').filter(Boolean);
  if (move === 'up' || move === 'down') {
    const items = navItems(root, col);
    const i = active ? items.indexOf(active) : -1;
    const n = items.length;
    if (n === 0) return false;
    items[(i + (move === 'down' ? 1 : -1) + n) % n].focus();
    return true;
  }
  if (move === 'left' || move === 'right') {
    if (active instanceof HTMLInputElement && active.type === 'range') {
      adjustRange(active, move === 'right' ? 1 : -1);
      return true;
    }
    const at = cols.indexOf(col);
    const next = cols[at + (move === 'right' ? 1 : -1)];
    return next ? focusFirst(root, next) : false;
  }
  if (move === 'confirm' && active) {
    active.click();
    return true;
  }
  return false;
}

/** Maps a keyboard event to a move, or null if it isn't one. Typing fields keep their keys. */
export function moveForKey(e: KeyboardEvent): NavMove | null {
  if (e.ctrlKey || e.metaKey || e.altKey) return null;
  switch (e.key) {
    case 'ArrowUp':
      return 'up';
    case 'ArrowDown':
      return 'down';
    case 'ArrowLeft':
      return 'left';
    case 'ArrowRight':
      return 'right';
    case 'Escape':
      return 'back';
    default:
      return null;
  }
}

/** Standard-layout gamepad buttons the menus read. */
const PAD = { A: 0, B: 1, START: 9, UP: 12, DOWN: 13, LEFT: 14, RIGHT: 15 } as const;
const STICK_ON = 0.6;
const REPEAT_MS = 180;
const FIRST_REPEAT_MS = 380;

/**
 * Polls the first standard gamepad every frame and calls `onMove` on button edges, with
 * repeat for the D-pad and the left stick. Returns a stop function. Buttons already held
 * when it starts are ignored until released, so the press that opened a menu doesn't also
 * confirm something in it.
 */
export function startGamepadNav(onMove: (m: NavMove) => void): () => void {
  let raf = 0;
  let stopped = false;
  const held = new Map<string, number>(); // move -> time of its next repeat
  let first = true;
  let primed: boolean[] = [];

  const pad = (): Gamepad | null => {
    if (typeof navigator === 'undefined' || !navigator.getGamepads) return null;
    for (const p of navigator.getGamepads()) if (p?.connected && p.mapping === 'standard') return p;
    return null;
  };

  const frame = (now: number) => {
    if (stopped) return;
    raf = requestAnimationFrame(frame);
    const p = pad();
    if (!p) return;
    const down = (i: number) => !!p.buttons[i]?.pressed;
    if (first) {
      primed = p.buttons.map((b) => !!b.pressed);
      first = false;
    }
    const wasPrimed = (i: number) => {
      if (primed[i] && down(i)) return true;
      primed[i] = false;
      return false;
    };
    const ax = p.axes[0] ?? 0;
    const ay = p.axes[1] ?? 0;
    const active: Record<string, boolean> = {
      up: (down(PAD.UP) && !wasPrimed(PAD.UP)) || ay < -STICK_ON,
      down: (down(PAD.DOWN) && !wasPrimed(PAD.DOWN)) || ay > STICK_ON,
      left: (down(PAD.LEFT) && !wasPrimed(PAD.LEFT)) || ax < -STICK_ON,
      right: (down(PAD.RIGHT) && !wasPrimed(PAD.RIGHT)) || ax > STICK_ON,
      confirm: down(PAD.A) && !wasPrimed(PAD.A),
      back: down(PAD.B) && !wasPrimed(PAD.B),
      menu: down(PAD.START) && !wasPrimed(PAD.START)
    };
    for (const move of Object.keys(active) as NavMove[]) {
      if (!active[move]) {
        held.delete(move);
        continue;
      }
      const due = held.get(move);
      if (due === undefined) {
        held.set(move, now + FIRST_REPEAT_MS);
        onMove(move);
      } else if (now >= due && move !== 'confirm' && move !== 'back' && move !== 'menu') {
        held.set(move, now + REPEAT_MS);
        onMove(move);
      }
    }
  };
  raf = requestAnimationFrame(frame);
  return () => {
    stopped = true;
    cancelAnimationFrame(raf);
  };
}
