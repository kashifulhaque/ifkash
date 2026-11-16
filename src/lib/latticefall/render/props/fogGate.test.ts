// Tests for the boss fog gate opening when its boss falls. Run with `bun test`.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { Group, Scene, type Object3D } from 'three';
import { PropsRenderer } from './index';

/** A scenario with one boss: B01, with its gate at z = 98. The map lists defeated bosses by gate. */
const ENTITIES = [
  {
    Boss: {
      name: 'b01',
      boss: 'b01_hollow_sentry',
      pos: [0, 0, 111],
      yaw: Math.PI,
      gate: { pos: [0, 0, 98], yaw: 0, half_width_m: 4, depth_m: 2 }
    }
  }
];

/** The gate's group: the props root holds one holder per prop, and the gate is the only one. */
function gateOf(scene: Scene): Object3D {
  const root = scene.getObjectByName('latticefall-props') as Group;
  const holder = root.children.find((c) => c.children.length === 1 && c.children[0].children.length === 2);
  assert.ok(holder, 'the fog gate is drawn');
  return holder.children[0];
}

describe('boss fog gate', () => {
  test('draws until the boss is defeated, then fades out and stops drawing', () => {
    const scene = new Scene();
    const props = new PropsRenderer(scene);
    props.build(ENTITIES as never);
    const gate = gateOf(scene);
    props.update(500);
    assert.equal(gate.visible, true);
    props.bossDefeated(0);
    props.update(500);
    assert.equal(gate.visible, true, 'the mist fades rather than vanishing');
    for (let i = 0; i < 10; i++) props.update(200);
    assert.equal(gate.visible, false);
    props.dispose();
  });

  test('never draws for a boss the save says is already defeated', () => {
    const scene = new Scene();
    const props = new PropsRenderer(scene);
    props.setDefeatedBosses([{ x: 0, z: 98 }]);
    props.build(ENTITIES as never);
    assert.equal(gateOf(scene).visible, false);
    props.dispose();
  });

  test('ignores another boss', () => {
    const scene = new Scene();
    const props = new PropsRenderer(scene);
    props.build(ENTITIES as never);
    props.setDefeatedBosses([{ x: 40, z: 10 }]);
    props.bossDefeated(3);
    for (let i = 0; i < 10; i++) props.update(200);
    assert.equal(gateOf(scene).visible, true);
    props.dispose();
  });
});
