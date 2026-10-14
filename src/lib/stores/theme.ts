import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'light' | 'dark';

const KEY = 'ifkash-theme';

function initial(): Theme {
	if (!browser) return 'light';
	// The inline script in app.html has already resolved the saved choice or
	// the system preference onto <html data-theme>, so read it back from there.
	return document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'light';
}

function apply(t: Theme): void {
	document.documentElement.setAttribute('data-theme', t);
	const meta = document.querySelector('meta[name="theme-color"]');
	if (meta) meta.setAttribute('content', t === 'dark' ? '#15181b' : '#f4f1ea');
}

export const theme = writable<Theme>(initial());

if (browser) {
	// Reflect every change onto <html data-theme> (the inline script in
	// app.html already sets this before paint, so there is no flash).
	theme.subscribe(apply);
}

export function toggleTheme(): void {
	if (!browser) return;
	theme.update((t) => {
		const next: Theme = t === 'dark' ? 'light' : 'dark';
		localStorage.setItem(KEY, next);
		return next;
	});
}

export function setTheme(t: Theme): void {
	if (!browser) return;
	localStorage.setItem(KEY, t);
	theme.set(t);
}
