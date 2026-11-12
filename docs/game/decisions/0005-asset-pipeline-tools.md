# 0005: Asset pipeline tools

- **Status:** Accepted
- **Date:** 2026-11-12
- **WP:** WP-013

## Context

[The presentation spec](../07-presentation.md#pipeline) has `lf-tools assets` run
`gltf-transform` to compress source art into `static/latticefall/`, with meshopt for
geometry and animation and KTX2 for textures. WP-013 is the first work package that needs
it. The owner approved the tools and the one new pack below in chat for this WP.

## Decision

Use these tool dependencies:

- **`@gltf-transform/cli` 4.5.1**, a root `devDependency` installed with Bun.
  `lf-tools assets` runs `node_modules/.bin/gltf-transform optimize` (and falls back to
  `bunx @gltf-transform/cli@4.5.1`). Its `@gltf-transform/core` dependency is also used by
  `game/scripts/trim-glb.mjs`, which strips meshes and unused clips from a source library.
- **KTX-Software's `toktx`**, a system tool needed only when a source file has textures.
  `lf-tools assets` checks the GLB for images and passes `--texture-compress ktx2` only then,
  and fails with a pointer to this ADR if `toktx` is missing. Install it from the official
  KhronosGroup/KTX-Software release. It isn't in Homebrew core (`brew install ktx-software`
  doesn't exist), and no current asset has textures, so it is not installed yet.
- **Quaternius Universal Animation Library**, Standard edition (CC0), downloaded from
  `https://quaternius.itch.io/universal-animation-library`. Its license and source are
  recorded in `game/assets-src/universal-animation-library/`.

The optimize step turns off `flatten`, `join`, `instance`, `palette`, and `simplify`.
Characters keep their scene graph so bones stay addressable, and keep flat material colors
so armor tints can edit them. Meshopt compresses geometry and animation.

## Consequences

- `bun install` provides the glTF tooling; contributors who don't touch art need nothing else.
- Output in `static/latticefall/` is committed, so CI and deploys don't run the pipeline.
- The first textured asset needs `toktx` installed, and the browser needs the Basis
  transcoder in `static/latticefall/basis/` (the renderer's `KTX2Loader` already points
  there). Neither is set up yet.
- Credits for the animation library come from `game/assets-src/universal-animation-library/`
  through `lf-tools credits` when that command exists.

## Alternatives considered

- **`bunx` only, no devDependency:** Downloads the CLI on each run and can't pin the
  scripting API the trim script needs.
- **Writing the glTF compression in Rust:** No mature meshopt and KTX2 crates, and the
  spec already names `gltf-transform`.
- **Quaternius's Pro edition of the library:** Not needed; the Standard edition has every
  locomotion clip WP-013 uses.
