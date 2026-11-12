"""Python bindings for the Latticefall simulation.

Importing the package registers the Gymnasium environments when ``gymnasium`` is
installed (the ``env`` extra); the base package needs only the extension.
"""

from ._latticefall import (
    BossArena,
    Sim,
    content_hash,
    embedded_content_hash,
    replay_final_expected,
    replay_frames,
    replay_scenario,
    replay_seed,
    run_replay,
)

__version__ = "0.1.0"

__all__ = [
    "BossArena",
    "Sim",
    "__version__",
    "content_hash",
    "embedded_content_hash",
    "replay_final_expected",
    "replay_frames",
    "replay_scenario",
    "replay_seed",
    "run_replay",
]

try:
    import gymnasium as _gym
except ImportError:  # the `env` extra isn't installed
    pass
else:
    if "Latticefall/BossArena-v0" not in _gym.registry:
        _gym.register(
            id="Latticefall/BossArena-v0",
            entry_point="latticefall.envs:BossArenaEnv",
        )
