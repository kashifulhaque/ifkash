import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { DEFAULT_DESIGN, DESIGNS } from '$lib/designs/metas';
import { applyThemeColor, isDesign, readTheme, type Design } from './appearance';

export type { Design };
export { isDesign, DESIGNS };

const KEY = 'ifkash-design';

function initial(): Design {
	if (!browser) return DEFAULT_DESIGN;
	// The inline script in app.html has already resolved the saved choice
	// onto <html data-design>, so read it back from there. A removed design
	// id falls back to the default.
	const saved = document.documentElement.getAttribute('data-design');
	return isDesign(saved) ? saved : DEFAULT_DESIGN;
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
	try {
		localStorage.setItem(KEY, next);
	} catch {
		// The choice still applies for this visit.
	}
	design.set(next);
}
