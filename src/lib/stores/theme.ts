import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'light' | 'dark';

function getInitialTheme(): Theme {
	if (browser) {
		const stored = localStorage.getItem('theme');
		if (stored === 'light' || stored === 'dark') return stored;
		if (window.matchMedia('(prefers-color-scheme: dark)').matches) return 'dark';
	}
	return 'light';
}

const themeStore = writable<Theme>(getInitialTheme());

export const theme = {
	subscribe: themeStore.subscribe,

	toggle: () => {
		themeStore.update((current) => {
			const next: Theme = current === 'light' ? 'dark' : 'light';
			if (browser) {
				localStorage.setItem('theme', next);
				document.documentElement.setAttribute('data-theme', next);
			}
			return next;
		});
	},

	set: (value: Theme) => {
		if (browser) {
			localStorage.setItem('theme', value);
			document.documentElement.setAttribute('data-theme', value);
		}
		themeStore.set(value);
	},

	init: () => {
		if (browser) {
			const current = getInitialTheme();
			document.documentElement.setAttribute('data-theme', current);
			themeStore.set(current);
		}
	}
};
