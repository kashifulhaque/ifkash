# Waking Hollow cave kit

The cave that Waking Hollow draws in place of its greybox: rock walls and ceilings, floors,
the jump ledge, the collapsed corridor, the arena's masonry, dome, and pillars, the
dressing (stalactites, stalagmites, rubble, roots, bones), torches, candles, and the
bricked-up hollow wall.

- **Geometry:** made by script, so it carries no third-party license.
  `game/art/cave/build_cave.py` builds it in headless Blender from the level's collision
  layout (`game/data/levels/waking_hollow/layout.ron`) and `game/art/cave/cave.json`.
- **Light:** baked into vertex colors by the same script with Cycles: ambient occlusion
  times the torches, candles, and Lantern listed in `cave.json`.
- **Textures:** five CC0 scanned materials from Poly Haven, at 1024 x 1024 (the roots at
  512 x 512), each with a base color map, an OpenGL normal map, and a packed occlusion,
  roughness, and metalness map. The game uses roughness and metalness from the packed map.

## Texture sources

Every Poly Haven asset is CC0 1.0 Universal (https://polyhaven.com/license). No
attribution is required; the authors are credited here anyway. The files were downloaded
on 2026-11-15 as the 1K JPEG set. `game/art/cave/textures.json` lists each file's MD5.

| Use | Poly Haven asset | Author | Source |
|---|---|---|---|
| Wet rock: walls, ceilings, ledge, rubble | `dark_rock_02` | Amal Kumar | https://polyhaven.com/a/dark_rock_02 |
| Packed mud floor | `brown_mud_03` | Rob Tuytel | https://polyhaven.com/a/brown_mud_03 |
| Old stone masonry: gate facade, arena walls, pillars, hollow wall | `stone_brick_wall_001` | Dimitrios Savva (photography), Rico Cilliers (processing) | https://polyhaven.com/a/stone_brick_wall_001 |
| Arena flagstones | `monastery_stone_floor` | Amal Kumar | https://polyhaven.com/a/monastery_stone_floor |
| Roots and torch handles | `bark_brown_02` | Rob Tuytel | https://polyhaven.com/a/bark_brown_02 |

## What is committed

- `cave.glb` and `hollow_wall.glb`: the script's output, with the JPEG textures embedded.
  `lf-tools assets` compresses them to `static/latticefall/kits/waking-hollow/` (meshopt
  geometry, KTX2 textures).
- `../art/waking_hollow.json`: the area art manifest the script writes.

The source JPEG files aren't committed. To rebuild the kit, run the following command
from the repository root:

```sh
mise run art:cave
```

The task downloads the textures to `~/cinderheart-art/cave-textures/` (checking each
MD5), runs the Blender script, and then runs `lf-tools assets`. To skip the light bake
while you iterate on shapes, run the script directly with `--no-bake`:

```sh
blender --background --factory-startup --python game/art/cave/build_cave.py -- --no-bake
```
