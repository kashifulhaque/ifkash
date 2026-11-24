import type { DesignMeta } from '../shared';

export const meta: DesignMeta<'almanac'> = {
	id: 'almanac',
	label: 'Almanac',
	// Keep these in sync with the color table in `src/app.html`.
	themeColors: { light: '#f6f1e7', dark: '#1a1714' },
	themeCopy: {
		toLight: 'Switch to paper',
		toDark: 'Switch to lamplight',
		light: 'Paper',
		dark: 'Lamplight',
	},
};
