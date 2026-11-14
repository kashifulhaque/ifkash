// Tests for the move-to-clip timing maths. Run with `bun test`.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { hitFrame, locateSpec, MAX_RECOVERY_RATE, moveClipFrame, specLength, type ClipSpec, type MoveAnimInfo } from './moveClips';

// The straight sword's first light attack: 27 frames, hitbox on frames 10 to 12.
const LIGHT: MoveAnimInfo = { anim: 'sword_slash_r', totalFrames: 27, strikeStart: 10, recoveryStart: 13 };

describe('moveClipFrame', () => {
  test('lands the clip contact in the middle of the hitbox window', () => {
    assert.equal(hitFrame(LIGHT), 11);
    assert.ok(Math.abs((moveClipFrame(46, 12, LIGHT, 11)) - (12)) < 1e-6);
  });

  test('starts at the first clip frame and never runs backward', () => {
    assert.equal(moveClipFrame(46, 12, LIGHT, 0), 0);
    let last = -1;
    for (let f = 0; f <= 27; f += 0.25) {
      const c = moveClipFrame(46, 12, LIGHT, f);
      assert.ok(c >= last);
      last = c;
    }
  });

  test('fits the recovery to the move, and caps its rate', () => {
    // 34 clip frames of recovery in 16 sim frames is 2.1x: under the cap, so it all plays.
    assert.ok(Math.abs((moveClipFrame(46, 12, LIGHT, 27)) - (46)) < 1e-6);
    // 60 frames in 16 would be 3.75x; the cap cuts the follow-through instead.
    assert.ok(Math.abs((moveClipFrame(72, 12, LIGHT, 27)) - (12 + 16 * MAX_RECOVERY_RATE)) < 1e-6);
    assert.ok(moveClipFrame(14, 8, LIGHT, 27) <= 14);
  });

  test('scales linearly without a hitbox', () => {
    const roll: MoveAnimInfo = { anim: 'roll', totalFrames: 24 };
    assert.ok(Math.abs((moveClipFrame(44, 0, roll, 12)) - (22)) < 1e-6);
    assert.ok(Math.abs((moveClipFrame(44, 0, roll, 30)) - (44)) < 1e-6);
  });
});

describe('clip specs', () => {
  const frames = (c: string) => ({ A: 14, A_Rec: 30, Long: 131 })[c] ?? 0;
  const spec: ClipSpec = { segs: [{ clip: 'A' }, { clip: 'A_Rec' }], contact: 8 };

  test('measures stretches end to end', () => {
    assert.equal(specLength(spec, frames), 13 + 29);
    assert.equal(specLength({ segs: [{ clip: 'Long', from: 20, to: 48 }], contact: 15 }, frames), 28);
  });

  test('locates a frame in the right stretch', () => {
    assert.deepEqual(locateSpec(spec, 8, frames, 30), { clip: 'A', time: 8 / 30 });
    assert.deepEqual(locateSpec(spec, 20, frames, 30), { clip: 'A_Rec', time: 7 / 30 });
    assert.deepEqual(locateSpec(spec, 99, frames, 30), { clip: 'A_Rec', time: 29 / 30 });
    assert.deepEqual(locateSpec({ segs: [{ clip: 'Long', from: 20, to: 48 }], contact: 15 }, 15, frames, 30), {
      clip: 'Long',
      time: 35 / 30
    });
  });
});
