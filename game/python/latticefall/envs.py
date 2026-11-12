"""Gymnasium environments. This first cut has `Latticefall/BossArena-v0` (WP-039)."""

from __future__ import annotations

from typing import Any

import gymnasium as gym
import numpy as np

from . import rewards, spaces
from ._latticefall import BossArena

#: Simulation ticks per second (`lf_core::world::DT`).
TICK_HZ = 30
#: Boss arena truncation (docs/game/09-rl-environment.md, "Scenarios"): 300 s.
BOSS_ARENA_SECONDS = 300

#: The scenario for each boss the environment can load.
BOSS_SCENARIOS = {"b01_hollow_sentry": "boss_b01"}


class BossArenaEnv(gym.Env):
    """One boss fight. The episode starts inside the arena with the fight on (``reset``
    walks the player through the fog gate) and ends when the player dies or the boss
    falls; it's truncated after ``max_seconds`` of fight.

    Args:
        boss: A key of ``BOSS_SCENARIOS``. Only B01 exists so far.
        frame_skip: Ticks per step (2 gives 15 decisions per second).
        max_seconds: Truncation, in seconds of fight.
        reward_weights: Overrides for ``rewards.BOSS_ARENA_WEIGHTS``.
        data_dir: A content directory (see ``latticefall.Sim``).
    """

    metadata = {"render_modes": []}

    def __init__(
        self,
        boss: str = "b01_hollow_sentry",
        frame_skip: int = 2,
        max_seconds: float = BOSS_ARENA_SECONDS,
        reward_weights: dict[str, float] | None = None,
        data_dir: str | None = None,
    ) -> None:
        if boss not in BOSS_SCENARIOS:
            raise ValueError(f"unknown boss {boss!r}; known: {sorted(BOSS_SCENARIOS)}")
        self.boss = boss
        self.sim = BossArena(BOSS_SCENARIOS[boss], frame_skip, data_dir)
        self.max_ticks = int(max_seconds * TICK_HZ)
        self.weights = {**rewards.BOSS_ARENA_WEIGHTS, **(reward_weights or {})}
        self.action_space = spaces.action_space()
        self.observation_space = spaces.observation_space()
        self._last: tuple | None = None
        self._episode_seed = 0

    def _unpack(self, r: tuple) -> tuple[dict[str, np.ndarray], dict[str, Any]]:
        (proprio, target, moves, mask, php, bhp, phase, fight_ticks, dead, won, _s, _p) = r
        obs = {
            "proprio": np.frombuffer(proprio, dtype="<f4").copy(),
            "target": np.frombuffer(target, dtype="<f4").copy(),
            "move": np.array(moves, dtype=np.int32),
        }
        info = {
            "action_mask": spaces.split_mask(np.frombuffer(mask, dtype=np.uint8)),
            "player_hp_frac": php,
            "target_hp_frac": bhp,
            "boss_phase": int(phase),
            "tick": int(fight_ticks),
            "player_died": bool(dead),
            "boss_defeated": bool(won),
        }
        return obs, info

    def reset(self, *, seed: int | None = None, options: dict | None = None):
        super().reset(seed=seed)
        if seed is None:
            seed = int(self.np_random.integers(0, 2**63 - 1))
        self._episode_seed = seed
        self._last = self.sim.reset(seed)
        obs, info = self._unpack(self._last)
        info["reward_components"] = {k: 0.0 for k in self.weights}
        return obs, info

    def step(self, action):
        if self._last is None:
            raise RuntimeError("call reset() before step()")
        heads = [int(a) for a in np.asarray(action).reshape(-1)]
        prev = self._last
        r = self.sim.step(heads)
        self._last = r
        obs, info = self._unpack(r)
        parts = rewards.components(
            prev[4],
            r[4],
            prev[5],
            r[5],
            died=r[8] and not prev[8],
            won=r[9] and not prev[9],
            stance_breaks=r[10],
            phase_starts=r[11],
        )
        info["reward_components"] = parts
        terminated = bool(r[8] or r[9])
        truncated = not terminated and r[7] >= self.max_ticks
        return obs, rewards.weighted(parts, self.weights), terminated, truncated, info
