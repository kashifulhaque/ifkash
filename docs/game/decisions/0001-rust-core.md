# 0001: Build the simulation as a deterministic Rust core

- **Status:** Accepted, pending confirmation at gate G1
- **Date:** 2026-11-11
- **WP:** none (planning)

## Context

The game must run in the browser for portfolio visitors and headless from Python for
RL training. RL needs thousands of steps per second per core, many parallel instances,
and bit-for-bit reproducibility, so that human replays become datasets and results are
comparable. The repo already uses Rust for the API Worker and Three.js for the old game.

## Decision

Write the whole simulation as one Rust crate (`lf-core`) with no web or Python
dependencies, using:

- `rapier3d` with the `enhanced-determinism` feature for collision and character
  movement, with no SIMD features.
- A fixed 30-tick-per-second simulation; rendering interpolates.
- Combat timing as frame data in content files, independent of skeletal animation.
- RON content files parsed only by Rust, with `ts-rs` types for TypeScript.
- A height-aware navigation grid with A*, not a navmesh.
- `libm`-backed math for every transcendental function.

Expose it to the browser with `wasm-bindgen` and to Python with PyO3 and maturin.

## Consequences

- One source of truth for gameplay; humans and agents face identical rules.
- Golden replays can run natively, in WASM, and from Python in CI.
- Two toolchains (Cargo and Bun) and a bridge to maintain.
- Rapier upgrades and Rust upgrades are determinism-breaking events that need
  re-recorded golden replays.
- Gameplay logic can't be written quickly in TypeScript, even for prototypes.

## Alternatives considered

- **TypeScript core in Bun for RL:** one language, but inter-process overhead per step
  caps RL throughput, and JavaScript math isn't guaranteed identical across engines.
  This is the fallback if gate G1 fails.
- **Bevy:** fast natively, but a large WASM bundle, API churn that makes agent-written
  code go stale, and no determinism guarantee for system ordering.
- **Godot with Godot RL Agents:** a good editor, but a large web export, a scene-based
  workflow that's hard for coding agents, and no documented deterministic headless web
  path.
- **Babylon.js or PlayCanvas:** good renderers, but no native RL path, so a second
  simulation would be needed anyway.
