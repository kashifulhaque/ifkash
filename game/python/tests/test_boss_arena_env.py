"""Checks the `Latticefall/BossArena-v0` Gymnasium environment (WP-039). Skipped when
the `env` extra (gymnasium and numpy) isn't installed."""

import pytest

gym = pytest.importorskip("gymnasium")
np = pytest.importorskip("numpy")

from latticefall import BossArena  # noqa: E402

HEADS = BossArena.heads()
IDLE = [0, 0, 0, 0, 0]


@pytest.fixture
def env():
    e = gym.make("Latticefall/BossArena-v0")
    yield e
    e.close()


def test_spaces_and_reset(env):
    obs, info = env.reset(seed=3)
    assert env.observation_space.contains(obs)
    assert tuple(env.action_space.nvec) == tuple(HEADS)
    assert [len(m) for m in info["action_mask"]] == list(HEADS)
    assert info["tick"] == 0 and info["target_hp_frac"] == 1.0


def test_seeded_resets_match(env):
    a, _ = env.reset(seed=11)
    b, _ = env.reset(seed=11)
    for k in a:
        np.testing.assert_array_equal(a[k], b[k])


def test_masked_random_steps_and_reward_sanity(env):
    rng = np.random.default_rng(0)
    obs, info = env.reset(seed=5)
    for _ in range(400):
        act = [rng.choice(np.flatnonzero(m)) for m in info["action_mask"]]
        obs, reward, term, trunc, info = env.step(act)
        assert env.observation_space.contains(obs)
        parts = info["reward_components"]
        assert all(parts[k] >= 0.0 for k in ("damage_dealt", "damage_taken"))
        weights = env.unwrapped.weights
        assert reward == pytest.approx(sum(parts[k] * weights[k] for k in parts))
        if term or trunc:
            break


def test_death_is_terminal_with_the_death_penalty(env):
    env.reset(seed=34)
    for _ in range(3000):
        _, reward, term, trunc, info = env.step(IDLE)
        if term:
            break
    assert term and not trunc and info["player_died"]
    assert info["reward_components"]["death"] == 1.0
    assert reward < -4.0


def test_truncation():
    e = gym.make("Latticefall/BossArena-v0", max_seconds=1.0)
    e.reset(seed=1)
    for _ in range(14):
        _, _, term, trunc, _ = e.step(IDLE)
        assert not (term or trunc)
    _, _, term, trunc, info = e.step(IDLE)
    assert trunc and not term and info["tick"] == 30
