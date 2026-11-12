// Hollowdeep's darkness (WP-062): how far the fog lets the player see when the core says they are
// outside the glowing zones. The core owns the rule (`lf_core::world_state::mechanics::darkness`:
// 6 m in the dark, 14 m with a light, no limit in a glowing zone or beside a Lantern) and sends
// the player's visibility in meters as one state word (0 for no limit); this module only turns
// that number into a fog look. Read-only: nothing here writes simulation state.

/** Fog starts this fraction of the visibility away: the dark is soft at the player's feet. */
export const DARK_NEAR_PER_M = 0.35;
/** Fog is opaque this many times the visibility away, so the stated distance is still faintly seen. */
export const DARK_FAR_PER_M = 1.25;
/** Light and fog color are scaled by this in the dark. */
export const DARK_LIGHT_SCALE = 0.3;
export const DARK_FOG_SCALE = 0.2;

export interface DarkFog {
  near: number;
  far: number;
  /** Multiplies the sun and hemisphere light. */
  light: number;
  /** Multiplies the fog color (the dark pulls the horizon to black). */
  color: number;
}

/**
 * The fog for a player who sees `visibilityM` meters, given the area's own fog. 0 (or less) means
 * no limit, and the area's fog is used as it is.
 */
export function darkFog(visibilityM: number, baseNear: number, baseFar: number): DarkFog {
  if (!(visibilityM > 0)) return { near: baseNear, far: baseFar, light: 1, color: 1 };
  return {
    near: Math.min(baseNear, visibilityM * DARK_NEAR_PER_M),
    far: Math.min(baseFar, visibilityM * DARK_FAR_PER_M),
    light: DARK_LIGHT_SCALE,
    color: DARK_FOG_SCALE
  };
}
