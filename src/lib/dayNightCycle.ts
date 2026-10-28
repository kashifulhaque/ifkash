/**
 * The day/night theme change, played as a short full-screen sky.
 *
 * A circle of sky opens out of the theme toggle, two hills rise, and the sun
 * and moon swap places on one arc while the sky passes through dusk (or dawn).
 * The theme flips while the sky covers the page, then the circle closes back
 * into the toggle to show the page in its new theme. Click or press any key to
 * skip ahead.
 *
 * Everything is a handful of DOM layers driven by one anime.js timeline, which
 * animates only transforms, opacity, and a single clip-path so it stays smooth.
 * With `prefers-reduced-motion: reduce` the theme changes at once.
 */
import { createTimeline, stagger } from 'animejs';
import { prefersReducedMotion } from './motion';
import type { Theme } from './stores/appearance';

export interface CycleOptions {
	/** Viewport point the sky opens out of, usually the toggle's centre. */
	origin: { x: number; y: number };
	/** The theme being switched to. */
	to: Theme;
	/** Caption shown mid-cycle, such as "Polar night". */
	label: string;
	/** Applies the new theme. Called once, while the sky covers the page. */
	onSwitch: () => void;
}

// Timeline marks, in milliseconds.
const OPEN = 650;
const SWITCH_AT = 1250;
const CLOSE_AT = 2000;
const CLOSE = 600;

const SKY = {
	day: 'linear-gradient(180deg, #7fb8e6 0%, #b9dcf2 55%, #f6ead2 100%)',
	dusk: 'linear-gradient(180deg, #2c2a5e 0%, #a5587a 55%, #f2a165 100%)',
	night: 'linear-gradient(180deg, #050816 0%, #111a3d 60%, #26305c 100%)'
};
const HILLS = {
	day: ['#9cb7c4', '#5f8582'],
	night: ['#18203f', '#0b1027']
};

function el(tag: string, style: Partial<CSSStyleDeclaration>, parent?: Element): HTMLElement {
	const node = document.createElement(tag);
	Object.assign(node.style, style);
	parent?.appendChild(node);
	return node;
}

const layer: Partial<CSSStyleDeclaration> = { position: 'absolute', inset: '0' };

export function playDayNightCycle(opts: CycleOptions): Promise<void> {
	if (prefersReducedMotion()) {
		opts.onSwitch();
		return Promise.resolve();
	}

	const W = window.innerWidth;
	const H = window.innerHeight;
	const base = Math.min(W, H);
	const { x: ox, y: oy } = opts.origin;
	const toNight = opts.to === 'dark';
	const [from, to] = toNight ? (['day', 'night'] as const) : (['night', 'day'] as const);
	const radius = Math.max(Math.hypot(ox, oy), Math.hypot(W - ox, oy), Math.hypot(ox, H - oy), Math.hypot(W - ox, H - oy));

	// ── Scene ─────────────────────────────────────────────────────────────

	const root = el('div', { position: 'fixed', inset: '0', zIndex: '2147483000', cursor: 'pointer' });
	root.setAttribute('aria-hidden', 'true');
	const scene = el('div', { ...layer, overflow: 'hidden', clipPath: `circle(0px at ${ox}px ${oy}px)` }, root);

	el('div', { ...layer, background: SKY[from] }, scene);
	const dusk = el('div', { ...layer, background: SKY.dusk, opacity: '0' }, scene);
	const sky = el('div', { ...layer, background: SKY[to], opacity: '0' }, scene);

	const stars = Array.from({ length: 48 }, () => {
		const size = Math.random() < 0.12 ? 2.5 : 1.5;
		return el(
			'span',
			{
				position: 'absolute',
				left: `${Math.random() * 100}%`,
				top: `${Math.random() * 62}%`,
				width: `${size}px`,
				height: `${size}px`,
				borderRadius: '50%',
				background: '#fff',
				boxShadow: size > 2 ? '0 0 6px 1px rgb(255 255 255 / 0.6)' : 'none',
				opacity: toNight ? '0' : '0.9'
			},
			scene
		);
	});

	// The sun and moon sit on opposite ends of one arm, pivoting below the
	// horizon, so a half turn sets one and raises the other.
	const horizon = H * 0.72;
	const arm = base * 0.56;
	const orbit = el(
		'div',
		{ position: 'absolute', left: `${W / 2}px`, top: `${horizon + base * 0.03}px`, width: '0', height: '0' },
		scene
	);
	const sunR = base * 0.055;
	const body = (dy: number, style: Partial<CSSStyleDeclaration>) =>
		el(
			'div',
			{
				position: 'absolute',
				left: `${-sunR}px`,
				top: `${dy - sunR}px`,
				width: `${sunR * 2}px`,
				height: `${sunR * 2}px`,
				borderRadius: '50%',
				...style
			},
			orbit
		);
	body(-arm, {
		background: 'radial-gradient(circle at 35% 35%, #fffbe8, #ffd27a 60%, #ff9f4a)',
		boxShadow: `0 0 ${sunR}px ${sunR * 0.3}px rgb(255 200 110 / 0.55), 0 0 ${sunR * 4}px ${sunR}px rgb(255 190 120 / 0.25)`
	});
	// A crescent cut with an inset shadow, so it needs no second disc.
	body(arm, {
		boxShadow: `inset ${-sunR * 0.45}px ${-sunR * 0.15}px 0 0 #f4f1e6, 0 0 ${sunR * 2}px rgb(190 210 255 / 0.25)`,
		transform: 'rotate(180deg)'
	});

	const svgNS = 'http://www.w3.org/2000/svg';
	const ground = document.createElementNS(svgNS, 'svg');
	ground.setAttribute('viewBox', '0 0 1440 400');
	ground.setAttribute('preserveAspectRatio', 'none');
	Object.assign(ground.style, { position: 'absolute', left: '0', bottom: '0', width: '100%', height: `${H - horizon + H * 0.08}px` });
	scene.appendChild(ground);
	const hills = [
		'M0 140 C 240 60 420 70 640 120 S 1080 40 1440 110 V400 H0 Z',
		'M0 230 C 300 170 520 200 760 240 S 1180 170 1440 210 V400 H0 Z'
	].map((d, i) => {
		const path = document.createElementNS(svgNS, 'path');
		path.setAttribute('d', d);
		path.setAttribute('fill', HILLS[from][i]);
		ground.appendChild(path);
		return path;
	});

	const caption = el(
		'div',
		{ position: 'absolute', left: '0', right: '0', top: `${H * 0.34}px`, textAlign: 'center', color: '#fff', textShadow: '0 2px 24px rgb(0 0 0 / 0.3)' },
		scene
	);
	const clock = el(
		'div',
		{ font: `500 ${Math.round(base * 0.024)}px var(--font-mono, ui-monospace, monospace)`, letterSpacing: '0.12em', opacity: '0' },
		caption
	);
	const title = el(
		'div',
		{ font: `500 ${Math.round(base * 0.09)}px var(--font-display, system-ui, sans-serif)`, lineHeight: '1.1', overflow: 'hidden', paddingBottom: '0.08em' },
		caption
	);
	const chars = [...opts.label].map((c) =>
		el('span', { display: 'inline-block', whiteSpace: 'pre', transform: 'translateY(110%)' }, title)
	);
	chars.forEach((s, i) => (s.textContent = opts.label[i]));

	// ── Timeline ──────────────────────────────────────────────────────────

	const iris = { r: 0 };
	const setIris = () => (scene.style.clipPath = `circle(${iris.r}px at ${ox}px ${oy}px)`);
	const hours = { h: toNight ? 17.5 : 5 };
	const setClock = () => {
		const hh = String(Math.floor(hours.h)).padStart(2, '0');
		const mm = String(Math.floor((hours.h % 1) * 60)).padStart(2, '0');
		clock.textContent = `${hh}:${mm}`;
	};
	setClock();

	return new Promise<void>((resolve) => {
		let switched = false;
		const doSwitch = () => {
			if (switched) return;
			switched = true;
			opts.onSwitch();
		};

		let done = false;
		const finish = () => {
			if (done) return;
			done = true;
			clearTimeout(fallback);
			tl.cancel();
			doSwitch();
			root.removeEventListener('pointerdown', skip);
			window.removeEventListener('keydown', onKey, true);
			root.remove();
			resolve();
		};

		const tl = createTimeline({ onComplete: finish });

		tl.add(iris, { r: [0, radius], duration: OPEN, ease: 'inOutQuart', onUpdate: setIris }, 0)
			.add(hills, { translateY: ['60%', '0%'], duration: 900, delay: stagger(90, { reversed: true }), ease: 'outExpo' }, 120)
			.add(orbit, { rotate: toNight ? [0, 180] : [180, 360], duration: 1700, ease: 'inOutSine' }, 200)
			.add(dusk, { opacity: [0, 1, 0], duration: 1700, ease: 'inOutSine' }, 200)
			.add(sky, { opacity: [0, 1], duration: 1100, ease: 'inOutSine' }, 800)
			.add(hills[0], { fill: HILLS[to][0], duration: 1200, ease: 'inOutSine' }, 500)
			.add(hills[1], { fill: HILLS[to][1], duration: 1200, ease: 'inOutSine' }, 500)
			.add(
				stars,
				{
					opacity: toNight ? [0, 0.9] : [0.9, 0],
					scale: toNight ? [0, 1] : [1, 0],
					duration: 600,
					delay: stagger(14, { from: 'center' }),
					ease: 'outQuad'
				},
				toNight ? 1000 : 300
			)
			.add(clock, { opacity: [0, 0.85], translateY: [8, 0], duration: 500, ease: 'outExpo' }, 450)
			.add(hours, { h: toNight ? 21.5 : 8, duration: 1500, ease: 'inOutSine', onUpdate: setClock }, 450)
			.add(chars, { translateY: ['110%', '0%'], duration: 800, delay: stagger(30), ease: 'outExpo' }, 500)
			.call(doSwitch, SWITCH_AT)
			.label('close', CLOSE_AT)
			.add(caption, { opacity: [1, 0], translateY: [0, -10], duration: 300, ease: 'inQuad' }, CLOSE_AT - 250)
			.add(iris, { r: [radius, 0], duration: CLOSE, ease: 'inOutQuart', onUpdate: setIris }, 'close');

		// Skipping jumps straight to the close.
		const skip = () => {
			if (tl.currentTime >= CLOSE_AT) return;
			doSwitch();
			tl.seek(CLOSE_AT);
		};
		const onKey = (e: KeyboardEvent) => {
			e.preventDefault();
			skip();
		};

		root.addEventListener('pointerdown', skip);
		window.addEventListener('keydown', onKey, true);
		document.body.appendChild(root);
		// If the renderer is throttled and frames never run, never leave the
		// overlay covering the page.
		const fallback = setTimeout(finish, CLOSE_AT + CLOSE + 1500);
	});
}
