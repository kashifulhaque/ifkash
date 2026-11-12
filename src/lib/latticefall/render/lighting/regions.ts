// Per-region lighting: how a named region bends the area's ambient settings (a warmer sun, a
// thicker haze). The area's `ambient` block in `area.ron` is the base; a region listed here
// multiplies it, and the renderer eases between looks as the player crosses a border.
// Regions not listed use the base unchanged.

export interface RegionLook {
  /** Multiplies the fog and sky horizon color, per channel. */
  fogTint?: [number, number, number];
  /** Multiplies fog near and far distances; below 1 is thicker. */
  fogScale?: number;
  /** Multiplies the sun color, per channel. */
  sunTint?: [number, number, number];
  sunScale?: number;
  ambientScale?: number;
  /** Strength of the low-lying height fog, 0 to 1 (default 0.4). */
  haze?: number;
}

export const REGION_LOOKS: Readonly<Record<string, RegionLook>> = {
  // Warm, low sun, haze (the Verge).
  meadow: { fogTint: [1.02, 0.97, 0.88], fogScale: 0.8, sunTint: [1.0, 0.92, 0.8], sunScale: 0.95, haze: 0.55 },
  // The cooler, clearer flats beyond the drop.
  far_flats: { fogTint: [0.94, 0.98, 1.04], fogScale: 1.15, sunTint: [0.96, 0.98, 1.0], sunScale: 0.9, ambientScale: 1.05, haze: 0.3 }
};

export const NEUTRAL_LOOK: RegionLook = {};
