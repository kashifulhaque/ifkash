// The touch buttons: which actions they send and where they sit. No DOM access.
// Docs: docs/game/07-presentation.md, "Mobile".

import type { ButtonAction } from '../bindings';

export interface TouchButton {
  action: ButtonAction;
  label: string;
  /** Accessible name. */
  title: string;
  /** Column and row in the right-hand cluster, counted from the bottom right. */
  col: number;
  row: number;
}

/** Every button holds its action while the finger is down; the composer latches quick taps. */
export const TOUCH_BUTTONS: readonly TouchButton[] = [
  { action: 'guard', label: 'Guard', title: 'Guard', col: 0, row: 0 },
  { action: 'dodge', label: 'Dodge', title: 'Dodge', col: 1, row: 0 },
  { action: 'light', label: 'Light', title: 'Light attack', col: 0, row: 1 },
  { action: 'heavy', label: 'Heavy', title: 'Heavy attack', col: 1, row: 1 },
  { action: 'art', label: 'Art', title: 'Art', col: 0, row: 2 },
  { action: 'useItem', label: 'Item', title: 'Use item', col: 1, row: 2 },
  { action: 'interact', label: 'Use', title: 'Interact', col: 0, row: 3 },
  { action: 'lock', label: 'Lock', title: 'Lock on', col: 1, row: 3 }
];
