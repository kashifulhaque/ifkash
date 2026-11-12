"""Observation and action spaces (docs/game/09-rl-environment.md, "Observations" and
"Actions"). The feature layout of each vector is documented in
``game/crates/core/src/obs/encode.rs``."""

from __future__ import annotations

import numpy as np
from gymnasium import spaces

from ._latticefall import BossArena

#: Sizes of the `move_dir`, `move_speed`, `action`, `lock`, and `quick` heads.
HEADS: tuple[int, ...] = tuple(BossArena.heads())
PROPRIO_LEN, TARGET_LEN = BossArena.obs_lengths()
#: Move registry numbers fit below this (registry numbers are below 100,000).
MOVE_ID_MAX = 100_000


def action_space() -> spaces.MultiDiscrete:
    return spaces.MultiDiscrete(np.array(HEADS, dtype=np.int64))


def observation_space() -> spaces.Dict:
    return spaces.Dict(
        {
            "proprio": spaces.Box(-1.0, 1.0, (PROPRIO_LEN,), np.float32),
            "target": spaces.Box(-1.0, 1.0, (TARGET_LEN,), np.float32),
            "move": spaces.Box(0, MOVE_ID_MAX, (2,), np.int32),
        }
    )


def split_mask(flat: np.ndarray) -> tuple[np.ndarray, ...]:
    """Splits the flat mask into one bool array per head."""
    return tuple(np.split(flat.astype(bool), np.cumsum(HEADS)[:-1]))
