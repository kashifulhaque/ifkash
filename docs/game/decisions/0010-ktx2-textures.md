# 0010: KTX2 textures with the `ktx` tool

- **Status:** Accepted
- **Date:** 2026-11-15
- **WP:** GitHub issue 85, a prerequisite of
  [the art direction research](../art-direction-research.md#prerequisites)

## Context

The art is moving from flat colors to textured PBR materials: color, normal, and
roughness and metalness maps. A PNG or JPEG texture is decoded to RGBA on the GPU, where
a 2048 x 2048 map takes about 22 MB with mipmaps. KTX2 with Basis Universal keeps textures
compressed on the GPU, at a quarter to an eighth of that.

[ADR 0005](0005-asset-pipeline-tools.md) named KTX-Software's `toktx` as the encoder but
left it uninstalled, because no asset had textures. Two facts changed the plan:

- `@gltf-transform/cli` 4.5.1, the version this repository pins, no longer runs `toktx`.
  It runs `ktx create`, and it needs KTX-Software 4.4.0 or later.
- KTX-Software isn't in Homebrew core, and its macOS release is an installer package
  that asks for an administrator password.

## Decision

Encode textures with **KTX-Software 4.4.2's `ktx` tool**, unpacked into a directory that
the contributor owns, and let `gltf-transform optimize` choose the settings for each map.
This replaces the `toktx` bullet in ADR 0005.

### Set up the encoder

To set up a machine for textured assets, run one of the following commands from the
repository root:

```sh
mise run game:ktx
bash game/scripts/install-ktx.sh
```

Both commands do the same thing. The script completes these steps:

1. Downloads the 4.4.2 release for the platform from
   `https://github.com/KhronosGroup/KTX-Software/releases`.
2. Compares the file's SHA-256 digest with the digest pinned in the script, which is the
   digest GitHub publishes for the release asset. A mismatch stops the script.
3. Unpacks `bin/ktx` and `lib/libktx` into `~/.local/share/latticefall/ktx-4.4.2/`. On
   macOS it expands the package with `pkgutil --expand-full` and doesn't run the
   installer, so it needs no password. On Linux it unpacks the `.tar.bz2` archive.

To install somewhere else, set `LF_KTX_DIR` for both the script and `lf-tools`. If
`XDG_DATA_HOME` is set, the default directory is under it.

`lf-tools assets` looks for `ktx` in that directory first, and then on the `PATH`, so a
system install of 4.4.0 or later also works. If a source file has textures and no usable
`ktx` exists, the command stops and prints the install command. Files without textures
need only `bun install`.

To upgrade, change the version and the four digests in `game/scripts/install-ktx.sh`,
and `KTX_VERSION` in `game/crates/tools/src/assets.rs`.

### Encoding rules

`lf-tools assets` runs `gltf-transform optimize --texture-compress ktx2`, which encodes
each texture by the material slot that uses it:

| Map | Codec | Color space | Why |
|---|---|---|---|
| Base color, emissive | ETC1S, quality 255 | sRGB | Color tolerates ETC1S artifacts, and the file is several times smaller than UASTC. |
| Normal | UASTC, level 4, Zstandard | Linear | ETC1S block artifacts show up as lighting errors. Rate-distortion optimization is off for normal maps, for the same reason. |
| Occlusion, roughness, metalness | UASTC, level 4, RDO lambda 4, Zstandard | Linear | The three values share one texture, and ETC1S mixes the channels. |

Every texture gets a full mipmap chain. gltf-transform reads the color space from the
slot and writes it into the KTX2 file, so the source images need no tagging. Normal maps
keep three channels, so the renderer needs no shader change to rebuild the Z component.

### Maximum texture size

No texture is written with an edge longer than the maximum texture size. A larger source
is scaled down before it's encoded. The default is 2048. You can change it in two places:

- For one run, pass `--max-texture-size SIZE` to `lf-tools assets`.
- For one file, set `"maxTextureSize": SIZE` on its entry in `game/assets-src/assets.json`.
  The entry's value wins.

`SIZE` is a power of two from 4 to 8192.

### Transcoder

The browser turns a KTX2 texture into a format its GPU supports with the Basis
transcoder. `lf-tools assets` copies `basis_transcoder.js` and `basis_transcoder.wasm`
from `node_modules/three/examples/jsm/libs/basis/` into `static/latticefall/basis/`
whenever an output has KTX2 textures, so the committed copy matches the installed
`three` version. After you upgrade `three`, run `lf-tools assets` again.

`src/lib/latticefall/render/assets/gltfLoader.ts` is the one place that configures model
loading. `createGltfLoader(renderer)` returns a loader with the meshopt decoder and the
renderer's single `KTX2Loader`, which calls `detectSupport(renderer)`. Use it for every
model. The loader fetches the transcoder (585 KB, about 260 KB with gzip)
the first time it parses a KTX2 texture, so the home page and the `/game` landing card
don't fetch it.

### Test model

`game/assets-src/polyhaven-stone-fire-pit/` holds a CC0 Poly Haven model with color,
normal, and roughness maps at 2K. The pipeline writes it to
`static/latticefall/dev/stone_fire_pit.glb` at 1K. To see it, open
`/game?dev=1&play=1&ktx2test=1`. The model stands 3 m in front of the start position, and
`window.__lfKtx2Test` holds each texture's GPU format, size, and load time.

## Measurements

The test model has 3,887 triangles and three maps. Measured on 2026-11-15 on an Apple M4
Pro, in Chromium 156 with WebGL 2 (ANGLE on Metal), from the local dev server:

| Maximum size | Source GLB (JPEG) | KTX2 GLB | GPU memory, KTX2 | GPU memory, RGBA8 |
|---|---|---|---|---|
| 2048 | 9.62 MB | 8.47 MB | 13.98 MB | 67.1 MB |
| 1024 | 2.53 MB | 2.29 MB | 3.50 MB | 16.8 MB |
| 512 | Not measured | 0.66 MB | 0.87 MB | 4.2 MB |

- The 1024 row's GPU memory is measured in the browser from the uploaded mip levels. The
  other rows are `gltf-transform inspect` estimates. The RGBA8 column is the size the same
  maps take when a PNG or JPEG is decoded, with mipmaps.
- The 1024 source size is Poly Haven's own 1K download. The repository holds only the 2K
  source.
- Per map at 1024: color 248 KB (ETC1S), normal 1.28 MB (UASTC), and roughness and
  metalness 734 KB (UASTC).
- GPU formats that the transcoder picked: ETC2 RGB for the color map, and ASTC 4x4 for
  the normal map and the roughness and metalness map. All three textures report
  `isCompressedTexture`. A desktop GPU without ASTC gets BC7 for the UASTC maps.
- Load time for the 1024 model: 68 to 113 ms for the first load, which includes the
  transcoder's download and startup, and 16 to 29 ms for a second load. These numbers come
  from a fast machine with no network delay, so treat them as a lower bound.
- Encoding time: about 45 seconds for the model at 1024, and about 3 minutes at 2048.
  UASTC takes nearly all of it.

KTX2 cuts GPU memory to about a fifth. It doesn't cut the download much, because the
UASTC normal map is as large as the JPEG it replaces.

## Consequences

- Textured assets need the `ktx` tool, set up with one command and no password. Output in
  `static/latticefall/` is committed, so CI and deploys don't need it.
- The download budget for textured art depends mostly on normal maps: about 1.3 MB for
  each 1K map and 5 MB for each 2K map with these settings. The art direction research
  must set texture sizes with that in mind.
- `manifest.json` has a `dev` region for the test model. The transcoder files aren't
  listed in the manifest.
- The quality presets' texture cap (`render/perf/textures.ts`) redraws images on a
  canvas, which can't shrink a compressed texture. To cap KTX2 textures per preset,
  either write the smaller size with `maxTextureSize` or skip the top mip levels at load
  time. Neither is built.
- The Linux branch of the install script has the pinned digests but hasn't been run.

## Alternatives considered

- **`toktx`, as ADR 0005 said:** gltf-transform 4.5 doesn't run it.
- **Run the macOS installer package:** Needs an administrator password, and installs into
  `/usr/local` where the version can't be pinned per project.
- **A WASM Basis encoder from npm, run with Bun:** No system tool, but gltf-transform has
  no hook for it, so the per-slot rules for codec, color space, and channels would have
  to be rewritten and maintained here.
- **Pin `ktx` in `mise.toml`:** The mise registry has no KTX-Software entry, and its
  GitHub release backend can't unpack a macOS installer package.
- **UASTC for color maps too:** Better color quality, at several times the download size.
  Revisit it for hero assets if ETC1S banding shows on the chosen art.
- **WebP or AVIF textures:** Smaller downloads, but the GPU holds them as RGBA8, about
  five times the memory.
