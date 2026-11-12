# Quaternius Universal Animation Library

- **Source:** https://quaternius.itch.io/universal-animation-library (the Standard zip, 15,904,933 bytes, downloaded 2026-11-12). Pack page: https://quaternius.com/packs/universalanimationlibrary.html
- **Author:** Quaternius
- **License:** CC0 1.0 Universal. The author's `License.txt` is `LICENSE.txt` here, and the upstream readme is `README.upstream.txt`. No attribution is required.
- **Rig:** The Unreal mannequin-style humanoid (`root`, `pelvis`, `spine_01` to `spine_03`, `thigh_l`, and so on). The renderer retargets it to the Quaternius modular-character rig by bone name.

## What is committed

`ual1_standard_trimmed.glb` is `Unreal-Godot/UAL1_Standard.glb` with the meshes and skin removed and only these 13 clips kept: `Idle_Loop`, `Walk_Loop`, `Jog_Fwd_Loop`, `Sprint_Loop`, `Jump_Start`, `Jump_Loop`, `Jump_Land`, `Roll`, `Hit_Chest`, `Death01`, `Sword_Idle`, `Sword_Attack`, `Interact`. The full library is not committed (63 MB unpacked).

The renderer measures each locomotion clip's stride from the planted foot at load time. The library's own root-motion variant (`UAL1_Standard_RM.glb`) is not used: its root distances are round numbers that don't match the feet, so they made the run clip slide.

Regenerate it from a fresh download:

```sh
bun game/scripts/trim-glb.mjs UAL1_Standard.glb game/assets-src/universal-animation-library/ual1_standard_trimmed.glb \
  --keep Idle_Loop,Walk_Loop,Jog_Fwd_Loop,Sprint_Loop,Jump_Start,Jump_Loop,Jump_Land,Roll,Hit_Chest,Death01,Sword_Idle,Sword_Attack,Interact
```
