/**
 * The day/night theme change, played as a full-screen sky.
 *
 * A sky bursts out of the theme toggle, mountains rise from the bottom of the
 * screen, and the sun sets (or rises) over a lake while the stars, aurora,
 * clouds, birds, and fireflies follow the light. The theme flips while the sky
 * covers the page, then a hole opens around the moon (or sun) to show the page
 * in its new theme. Click or press any key to skip ahead.
 *
 * Everything is drawn on one canvas that is removed when the cycle ends. With
 * `prefers-reduced-motion: reduce` the theme changes at once.
 */
import type { Theme } from './stores/appearance';

export interface CycleOptions {
	/** Viewport point the sky bursts out of, usually the toggle's centre. */
	origin: { x: number; y: number };
	/** The theme being switched to. */
	to: Theme;
	/** Caption shown mid-cycle, such as "Polar night". */
	label: string;
	/** Applies the new theme. Called once, while the sky covers the page. */
	onSwitch: () => void;
}

type RGB = [number, number, number];

const DURATION = 3600;
const TAU = Math.PI * 2;

// Timeline marks, as fractions of DURATION.
const IRIS_IN_END = 0.16;
const SWITCH_AT = 0.5;
const IRIS_OUT_START = 0.8;

// Palette stops for darkness 0 (day), 0.5 (dusk), and 1 (night).
const SKY_TOP: RGB[] = [[86, 164, 222], [58, 48, 112], [5, 8, 26]];
const SKY_MID: RGB[] = [[150, 202, 236], [214, 92, 120], [15, 21, 54]];
const SKY_LOW: RGB[] = [[255, 234, 196], [255, 150, 78], [34, 38, 86]];
const RIDGE_FAR: RGB[] = [[132, 162, 184], [122, 72, 112], [24, 28, 56]];
const RIDGE_MID: RGB[] = [[88, 122, 124], [82, 44, 82], [14, 17, 40]];
const SHORE: RGB[] = [[48, 78, 66], [42, 22, 48], [6, 8, 20]];
const CLOUD: RGB[] = [[255, 255, 255], [255, 170, 170], [70, 76, 110]];
const SUN_HIGH: RGB = [255, 246, 210];
const SUN_LOW: RGB = [255, 112, 48];

const clamp01 = (x: number) => Math.min(1, Math.max(0, x));
const range = (p: number, a: number, b: number) => clamp01((p - a) / (b - a));
const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const easeInOut = (x: number) => (x < 0.5 ? 4 * x * x * x : 1 - (-2 * x + 2) ** 3 / 2);
const easeInOutSine = (x: number) => -(Math.cos(Math.PI * x) - 1) / 2;
const easeOut = (x: number) => 1 - (1 - x) ** 3;
const easeOutBack = (x: number) => 1 + 2.70158 * (x - 1) ** 3 + 1.70158 * (x - 1) ** 2;

function mix(a: RGB, b: RGB, t: number): RGB {
	return [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)];
}

/** Picks a colour from three evenly spaced stops. */
function ramp(stops: RGB[], d: number): RGB {
	return d < 0.5 ? mix(stops[0], stops[1], d * 2) : mix(stops[1], stops[2], (d - 0.5) * 2);
}

function rgb(c: RGB, a = 1): string {
	return `rgb(${c[0] | 0} ${c[1] | 0} ${c[2] | 0} / ${a})`;
}

function rand(a: number, b: number): number {
	return a + Math.random() * (b - a);
}

/** A mountain ridge as y offsets above its base line, one per `step` pixels. */
function ridge(width: number, amp: number, step: number, jag: number): number[] {
	const f = [rand(0.002, 0.004), rand(0.006, 0.01), rand(0.02, 0.03)];
	const ph = [rand(0, TAU), rand(0, TAU), rand(0, TAU)];
	const out: number[] = [];
	for (let x = 0; x <= width + step; x += step) {
		const h =
			0.55 +
			0.3 * Math.sin(x * f[0] + ph[0]) +
			0.15 * Math.sin(x * f[1] + ph[1]) +
			jag * Math.abs(Math.sin(x * f[2] + ph[2]));
		out.push(Math.max(0.05, h) * amp);
	}
	return out;
}

function prefersReducedMotion(): boolean {
	return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export function playDayNightCycle(opts: CycleOptions): Promise<void> {
	if (prefersReducedMotion()) {
		opts.onSwitch();
		return Promise.resolve();
	}

	const W = window.innerWidth;
	const H = window.innerHeight;
	const dpr = Math.min(window.devicePixelRatio || 1, 2);
	const base = Math.min(W, H);
	const horizon = H * 0.66;
	const shoreTop = H * 0.88;
	const toNight = opts.to === 'dark';

	const canvas = document.createElement('canvas');
	canvas.setAttribute('aria-hidden', 'true');
	canvas.width = Math.round(W * dpr);
	canvas.height = Math.round(H * dpr);
	Object.assign(canvas.style, {
		position: 'fixed',
		inset: '0',
		width: `${W}px`,
		height: `${H}px`,
		zIndex: '2147483000',
		cursor: 'pointer',
	});
	const ctx = canvas.getContext('2d');
	if (!ctx) {
		opts.onSwitch();
		return Promise.resolve();
	}
	ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

	const fontDisplay =
		getComputedStyle(document.documentElement).getPropertyValue('--font-display').trim() ||
		'system-ui, sans-serif';
	const fontMono =
		getComputedStyle(document.documentElement).getPropertyValue('--font-mono').trim() ||
		'ui-monospace, monospace';

	// ── The scenery, generated once per cycle ─────────────────────────────

	const farStep = 6;
	const far = ridge(W, H * 0.2, farStep, 0.35);
	const mid = ridge(W, H * 0.12, farStep, 0.2);
	const shore = ridge(W, H * 0.05, farStep, 0.1);

	const pines = Array.from({ length: Math.ceil(W / 28) }, () => ({
		x: rand(-20, W + 20),
		h: rand(0.06, 0.15) * H,
	}))
		// Keep the middle of the shore open so the reflection stays visible.
		.filter((p) => Math.abs(p.x - W / 2) > W * 0.18 || Math.random() < 0.2);

	const stars = Array.from({ length: Math.round((W * horizon) / 4200) }, () => ({
		x: rand(0, W),
		y: rand(0, horizon * 0.95),
		r: Math.random() < 0.08 ? rand(1.3, 2) : rand(0.4, 1.1),
		phase: rand(0, TAU),
		speed: rand(0.002, 0.007),
		tint: (Math.random() < 0.15 ? [255, 214, 180] : Math.random() < 0.2 ? [190, 210, 255] : [255, 255, 255]) as RGB,
	}));

	const clouds = Array.from({ length: 6 }, (_, i) => ({
		x: rand(-0.1, 1) * W,
		y: rand(0.08, 0.4) * horizon,
		scale: rand(0.6, 1.3) * (base / 700),
		speed: rand(0.01, 0.03) * (i % 2 ? 1 : 0.6),
		puffs: Array.from({ length: 5 }, (_, j) => ({ dx: (j - 2) * 34 + rand(-8, 8), dy: rand(-14, 8), r: rand(24, 42) })),
	}));

	const shootingStars = [0.36, 0.47, 0.55, 0.63, 0.71].map((at) => ({
		at,
		x: rand(0.1, 0.9) * W,
		y: rand(0.04, 0.3) * H,
		angle: rand(0.35, 0.65) * Math.PI,
		len: rand(0.18, 0.32) * base,
	}));

	const fireflies = Array.from({ length: 42 }, () => ({
		x: rand(0, W),
		y: rand(horizon + 10, H),
		phase: rand(0, TAU),
		drift: rand(10, 30),
	}));

	const flock = Array.from({ length: 9 }, (_, i) => ({
		dx: -Math.abs(i - 4) * 26 + rand(-6, 6),
		dy: (i - 4) * 14 + Math.abs(i - 4) * 8 + rand(-5, 5),
		phase: rand(0, TAU),
		size: rand(7, 11),
	}));

	// ── Where the sun and moon are for a given darkness d ─────────────────

	const sunR = base * 0.06;
	const moonR = base * 0.045;

	function sunAt(d: number) {
		return {
			x: lerp(W * 0.74, W * 0.56, d),
			y: lerp(H * 0.16, horizon + sunR * 1.6, d ** 1.4),
			r: sunR * lerp(1, 1.35, d),
		};
	}

	function moonAt(d: number) {
		const m = easeOut(range(d, 0.35, 1));
		return { x: lerp(W * 0.12, W * 0.28, m), y: lerp(horizon + moonR * 2, H * 0.19, m), m };
	}

	const reveal = toNight ? moonAt(1) : sunAt(0);
	const farthest = (x: number, y: number) =>
		Math.max(Math.hypot(x, y), Math.hypot(W - x, y), Math.hypot(x, H - y), Math.hypot(W - x, H - y));
	const irisInMax = farthest(opts.origin.x, opts.origin.y) + 4;
	const feather = 80;
	const irisOutMax = farthest(reveal.x, reveal.y) + feather;

	// ── Painters ──────────────────────────────────────────────────────────

	function paintSky(d: number) {
		const g = ctx!.createLinearGradient(0, 0, 0, horizon);
		g.addColorStop(0, rgb(ramp(SKY_TOP, d)));
		g.addColorStop(0.6, rgb(ramp(SKY_MID, d)));
		g.addColorStop(1, rgb(ramp(SKY_LOW, d)));
		ctx!.fillStyle = g;
		ctx!.fillRect(0, 0, W, horizon + 1);
	}

	function paintStars(d: number, t: number) {
		const a = range(d, 0.45, 0.85);
		if (a <= 0) return;
		for (const s of stars) {
			const tw = 0.55 + 0.45 * Math.sin(t * s.speed + s.phase);
			ctx!.fillStyle = rgb(s.tint, a * tw);
			ctx!.beginPath();
			ctx!.arc(s.x, s.y, s.r, 0, TAU);
			ctx!.fill();
			if (s.r > 1.2) {
				// The bright ones get a four-point glint.
				ctx!.strokeStyle = rgb(s.tint, a * tw * 0.5);
				ctx!.lineWidth = 0.6;
				const l = s.r * 4 * tw;
				ctx!.beginPath();
				ctx!.moveTo(s.x - l, s.y);
				ctx!.lineTo(s.x + l, s.y);
				ctx!.moveTo(s.x, s.y - l);
				ctx!.lineTo(s.x, s.y + l);
				ctx!.stroke();
			}
		}
	}

	function paintAurora(d: number, t: number) {
		const a = range(d, 0.6, 0.95) * 0.5;
		if (a <= 0) return;
		ctx!.save();
		ctx!.globalCompositeOperation = 'lighter';
		const bands: [RGB, RGB][] = [
			[[60, 255, 170], [120, 80, 255]],
			[[90, 220, 255], [60, 255, 160]],
			[[200, 90, 255], [60, 200, 255]],
		];
		bands.forEach(([c1, c2], i) => {
			const top = H * (0.06 + i * 0.07);
			const g = ctx!.createLinearGradient(0, top - H * 0.08, 0, top + H * 0.3);
			g.addColorStop(0, rgb(c2, 0));
			g.addColorStop(0.3, rgb(c2, a * 0.6));
			g.addColorStop(0.55, rgb(c1, a));
			g.addColorStop(1, rgb(c1, 0));
			ctx!.fillStyle = g;
			for (let x = 0; x < W; x += 4) {
				const wave =
					Math.sin(x * 0.004 + t * 0.0011 + i * 1.7) * H * 0.05 +
					Math.sin(x * 0.011 - t * 0.0017 + i) * H * 0.015;
				const len = H * 0.16 * (0.55 + 0.45 * Math.sin(x * 0.007 + t * 0.002 + i * 2.3));
				ctx!.globalAlpha = 0.5 + 0.5 * Math.sin(x * 0.02 + t * 0.003 + i);
				ctx!.fillRect(x, top + wave, 3, len);
			}
		});
		ctx!.restore();
	}

	function paintShootingStars(d: number, p: number) {
		const a = range(d, 0.5, 0.8);
		if (a <= 0) return;
		for (const s of shootingStars) {
			const q = range(p, s.at, s.at + 0.09);
			if (q <= 0 || q >= 1) continue;
			const head = easeOut(q) * s.len;
			const tail = Math.min(head, s.len * 0.4);
			const hx = s.x + Math.cos(s.angle) * head;
			const hy = s.y + Math.sin(s.angle) * head;
			const tx = s.x + Math.cos(s.angle) * (head - tail);
			const ty = s.y + Math.sin(s.angle) * (head - tail);
			const fade = a * Math.sin(q * Math.PI);
			const g = ctx!.createLinearGradient(tx, ty, hx, hy);
			g.addColorStop(0, 'rgb(255 255 255 / 0)');
			g.addColorStop(1, `rgb(255 255 255 / ${fade})`);
			ctx!.strokeStyle = g;
			ctx!.lineWidth = 2;
			ctx!.lineCap = 'round';
			ctx!.beginPath();
			ctx!.moveTo(tx, ty);
			ctx!.lineTo(hx, hy);
			ctx!.stroke();
			ctx!.fillStyle = `rgb(255 255 255 / ${fade})`;
			ctx!.beginPath();
			ctx!.arc(hx, hy, 1.8, 0, TAU);
			ctx!.fill();
		}
	}

	function paintSun(d: number, t: number) {
		const sun = sunAt(d);
		const color = mix(SUN_HIGH, SUN_LOW, easeInOut(d));
		ctx!.save();
		ctx!.globalCompositeOperation = 'lighter';

		// God rays, wheeling slowly while the sun is up.
		const rayA = (1 - d) * 0.16;
		if (rayA > 0.005) {
			ctx!.save();
			ctx!.translate(sun.x, sun.y);
			ctx!.rotate(t * 0.00018);
			for (let i = 0; i < 14; i++) {
				ctx!.rotate(TAU / 14);
				const len = base * (0.6 + 0.25 * Math.sin(t * 0.002 + i * 1.3));
				const g = ctx!.createLinearGradient(0, 0, len, 0);
				g.addColorStop(0, rgb(color, rayA));
				g.addColorStop(1, rgb(color, 0));
				ctx!.fillStyle = g;
				ctx!.beginPath();
				ctx!.moveTo(0, 0);
				ctx!.lineTo(len, -len * 0.06);
				ctx!.lineTo(len, len * 0.06);
				ctx!.closePath();
				ctx!.fill();
			}
			ctx!.restore();
		}

		// Halo, wider and redder as it sinks.
		const halo = ctx!.createRadialGradient(sun.x, sun.y, sun.r * 0.5, sun.x, sun.y, sun.r * lerp(5, 9, d));
		halo.addColorStop(0, rgb(color, 0.55));
		halo.addColorStop(0.35, rgb(color, 0.16));
		halo.addColorStop(1, rgb(color, 0));
		ctx!.fillStyle = halo;
		ctx!.fillRect(0, 0, W, horizon);
		ctx!.restore();

		const disc = ctx!.createRadialGradient(sun.x - sun.r * 0.3, sun.y - sun.r * 0.3, 0, sun.x, sun.y, sun.r);
		disc.addColorStop(0, rgb(mix(color, [255, 255, 255], 0.6)));
		disc.addColorStop(1, rgb(color));
		ctx!.fillStyle = disc;
		ctx!.beginPath();
		ctx!.arc(sun.x, sun.y, sun.r, 0, TAU);
		ctx!.fill();

		// Lens-flare ghosts strung along the line through the screen centre.
		const flareA = range(1 - d, 0.55, 1) * 0.22;
		if (flareA > 0) {
			ctx!.save();
			ctx!.globalCompositeOperation = 'lighter';
			const cx = W / 2;
			const cy = H / 2;
			const ghosts: [number, number, RGB][] = [
				[0.5, 0.5, [255, 200, 120]],
				[0.9, 0.25, [120, 200, 255]],
				[1.3, 0.8, [200, 140, 255]],
				[1.7, 0.4, [255, 240, 160]],
			];
			for (const [k, size, c] of ghosts) {
				const gx = sun.x + (cx - sun.x) * k * 2;
				const gy = sun.y + (cy - sun.y) * k * 2;
				const gr = sun.r * size * 1.6;
				const g = ctx!.createRadialGradient(gx, gy, 0, gx, gy, gr);
				g.addColorStop(0, rgb(c, flareA));
				g.addColorStop(0.7, rgb(c, flareA * 0.4));
				g.addColorStop(1, rgb(c, 0));
				ctx!.fillStyle = g;
				ctx!.beginPath();
				ctx!.arc(gx, gy, gr, 0, TAU);
				ctx!.fill();
			}
			ctx!.restore();
		}
	}

	function paintMoon(d: number) {
		const moon = moonAt(d);
		if (moon.m <= 0) return;
		const a = range(d, 0.35, 0.6);
		ctx!.save();
		ctx!.globalCompositeOperation = 'lighter';
		const glow = ctx!.createRadialGradient(moon.x, moon.y, moonR, moon.x, moon.y, moonR * 6);
		glow.addColorStop(0, `rgb(190 210 255 / ${0.35 * a})`);
		glow.addColorStop(1, 'rgb(190 210 255 / 0)');
		ctx!.fillStyle = glow;
		ctx!.fillRect(moon.x - moonR * 6, moon.y - moonR * 6, moonR * 12, moonR * 12);
		ctx!.restore();

		ctx!.globalAlpha = a;
		const disc = ctx!.createRadialGradient(moon.x - moonR * 0.4, moon.y - moonR * 0.4, 0, moon.x, moon.y, moonR);
		disc.addColorStop(0, '#fbf8ef');
		disc.addColorStop(1, '#cfd3dc');
		ctx!.fillStyle = disc;
		ctx!.beginPath();
		ctx!.arc(moon.x, moon.y, moonR, 0, TAU);
		ctx!.fill();
		// Maria.
		ctx!.fillStyle = 'rgb(150 156 172 / 0.35)';
		for (const [ox, oy, r] of [
			[-0.3, -0.2, 0.22],
			[0.25, 0.1, 0.3],
			[-0.1, 0.4, 0.15],
			[0.35, -0.4, 0.1],
		]) {
			ctx!.beginPath();
			ctx!.arc(moon.x + ox * moonR, moon.y + oy * moonR, r * moonR, 0, TAU);
			ctx!.fill();
		}
		ctx!.globalAlpha = 1;
	}

	function paintClouds(d: number, t: number) {
		const a = (1 - range(d, 0.55, 0.95) * 0.75) * 0.9;
		const color = ramp(CLOUD, d);
		for (const c of clouds) {
			const x = ((c.x + t * c.speed + 200) % (W + 400)) - 200;
			ctx!.fillStyle = rgb(color, a * 0.85);
			ctx!.beginPath();
			for (const puff of c.puffs) {
				ctx!.moveTo(x + puff.dx * c.scale + puff.r * c.scale, c.y + puff.dy * c.scale);
				ctx!.arc(x + puff.dx * c.scale, c.y + puff.dy * c.scale, puff.r * c.scale, 0, TAU);
			}
			ctx!.fill();
		}
	}

	function paintRidge(heights: number[], baseY: number, color: RGB) {
		ctx!.fillStyle = rgb(color);
		ctx!.beginPath();
		ctx!.moveTo(0, H);
		heights.forEach((h, i) => ctx!.lineTo(i * farStep, baseY - h));
		ctx!.lineTo(W, H);
		ctx!.closePath();
		ctx!.fill();
	}

	function paintLake(d: number, t: number, lift: number) {
		const top = horizon + lift;
		const g = ctx!.createLinearGradient(0, top, 0, H);
		g.addColorStop(0, rgb(mix(ramp(SKY_LOW, d), ramp(SHORE, d), 0.35)));
		g.addColorStop(1, rgb(mix(ramp(SKY_TOP, d), ramp(SHORE, d), 0.6)));
		ctx!.fillStyle = g;
		ctx!.fillRect(0, top, W, H - top);

		// Shimmering reflections of whichever light is above the water.
		const sun = sunAt(d);
		const moon = moonAt(d);
		const lights: [number, RGB, number][] = [
			[sun.x, mix(SUN_HIGH, SUN_LOW, d), clamp01((horizon - sun.y) / (H * 0.4)) * 0.9],
			[moon.x, [220, 230, 255], range(d, 0.4, 0.8) * 0.7],
		];
		ctx!.save();
		ctx!.globalCompositeOperation = 'lighter';
		for (const [lx, color, strength] of lights) {
			if (strength <= 0.01) continue;
			for (let y = top + 3; y < H; y += 5) {
				const depth = (y - top) / (H - top);
				const w = (sunR * 2.2) * (1 - depth * 0.4) * (0.4 + 0.6 * Math.abs(Math.sin(y * 0.3 + t * 0.006)));
				const off = Math.sin(y * 0.12 + t * 0.004) * 6;
				ctx!.fillStyle = rgb(color, strength * (1 - depth) * 0.6);
				ctx!.fillRect(lx - w / 2 + off, y, w, 1.6);
			}
		}
		ctx!.restore();
	}

	function paintShore(d: number, lift: number) {
		const color = ramp(SHORE, d);
		paintRidge(shore, shoreTop + lift + H * 0.05, color);
		ctx!.fillStyle = rgb(color);
		for (const p of pines) {
			const baseY = shoreTop + lift + H * 0.05 - shore[Math.max(0, Math.min(shore.length - 1, Math.round(p.x / farStep)))] + 4;
			const w = p.h * 0.36;
			ctx!.beginPath();
			for (let k = 0; k < 3; k++) {
				const tierTop = baseY - p.h + k * p.h * 0.24;
				const tierW = w * (0.5 + k * 0.28);
				ctx!.moveTo(p.x, tierTop);
				ctx!.lineTo(p.x + tierW / 2, tierTop + p.h * 0.42);
				ctx!.lineTo(p.x - tierW / 2, tierTop + p.h * 0.42);
				ctx!.closePath();
			}
			ctx!.fill();
			ctx!.fillRect(p.x - 1.5, baseY - p.h * 0.15, 3, p.h * 0.18);
		}
	}

	function paintFireflies(d: number, t: number) {
		const a = range(d, 0.6, 0.95);
		if (a <= 0) return;
		ctx!.save();
		ctx!.globalCompositeOperation = 'lighter';
		for (const f of fireflies) {
			const x = f.x + Math.sin(t * 0.0012 + f.phase) * f.drift;
			const y = f.y + Math.cos(t * 0.0009 + f.phase * 1.3) * f.drift * 0.6;
			const blink = a * (0.5 + 0.5 * Math.sin(t * 0.006 + f.phase * 3)) ** 2;
			const g = ctx!.createRadialGradient(x, y, 0, x, y, 9);
			g.addColorStop(0, `rgb(230 255 140 / ${blink})`);
			g.addColorStop(1, 'rgb(230 255 140 / 0)');
			ctx!.fillStyle = g;
			ctx!.fillRect(x - 9, y - 9, 18, 18);
		}
		ctx!.restore();
	}

	function paintBirds(d: number, p: number, t: number) {
		if (toNight) return;
		const q = range(p, 0.42, 0.9);
		if (q <= 0 || q >= 1) return;
		const a = (1 - d) * Math.sin(q * Math.PI);
		const leadX = lerp(-W * 0.1, W * 1.15, q);
		const leadY = H * 0.3 + Math.sin(q * 5) * H * 0.03;
		ctx!.strokeStyle = `rgb(30 34 44 / ${a})`;
		ctx!.lineWidth = 1.8;
		ctx!.lineCap = 'round';
		for (const b of flock) {
			const x = leadX + b.dx;
			const y = leadY + b.dy;
			const flap = Math.sin(t * 0.02 + b.phase) * b.size * 0.6;
			ctx!.beginPath();
			ctx!.moveTo(x - b.size, y - flap);
			ctx!.quadraticCurveTo(x - b.size * 0.4, y - b.size * 0.25, x, y);
			ctx!.quadraticCurveTo(x + b.size * 0.4, y - b.size * 0.25, x + b.size, y - flap);
			ctx!.stroke();
		}
	}

	function paintCaption(p: number, s: number) {
		const a = range(p, 0.3, 0.4) * (1 - range(p, 0.7, 0.8));
		if (a <= 0) return;
		const [h0, h1] = toNight ? [17.5, 22] : [5, 8.5];
		const hours = lerp(h0, h1, s);
		const hh = String(Math.floor(hours)).padStart(2, '0');
		const mm = String(Math.floor((hours % 1) * 60)).padStart(2, '0');
		const scale = lerp(0.94, 1.04, range(p, 0.3, 0.8));

		ctx!.save();
		ctx!.translate(W / 2, H * 0.42);
		ctx!.scale(scale, scale);
		ctx!.textAlign = 'center';
		ctx!.shadowColor = 'rgb(0 0 0 / 0.35)';
		ctx!.shadowBlur = 24;
		ctx!.fillStyle = `rgb(255 255 255 / ${a * 0.85})`;
		ctx!.font = `500 ${Math.round(base * 0.026)}px ${fontMono}`;
		ctx!.fillText(`${hh}:${mm}`, 0, -base * 0.07);
		ctx!.fillStyle = `rgb(255 255 255 / ${a})`;
		ctx!.font = `500 ${Math.round(base * 0.1)}px ${fontDisplay}`;
		ctx!.fillText(opts.label, 0, base * 0.03);
		ctx!.restore();
	}

	function paintScene(p: number, t: number) {
		const s = easeInOutSine(range(p, 0.06, IRIS_OUT_START));
		const d = toNight ? s : 1 - s;

		// The land rises into place with a little overshoot, far ridges last.
		const rise = (delay: number) => (1 - easeOutBack(range(p, delay, delay + 0.26))) * H * 0.35;

		paintSky(d);
		paintStars(d, t);
		paintAurora(d, t);
		paintShootingStars(d, p);
		paintSun(d, t);
		paintMoon(d);
		paintClouds(d, t);
		paintBirds(d, p, t);
		paintRidge(far, horizon + rise(0.06), ramp(RIDGE_FAR, d));
		paintRidge(mid, horizon + rise(0.03), ramp(RIDGE_MID, d));
		paintLake(d, t, rise(0.03));
		paintFireflies(d, t);
		paintShore(d, rise(0));
		paintCaption(p, s);

		// A soft bloom right as the theme flips underneath.
		const flash = 1 - range(p, SWITCH_AT, SWITCH_AT + 0.08);
		if (p >= SWITCH_AT && flash > 0) {
			ctx!.fillStyle = `rgb(255 255 255 / ${flash * 0.18})`;
			ctx!.fillRect(0, 0, W, H);
		}
	}

	// ── Timeline ──────────────────────────────────────────────────────────

	return new Promise<void>((resolve) => {
		let start = performance.now();
		let switched = false;
		let frame = 0;

		const doSwitch = () => {
			if (switched) return;
			switched = true;
			opts.onSwitch();
		};

		// Skipping jumps straight to the reveal.
		const skip = () => {
			const p = (performance.now() - start) / DURATION;
			if (p >= IRIS_OUT_START) return;
			doSwitch();
			start = performance.now() - IRIS_OUT_START * DURATION;
		};
		const onKey = (e: KeyboardEvent) => {
			e.preventDefault();
			skip();
		};

		const finish = () => {
			cancelAnimationFrame(frame);
			doSwitch();
			canvas.removeEventListener('pointerdown', skip);
			window.removeEventListener('keydown', onKey, true);
			canvas.remove();
			resolve();
		};

		const tick = (now: number) => {
			const t = now - start;
			const p = clamp01(t / DURATION);
			if (p >= SWITCH_AT) doSwitch();

			ctx.clearRect(0, 0, W, H);
			ctx.save();
			if (p < IRIS_IN_END) {
				const r = easeInOut(range(p, 0, IRIS_IN_END)) * irisInMax;
				ctx.beginPath();
				ctx.arc(opts.origin.x, opts.origin.y, r, 0, TAU);
				ctx.clip();
			}
			paintScene(p, t);
			ctx.restore();

			if (p < IRIS_IN_END) {
				// A bright rim on the expanding sky.
				const r = easeInOut(range(p, 0, IRIS_IN_END)) * irisInMax;
				ctx.strokeStyle = `rgb(255 240 200 / ${0.8 * (1 - range(p, 0.1, IRIS_IN_END))})`;
				ctx.lineWidth = 3;
				ctx.shadowColor = 'rgb(255 220 150 / 0.9)';
				ctx.shadowBlur = 30;
				ctx.beginPath();
				ctx.arc(opts.origin.x, opts.origin.y, r, 0, TAU);
				ctx.stroke();
				ctx.shadowBlur = 0;
			}

			if (p > IRIS_OUT_START) {
				const r = easeInOut(range(p, IRIS_OUT_START, 1)) * irisOutMax;
				const g = ctx.createRadialGradient(reveal.x, reveal.y, Math.max(0, r - feather), reveal.x, reveal.y, Math.max(1, r));
				g.addColorStop(0, 'rgb(0 0 0 / 1)');
				g.addColorStop(1, 'rgb(0 0 0 / 0)');
				ctx.globalCompositeOperation = 'destination-out';
				ctx.fillStyle = g;
				ctx.fillRect(0, 0, W, H);
				ctx.globalCompositeOperation = 'source-over';
			}

			if (p >= 1) finish();
			else frame = requestAnimationFrame(tick);
		};

		canvas.addEventListener('pointerdown', skip);
		window.addEventListener('keydown', onKey, true);
		document.body.appendChild(canvas);
		frame = requestAnimationFrame(tick);
	});
}
