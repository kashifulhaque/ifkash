# 0008: Model packs for the art pass

- **Status:** Accepted (2026-11-14, by the owner)
- **Date:** 2026-11-14
- **WP:** None yet; staged ahead of the art-pass integration phase

## Context

Most of Cinderheart's levels are greybox, the Lantern checkpoint has no model, and the
Hollowhearth NPCs are greybox figures. The enemies, bosses, weapons, and armor are already
procedural. [The presentation spec](../07-presentation.md#sources) prefers CC0 packs, keeps
each pack's license next to its files in `game/assets-src/PACK_NAME/`, and says the owner
approves every new pack in an ADR.

The packs were researched, license-checked, and staged on the branch
`claude/game-models-staging-b685dc` without touching the game. The details, triangle
counts, and the integration checklist are in [MODELS.md](../assets/MODELS.md).

## Decision

On 2026-11-14 the owner approved the following eight packs for use in the game. Each is CC0
1.0, with its license checked on its own page on 2026-11-14 and the author's license file
committed next to the files:

| Pack | Folder in `game/assets-src/` | License evidence | Files staged |
|---|---|---|---|
| Kenney Castle Kit 2.0 | `kenney-castle-kit` | https://kenney.nl/assets/castle-kit; `LICENSE.txt` | 45 |
| Kenney Graveyard Kit 5.0 | `kenney-graveyard-kit` | https://kenney.nl/assets/graveyard-kit; `LICENSE.txt` | 60 |
| Kenney Nature Kit 2.1 | `kenney-nature-kit` | https://kenney.nl/assets/nature-kit; `LICENSE.txt` | 87 |
| Kenney Fantasy Town Kit 2.0 | `kenney-fantasy-town-kit` | https://kenney.nl/assets/fantasy-town-kit; `LICENSE.txt` | 51 |
| Kenney Survival Kit 2.0 | `kenney-survival-kit` | https://kenney.nl/assets/survival-kit; `LICENSE.txt` | 15 |
| KayKit Dungeon Remastered 1.0 | `kaykit-dungeon-remastered` | `LICENSE.txt` in the author's GitHub repository | 88 |
| KayKit Character Pack: Adventurers 1.0 | `kaykit-adventurers` | `LICENSE.txt` in the author's GitHub repository | 16 (weapons and shields only) |
| KayKit Character Pack: Skeletons 1.0 | `kaykit-skeletons` | `LICENSE.txt` in the author's GitHub repository; the spec already lists it as verified | 7 (weapons and shields only) |

Two more items ride along and need no new pack approval:

- **Procedural models** in `game/assets-src/procedural/` are original work of this
  repository, like the audio.
- **Five NPC models** from the Quaternius modular characters already in the repo. They
  depend on the open license-file question in GitHub issue 42, not on this ADR.

The approval covers the packs only. Integration (the bake step in `lf-tools
assets`, kit catalog entries, and renderer changes) is a separate work package.

## Consequences

- None of the packs needs credits. `lf-tools credits` can still credit Kenney and Kay
  Lousberg, which both authors welcome.
- The pipeline gains a vertex-color bake (`game/scripts/bake-vertex-colors.mjs`), because
  every pack colors its faces through a palette texture or material colors and the kit
  renderer draws vertex colors only. ADR 0005's `gltf-transform` setup is unchanged, and
  no texture reaches the game, so `toktx` stays unneeded.
- The repository grows by about 10 MB of source models and scripts and 3.8 MB of optimized output, of
  which integration copies only the pieces levels use.
- The eight packs are listed in
  [the presentation spec's Sources section](../07-presentation.md#sources).

## Alternatives considered

- **Quaternius MegaKits (Stylized Nature, Fantasy Props, Medieval Village) and Modular
  Character Outfits - Fantasy:** CC0, but they use multi-megabyte PBR texture sets that
  don't fit the flat, vertex-colored look, and the outfits don't fit the Wayfarer's rig.
- **Quaternius Ultimate Modular Ruins:** CC0 and a good fit, but only on Google Drive, so
  it wasn't staged. It can be added to this ADR if the owner downloads it.
- **Build everything procedurally:** works for props and landmarks (18 are staged), but
  modular walls, furniture, and foliage would take far longer to build than the packs
  save.
- **Keep the greybox:** costs nothing, but the spec's art direction and the portfolio need
  a dressed world.
