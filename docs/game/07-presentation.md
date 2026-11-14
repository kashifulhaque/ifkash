# Presentation

This document covers what the player sees, hears, and touches: art direction, asset
sources and licensing, characters and animation, VFX, lighting, the camera, controls,
the HUD and menus, audio, accessibility, and mobile. Read it before any WP in the
`render` or `ui` lanes.

## Art direction

> **Under review:** The owner wants to move to a more realistic look, in the style of
> *Demon's Souls* and *Dark Souls*. See [the art direction research plan](art-direction-research.md).
> This section stands until an ADR replaces it.

**Low-poly, flat-shaded, atmospheric.** Silhouettes and fog do the work that texture
detail does in larger games. The look sits next to the existing site: muted, calm,
restrained color, with one accent color per region.

- **Geometry:** Low-poly meshes with flat or lightly smoothed shading. No photographic
  textures. Small gradient textures and vertex colors only.
- **Atmosphere:** Height fog and distance fog in every outdoor region. The Spire is
  always visible through it as a silhouette.
- **Palette:** Each region has a base palette and one accent. Accent colors are reserved
  for things that matter: Lanterns, items, enemy telegraphs, and the Spire.
- **Readability over beauty:** Enemy telegraphs (glows, ground markers) must stay
  readable on every quality preset and in every color-blind mode.

| Region | Base palette | Accent | Light |
|---|---|---|---|
| The Verge | Ash grays, dry ochre grass | Lantern amber | Warm, low sun, haze |
| Mirrormere | Slate blue, pale reeds | Cathedral teal | Cool, overcast, mirror lake |
| Hollowdeep | Near-black, deep violet | Bioluminescent cyan | Dark with emissive fungi |
| Castle Graymantle | Stone gray, sea green | Banner crimson | Flat overcast, sea fog |
| Rimeshelf | White, ice blue | Foundry orange | Bright, snowfall |
| The Lattice Spire | Dusk indigo | Thread gold | Twilight, emissive threads |
| Hollowhearth | Warm wood, candle | Candle yellow | Candlelight only |

## Assets

### Sources

Prefer CC0 assets so no per-asset attribution is needed. Every pack used keeps its
license file next to its source files in `game/assets-src/PACK_NAME/`, and
`lf-tools credits` generates the in-game and on-site credits from those files.

| Source | License | Use for |
|---|---|---|
| Quaternius Universal Animation Library | CC0 (verified) | Humanoid locomotion, combat, and gesture clips |
| Quaternius Universal Animation Library 2 | CC0 (verified; [ADR 0009](decisions/0009-animation-library-2.md)) | Sword, heavy, shield, and guard clips |
| Quaternius modular character packs | CC0 (already in `static/models/npc`) | Player, Husks, NPCs |
| Quaternius nature, monster, and modular ruin packs | CC0 (verify each pack page before use) | Terrain props, ruins, some creatures |
| KayKit Character Pack: Skeletons | CC0 (verified) | Whitewarden Skeletons, Ossuary Keeper adds |
| KayKit Dungeon and Adventurers packs | CC0 per the catalog (verify each pack page before use) | Dungeon kits, weapons, armor attachments |
| Kenney asset packs | CC0 (verify the specific pack's listing) | Props, UI icons, sound effects |
| Poly Pizza | Per model (CC0 or CC-BY) | Gap-filling; CC-BY models need credits |
| Mixamo | Not verified; don't use without owner approval | n/a |

Don't use assets whose license forbids redistribution in a public repo, and don't use
AI-generated assets whose license is unclear. The owner approves every new pack in an
ADR.

### Procedural models

Some creatures don't exist in any CC0 pack: the Lamplight Matriarch, the Forgeheart
Colossus, the Unspooled, Lamp Mites, and Shard Golems. Build these in code in
`src/lib/latticefall/render/procedural/` from primitives with a small bone hierarchy and
procedural animation driven by `move_id` and `move_frame`. This matches the low-poly
style and needs no art tools.

### Pipeline

1. Put source files in `game/assets-src/` with their license.
2. Run `lf-tools assets`, which calls `gltf-transform` to prune, deduplicate, weld,
   simplify where needed, compress with meshopt, and convert textures to KTX2.
3. Output goes to `static/latticefall/` with a per-region manifest.
4. Kit pieces need an entry in `game/data/kits/*.ron` with a collider shape before a
   level can use them.

## Characters and animation

- **The Wayfarer** uses a Quaternius modular humanoid. Armor is a material tint plus
  attachments (helmet, pauldrons, cloak) from the kit packs, so 7 armor sets don't need
  7 full models.
- **Weapons** attach to hand bones. Each weapon class has at least one mesh; weapons in
  the same class vary by mesh, material, and scale.
- **Animation mapping:** Each move's `anim` field maps to one or more clip stretches
  (`render/actors/moveClips.ts`), with fallbacks if a library lacks a clip. The renderer
  retimes the clip so its contact frame (the hand's peak speed) lands in the middle of the
  move's hitbox window, plays the recovery at no more than 2.5 times the clip's rate, and
  blends locomotion by speed. The frame data is never changed to fit an animation.
- **Hitstop:** The drawn pose freezes for the hit's hitstop while the core keeps ticking,
  then plays faster until it's back in step with the move.
- **Missing clips:** If a clip is missing, the renderer plays a generic pose for the
  move's phase (wind-up, strike, recovery) and logs a warning in dev mode. This keeps
  gameplay WPs unblocked by art.

## VFX

All VFX are render-only and triggered by core events.

| Effect | Trigger | Notes |
|---|---|---|
| Weapon trail | Hitbox active frames | Ribbon mesh per weapon class |
| Hit sparks | `Hit` event | Color by damage type |
| Telegraph glow | Move data `vfx` at wind-up | Must stay visible on low quality |
| Ground marker | Move data, for AoE and dives | Projected decal, also shown as an outline for color-blind modes |
| Status aura | Actor `status_mask` | One small particle system per status |
| Spell projectiles | Projectile snapshots | Instanced meshes plus a light on high quality |
| Hitstop | Hitbox `hitstop` | Freezes the attacker and victim's animation for N frames; the core doesn't pause |
| Camera shake | Heavy hits, stomps | Off when reduced motion is on |

## Camera

- **Free camera:** Orbits the player at 4.5 m, pivot 1.6 m above the feet. Mouse or right
  stick controls yaw and pitch; pitch is clamped to -35 to +60 degrees.
- **Collision:** The camera casts a sphere from the pivot toward its desired position
  and stops just short of any hit. Queries use the core's read-only `raycast` export, so
  the renderer needs no separate physics world.
- **Lock-on:** The camera frames the midpoint between the player and the target, and
  pulls back for large targets (by the target's `camera_radius` in data).
- **Boss intro:** A short scripted move along a spline, defined per boss in data.
- **Dev camera:** `&cam=orbit` and `&cam=fixed:X,Y,Z,YAW,PITCH` override everything (see
  [the agent playbook](11-agent-playbook.md#verify-visually)).

## Controls

All bindings are remappable. The gamepad defaults follow Elden Ring's layout. The
following table lists the defaults:

| Action | Keyboard and mouse | Gamepad (standard layout) |
|---|---|---|
| Move | W, A, S, D | Left stick |
| Camera | Mouse | Right stick |
| Light attack | Left mouse | R1 or RB |
| Heavy attack | Shift and left mouse | R2 or RT |
| Guard or parry | Right mouse | L1 or LB |
| Art | Shift and right mouse | L2 or LT |
| Dodge, sprint | Space (tap, hold) | Circle or B (tap, hold) |
| Jump | F | Cross or A |
| Interact (rest, read, open) | E | Triangle or Y |
| Use item | R | Square or X |
| Lock-on | Q or middle mouse | Right stick press |
| Switch target | Mouse wheel | Right stick flick |
| Cycle slots | Arrow keys | D-pad |
| Two-hand | V | Hold Triangle or Y, then press R1 or RB |
| Map | M | Touchpad or View |
| Menu and pause | Escape | Options or Menu |

A **one-button-attack** option merges light and heavy (tap and hold), and every hold
action can be switched to a toggle.

## HUD

The HUD is minimal and drawn in DOM with Svelte over the canvas, using the site's design
tokens and fonts, so it inherits the site's design and theme.

- **Top left:** HP bar (`--falu`), FP bar (`--fjord`), stamina bar (`--moss`). Bar length
  grows with the stat's maximum.
- **Status meters:** Small meters appear above the bars only while a status is building.
- **Bottom left:** Four quick slots in a cross: right hand, left hand, spell, and item,
  with the flask count.
- **Bottom right:** Glimmer held.
- **Bottom center:** Boss bar with the boss's name.
- **Center:** Lock-on marker, interaction prompts, and short notifications (item
  picked up, Lantern lit).
- **Hide options:** HUD scale (75-150%), auto-hide out of combat, and fully hidden.

## Death and victory

These treatments are original and must not imitate other games' screens. See
[the IP rules](00-vision-and-scope.md#ip-rules).

- **Death:** The image desaturates toward the region's fog color while the Wayfarer's
  lantern gutters out. A single short line appears in small capitals in the site's sans
  font, left-aligned near the bottom third: *The thread slips.* After 90 ticks, a button
  reads **Wake at LANTERN_NAME**.
- **Boss defeated:** A thin gold line draws across the screen like a thread pulled
  taut. The boss's name appears above it in the display font, and the word
  *Remembered* below it.
- **Great Sigil:** As for a boss, then the Sigil's name and a short line about which
  door it opens.

## Menus

Menus pause the game. The pause menu always contains, in this order: **Resume**,
**Equipment**, **Inventory**, **Map**, **Codex**, **Settings**, **Read the plain site**,
and **Quit to title**.

Every menu is fully keyboard, gamepad, and screen-reader navigable, because it's DOM,
not canvas.

## Audio

- **Engine:** Web Audio with buses for master, music, effects, ambience, and UI. Effects
  are positional (`PannerNode`) for actors and projectiles.
- **Music:** Each region has a calm exploration loop. Each boss has an intro stem, a loop,
  and a phase-2 stem layered in. The Bellwright's music carries the fight's beat.
  Music must be original or CC0 or CC-BY licensed; nothing that imitates a known score.
- **Ambience:** Wind, water, cave drips, and fire, layered by region.
- **Effects:** Footsteps by surface, weapon swings by class, hits by damage type, Lantern
  ignition, item pickup, UI clicks. Kenney's CC0 audio packs are the default source.
- **Placeholder:** Until real music exists, the existing procedural synth approach in
  `src/lib/game/audio.ts` is a reference for a procedural ambient bed.
- **Defaults:** The game starts with sound on at 60% but respects a muted tab. A sound
  toggle is always on the HUD's pause menu.

## Accessibility

The baseline is the Game Accessibility Guidelines' basic tier plus the following
options, all in **Settings**:

- Difficulty settings and Wayfarer mode (see [the combat spec](03-combat.md#difficulty-settings)).
- Remappable controls, toggle instead of hold, and one-button-attack mode.
- Color-blind modes (protanopia, deuteranopia, tritanopia). Every color-coded signal
  also has a shape or an outline.
- Reduced motion: no camera shake, no head bob, reduced particle flashes. On by default
  when the operating system requests it.
- Text size (75-200%) and HUD scale.
- Subtitles for all NPC lines and for important sounds (boss roars, bell tolls).
- Pause anywhere outside a boss fight; inside a boss fight, pause freezes the game too,
  but **Quit to title** is disabled until the fight ends.
- Field of view (60-90 degrees) and camera distance.

## Mobile

Phones get a different default, because souls-like combat on a touchscreen frustrates
most players.

- On a phone-sized screen, `/game` opens the **codex view**: the plain portfolio with a
  short trailer clip of the game and a **Try touch mode** button.
- **Touch mode** adds a virtual stick, camera drag on the right half, and buttons for
  light, heavy, guard, dodge, Art, use item, interact, and lock-on. Lock-on is on by default. Quality drops to
  the low preset and the frame rate target is 30.

## Quality presets

| Setting | Low | Medium | High |
|---|---|---|---|
| Render scale | 0.75 | 1.0 | 1.0, up to device pixel ratio 2 |
| Shadows | Player only, 1024 | Sun, 2048 | Sun, 4096 with 2 cascades |
| Draw distance | 120 m | 200 m | 320 m |
| Grass density | 25% | 60% | 100% |
| Post-processing | Fog only | Fog, bloom | Fog, bloom, ambient occlusion |
| Mirrormere reflections | Environment map | Environment map | Planar reflection |

The game picks a preset on first load from a 2-second benchmark scene and lets the
player change it.
