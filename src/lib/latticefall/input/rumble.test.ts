// Tests for controller rumble. Run with `bun test`.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { RUMBLE, rumble, rumbleForHit, rumbleParams } from './rumble';

test('rumbleForHit picks the pulse from the player\'s side of the hit', () => {
  const hit = (attacker: number, target: number, result = 0, flags = 0) => ({ attacker, target, result, flags });
  assert.equal(rumbleForHit(hit(1, 2), 1), 'hitLanded');
  assert.equal(rumbleForHit(hit(1, 2, 1), 1), 'hitGuarded');
  assert.equal(rumbleForHit(hit(2, 1), 1), 'hitTaken');
  assert.equal(rumbleForHit(hit(2, 1, 1), 1), 'blocked');
  assert.equal(rumbleForHit(hit(2, 1, 2), 1), 'guardBroken');
  assert.equal(rumbleForHit(hit(2, 1, 0, 2), 1), 'death');
  assert.equal(rumbleForHit(hit(2, 3), 1), null);
});

test('being hit is stronger than landing a hit, and a guard break stronger still', () => {
  assert.ok(RUMBLE.hitTaken.strong > RUMBLE.hitLanded.strong);
  assert.ok(RUMBLE.guardBroken.strong > RUMBLE.hitTaken.strong);
});

test('rumbleParams clamps to the ranges the Gamepad API accepts', () => {
  assert.deepEqual(rumbleParams({ duration: 99999, strong: 2, weak: -1 }), { startDelay: 0, duration: 5000, strongMagnitude: 1, weakMagnitude: 0 });
});

test('rumble is safe without a pad or an actuator, and plays dual-rumble with one', () => {
  assert.equal(rumble('hitLanded', null), false);
  assert.equal(rumble('hitLanded', {}), false);
  assert.equal(rumble('hitLanded', { vibrationActuator: {} }), false);
  const calls: Array<[string, Record<string, number>]> = [];
  const pad = { vibrationActuator: { playEffect: (type: string, params: Record<string, number>) => void calls.push([type, params]) } };
  assert.equal(rumble('hitTaken', pad), true);
  assert.equal(calls[0][0], 'dual-rumble');
  assert.equal(calls[0][1].strongMagnitude, RUMBLE.hitTaken.strong);
  // A throwing actuator is swallowed.
  assert.equal(rumble('hitTaken', { vibrationActuator: { playEffect: () => { throw new Error('no'); } } }), false);
});
