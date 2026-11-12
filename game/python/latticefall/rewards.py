"""Reward components and default weights (docs/game/09-rl-environment.md, "Rewards").

Each step reports every component unweighted in ``info["reward_components"]``; the
reward is their weighted sum. This first cut has the components a boss arena uses. The
reachability gate and the stall penalty under "Reward hacking safeguards" aren't in yet:
B01's arena is closed, and the boss always reaches the player.
"""

from __future__ import annotations

from typing import Mapping

#: Default weights for `Latticefall/BossArena-v0`.
BOSS_ARENA_WEIGHTS: dict[str, float] = {
    "damage_dealt": 1.0,
    "damage_taken": -0.5,
    "win": 10.0,
    "death": -5.0,
    "stance_break": 0.5,
    "phase": 1.0,
    "time": -0.001,
    "heal_waste": -0.1,
}

#: HP fraction above which a heal counts as wasted.
HEAL_WASTE_ABOVE = 0.9


def components(
    prev_player_hp: float,
    player_hp: float,
    prev_boss_hp: float,
    boss_hp: float,
    *,
    died: bool,
    won: bool,
    stance_breaks: int,
    phase_starts: int,
) -> dict[str, float]:
    """Returns each reward component for one step, unweighted."""
    return {
        "damage_dealt": max(0.0, prev_boss_hp - boss_hp),
        "damage_taken": max(0.0, prev_player_hp - player_hp),
        "win": 1.0 if won else 0.0,
        "death": 1.0 if died else 0.0,
        "stance_break": float(stance_breaks),
        "phase": float(phase_starts),
        "time": 1.0,
        "heal_waste": 1.0
        if player_hp > prev_player_hp and prev_player_hp > HEAL_WASTE_ABOVE
        else 0.0,
    }


def weighted(parts: Mapping[str, float], weights: Mapping[str, float]) -> float:
    """The weighted sum of `parts`; components without a weight count 0."""
    return float(sum(v * weights.get(k, 0.0) for k, v in parts.items()))
