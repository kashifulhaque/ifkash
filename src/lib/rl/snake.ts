/**
 * Snake RL — browser port of the trained DQN agent.
 *
 * A faithful TypeScript port of the Python experiment in the `snake-rl` repo:
 * `snake_env.py` (environment + rules) and `model.py` (forward pass only, no
 * training) with the weights from the trained `model.pth` checkpoint.
 *
 * The network is a small MLP: 11 inputs → 256 ReLU → 3 outputs. Weights are
 * stored in `snake-dqn-weights.json` (flattened row-major) and reshaped at
 * load time so the shapes match the PyTorch state dict.
 */

import weights from './snake-dqn-weights.json';

export const BLOCK_SIZE = 20;
export const BOARD_W = 32; // 640 / BLOCK_SIZE
export const BOARD_H = 24; // 480 / BLOCK_SIZE

const MAX_MEMORY = 100_000;
const BATCH_SIZE = 1000;
const LR = 0.001;

export type Direction = 'up' | 'down' | 'left' | 'right';
export type Action = [number, number, number]; // [straight, right, left]

const DIRECTIONS = ['up', 'right', 'down', 'left'] as const;
const CLOCKWISE: Direction[] = ['up', 'right', 'down', 'left'];

/** Flattened row-major weights for the two Linear layers (input → 256 → 3). */
export type SnakeWeights = {
  'net.0.weight': number[];
  'net.0.bias': number[];
  'net.2.weight': number[];
  'net.2.bias': number[];
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

/** Forward pass through Linear(11→256) → ReLU → Linear(256→3). */
function predict(state: number[]): number[] {
  const h = relu(matvec(W['net.0.weight'], W['net.0.bias'], state, 11));
  return matvec(W['net.2.weight'], W['net.2.bias'], h, 256);
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
    this._placeFood();
  }

  playStep(action: Action): { reward: number; done: boolean; score: number } {
    this.frameIteration += 1;
    this._move(action);
    this.snake.unshift({ ...this.head });

    let reward = 0;
    let done = false;
    // Frame cap stops the agent looping forever once the snake gets long.
    if (this.isCollision() || this.frameIteration > 100 * this.snake.length) {
      done = true;
      reward = -10;
      return { reward, done, score: this.score };
    }

    if (this.head.x === this.food.x && this.head.y === this.food.y) {
      this.score += 1;
      reward = 10;
      this._placeFood();
    } else {
      this.snake.pop();
    }

    return { reward, done, score: this.score };
  }

  isCollision(pt: Point | null = null): boolean {
    const p = pt ?? this.head;
    if (p.x > this.w - BLOCK_SIZE || p.x < 0 || p.y > this.h - BLOCK_SIZE || p.y < 0) {
      return true;
    }
    return this.snake.slice(1).some((s) => s.x === p.x && s.y === p.y);
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

  private _move(action: Action): void {
    // action is relative to current heading: [straight, right turn, left turn]
    const idx = CLOCKWISE.indexOf(this.direction);
    let newIdx = idx;
    if (action[0] === 1) newIdx = idx;
    else if (action[1] === 1) newIdx = (idx + 1) % 4;
    else newIdx = (idx - 1 + 4) % 4;
    this.direction = CLOCKWISE[newIdx];

    let x = this.head.x;
    let y = this.head.y;
    switch (this.direction) {
      case 'right':
        x += BLOCK_SIZE;
        break;
      case 'left':
        x -= BLOCK_SIZE;
        break;
      case 'down':
        y += BLOCK_SIZE;
        break;
      case 'up':
        y -= BLOCK_SIZE;
        break;
    }
    this.head = { x, y };
  }
}

/** The 11-dim state vector used at training time (see `train.py::get_state`). */
export function getState(game: SnakeGame): number[] {
  const head = game.head;
  const pointL = { x: head.x - BLOCK_SIZE, y: head.y };
  const pointR = { x: head.x + BLOCK_SIZE, y: head.y };
  const pointU = { x: head.x, y: head.y - BLOCK_SIZE };
  const pointD = { x: head.x, y: head.y + BLOCK_SIZE };

  const dirL = game.direction === 'left';
  const dirR = game.direction === 'right';
  const dirU = game.direction === 'up';
  const dirD = game.direction === 'down';

  return [
    (dirR && game.isCollision(pointR)) || (dirL && game.isCollision(pointL)) || (dirU && game.isCollision(pointU)) || (dirD && game.isCollision(pointD)) ? 1 : 0,
    (dirU && game.isCollision(pointR)) || (dirD && game.isCollision(pointL)) || (dirL && game.isCollision(pointU)) || (dirR && game.isCollision(pointD)) ? 1 : 0,
    (dirD && game.isCollision(pointR)) || (dirU && game.isCollision(pointL)) || (dirR && game.isCollision(pointU)) || (dirL && game.isCollision(pointD)) ? 1 : 0,
    dirL ? 1 : 0,
    dirR ? 1 : 0,
    dirU ? 1 : 0,
    dirD ? 1 : 0,
    game.food.x < game.head.x ? 1 : 0,
    game.food.x > game.head.x ? 1 : 0,
    game.food.y < game.head.y ? 1 : 0,
    game.food.y > game.head.y ? 1 : 0
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

// Keep the training constants referenced so the port stays honest to the original.
void MAX_MEMORY;
void BATCH_SIZE;
void LR;
void DIRECTIONS;
