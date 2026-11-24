// Tests for the art character clip mapping. Run with `bun test`.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import {
  artAssetBase,
  artGuard,
  artGuards,
  artSpecFor,
  moveRole,
  pickGuard,
  artTwoHand,
  enemyArt,
  ENEMY_ART,
  lanternGlow,
  LANTERN_PEAK,
  MIXAMO_BONE_MAP,
  PLAYER_ART,
  playerArt,
  PRIVATE_ART,
  type ArtClipInfo
} from './artClips';
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

  test('two-handed, light attacks take the two-handed slash and heavies the two-handed heavy', () => {
    const two = { ...CLIPS, twohand_slash: clip(16), twohand_heavy: clip(9) };
    assert.equal(artSpecFor('sword_slash_r', true, two, true)?.segs[0].clip, 'twohand_slash');
    assert.deepEqual(artSpecFor('sword_overhead', true, two, true), { segs: [{ clip: 'twohand_heavy' }], contact: 9 });
    // One-handed, or not an attack, nothing changes.
    assert.equal(artSpecFor('sword_slash_r', true, two, false)?.segs[0].clip, 'slash_a');
    assert.equal(artSpecFor('roll', false, two, true)?.segs[0].clip, 'roll');
  });

  test('reads a move role from its id', () => {
    assert.equal(moveRole('ss_light_2'), 'light_2');
    assert.equal(moveRole('gs_heavy_charged'), 'heavy_charged');
    assert.equal(moveRole('sh_bash_3'), 'light_3');
    assert.equal(moveRole(undefined), undefined);
  });

  test('an unlisted class plays the clips for its role', () => {
    const more = { ...CLIPS, slash_b: clip(18), chop: clip(12), leap: clip(20) };
    assert.equal(artSpecFor('dagger_stab_l', true, more, { moveId: 'da_light_2' })?.segs[0].clip, 'slash_b');
    assert.equal(artSpecFor('axe_chop', true, more, { moveId: 'ax_heavy' })?.segs[0].clip, 'chop');
    assert.equal(artSpecFor('axe_jump', true, more, { moveId: 'ax_jump_light' })?.segs[0].clip, 'leap');
  });

  test('two-handed, each role takes its own two-handed clip', () => {
    const two = { ...CLIPS, twohand_slash: clip(16), twohand_slash_b: clip(24), twohand_slam: clip(13), twohand_heavy: clip(9) };
    const pick = (anim: string, id: string) => artSpecFor(anim, true, two, { twoHanded: true, moveId: id })?.segs[0].clip;
    assert.equal(pick('greatsword_sweep_l', 'gs_light_2'), 'twohand_slash_b');
    assert.equal(pick('greatsword_overhead', 'gs_heavy'), 'twohand_slam');
    assert.equal(pick('greatsword_overhead_charged', 'gs_heavy_charged'), 'twohand_heavy');
    // A shield and fists keep their own strikes.
    assert.equal(pick('shield_bash_r', 'sh_bash_1'), 'slash_a');
  });

  test('a left-hand swing plays the mirrored clip, else the right-hand one', () => {
    const both = { ...CLIPS, slash_a_lh: clip(13) };
    assert.equal(artSpecFor('sword_slash_r', true, both, { left: true })?.segs[0].clip, 'slash_a_lh');
    assert.equal(artSpecFor('sword_overhead', true, CLIPS, { left: true })?.segs[0].clip, 'heavy');
    // Left-hand swings never take the two-handed clips.
    const two = { ...both, twohand_slash: clip(16) };
    assert.equal(artSpecFor('sword_slash_r', true, two, { left: true, twoHanded: true })?.segs[0].clip, 'slash_a_lh');
  });

  test('two-handed without two-handed clips keeps the one-handed ones', () => {
    assert.equal(artSpecFor('sword_overhead', true, CLIPS, true)?.segs[0].clip, 'heavy');
  });
});

describe('art clip stretches', () => {
  test('a chain of clips plays back to back and asks for a crossfade at each join', () => {
    const clips = { idle: clip(null), drink: clip(null), drink_end: clip(null) };
    assert.deepEqual(artSpecFor('flask_drink', false, clips), { segs: [{ clip: 'drink' }, { clip: 'drink_end' }], contact: 0, blend: true });
    // Without the second clip the first plays alone.
    assert.deepEqual(artSpecFor('flask_drink', false, { idle: clip(null), drink: clip(null) }), { segs: [{ clip: 'drink' }], contact: 0 });
  });

  test('the light chain plays three different clips', () => {
    const clips = { slash_a: clip(18), slash_b: clip(18), slash_c: clip(16) };
    const ids = ['sword_slash_r', 'sword_slash_l', 'sword_thrust'].map((a) => artSpecFor(a, true, clips)?.segs[0].clip);
    assert.deepEqual(ids, ['slash_a', 'slash_b', 'slash_c']);
  });

  test('the backstep has a stand-in instead of holding the idle', () => {
    assert.equal(artSpecFor('backstep', false, { idle: clip(null), run_back: clip(null) })?.segs[0].clip, 'run_back');
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

  test('the two-handed stance is the two-handed idle, when there is one', () => {
    assert.deepEqual(artTwoHand((c) => c === 'twohand_idle'), { clip: 'twohand_idle' });
    assert.equal(artTwoHand(() => false), undefined);
  });
});

describe('art assets', () => {
  test('private art comes from the dev server locally and from the API in production', () => {
    assert.equal(artAssetBase('paladin', true, 'https://ifkash.dev'), '/latticefall/private/');
    assert.equal(artAssetBase('paladin', false, 'https://ifkash.dev'), 'https://ifkash.dev/api/art/private/');
    assert.equal(artAssetBase('standin', false, 'https://ifkash.dev'), '/latticefall/characters/');
  });

  test('the player is PLAYER_ART unless &art= names another, or off for the Wayfarer', () => {
    assert.equal(playerArt(null), PLAYER_ART);
    assert.equal(playerArt('standin'), 'standin');
    assert.equal(playerArt('off'), null);
  });
});

describe('enemy art', () => {
  test('husks and B01 have art characters, and &enemyart=off turns them off', () => {
    assert.equal(enemyArt(7000, null), 'husk');
    assert.equal(enemyArt(9011, null), 'sentry');
    assert.equal(enemyArt(7010, null), undefined);
    assert.equal(enemyArt(7000, 'off'), undefined);
    for (const name of ENEMY_ART.values()) assert.ok(PRIVATE_ART.has(name), `${name} is private`);
  });

  test('every husk and B01 move maps to a clip with its contact frame', () => {
    const clips = { idle: clip(null), attack_a: clip(14), attack_b: clip(17), slam: clip(20), sweep: clip(15), bash: clip(9), combo_1: clip(12), combo_2: clip(11), combo_3: clip(19) };
    const want: Record<string, [string, number]> = {
      husk_swipe_1: ['attack_a', 14],
      husk_swipe_2: ['attack_b', 17],
      sentry_overhead: ['slam', 20],
      sentry_sweep: ['sweep', 15],
      sentry_bash: ['bash', 9],
      sentry_combo_1: ['combo_1', 12],
      sentry_combo_2: ['combo_2', 11],
      sentry_combo_3: ['combo_3', 19]
    };
    for (const [anim, [id, contact]] of Object.entries(want)) {
      assert.deepEqual(artSpecFor(anim, true, clips), { segs: [{ clip: id }], contact }, anim);
    }
  });

  test('the lantern swells through the wind-up, peaks at the strike, and settles', () => {
    const slam = { totalFrames: 74, strikeStart: 30, recoveryStart: 34 };
    assert.equal(lanternGlow(slam, 0), 1);
    assert.ok(lanternGlow(slam, 15) > 1 && lanternGlow(slam, 15) < lanternGlow(slam, 29));
    assert.equal(lanternGlow(slam, 30), LANTERN_PEAK);
    assert.equal(lanternGlow(slam, 60), 1);
    assert.equal(lanternGlow(undefined, 10), 1);
    assert.equal(lanternGlow({ totalFrames: 30 }, 10), 1);
  });
});

describe('pickGuard', () => {
  const all = artGuards((c) => ['block_idle', 'block_idle_lh', 'twohand_guard'].includes(c));

  test('a left shield raises the left arm, a right shield the right arm', () => {
    assert.equal(pickGuard(all, 'StraightSword', 'Shield', false)?.clip, 'block_idle');
    assert.equal(pickGuard(all, 'Shield', undefined, false)?.clip, 'block_idle_lh');
    assert.equal(pickGuard(all, 'Shield', 'Shield', false)?.clip, 'block_idle');
  });

  test('two-handing guards with the blade, or with the shield held in both hands', () => {
    assert.equal(pickGuard(all, 'Greatsword', 'Shield', true)?.clip, 'twohand_guard');
    assert.equal(pickGuard(all, 'Shield', undefined, true)?.clip, 'block_idle_lh');
  });

  test('falls back to the left-arm guard without the other clips', () => {
    const left = artGuards((c) => c === 'block_idle');
    assert.equal(pickGuard(left, 'Greatsword', undefined, true)?.clip, 'block_idle');
    assert.equal(pickGuard(left, 'Shield', undefined, false)?.clip, 'block_idle');
  });
});
