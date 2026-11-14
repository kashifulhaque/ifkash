# Art direction research plan: the 2010s souls look

- **Status:** Planned, not started (GitHub issue 86). Start it once the [prerequisites](#prerequisites) are done (issues 84 and 85).
- **Raised:** 2026-11-15, by the owner
- **Replaces, if adopted:** the low-poly art direction in
  [the presentation spec](07-presentation.md#art-direction), and parts of
  [ADR 0008](decisions/0008-model-packs.md) (model packs) and
  [ADR 0009](decisions/0009-animation-library-2.md) (animation clips)

## The goal

The owner wants Cinderheart to move away from the low-poly, flat-shaded look toward a more
realistic 3D look, in the style of the souls-likes of the early 2010s: *Demon's Souls*
(2009) and *Dark Souls* (2011). The target is the look of those games, not of *Sekiro* or
*Elden Ring*.

What that look means in practice, as a starting point for the research to confirm:

- **Proportions:** Realistic human proportions and armor, not stylized or chunky.
- **Materials:** Textured surfaces with color, normal, and roughness maps: worn stone,
  rusted metal, cloth, and leather. No flat colors.
- **Geometry:** Mid-poly. As a first guess, about 10,000-25,000 triangles for the player,
  5,000-15,000 for a regular enemy, and 20,000-60,000 for a boss.
- **Light and mood:** Muted, desaturated palettes, heavy fog, dark interiors lit by a few
  warm sources, and strong silhouettes. The existing per-region palettes and accent colors
  can carry over.
- **Animation:** Weighty, grounded melee: long wind-ups, committed swings, and readable
  recoveries. The animations matter as much as the models for this look.

The look must stay inside what the game already promises: it runs in a browser, on the
site's free hosting, on a 2021 integrated-graphics laptop at 60 fps, and in touch mode on
phones.

## What the research must answer

The research ends with an ADR for the owner to approve. To write it, it has to answer
these questions, each with evidence (links, license text, measured numbers).

### 1. Where the art comes from

Agents can't model or animate by hand, so every asset comes from a pack, a library, or a
generator. For each candidate source, record the license, the price, what it covers, and
whether its style matches the others.

- **Characters and armor:** realistic humanoid models with a standard skeleton, armor
  sets, and NPCs.
- **Enemies:** 16 enemies and 5 elites (husks, hounds, knights, skeletons, and the rest
  in [the enemies spec](05-enemies-and-bosses.md)). Humanoids can share the player's
  skeleton; creatures need their own models and animations.
- **Bosses:** 12 bosses, several of them non-human (a stag, a colossus, a matriarch).
  This is likely the hardest and most expensive part.
- **Environments:** castle, cathedral, crypt, mine, cave, village, and swamp kits;
  rocks, cliffs, and trees; and tiling ground textures for the terrain.
- **Weapons:** the 14 weapon classes, shields and catalysts included.
- **Animations:** player melee for every weapon class, guard, parry, rolls, Arts, spells,
  enemy movesets, and boss movesets.

Sources to check include CC0 libraries (Poly Haven, ambientCG), CC-BY models
(Sketchfab), paid packs (for example Fab, the Unity Asset Store, and itch.io), Mixamo,
motion-capture libraries, and AI 3D and animation generators. For each one, check
whether its license allows serving the raw model files from a public website. Some
licenses allow use "in a game" but forbid redistributing the files, and a web game
serves its files to every visitor. Mixamo and engine-store licenses need a careful
reading here.

### 2. Whether the browser can draw it

- **Renderer:** physically based materials in Three.js, real-time shadows, fog, and
  post-processing (ambient occlusion, bloom, color grading). Decide between baked lighting
  (lightmaps) and real-time lighting for interiors. Check whether WebGPU is worth it.
- **Textures:** the compressed-texture pipeline (KTX2), which the asset tool already
  half-supports but which needs `toktx` installed (see
  [ADR 0005](decisions/0005-asset-pipeline-tools.md)). Set the texture sizes per quality
  preset.
- **Budgets:** the draw-call, triangle, and memory budgets in
  [the architecture doc](01-architecture.md#performance-budgets) were set for low-poly
  art. The download budget (first region under 8 MB, the whole game under 60 MB) will
  almost certainly break. Propose new numbers, and measure them.
- **Loading:** streaming each area's models and textures as the player reaches it, with a
  loading screen or a seamless fade.
- **Phones:** what touch mode drops (shadows, texture size, effects) to stay playable.

### 3. How the animations work

- Pick one skeleton standard for every humanoid (player, NPCs, humanoid enemies), so one
  animation set serves all of them. The renderer already retargets clips by bone name
  (`render/actors/retarget.ts`), and the move-to-clip timing layer
  (`render/actors/moveClips.ts`) carries over with new clip names.
- Find melee animation sets with the weight of the target games, and check how much
  retiming they need to fit the frame data, which never changes to fit an animation.
- Decide how creatures and bosses get animated.
- The owner judged the current Universal Animation Library clips as not good enough
  (2026-11-15). The new set must be tested on a real pad before it's adopted.

### 4. What it costs and how long it takes

- Money: the total for any paid packs or tools.
- Download size per area and for the whole game.
- Work: how many work packages the migration takes, and in what order.

## What the research delivers

1. A reference board: screenshots from the target games and candidate assets, with notes
   on what to match (fog, color, armor detail, animation weight).
2. A source shortlist with license evidence for each source.
3. A budget table with the proposed new numbers.
4. A prototype, on a branch and off by default: Waking Hollow's first rooms with the new
   Wayfarer, one enemy, the terrain, and one interior, measured against the budgets on the
   target laptop and a phone. The owner plays it with a pad.
5. An ADR that proposes the new art direction, the sources, and the budgets, for the
   owner to approve.
6. Work package cards for the migration, in order.

## Prerequisites

Some work is useful in any art style, and the research and prototype need it. Do these
first:

1. **Load real model files into levels.** Today, levels draw boxes and code-built stand-ins
   (`render/kits/`). The renderer needs to load model files for level pieces, props, the
   Lantern, and NPCs, and stream them per area. This is the engine work that
   [ADR 0008](decisions/0008-model-packs.md) left as "a separate work package".
2. **The textured-asset pipeline.** Install `toktx`, compress textures to KTX2 in
   `lf-tools assets`, and serve the Basis transcoder from `static/latticefall/basis/`.

Don't do this, because the new look would throw it away:

- Dressing every region with the low-poly Kenney and KayKit packs from ADR 0008. One
  test area, to prove prerequisite 1 works, is enough.
- More animation polish on the Universal Animation Library clips. Animation should start
  after the research picks the character model and its skeleton.

## What changes if it's adopted

- The art direction section of [the presentation spec](07-presentation.md) is rewritten.
- The Wayfarer model, the armor, the NPCs, and the weapon models are replaced.
- The procedural enemies, elites, and 12 bosses (`render/procedural/`,
  `render/bosses/`) are replaced by models, or kept for some creatures if the research
  finds no source.
- The low-poly packs in ADR 0008 and the clips in ADR 0009 are superseded where the new
  sources cover them.
- Nothing in the simulation changes: the core, the frame data, the golden replays, and the
  RL environment don't depend on how the game looks.
