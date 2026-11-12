# 0002: Render with Three.js on WebGL 2

- **Status:** Accepted
- **Date:** 2026-11-11
- **WP:** none (planning)

## Context

The renderer only draws snapshots from the core; it holds no gameplay state. The repo
already depends on Three.js r182 and has a working low-poly renderer in
`src/lib/game/` to learn from. Three.js has a WebGPU renderer, but its node-material
system and APIs differ from the WebGL path, and support varies across browsers.

## Decision

Use Three.js with `WebGLRenderer` (WebGL 2), staying on r182 until a WP has a reason to
upgrade. Upgrades read the Three.js migration guide for every release in between.

## Consequences

- Widest browser and device support, and the most examples for agents to draw on.
- Some techniques (compute-based particles, GPU-driven culling) stay out of reach.
- A later move to `WebGPURenderer` would need material rewrites; write an ADR for it.

## Alternatives considered

- **`WebGPURenderer` from the start:** better future performance, but more API churn
  and fewer examples.
- **Babylon.js:** capable, but a second 3D library in the repo for no gain.
