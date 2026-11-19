// Tests for the move-to-clip timing maths. Run with `bun test`.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import {
  anticipationFrame,
  fitMove,
  hitFrame,
  locateSpec,
  MAX_RECOVERY_RATE,
  MAX_REACT_RATE,
  MAX_WINDUP_RATE,
  MIN_RECOVERY_RATE,
  MIN_WINDUP_RATE,
  moveClipAt,
  moveClipFrame,
  reactClipTime,
  SETTLE_FRAMES,
  specLength,
  type ClipSpec,
  type MoveAnimInfo
} from './moveClips';

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

describe('fitMove', () => {
  // A heavy attack: 42 frames, hitbox on frames 18 to 21, so the contact lands on 19.5.
  const HEAVY: MoveAnimInfo = { anim: 'sword_overhead', totalFrames: 42, strikeStart: 18, recoveryStart: 22 };
  const near = (a: number, b: number) => assert.ok(Math.abs(a - b) < 1e-6, `${a} is not ${b}`);

  test('a short wind-up plays at its authored rate and holds the anticipation pose', () => {
    // 12 clip frames over 19.5 sim frames would be 0.62x: slow motion.
    assert.ok(12 / 19.5 < MIN_WINDUP_RATE);
    const fit = fitMove(31, 12, HEAVY);
    assert.equal(fit.windup, 1);
    near(fit.hold, 7.5);
    assert.equal(fit.anticipation, anticipationFrame(12));
    // Authored rate up to the pose, the hold, then the swing at its authored rate.
    near(moveClipAt(31, 12, HEAVY, 4).frame, 4);
    near(moveClipAt(31, 12, HEAVY, 8).frame, 8);
    near(moveClipAt(31, 12, HEAVY, 12).frame, 8);
    near(moveClipAt(31, 12, HEAVY, 15.5).frame, 8);
    near(moveClipAt(31, 12, HEAVY, 17.5).frame, 10);
    // The contact still lands in the middle of the hitbox window.
    near(moveClipAt(31, 12, HEAVY, 19.5).frame, 12);
  });

  test('a long wind-up starts part way in instead of rushing', () => {
    // 72 clip frames over 11 sim frames would be 6.5x.
    const fit = fitMove(106, 72, LIGHT);
    assert.equal(fit.windup, MAX_WINDUP_RATE);
    near(fit.start, 72 - 11 * MAX_WINDUP_RATE);
    near(moveClipAt(106, 72, LIGHT, 0).frame, fit.start);
    near(moveClipAt(106, 72, LIGHT, 11).frame, 72);
  });

  test('a short recovery never plays in slow motion: it ends early and settles into the idle', () => {
    // 25 clip frames of recovery over 42.5 sim frames would be 0.59x.
    const BOSS: MoveAnimInfo = { anim: 'sentry_overhead', totalFrames: 74, strikeStart: 30, recoveryStart: 34 };
    const fit = fitMove(65, 40, BOSS);
    assert.equal(fit.recovery, MIN_RECOVERY_RATE);
    near(fit.clipEnd, 31.5 + 25 / MIN_RECOVERY_RATE);
    assert.equal(moveClipAt(65, 40, BOSS, fit.clipEnd - 1).settle, 0);
    near(moveClipAt(65, 40, BOSS, fit.clipEnd + SETTLE_FRAMES / 2).settle, 0.5);
    assert.deepEqual(moveClipAt(65, 40, BOSS, 74), { frame: 65, settle: 1 });
  });

  test('keeps the contact on the hit when the frame data changes', () => {
    for (const strikeStart of [8, 10, 14, 20, 30]) {
      const info: MoveAnimInfo = { anim: 'x', totalFrames: strikeStart + 30, strikeStart, recoveryStart: strikeStart + 4 };
      for (const [length, contact] of [[45, 18], [27, 18], [56, 29], [30, 12]]) {
        near(moveClipAt(length, contact, info, hitFrame(info)!).frame, contact);
        let last = -1;
        for (let f = 0; f <= info.totalFrames; f += 0.5) {
          const c = moveClipAt(length, contact, info, f).frame;
          assert.ok(c >= last - 1e-9 && c <= length);
          last = c;
        }
      }
    }
  });

  test('a move without a hitbox is scaled as one piece, within the rate bounds', () => {
    const flask: MoveAnimInfo = { anim: 'flask_drink', totalFrames: 36 };
    near(fitMove(54, 0, flask).recovery, 1.5);
    // A 266-frame clip is cut, not played at 7x.
    assert.equal(fitMove(266, 0, flask).recovery, MAX_RECOVERY_RATE);
  });
});

describe('reactClipTime', () => {
  const near = (a: number, b: number) => assert.ok(Math.abs(a - b) < 1e-6, `${a} is not ${b}`);

  test('plays the whole clip over a lockout it can span', () => {
    // A 1 s clip over a 0.8 s stagger: 1.25x.
    near(reactClipTime(1, 0.8, 0.4).time, 0.5);
    assert.deepEqual(reactClipTime(1, 0.8, 0.8), { time: 1, tail: false });
  });

  test('a short lockout shows the flinch, then the end of the clip', () => {
    // A 2.2 s clip over a 0.4 s stagger used to show its first 18 percent.
    const d = 2.2;
    const head = reactClipTime(d, 0.4, 0.2);
    near(head.time, 0.2 * MAX_REACT_RATE);
    assert.equal(head.tail, false);
    const tail = reactClipTime(d, 0.4, 0.3);
    assert.equal(tail.tail, true);
    near(tail.time, d - 0.1 * MAX_REACT_RATE);
    near(reactClipTime(d, 0.4, 0.4).time, d);
  });

  test('plays at the authored rate when the lockout length is unknown', () => {
    assert.deepEqual(reactClipTime(1, 0, 0.3), { time: 0.3, tail: false });
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
    assert.deepEqual(locateSpec(spec, 8, frames, 30), { clip: 'A', time: 8 / 30, seg: 0 });
    assert.deepEqual(locateSpec(spec, 20, frames, 30), { clip: 'A_Rec', time: 7 / 30, seg: 1 });
    assert.deepEqual(locateSpec(spec, 99, frames, 30), { clip: 'A_Rec', time: 29 / 30, seg: 1 });
    assert.deepEqual(locateSpec({ segs: [{ clip: 'Long', from: 20, to: 48 }], contact: 15 }, 15, frames, 30), {
      clip: 'Long',
      time: 35 / 30,
      seg: 0
    });
  });
});
