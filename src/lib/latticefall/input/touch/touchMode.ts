// Touch mode's settings: the low quality preset and the 30 frames per second target.
// Imports only the quality module, so it pulls in no Three.js.

import { getQuality, setQuality, type QualityName } from '../../render/lighting/quality';

/** Frame rate touch mode aims for (docs/game/07-presentation.md, "Mobile"). */
export const TOUCH_FPS_TARGET = 30;

/**
 * Drops to the low preset for touch mode and returns a function that puts the previous
 * preset back, so a visitor who returns to a desktop keeps their own choice.
 */
export function enterTouchQuality(): () => void {
  const before: QualityName = getQuality().name;
  setQuality('low');
  return () => setQuality(before);
}
