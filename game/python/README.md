# latticefall (Python)

Python bindings for the Latticefall simulation, built with maturin from `crates/py`.
This is the stub: one `Sim` and the replay helpers.

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
