import { readable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'dark';

const themeStore = readable<Theme>('dark');

function applyDark() {
	if (browser) {
		document.documentElement.setAttribute('data-theme', 'dark');
	}
}

export const theme = {
	subscribe: themeStore.subscribe,
	toggle: applyDark,
	set: applyDark,
	init: applyDark
};
