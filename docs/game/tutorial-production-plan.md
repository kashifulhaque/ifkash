# Tutorial production plan: Waking Hollow as a finished game

- **Status:** Proposed, 2026-11-15. Waits for the owner to set up the tools and accounts
  in [What the owner sets up](#what-the-owner-sets-up).
- **Goal:** Make Waking Hollow, the tutorial cave from Lantern L01 to the Hollow Sentry
  (B01), a complete, polished game in the 2010s souls look. Only then move on to the next
  area.
- **Live game today:** Players can only enter Waking Hollow
  (`src/lib/latticefall/tutorialLock.ts`). Beating B01 shows an end-of-tutorial card. The
  other 15 areas stay in the repo and open with `?dev=1`.
- **Evidence:** [Research 1](art-research/01-characters-and-animation.md) (characters and
  animation), [research 2](art-research/02-environments-and-renderer.md) (environments
  and renderer), and [research 3](art-research/03-ai-tooling-and-pipeline.md) (AI tools and
  the pipeline, written after the owner accepted AI-generated art).

## Where the art comes from

Nothing is modeled by hand, and nothing comes from a paid asset store. Store licenses (Fab,
Unity Asset Store, Sketchfab Standard, Synty) forbid a public repository and require that
players can't extract the files, which a web game can't promise. Each kind of asset comes
from the source that fits it best:

| Asset | Source | Why |
|---|---|---|
| Player, husks, hound, B01, NPCs, weapons, unique props | **Generated** with Meshy Pro from a concept image, then cleaned in Blender | On a paid plan the owner owns the output, so it can sit in a public repo |
| Cave rock, stone, dirt, and wood surfaces | **Downloaded**, CC0 scans from Poly Haven and ambientCG | Public domain and photographic, the best match for the look |
| Rocks, roots, rubble | **Downloaded** (Poly Haven, CC0) or generated locally with Hunyuan3D 2.1 | Free; generated only where no scan fits |
| Cave walls, floors, the round arena, pillars | **Built by script** in headless Blender, covered with the scanned surfaces | Simple shapes with good surfaces and light are what the souls look is made of, and agents can build them |
| Basic motion: idle, walk, run, hit reactions, deaths | **Downloaded** from Mixamo (free, Adobe account) | Real motion capture; most of the basics already exist |
| Attacks, rolls, guard, parry, boss moves | **Drafted** with Uthana text-to-motion or the owner acting on a phone camera (Move One), then **polished by hand** in Cascadeur | No AI tool makes weighty sword combat on its own; the polish is what makes it look smooth |
| Creature motion (the hound) | Mesh2Motion's CC0 quadruped rig, drafted, then polished in Cascadeur | Mixamo has no animals |

Mixamo is a good base, but it can't carry the game alone: it's thin on parries,
greatsword and boss-sized moves, and it has no animals. Research 3 has the license quotes
and the rejected options, such as Cascadeur's free plan (non-commercial) and the TRELLIS.2
Mac port (a non-commercial dependency).

## What the tutorial needs

The list follows the cave as it's built in `game/data/levels/waking_hollow/`, plus the
additions the owner asked for: NPCs, more collectibles, and a hound.

### Characters and creatures

| Model | Count in the cave | Triangle budget | Notes |
|---|---|---|---|
| The Wayfarer (player) | 1 | 10-25K | Worn travel armor with a hood; hides the weak spots of generated faces and hands |
| Wayfarer's Blade and a round shield | 1 each | 2-5K each | Starting gear |
| Husk | 4 | 5-15K | Hollowed humans in rags. One body with two or three variants (helmet, weapon). In the game: Mixamo's Romero, 15,000 triangles |
| Thorn Hound | 1, proposed 2-3 | 5-15K | The dog: lean and thorn-grown. Uses its own quadruped skeleton. In the game: the procedural model |
| B01 Hollow Sentry | 1 | 20-60K | Empty armor with a lantern burning in its chest, a shield, and a heavy weapon. The lantern flares before each attack. In the game: Mixamo's Castle Guard 02 with a greatsword and no shield, 4,421 triangles |
| NPC: a wounded knight at L01 (proposed) | 1 | 10-20K | Sits by the Lantern and gives the opening lines and a gift. New content: needs dialogue and a WP |
| Corpses and remains | 4-6 | 2-5K | Loot markers that make the cave feel used |

### Animation clips

About 60 clips, which is the biggest cost in the plan.

- **Wayfarer, about 30:** idle, walk, run, sprint, strafe while locked on, jump, fall,
  land, rolls, a backstep, three light attacks in a chain, a heavy attack, a sprint attack,
  guard (idle, hit, break), parry, riposte, drink the flask, use an item, pick up, open a
  chest, rest at a Lantern, light and heavy hit reactions, a stagger, a knockdown and
  getting up, and a death.
- **Husk, about 8:** rise from the ground, idle, shamble, two attacks, a hit, a stagger,
  and a death.
- **Thorn Hound, about 8:** idle, trot, run, a lunge bite, a pounce, a hit, a death, and a
  growl.
- **Hollow Sentry, about 12:** intro, idle, walk, overhead slam (30-tick wind-up), sweep,
  shield bash (14-tick wind-up), a three-hit combo with a 40-tick punish window, a turn, a
  stagger, and a death.
- **NPC, about 3:** sitting, talking, and pointing.

Every attack is timed against the frame data in `game/data/moves/`. The simulation decides
when a hit lands, so a clip must match its move's wind-up, active, and recovery frames
rather than the other way around.

### Environment

- A cave kit: rock walls, floors, a ceiling, the jump ledge, a collapsed corridor, the
  cracked wall (hollow wall), and the 18 m round arena with four pillars.
- Surfaces: wet rock, packed dirt, old stone masonry, and roots, each with color, normal,
  and roughness maps (1K, KTX2).
- Dressing: stalagmites, rubble, roots through the ceiling, bones, broken carts, a few
  candles.
- Light: baked lightmaps for the cave and arena, plus the warm Lantern, the Sentry's chest
  lantern, and torches as flickering point lights. Fog and a cold color grade.
- Props: the Lantern, chests, pickups with a glint, message markers, and the fog gate
  effect.

### Items and collectibles

The cave has salts, clotweed, a closet chest, and B01's Wick Bead. Proposed additions: one
relic (a portfolio project) behind the hollow wall, a Chronicle Page with the earliest work
entry, and a few more Glimmer drops on corpses. Each item needs an icon and a ground model.

## What the owner sets up

| What | Why | Cost |
|---|---|---|
| Blender: `brew install --cask blender` | Agents clean meshes, bake maps and lightmaps, and export GLB with scripts | Free |
| Cascadeur, Indie plan | Polishing every attack, roll, and boss move. The free plan is non-commercial and can't export | USD 19 a month, or USD 96 a year |
| Meshy Pro account | Characters, creatures, weapons, and props. Keep the invoices and a copy of the terms | About USD 20 a month |
| Adobe account (Mixamo) | Basic motion and a fallback auto-rigger. Mixamo has no API, so you download the clips agents list | Free |
| Uthana account | Drafts of attacks that Mixamo lacks; agents can call its API | USD 0.10 per second of motion, pay as you go |
| Optional: Tripo API credits | Rigs a generated mesh straight to the Mixamo skeleton, without the Mixamo website | Per use |
| Optional: Move One | You act out heavy moves on a phone camera for real weight | USD 18 for a month |

Expect about USD 45-60 a month for the two or three months the art is being made, and
nothing after. Agents can't enter passwords or payment details, so you do every sign-up
and download by hand.

**Your time** is the real cost. Agents can generate, clean, rig, retime, and import, but
they can't judge on a pad whether a swing has weight. Research 3 estimates 1-3 hours in
Cascadeur per hero attack. That's roughly 30-60 hours for the tutorial's attacks and boss
moves, spread over the phases below.

## Phases

Each phase ends with something you can play, and a pad check before the next one starts.

1. **Pipeline and the Wayfarer.** Agents write the Blender scripts (clean, rig check,
   bake, export), teach the renderer to play skinned GLB characters with baked clips timed
   to the move frame data, and bring in a generated Wayfarer with Mixamo basics and drafted
   attacks. You polish the first light attack in Cascadeur. **Check:** the Wayfarer moves
   and swings in the cave and looks better than today, on a pad, at 60 fps.
2. **The full player moveset.** All 30 player clips, polished. **Check:** a pad run of the
   whole cave with the old enemies.
3. **Enemies and the boss.** Husks, hounds, and the Hollow Sentry, each with its full
   moveset retimed to its frame data. **Check:** the boss fight reads well: every wind-up
   is visible and every punish window feels fair.
4. **The cave.** The script-built kit, scanned surfaces, dressing, baked light, and fog.
   **Check:** draw calls, triangles, download size, and frame time on the integrated-
   graphics laptop target.
5. **Content and finish.** The NPC and dialogue, the new collectibles, item icons,
   sounds for new motions, the landing card's screenshots, and the controls teaching for
   a pad. **Check:** a first-time player gets from L01 to the end card without help.

Phase 1 replaces the prototype that the
[art direction plan](art-direction-research.md) asked for. Its result decides whether the
pipeline is good enough to write the ADR and run phases 2-5.

## Run the character pipeline

The engineering half of phase 1 is in place: one command turns a Mixamo-rigged character
and its Mixamo clips into a player model the game can draw. The scripts run headless
Blender, so you don't open Blender yourself.

### Prepare the files

Keep the source files outside the repository. Raw Mixamo and Meshy files must never be
committed; only the baked GLB and its sidecar ship.

1. Generate the model in Meshy and download it as FBX with its textures.
1. Upload the FBX to Mixamo and let it rig the character.
1. Download the character from Mixamo as FBX, **With Skin**, in its T-pose.
1. Download each clip as FBX, **Without Skin**, at **30** frames per second, with
   **In Place** turned on where Mixamo offers it.
1. Put every file in `~/cinderheart-art/wayfarer/`.

### Bake the character

To bake the Wayfarer, run the following command from the repository root:

```sh
mise run art:character -- wayfarer
```

The first time, also run `mise run game:ktx`, which installs the texture compressor.

The command takes about a minute and does the following:

1. Runs `game/art/blender/bake_character.py` in headless Blender, which reads
   `game/art/characters/wayfarer.json` and writes
   `game/assets-src/characters/wayfarer/wayfarer.glb` and `wayfarer.json`:
   - Joins the meshes, merges duplicate vertices, recalculates normals, decimates to the
     triangle budget (20,000 by default), and keeps four bone weights per vertex.
   - Scales textures down to 1024 pixels.
   - Strips the `mixamorig:` prefix from bone names.
   - Moves each clip onto the character in armature space, so a clip file with another
     rest pose still plays correctly.
   - Removes horizontal root travel, so every clip stays in place and loops meet.
   - Finds each attack's contact frame, where the right hand moves fastest.
   - Smooths capture jitter with a small filter that leaves the frames around the contact
     untouched.
1. Adds the character to `game/assets-src/assets.json` the first time.
1. Runs `lf-tools assets`, which compresses the textures to KTX2 (ADR 0010) and writes
   `static/latticefall/characters/wayfarer.glb` and `wayfarer.json`.

The command prints each step's result: triangles before and after, each clip's frames,
how much travel it removed, and each contact frame. It warns about a clip it can't find
and about files in the folder that the config doesn't name.

### Edit the clip mapping

The config, `game/art/characters/wayfarer.json`, maps Mixamo file names to the game's clip
ids. Mixamo names its downloads after the animation, so edit the `file` entries to match
what you downloaded. A `file` can list several names; the first one found wins. Case,
extensions, and repeated spaces don't matter.

`ART_CLIP_IDS` in `src/lib/latticefall/render/actors/artClips.ts` lists the clip ids. Only
`idle`, `walk`, and `run` are required. The same file says which moves play which clip:

- `ART_ANIMS` maps the sword, shield, and fist moves by their `anim` name, with fallbacks
  for a missing clip.
- A weapon class without a listing plays the one-handed clips for the move's role in its
  moveset (`light_2`, `heavy`, `rolling`), read from the move's id.
- While the player two-hands a weapon, `twohand_idle` is the arms' stance, `twohand_guard`
  the guard, and attacks play the two-handed clip for their role (`twohand_slash`,
  `twohand_slash_b`, `twohand_slam`, and so on). Mixamo's Great Sword Pack has all of them.
  A shield and fists keep their own strikes.
- A left-hand swing plays the clip's mirrored bake, named with the suffix `_lh`. With a
  shield in the right hand, the guard is `block_idle_lh`.

The Paladin's fist clips (`punch_jab`, `punch_cross`, `punch_hook`, `punch_uppercut`, and
`punch_heavy`) come from Mixamo's boxing animations. Download each one "Without Skin" at
30 fps into the source folder, saved under the clip id (for example, `punch_jab.fbx`), and
rebake. Until then, fists swing the sword clips.

Each clip takes these options:

- `loop`: `true` for idle, walk, run, and block idle. Smoothing then wraps around the
  ends.
- `contact`: `"auto"` finds the contact frame; a number sets it by hand. Leave it out
  for clips without a strike.
- `root`: `"lock"` pins the hips over the feet instead of only removing drift. The roll
  uses it.
- `smooth`: the filter width in frames (default 1). Use `0` to turn it off.
- `trim`: `[FIRST, LAST]` frames to keep.
- `contactBone` and `contactWindow`: which bone and which part of the clip (default
  `[0.1, 0.85]`) the contact search uses.
- `mirror`: `true` bakes the clip left for right, for the left hand. Give the mirrored
  clip the right-hand clip's id plus `_lh`. Its contact bone flips too.

The config also sets `triangles`, `textureSize`, and `character`, the character file's
name. Without `character`, the one FBX that no clip uses is the character.

### Set the grip

Mixamo's sword clips were captured holding a prop, so a weapon held at any other angle
looks broken. A config's `grips` stores how the hand holds the prop. Measure it once from
a Mixamo "W/Prop" character, such as Paladin W/Prop J Nordstrom, downloaded as FBX:

```sh
blender --background --factory-startup --python game/art/blender/measure_grip.py -- ~/cinderheart-art/reference/paladin_wprop.fbx
```

Copy the printed list into the config as `"grips": {"R": [...]}`. The bake places that grip
on the character's own hand, and the game attaches every held weapon there, except shields
and bows. Every Mixamo skeleton shares the same hand bones, so one measurement serves any
character that uses Mixamo's sword clips. A config without `grips` keeps the computed grip.
With `"mirrorGrips": true`, the bake reflects the right grip onto the left hand, so a weapon
held in the left hand sits as the mirrored clips expect.

### Check it in the game

Start the dev server and open
`/game?dev=1&play=1&art=wayfarer&scenario=waking_hollow`. Without `&art=`, the game draws
the old Wayfarer, and players never see an art character. To hold one pose for a
close look, add the animation preview parameters, for example
`&anim=move:sword_slash_r&animt=0.41&animcam=front`.

The renderer plays the clips at the move's own frame data: it stretches or squeezes each
attack so that the clip's contact frame lands in the middle of the move's hitbox window,
and it crossfades between locomotion, guard, and moves.

### Private art

Mixamo's own characters, such as the Paladin, may ship inside the game but may not sit in
the public repository. A config with `"private": true` keeps the character out of Git from
end to end:

- The bake writes its sources to `game/assets-src/private/` and its jobs to
  `game/assets-src/assets.private.json`. `lf-tools assets` builds them into
  `static/latticefall/private/` and leaves them out of `manifest.json`. All three paths
  are gitignored.
- In development, the game loads the character from `/latticefall/private/`.
- In production, it loads it from `GET /api/art/private/NAME.glb`, which the API Worker
  serves from the `ifkash-art` R2 bucket (`api/src/handlers/art.rs`).
- `PRIVATE_ART` in `src/lib/latticefall/render/actors/artClips.ts` lists the private
  characters, and `PLAYER_ART` names the one every player plays. `&art=off` draws the old
  Wayfarer. If a character fails to load, the game draws the Wayfarer instead.

After a bake, upload both files so players get the new version (within an hour, the
route's cache time). From the `api` folder, run the following commands:

```sh
npx wrangler r2 object put ifkash-art/private/NAME.glb --file ../static/latticefall/private/NAME.glb --content-type model/gltf-binary --remote
npx wrangler r2 object put ifkash-art/private/NAME.json --file ../static/latticefall/private/NAME.json --content-type application/json --remote
```

Replace `NAME` with the character's name: `paladin`, `husk`, or `sentry`.

The raw Mixamo downloads stay in `~/cinderheart-art/`. Back them up yourself: they're in
neither Git nor R2.

### Give an enemy an art character

Enemies and bosses use the same pipeline as the player. The husks in Waking Hollow are
Mixamo's Romero with Mixamo zombie clips (`game/art/characters/husk.json`), and B01 is
Mixamo's Castle Guard 02 at 2.9 m with the Great Sword Pack
(`game/art/characters/sentry.json`). Both are private art. The Thorn Hound keeps its
procedural model.

To give another enemy kind an art character, do the following:

1. Write its config in `game/art/characters/`, with `"private": true` for a Mixamo
   character, and bake it with `mise run art:character -- NAME`. Besides the player's ids,
   an enemy config can fill these clip ids:
   - `rise`: the enemy lies at the start of this clip until it first moves, starts a move,
     or is struck, and then plays the rest. The husks use Zombie Stand Up.
   - `stagger`: plays while the snapshot's `STAGGERED` flag is set. Without it, `hit` plays.
   - `attack_a`, `attack_b`, `slam`, `sweep`, `bash`, and `combo_1` to `combo_3`: the
     attacks that `ART_ANIMS` maps the enemy's moves to.

   `hit` plays on an unguarded hit, and `death` plays once and holds its last frame.
   One clip file can fill several ids with `trim`; B01's 3-hit combo is one Mixamo clip
   cut into three.
1. In `src/lib/latticefall/render/actors/artClips.ts`, do the following:
   - Add the enemy's registry number to `ENEMY_ART`.
   - Add its name to `PRIVATE_ART`.
   - Map its move `anim` names to clip ids in `ART_ANIMS`.
   - Give it an `ART_CHARACTERS` entry with its height in meters, and optionally a held
     `weapon` and its `weaponScale`, or a chest `lantern`.
1. Upload both baked files to R2, as in [Private art](#private-art).

An enemy walks differently from the player. It plays one loop at a time, `walk` or `run`,
whichever loop's natural speed is nearest its ground speed, at the rate that keeps the
feet planted (`pickGait` in `render/actors/locomotion.ts`). Mixamo loops differ in length
and in which foot leads, so a blend of two of them makes the legs fight. When the
simulation moves an enemy sideways while it faces its target, the body turns along the
path; when it backs away, the loop plays in reverse (`gaitFacing`). For each loop, pick a
Mixamo clip whose speed is near the speed the enemy moves at: the husk's walk is Zombie
Walk with **Stride** and **Overdrive** at 100, about 1 m/s, for the 1.12 m/s it circles
at.

The renderer starts loading every enemy character with the player. For up to four
seconds, an enemy whose character is still loading isn't drawn. After that, its
procedural model draws until the character arrives. If the character fails to load, the
procedural model stays. To compare, open the game with `&dev=1&enemyart=off`, which draws
every enemy as its procedural model.

B01's lantern is an emissive flame on the `Spine2` bone with a point light. Its light
swells through each attack's wind-up, peaks as the hitbox opens, and goes out when B01
dies (`lanternGlow` in `artClips.ts`). To check a pose, draw the character as the player,
for example `&art=sentry&anim=move:sentry_overhead&animt=0.36&animcam=three`.

### Test without Mixamo files

`mise run art:standin` writes a scripted stand-in to `~/cinderheart-art/standin/`: a
Mixamo-named skeleton, a 62,000-triangle mesh with duplicate vertices and a 2048-pixel
texture, and 13 clips that travel and jitter. `mise run art:character -- standin` bakes it,
and `&art=standin` draws it. The stand-in is made by script, so it carries no third-party
license.

## The cave kit

Phase 4's environment is built: Waking Hollow draws a script-built cave in place of its
greybox. The simulation is unchanged. The art lines up with the colliders, and the
build script stops if the rooms in its config don't match the layout's floor blocks.

- **Kit:** `game/art/cave/build_cave.py` carves the rooms with a voxel remesh in headless
  Blender, pushes the rock outward with noise and strata, and adds the floors, the jump
  ledge, a collapsed corridor, the gate facade and the arena in old masonry, four broken
  pillars, a floor ring, stalactites, stalagmites, rubble, hanging roots, bones, torches,
  and candles. Rock never stands more than 0.15 m in front of a wall collider, and
  dressing in walkable space is low or above head height. The hollow wall is a bricked-up
  plug that sinks into the floor when struck.
- **Surfaces:** five CC0 Poly Haven materials (wet rock, mud, masonry, flagstones, bark)
  at 1K, compressed to KTX2. Sources:
  [`SOURCE.md`](../../game/assets-src/waking-hollow-cave/SOURCE.md).
- **Light:** Cycles bakes occlusion and the torch, candle, and Lantern light into vertex
  colors. In the game, a pool of point lights (3, 6, or 8 by quality preset) follows the
  player between the 22 light spots. A darker fog, a cool ambient, ACES tone mapping, and
  a desaturated grade come from the art manifest. A cool shaft of light falls from the
  arena's dome. The boss's chest lantern is the enemies' work, not the kit's.
- **Budgets:** the benchmark (`&bench=waking_hollow`, medium preset, 1920 x 1080, with
  the bench's 39-actor crowd) passes: at most 137 draw calls and 177,000 triangles, and
  18.9 MB of texture memory. The kit downloads 12.1 MB (`cave.glb` 11.4 MB and
  `hollow_wall.glb` 0.6 MB). Frame time wasn't measured on the integrated-graphics
  target.
- **Rebuild:** `mise run art:cave` fetches the textures, runs the script, and compresses
  the output. Light and fog values are in `game/art/cave/cave.json`.

After the owner's first playtest (2026-11-16), the kit gained entity props built by
`game/art/cave/build_props.py`: a stone grave-lantern with a lit iron chamber for the
Lantern, an iron-bound wooden chest whose lid swings on its hinge, a small clay jar of
salts with a soft glint for ground pickups, and a glowing scrawled inscription for message
marks. The ambient light rose from 0.6 to 1.5 and the grade's shadow lift rose, so the
dark areas read without washing out the warm pools. A boss's fog gate fades out over about
1.5 seconds when the core reports the boss defeated, and never draws if the save says the
boss is already beaten.

What needs a pad check: whether the cave is too dark on a real screen, whether the
cracked wall reads as breakable, and the camera in the low corridors. The camera collides
with the simulation's colliders, not the art, so at a steep angle it can rise above a
corridor's ceiling and see over it.

## Decisions for the owner

1. **NPC and new content.** Do you want the wounded knight at L01, a relic, and a
   Chronicle Page in the tutorial? They need dialogue and new work packages.
2. **Your Cascadeur time.** Will you do the polish yourself? If not, the animation stays
   at draft quality: Mixamo plus AI drafts.
3. **Hosting.** Generated models and textures for one area are about 20 MB. Keep them in
   the repo for the tutorial, or move art to an R2 bucket now (research 2 recommends R2
   for the full game)?

## Risks

1. **The motion still looks stiff.** If AI drafts are poor, polish takes longer. Phase 1
   finds this out on one attack before anything else is built.
2. **Generated meshes bend badly at the joints.** Armor over the shoulders and hips, and
   hand fixes to skin weights in Blender, reduce it.
3. **Performance.** Realistic surfaces and baked light cost more than flat colors. Phase 4
   measures it on the target laptop; the low quality preset keeps smaller textures.
