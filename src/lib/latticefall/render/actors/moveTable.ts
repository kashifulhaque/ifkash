// Move numbers to animation info, from the generated content digest (names and frame counts
// only; hitbox windows aren't in the digest). Lets the actor renderer play a move's clip
// from the snapshot's `move_id`.

import digest from '../../gen/digest.json';
import type { MoveAnimInfo, MoveResolver } from './moveClips';

interface DigestMove {
  num: number;
  anim: string;
  totalFrames: number;
}

/** A resolver over the committed digest. Unknown numbers resolve to undefined. */
export function digestMoveResolver(): MoveResolver {
  const table = new Map<number, MoveAnimInfo>();
  for (const m of (digest as { moves: DigestMove[] }).moves) {
    table.set(m.num, { anim: m.anim, totalFrames: m.totalFrames });
  }
  return (id) => table.get(id);
}
