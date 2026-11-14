# Quaternius Universal Animation Library 2

- **Source:** https://quaternius.itch.io/universal-animation-library-2 (the Standard zip, `Universal Animation Library 2[Standard].zip`, 18,735,003 bytes, downloaded 2026-11-14). Pack page: https://quaternius.com/packs/universalanimationlibrary2.html
- **Author:** Quaternius
- **License:** CC0 1.0 Universal. The author's `License.txt` is `LICENSE.txt` here, and the upstream readme is `README.upstream.txt`. No attribution is required.
- **Approval:** The owner approved the pack on 2026-11-14 ([ADR 0009](../../../docs/game/decisions/0009-animation-library-2.md)).
- **Rig:** The same Unreal mannequin-style humanoid as Library 1, so the renderer retargets it with the same bone map.

## What is committed

`ual2_standard_trimmed.glb` is `Unreal-Godot/UAL2_Standard.glb` with the meshes and skin removed and only these 19 clips kept: `Sword_Regular_A`, `Sword_Regular_A_Rec`, `Sword_Regular_B`, `Sword_Regular_B_Rec`, `Sword_Regular_C`, `Sword_Heavy_Combo`, `Sword_Dash`, `Sword_Block`, `Idle_Shield_Loop`, `Idle_Shield_Break`, `Shield_Dash`, `Shield_OneShot`, `Melee_Hook`, `Melee_Hook_Rec`, `OverhandThrow`, `TreeChopping_Loop`, `Consume`, `NinjaJump_Start`, `NinjaJump_Land`. The full library is not committed.

To regenerate it from a fresh download, run the following command from the repository root:

```sh
bun game/scripts/trim-glb.mjs UAL2_Standard.glb game/assets-src/universal-animation-library-2/ual2_standard_trimmed.glb \
  --keep Sword_Regular_A,Sword_Regular_A_Rec,Sword_Regular_B,Sword_Regular_B_Rec,Sword_Regular_C,Sword_Heavy_Combo,Sword_Dash,Sword_Block,Idle_Shield_Loop,Idle_Shield_Break,Shield_Dash,Shield_OneShot,Melee_Hook,Melee_Hook_Rec,OverhandThrow,TreeChopping_Loop,Consume,NinjaJump_Start,NinjaJump_Land
```

Then run `lf-tools assets` to write `static/latticefall/actors/ual2.glb`.
