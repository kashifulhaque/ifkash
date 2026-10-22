import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { applyThemeColor, readTheme, type Design } from './appearance';

export type { Design };

export const DESIGNS: { id: Design; label: string; short: string }[] = [
	{ id: 'nordisk', label: 'Nordisk', short: 'N' },
	{ id: 'broadsheet', label: 'Broadsheet', short: 'B' },
];

const KEY = 'ifkash-design';

export function isDesign(value: string | null): value is Design {
	return value === 'nordisk' || value === 'broadsheet';
}

function initial(): Design {
	if (!browser) return 'nordisk';
	// The inline script in app.html has already resolved the saved choice
	// onto <html data-design>, so read it back from there.
	const saved = document.documentElement.getAttribute('data-design');
	return isDesign(saved) ? saved : 'nordisk';
}

function apply(design: Design): void {
	document.documentElement.setAttribute('data-design', design);
	applyThemeColor(design, readTheme());
}

export const design = writable<Design>(initial());

if (browser) {
	// Reflect every change onto <html data-design> (the inline script in
	// app.html already sets this before paint, so there is no flash).
	design.subscribe(apply);
}

export function setDesign(next: Design): void {
	if (!browser) return;
	localStorage.setItem(KEY, next);
	design.set(next);
}
