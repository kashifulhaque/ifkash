# KayKit Dungeon Remastered 1.0

- **Source:** https://github.com/KayKit-Game-Assets/KayKit-Dungeon-Remastered-1.0 (commit `b0ca9bd96a80`, files fetched one by one from `addons/kaykit_dungeon_remastered/Assets/gltf/` on 2026-11-14). Pack page: https://kaylousberg.itch.io/kaykit-dungeon-remastered
- **Author:** Kay Lousberg (https://www.kaylousberg.com)
- **License:** CC0 1.0 Universal (https://creativecommons.org/publicdomain/zero/1.0/). The author's `LICENSE.txt` in the repository, copied here, states it; checked 2026-11-14. Crediting Kay Lousberg is welcome but not required.
- **Status:** Used by one test area, Hollowhearth (see [MODELS.md](../../../docs/game/assets/MODELS.md#model-files-in-the-engine)). The owner approved the pack on 2026-11-14 in [ADR 0008](../../../docs/game/decisions/0008-model-packs.md).

## What it's for

Modular interiors for the castle keep, the catacombs, the mine, and Hollowhearth: stone walls, arches, floors, stairs, pillars, scaffolds (mine supports), torches, candles, banners, shelves (the Archive), tables, chairs, beds, barrels, crates, and chests.

## What is committed

The 16 models that Hollowhearth uses, out of the 88 that the branch `claude/game-models-staging-b685dc` stages. Each GLB embeds the pack's palette texture and colors its faces through it.

- `baked/` holds 15 of them after `game/scripts/bake-vertex-colors.mjs` turned the palette colors into vertex colors and dropped the texture. `lf-tools assets` optimizes these into `static/latticefall/kits/kaykit-dungeon-remastered/`. To rebuild them, run `bun game/scripts/bake-vertex-colors.mjs game/assets-src/kaykit-dungeon-remastered/baked FILE.glb` for each source file except `chest.glb`.
- `chest.glb` isn't baked. `lf-tools assets` copies it as is, with its PNG palette texture, so the game has one textured model to prove that the renderer keeps a model's own materials. When `toktx` is installed, drop `"copy": true` from its entry in `game/assets-src/assets.json` to compress its texture to KTX2.
