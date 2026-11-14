// Move numbers to animation info, from the generated content digest: names, frame counts, and
// the strike window (first hitbox frame to the frame after the last). Lets the actor renderer
// play a move's clip from the snapshot's `move_id`, with the clip's contact on the hit.

import digest from '../../gen/digest.json';
import type { MoveAnimInfo, MoveResolver } from './moveClips';

interface DigestMove {
  num: number;
  anim: string;
  totalFrames: number;
  strikeStart?: number | null;
  recoveryStart?: number | null;
}

/** A resolver over the committed digest. Unknown numbers resolve to undefined. */
export function digestMoveResolver(): MoveResolver {
  const table = new Map<number, MoveAnimInfo>();
  for (const m of (digest as { moves: DigestMove[] }).moves) {
    table.set(m.num, {
      anim: m.anim,
      totalFrames: m.totalFrames,
      strikeStart: m.strikeStart ?? undefined,
      recoveryStart: m.recoveryStart ?? undefined
    });
  }
  return (id) => table.get(id);
}

/** The first move in the digest that plays `anim`, for the dev preview's timing. */
export function digestAnimInfo(anim: string): MoveAnimInfo | undefined {
  const m = (digest as { moves: DigestMove[] }).moves.find((x) => x.anim === anim);
  if (!m) return undefined;
  return { anim, totalFrames: m.totalFrames, strikeStart: m.strikeStart ?? undefined, recoveryStart: m.recoveryStart ?? undefined };
}
