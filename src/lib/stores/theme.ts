import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export type Theme = 'light' | 'dark';

const KEY = 'ifkash-theme';

function preferred(): Theme {
	return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function initial(): Theme {
	if (!browser) return 'light';
	const saved = localStorage.getItem(KEY);
	return saved === 'light' || saved === 'dark' ? saved : preferred();
}

function apply(t: Theme): void {
	document.documentElement.setAttribute('data-theme', t);
	const meta = document.querySelector('meta[name="theme-color"]');
	if (meta) meta.setAttribute('content', t === 'dark' ? '#0e0e10' : '#1b3fa0');
}

export const theme = writable<Theme>(initial());

if (browser) {
	// Reflect every change onto <html data-theme> (the inline script in
	// app.html already sets this before paint, so there is no flash).
	theme.subscribe(apply);

	// Follow the OS preference live, but only until the user picks explicitly.
	window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
		if (!localStorage.getItem(KEY)) theme.set(e.matches ? 'dark' : 'light');
	});
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
