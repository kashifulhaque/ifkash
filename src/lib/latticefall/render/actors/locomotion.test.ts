// Tests for the enemy gait helpers. Run with `bun test`.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { gaitFacing, GAIT_MAX_RATE, pickGait, type LocoBands } from './locomotion';

// The husk's measured bands: a slow shamble and a fast run, far apart.
const HUSK: LocoBands = {
  walk: { metersPerCycle: 1.57, speed: 0.39 },
  run: { metersPerCycle: 2.48, speed: 3.1 },
  sprint: { metersPerCycle: 2.48, speed: 4.0 }
};

describe('pickGait', () => {
  test('idles when still', () => {
    assert.deepEqual(pickGait(0.05, HUSK, 'walk'), { gait: 'idle', rate: 1 });
  });

  test('plays one loop at the rate that matches the ground speed', () => {
    const chase = pickGait(2.8, HUSK, 'idle');
    assert.equal(chase.gait, 'run');
    assert.ok(Math.abs(chase.rate - 2.8 / 3.1) < 1e-9);
    assert.equal(pickGait(0.4, HUSK, 'idle').gait, 'walk');
  });

  test('keeps the current loop until the other fits clearly better', () => {
    // Geometric middle of the two speeds: either loop fits as well, so the current one stays.
    const mid = Math.sqrt(0.39 * 3.1);
    assert.equal(pickGait(mid, HUSK, 'walk').gait, 'walk');
    assert.equal(pickGait(mid, HUSK, 'run').gait, 'run');
  });

  test('caps the playback rate', () => {
    assert.equal(pickGait(20, HUSK, 'run').rate, GAIT_MAX_RATE);
  });
});

describe('gaitFacing', () => {
  const deg = (d: number) => (d * Math.PI) / 180;

  test('keeps the facing for forward travel and when slow', () => {
    assert.deepEqual(gaitFacing(0, deg(20), 2), { yaw: 0, reverse: false });
    assert.deepEqual(gaitFacing(0, deg(90), 0.1), { yaw: 0, reverse: false });
  });

  test('turns the body along a sideways path', () => {
    assert.deepEqual(gaitFacing(0, deg(90), 1.1), { yaw: deg(90), reverse: false });
  });

  test('keeps facing the target and plays backward when backing away', () => {
    const back = gaitFacing(0, deg(180), 1.1);
    assert.equal(back.reverse, true);
    assert.ok(Math.abs(back.yaw) < 1e-9);
  });
});
