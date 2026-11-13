// What a touch surface may do to the input layer. The input controller implements this, so
// touch input reaches the simulation only as the same ActionFrame that keyboard and gamepad
// make. No DOM access.

import type { Action } from '../bindings';

export interface TouchSink {
  /** Holds `action` down for `source` (such as `Touch:light`) until `release`. */
  press(action: Action, source: string): void;
  release(action: Action, source: string): void;
  /** A tap that is over before the next tick still counts. */
  pulse(action: Action): void;
  /** Virtual stick: x is right and y is forward, each in [-1, 1]. */
  setStick(x: number, y: number): void;
  /** Camera drag in CSS pixels. Positive x turns right; positive y looks down. */
  look(dx: number, dy: number): void;
}
