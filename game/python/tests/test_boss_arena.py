"""Checks the `BossArena` extension class (WP-039). These need no extras; the Gymnasium
environment's tests are in `test_boss_arena_env.py`."""

import struct

import pytest

from latticefall import BossArena

PROPRIO_LEN, TARGET_LEN = BossArena.obs_lengths()
HEADS = BossArena.heads()
IDLE = [0, 0, 0, 0, 0]


def floats(b):
    return struct.unpack(f"<{len(b) // 4}f", b)


def test_reset_reports_shapes_and_starts_inside_the_fight():
    sim = BossArena()
    r = sim.reset(7)
    proprio, target, moves, mask = r[:4]
    assert len(floats(proprio)) == PROPRIO_LEN == 48
    assert len(floats(target)) == TARGET_LEN == 32
    assert len(moves) == 2
    assert len(mask) == sum(HEADS) == 28
    player_hp, boss_hp, phase, fight_ticks, died, won = r[4:10]
    assert (player_hp, boss_hp, fight_ticks, died, won) == (1.0, 1.0, 0, False, False)
    assert phase >= 1
    assert floats(target)[0] == 1.0  # the boss is the target
    assert sim.tick > 0  # the walk-in ticks ran


def test_step_runs_frame_skip_ticks():
    sim = BossArena(frame_skip=3)
    sim.reset(1)
    before = sim.tick
    r = sim.step(IDLE)
    assert sim.tick == before + 3
    assert r[7] == 3


def test_reset_with_a_seed_is_deterministic():
    a, b = BossArena(), BossArena()
    ra, rb = a.reset(42), b.reset(42)
    assert ra[:2] == rb[:2] and a.state_hash() == b.state_hash()
    actions = [[1, 1, 1, 1, 0], [3, 0, 5, 0, 0], IDLE, [5, 1, 4, 2, 3]] * 25
    for act in actions:
        a.step(act)
        b.step(act)
    assert a.state_hash() == b.state_hash()


def test_an_idle_player_dies_and_the_episode_stops():
    sim = BossArena()
    sim.reset(34)
    for _ in range(3000):
        r = sim.step(IDLE)
        if r[8]:
            break
    assert r[8] and not r[9], "the idle player should die"
    assert r[4] == 0.0
    # The boss HP is from before the death resets the fight.
    assert r[5] == 1.0
    tick = sim.tick
    sim.step(IDLE)
    assert sim.tick == tick, "no ticks after the end: no respawn loop"


def test_bad_arguments_raise():
    with pytest.raises(ValueError):
        BossArena(frame_skip=0)
    with pytest.raises(ValueError):
        BossArena("m0_flat")  # no boss
    with pytest.raises(ValueError):
        BossArena("nope")
