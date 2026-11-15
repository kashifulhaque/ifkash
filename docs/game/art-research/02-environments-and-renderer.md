# Art research 02: environments, textures, and the renderer

- **Answers:** question 1 (environments and textures only) and question 2 of
  [the art direction research plan](../art-direction-research.md)
- **Date checked:** 2026-11-15, for every license, limit, and version in this document
- **Method:** desk research only. Nothing was downloaded, bought, or run, and no asset was
  viewed as an image. Style-fit ratings come from catalog names, descriptions, and what
  each library is known for, so the prototype has to confirm them by eye.

Each claim carries one of these evidence labels:

- **Read:** I read the license page, the documentation page, or the code on the date checked.
- **Reported:** a news article, forum post, or third-party page says so.
- **Estimate:** my own arithmetic or judgment. Nothing measured.

## Recommendation

**Sources.** Build the environments from CC0 material only (CC0 is a public-domain
dedication: anyone can copy and republish the files, with no credit needed):

- **Walls, floors, arches, stairs, towers:** modular kit pieces built by script (the
  existing `render/kits/pieceGeometry.ts` approach, or headless Blender Python for bevels
  and chipped edges), covered with tiling scanned textures from Poly Haven and ambientCG.
  *Demon's Souls* and *Dark Souls* architecture is mostly simple geometry with good
  textures and lighting, which is the one kind of modeling that agents can do.
- **Rocks, cliffs, dead trees, roots, and props:** Poly Haven's scanned models, simplified
  by the asset pipeline.
- **Gaps** (gothic ornament, bones, coffins, a mine cart, fungi): single CC0 or CC-BY
  models checked one by one, and AI-generated props from a paid plan if the owner approves.

No paid environment kit is recommended. Every store license I read (Fab, Unity Asset
Store, Sketchfab Standard, Synty) forbids putting the files in a public repository, and
Fab requires the game to stop players from extracting them, which a web game that serves
plain model files doesn't do.

**Lighting.** Outdoors: one real-time sun with two shadow cascades, plus the existing
hemisphere light. Interiors: bake the lighting into lightmaps with a headless Blender
script, and use a small fixed pool of unshadowed point lights for flicker and for lighting
characters. The baked pipeline is the largest technical unknown, so the prototype must
build it for one room before the ADR commits to it.

**Post-processing.** Low: none. Medium: tone mapping, a per-region color grade, and
half-resolution bloom. High: the same plus half-resolution ambient occlusion (N8AO).

**WebGL or WebGPU.** Stay on `WebGLRenderer`. Upgrade Three.js from r182 to r187 first,
because r186 added cascaded sun shadows and a light-probe grid for WebGL. Revisit WebGPU
later, when a measured bottleneck needs it.

**Budgets (medium preset).** 350 draw calls, 1.2 million triangles per frame with shadow
passes, 128 MB of GPU texture memory, 20 MB for the first area, about 12 MB for each
later area, and about 200 MB for the whole game, streamed by area. The low preset keeps
the old download numbers (8 MB and 60 MB) by using 512-pixel color textures without
normal maps.

**Hosting.** Keep the prototype on Cloudflare Pages. For the full migration, serve models
and textures from an R2 bucket on a custom subdomain, and keep them out of the Git
history.

**Price.** USD 0 for the recommended environment sources and tools. Optional: one month
of an AI model generator's paid plan (about USD 20, not verified) for gap props.

**Biggest risks.**

1. Script-built kits look too plain, and no CC0 source fills the gothic-ornament gap.
2. The lightmap bake pipeline is fragile without an artist to check each bake.
3. Frame time on integrated graphics: every per-effect cost in this document is an
   estimate until the prototype measures it.

## Shortlist

All prices are in US dollars. The verdict columns answer two questions: (a) can the
converted files (GLB models with KTX2 textures) be served from a public website, and
(b) can they be committed to a public GitHub repository.

### Tiling textures (ground, walls, wood, metal)

The following table lists the texture sources:

| Source | Covers | Style fit | Sizes | Price, account | (a) Web | (b) Repo |
|---|---|---|---|---|---|---|
| [Poly Haven textures](https://polyhaven.com/textures) | 869 scanned materials (Read, catalog API): 91 brick, 81 rock, 39 plaster, 36 plank, 26 cobblestone, 16 rust, 8 snow, and 18 named castle, church, or medieval | Good: photo-scanned, worn, European stone and brick | 1K to 8K source, with color, normal, roughness, AO, and height | Free, no account | Allowed | Allowed |
| [ambientCG](https://ambientcg.com) | About 2,040 materials (Read, API count), 61 foliage and debris atlases, 127 decals | Good: scanned and procedural, wide range of stone, ground, and metal | 1K to 8K source | Free, no account | Allowed | Allowed |
| [ShareTextures](https://www.sharetextures.com) | Scanned materials and some models | Good | Up to 4K | Free | Allowed | Unclear: its "custom CC0" forbids redistribution in collections |
| OpenGameArt, the Yughues packs | 75 free material packs and scanned rocks | Partial: older, mixed quality | Mixed | Free | Unclear: listed as CC0, pack pages not confirmed | Unclear |

### Rocks, cliffs, trees, and foliage

The following table lists the nature sources:

| Source | Covers | Style fit | Triangles | Price, account | (a) Web | (b) Repo |
|---|---|---|---|---|---|---|
| [Poly Haven models](https://polyhaven.com/models) | 37 rocks and cliffs, 20 trees, 57 plants, stumps, roots, moss (Read, catalog API) | Good: photo-scans. Several sets are South African coast and desert, so pick by shape | Rocks 6,000 to 200,000; coastal cliffs 0.9 to 2.9 million; trees 150,000 to 17 million. All need simplifying | Free, no account | Allowed | Allowed |
| Quixel Megascans on Fab | The largest scanned library: rocks, cliffs, ruins, plants | Good | Several levels of detail each | Free since 2026-11-06 (Reported); Epic account | Unclear: must restrict extraction | Not allowed |
| ambientCG atlases | 61 leaf, grass, and debris atlases for alpha cards | Good | Flat cards | Free | Allowed | Allowed |
| Quaternius and Kenney nature packs | Trees, rocks | Poor: stylized low-poly | Low | Free | Allowed | Allowed |

### Modular kits (castle, cathedral, crypt, mine, cave, village, swamp)

The following table lists the kit sources:

| Source | Covers | Style fit | Detail | Price, account | (a) Web | (b) Repo |
|---|---|---|---|---|---|---|
| Script-built kit with CC0 textures (this repo) | Any shape an agent can describe in code: walls, arches, vaults, stairs, towers, timber frames, mine supports | Partial to good: right for plain masonry, weak for ornament | 200 to 2,000 triangles a piece, by design | Free | Allowed | Allowed |
| [Dungeon Building Kit on OpenGameArt](https://opengameart.org/content/dungeon-building-kit) | Halls and rooms on a 4 m grid, with collision meshes (Read) | Unknown: not viewed. Textures are only 512 pixels | 31 MB Blender asset library | Free, no account | Allowed | Allowed |
| Fab medieval and dungeon kits (free and paid) | Many castle, crypt, and village kits | Good, for the realistic ones | Varies | Free to about 100 | Unclear: must restrict extraction | Not allowed |
| Unity Asset Store kits | Many | Good, for the realistic ones | Varies | About 10 to 100 | Unclear: "embedded" only | Not allowed |
| Sketchfab Standard-license kits | Some | Varies | Varies | Varies | Not allowed as plain files | Not allowed |
| KayKit Dungeon, Kenney, Quaternius kits | Dungeon, castle, village | Poor: stylized, gradient-atlas texturing | Low | Free | Allowed | Allowed |
| Synty packs | Dungeon, fantasy kingdom | Poor: flat-shaded stylized | Low | About 50 to 300 | Allowed by wording | Not allowed |
| Heritage scans (single CC0 or CC-BY models of real crypts, churches, ruins) | One-off landmarks, not kits | Good, but each is one fused mesh of millions of triangles | Needs heavy simplifying | Free; account to download | Per model | Per model |

### Props (doors, chests, braziers, statues, barrels, bones, banners)

The following table lists the prop sources:

| Source | Covers | Style fit | Triangles | Price, account | (a) Web | (b) Repo |
|---|---|---|---|---|---|---|
| Poly Haven models | `large_castle_door`, `large_iron_gate`, `treasure_chest`, `wine_barrel_01`, `wooden_barrels_01`, crates, buckets, lanterns, chandeliers, `gothic_statue`, `marble_bust_01`, `kite_shield`, `stone_fire_pit`, gothic furniture, worn bookshelf (Read) | Good | 1,000 to 40,000; the chest is 103,000 | Free | Allowed | Allowed |
| Script-built props | Braziers, bells, urns, pillars (lathe shapes), banners (cloth planes), ladders, rails | Good for simple shapes | 100 to 1,500 | Free | Allowed | Allowed |
| CC-BY models (Sketchfab, Fab's CC-BY listings, Poly Pizza) | Bones, skulls, coffins, statues, carts | Varies; check each | Varies | Free; account | Allowed with credit | Allowed with credit |
| Smithsonian Open Access and museum scans | Statues, busts, reliefs | Good for statues | Millions; needs simplifying | Free | Unclear: FAQ page blocked my request | Unclear |
| AI model generators (Meshy and similar) | Any single prop from a text or image prompt | Partial: usable at mid-poly after cleanup, messy topology, lighting baked into color | 5,000 to 50,000 after remesh | Free tier, or about 20 a month | Allowed with conditions | Allowed with conditions |

## License evidence

Every quote is from the page linked, read on 2026-11-15 unless marked otherwise.

**Poly Haven: allowed for both.** The license page says:
"Our assets are all licensed as CC0, which is effectively Public Domain". It also states
that the assets can be redistributed. The site terms forbid scraping, so download through
the normal pages or the public API. See [the Poly Haven license](https://polyhaven.com/license).

**ambientCG: allowed for both.** The license page says:
"You can include the raw files in your project, for example a video game."
See [the ambientCG license](https://docs.ambientcg.com/license/).

**Fab Standard License, which covers Megascans and most Fab kits: (a) unclear, (b) not
allowed.** I read the full agreement text, dated October 1, 2024, at
[the Fab EULA](https://www.fab.com/eula).

- Section 4(c) lets a game ship with the content, then adds:
  "you must restrict end users from extracting or otherwise using Content outside".
  A browser game that serves ordinary GLB and KTX2 files doesn't restrict anything. Packing
  the files in a custom container might satisfy the clause, but that's a legal judgment,
  and the unpacking code would be public in this repo.
- Section 5(a) allows sharing the files with collaborators only, and gives a private
  repository as its example. A public repository is distribution "on a standalone basis".
- The license isn't limited to Unreal Engine. The summary on the same page says usage "is
  not limited to Unreal Engine".
- Megascans price (Reported): CG Channel wrote on 2026-11-06 that
  "The full Quixel Megascans library is going free." and that the license is unchanged.
  I couldn't load a Fab listing to confirm the price.
  See [the CG Channel article](https://www.cgchannel.com/?p=179945).

**Fab CC-BY listings and Sketchfab Creative Commons models: allowed with credit, per
model.** CC BY 4.0 grants the right to
"reproduce and Share the Licensed Material, in whole or in part". It requires credit to
the author and forbids adding technical locks. `lf-tools credits` already handles credits.
CC BY-SA models add a share-alike rule, and CC BY-NC models forbid commercial use; avoid
both. See [the CC BY 4.0 legal code](https://creativecommons.org/licenses/by/4.0/legalcode.en).

**Sketchfab Standard and Editorial licenses: not allowed.** Section 2.2(b) forbids
offering the model "as a stand-alone file (or group of files)". See
[the Sketchfab license page](https://sketchfab.com/licenses). Sketchfab moved from Epic
to KitBash in August 2026 (Reported), and I couldn't confirm that free Creative Commons
downloads still work.

**Unity Asset Store: (a) unclear, (b) not allowed.** The EULA, last updated December 4,
2024, permits distribution only "as incorporated and embedded in that Licensed Product"
(section 2.2.1(b)), and forbids any distribution it doesn't expressly permit (section
2.2.1.1(d)). I found no engine restriction for ordinary assets. See
[the Unity Asset Store terms](https://unity.com/legal/as-terms).

**Synty: (a) allowed by wording, (b) not allowed, and the style is wrong.** The license,
dated 9 July 2026, says: "You must not share the source files of any Assets outside your team."
See [the Synty license](https://syntystore.com/pages/end-user-licence-agreement).

**Kenney, Quaternius, KayKit: allowed for both, wrong style.** Kenney's support page says
"all game assets on the asset pages are public domain licensed (CC0)". Quaternius's FAQ
says "All models are under the CC0 License." The KayKit Dungeon page marks the pack as CC0.
See [Kenney support](https://kenney.nl/support),
[the Quaternius FAQ](https://quaternius.com/faq.html), and
[the KayKit Dungeon pack](https://kaylousberg.itch.io/kaykit-dungeon-remastered).

**OpenGameArt: per asset.** The Dungeon Building Kit page reads "License(s): CC0". The
site's FAQ warns that CC-BY art must be distributed "in a way that does not include DRM",
which a plain web game satisfies. Avoid GPL and CC-BY-SA art. See
[the OpenGameArt FAQ](https://opengameart.org/content/faq).

**ShareTextures: (a) allowed, (b) unclear.** The license page calls its terms a custom
CC0 and lists: "No asset redistribution on other websites, in plugins, or as part of collections".
A public repo full of its textures could count. Poly Haven and ambientCG cover the same
ground without the question. See
[the ShareTextures license](https://www.sharetextures.com/p/license).

**itch.io: per asset.** itch.io has no site-wide asset license. Each seller writes their
own terms, and many forbid redistribution. Read each page before use.

**AI model generators: allowed with conditions.**

- Meshy (Read, help center): "on a free plan, we grant you a CC BY 4.0 license", and
  paid customers own their output. See
  [Meshy's commercial-use article](https://help.meshy.ai/en/articles/9992001-can-i-use-my-generated-assets-for-commercial-projects).
- Tripo (Reported): its own pages disagree on whether free output allows commercial use.
- TRELLIS.2 from Microsoft (Reported): MIT-licensed, runs on your own GPU with about
  24 GB of video memory.
- Hunyuan3D 2.1 from Tencent (Reported): its community license excludes the European
  Union, the United Kingdom, and South Korea.

The presentation spec says not to use AI-generated assets whose license is unclear. A
paid Meshy plan is the clearest of the four, and it needs an owner account, a
subscription, and an ADR.

## Coverage

*Kit* means the script-built kit with CC0 textures. *PH* means Poly Haven. Names in code
font are Poly Haven asset IDs.

| Area | Needs | Proposed sources | Gaps |
|---|---|---|---|
| Hollowhearth (hub) | Hollow-tree interior, candles, shelves, furniture | Bark and wood textures (PH has 23 bark, 138 wood); `root_cluster_01`, `pine_roots`; `wooden_bookshelf_worn`, gothic furniture, `wooden_candlestick`, lanterns | The tree shell itself: script-built with bark textures |
| The Verge | Ash meadow, dead trees, aqueduct, chapel ruins, windmill, tutorial cave, briars | Ground and dry-grass textures; `dead_tree_trunk`, `dead_tree_trunk_02`, `dry_branches_medium_01`, stumps; kit arches with `medieval_blocks_*`; `rock_face_01`, `rock_07` | Only two dead-tree scans. Windmill. Thorn bushes |
| Mirrormere | Lake shore, flooded village, reed marsh, causeway, cathedral | Kit timber-frame houses with plaster (33), roofing (23), and plank (36) textures; `modular_wooden_pier`; reed and grass atlases from ambientCG | **Gothic cathedral ornament** (tracery, vault ribs, statues). Rowboat. Reeds as models |
| Hollowdeep | Fungal caverns, glowing lake, mine, crystals, Lamp Tree | Rock textures (81) on script-built cave shells; `moss_01`, `fern_02`; kit mine supports with wood and rust textures | Giant fungi and the Lamp Tree: keep them procedural and emissive. Mine cart |
| Castle Graymantle | Curtain walls, towers, gatehouse, sea cliffs, banners, armory | Kit with `castle_brick_*`, `castle_wall_slates`, `medieval_wall_*`; `large_castle_door`, `large_iron_gate`; `coastal_cliff_*` and `coast_rocks_*` simplified; barrels, crates, `kite_shield` | Banners (cloth planes with a fabric texture). Statues and heraldry |
| Rimeshelf | Snow highland, foundry, frozen bell, standing stones | Snow textures (8); rocks; kit foundry with rust and metal textures; script-built bell and braziers | Ice material (a shader, not a texture). Anvils and forge props |
| The Lattice Spire | Tower of woven light | Stays procedural and emissive; no pack matches it | None from packs, by design |
| Mossgut Cave, Glasswing Grotto | Low cave, moss; crystal cave | Cave shells, moss textures (11), roots; procedural crystals | Cave walls need a displaced-rock generator |
| Sunken Ossuary, Whitewarden Catacombs | Crypt halls, niches, braziers | Kit vaults and niches; `stone_fire_pit`; possibly the OpenGameArt dungeon kit | **Bones, skulls, coffins, sarcophagi** |
| Drowned Belfry | Sunken ruin, four bells | Kit ruin pieces; script-built bells | None large |
| Cinderdeep Mine | Tunnels, rails, carts | Kit supports and rails | Mine cart |
| The Bound Ring | Standing stones | PH rocks, scaled and stood upright | None |

The three largest gaps are gothic ornament, bones and coffins, and fungi. Fungi can stay
procedural. The other two need single CC0 or CC-BY models, or AI generation.

## Renderer findings

### What the renderer does today

This is from reading `src/lib/latticefall/render/` and `engine/scene.ts` (Read):

- **Three.js version:** r182 is installed. The latest release is r187, from 30 October.
- **Materials:** `MeshStandardMaterial` is already the main material (66 uses), with flat
  colors and vertex colors instead of texture maps. `MeshBasicMaterial` covers the sky
  and effects.
- **Lights:** one hemisphere light, one directional sun, and a few unshadowed point lights
  on hearth and brazier props. There's no environment map, so metals have nothing to
  reflect.
- **Shadows:** one sun shadow map, 2048 pixels on medium and 4096 on high, covering a
  92 m square around the player, with `PCFSoftShadowMap`. Low has no sun shadow.
- **Fog:** Three's linear distance fog, plus a height-fog patch added to materials with
  `onBeforeCompile` (`lighting/fogPatch.ts`).
- **Post-processing:** none. The renderer draws straight to the canvas with built-in
  anti-aliasing, and no tone mapping is set. The preset's `postProcessing` field is unread.
- **Geometry:** kit pieces draw through one `BatchedMesh` with a single shared material;
  scatter uses `InstancedMesh`; `perf/culling.ts` hides instances by distance; terrain
  chunks drop resolution by distance.
- **Perf tooling:** `perf/bench.ts` flies a fixed route through 17 areas on medium and
  reports frame work in milliseconds (JavaScript plus GPU, forced with `gl.finish()`),
  draw calls, and triangles. `perf/textures.ts` estimates GPU texture memory against a
  96 MB budget.

### Physically based materials on integrated graphics

*Physically based rendering* (PBR) describes a surface with texture maps instead of one
color. The standard set is a *color* map, a *normal* map (fakes small bumps so a flat
wall catches light like rough stone), and an *ORM* map (three grayscale maps packed into
one image: ambient occlusion, roughness, and metalness).

- The game already pays for the PBR lighting math, because it already uses
  `MeshStandardMaterial`. Adding the three maps adds three texture reads and the normal-map
  math for each pixel. Estimate: 10% to 25% more time in the main pass.
- The larger costs are elsewhere: each extra light is computed for every pixel on screen,
  each shadowed light adds shadow lookups, and overlapping surfaces get shaded more than
  once.
- Metals need an environment map to look like metal. One small prefiltered cube map for
  each region costs one more texture read (Estimate: under 0.3 ms).
- I found no published measurement of `MeshStandardMaterial` on Intel Iris Xe. Every
  number here is an estimate for the prototype to replace.

**Recommendation:** Use `MeshStandardMaterial` with color, normal, and ORM maps on medium
and high, plus one small environment map for each region. On low, use
`MeshLambertMaterial` with the color map only. That also removes the normal maps, the
largest files, from the low preset's download. The original games predate PBR, so the
simpler material isn't a loss of style.

### Shadows and baked lighting

A *shadow map* is a picture of the scene drawn from the light's point of view, redrawn
every frame, so it costs a second pass over the geometry. *Cascades* split the sun's
shadow into a sharp near map and a coarse far map. A *lightmap* is a texture that stores
lighting computed ahead of time, including soft shadows and bounced light, so it costs
one texture read at run time.

**Outdoors.**

- Two cascades at 2048 pixels are enough for the 4.5 m camera. The only measured figure I
  found is from a Three.js open-world dev diary: three cascades at 1024 pixels in under
  2 ms of GPU time on unnamed target hardware (Reported). See
  [the Cinevva shadow spike write-up](https://app.cinevva.com/blog/2026-02-25-open-world-browser-part-05-budgeting-the-pretty).
- r186 added a `SunLight` addon with cascaded shadows for `WebGLRenderer` (Read, release
  notes and docs). On r182 the older `CSM` addon does the same job.
- r186 removed the `PCFSoftShadowMap` code, and the game uses that setting, so the
  upgrade must switch to `PCFShadowMap` (Read, release notes).

**Interiors.**

- Real-time only: `WebGLRenderer` computes every light for every pixel, and a shadowed
  point light redraws the scene six times. A budget of four unshadowed point lights and
  at most one shadowed spot light is realistic (Estimate). Unshadowed lights shine
  through walls into the next room, and there's no bounced light, so rooms look flat.
- Baked lightmaps: soft shadows and bounce at almost no run-time cost, which is how the
  target games lit their interiors, and the cheapest option for phones.
- What a bake needs with no artist:
  1. Assemble the area's static pieces in Blender from the level data, by script.
  2. Give every placed piece its own rectangle in a second UV layout (Blender's lightmap
     pack operator).
  3. Bake diffuse lighting with Cycles, denoise it, and write one 1K or 2K image for each
     interior.
  4. Export the image and each piece's rectangle. The renderer reads the rectangle for
     each instance, so pieces stay instanced. This needs a shader patch on the batched
     material.
- Is it realistic for agents? Yes, with risk. `blender --background --python` runs with
  no window, and every step is scriptable. Blender isn't installed on this machine. The
  risks are seams and light leaks that only show in a screenshot, and state that changes
  during play, such as a brazier the player lights. Bake those lights as off and add a
  real-time point light when lit.
- A middle option arrived in r186: `LightProbeGridWebGL`, a 3D grid of light samples that
  the browser bakes on the GPU, with one bounce and incremental baking (Read, docs). It
  needs no Blender and no second UV set, and it lights moving characters too. It has no
  shadows of its own, coarse grids leak light through walls, and its bake time on
  integrated graphics is unknown.

**Recommendation:** Outdoors, use a real-time sun with two cascades on medium and high, and
blob shadows under characters on low. For interiors, bake lightmaps with headless Blender
and keep a fixed pool of four unshadowed point lights (two on low) for flicker and
characters. In the prototype, build the same room twice, once with a lightmap and once
with the probe grid, and keep the probe grid as the fallback if the bake pipeline proves
too fragile.

### Fog and post-processing

*Post-processing* means drawing the scene to an off-screen image and then running
full-screen filters over it. Each filter costs time in proportion to the pixel count.

**Fog.**

- Fog computed inside each material, which the game does already, is nearly free. Switch
  the curve to exponential height fog and tint it toward the sun's color when looking
  sunward. That costs a few arithmetic operations for each pixel.
- Cheap volumetric tricks: soft fog cards near the ground, additive cone meshes for light
  shafts at windows, and glow sprites on flames. Keep fog cards to about two layers of
  screen coverage, because each layer is shaded again.
- True volumetric fog (ray marching) is too expensive for integrated graphics.

**Post-processing costs.** The following table gives estimates at 1080p on a 2021
integrated GPU. None are measured:

| Effect | What it does | Cost (Estimate) | Preset |
|---|---|---|---|
| Tone mapping (AgX) | Maps bright values to the screen without clipping; AgX desaturates highlights, which suits a muted look | About 0 without other effects; inside the output pass otherwise | All |
| Off-screen target and output pass | Needed once any filter is on | 0.5 to 1 ms | Medium, high |
| Color grade | Region tint, contrast, and saturation as a few numbers, in the output pass | Under 0.2 ms | Medium, high |
| Bloom at half resolution | Glow around flames and Lanterns | 0.7 to 1.5 ms | Medium, high |
| Ambient occlusion, N8AO at half resolution | Darkens creases and corners | 1.5 to 3 ms | High only |
| Ambient occlusion at full resolution (N8AO or `GTAOPass`) | The same, sharper | 4 to 8 ms | Never |
| Anti-aliasing, 4x multisampling | Smooths edges | 10% to 20% of the main pass | Medium, high |
| FXAA | Cheaper edge smoothing, blurs slightly | 0.3 to 0.5 ms | Optional on low |

- N8AO's author states that half-resolution mode is two to four times faster than full
  resolution (Reported, from a search summary of the project README). The Three.js docs say `GTAOPass` costs more than
  `SSAOPass` (Read). I found no millisecond figures for either on integrated graphics.
- On medium, the ORM map's baked occlusion and the lightmaps already darken creases, so
  screen-space ambient occlusion isn't needed there.
- A lookup-table (LUT) grade gives finer control than three numbers, for one more texture
  read. Add it only if the simple grade isn't enough.

**Recommendation:** Low: fog and tone mapping inside the materials, no off-screen pass.
Medium: add the off-screen pass with bloom and the color grade. High: add half-resolution
N8AO. Target frame on medium (Estimate): 3 ms of JavaScript, 2 ms of shadows, 7 ms for the
main pass, 2 ms of post-processing, and about 2.5 ms spare.

### WebGPU

*WebGPU* is the newer browser graphics API. Three.js has a separate `WebGPURenderer`,
and materials for it are written in TSL, Three's shading language, instead of the shader
patches this game uses.

- **Browser support (Read, the WebGPU implementation status page):** Chrome and Edge 113
  and later on Windows, macOS, and ChromeOS; Chrome 121 and later on Android 12 and later;
  Chrome on Linux only for some GPUs. Firefox 141 and later on Windows and 147 on Apple
  Silicon Macs, but not on Linux or Android. Safari 26 on macOS, iOS, and iPadOS. Can I
  Use reports about 87% of global usage. See
  [the WebGPU implementation status page](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status).
- **Fallback:** phones on iOS 18 and earlier, and Firefox on Linux and Android, don't have
  it. `WebGPURenderer` falls back to WebGL 2 on those.
- **What it would give this game:** clustered lighting (many point lights for almost the
  cost of a few), temporal anti-aliasing, and newer ambient occlusion and
  global-illumination nodes.
- **What it would cost:** rewriting the height-fog patch and the six `ShaderMaterial`
  uses in TSL, replacing the post-processing stack, and absorbing API changes. The
  migration guide shows renames or removals in every release from r183 to r187 (Read).
  Forum threads from 2025 report `WebGPURenderer` running slower than `WebGLRenderer` for
  scenes with many separate meshes (Reported, thread titles only).

**Recommendation:** Later, not now. Baked interior lighting removes the main reason to
want it. Upgrade Three.js to r187 on `WebGLRenderer` in its own work package before the
prototype, reading the migration guide for r183 to r187. Revisit WebGPU in an ADR when
the bench shows a bottleneck that only WebGPU fixes.

### Textures

*KTX2 with Basis Universal* is a texture file format that stays compressed in GPU memory.
It has two modes: *ETC1S* (small files, visible blockiness on smooth gradients) and
*UASTC* (about four times larger, close to the original). *Texel density* is how many
texture pixels cover one meter of a surface.

**Mode for each map type.**

| Map | Mode | Why |
|---|---|---|
| Color | ETC1S | Stone and wood hide its artifacts |
| Normal | UASTC with rate-distortion optimization and Zstandard | ETC1S makes lighting look blocky |
| ORM | UASTC at half the set's resolution | Its three channels are unrelated, which ETC1S handles badly; roughness needs less resolution |
| Lightmap | UASTC | Smooth gradients band under ETC1S |
| Foliage color with alpha | ETC1S | Small, and alpha edges are forgiving |

**Texel density.** At 1080p with the camera 4.5 m behind the player, one meter of wall
at that distance covers about 210 screen pixels, and about 470 at 2 m. A 1K texture
tiled every 2 m gives 512 pixels a meter, which is as sharp as the screen can show in
normal play. The target games shipped at 720p with textures of 512 to 1K pixels. Larger
textures would be wasted.

| Preset | Target density | Texture set |
|---|---|---|
| Low | 256 pixels a meter | 512 color only |
| Medium | 512 pixels a meter | 1K color, 1K normal, 512 ORM |
| High | 512 pixels a meter; 1,024 for terrain and about four hero surfaces | 2K color, 2K normal, 1K ORM for those; medium sets for the rest |

**Size of one texture set.** All figures are estimates. The KTX2 work package can replace
the download sizes with measured ones. GPU sizes follow from the format: 4 bits a pixel
for ETC1S without alpha, 8 for UASTC, and one third more for *mipmaps* (the smaller
copies used at a distance).

| Set | Download | GPU memory |
|---|---|---|
| 512 color only | About 0.08 MB | 0.17 MB |
| 1K (color 0.2, normal 0.7, ORM 0.2) | About 1.1 MB | 0.67 + 1.33 + 0.33 = 2.3 MB |
| 2K (color 0.7, normal 2.8, ORM 0.7) | About 4.2 MB | 2.67 + 5.33 + 1.33 = 9.3 MB |

On GPUs that only support ASTC, ETC1S textures can take 8 bits a pixel, which raises a 1K
set to about 3 MB.

**Recommendation:** Use the modes and sizes in these tables. Ship each texture as a
separate file for each tier, so a low-preset player never downloads normal maps. In the
prototype, compare an ETC1S normal map with a UASTC one on a stone wall. If ETC1S is
acceptable, the 1K set drops from about 1.1 MB to about 0.5 MB.

### Geometry

*Instancing* draws many copies of one mesh in a single *draw call* (one instruction to
the GPU; each has a fixed CPU cost). A *level of detail* (LOD) is a simpler copy of a
mesh, used at a distance.

- **Batching:** `BatchedMesh` draws any mix of pieces that share one material in one
  call. Textured kits therefore cost one call for each material in each pass, not for each
  piece. Design each area's kit around about a dozen materials. r187 fixes a `BatchedMesh`
  sort corruption and a memory leak (Read, release notes).
- **LODs:** Generate two simplified levels for rocks, trees, statues, and large props
  with `gltf-transform simplify`. In a batch, switch an instance's geometry by distance
  with `setGeometryIdAt`, next to the existing distance culler.
- **Scans need more than simplifying.** A coastal cliff of 2 million triangles reduced to
  3,000 loses its surface detail unless the detail is re-baked into a normal map. That's
  another headless Blender step. Start with scans under 200,000 triangles, which simplify
  well with their own maps.
- **Compression:** Keep meshopt, which the pipeline and the loader already use. Draco
  makes static meshes somewhat smaller, but it decodes slower, needs its own decoder
  download, and doesn't compress animation.
- **Culling:** `BatchedMesh` already skips instances outside the camera's view, and the
  distance culler handles range. Three.js has no occlusion culling, so interiors would
  draw rooms hidden behind walls. Split interior batches by room and rely on fog and a 40
  to 60 m interior draw distance first. Add baked room-to-room visibility lists only if
  the bench shows a problem.
- **Piece budgets (Estimate):** wall module 200 to 800 triangles, pillar or arch 500 to
  2,000, door 1,000 to 3,000, small prop 300 to 800, rock 500 to 2,000, tree 3,000 to
  8,000, statue 3,000 to 8,000.

**Recommendation:** One `BatchedMesh` for each material for each area, two generated LODs
for scanned and organic assets, meshopt compression, and room-split batches with fog-based
culling indoors.

### Loading and hosting

- **Cloudflare Pages limits (Read):** the docs state
  "The maximum file size for a single Cloudflare Pages site asset is 25 MiB." The free
  plan allows 20,000 files for each site. The page lists no bandwidth limit. See
  [the Pages limits page](https://developers.cloudflare.com/pages/platform/limits/).
- **R2 free tier (Read):** 10 GB of storage, 1 million write-class and 10 million
  read-class requests a month, and no egress fee. Public access through `r2.dev` is
  "rate-limited and should only be used for development purposes", so production needs a
  custom domain, which also enables Cloudflare's cache. See
  [R2 pricing](https://developers.cloudflare.com/r2/pricing/) and
  [R2 public buckets](https://developers.cloudflare.com/r2/buckets/public-buckets/).
- **Cache rule needed (Read):** Cloudflare caches only a fixed list of file extensions by
  default. `.bin` is on the list; `.glb` and `.ktx2` aren't. Add one rule that caches
  everything on the asset hostname. The free plan caches files up to 512 MB.
- **Size check:** three texture tiers and the models come to roughly 0.5 GB and a few
  thousand files (Estimate). That fits both hosts. The problem with Pages is Git: Pages
  deploys from the repository, so every re-export of a texture stays in the public
  history forever.
- **Streaming:** each area already has its own manifest. Add an asset list with a file
  for each tier. Load an area's geometry and 512 textures first, show the area, and swap
  in the 1K textures as they arrive. Preload the next area while the player is near its
  door, or riding its lift. Drop the textures of areas left behind when GPU memory nears
  the budget.
- **Hitches:** KTX2 files are transcoded in web workers. Call `renderer.compileAsync()`
  and upload textures before the fade-in so the first frame doesn't stall.
- **Caching:** put a content hash in every asset filename and serve the files with
  `Cache-Control: public, max-age=31536000, immutable`. The browser's HTTP cache then
  keeps them between visits. No service worker is needed.
- **Loading screen or fade:** use a real loading screen with a progress bar for the first
  load and for Lantern travel. Use a short fade at doors, lifts, and fog gates when the
  next area is already preloaded. Lifts make natural loading corridors.

**Recommendation:** Keep the prototype on Pages with committed files, because it's small.
For the full migration, publish models and textures to an R2 bucket on a subdomain such
as `assets.ifkash.dev`, with CORS headers and a cache-everything rule, and commit only the
manifest with the file hashes. This changes one consequence of
[ADR 0005](../decisions/0005-asset-pipeline-tools.md), which commits pipeline output, so
the new ADR must say so.

### Phones

- **What touch mode drops:** real-time shadows (blob shadows instead), all
  post-processing, normal and ORM maps, and textures above 512 pixels. It keeps lightmaps,
  because they're cheap. It limits point lights to two, drawn actors to about 16, and draw
  distance to 60 to 80 m with thicker fog, and it always uses the simplest LOD.
- **Resolution:** the low preset's render scale is 0.75 of CSS pixels, which is blurry on
  a phone, where one CSS pixel is two or three device pixels. Use a pixel ratio of up to
  1.5 on phones instead, and measure it.
- **iOS memory:** Apple publishes no limit. A WebKit bug from October 2025 describes
  texture-heavy WebGL pages being closed and reloaded under memory pressure (Reported).
  One game-development site gives 300 to 500 MB without a source. Treat about 300 MB for
  the whole page as the ceiling until a real iPhone says otherwise, with 48 MB of that for
  textures. See
  [WebKit bug 300782](https://bugs.webkit.org/show_bug.cgi?id=300782).
- **Context loss:** Safari can drop the WebGL context under memory pressure or after the
  tab is in the background. Handle `webglcontextlost` by reloading the area instead of
  showing a black screen.

**Recommendation:** Touch mode stays on the low preset at 30 fps, with the drops listed
here, a 48 MB texture budget, and a context-loss handler. The prototype must run on one
real iPhone and one mid-range Android phone.

## Budget table

The following table compares the old budgets with the proposed ones:

| Budget | Old | Low (and touch) | Medium | High |
|---|---|---|---|---|
| Frame time | 16.7 ms | 16.7 ms on laptops; 33 ms in touch mode | 16.7 ms | 16.7 ms |
| Draw calls per frame, shadow passes included | 250 | 150 | 350 | 500 |
| Triangles per frame, shadow passes included | 600,000 | 500,000 | 1,200,000 | 2,000,000 |
| GPU texture memory | 96 MB (a guess) | 48 MB | 128 MB | 256 MB |
| Longest texture edge | 512, 1024, 2048 | 512 | 1024 | 2048 |
| First area download | 8 MB | 8 MB | 20 MB | 35 MB |
| Each later area, new files only | Not set | 4 MB | 12 MB | 20 MB |
| Whole game, never fetched at once | 60 MB | 60 MB | 200 MB | 350 MB |
| One file | Not set | 8 MB | 8 MB | 8 MB |

The arithmetic behind the medium column, all estimates:

- **Draw calls:** in the main pass, about 12 kit and prop materials, 20 terrain chunks,
  20 instanced scatter and prop groups, 80 for 40 actors at two meshes each, and 20
  effects: about 150. Each of two shadow cascades redraws about 100 of them. Total: 350.
- **Triangles:** environment 400,000 and actors 300,000 in the main pass, and about
  500,000 more across the two shadow cascades. Total: 1.2 million. The actor share
  assumes most of the 40 actors are at a distance LOD. The characters document owns that
  number.
- **GPU texture memory:** 20 environment sets at 2.3 to 3 MB is about 55 MB, two
  interior lightmaps at 2K are 11 MB, and about 10 character and weapon sets are 27 MB.
  That's about 95 MB, so 128 MB leaves room for the next area's preload.
- **First area:** a shared core of six sets (6.6 MB), four cave sets (4.4 MB), geometry
  (2 MB), one lightmap (0.7 MB), and about 6 MB for the player and one enemy. Total:
  about 20 MB. On low, ten 512 color textures (0.8 MB), simplified geometry (1.5 MB), and
  characters (about 3 MB) come to about 5 MB.
- **Each later area:** six to eight new sets (7 to 9 MB), geometry (2 MB), and a lightmap
  for interiors: about 12 MB.
- **Whole game:** a 9 MB core, 16 more areas at about 9 MB each (144 MB), about 40 MB of
  characters and bosses, and about 10 MB of audio: about 200 MB. The 83-minute main path
  visits six to eight areas, so a typical player downloads about 100 MB over the session.
- **One file:** 8 MB keeps every file well under the Pages limit of 25 MiB and lets
  downloads run in parallel.

The planned prototype must measure these, on the target laptop and on a phone:

1. Frame work at 1080p in one interior and one outdoor view, with each of these switched
   on and off: normal maps, the second shadow cascade, bloom, ambient occlusion, and four
   point lights.
2. Real download sizes for one 512, one 1K, and one 2K texture set, and for an ETC1S
   normal map against a UASTC one.
3. Whether Pages or R2 serves `.glb` and `.ktx2` files with transport compression, which
   changes geometry sizes by up to half.
4. GPU texture memory after loading two neighboring areas, from `perf/textures.ts`.
5. Time from pressing Play to the first frame on a 20 Mbps connection.
6. Lightmap bake time for one interior, and how many bakes needed a manual fix.
7. Probe-grid bake time in the browser for the same interior.
8. Whether an iPhone survives a walk through three areas without a reload.

I searched for shipped Three.js or WebGL games with a realistic look and published
measurements, and found none with draw calls, triangles, memory, and download size
together. The one usable figure is the cascade timing cited in the shadows section.

## Rejected options

- **Megascans and other Fab Standard content:** the license requires restricting
  extraction and forbids public repositories, even though the library is free.
- **Unity Asset Store kits:** distribution only as embedded in a product; not in a public
  repository.
- **Sketchfab Standard-license models:** forbidden as stand-alone files.
- **Synty, KayKit, Kenney, and Quaternius kits:** stylized low-poly; wrong look.
- **ShareTextures:** redistribution limits on a library that Poly Haven and ambientCG
  already cover.
- **TurboSquid, CGTrader, KitBash3D, and Poliigon:** not read in this pass. Commercial
  libraries of this kind normally forbid redistributing source files, so they'd fail the
  public-repository test the same way.
- **Whole heritage scans as level geometry:** one fused mesh of millions of triangles,
  not modular, and collision would have to be authored by hand.
- **CC BY-SA, CC BY-NC, and GPL art:** share-alike or non-commercial terms that the rest
  of the game's assets don't follow.
- **Free-tier AI generators as a main source:** inconsistent style, no modular snapping,
  and unsettled terms outside Meshy.
- **`WebGPURenderer` today:** every custom shader rewritten, and an API that still changes
  each release, for gains the baked-lighting plan mostly removes.
- **Real-time shadows from every interior light:** each shadowed point light redraws the
  scene six times.
- **Full-resolution ambient occlusion:** an estimated 4 to 8 ms of a 16.7 ms frame.
- **Ray-marched volumetric fog:** too expensive on integrated graphics; fog cards give
  most of the mood.
- **4K textures:** sharper than the screen can show at this camera distance.
- **Draco compression:** meshopt is already in the pipeline and the loader.
- **Committing all texture tiers to Git for the full game:** permanent history growth in
  a public repo.

## Open questions for the owner

1. **Megascans.** The library is free and is the best style match, but the license
   requires restricting extraction and forbids a public repo. Do you want to rule it out,
   as recommended, or ask Epic whether a packed web build is acceptable?
2. **Plain architecture.** Is a script-built kit with scanned textures good enough for
   castles and crypts, knowing that ornament will be sparse? The prototype room answers
   this by eye.
3. **AI props.** May gap props (bones, coffins, statues, a mine cart) come from a paid AI
   generator plan, which needs your account and an ADR?
4. **Blender.** May agents install Blender on the build machine for lightmap bakes and
   normal-map re-bakes?
5. **Hosting.** Is it acceptable to move game art to an R2 bucket on a subdomain and stop
   committing pipeline output?
6. **Download size.** Is about 20 MB before play on medium acceptable, with low kept at
   8 MB?
7. **Credits.** CC-BY gap models need visible credit. Is a credits page linked from the
   pause menu enough?

## What I couldn't verify

- **Appearance.** I didn't view any asset. Every style rating needs a look at the real
  files.
- **Megascans price.** The free relaunch comes from one news article; Fab's listing pages
  didn't load.
- **Sketchfab.** Its search page returned nothing to my request, and the site changed
  owner in August 2026. I couldn't confirm that CC0 and CC-BY downloads still work, and I
  verified no specific Sketchfab model.
- **Smithsonian Open Access and Three D Scans.** Their license pages blocked or didn't
  render for my request.
- **The Yughues packs on OpenGameArt.** The profile lists them; I didn't confirm the
  license on each pack page.
- **The OpenGameArt Dungeon Building Kit's look.** I read the page text only.
- **itch.io and Unity Asset Store kits.** I evaluated the licenses, not individual kits,
  because the license rules them out or makes them per-asset.
- **TurboSquid, CGTrader, KitBash3D, and Poliigon licenses.** Not read.
- **AI generator prices, and the Tripo, TRELLIS.2, and Hunyuan3D terms.** These come from
  third-party pages.
- **Every millisecond figure and every file-size figure.** They're estimates. I found no
  published benchmark for these effects on Intel integrated graphics.
- **iOS Safari's memory ceiling.** There's no official number.
- **`SunLight` on `WebGLRenderer`.** The r186 release notes list it under
  `WebGLRenderer`, but its docs page only shows WebGPU setup. Confirm it in the upgrade
  work package.
- **Transport compression for `.glb` on Pages and R2,** and whether cached R2 responses
  count against the read quota.
- **GitHub repository size limits.** Not checked; the hosting advice doesn't depend on
  the exact number.
- **The release year of r187.** The releases page shows "30 Oct" without a year.
