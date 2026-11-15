# Models: needs, sources, and integration handoff

This document lists every 3D model Cinderheart needs, what the game already draws, and the
models staged for a later integration phase: where each comes from, its license, and what
integration needs. It was written on 2026-11-14 on the branch
`claude/game-models-staging-b685dc`.

**Two areas are wired into the game: Hollowhearth and Waking Hollow.** GitHub issue 84 added the engine
path that loads model files, described in
[Model files in the engine](#model-files-in-the-engine), and brought over only the files
that area needs. Every other area draws its greybox, and the rest of the staged files stay
on the staging branch. The simulation and the level and kit data are unchanged.

The low-poly packs aren't going to dress the other regions:
[the art direction research](../art-direction-research.md) plans to replace them. The
sections after the next one describe the staging branch as it was written, and their
integration checklist is kept for reference only.

The owner approved the eight new third-party packs on 2026-11-14 in
[ADR 0008](../decisions/0008-model-packs.md).

## Model files in the engine

The renderer draws a model file in place of greybox when data names one. A model keeps the
materials its file carries, so vertex colors, flat colors, and textured PBR maps all draw
as authored. Swapping art is a data change: replace the file, or point the data at
another one.

Data names a model in two places:

- **The kit catalog.** A piece in `game/data/kits/*.ron` with `model: Some("PATH")` draws
  that file wherever a level places or scatters the piece. No shipped piece has a model,
  because the kit catalog is part of the content hash and a change to it needs the golden
  replays rehashed.
- **The area art manifest**, `game/assets-src/art/AREA.json`. It is render data only, and
  the simulation never reads it. `lf-tools assets` copies it to
  `static/latticefall/art/AREA.json`. The format is documented at the top of
  `src/lib/latticefall/render/kits/art.ts`. It has the following parts:
  - `models` gives each model an ID and a file path relative to `static/latticefall/`.
  - `instances` fills a named layout box with copies of a model. The copies are scaled to
    the box (`stretch`) or keep their proportions (`contain`), so the drawing follows the
    collider.
  - `props` places models freely, with no collider. A prop with `covers` replaces those
    layout instances' greybox, so one model built around the colliders can stand in for
    many boxes.
  - `entities` names the Lantern's model, each NPC's figure, and the hollow wall's model.
  - `lights`, `look`, and `grade` give fixed light spots (lit by a small pool of point
    lights), fog and ambient overrides, and a color grade. They're render data, like the
    rest of the file.
  - A model with `"nodes": true` keeps each mesh node as its own part, so the renderer can
    cull a large world-space model piece by piece.

The following rules apply to every model:

- **Fallback:** Until a file loads, and for good if it fails to load, the greybox or the
  procedural stand-in draws. A failed file logs a console warning.
- **Streaming:** The art manifest loads with its area. A model file loads when something
  that uses it first comes within the draw distance, and unloads 20 seconds after nothing
  is near. Leaving the area frees everything.
- **Cost:** All the copies of one model in an area take one draw call per material in the
  file, plus one more per shadow pass. A model with `"shadow": false` skips the shadow pass.
- **NPCs:** A figure is a skinned model on the Wayfarer's rig. It loops one clip of the
  animation library through the Wayfarer's retargeting and casts no shadow.

To add a model to an area, do the following:

1. Put the source file in a pack folder in `game/assets-src/`, next to the pack's license
   and `SOURCE.md`.
2. Add an entry to `game/assets-src/assets.json`. A file with textures needs `toktx`
   ([ADR 0005](../decisions/0005-asset-pipeline-tools.md)) or `"copy": true`.
3. Name the output file in the area's art manifest.
4. Run `cargo run -p lf-tools -- assets` in `game/`, and commit the output in
   `static/latticefall/`.
5. Run the benchmark for the area, `/game?dev=1&bench=AREA`, and check the draw calls and
   triangles against [the performance budgets](../01-architecture.md#performance-budgets).

Waking Hollow draws a cave kit built by script in headless Blender from its collision
layout, with five CC0 Poly Haven textures and light baked into vertex colors. For its
sources, see `game/assets-src/waking-hollow-cave/SOURCE.md`; to rebuild it, run
`mise run art:cave`.

Hollowhearth uses 16 KayKit Dungeon Remastered models (15 baked to vertex colors and one
chest that keeps its palette texture), the procedural Lantern, and the five Quaternius NPC
figures.

## What the game already draws

The following models exist, so they aren't gaps:

- **The Wayfarer:** `static/latticefall/actors/wayfarer.glb` (Quaternius `Adventurer.glb`)
  with 13 clips from the Universal Animation Library (`ual.glb`).
- **All 22 regular enemies and elites:** procedural, in
  `src/lib/latticefall/render/procedural/models/` (husks, hounds, knights, archers, Lamp
  Mites, Shard Golems, skeletons, and the rest). Each has a bone hierarchy and poses driven
  by `move_id` and `move_frame`.
- **All 12 bosses:** procedural. B01 is `procedural/models/hollowSentry.ts`; B02 to B12 are
  in `render/bosses/b02/` to `b12/`. That includes the five creatures the presentation spec
  names as procedural: the Forgeheart Colossus (`b08/colossus.ts`), the Lamplight Matriarch
  (`b09/matriarch.ts`), the Unspooled (`b12/unspooled.ts`), Lamp Mites
  (`procedural/models/lampMite.ts`), and Shard Golems (`procedural/models/shardGolem.ts`).
- **Weapons and shields:** one procedural builder per weapon class in
  `render/equipment/weapons.ts`, tinted per weapon.
- **Armor:** tints plus procedural attachments per set in `render/equipment/armor.ts`, on
  the Wayfarer's bones.
- **World mechanics:** greybox props in `render/props/` (ladders, lifts, doors, levers,
  chests, pools, and fog gates), and greybox NPC figures in `render/props/hearth.ts`.

Because the five spec-named creatures already have procedural rigs and poses in the
renderer, this branch doesn't prototype them again. A second copy would drift from the one
the game plays.

## What's missing

The following table lists the real gaps and what this branch stages for each:

| Category | Gap today | Staged here |
|---|---|---|
| Environment kits | Only the greybox kit (5 blocks) and 4 meadow scatter pieces with no model; every region draws greybox | 5 Kenney kits and the KayKit dungeon kit (331 pieces), plus 7 procedural landmark and dressing pieces |
| Lantern checkpoint | No model at all | Procedural `lantern.glb` |
| Mechanics props | Greybox shapes | Procedural lever, ladder, mine rails, mine cart, Glasswing mirror, and Warden Brazier; Kenney and KayKit doors, gates, and chests |
| NPCs | Greybox robed figures | Five Quaternius modular characters already in the repo, on the Wayfarer's rig |
| Weapons and shields | Procedural, by class | 23 KayKit weapon and shield meshes as an optional art upgrade |
| Armor | Procedural attachments | Nothing new; see [Still missing](#still-missing) |
| Enemies and bosses | None | Nothing new needed |

## Staged files

The following table summarizes what's committed. Sizes are on disk. The optimized total is
what integration copies into `static/latticefall/`.

| Folder | Contents | Size |
|---|---|---|
| `game/assets-src/kenney-*/` (5 packs) | 258 source GLBs, licenses, and `SOURCE.md` files | 4.6 MB |
| `game/assets-src/kaykit-*/` (3 packs) | 111 source models, licenses, and `SOURCE.md` files | 4.8 MB |
| `game/assets-src/procedural/` | `build.mjs` and its 18 GLBs | 0.7 MB |
| `game/assets-src/quaternius-modular-characters/` | 5 NPC GLBs, byte-identical to files in `static/models/npc/`, so they add nothing to the repository | 0 MB added |
| `game/assets-staging/out/` | All 392 models optimized by `lf-tools assets` (meshopt, no textures) | 3.8 MB |
| `game/assets-staging/models.tsv` | Per-model triangle count, size in meters, animated node names, and optimized bytes | 28 KB |

The scripts are as follows:

- `game/scripts/bake-vertex-colors.mjs` bakes each model's palette texture or material
  colors into `COLOR_0` vertex colors and drops textures and UVs. The kit renderer keeps
  only position, normal, and color (`render/kits/pieceGeometry.ts`), and every chosen pack
  colors its faces through a texture or material colors, so this step is required.
- `game/assets-staging/stage.sh` bakes every pack into `game/assets-staging/baked/`
  (gitignored), writes `game/assets-staging/assets.json`, and runs
  `lf-tools assets --src game/assets-staging --out game/assets-staging/out`.
- `game/assets-staging/stats.mjs` writes `models.tsv` from the optimized output.

To rebuild everything, run the following commands from the repository root:

```sh
bun game/assets-src/procedural/build.mjs
bash game/assets-staging/stage.sh
bun game/assets-staging/stats.mjs
```

`lf-tools credits` lists every folder in `game/assets-src/` that has a `SOURCE.md`, so on
this branch it would add the staged packs to `src/lib/latticefall/audio/credits.json`.
Don't run it with the default output until the packs are integrated. Each staged
`SOURCE.md` parses without errors (checked with `--out` pointed at a scratch file).

## Sources

Every pack was checked on its own page on 2026-11-14. All are CC0 1.0, so none needs
credits; Kenney and Kay Lousberg welcome optional credit.

| Pack | Folder | License check | Use |
|---|---|---|---|
| Kenney Castle Kit 2.0 | `kenney-castle-kit` | https://kenney.nl/assets/castle-kit states "Creative Commons CC0"; `License.txt` in the zip | Castle Graymantle walls, towers, gates, bridge, stairs, ballista, and flags |
| Kenney Graveyard Kit 5.0 | `kenney-graveyard-kit` | https://kenney.nl/assets/graveyard-kit, same | Ossuary and catacomb crypts, coffins, gravestones, fences, candles, lanterns, and urns |
| Kenney Nature Kit 2.1 | `kenney-nature-kit` | https://kenney.nl/assets/nature-kit, same | Trees, rocks, cliffs, grass, mushrooms, lily pads, ruined statues, bridges, and the rowboat |
| Kenney Fantasy Town Kit 2.0 | `kenney-fantasy-town-kit` | https://kenney.nl/assets/fantasy-town-kit, same | Windmill, Stillwater Village houses, arches, stalls, carts, and the watermill |
| Kenney Survival Kit 2.0 | `kenney-survival-kit` | https://kenney.nl/assets/survival-kit, same | Anvil and workbench (the Smith, the Foundry), pickaxe, hammer, and signposts |
| KayKit Dungeon Remastered 1.0 | `kaykit-dungeon-remastered` | `LICENSE.txt` in https://github.com/KayKit-Game-Assets/KayKit-Dungeon-Remastered-1.0 (commit `b0ca9bd96a80`) | Castle keep, catacomb, mine, and Hollowhearth interiors and furniture |
| KayKit Character Pack: Adventurers 1.0 | `kaykit-adventurers` | `LICENSE.txt` in https://github.com/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0 (commit `672074b73ba2`) | Swords, axes, dagger, staff, wand, crossbow, spellbooks, and shields |
| KayKit Character Pack: Skeletons 1.0 | `kaykit-skeletons` | `LICENSE.txt` in https://github.com/KayKit-Game-Assets/KayKit-Character-Pack-Skeletons-1.0 (commit `15b62b9bad12`) | Skeleton blade, axe, staff, and shields |
| Quaternius modular characters | `quaternius-modular-characters` | Already in the repo; CC0 per `static/models/CREDITS.md`. The missing license file is open as GitHub issue 42 | The five NPCs |

Each pack folder has a `SOURCE.md` with the download details and a `LICENSE.txt` with the
author's own license text. The files were treated as data: nothing from a download was
run.

### Sources considered and rejected

- **Quaternius Stylized Nature MegaKit, Fantasy Props MegaKit, Medieval Village MegaKit,
  and Modular Character Outfits - Fantasy** (CC0, checked 2026-11-14): their models use
  PBR texture sets (base color, normal, and ORM maps of 1-14 MB each) and their zips are
  99-280 MB. That conflicts with the art direction (vertex colors and small gradient
  textures only) and would need `toktx`. The outfits are also built for the Universal
  Base Characters, not the Wayfarer's rig. Nothing was downloaded beyond the zip listings.
- **Quaternius Ultimate Modular Ruins** (CC0, flat colors): a good fit for the Verge's
  chapel and aqueduct, but it's served only from a Google Drive folder that can't be
  fetched without a browser. See [Still missing](#still-missing).
- **Quaternius Bestiary - Dungeon Monsters Kit:** its page names the Quaternius Asset
  License (QAL), not CC0.
- **`static/models/nature/`, `props/`, and `vehicles/`** (threejsassets.com): not CC0, and
  the license forbids reselling the files as a pack. Not used.
- **Mixamo:** not used, per the spec.

## Procedural models

The procedural models are in `game/assets-src/procedural/`, built by `build.mjs`. They
follow the same conventions as the baked packs: meters, +Y up, front toward +Z, the origin
at the base center, flat shading, and `COLOR_0` vertex colors on a white material. Parts
that glow use a second material named `glow` with an emissive color, so the renderer can
find, dim, and brighten them. Parts that move are named nodes with the pivot at the node
origin. These are props, so none has a skin; their animation is rotating or toggling the
named nodes.

The following table lists each model with its triangle count after optimization:

| Model | Triangles | Size (m, x × y × z) | Named nodes | For |
|---|---|---|---|---|
| `lantern` | 224 | 1.0 × 1.9 × 1.0 | `cage`, `flame` | Every Lantern (L00-L23). Light `flame` when lit; sway `cage` gently |
| `warden_brazier` | 204 | 1.0 × 1.6 × 1.0 | `fire` | Whitewarden braziers; show `fire` when `brazier.<id>.lit` is set |
| `glass_mirror` | 128 | 1.3 × 1.8 × 1.0 | `yoke` (turns about Y), `mirror` (tilts about X) | Glasswing Grotto light-beam mirrors (D5) |
| `crystal_cluster_small`, `_medium`, `_large` | 120, 168, 240 | 0.6, 1.1, and 2.4 wide | none | Hollowdeep and Glasswing Grotto dressing |
| `glow_fungus_cluster` | 460 | 0.7 × 0.7 × 1.2 | none | Fungal Shore dressing; matches the darkness rule's glow zones |
| `lamp_tree` | 876 | 14 × 19.5 × 14 | `pods` (sway) | Hollowdeep's landmark, the Lamp Tree |
| `mine_rail_straight` | 84 | 1.3 × 0.1 × 2.0 | none | Cinderdeep Mine rails, one 2 m grid cell |
| `mine_cart` | 152 | 1.3 × 1.0 × 1.6 | `body`, `wheel_fl`, `wheel_fr`, `wheel_bl`, `wheel_br` | The D4 mine carts; spin the wheels about X |
| `lever` | 60 | 0.6 × 1.5 × 0.8 | `handle` (about X, +0.6 to -0.6 rad) | Sluice, gatehouse, and lift levers |
| `ladder_3m` | 120 | 0.6 × 3.0 × 0.1 | none | Ladders; stack per 3 m of climb |
| `reed_clump` | 256 | 0.8 × 2.0 × 0.8 | none | Mirrormere's Reed Marsh scatter |
| `spire_thread_pillar` | 1,512 | 1.4 × 6.0 × 1.2 | none | The Lattice Spire's woven columns |
| `spire_ramp_segment` | 168 | 2.5 × 2.0 × 3.9 | none | The Spire's outer spiral ramp (rises 1 m over 4 m) |
| `great_bell` | 328 | 6.2 × 8.0 × 6.2 | none | Rimeshelf's landmark, the Great Bell frozen in the cliff |
| `aqueduct_bay`, `aqueduct_bay_broken` | 216, 228 | 10.3 × 10.6 × 2.3 | none | The Verge's landmark arch; bays repeat every 8 m along X |

## Integration checklist

Integration is a separate phase that needs the owner's go-ahead. ADR 0008 is accepted. It
must do the following:

1. **Add the bake to the pipeline.** Either have `lf-tools assets` call
   `game/scripts/bake-vertex-colors.mjs` before `gltf-transform optimize`, or add a `bake`
   flag per entry in `game/assets-src/assets.json`. Characters (the Wayfarer and the NPCs)
   skip the bake, because armor tints edit their material colors (ADR 0005).
2. **Copy the chosen entries** from `game/assets-staging/assets.json` into
   `game/assets-src/assets.json`, and run `lf-tools assets` into `static/latticefall/`. Only
   copy the pieces a level uses; the 3.8 MB staged set is the upper bound.
3. **Add kit catalog entries.** Each piece needs an entry in a `game/data/kits/*.ron` file
   and an ID in `game/data/registry.ron` (kit pieces use the 10100s; the next free numbers
   follow `dead_tree` at 10153). We recommend one catalog per region: `verge.ron`,
   `mirrormere.ron`, `hollowdeep.ron`, `castle.ron`, `rimeshelf.ron`, `spire.ron`, and
   `hearth.ron`. The following example shows one entry:

   ```ron
   (
     id: "castle_wall",
     model: Some("kits/kenney-castle-kit/wall.glb"),
     tags: ["castle_wall"],
     colliders: [Box(center: (0.0, 1.31, 0.0), half_extents: (1.0, 1.31, 1.0))],
   ),
   ```

   Colliders are axis-aligned boxes only (a capsule bakes to its box; see `KitCollider` in
   `game/crates/content/src/level.rs`). Size each box from `models.tsv` times the piece's
   scale. Wall and floor pieces need full boxes; scatter (grass, flowers, reeds, small
   rocks, and mushrooms) needs none.
4. **Fix the scale.** The kit grid is 2 m (`docs/game/02-world.md`). Kit pieces have no
   scale field yet, so either add one or scale the files in the bake. The packs use these
   units:
   - **Kenney castle, graveyard, and fantasy town:** 1-unit tiles. Scale by 2, so a castle
     wall is 2 m wide and 2.6 m tall; stack two for curtain walls.
   - **Kenney nature:** trees are 1.2-1.5 units tall. Scale trees by 4-6 and rocks by 2-4.
   - **KayKit dungeon:** 4-unit tiles with 4-unit walls. Use them at scale 1 on a 2-cell
     (4 m) rhythm.
   - **KayKit weapons:** about twice the Wayfarer's scale (`sword_1handed` is 1.78 long).
     Scale by about 0.45 and match the grip to the procedural convention in
     `render/equipment/weapons.ts` (grip at the origin, blade along +Y).
   - **Procedural models:** already in meters; scale 1.
5. **Tint to region palettes.** The packs come in their authors' colors: Kenney's castle is
   sandstone tan, and its nature kit's leaves are teal (their material values are sRGB
   numbers stored as linear). Remap colors per region in the bake or in the kit material,
   using the palette table in [the presentation spec](../07-presentation.md#art-direction).
6. **Swap greybox props.** In `render/props/`, replace the greybox geometry with the
   procedural models (Lantern, lever, ladder, brazier, mirror, and mine cart) and drive
   their named nodes from the core's mechanics state words. The Lantern has no renderer at
   all, so it needs a small one that reads Lantern entities from the level manifest.
7. **Use the NPCs.** In `render/props/hearth.ts`, replace the greybox figures with the
   staged NPCs and play the UAL `Idle_Loop` (and `Interact` when talking) through the
   Wayfarer's retargeting. The proposed casting is `W_Medieval` as the Archivist, `Worker`
   as the Smith, `W_Adventurer` as the Cartographer, `Farmer` as the Courier, and
   `W_Witch` as the Peddler. Close issue 42 first.
8. **Check the budgets.** The frame budget is 600,000 triangles
   (`docs/game/01-architecture.md`, "Performance budgets"). The largest staged kit piece is
   2,173 triangles (`box_stacked`) and most are 30-700, so a room of 200 pieces stays near
   100,000. Each NPC is 5,200-6,900 triangles. No staged file has a texture, so the KTX2 and `toktx` path stays unused. Run
   the route benchmark (`/game?dev=1&engine=lf&bench=1`) after each region is dressed.

## Still missing

The following items have no staged model:

- **Region-specific ruins for the Verge** (the Ninefold Chapel and the windmill ruins):
  covered roughly by the fantasy town walls, castle walls, and Kenney ruined statues. The
  Quaternius Ultimate Modular Ruins pack would fit better; the owner can download it from
  https://quaternius.com/packs/ultimatemodularruins.html (Google Drive) and stage it.
- **Snow and ice dressing for Rimeshelf:** no CC0 flat-shaded snow kit was found. Tinting
  the nature kit's rocks, cliffs, and pines white and ice blue is the planned approach.
- **The Frozen Foundry's machinery** (furnace, crucible, and chimney): only the anvil and
  workbench are staged. Build these procedurally like the Great Bell.
- **Bows, spears, halberds, colossal weapons, and fist weapons:** no CC0 pack has
  flat-shaded meshes for them, so they stay procedural.
- **Armor attachments:** no CC0 flat-shaded attachment pack fits the Wayfarer's rig. The
  procedural attachments in `render/equipment/armor.ts` stay.
- **Hollowhearth's hollow-tree shell:** the room inside the giant tree's roots. Build it
  procedurally or from KayKit wood floors and Kenney trunks.
- **Fog gates:** greybox props in `render/props/` today. A fog gate reads as a VFX effect
  more than a model, so it belongs in the VFX pass, not a pack.
