# 0006: RL smoke test dependencies and findings

- **Status:** Accepted
- **Date:** 2026-11-12
- **WP:** WP-039

## Context

WP-039 builds a minimal `Latticefall/BossArena-v0` for B01 with the `proprio` and
`target` observations, the five-head `MultiDiscrete` action space, and the default boss
arena rewards ([RL environment](../09-rl-environment.md#scenarios)), then trains PPO for
30 minutes on a laptop CPU. The WP exists to catch RL design mistakes before content
scales up. It needs Python packages the base package didn't have, and it raised
questions the spec didn't answer.

## Decision

Add two optional extras to `game/python/pyproject.toml`, and keep the base package and
CI free of them. Extend the `target` observation with move timing, and decode taps as a
press on a step's first tick.

### Dependencies

- **`env`:** `gymnasium>=1.0,<2` and `numpy>=1.26`, for the Gymnasium environment.
- **`train`:** `env` plus `torch>=2.4`, for `examples/ppo_smoke.py`, a single-file
  CleanRL-style PPO. A hand-written PPO is about 150 lines, handles the masked
  `MultiDiscrete` heads and truncation bootstrapping directly, and adds no RL framework.

The local venv has them installed with `uv pip install`. The CI `python` job doesn't:
`tests/test_boss_arena.py` covers the extension class with the standard library only,
and `tests/test_boss_arena_env.py` skips without Gymnasium. Adding `.[env]` to the
`game:py` task later is cheap (about 20 MB) if CI should run the wrapper tests too.

### Environment shape

- **Observations:** `lf-core`'s new `obs` module encodes `proprio`, `target`, and `move`
  as pure reads of the world, with the layout documented on each encoder. It's a plain
  module, not the `obs` Cargo feature the spec mentions: it has no dependencies to gate.
- **Episode start:** `reset` walks the player forward through the fog gate until the
  fight starts, so the gate and intro logic stay in charge; the walk-in isn't part of the
  episode. No extra scenario file is needed.
- **Episode end:** a step stops on the tick the player dies or the boss falls, and later
  steps do nothing, so the world never reaches the respawn. A death resets the boss in
  the same tick, so the reported boss HP is the value from before the death.
- **Rewards:** computed in Python from the HP fractions and event counts that the
  extension reports. The reachability gate and the stall penalty aren't implemented yet;
  B01's arena is closed and the boss always reaches the player.

### Spec changes

- **`target` gets move timing.** The spec listed only the target's "move frame
  fraction". A fraction of a whole move doesn't say when its hit lands, and the `move` ID
  key needs an embedding table that a small MLP doesn't have. `target` now also carries
  whether the hitbox is active now, ticks to the next hit frame, ticks left in the move,
  and whether the move is the last attack of its combo. A human reads the same thing from
  the animation, so it isn't privileged information. Edited:
  [Observations](../09-rl-environment.md#observations).
- **Taps press on a step's first tick.** Every button is a press edge in the core, so a
  tap held for a whole step would never repeat. Guard and heavy (hold) are held, and dodge
  chosen on consecutive steps is held so that it becomes a sprint. A tap repeated on
  consecutive steps needs `frame_skip` of at least 2. Edited:
  [Actions](../09-rl-environment.md#actions).

## Results

`examples/ppo_smoke.py --minutes 30` (seed 1, 16 environments, Apple M4 Pro CPU) ran
20.9 million steps in 30 minutes: 11,600 steps per second overall, and about 26,000 per
second for the environments alone. On 50 held-out seeds:

| Policy | Mean return | Win rate | Boss HP removed | Steps per episode |
|---|---|---|---|---|
| Random (masked) | -5.65 ± 2.18 | 0.02 | 0.28 | 537 |
| Idle | -5.91 ± 0.09 | 0.00 | 0.00 | 411 |
| Scripted (observations only) | 8.90 ± 0.54 | 1.00 | 1.00 | 2,100 |
| PPO, greedy | 10.76 ± 0.08 | 1.00 | 1.00 | 164 |

PPO reached a 95% win rate after 1 minute and 100% before 3 minutes. The scripted
policy, a port of the WP-034 test bot that reads only `proprio` and `target`, also wins
every episode, so the observations carry enough to play the fight as designed.

What PPO learned is the main finding. Its greedy policy never dodges: about 29% of steps
guard, 20% heavy, and 16% light, and it kills the 1,200 HP Sentry in about 11 seconds of
fight while keeping 81% of its HP (20 seeds). The fight is meant to teach "read the
telegraph, roll through, punish" ([B01](../05-enemies-and-bosses.md#b01-hollow-sentry)),
but guarding and trading hits beat it 13 times faster than the patient bot. A 5-minute ablation with the four new move-timing slots
zeroed did just as well (10.70 return, 100% wins), because the winning strategy doesn't
need timing. So this run doesn't show whether the timing slots help PPO; the scripted
policy does need them.

## Consequences

- RL work packages have a working env, PPO script, and baseline numbers to compare
  against. `VecEnv`, `clone_state`, and the other scenarios are still to come.
- The new `target` slots are now part of the observation contract: a browser policy must
  compute them the same way (they come from `lf_core::obs`, so it will).
- B01's tuning is a content finding for the boss lane, not an RL one: the Sentry
  doesn't punish guarding or trading hits. Checking each boss with this script (does PPO
  beat it without dodging?) is a cheap way to catch the same problem in later bosses.
- Smaller findings: `time` is charged per step, so its total depends on `frame_skip`;
  the reward hacking safeguards wouldn't flag this policy, since it's a real win; and
  `Sim.step` never drained the event queue, which grew without bound over long runs
  (fixed in `lf-py`).

## Alternatives considered

- **stable-baselines3:** a solid PPO, but it adds a framework and its `MultiInputPolicy`
  would feed the raw `move` IDs into an MLP as floats. Not worth it for a smoke test.
- **A NumPy-only PPO:** no torch, but hand-written backpropagation is more code to trust
  than the PPO itself.
- **Spawning past the gate with a new scenario file:** faster resets, but it would bypass
  the gate and intro logic the real fight runs through, and resets are already cheap.
- **Keeping `target` as specified and relying on `move` embeddings:** a policy would
  have to learn each move's timing from its ID. The scripted observation-only policy
  needs the timing to dodge, and later bosses are meant to require dodging.
