// Ground palettes per region (docs/game/07-presentation.md, "Art direction": each region has
// a base palette and one accent). Accent colors are reserved for Lanterns, items,
// telegraphs, and the Spire, so ground palettes carry no accent. Regions the table doesn't
// know get the Verge palette: ash grays and dry ochre grass.

export interface GroundPalette {
  /** Flat, gentle ground. */
  grass: [string, string];
  /** Steep ground and cliff faces. */
  rock: string;
  /** Tops of raised ground, a little lighter. */
  high: string;
}

const VERGE: GroundPalette = {
  grass: ['#8c8352', '#a09560'],
  rock: '#77756f',
  high: '#9b978c'
};

/** Palettes by region name; a region not listed here uses `VERGE`. */
export const REGION_PALETTES: Readonly<Record<string, GroundPalette>> = {
  meadow: { grass: ['#8f8650', '#a69a5c'], rock: '#78756e', high: '#9b978c' },
  far_flats: { grass: ['#9c9786', '#aca695'], rock: '#7b7a76', high: '#a39f94' }
};

export function paletteFor(region: string | null): GroundPalette {
  return (region && REGION_PALETTES[region]) || VERGE;
}
