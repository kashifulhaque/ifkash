"""Steps one Sim for 100,000 ticks and prints steps per second."""

import time

from latticefall import Sim

TICKS = 100_000
FRAME = bytes([20, 100, 0, 0])


def main() -> None:
    sim = Sim()
    sim.reset(1)
    start = time.perf_counter()
    for _ in range(TICKS):
        sim.step(FRAME)
    per_call = TICKS / (time.perf_counter() - start)

    sim.reset(1)
    frames = FRAME * TICKS
    start = time.perf_counter()
    sim.step_many(frames)
    batched = TICKS / (time.perf_counter() - start)

    print(f"step:      {per_call:>14,.0f} steps/s")
    print(f"step_many: {batched:>14,.0f} steps/s")


if __name__ == "__main__":
    main()
