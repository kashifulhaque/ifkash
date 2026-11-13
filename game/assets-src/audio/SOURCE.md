# Latticefall audio (generated in code)

- **Author:** The Cinderheart project
- **License:** Original work of this repository. See `LICENSE.md` in this folder. It contains no third-party audio.
- **Source:** Synthesized in code with the Web Audio API, in `src/lib/latticefall/audio/` (`synth.ts`, `beds.ts`, `bosses.ts`, and `voices.ts`). There are no sample files.

## What it covers

- **Region music:** an ambience bed and an exploration loop (a pad under a plucked melody) for each of the 17 areas, with a default for any other area. The numbers are in `voices.ts`.
- **Boss music:** an intro swell, a loop stem with the boss's beat and drone, and a phase-2 arpeggio, for each of the 12 bosses. The Bellwright's beat follows the core's `BELL_BEAT` events, not the audio clock.
- **Effects:** footsteps, weapon swings, hits, the Lantern, pickups, UI clicks, the Bellwright's bell and toll, ice cracking and breaking, the chain snapping, and the defeat sting.

## Why there are no samples

The presentation spec's audio section (`docs/game/07-presentation.md#audio`) allows a procedural approach, and no audio pack has been verified for this project yet. Synthesizing everything keeps the license trail to this repository and adds no download.
