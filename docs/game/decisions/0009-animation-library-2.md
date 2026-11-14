# 0009: Universal Animation Library 2 for melee clips

- **Status:** Accepted (2026-11-14, by the owner)
- **Date:** 2026-11-14
- **WP:** None; raised by GitHub issues 21, 83, and 22 (attack, stance, and Art animations)

## Context

The Wayfarer is animated with clips from the Quaternius Universal Animation Library 1,
Standard edition ([ADR 0005](0005-asset-pipeline-tools.md)). That library has one sword
clip, `Sword_Attack`, so every light attack replays it, and every heavy, thrust, and Art
falls back to a generic pose built from bone offsets. The owner's playtest on 2026-11-13
found the light and heavy attacks janky (issue 21) and the Arts wrong (issue 22).

Quaternius also publishes Universal Animation Library 2. Its Standard edition is a free
download under CC0 1.0, uses the same Unreal mannequin rig, and retargets onto the Wayfarer
with the existing code and no changes. It has these melee clips:

- Sword cuts: `Sword_Regular_A`, `Sword_Regular_B`, and `Sword_Regular_C`, with
  `_Rec` recoveries, plus `Sword_Regular_Combo`.
- Heavy hits: `Sword_Heavy_Combo`, four distinct hits (a wound-up sweep, an overhead chop,
  a low lunge, and a rising cut).
- Movement attacks: `Sword_Dash` and `Shield_Dash`.
- Guard: `Sword_Block`, `Idle_Shield_Loop`, `Idle_Shield_Break`, and `Shield_OneShot`.
- Others: `Melee_Hook`, `OverhandThrow`, `TreeChopping_Loop`, `NinjaJump_*`, `Consume`,
  and `Hit_Knockback`.

## Decision

On 2026-11-14 the owner approved Universal Animation Library 2, Standard edition, as a
second clip library.

- Source: `https://quaternius.itch.io/universal-animation-library-2`, the file
  `Universal Animation Library 2[Standard].zip` (18,735,003 bytes, downloaded 2026-11-14).
  Its `License.txt` states CC0 1.0 Universal.
- `game/assets-src/universal-animation-library-2/` holds the author's license and readme,
  a `SOURCE.md`, and `UAL2_Standard.glb` trimmed with `game/scripts/trim-glb.mjs` to the
  clips the game plays. `lf-tools assets` writes `static/latticefall/actors/ual2.glb`.
- The renderer loads `ual2.glb` beside `ual.glb` and merges the clips. Each move `anim`
  lists clip choices, best first, in `render/actors/moveClips.ts`, so a move falls back to
  a Library 1 clip or the generic pose if a clip is missing.

## Consequences

- Light, heavy, running, jump, plunge, and counter attacks for every melee weapon class
  play authored clips, and the guard and Arts can use the block, shield, and heavy clips.
- The game loads one more GLB of about 1 MB when the visitor presses Play. The home page
  bundle doesn't change.
- No credits are required. `lf-tools credits` can credit Quaternius, as it does for
  Library 1.
- The library is listed in
  [the presentation spec's Sources section](../07-presentation.md#sources).

## Alternatives considered

- **Library 1 only:** It has a single sword clip, so combos repeat one swing and heavies
  keep the generic pose.
- **The paid Source editions of either library:** They're also CC0 and have more clips,
  but they cost money, and the free editions cover the moves the issues name.
- **Hand-authored bone-offset poses:** They're what reads as janky today.
