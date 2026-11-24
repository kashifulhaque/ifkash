/**
 * Geometry for the ensō (brush circle) on the home page.
 *
 * `ensoShape` is a filled outline whose width follows brush pressure: a
 * heavy start, an even middle, and a thin dry tail. `ensoCenterline` is the
 * path through the middle of that outline. The home page strokes the
 * centerline inside a mask to paint the shape in.
 */

const CX = 200;
const CY = 200;
const STEPS = 140;
const START = -Math.PI * 0.64;
const SWEEP = Math.PI * 2 * 0.93;

/** Brush pressure from 0 to 1 at position `t` along the stroke. */
function pressure(t: number): number {
	const attack = 1 - Math.exp(-t * 16);
	const release = t < 0.62 ? 1 : Math.max(0, 1 - ((t - 0.62) / 0.38) ** 1.6);
	const body = 1 - 0.18 * t;
	return attack * release * body;
}

function radius(t: number): number {
	// The circle drifts inward a little toward the end, as a hand would.
	return 138 - 20 * t * t + 3 * Math.sin(t * 8.5);
}

function point(t: number, offset: number): [number, number] {
	const a = START + t * SWEEP;
	const r = radius(t) + offset;
	return [CX + Math.cos(a) * r, CY + Math.sin(a) * r];
}

function fmt(p: [number, number]): string {
	return `${p[0].toFixed(1)} ${p[1].toFixed(1)}`;
}

function build(): { shape: string; center: string } {
	const outer: string[] = [];
	const inner: string[] = [];
	const center: string[] = [];
	for (let i = 0; i <= STEPS; i++) {
		const t = i / STEPS;
		const w = 2.5 + 31 * pressure(t);
		outer.push(fmt(point(t, w / 2)));
		inner.push(fmt(point(t, -w / 2)));
		center.push(fmt(point(t, 0)));
	}
	const shape = `M${outer.join(' L')} L${inner.reverse().join(' L')} Z`;
	return { shape, center: `M${center.join(' L')}` };
}

const built = build();

export const ensoShape = built.shape;
export const ensoCenterline = built.center;
