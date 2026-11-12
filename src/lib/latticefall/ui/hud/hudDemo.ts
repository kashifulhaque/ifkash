// A fixed HUD state with everything showing, for `&hud-demo=1`: lets the HUD be checked in
// both themes and at every scale without staging a fight. Not used by the game itself.

import type { HudState } from './hudState';

export function demoHud(): HudState {
  return {
    hp: { value: 310, max: 420 },
    fp: { value: 60, max: 90 },
    stamina: { value: 150, max: 160 },
    glimmer: 12480,
    vigil: { value: 3, max: 4 },
    lucid: { value: 1, max: 2 },
    right: { num: 1, name: 'Straight Sword', count: null, flask: false },
    left: { num: 2, name: 'Oak Buckler', count: null, flask: false },
    spell: { num: 0, name: '', count: null, flask: false },
    item: { num: 3, name: 'Vigil Flask', count: 3, flask: true },
    inCombat: true,
    prompt: 1,
    boss: { name: 'The Hollow Sentry', frac: 0.62, phase: 2 },
    status: [0.55, 0, 0.2, 0, 0.8, 0],
    dead: false
  };
}

export const DEMO_NOTICES = ['Lantern lit', '+340 Glimmer'];
