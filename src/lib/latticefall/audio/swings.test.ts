// Tests for the swing cues. Run with `bun test`.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { SwingCues, SWING_LEAD_FRAMES, type SwingActor } from './swings';

const TIMING: Record<number, { strikeStart?: number }> = { 1: { strikeStart: 10 }, 2: {} };
const resolve = (id: number) => TIMING[id];
const actor = (moveId: number, moveFrame: number, flags = 0): SwingActor => ({ id: 7, kind: 0, flags, moveId, moveFrame });

describe('SwingCues', () => {
  test('fires the swing once, a little before the hitbox opens', () => {
    const cues = new SwingCues();
    const fired: number[] = [];
    for (let f = 0; f < 27; f++) {
      for (const c of cues.step([actor(1, f)], resolve)) if (c.what === 'swing') fired.push(f);
    }
    assert.deepEqual(fired, [10 - SWING_LEAD_FRAMES]);
  });

  test('reports the start of a move once', () => {
    const cues = new SwingCues();
    const starts = [0, 1, 2, 3].flatMap((f) => cues.step([actor(1, f)], resolve)).filter((c) => c.what === 'start');
    assert.equal(starts.length, 1);
    assert.equal(starts[0].windup, 10);
  });

  test('fires again when the same move starts again', () => {
    const cues = new SwingCues();
    let swings = 0;
    for (const f of [0, 4, 8, 12, 20, 0, 4, 8, 12]) swings += cues.step([actor(1, f)], resolve).filter((c) => c.what === 'swing').length;
    assert.equal(swings, 2);
  });

  test('stays quiet for a move without a hitbox, a dead actor, and a move seen late', () => {
    const cues = new SwingCues();
    assert.deepEqual(cues.step([actor(2, 5)], resolve), []);
    assert.deepEqual(cues.step([actor(1, 9, 1)], resolve), []);
    const late = new SwingCues().step([actor(1, 20)], resolve);
    assert.equal(late.filter((c) => c.what === 'swing').length, 0);
  });

  test('still fires when a tick skips the lead frame', () => {
    const cues = new SwingCues();
    cues.step([actor(1, 6)], resolve);
    assert.equal(cues.step([actor(1, 9)], resolve).filter((c) => c.what === 'swing').length, 1);
  });
});
