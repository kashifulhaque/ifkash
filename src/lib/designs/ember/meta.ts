import type { DesignMeta } from '../shared';

export const meta: DesignMeta<'ember'> = {
	id: 'ember',
	label: 'Ember',
	// Keep these in sync with the color table in `src/app.html`.
	themeColors: { light: '#e8dcc0', dark: '#0b0a09' },
	themeCopy: {
		toLight: 'Switch to vellum',
		toDark: 'Switch to night',
		light: 'Vellum',
		dark: 'Night',
	},
};
