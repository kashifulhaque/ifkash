"""Checks that the Python host agrees with the native simulation."""

import re
from pathlib import Path

import pytest

from latticefall import (
    Sim,
    content_hash,
    embedded_content_hash,
    replay_final_expected,
    replay_frames,
    replay_scenario,
    replay_seed,
    run_replay,
)

REPLAY_DIR = Path(__file__).resolve().parents[2] / "tests" / "replays"
REPLAYS = sorted(REPLAY_DIR.glob("*.lfr"))
# These replays send `Command::Rest` at a Lantern, which changes the run. `Sim.step` takes
# action frames only, so the Python host can't reproduce them by stepping (it still runs
# them whole with `run_replay`). Drop this when lf-py can send commands.
NEEDS_COMMANDS = {"m2_death_recover.lfr", "m2_death_lose_remnant.lfr"}
STEPPABLE = [p for p in REPLAYS if p.name not in NEEDS_COMMANDS]


def test_replays_exist():
    assert REPLAYS, f"no .lfr files in {REPLAY_DIR}"


@pytest.mark.parametrize("path", REPLAYS, ids=lambda p: p.name)
def test_run_replay_matches_recorded_hash(path):
    data = path.read_bytes()
    assert run_replay(data) == replay_final_expected(data)


@pytest.mark.parametrize("path", REPLAYS, ids=lambda p: p.name)
def test_readme_hash_if_listed(path):
    readme = (REPLAY_DIR / "README.md").read_text()
    # Only checks a hash that the README lists next to the file name.
    match = re.search(re.escape(path.name) + r"\W+(?:.*?)\b([0-9a-f]{16})\b", readme)
    if match is None:
        pytest.skip("README does not list a hash for this replay")
    assert int(match.group(1), 16) == replay_final_expected(path.read_bytes())


@pytest.mark.parametrize("path", STEPPABLE, ids=lambda p: p.name)
def test_stepping_matches_recorded_hash(path):
    data = path.read_bytes()
    frames = replay_frames(data)
    sim = Sim()
    sim.reset(replay_seed(data), replay_scenario(data))
    for i in range(0, len(frames), 4):
        sim.step(frames[i : i + 4])
    assert sim.tick == len(frames) // 4
    assert sim.state_hash() == replay_final_expected(data)


@pytest.mark.parametrize("path", STEPPABLE, ids=lambda p: p.name)
def test_step_many_matches_step(path):
    data = path.read_bytes()
    sim = Sim()
    sim.reset(replay_seed(data), replay_scenario(data))
    assert sim.step_many(replay_frames(data)) == sim.tick
    assert sim.state_hash() == replay_final_expected(data)


def test_clone_diverges_independently_then_converges():
    data = REPLAYS[0].read_bytes()
    frames = replay_frames(data)
    half = (len(frames) // 8) * 4
    sim = Sim()
    sim.reset(replay_seed(data), replay_scenario(data))
    sim.step_many(frames[:half])
    copy = sim.clone()
    assert copy.state_hash() == sim.state_hash()
    sim.step_many(frames[half:])
    assert copy.tick == half // 4  # stepping the original left the copy alone
    copy.step_many(frames[half:])
    assert copy.state_hash() == sim.state_hash()


def test_content_hash_disk_equals_embedded():
    assert content_hash() == embedded_content_hash()
    assert Sim().content_hash == embedded_content_hash()


def test_explicit_data_dir_and_bad_dir(tmp_path):
    data_dir = Path(__file__).resolve().parents[2] / "data"
    assert Sim(str(data_dir)).content_hash == embedded_content_hash()
    with pytest.raises(ValueError):
        Sim(str(tmp_path / "missing"))


def test_step_rejects_bad_lengths():
    sim = Sim()
    for bad in (b"", b"\0\0\0", b"\0\0\0\0\0"):
        with pytest.raises(ValueError):
            sim.step(bad)
    with pytest.raises(ValueError):
        sim.step_many(b"\0" * 5)
    assert sim.tick == 0


def test_reset_rejects_unknown_scenario():
    with pytest.raises(ValueError):
        Sim().reset(1, "nope")


def test_run_replay_reports_mismatch():
    data = bytearray(REPLAYS[0].read_bytes())
    with pytest.raises(ValueError):
        run_replay(bytes(data[:10]))  # truncated: cannot decode
