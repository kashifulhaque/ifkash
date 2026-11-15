# Art research 1: characters, enemies, bosses, weapons, and animation

- **Status:** Desk research for [the art direction research plan](../art-direction-research.md),
  questions 1 (characters, enemies, bosses, weapons, animations) and 3 (how the animations
  work). Environments, textures, and renderer feasibility are in a separate document.
- **Date checked:** 2026-11-15, for every license and price in this document.
- **Method:** Reading and comparing only. No packs were downloaded, no accounts were made,
  and no models were generated. No video could be played in the research environment, so
  every statement about how an animation looks is second-hand. See
  [What this research couldn't verify](#what-this-research-couldnt-verify).

A few art terms, used throughout:

- A *triangle count* is how many flat faces a model has. More triangles means a smoother
  shape and more work for the graphics card.
- *PBR textures* (physically based rendering) are a set of images per model: color,
  surface bumps (the normal map), and how rough or metallic each spot is.
- A *skeleton*, or *rig*, is the set of named bones inside a model. *Skinning* ties each
  part of the surface to those bones so the model bends when the bones move.
- *Retargeting* copies an animation made for one skeleton onto another skeleton.
- *Mocap* (motion capture) is animation recorded from a real performer. *Hand-keyed*
  animation is posed by an animator, frame by frame.

## Recommendation

**No paid store pack can ship in this game as it's built today.** Fab, the Unity Asset
Store, the Sketchfab store, Reallusion, and MoCap Online all require that players can't
pull the files back out of the game. A web game that serves plain GLB files from a public
site, built from a public repository, can't meet that. This rules out every well-known
souls-like animation pack (Kubold, Frank Climax, RancidMilk's store packs, and the rest),
however good they look. The stack has to come from public-domain (CC0) sources,
attribution (CC BY) sources, generated assets the owner owns, and Mixamo under conditions.

The proposed stack:

- **Skeleton:** The Mixamo skeleton for every humanoid, with the existing bone-name
  retargeting kept so the Quaternius clips still work as a fallback.
- **Animation:** Mixamo's mocap library (free, needs an Adobe account) for sword and
  shield, greatsword, axe, bow, casting, rolls, hit reactions, and deaths. A build step
  converts the clips into the game's own baked clip format, and the raw Mixamo files stay
  out of the public repository. Spear, halberd, dagger, and parry clips are thin on
  Mixamo. Fill them with the Quaternius clips or with one month of video-to-motion
  capture.
- **Player, armor, humanoid enemies, and humanoid bosses:** Generate the meshes with a
  paid AI 3D generator plan (Meshy or Tripo), which makes the owner the owner of the
  output, and rig them all to the one skeleton. Use MakeHuman (CC0) bodies for unarmored
  characters such as Husks. Mixamo's own character library is the fallback if generated
  quality fails the prototype.
- **Creatures and non-human bosses:** Generated meshes, animated with CC0 creature rigs
  from Mesh2Motion where one fits (hounds, wolves, crawlers), and with the existing
  procedural animation code for the rest. Eight of the 12 bosses are humanoid and use the
  shared skeleton. The Unspooled's second phase, the Lamplight Matriarch, the Mossgut
  Brood, Cave Moths, and Lamp Mites have no good animation source and stay procedural.
- **Weapons:** CC BY weapon sets from Sketchfab (swords, polearms, bows, shields),
  generated models for the catalysts and anything missing.

**Total price:** About USD 40-100. That's two or three months of a paid generator plan
(the Meshy Pro plan showed ₹1,699 a month, roughly USD 20), and optionally one month of a
video-to-motion app (Move One listed USD 15). Everything else is free.

**What the owner has to do:**

- Make an Adobe account and download from Mixamo by hand. Mixamo has no API, and an agent
  can't enter the password.
- Buy a generator plan for the months the art is being made, and keep the invoices and a
  copy of the terms from those dates.
- Approve keeping Mixamo source files in a private store (for example the R2 bucket), not
  in the public repository.

**Biggest risks:**

1. **Quality is unproven.** Nobody has looked at a generated knight or a Mixamo greatsword
   swing in this game yet. The prototype must compare them on a pad before anything is
   adopted.
2. **The Mixamo license is a reading, not a grant.** Adobe allows distribution inside an
   end product and forbids stand-alone distribution. A baked clip format is a defensible
   reading of "inside the product", but Adobe hasn't confirmed it for web games.
3. **Two animation styles in one game.** Mocap humanoids next to procedural creatures and
   bosses can look mismatched. The souls look depends on the animation more than on the
   models.

## Shortlist

License verdicts use two tests. Test (a) is shipping the files in a public web game. Test
(b) is committing the files to the public repository. The evidence for each verdict is in
[License evidence](#license-evidence).

### Characters, armor, and humanoid enemies

| Source | Covers | Style fit | Triangles and textures | Rig | Price | Verdict |
|---|---|---|---|---|---|---|
| AI generators on a paid plan: [Meshy](https://www.meshy.ai), [Tripo](https://www.tripo3d.ai) | Anything described in text or a concept image: armor sets, knights, husks, cultists, creatures, bosses | Partial. Reviews report good overall shape and weak faces, hands, and hairlines. One prompt style keeps the set consistent. Helmets and hoods hide the weak spots | Chosen at export. Meshy's remesh tool is reported to go from 1,000 to 300,000 triangles. PBR textures built in | Built-in auto-rig, reported as basic. Re-rig to the shared skeleton | Meshy Pro ₹1,699 a month (about USD 20) for 1,000 credits. Tripo Pro reported at USD 19.90 a month | (a) allowed, (b) allowed |
| [MakeHuman and MPFB](https://static.makehumancommunity.org/mpfb.html) | Realistic human bodies of any build, with simple clothes. No armor | Good for bodies. Husks, the Drowned, and bodies under armor | About 13,000-28,000 triangles before reduction (from memory), clean layout | Several skeleton presets, including a game rig. Runs inside Blender, so an agent can script it | Free | (a) allowed, (b) allowed (CC0, not re-verified, see the last section) |
| [Mixamo characters](https://www.mixamo.com) | About 100 ready characters already on the Mixamo skeleton: knights, a paladin, an archer, a wizard, zombies, mutants, peasants (from memory) | Good. They were made for games of about this era, with color, normal, and specular maps | Not published. Must be measured after download | Mixamo, no extra work | Free, Adobe account | (a) allowed with conditions, (b) not allowed |
| [Sketchfab CC BY models](https://sketchfab.com) | Single models. For example [Medieval Knight, rigged for UE4/5](https://sketchfab.com/3d-models/medieval-knight-rigged-for-ue45-1c7a83c0832a4ab28d9f1a46d282fcf9) (51,054 triangles), [Hospitalier Knight, Mixamo rigged](https://sketchfab.com/3d-models/hospitalier-knight-mixamo-rigged-game-ready-6a6657a61d694fa59700da7f05d0219b) (43,901), [Undead Knight](https://sketchfab.com/3d-models/undead-knight-low-poly-0453f93869af4776ac69d9ef896aa6a1) (15,656), [Armored King](https://sketchfab.com/3d-models/armored-king-e3978fb660cf47278485d44eeba488ef) (17,291) | Partial. Each one can look right, but every author has a different style, so a full cast looks like a patchwork | Per model, from the Sketchfab API | Varies. Some unrigged | Free | (a) allowed with credit, (b) allowed with credit |
| [Quaternius Universal Base Characters](https://quaternius.itch.io/universal-base-characters) and [Modular Character Outfits: Fantasy](https://quaternius.itch.io/modular-character-outfits-fantasy) | 6-8 bodies, 12 modular outfits (knight, noble, wizard, peasant) | Poor. Listed as stylized. Realistic proportions, but not realistic surfaces | About 13,000 triangles average for the bodies | Works with the animation library in use | Free, or USD 19.99-20 for source files | (a) allowed, (b) allowed |
| Fab and Unity Asset Store character packs | Many realistic modular knights and undead | Good | Per listing | Mostly Unreal mannequin | USD 20-150 each | (a) not allowed as plain files, (b) not allowed |

### Creatures

| Source | Covers | Style fit | Triangles and textures | Rig | Price | Verdict |
|---|---|---|---|---|---|---|
| [Mesh2Motion](https://mesh2motion.org/) | Skeletons and over 170 animations for humans, quadrupeds (fox, horse), birds, dragons, spiders, snakes, and fish. You bring the mesh | Unknown until seen. The listing doesn't show attack clips per creature | Not applicable (animation only) | Its own, exported as GLB or FBX | Free | (a) allowed, (b) allowed (CC0) |
| AI generators (as in the preceding table) | Hound, wolf, crawler, moth, golem, mite, and boss meshes | Partial. Creatures are more forgiving than human faces | Chosen at export | Meshy reports humanoid and quadruped auto-rig | In the plan | (a) allowed, (b) allowed |
| Sketchfab CC BY creatures | [Realistic Animated Elk](https://sketchfab.com/3d-models/realistic-animated-elk-3d-model-787834f9caa2474d9f1814b807c072d7) (7,886 triangles, 53 animations), [Animated Spider](https://sketchfab.com/3d-models/animated-spider-af87017501fc44e39a33c220f2435100) (20,588 triangles, 7 animations: walks, run, attack, die), several mimic chests | Partial. The elk's author says it has known rigging problems | Per model | Per model | Free | (a) allowed with credit, (b) allowed with credit |
| Existing procedural code (`render/procedural/`, `render/bosses/`) | Any shape, animated from the move's frame number | Poor for mammals, fine for abstract or insect-like things | Not applicable | Code-built | Free | Not a license question |

### Weapons

| Source | Covers | Style fit | Triangles and textures | Price | Verdict |
|---|---|---|---|---|---|
| Sketchfab CC BY sets by one author: [10 Swords](https://sketchfab.com/3d-models/10-swords-022672498cd1481da0a1bd046c7d57b7), [10 Polearms](https://sketchfab.com/3d-models/10-polearms-31f7cd2d75914dc2af54a360039a1a05), [10 Bows and Crossbows](https://sketchfab.com/3d-models/10-bows-and-cross-bows-1774a072f1cd45bf9c0875c1a23919a7) | Straight, great, curved, thrusting, and colossal swords; spears and halberds; bows | Good. Realistic shapes, one material each | About 2,500-5,400 triangles per weapon. 4K textures, to be shrunk to 1K | Free | (a) and (b) allowed with credit |
| Other Sketchfab CC BY singles: [Medieval Kite Shield](https://sketchfab.com/3d-models/medieval-kite-shield-ef454e3700a7462eb27af245e685529a) (5,644), [Medieval Spear](https://sketchfab.com/3d-models/medieval-spear-f61070945153418fa11c4abae286b292) (2,270), [War hammer](https://sketchfab.com/3d-models/war-hammer-25aab91fbf8145059a01fb8863e49b46) (5,263), [Thief's Dagger](https://sketchfab.com/3d-models/thiefs-dagger-e1b75164b5084df8a32303c491d61f0b) (9,904) | Shields, hammer, dagger, axe | Good, with mixed authors | As listed | Free | (a) and (b) allowed with credit |
| AI generators | Catalysts (stylus, reliquary), fist and claw weapons, bucklers, greatshields, boss weapons, the bell | Good. Rigid props are what generators do most reliably | Chosen at export | In the plan | (a) and (b) allowed |

Skip any Sketchfab model that copies a named game's weapon or character (several results
were labeled as copies from *Elden Ring* and *For Honor*). The uploader's CC BY label
doesn't give rights they don't hold, and the project's IP rules forbid it.

### Animation

| Source | Covers | Style fit | Rig | Price | Verdict |
|---|---|---|---|---|---|
| [Mixamo](https://www.mixamo.com) | About 2,400 mocap clips (from memory): sword and shield, greatsword, axe, longbow, and magic packs; rolls, dodges, hit reactions, knockdowns, deaths, idles, and locomotion. Thin on spear, halberd, dagger, and parry | Partial to good. Real performers with weapon props, so feet stay planted and swings carry the body. Wind-ups are shorter than in the target games | Mixamo | Free, Adobe account | (a) allowed with conditions, (b) not allowed |
| [Quaternius Universal Animation Library 1 and 2](https://quaternius.itch.io/universal-animation-library-2) | Already in the game | Poor. The owner rejected it on 2026-11-15. Keep only as a fallback for missing clips | Unreal mannequin | Free | (a) and (b) allowed (CC0) |
| Video-to-motion apps: [Move One](https://www.move.ai), DeepMotion | Any move the owner acts out on a phone camera, for example spear thrusts with a broom handle | Unknown. Real weight, but single-camera capture is noisy and needs cleanup | Exports to common rigs, Mixamo included | Move One listed a free plan and USD 15 a month | (a) and (b) likely allowed (the user owns the output; not fully read) |
| [Mesh2Motion](https://mesh2motion.org/) human clips | General human clips. No melee listed | Unknown | Its own | Free | (a) and (b) allowed (CC0) |
| [Kubold](https://www.kubold.com/faq-unity) animsets (One Handed Sword Animset Pro USD 30, Longsword Animset Pro USD 50) | Mocap sword sets, the closest known match to the target feel | Good by reputation | Same bone names as Mixamo | USD 30-50 each | (a) not allowed as plain files, (b) not allowed |
| [RPG Animations Pack](https://assetstore.unity.com/packages/3d/animations/rpg-animations-pack-282348) by DoubleL and similar Unity or Fab packs | Souls-like sets with rolls, hits, and several weapon stances | Good by description | Humanoid | USD 130 | (a) not allowed as plain files, (b) not allowed |
| [MoCap Online](https://mocaponline.com/pages/license) | Mocap sword packs | Good by description | Several | Varies | (a) not allowed, (b) not allowed |
| [RancidMilk free library](https://rancidmilk.itch.io/free-character-animations) | Over 2,000 clips converted from the Carnegie Mellon mocap database: everyday motion, sports, some fighting | Poor. Not weapon combat | Retargeted to a Quaternius model | Free | (a) and (b) allowed, no selling the clips |

## License evidence

Each row gives the deciding clause, kept short, and how it was read. "Browser" means the
live page was opened and the text was read directly. "Fetch" means a fetch tool returned
the text through a summarizing step, which can misquote. "Search" means a search result
snippet only.

| Source | Deciding text | URL | How read | Verdict |
|---|---|---|---|---|
| Fab Standard License (covers the former Unreal Marketplace, Quixel, and the Sketchfab store on Fab) | "you must restrict end users from extracting or otherwise using Content". The summary also says use "is not limited to Unreal Engine", and it allows sharing only through a private repository with collaborators. EULA last updated 2024-10-01 | https://www.fab.com/eula | Browser | (a) Not allowed as plain GLB. (b) Not allowed |
| Fab CC BY listings | The summary says CC BY listings are governed by the Creative Commons license | https://www.fab.com/eula | Browser | (a) and (b) allowed with credit |
| Mixamo FAQ | "royalty free for personal, commercial, and non-profit projects", with video games listed. Says nothing about redistribution. Last updated 2021-09-14 | https://helpx.adobe.com/creative-cloud/faq/mixamo-faq.html | Browser | See the next row |
| Adobe General Terms of Use, "Content Files" (the terms that govern Mixamo) | "under no circumstances can you distribute the Content Files on a stand-alone basis". Distribution is allowed "only in connection with your End Use". Published 2025-10-03 | https://www.adobe.com/legal/terms.html | Browser | (a) Allowed with conditions: ship only data built into the game, in the game's own baked format. (b) Not allowed: raw files in a public repository are stand-alone distribution |
| Unity Asset Store EULA, section 2.2.1 | The license is to "distribute the Asset as incorporated and embedded in that Licensed Product". Anything else is forbidden "except as expressly permitted". Last updated 2024-12-04 | https://unity.com/legal/as-terms | Browser | See the next row |
| Unity support article on other engines | Assets aren't restricted to Unity projects, but you can't distribute them in a way that lets others extract the raw files (paraphrased by the fetch tool) | https://support.unity.com/hc/en-us/articles/34387186019988-Can-I-use-assets-from-the-Asset-Store-with-other-engines | Fetch | (a) Not allowed as plain GLB, on Unity's own reading. (b) Not allowed |
| Kubold FAQ (publisher's reading of the same EULA) | "the license is for YOU only". The FAQ names sharing project files on GitHub as not allowed, and describes allowed games as compiled and built | https://www.kubold.com/faq-unity | Fetch | (a) Not allowed without written permission. (b) Not allowed |
| Sketchfab Standard license (store models) | Forbids making the model available in a way "intended to allow or invite a third party to download, extract, redistribute" | https://sketchfab.com/licenses | Fetch | (a) Unclear, leaning not allowed. (b) Not allowed |
| Sketchfab CC BY and CC0 models | Standard Creative Commons licenses, shown per model by the Sketchfab API | https://sketchfab.com | API | (a) and (b) allowed. CC BY needs credit, which `lf-tools credits` already handles |
| Reallusion content EULA (ActorCore, Character Creator) | Shipped content must be in formats that "cannot be opened, extracted, or imported into third-party 3D software". Updated 2026-11-01 | https://www.reallusion.com/Content/EULA/EULA.htm | Fetch | (a) Not allowed. (b) Not allowed |
| MoCap Online license | "Distribution is allowed only as binary-only." Integration must prevent extraction | https://mocaponline.com/pages/license | Fetch | (a) Not allowed. (b) Not allowed |
| Meshy terms, section 3.2 | "customers on a paid Meshy plan own their Customer Output". Free-plan output is CC BY 4.0 with credit to Meshy | https://www.meshy.ai/terms-of-use | Fetch | (a) and (b) allowed. Free plan needs credit |
| Tripo terms, section 5.2.2 | "Rights in Paid Content belong to you". Paid status survives the end of the subscription. Free output is under a public license. Last updated 2026-11-13 | https://www.tripo3d.ai/terms | Browser | (a) and (b) allowed on a paid plan |
| Hunyuan3D 2.1 community license (open model, run locally) | Tencent "claims no rights in Outputs You generate". The license doesn't apply in the European Union, the United Kingdom, or South Korea | https://github.com/Tencent-Hunyuan/Hunyuan3D-2.1/blob/main/LICENSE | Fetch | (a) and (b) allowed outside those territories |
| Quaternius packs | "Free to use in personal, educational and commercial projects" (CC0) | https://quaternius.itch.io/universal-base-characters | Fetch | (a) and (b) allowed |
| Mesh2Motion | "Free for personal or commercial use. Licensed under CC0." The tool's code is MIT | https://mesh2motion.org/ | Fetch | (a) and (b) allowed |
| RancidMilk free library (Carnegie Mellon data) | "Free to use/modify/redistribute so long as you don't sell the animations themselves." | https://rancidmilk.itch.io/free-character-animations | Fetch | (a) and (b) allowed |
| Bandai Namco Research motion dataset | CC BY-NC 4.0 (non-commercial) | https://github.com/BandaiNamcoResearchInc/Bandai-Namco-Research-Motiondataset | Fetch | Allowed only for non-commercial use. A portfolio site that helps a career is a gray area. Avoid |
| Move.ai (Move One) | "you own and shall retain ownership of all Intellectual Property Rights" in the outputs | https://docs.move.ai/knowledge/eula-creator-plan-1 | Search | (a) and (b) likely allowed. Read the full EULA before paying |

Two notes on these verdicts:

- **"Not allowed as plain GLB" isn't "never".** A store publisher can give written
  permission, and some do for web games. That's a per-pack email, and the answer has to be
  kept with the asset.
- **Hiding the files isn't a fix.** Packing store assets in a scrambled format doesn't
  hold up when the code that unscrambles them is in a public repository.

## Coverage

"Humanoid" means the shared skeleton with Mixamo clips. "Generated" means an AI-generated
mesh on a paid plan. "Procedural" means the existing code-driven animation in
`render/procedural/` or `render/bosses/`, with a new mesh.

### Enemies and elites

| ID | Model source | Animation source | Gap |
|---|---|---|---|
| `husk` and variants | MakeHuman body with generated rags, lantern, and frost variants | Humanoid: zombie and injured walks, slow two-hit swing | None expected |
| `thorn_hound` | Generated hound with thorns | Mesh2Motion quadruped rig (fox) | Lunge and retreat clips unconfirmed. Procedural fallback |
| `lantern_thief` | Generated | Humanoid: throw, run away | None expected |
| `acolyte` | Generated robed figure | Humanoid: Mixamo casting clips | Long robes skin badly. Keep robes above the ankle |
| `drowned` | MakeHuman body, generated bloat and weeds | Humanoid: slow walk, grab | Grab needs a paired pose on the player. Use a generic grab |
| `reed_stalker` | Generated lanky creature on the humanoid skeleton | Humanoid: crouch, lunge | Partial. Stretched limbs can distort mocap |
| `marsh_crawler` | Generated, or the CC BY Animated Spider | Mesh2Motion spider rig, or the spider's own 7 clips | Bite clip unconfirmed |
| `cave_moth` | Generated | Procedural hover and flutter | **Gap:** no clip source. Stays procedural |
| `lamp_mite` | Generated, very low detail | Procedural | **Gap:** stays procedural. Fine at this size |
| `shard_golem` | Generated, built from rigid chunks | Humanoid: slow heavy swings, played at a slower rate | None expected. Rigid chunks skin cleanly |
| `glasswing_stalker` | Generated | Humanoid or procedural | Partial. The shimmer effect matters more than the mesh |
| `gray_knight` | Generated plate armor, or the CC BY Medieval Knight | Humanoid: sword and shield pack | None expected |
| `gray_archer` | Generated | Humanoid: longbow pack | None expected |
| `rime_wolf` | Generated wolf | Mesh2Motion quadruped rig | Circling and lunge unconfirmed. Procedural fallback |
| `whitewarden_skeleton` | Generated, or a CC BY skeleton warrior | Humanoid, plus a collapse and a rise for the revive | None expected |
| `mimic_chest` | A CC BY mimic, or generated | Procedural lid and tongue | **Gap:** stays procedural. One in the game |
| `bramble_brute` | Generated, large | Humanoid, scaled up: heavy combos | None expected |
| `choir_knight` | Generated | Humanoid: polearm | **Gap:** halberd clips are thin on Mixamo. Use greatsword clips or captured ones |
| `gray_captain` | Generated | Humanoid: greatsword pack | None expected |
| `foundry_smith` | Generated | Humanoid: axe and overhead slam clips | None expected |
| `bell_warden` | Generated | Humanoid: one-hand swings | Bell-strike pose needs a custom or captured clip |

### Bosses

| ID | Boss | Plausible source | Verdict |
|---|---|---|---|
| B01 | Hollow Sentry | Generated armor on the humanoid skeleton. Sword and shield and overhead clips | Covered |
| B02 | The Briar Stag | The CC BY elk (53 clips, known rig problems) with generated thorns, or a generated stag on the Mesh2Motion quadruped rig. Charge, toss, and stomp are likely procedural | Partial. Hardest of the animal bosses |
| B03 | Bridgewarden Corvane | Generated knight on the humanoid skeleton | Covered, except halberd clips (thin) |
| B04 | Mossgut Brood | Generated mass. It doesn't move in phase 1 and crawls in phase 2 | **No clip source.** Stays procedural, which suits it |
| B05 | Ossuary Keeper | Generated skeletal warden on the humanoid skeleton | Covered |
| B06 | The Drowned Choir | Three generated figures on the humanoid skeleton: caster, heavy, and fast flanker | Covered |
| B07 | Castellan Voss | Generated lord on the humanoid skeleton. Greatsword pack, kick, leap | Covered. The showcase duel, so it gets the best clips |
| B08 | Forgeheart Colossus | Generated golem of rigid parts. Humanoid skeleton scaled up and slowed, or procedural | Partial. Mocap on a cavern-sized body can look weightless. Test both |
| B09 | The Lamplight Matriarch | Generated moth. Mesh2Motion has bird and dragon rigs with flight | **No good source.** Likely stays procedural |
| B10 | The Bellwright | Generated giant smith on the humanoid skeleton | Covered. The bell on a chain is procedural |
| B11 | The Bound Exile | Generated sorcerer on the humanoid skeleton. Casting clips | Covered |
| B12 | The Unspooled | Phase 1: a tall generated figure on the humanoid skeleton. Phase 2: a many-limbed thread shape | Phase 1 covered. **Phase 2 has no source** and stays procedural. An abstract thread shape suits code better than a model |

Nothing needs to be re-themed. If the prototype shows procedural creatures clashing with
the mocap humanoids, the candidates to re-theme toward a humanoid are B09 and B02.

## Skeleton and animation pipeline

### Skeleton standard

Use the **Mixamo skeleton** as the one standard for every humanoid. The following table
compares the options:

| Skeleton | Animation sources that target it | Retargeting effort | Notes |
|---|---|---|---|
| Mixamo | Mixamo's library. Kubold's files use the same bone names. Most video-to-motion apps export to it | Low. Mixamo's free auto-rigger puts this exact skeleton into any humanoid mesh you upload, so generated and CC BY models all end up identical | Recommended |
| Unreal mannequin (UE4 and UE5) | Quaternius libraries (CC0), and nearly every Fab animation pack | None today: the code assumes it. But the Fab packs can't ship, so the only usable clips on it are the rejected ones | Keep as a fallback source |
| Unity Humanoid | Unity Asset Store packs | Not a real skeleton. It's a mapping layer inside Unity, so it needs Unity to convert | Not useful outside Unity |
| VRM | Avatar apps and anime-style models | Low, but almost no melee clips target it | Wrong style |
| Reallusion (Character Creator, ActorCore) | ActorCore's mocap store | Not relevant: the license blocks shipping | Rejected |

What changes in the code:

- `render/actors/retarget.ts` maps 20 Unreal-style source bones (`pelvis`, `spine_01`,
  `upperarm_l`) to the current model's bones. A Mixamo source needs a second map with the
  Mixamo names (`Hips`, `Spine`, `LeftArm`, and so on, each with the `mixamorig:` prefix).
  The method (world-space rotation differences) already copes with different rest poses,
  so the Mixamo T-pose and the Unreal A-pose both work.
- If every model uses the Mixamo skeleton, Mixamo clips need no retargeting at all, and
  only the Quaternius fallback clips go through the map.
- `render/actors/moveClips.ts` carries over. Each move gets new clip names and contact
  frames.

### Build pipeline

1. **Mesh:** Generate or download the model. Reduce it to the triangle budget and shrink
   textures to 1K or 2K in the existing `lf-tools assets` step.
2. **Rig:** Put the shared skeleton in. The dependable path is Mixamo's auto-rigger, by
   hand, about five minutes per character for roughly 30 humanoid meshes. The scripted
   path is headless Blender: fit one template skeleton to the mesh and skin it with
   automatic weights. That works for an agent but gives worse shoulders and hips, and
   needs a test.
3. **Clips:** Download Mixamo clips once, "without skin" (animation only), into a private
   store. A build script bakes them to the same 30 Hz rotation arrays that `retarget.ts`
   produces at load today, and writes one binary file per clip set.
4. **Ship:** The public site serves the baked file and the game's own models. The public
   repository holds the scripts, the clip list, and the baked output only if the owner
   accepts that reading of the Adobe terms. Otherwise the baked file also lives in R2.

Because the repository is public and Cloudflare Pages builds from it, anything kept out of
the repository must come from the R2 bucket at build time or at run time. That's a small
pipeline change the ADR has to include.

### Retiming against the frame data

The frame data never changes. At 30 ticks per second, a straight sword light attack is 10
startup, 3 active, and 14 recovery ticks (0.9 s in total). A heavy is 18, 4, and 20
(1.4 s). Enemy heavy wind-ups are 18-36 ticks (0.6-1.2 s).

The following estimates come from typical mocap clip lengths, not from measured files.
The prototype must measure them:

| Move | Game timing | Typical mocap clip | What the retimer does |
|---|---|---|---|
| Player light attack | 0.33 s wind-up, 0.47 s recovery | 0.35-0.5 s wind-up, 0.5-0.9 s recovery | Wind-up 1.0-1.5 times faster. Recovery 1.1-1.9 times faster, inside the 2.5 limit |
| Player heavy attack | 0.6 s wind-up, 0.67 s recovery | 0.6-0.9 s wind-up, 0.9-1.5 s recovery | Wind-up 1.0-1.5 times faster. Recovery 1.3-2.2 times faster, close to the limit |
| Enemy or boss heavy | 0.6-1.2 s wind-up | 0.5-0.9 s wind-up | Wind-up up to 2 times *slower* |
| Roll | 24 ticks (0.8 s) | 0.9-1.3 s | 1.1-1.6 times faster |

Two findings follow from this:

- **Player attacks are quick by the target games' standards.** The frame data gives a
  light attack a third of a second of wind-up. No clip can show a long wind-up in that
  time. Weight on the player's side has to come from the pose at contact, the hitstop that
  already exists, and a recovery that visibly settles. Mocap helps with all three.
- **Slow wind-ups need a hold, not slow motion.** `moveClipFrame` stretches the wind-up
  evenly. Stretching a 0.6 s wind-up to 1.2 s looks like the enemy is moving through water.
  The target games reach a raised pose quickly and then hold it. Add an optional
  anticipation-pose frame to each `ClipSpec`, so the clip plays to that pose at normal
  speed, holds with a slight drift, and releases on time. This is a small change to the
  timing layer and likely matters more for the souls feel than the choice of pack.

### Creatures and bosses

- **Humanoid bosses and elites** (8 bosses, 5 elites) use the shared skeleton and the same
  clip pool as the player, with their own contact frames and holds. Scale the skeleton for
  giants and play clips slower.
- **Four-legged creatures** (hounds, wolves, the stag) use a generated mesh on a
  Mesh2Motion quadruped skeleton with its CC0 walk and run clips. Attacks that the clip
  set lacks are procedural poses layered on top.
- **Everything else** (moths, mites, the mimic, the Mossgut Brood, the Unspooled's second
  phase) keeps the procedural animation, driven by `move_id` and `move_frame` as today,
  with a generated mesh in place of the code-built primitives.
- **Generated motion** (text-to-motion models such as Tencent's HY-Motion) isn't
  recommended as a main source. It's for humans only, its quality on weapon combat is
  unproven, and its license excludes some territories. Video-to-motion of the owner acting
  a move is the better way to fill a missing humanoid clip.

## Rejected sources

- **Fab Standard License content** (includes former Unreal Marketplace packs, Quixel, and
  Frank Climax's packs): the EULA requires blocking extraction, which plain GLB can't do.
- **Unity Asset Store packs** (Kubold, DoubleL's RPG Animations Pack, Protofactor
  creatures): embedded use only, and Unity's support article forbids extractable files.
- **Sketchfab store (Standard license) models:** the license forbids setups that let
  others download or extract the files.
- **Reallusion ActorCore and Character Creator:** shipped content must be unreadable by
  3D tools. AccuRIG falls under the same vendor terms and wasn't read separately.
- **MoCap Online:** binary-only distribution, with extraction prevented.
- **Quaternius, KayKit, and Kenney characters:** CC0, but stylized, not realistic. KayKit
  and Kenney have no realistic characters at all.
- **Quaternius Universal Animation Library 1 and 2 as the main set:** the owner judged
  the clips not good enough. Kept as a fallback only.
- **Poly Haven:** CC0 and high quality, but it has no characters, creatures, or
  animations. Useful for the environment research only.
- **OpenGameArt:** no realistic rigged dark-fantasy set was found. Quality and style vary
  per upload.
- **Bandai Namco Research motion dataset:** non-commercial license, and no weapon combat.
- **Carnegie Mellon mocap database** (and RancidMilk's conversion of it): free to use,
  but everyday motion, not melee.
- **100STYLE and Ubisoft LaFAN1:** walking-style datasets, no weapon combat. Licenses not
  read, because the content doesn't fit.
- **Rokoko's free motion library:** terms on shipping raw files weren't found, and the
  free set is small.
- **Truebones Zoo** (animal mocap): no longer free (listed at USD 45-100), and no license
  text was found.
- **Kevin Iglesias's Human Melee Animations** (itch.io, USD 23, 284 clips): the page shows
  no license text. Hand-keyed and aimed at strategy games.
- **Hunyuan3D** (open 3D generator, run locally): the output license is fine outside the
  excluded territories, but it has no rigging and needs a strong graphics card. Possible
  later as a free replacement for the paid generator.
- **Hyper3D Rodin:** reported as the best mesh quality for characters, but it doesn't rig,
  and its output license wasn't read. A possible swap for Meshy or Tripo.
- **Cascadeur:** an animation editor, not a source. An agent can't operate it.
- **VRM avatars:** anime style.

## Open questions for the owner

1. **Is the Mixamo reading acceptable?** The plan ships Mixamo motion only as baked data
   inside the game and keeps the raw files private. If that's too gray, the fallback is
   the CC0 clips you rejected plus video-to-motion capture, or asking Adobe.
2. **Are AI-generated models acceptable for the portfolio?** The presentation spec says
   not to use generated assets "whose license is unclear". On a paid plan the license is
   clear, but you may still not want generated art on a portfolio site.
3. **Would you email a publisher for written web permission?** One good paid animation
   set (for example Kubold's sword sets, USD 30-50) with a written yes would beat anything
   free. It costs one email per pack.
4. **Are you willing to act out moves on camera?** That's the cheapest way to get spear,
   halberd, parry, and bell-strike clips with real weight.
5. **Can armor be whole-body sets?** Generators make one fused mesh per character, so the
   seven armor sets become seven full models, not mix-and-match pieces. Per-slot mixing
   would need a different source.
6. **Where do private source files live?** The R2 bucket is the obvious place. Cloudflare
   Pages builds from the public repository, so the build needs a way to fetch them.

## What this research couldn't verify

- **How any animation looks.** No video could be played. The style-fit column for
  animation rests on listings, on whether a set is mocap or hand-keyed, and on prior
  knowledge. The owner's pad test is the only real evidence, and it hasn't happened.
- **Mixamo's contents.** The library needs a sign-in. The pack names, the clip count, the
  thin coverage of spear, halberd, and dagger, and the character list are from memory.
  Character triangle counts and texture sizes aren't published.
- **Generated model quality.** No model was generated. Topology, skin weights, and
  texture claims come from vendor pages and affiliate reviews, which are weak evidence.
  No independent test of rigged, armored humanoids was found.
- **Generator prices in US dollars and credit costs per model.** The Meshy page showed
  rupee prices and no per-operation credit costs. The Tripo price is from a third-party
  page. The 40-100 dollar total is an estimate.
- **The Unity support article's exact words.** The fetch tool paraphrased it. The EULA
  text itself was read in a browser.
- **The Reallusion, MoCap Online, Sketchfab Standard, Meshy, Quaternius, Mesh2Motion, and
  Hunyuan3D quotes.** They came through the fetch tool's summarizing step. Re-read each
  one before the ADR cites it.
- **The MakeHuman CC0 statement.** Its license page failed to load (certificate error,
  then a 404). CC0 for exported models is from memory. The MPFB skeleton presets and
  triangle counts are also from memory.
- **The Carnegie Mellon terms at the source.** The site failed a certificate check. The
  terms are as quoted on RancidMilk's page.
- **The Epic content license page and the Hyper3D, DeepMotion, Rokoko, and Truebones
  terms.** They either refused the request or weren't found.
- **The Sketchfab CC BY models.** Triangle counts and licenses are from the Sketchfab API.
  Rig quality, texture sets, and whether each uploader really made the model weren't
  checked. Check each one before use.
- **Mesh2Motion's clip list per creature.** The site says over 170 animations but doesn't
  list attacks per rig.
