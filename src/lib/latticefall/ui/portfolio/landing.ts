// Landing card settings. No Three.js or WASM import: the card loads before the game does.

const KEY = 'ifkash-game-landing';

/** Rounded download size of the game, in megabytes, for the card's size line. */
export const DOWNLOAD_MB = 4;

/** Whether a visitor chose the plain site and asked to remember it. */
export function readPlainPreference(): boolean {
  try {
    return typeof localStorage !== 'undefined' && localStorage.getItem(KEY) === 'plain';
  } catch {
    return false;
  }
}

/** Saves the choice, or forgets it when `remember` is false. */
export function writePlainPreference(remember: boolean): void {
  try {
    if (remember) localStorage.setItem(KEY, 'plain');
    else localStorage.removeItem(KEY);
  } catch {
    // Blocked storage: the choice lasts for this page only.
  }
}

/** Dev and screenshot URLs skip the card: `&dev=1` or `&play=1` go straight to the game. */
export function skipsLanding(params: URLSearchParams): boolean {
  return params.get('dev') === '1' || params.get('play') === '1';
}
