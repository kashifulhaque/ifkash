import { get, writable } from 'svelte/store';
import { browser } from '$app/environment';
import { applyThemeColor, readDesign, type Theme } from './appearance';

export type { Theme };

/** `system` follows the OS. `light` and `dark` stay put until you choose again. */
export type ThemeChoice = Theme | 'system';

const KEY = 'ifkash-theme';

function readStored(): ThemeChoice {
	if (!browser) return 'system';
	try {
		const saved = localStorage.getItem(KEY);
		if (saved === 'light' || saved === 'dark') return saved;
	} catch {
		// Storage can be blocked. Follow the system theme for this visit.
	}
	return 'system';
}

function writeStored(next: ThemeChoice): void {
	try {
		if (next === 'system') localStorage.removeItem(KEY);
		else localStorage.setItem(KEY, next);
	} catch {
		// The choice still applies for this visit.
	}
}

/** The theme the page should paint for a choice. */
export function resolveTheme(choice: ThemeChoice): Theme {
	if (choice === 'light' || choice === 'dark') return choice;
	if (!browser) return 'light';
	return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/**
 * Next click: leave the system for the other appearance, then swap appearances,
 * then return to the system.
 */
export function nextThemeChoice(current: ThemeChoice, resolved: Theme): ThemeChoice {
	if (current === 'system') return resolved === 'dark' ? 'light' : 'dark';
	if (current === 'light') return 'dark';
	return 'system';
}

const initialChoice = readStored();

export const themeChoice = writable<ThemeChoice>(initialChoice);
export const theme = writable<Theme>(resolveTheme(initialChoice));

function apply(t: Theme): void {
	document.documentElement.setAttribute('data-theme', t);
	applyThemeColor(readDesign(), t);
}

if (browser) {
	// The inline script in app.html already painted this, so the first apply matches it.
	theme.subscribe(apply);

	const scheme = window.matchMedia('(prefers-color-scheme: dark)');
	scheme.addEventListener('change', (event) => {
		if (get(themeChoice) !== 'system') return;
		theme.set(event.matches ? 'dark' : 'light');
	});
}

export function setThemeChoice(next: ThemeChoice): void {
	if (!browser) return;
	writeStored(next);
	themeChoice.set(next);
	theme.set(resolveTheme(next));
}

export function setTheme(t: Theme): void {
	setThemeChoice(t);
}
