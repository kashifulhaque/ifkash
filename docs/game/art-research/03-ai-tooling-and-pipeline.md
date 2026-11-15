# Art research 3: AI tooling and the production pipeline

- **Status:** Desk research that follows [research 1](01-characters-and-animation.md) and
  [research 2](02-environments-and-renderer.md). It doesn't repeat what those cover: Meshy
  and Tripo model licenses, Mixamo, Quaternius, Mesh2Motion, Sketchfab, Poly Haven,
  ambientCG, MakeHuman, store-pack exclusions, and the renderer.
- **Date checked:** 2026-11-15.
- **Changed premise:** The owner accepts AI-generated models, will install Blender and other
  tools, and wants polished, smooth animation above everything else.
- **Method:** Reading only. Nothing was installed, downloaded, generated, or signed up for.
- **Evidence marks:** **[V]** means the source page was read and the URL is given. **[U]**
  means memory, a search summary, or a third-party page that wasn't checked against the
  vendor. License claims quote the page.

## Recommendation

Use AI for *meshes* and for *first-draft motion*, and use a human-in-the-loop keyframe tool
for the *final* motion. No AI motion tool found in this research claims sword-combat
quality with weight, and none was shown doing it. The smooth, weighty feel the owner wants
comes from posing and timing, which is what Cascadeur is built for.

The proposed toolchain:

| Step | Tool | Who |
|---|---|---|
| Concept image | Any image generator, or a sketch | Owner picks |
| Mesh and PBR textures | Meshy Pro (paid, owner owns output). Hunyuan3D 2.1 locally for free drafts | Agent through API, owner approves |
| Cleanup, UVs, bakes, export | Blender 5.3, headless scripts, plus blender-mcp for inspection | Agent |
| Rig to the Mixamo skeleton | Tripo rig v3.0 with the Mixamo spec (API), or Mixamo's web auto-rigger | Agent (Tripo) or owner (Mixamo) |
| Base motion (walk, run, idle, hit, death) | Mixamo clips, already chosen in research 1 | Owner downloads |
| Draft attack motion | Uthana text-to-motion (pay per second), Meshy text-to-motion, or Move One video of the owner | Agent (text), owner (video) |
| Polish | Cascadeur Indie: AutoPosing, AutoPhysics, Inbetweening | Owner, with an agent scripting the repetitive parts |
| Game import | Existing `lf-tools assets` step, KTX2, baked clips | Agent |

**Monthly cost while art is being made:** about USD 45-60. Meshy Pro about USD 20,
Cascadeur Indie USD 19 a month (or USD 96 a year), and Uthana usage of about USD 5-20.
Optional: one month of Move One Starter at USD 18, or a Tripo plan if its rig is used at
volume. After production, the cost is zero.

**Top three risks:**

1. **Polish is human time.** Every tool here produces a draft. Weight in a swing comes from
   an animator holding a pose and settling the recovery. An agent can't judge that on a pad.
   Budget the owner's time in Cascadeur, roughly 1-3 hours per hero attack (estimate, **[U]**).
2. **AI-mesh topology deforms badly.** Generated meshes have triangle soup at the shoulders,
   elbows, and hips. Auto-riggers smooth the weights, and close-up melee shows the collapse.
   Armor that hides joints, plus Blender remeshing before rigging, reduces it.
3. **License edges.** Cascadeur's Free plan is non-commercial and can't export. Hunyuan3D 2.1
   excludes the EU, the UK, and South Korea. The TRELLIS.2 Mac port depends on a
   non-commercial background-removal model. None of these blocks the pipeline, but each
   has a wrong path that's easy to take.

## 1. AI animation and motion tools

None of the following pages shows or claims weapon combat with weight. Weapon clips exist
in libraries, but their quality is unseen. No video could be played in this research.

| Tool | What it does | Melee coverage | Output and skeleton | Price | License for a public web game |
|---|---|---|---|---|---|
| **Meshy animation library** | Preset clips on a Meshy-rigged character | 146 fighting clips: 35 "Attacking with Weapon" (Charged Slash, Triple Combo Attack, Axe Spin Attack), 18 blocks, Roll Dodge, 10 hit reactions, 11 deaths. "5 of the 146 animations in the Fighting category are included on every plan"; the rest need Pro **[V]** ([fighting library](https://www.meshy.ai/animation-library/fighting)) | GLB or FBX on Meshy's rig. Bone names are non-standard; StraySpark had to remap spine and clavicle names **[V]** ([StraySpark](https://www.strayspark.studio/blog/ai-auto-rigging-showdown-2026-tripo-meshy-cascadeur-mixamo)) | In Meshy Pro, ₹1,699 a month (about USD 20) **[V]** ([pricing](https://www.meshy.ai/pricing)) | Paid: "If you are on a premium plan, you own all assets you create with Meshy." Free: "CC BY 4.0 license (commercial use with attribution)" **[V]** ([pricing](https://www.meshy.ai/pricing)) |
| **Meshy text-to-motion** | Text prompt to a 2-10 s clip | Not shown. Docs examples are "a character waving" and "a zombie shambling forward" **[V]** | `prime` gives FBX, `swift` gives BVH. "Result files expire 3 days after the task finishes." Skeleton not documented **[V]** ([docs](https://docs.meshy.ai/api/text-to-motion.md)) | "A `prime` task costs 10 credits"; "A `swift` task costs 3 credits" **[V]** | As Meshy paid plan |
| **Tripo retarget** | Preset clips on a Tripo-rigged model | Presets listed are idle, walk, run, jump; 90+ biped presets on rig v1.0 **[V]** ([rig docs](https://developers.tripo3d.com/en/models/rig)). Weapon presets not found | Animated GLB or FBX **[U]** ([retarget docs](https://developers.tripo3d.com/en/docs/animations-retarget), search summary) | Plan credits | As Tripo paid plan (research 1) |
| **Uthana** | Text-to-motion and video-to-motion, web app and GraphQL API | Not shown. Marketed as studio-grade mocap data **[U]** | Retargets to your skeleton; formats on a separate data-spec page not read **[U]** | Text-to-motion 3.0 USD 0.10 a second; video-to-motion 3.0 USD 0.06 a second; no subscription **[V]** ([pricing](https://uthana.com/pricing)) | Paid: "a Paying Customer owns Customer Outputs"; "You may reproduce, modify, display, distribute, and commercially use your outputs in games." Free: attribution "Animation generated with Uthana (uthana.com)." **[V]** ([terms](https://uthana.com/terms)) |
| **DeepMotion Animate 3D** | Video to motion | Not shown | FBX, BVH, MP4, GLB ("GLB is available only for custom characters") **[V]** ([pricing](https://www.deepmotion.com/pricing-animate3d)) | Free 60 s a month; paid prices not on the page. Third parties list USD 15-199 **[U]** | "Freemium accounts are available for personal, non-commercial use." Paid plan required for licensed animation **[V]** |
| **DeepMotion SayMotion** | Text to motion | Not shown | As Animate 3D **[U]** | USD 9-39 a month **[U]** | Paid required **[U]** |
| **Move One (Move AI)** | Single-phone video to motion | Real performance, so weight is real if the owner acts it. Noise on fast swings is the typical failure **[U]** | FBX, USD **[U]** | Starter USD 18 (60 credits), Standard USD 48; free plan 30 one-time credits; Gen 2 "costs 2 credits per person animated per second" **[V]** ([pricing](https://docs.move.ai/knowledge/move-one-pricing)) | Not read: the Move One EULA URL returned 404. A third party says commercial use needs a paid tier **[U]** |
| **Rokoko Vision** | Webcam or phone video to motion, cleanup in Rokoko Studio | Not shown | Free: ".FBX only", "Retargeting: default meshes only". Plus adds custom characters and BVH. Mixamo skeleton preset on export **[V]** ([pricing](https://www.rokoko.com/pricing)) | Free 30 s a month; Plus USD 20 a month yearly or USD 28 monthly **[V]** | Only the Motion Library line was found: "Export and use in any of your projects, including commercial ones." Vision output terms not read **[U]** |
| **Plask, Kinetix** | Video to motion | Not shown | FBX, GLB, BVH (Plask) **[U]** | Plask USD 18-50 **[U]** | Not read. Kinetix has moved toward Snap Lens Studio plugins **[U]** |
| **HY-Motion 1.0 (Tencent, open weights)** | Text to motion, 1.0B and 0.46B models | Not mentioned; limitations list excludes multi-person and special modes **[V]** | Skeleton not named on the card **[V]** | Free; "26GB minimum VRAM" (Lite 24 GB). Says it supports macOS, but no MPS notes **[V]** ([model card](https://huggingface.co/tencent/HY-Motion-1.0)) | `tencent-hunyuan-community`, the family that excludes the EU, the UK, and South Korea **[U]** for this model (not read) |
| **Kimodo (NVIDIA)** | Controllable text-to-motion, Python API | Not shown | SOMA skeleton (and G1 for robots), so it needs retargeting **[V]** ([project](https://research.nvidia.com/labs/sil/projects/kimodo/)) | Free | License not stated on the project page; check [the repo](https://github.com/nv-tlabs/kimodo) **[U]** |

**Verdict for question 1:**

- For **drafts of attacks Mixamo lacks** (spear, halberd, parry), use Uthana. It's pay as you
  go, its paid terms explicitly cover games, and it retargets to custom skeletons. At
  USD 0.10 a second, 100 drafts of 3 s cost USD 30.
- For **moves that need real body weight** (greatsword overhead, a stagger, a heavy landing),
  the owner acting them for Move One or Rokoko Vision captures weight that text models
  don't. Read the Move One EULA before paying; it wasn't readable in this research.
- Don't run HY-Motion or Kimodo locally on this Mac. HY-Motion needs 24-26 GB of VRAM,
  which leaves nothing on a 24 GB unified-memory machine, and both are human-only research
  models with unproven combat output.
- Every draft goes through Cascadeur (question 2) before it ships.

## 2. Cascadeur

Cascadeur is a keyframe animation tool with physics-aware AI assists. *AutoPosing* places
the whole body from a few controlled points. *AutoPhysics* converts a keyed animation into
"a physically accurate version of itself" **[V]** ([CG Channel, 2026.1](https://www.cgchannel.com/?p=175211)).
*Inbetweening* and *Motion Generation* (formerly Root Motion) fill frames between key poses;
2026.2 retrained both "on larger datasets for better output" **[V]**
([2026.2 notes](https://cascadeur.com/blog/view/cascadeur-2026-2-animation-layers-easing-new-languages)).

This is the tool that matches "polished, smoothed": the owner sets the wind-up pose, the
contact pose, and the recovery, and the tool adds overlap, balance, and momentum.

**License tiers** **[V]** ([plans](https://cascadeur.com/plans)):

| Plan | Price | Commercial use, quoted | Export |
|---|---|---|---|
| Free | USD 0 | "Only non-commercial use" | CASC only; animation export "limited to 300 frames per scene and 120 joints per scene" |
| Indie | USD 19 a month or USD 96 a year | "restricted commercial use"; "The revenue must be less than $100k per year" | CASC, FBX, DAE, USD, glTF |
| Pro | USD 49 a month or USD 396 a year | Unlimited | As Indie |

- **Indie fits this project.** A free personal portfolio game has no revenue. The
  "restricted" wording is the revenue cap, according to the plans page.
- **Rent to own:** both paid plans grant "a perpetual license after one year" **[V]**
  ([CG Channel, 2026.2](https://www.cgchannel.com/2026/08/nekki-releases-cascadeur-2026-2-with-animation-layers/)).
  The yearly Indie plan at USD 96 is the cheapest path and leaves a perpetual copy.
- **Output ownership:** the EULA URL tried (`cascadeur.com/eula`) returned 404, so the
  ownership clause wasn't read. No source found claims Cascadeur takes rights in exported
  animation **[U]**.
- **macOS:** "Apple Silicon (ARM) Macs running macOS 13.3 or later are supported" **[V]**
  ([plans](https://cascadeur.com/plans)).
- **Scripting:** Cascadeur embeds Python 3.8 with a `csc` module for scene changes, and ships
  sample scripts **[V]** ([Python scripting help](https://cascadeur.com/help/tools/animation_tools/python_scripting_in_cascadeur)).
  2026.2 added "a simple MCP server", reached from the Scripts menu **[V]** (CG Channel 2026.2;
  the official 2026.2 notes don't mention it). The scripts run inside the GUI, not headless
  **[U]**. An agent can batch-import clips, apply AutoPhysics, and export, but the owner
  must have the app open.
- **Quadrupeds:** 2025.3 added "AutoPosing and the Quick Rigging Tool now include support
  for four-legged characters", in Alpha **[V]**
  ([2025.3 notes](https://cascadeur.com/blog/view/cascadeur-2025-3-new-inbetweening-quadrupeds)).
  This covers the hound.
- **Input:** 2026.2 added BVH import **[V]**, alongside FBX, so Uthana, Meshy `swift`, and Move
  One drafts all load.

## 3. Running 3D generators locally (M4 Pro, 24 GB)

| Model | Runs on this Mac? | Time per model | Output | License |
|---|---|---|---|---|
| **Hunyuan3D 2.1** (community Mac port) | Yes. Measured on "an M4 Pro with 24 GB unified memory", this machine's exact spec. Shape and PBR texture "verified end to end" **[V]** ([port README](https://raw.githubusercontent.com/VladimirTalyzin/hunyuan3d-2.1-mac/main/README.md)) | Shape 5.7 min; PBR texture 8.5 min at the only preset that fits ("6 views, 256 px"). "24 GB is tight for this stage — the run peaks at roughly 8 GB of swap" **[V]** | Mesh plus PBR. 256 px views mean soft textures; plan to re-texture or bake in Blender | Tencent Hunyuan 3D 2.1 Community License. Territory is "the worldwide territory, excluding the territory of the European Union, United Kingdom and South Korea." "Tencent claims no rights in Outputs You generate." A license is needed above "1 million monthly active users" **[V]** ([LICENSE](https://github.com/Tencent-Hunyuan/Hunyuan3D-2.1/blob/main/LICENSE)) |
| **Hunyuan3D 2.5, 3.0, 3.1** | No. Hosted only; 2.1 is the last open-weight release **[U]** ([triposr.org](https://triposr.org/blog/hunyuan3d-versions)) | Not applicable | Not applicable | Not applicable |
| **TRELLIS.2** (trellis-mac port) | Yes, PyTorch MPS. Benchmarked on M4 Pro 24 GB **[V]** ([trellis-mac](https://github.com/shivampkumar/trellis-mac)) | About 3 min 20 s per model after a 1 min 43 s load; peak about 18 GB. One run slowed to about 36 min under thermal throttling **[V]** | GLB with base color, metallic, and roughness; about 800K triangles reduced to about 200K before baking; "Hole filling is disabled" **[V]** | TRELLIS.2 weights and port code MIT. But a required dependency is "RMBG-2.0: CC BY-NC 4.0 (non-commercial; commercial use requires a license from BRIA)", and DINOv3 is "gated, review before commercial use" **[V]** |
| **Stable Fast 3D, SPAR3D** | Experimental MPS; SPAR3D needs macOS 15.2+ **[U]** | Seconds per model on CUDA **[U]** | Low-detail single-image meshes; fine for props, weak for characters **[U]** | Stability AI Community License: free under USD 1M annual revenue **[U]** ([SF3D announcement](https://stability.ai/news/introducing-stable-fast-3d)) |

**Quality versus Meshy and Tripo:** no side-by-side was found that this research could
verify. The common report is that hosted commercial models (Meshy 5/6, Tripo 3, Hunyuan 3.x)
beat the open 2.1-era weights on texture sharpness and thin parts **[U]**. The Mac port's
256 px texture views support that for Hunyuan3D 2.1.

**Verdict for question 3:** Use local generation for free *blockouts and props*: rocks,
cave formations, barrels, bones. Hunyuan3D 2.1 is the one that both runs here and has a
license that clearly allows output use outside the excluded territories. Whether serving a
game to EU players counts as use "outside the Territory" isn't settled by the text; the
license restricts use of the *model*, and the owner runs the model outside the EU, but an
owner in the EU, UK, or South Korea must not use it. Don't use the TRELLIS.2 Mac port as
packaged, because of the non-commercial RMBG-2.0 step. Use Meshy for the hero characters.

## 4. Agent-driven Blender

**Version and install.** Blender 5.3 was released on 2026-11-10 **[U]**
([Phoronix](https://phoronix.com/news/Blender-5.3-Released)). The Homebrew cask is at 5.3.0
for Apple Silicon; install it with `brew install --cask blender` (macOS 13 or later) **[V]**
([cask](https://formulae.brew.sh/cask/blender)). Blender 5.0 and later only support Apple
Silicon Macs **[U]**. The cask is easier for an agent to call (`/Applications/Blender.app/Contents/MacOS/Blender`)
and to update; the DMG from blender.org is equivalent.

**blender-mcp** (MIT) connects a running Blender to an MCP client. It exposes
`execute_blender_code`, `look` (renders views, including animation frames), `get_scene_info`,
`generate_3d` (Tripo, Hunyuan3D, Hyper3D Rodin, Meshy), and asset search for Poly Haven,
Sketchfab, and Poly Pizza **[V]** ([repo](https://github.com/ahujasid/blender-mcp)). The
README warns that `execute_blender_code` "allows running arbitrary Python code in Blender"
and to "ALWAYS save your work before using it"; the socket server has no authentication,
so keep it on localhost. Telemetry is on by default unless `DISABLE_TELEMETRY=true` **[V]**.
Its install line pipes a remote script into `sh`; prefer the manual `uvx` route.

**Headless** (`blender --background --python script.py -- args`) is the better fit for
repeatable steps: deterministic, logged, and runnable in a loop without a GUI. Use the MCP
for looking (renders of a rig under a pose) and the headless scripts for production.

What an agent can do reliably, from Blender's Python API **[U]** (standard `bpy` operators,
not tested here):

| Task | Reliability | Notes |
|---|---|---|
| Import GLB and FBX, apply transforms, set scale | High | Scripted and deterministic |
| Decimate to a triangle budget | High | Collapse decimation keeps UVs; quality drops on faces and hands |
| Voxel or QuadriFlow remesh before rigging | Medium | Gives even joints for skinning, but loses the generator's UVs, so textures must be re-baked |
| Smart UV Project and pack | Medium | Works; seams land in visible places on faces |
| Bake normal, AO, and base color from high-poly to low-poly (Cycles) | High | Needs a cage or ray distance tuned per model; the agent checks results with `look` renders |
| Bake lightmaps to a second UV set for caves | High | Same as normal bakes; slow on Cycles Metal for big scenes |
| Retarget a Mixamo clip onto a Mixamo-named rig | High | Same bone names, so it's a copy, not a retarget |
| Retarget between different skeletons | Medium | Needs a bone map and rest-pose correction; Auto-Rig Pro's Remap tool helps (USD 25-50) **[U]** |
| Automatic weights on a template skeleton | Low to medium | Works on clean meshes, fails silently on non-manifold AI meshes; the agent can't judge deformation quality |
| Export GLB with several actions as named animations | High | glTF exporter supports actions and NLA tracks; Draco is built in |
| Judge whether a swing looks weighty | Not possible | Owner on a pad |

## 5. Auto-rigging

| Tool | Platform | Skeleton | Weight quality | License |
|---|---|---|---|---|
| **Mixamo auto-rigger** | Web | Mixamo | Acceptable for armored bodies; no published test. Humanoid only **[U]** | Free, Adobe account (research 1) |
| **AccuRIG 2** | "available for Windows 8+" **[V]** ([CG Channel](https://www.cgchannel.com/2025/07/rig-and-animate-3d-characters-for-free-with-accurig-2-0/)); no macOS build found | 19-joint body plus fingers; FBX or USD with engine presets **[V]**. Mixamo naming not stated | "cleanest pure-rigging output" in a 2026 test **[V]** ([StraySpark](https://www.strayspark.studio/blog/ai-auto-rigging-showdown-2026-tripo-meshy-cascadeur-mixamo)) | Staff reply: "you can certainly use AccuRig to rig your character for use in your game"; "If you are using only your own content then that is obviously fine." ActorCore motions can't leave AccuRig **[V]** ([forum](https://discussions.reallusion.com/t/question-about-accurig-2s-eula/15034)) |
| **Tripo rig v3.0** | API or web | Tripo or **Mixamo** spec for bipeds; quadruped, avian, and others on v2.5 **[V]** ([rig docs](https://developers.tripo3d.com/en/models/rig)) | "Weight painting on the shoulders and chest is over-smoothed" (tested before v3.0) **[V]** (StraySpark) | Tripo paid plan. Auto Rigging costs 25 credits **[V]** |
| **Meshy auto-rig** | Web or API | Meshy | Hip pivots "5-15 cm" off, causing foot sliding **[V]** (StraySpark) | Meshy paid plan |
| **Rigify** | Blender, free | Rigify `DEF-` bones; needs renaming to Mixamo | Good, but needs manual placement and weight checks **[U]** | GPL tool; output is yours **[U]** |
| **Auto-Rig Pro** | Blender add-on, USD 50 full or USD 25 core | Exports humanoid FBX or glTF; Remap tool for retargeting **[U]** ([Superhive](https://superhivemarket.com/products/auto-rig-pro)) | Good with guided marker placement **[U]** | Paid tool; output is yours **[U]** |
| **UniRig** | CUDA only, "CUDA-enabled GPU with at least 8GB VRAM" **[V]** ([repo](https://github.com/VAST-AI-Research/UniRig)) | Its own; full skinning checkpoints still listed as future release **[V]** | Unproven | MIT **[V]** |

**Verdict for question 5:** No tool here gives skin weights that hold up in close-up
without a check. For the player and the boss, use Tripo v3.0 with the Mixamo spec, which
an agent can call, then fix shoulders and hips in Blender weight paint (owner, or a
volunteer with Blender skills). Fall back to Mixamo's web rigger. AccuRIG gave the cleanest
weights in the one test found, but it's Windows only. For enemies seen at a distance, take
the auto-rig as is.

## 6. Pipelines

Common to all: the agent stores the source GLB, the Blender scripts, and the clip list in
the repository; Mixamo source files stay in R2 as research 1 decided. Target counts follow
the brief.

### Player character (10-25K triangles)

1. Owner approves a concept image (front and side, A-pose, armor covering the joints).
2. Agent calls Meshy image-to-3D (Pro) at high detail, then Blender headless: remesh the
   body, decimate to 20K, re-UV, bake normals and base color from the original to 2K.
3. Agent rigs with Tripo v3.0, Mixamo spec. Owner checks a turntable of 10 test poses; fixes
   weights at shoulders and hips if needed.
4. Base clips from Mixamo (owner downloads). Attack drafts from Uthana, or Move One of the
   owner acting with a broom handle.
5. Owner polishes each attack in Cascadeur: wind-up pose, contact pose, hold, recovery,
   AutoPhysics. Agent batch-exports FBX, converts to the game's baked clips, and retimes
   against frame data.
- **Owner:** concept approval, Meshy and Tripo payments, Adobe login, acting on camera,
  Cascadeur polish, pad review.
- **Biggest risk:** shoulder deformation during overhead swings.

### Humanoid husk enemy (5-15K triangles)

Same as the player at 10K, with a MakeHuman body or a Meshy mesh. Rig with Tripo or Mixamo
and take the weights as is. Mixamo's zombie and mutant clips cover locomotion and grabs;
Uthana drafts the shamble attacks; Cascadeur only for the two or three signature attacks.
- **Biggest risk:** reused Mixamo zombie clips look generic.

### Hound (5-15K triangles)

1. Meshy or Tripo mesh from a concept image.
2. Rig with Tripo v2.5 quadruped (agent, API), or Cascadeur's quadruped Quick Rigging Tool
   (Alpha). No Mixamo skeleton exists for four legs.
3. Walk and run from Mesh2Motion's CC0 quadruped clips, retargeted in Blender.
4. Owner keys bite, lunge, and stagger in Cascadeur with quadruped AutoPosing.
- **Biggest risk:** text-to-motion and video tools are humans only, so every hound attack is
  hand-keyed.

### Armored humanoid boss with a lantern (20-60K triangles)

1. Meshy mesh at 40K, lantern generated as a separate prop.
2. Tripo rig, Mixamo spec, scaled. Lantern parented to the hand bone in the game, with its
   light driven by code; a chain swing can be a two-bone physics chain at runtime or keyed.
3. Owner keys the heavy moves in Cascadeur: long wind-up held at the peak, then a fast
   release, per the anticipation-hold plan in research 1. AutoPhysics settles the landing.
- **Biggest risk:** boss weight needs the most hand-keying; plan 8-15 polished clips.

### Cave environment

1. Textures from Poly Haven and ambientCG (research 2).
2. Rock walls, stalagmites, and props generated locally with Hunyuan3D 2.1 (free) or Meshy;
   agent decimates, bakes normals, and builds a modular kit.
3. Agent bakes lightmaps to a second UV set in Blender headless, then exports GLB through
   the existing KTX2 step.
- **Owner:** layout approval in the game.
- **Biggest risk:** AI rocks look blobby at scale; tiling textures with trim sheets may beat
  generated meshes.

### What agents can and can't do unattended

- **Unattended:** Meshy and Tripo API calls (once a key is in `.env`), local Hunyuan3D runs,
  all Blender headless steps, retarget and bake scripts, GLB export, `lf-tools` conversion,
  Uthana API drafts, contact-frame and retiming data.
- **Owner only:** accounts and payments (Meshy, Tripo, Uthana, Cascadeur, Adobe, Hugging Face
  for gated weights), downloading from Mixamo, acting on camera, final animation polish,
  and judging feel on a pad.

### What the owner installs or signs up for

- Install: Blender 5.3 (`brew install --cask blender`), Cascadeur (download from
  cascadeur.com, needs an account), `uv` for blender-mcp. Optional: Hunyuan3D 2.1 Mac port
  (about 30 GB of weights).
- Sign up: Meshy Pro, Cascadeur Indie (yearly), Uthana credits, Adobe (Mixamo), Tripo API
  credits. Optional: Move One Starter for one month.

## What this research couldn't verify

- Any animation's look. No video was played, and no tool was run.
- The Move One EULA and the Cascadeur EULA (both URLs returned 404). Rokoko Vision output
  terms beyond the motion library line.
- Uthana's output formats and the skeletons its retargeter supports.
- The HY-Motion license text and Kimodo's license.
- Whether Cascadeur's Python or MCP server can run without the GUI.
- Blender 5.3 details from blender.org (the download page returned 403).
- The quality gap between Hunyuan3D 2.1 local output and Meshy.
