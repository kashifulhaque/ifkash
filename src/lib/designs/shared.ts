/**
 * Data and helpers every design can reuse. Nothing here depends on a
 * design, a Svelte component, or Three.js.
 */
import { dev } from '$app/environment';

export type Theme = 'light' | 'dark';

/** Copy for the theme toggle: the action labels and the names of each theme. */
export interface ThemeCopy {
	/** Label for the action that switches to the light theme. */
	toLight: string;
	/** Label for the action that switches to the dark theme. */
	toDark: string;
	/** Name of the light theme. */
	light: string;
	/** Name of the dark theme. */
	dark: string;
}

/** What `src/lib/stores/appearance.ts` and the inline script in `app.html` need. */
export interface DesignMeta<Id extends string = string> {
	id: Id;
	label: string;
	/** Browser chrome color (`<meta name="theme-color">`) for each theme. */
	themeColors: Record<Theme, string>;
	themeCopy: ThemeCopy;
}

export interface NavItem {
	href: string;
	label: string;
}

export const navItems: NavItem[] = [
	{ href: '/work', label: 'Work' },
	{ href: '/projects', label: 'Projects' },
	{ href: '/rl', label: 'RL' },
	{ href: '/roadmap', label: 'Roadmap' },
	{ href: '/tools', label: 'Tools' },
	{ href: '/education', label: 'Education' },
	{ href: '/blog', label: 'Blog' },
];

export function isActive(href: string, path: string): boolean {
	return path === href || path.startsWith(href + '/');
}

/** Routes that render without the shell chrome (top bar, footer). */
export function isFullWidthPath(path: string): boolean {
	return (
		path === '/game' ||
		path === '/editor' ||
		path === '/admin/notes' ||
		(path.startsWith('/tools/') && path !== '/tools')
	);
}

export const owner = 'Kashiful Haque';

export const blurb =
	'ML engineer building language-model systems, reinforcement learning pipelines, and fast inference in C++ and Rust.';

export const coordinates = 'Bengaluru 12.97° N, 77.59° E';

export interface FooterLink {
	href: string;
	label: string;
	external?: boolean;
}

export const footerGroups: { title: string; links: FooterLink[] }[] = [
	{
		title: 'Elsewhere',
		links: [
			{ href: 'https://github.com/kashifulhaque', label: 'GitHub', external: true },
			{ href: 'https://hf.co/ifkash', label: 'Hugging Face', external: true },
			{ href: 'https://linkedin.com/in/kashifulhaque', label: 'LinkedIn', external: true },
		],
	},
	{
		title: 'Here',
		links: [
			{ href: '/api/cv?format=view', label: 'Résumé' },
			{ href: '/game', label: 'Tiny planet' },
			{ href: '/fitness', label: 'Fitness' },
			{ href: '/admin', label: 'Admin' },
		],
	},
];

export const resumeUrl = dev
	? 'http://localhost:8787/api/cv?format=view'
	: '/api/cv?format=view';

export interface IndexRow {
	n: string;
	name: string;
	desc: string;
	href: string;
}

export const indexRows: IndexRow[] = [
	{ n: '01', name: 'Work', desc: 'Roles and companies', href: '/work' },
	{ n: '02', name: 'Projects', desc: 'Open source and independent work', href: '/projects' },
	{ n: '03', name: 'RL', desc: 'Reinforcement learning experiments', href: '/rl' },
	{ n: '04', name: 'Tools', desc: 'Useful in-browser utilities', href: '/tools' },
	{ n: '05', name: 'Education', desc: 'Academic background', href: '/education' },
	{ n: '06', name: 'Blog', desc: 'Notes on engineering and ML', href: '/blog' },
	{ n: '07', name: 'Game', desc: 'Cinderheart, a small action RPG', href: '/game' },
	{ n: '08', name: 'Roadmap', desc: 'Interview course anchored to shipped work', href: '/roadmap' },
];

export const facts: { k: string; v: string }[] = [
	{ k: 'Focus', v: 'LLM systems' },
	{ k: 'Experience', v: '4+ years' },
	{ k: 'Core stack', v: 'C++, Rust, Python' },
	{ k: 'Based in', v: 'Bengaluru, India' },
];
