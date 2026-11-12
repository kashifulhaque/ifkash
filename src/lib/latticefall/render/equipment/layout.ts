// Where the armory's sections stand and where the dev camera starts for each. No Three.js, so
// `engine/index.ts` can read it before the armory itself is loaded. The armory sits well off
// the playable ground: sections run along +X at `ARMORY_Z`, each facing +Z, and the camera
// looks back toward -Z (yaw 180 degrees).

export const ARMORY_Z = 60;
export const SECTION_SPACING = 20;

export const SECTIONS = ['blades', 'heavy', 'hands', 'catalysts', 'armor'] as const;
export type SectionName = (typeof SECTIONS)[number];

export const sectionX = (s: SectionName): number => SECTIONS.indexOf(s) * SECTION_SPACING;

/** Camera presets, as the `cam=fixed:X,Y,Z,YAW,PITCH` parts (degrees), by section. */
export function sectionCam(s: SectionName): { pos: [number, number, number]; yaw: number; pitch: number } {
  const x = sectionX(s);
  switch (s) {
    case 'blades':
      return { pos: [x, 3.2, ARMORY_Z + 4.8], yaw: Math.PI, pitch: (21 * Math.PI) / 180 };
    case 'heavy':
      return { pos: [x, 3.1, ARMORY_Z + 6.4], yaw: Math.PI, pitch: (15 * Math.PI) / 180 };
    case 'hands':
      return { pos: [x, 2.8, ARMORY_Z + 5.4], yaw: Math.PI, pitch: (17 * Math.PI) / 180 };
    case 'catalysts':
      return { pos: [x, 2.7, ARMORY_Z + 6.4], yaw: Math.PI, pitch: (13 * Math.PI) / 180 };
    case 'armor':
      return { pos: [x, 1.6, ARMORY_Z + 6.6], yaw: Math.PI, pitch: (4 * Math.PI) / 180 };
  }
}

export function parseSection(raw: string | null): SectionName {
  return (SECTIONS as readonly string[]).includes(raw ?? '') ? (raw as SectionName) : 'blades';
}
