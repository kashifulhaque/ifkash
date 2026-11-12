# latticefall (Python)

Python bindings for the Latticefall simulation, built with maturin from `crates/py`.
It has one `Sim`, the replay helpers, and the `Latticefall/BossArena-v0` Gymnasium
environment (WP-039).

## Set up and test

From the repository root, run:

```sh
mise run game:py
```

The task creates `game/python/.venv`, installs maturin and pytest into it, builds the
extension in release mode, runs pytest, and runs the benchmark. To do the steps by hand:

```sh
cd game/python
uv venv --python 3.11 .venv
uv pip install --python .venv/bin/python maturin pytest
VIRTUAL_ENV=$PWD/.venv PATH=$PWD/.venv/bin:$PATH maturin develop --release
.venv/bin/python -m pytest -q
.venv/bin/python examples/bench.py
```

Set `RUSTUP_TOOLCHAIN=1.99.1` so Cargo uses the pinned toolchain.

## Content

`Sim(data_dir=None)` loads content from disk, so you can edit `game/data` without
recompiling. It looks in this order:

1. The `data_dir` argument.
2. The `LATTICEFALL_DATA` environment variable.
3. `game/data` in the source tree the module was built from.
4. The content embedded in the extension.

## Example

```python
from latticefall import Sim

sim = Sim()
sim.reset(seed=1)
sim.step(bytes([0, 100, 0, 0]))  # move_x, move_z, buttons (u16, little endian)
print(sim.tick, hex(sim.state_hash()))
```

## Boss arena environment

`Latticefall/BossArena-v0` is one boss fight, B01 for now, with the `proprio` and
`target` observations, the `move` IDs, the five-head `MultiDiscrete` action space with
masks, and the default boss arena rewards (docs/game/09-rl-environment.md). `reset` walks
the player through the fog gate, so each episode starts inside the arena with the fight
on. An episode ends when the player dies or the boss falls, and it's truncated after
300 s of fight. ADR 0006 records the design choices and what the smoke test found.

Install the extras into the venv first. `env` adds Gymnasium and NumPy; `train` adds
PyTorch for the PPO example:

```sh
uv pip install --python .venv/bin/python -e ".[train]"
```

```python
import gymnasium as gym
import numpy as np
import latticefall  # registers Latticefall/BossArena-v0

env = gym.make("Latticefall/BossArena-v0")
obs, info = env.reset(seed=7)
done = False
while not done:
    action = [np.random.choice(np.flatnonzero(m)) for m in info["action_mask"]]
    obs, reward, terminated, truncated, info = env.step(action)
    done = terminated or truncated
```

### PPO smoke test

`examples/ppo_smoke.py` trains a CleanRL-style PPO on a laptop CPU for a wall-clock
budget and then evaluates random, idle, scripted, and PPO policies on held-out seeds. It
writes `log.txt`, `summary.json`, and `policy.pt` to `game/python/runs/<name>/`, which
Git ignores.

```sh
.venv/bin/python examples/ppo_smoke.py --minutes 30
```

Results of the WP-039 run (`--minutes 30`, seed 1, 16 environments, Apple M4 Pro CPU):
20.9 million steps in 30 minutes, 11,600 steps per second overall (the environments alone
step at about 26,000 per second). Evaluation on 50 held-out seeds:

| Policy | Mean return | Win rate | Boss HP removed | Steps per episode |
|---|---|---|---|---|
| Random (masked) | -5.65 ± 2.18 | 0.02 | 0.28 | 537 |
| Idle | -5.91 ± 0.09 | 0.00 | 0.00 | 411 |
| Scripted (observations only) | 8.90 ± 0.54 | 1.00 | 1.00 | 2,100 |
| PPO, sampled | 10.75 ± 0.08 | 1.00 | 1.00 | 165 |
| PPO, greedy | 10.76 ± 0.08 | 1.00 | 1.00 | 164 |

PPO wins every episode within the first 3 minutes. ADR 0006 discusses what it learned.
