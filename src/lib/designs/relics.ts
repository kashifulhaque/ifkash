/**
 * Relic progress from the Cinderheart codex. It lives in localStorage, so
 * call this after mount; the server render never has a badge. Neither
 * import pulls in Three.js or the WASM core. Keep it that way.
 */
import { findableEntries } from '$lib/latticefall/codex/entries';
import { readCodex } from '$lib/latticefall/codex/store';

export interface RelicBadge {
	text: string;
	href: string;
}

const relicKeys = new Set(
	findableEntries()
		.filter((e) => e.section === 'projects')
		.map((e) => e.key),
);

/** Returns the badge, or `null` when the visitor has found no relics. */
export function readRelicBadge(): RelicBadge | null {
	const found = readCodex().entries.filter((k) => relicKeys.has(k)).length;
	if (found === 0) return null;
	return {
		text: `${found} of ${relicKeys.size} relics found`,
		href: found === relicKeys.size ? '/codex' : '/game',
	};
}
