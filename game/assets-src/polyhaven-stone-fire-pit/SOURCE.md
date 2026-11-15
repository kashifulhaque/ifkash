# Poly Haven Stone Fire Pit

- **Source:** https://polyhaven.com/a/stone_fire_pit (the glTF download at 2K with JPEG textures, 9,617,101 bytes in five files, downloaded 2026-11-15)
- **Author:** Sebastian Platen
- **License:** CC0 1.0 Universal (https://polyhaven.com/license). Every Poly Haven asset is CC0. The download ships no license file. No attribution is required.
- **Use:** The test model for the KTX2 texture pipeline ([ADR 0010](../../../docs/game/decisions/0010-ktx2-textures.md)). It isn't part of any level. The game draws it only for `/game?dev=1&ktx2test=1`.
- **Contents:** One mesh with 3,887 triangles and one material with three 2048 x 2048 maps: base color, normal (OpenGL convention), and a packed occlusion, roughness, and metalness map that the material uses for roughness and metalness.

## What is committed

`stone_fire_pit_2k.glb` is the download packed into one binary file, with nothing else changed.

To regenerate it from a fresh download, complete the following steps:

1. Download these files, and keep the JPEG files in a `textures/` folder next to the `.gltf` file:

   ```text
   https://dl.polyhaven.org/file/ph-assets/Models/gltf/2k/stone_fire_pit/stone_fire_pit_2k.gltf
   https://dl.polyhaven.org/file/ph-assets/Models/gltf/4k/stone_fire_pit/stone_fire_pit.bin
   https://dl.polyhaven.org/file/ph-assets/Models/jpg/2k/stone_fire_pit/stone_fire_pit_diff_2k.jpg
   https://dl.polyhaven.org/file/ph-assets/Models/jpg/2k/stone_fire_pit/stone_fire_pit_nor_gl_2k.jpg
   https://dl.polyhaven.org/file/ph-assets/Models/jpg/2k/stone_fire_pit/stone_fire_pit_arm_2k.jpg
   ```

2. From the repository root, pack them:

   ```sh
   node_modules/.bin/gltf-transform copy stone_fire_pit_2k.gltf \
     game/assets-src/polyhaven-stone-fire-pit/stone_fire_pit_2k.glb
   ```

Then run `lf-tools assets` to write `static/latticefall/dev/stone_fire_pit.glb`. The job in
`assets.json` sets `maxTextureSize` to 1024, so the 2K source is written at 1K.
