// Tests for the art character clip mapping. Run with `bun test`.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { artGuard, artSpecFor, MIXAMO_BONE_MAP, type ArtClipInfo } from './artClips';
import { BONE_MAP } from './retarget';

const clip = (contact: number | null): ArtClipInfo => ({ frames: 30, duration: 1, loop: false, contact, source: 'x.fbx' });
const CLIPS = { idle: clip(null), slash_a: clip(13), heavy: clip(22), block: clip(null), roll: clip(null) };

describe('artSpecFor', () => {
  test('plays the listed clip with its contact frame', () => {
    assert.deepEqual(artSpecFor('sword_overhead', true, CLIPS), { segs: [{ clip: 'heavy' }], contact: 22 });
  });

  test('falls back down the list when a clip is missing', () => {
    assert.deepEqual(artSpecFor('sword_slash_l', true, CLIPS), { segs: [{ clip: 'slash_a' }], contact: 13 });
  });

  test('plays a slash for an unlisted attack and nothing for an unlisted non-attack', () => {
    assert.equal(artSpecFor('halberd_sweep_r', true, CLIPS)?.segs[0].clip, 'slash_a');
    assert.equal(artSpecFor('interact', false, CLIPS), undefined);
  });

  test('scales a clip without a contact frame to the move', () => {
    assert.equal(artSpecFor('roll', false, CLIPS)?.contact, 0);
  });

  test('takes a raw clip id, for the animation preview', () => {
    assert.equal(artSpecFor('block', false, CLIPS)?.segs[0].clip, 'block');
  });
});

describe('art rig', () => {
  test('the Mixamo map names every bone the renderer looks up, in the same order', () => {
    assert.deepEqual(
      MIXAMO_BONE_MAP.map(([, logical]) => logical),
      BONE_MAP.map(([, target]) => target)
    );
  });

  test('the guard prefers the looping block idle', () => {
    assert.deepEqual(artGuard((c) => c === 'block' || c === 'block_idle'), { clip: 'block_idle' });
    assert.deepEqual(artGuard((c) => c === 'block'), { clip: 'block', frame: 8 });
    assert.equal(artGuard(() => false), undefined);
  });
});
