/**
 * Snake RL — browser port of the trained dueling Double DQN agent.
 *
 * A faithful TypeScript port of the Python experiment in the `snake-rl` repo:
 * `snake_env.py` (environment + rules) and `state.py` (28-feature egocentric
 * encoding) with the forward pass from `model.py` (DuelingQNet) and the
 * weights from the trained `runs/v2/best.pth` checkpoint.
 *
 * The network is an MLP: 28 inputs → 256 ReLU → 256 ReLU, then separate
 * value/advantage heads summed dueling-style. Weights are stored in
 * `snake-dqn-weights.json` (flattened row-major) and reshaped at load time so
 * the shapes match the PyTorch state dict.
 */

import weights from './snake-dqn-weights.json';

export const BLOCK_SIZE = 20;
export const BOARD_W = 32; // 640 / BLOCK_SIZE
export const BOARD_H = 24; // 480 / BLOCK_SIZE
const GRID_W = BOARD_W;
const GRID_H = BOARD_H;

export type Direction = 'up' | 'down' | 'left' | 'right';
export type Action = [number, number, number]; // [straight, right, left]

const CLOCKWISE: Direction[] = ['up', 'right', 'down', 'left'];
// heading-relative moves: 0 = straight, +1 = right, -1 = left
const MOVES = [0, 1, -1] as const;

/** Flattened row-major weights for the dueling network. */
export type SnakeWeights = {
  'body.0.weight': number[];
  'body.0.bias': number[];
  'body.2.weight': number[];
  'body.2.bias': number[];
  'value.weight': number[];
  'value.bias': number[];
  'advantage.weight': number[];
  'advantage.bias': number[];
};

const W = weights as SnakeWeights;

function matvec(w: number[], bias: number[], input: number[], outCols: number): number[] {
  const rows = bias.length;
  const out = new Array<number>(rows);
  for (let r = 0; r < rows; r++) {
    let acc = bias[r];
    for (let c = 0; c < outCols; c++) acc += w[r * outCols + c] * input[c];
    out[r] = acc;
  }
  return out;
}

function relu(v: number[]): number[] {
  for (let i = 0; i < v.length; i++) if (v[i] < 0) v[i] = 0;
  return v;
}

/**
 * Dueling forward pass: body(28→256→256) then V(s) + A(s,a) − mean A(s,a).
 * Matches `DuelingQNet.forward` in `model.py` exactly.
 */
function predict(state: number[]): number[] {
  const h1 = relu(matvec(W['body.0.weight'], W['body.0.bias'], state, 28));
  const h2 = relu(matvec(W['body.2.weight'], W['body.2.bias'], h1, 256));
  const v = matvec(W['value.weight'], W['value.bias'], h2, 256)[0];
  const a = matvec(W['advantage.weight'], W['advantage.bias'], h2, 256);
  const aMean = (a[0] + a[1] + a[2]) / 3;
  return [v + a[0] - aMean, v + a[1] - aMean, v + a[2] - aMean];
}

export type Point = { x: number; y: number };

export interface SnakeOptions {
  w?: number;
  h?: number;
  seed?: number;
}

export class SnakeGame {
  readonly w: number;
  readonly h: number;
  direction: Direction = 'up';
  head: Point = { x: 0, y: 0 };
  snake: Point[] = [];
  score = 0;
  food: Point = { x: 0, y: 0 };
  frameIteration = 0;
  stepsSinceFood = 0;

  private rng: () => number;

  constructor(opts: SnakeOptions = {}) {
    this.w = opts.w ?? 640;
    this.h = opts.h ?? 480;
    this.rng = seededRng(opts.seed ?? Math.floor(Math.random() * 2 ** 31));
    this.reset();
  }

  reset(): void {
    this.direction = 'right';
    this.head = { x: this.w / 2, y: this.h / 2 };
    this.snake = [
      this.head,
      { x: this.head.x - BLOCK_SIZE, y: this.head.y },
      { x: this.head.x - 2 * BLOCK_SIZE, y: this.head.y }
    ];
    this.score = 0;
    this.frameIteration = 0;
    this.stepsSinceFood = 0;
    this._placeFood();
  }

  /**
   * One env step, mirroring `snake_env.py::step` exactly: turn, compute the
   * new head, death-check (tail-vacate rule included), idle-cap truncation,
   * then move. Returns (reward, done, score).
   */
  playStep(action: Action): { reward: number; done: boolean; score: number } {
    this.frameIteration += 1;
    this.stepsSinceFood += 1;

    // action is relative to current heading: [straight, right turn, left turn]
    const idx = CLOCKWISE.indexOf(this.direction);
    let newIdx = idx;
    if (action[0] === 1) newIdx = idx;
    else if (action[1] === 1) newIdx = (idx + 1) % 4;
    else newIdx = (idx - 1 + 4) % 4;
    this.direction = CLOCKWISE[newIdx];

    const d = DELTA[this.direction];
    const newHead = { x: this.head.x + d.x * BLOCK_SIZE, y: this.head.y + d.y * BLOCK_SIZE };

    if (this.blocked(newHead)) {
      return { reward: -10, done: true, score: this.score };
    }

    // Frame cap (idle truncation) stops the agent looping forever once the
    // snake gets long — counted from the last food, like the Python env.
    if (this.stepsSinceFood > 100 * this.snake.length) {
      return { reward: 0, done: true, score: this.score };
    }

    this.head = newHead;
    this.snake.unshift({ ...this.head });

    if (this.head.x === this.food.x && this.head.y === this.food.y) {
      this.score += 1;
      this.stepsSinceFood = 0;
      this._placeFood();
      return { reward: 10, done: false, score: this.score };
    }

    this.snake.pop();
    return { reward: 0, done: false, score: this.score };
  }

  isCollision(pt: Point | null = null): boolean {
    const p = pt ?? this.head;
    if (p.x > this.w - BLOCK_SIZE || p.x < 0 || p.y > this.h - BLOCK_SIZE || p.y < 0) {
      return true;
    }
    return this.snake.slice(1).some((s) => s.x === p.x && s.y === p.y);
  }

  /**
   * True if `p` is off the board or on any part of the snake (head included) —
   * matches `snake_env.py::is_collision`.
   */
  occupied(p: Point): boolean {
    if (p.x < 0 || p.y < 0 || p.x > this.w - BLOCK_SIZE || p.y > this.h - BLOCK_SIZE) return true;
    return this.snake.some((s) => s.x === p.x && s.y === p.y);
  }

  /**
   * True if moving into `p` on this tick would kill the snake. Moving into
   * the current tail cell is legal — it vacates as the snake advances —
   * unless the snake is about to eat, which keeps the tail put.
   */
  blocked(pt: Point): boolean {
    if (this.occupied(pt)) {
      const tail = this.snake[this.snake.length - 1];
      const isTail = pt.x === tail.x && pt.y === tail.y;
      const eating = pt.x === this.food.x && pt.y === this.food.y;
      return !(isTail && !eating);
    }
    return false;
  }

  private _placeFood(): void {
    const cols = Math.floor(this.w / BLOCK_SIZE);
    const rows = Math.floor(this.h / BLOCK_SIZE);
    for (;;) {
      const x = Math.floor(this.rng() * cols) * BLOCK_SIZE;
      const y = Math.floor(this.rng() * rows) * BLOCK_SIZE;
      if (!this.snake.some((s) => s.x === x && s.y === y)) {
        this.food = { x, y };
        return;
      }
    }
  }
}

// --------------------------------------------------------------------------
// 28-feature egocentric state encoding — a line-for-line port of `state.py`.
// --------------------------------------------------------------------------

const DELTA: Record<Direction, Point> = {
  right: { x: 1, y: 0 },
  down: { x: 0, y: 1 },
  left: { x: -1, y: 0 },
  up: { x: 0, y: -1 }
};

/** Unit vectors for 'forward' and 'right' given the current heading. */
function rot(direction: Direction): [Point, Point] {
  const f = DELTA[direction];
  const r = DELTA[CLOCKWISE[(CLOCKWISE.indexOf(direction) + 1) % 4]];
  return [f, r];
}

/** Free cells ahead in direction `d` before hitting something, normalised. */
function ray(game: SnakeGame, start: Point, d: Point, limit: number): number {
  let x = start.x;
  let y = start.y;
  let n = 0;
  while (n < limit) {
    x += d.x * BLOCK_SIZE;
    y += d.y * BLOCK_SIZE;
    if (game.occupied({ x, y })) break;
    n += 1;
  }
  return n / limit;
}

/**
 * Flood-fill reachable free cells from `start`, stopping at `cap`. The tail
 * cell is treated as free because it vacates as the snake moves.
 */
function freeSpace(game: SnakeGame, start: Point, cap: number): number {
  const tail = game.snake[game.snake.length - 1];
  const isTail = (p: Point) => p.x === tail.x && p.y === tail.y;
  if (game.occupied(start) && !isTail(start)) return 0;

  const seen = new Set<string>([`${start.x},${start.y}`]);
  const stack: Point[] = [{ ...start }];
  let count = 0;
  while (stack.length) {
    const p = stack.pop()!;
    count += 1;
    if (count >= cap) return cap;
    for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as const) {
      const np = { x: p.x + dx * BLOCK_SIZE, y: p.y + dy * BLOCK_SIZE };
      const key = `${np.x},${np.y}`;
      if (seen.has(key)) continue;
      if (game.occupied(np) && !isTail(np)) continue;
      seen.add(key);
      stack.push(np);
    }
  }
  return count;
}

/**
 * The 28-dim state vector used at training time (see `state.py::get_state`).
 * Danger, rays and free space per candidate move; absolute heading; food and
 * tail in egocentric (forward/right) frame; board occupancy.
 */
export function getState(game: SnakeGame): number[] {
  const head = game.head;
  const [fwd, right] = rot(game.direction);
  const nSnake = game.snake.length;
  const diag = GRID_W + GRID_H;
  const limit = Math.max(GRID_W, GRID_H);
  // a pocket bigger than the snake is escapable, so exact size stops mattering
  const cap = nSnake + 1;

  const dirs: Point[] = [];
  for (const off of MOVES) {
    dirs.push(DELTA[CLOCKWISE[(CLOCKWISE.indexOf(game.direction) + off + 4) % 4]]);
  }

  const danger1: number[] = [];
  const danger2: number[] = [];
  const rays: number[] = [];
  const space: number[] = [];
  for (const d of dirs) {
    const p1 = { x: head.x + d.x * BLOCK_SIZE, y: head.y + d.y * BLOCK_SIZE };
    const blocked1 = game.blocked(p1);
    danger1.push(blocked1 ? 1 : 0);
    danger2.push(blocked1 || game.occupied({ x: head.x + 2 * d.x * BLOCK_SIZE, y: head.y + 2 * d.y * BLOCK_SIZE }) ? 1 : 0);
    rays.push(ray(game, head, d, limit));
    // how much room is left after committing to this move -- the signal
    // that distinguishes "turn into a dead end" from "turn into open board"
    space.push(blocked1 ? 0 : freeSpace(game, p1, cap) / cap);
  }

  // Python's Direction enum: RIGHT=0, DOWN=1, LEFT=2, UP=3 (see state.py)
  const HEADING_IDX: Record<Direction, number> = { right: 0, down: 1, left: 2, up: 3 };
  const heading = [0, 0, 0, 0];
  heading[HEADING_IDX[game.direction]] = 1;

  function egocentric(target: Point): [number, number] {
    const vx = target.x - head.x;
    const vy = target.y - head.y;
    const ahead = (vx * fwd.x + vy * fwd.y) / BLOCK_SIZE;
    const side = (vx * right.x + vy * right.y) / BLOCK_SIZE;
    return [ahead, side];
  }

  const [fAhead, fSide] = egocentric(game.food);
  const tail = game.snake[game.snake.length - 1];
  const [tAhead, tSide] = egocentric(tail);

  return [
    ...danger1, // 3 immediate death
    ...danger2, // 3 death in two steps
    ...rays, // 3 clear distance ahead
    ...space, // 3 reachable room per move
    ...heading, // 4 absolute heading
    fAhead > 0 ? 1 : 0, fAhead < 0 ? 1 : 0, // 2 food quadrant, ego frame
    fSide > 0 ? 1 : 0, fSide < 0 ? 1 : 0, // 2
    fAhead / limit, fSide / limit, // 2 signed food offset
    (Math.abs(fAhead) + Math.abs(fSide)) / diag, // 1 food distance
    tAhead > 0 ? 1 : 0, tSide > 0 ? 1 : 0, // 2 tail quadrant
    tAhead / limit, tSide / limit, // 2 signed tail offset
    nSnake / (GRID_W * GRID_H) // 1 how full the board is
  ];
}

/** Greedy action (ε = 0, matching `play.py` evaluation mode). */
export function getAction(game: SnakeGame): Action {
  const q = predict(getState(game));
  let idx = 0;
  if (q[1] > q[idx]) idx = 1;
  if (q[2] > q[idx]) idx = 2;
  const move: Action = [0, 0, 0];
  move[idx] = 1;
  return move;
}

/** Deterministic LCG so every game in a run is reproducible. */
function seededRng(seed: number): () => number {
  let s = seed >>> 0;
  return () => {
    s = (s * 1664525 + 1013904223) >>> 0;
    return s / 0x100000000;
  };
}
