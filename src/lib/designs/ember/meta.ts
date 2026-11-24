import type { DesignMeta } from '../shared';

export const meta: DesignMeta<'ember'> = {
	id: 'ember',
	label: 'Ember',
	// Keep these in sync with the color table in `src/app.html`.
	themeColors: { light: '#eee4cf', dark: '#0e0c09' },
	themeCopy: {
		toLight: 'Switch to parchment',
		toDark: 'Switch to embers',
		light: 'Parchment',
		dark: 'Embers',
	},
};
