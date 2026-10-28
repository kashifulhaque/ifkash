/**
 * Shared anime.js motion helpers, exposed as Svelte actions.
 *
 * Every helper is a no-op when the visitor prefers reduced motion, and none of
 * them hide content during the server render: initial states are applied in
 * the browser when the action mounts, so the page still reads without
 * JavaScript.
 */
import { animate, createSpring, splitText, stagger, utils } from 'animejs';

export function prefersReducedMotion(): boolean {
	return (
		typeof window !== 'undefined' &&
		window.matchMedia('(prefers-reduced-motion: reduce)').matches
	);
}

/** Springy settle used by hover and press interactions. */
export const springOut = createSpring({ stiffness: 220, damping: 14 });

export interface RevealOptions {
	/** CSS selector for children to stagger; reveals the node itself when omitted. */
	children?: string;
	/** Start offset on the y axis, in pixels. */
	y?: number;
	/** Delay before the first element moves, in milliseconds. */
	delay?: number;
	/** Gap between staggered children, in milliseconds. */
	gap?: number;
	duration?: number;
	/** Fraction of the viewport's height, from the bottom, the node must clear to reveal. */
	threshold?: number;
}

/**
 * Fades and lifts an element, or a staggered set of its children, the first
 * time it scrolls into view.
 *
 * Usage: `<ol use:reveal={{ children: 'li', gap: 50 }}>`
 */
export function reveal(node: HTMLElement, options: RevealOptions = {}) {
	if (prefersReducedMotion()) return {};
	const { children, y = 18, delay = 0, gap = 60, duration = 700, threshold = 0.05 } = options;
	const targets = children ? Array.from(node.querySelectorAll<HTMLElement>(children)) : [node];
	if (!targets.length) return {};

	utils.set(targets, { opacity: 0, translateY: y });
	const show = () =>
		targets.forEach((t) => {
			t.style.removeProperty('opacity');
			t.style.removeProperty('transform');
		});
	let fallback: ReturnType<typeof setTimeout> | undefined;

	// A root margin rather than an intersection ratio, so nodes taller than the
	// viewport still reveal.
	const observer = new IntersectionObserver(
		(entries) => {
			if (!entries.some((e) => e.isIntersecting)) return;
			observer.disconnect();
			const animation = animate(targets, {
				opacity: [0, 1],
				translateY: [y, 0],
				duration,
				delay: stagger(gap, { start: delay }),
				ease: 'outExpo',
				// Not utils.cleanInlineStyles: it restores the hidden start state.
				onComplete: show
			});
			// Throttled renderers (background tabs, crawlers) may never run the
			// animation frames; never leave content hidden.
			fallback = setTimeout(() => {
				if (animation.completed) return;
				animation.cancel();
				show();
			}, delay + gap * targets.length + duration + 500);
		},
		{ rootMargin: `0px 0px -${Math.round(threshold * 100)}% 0px` }
	);
	observer.observe(node);

	return {
		destroy() {
			observer.disconnect();
			clearTimeout(fallback);
		}
	};
}

export interface SplitRevealOptions {
	delay?: number;
	/** Gap between characters, in milliseconds. */
	gap?: number;
}

/**
 * Splits a heading into characters and rolls them up into place. The original
 * markup is restored when the animation ends, so links and screen readers see
 * the untouched heading.
 *
 * Usage: `<h1 use:splitReveal>`
 */
export function splitReveal(node: HTMLElement, options: SplitRevealOptions = {}) {
	if (prefersReducedMotion()) return {};
	const { delay = 0, gap = 28 } = options;
	const split = splitText(node, { chars: { wrap: 'clip' } });

	const animation = animate(split.chars, {
		translateY: ['110%', '0%'],
		opacity: [0, 1],
		duration: 900,
		delay: stagger(gap, { start: delay }),
		ease: 'outExpo',
		onComplete: () => split.revert()
	});
	// Same safety net as reveal: restore the heading if frames never run.
	const fallback = setTimeout(() => {
		animation.cancel();
		split.revert();
	}, delay + gap * split.chars.length + 1400);

	return {
		destroy() {
			clearTimeout(fallback);
			animation.cancel();
			split.revert();
		}
	};
}

export interface MagneticOptions {
	/** How far the element follows the pointer, as a fraction of the offset. */
	strength?: number;
}

/**
 * Pulls an element a little toward the pointer while hovered and springs it
 * back on leave. Ignored on touch-only devices.
 *
 * Usage: `<a class="btn" use:magnetic>`
 */
export function magnetic(node: HTMLElement, options: MagneticOptions = {}) {
	if (prefersReducedMotion() || !window.matchMedia('(hover: hover)').matches) return {};
	const { strength = 0.25 } = options;

	function move(e: PointerEvent) {
		const r = node.getBoundingClientRect();
		const x = (e.clientX - (r.left + r.width / 2)) * strength;
		const y = (e.clientY - (r.top + r.height / 2)) * strength;
		animate(node, { translateX: x, translateY: y, duration: 300, ease: 'outQuad' });
	}

	function leave() {
		animate(node, { translateX: 0, translateY: 0, ease: springOut });
	}

	node.addEventListener('pointermove', move);
	node.addEventListener('pointerleave', leave);
	return {
		destroy() {
			node.removeEventListener('pointermove', move);
			node.removeEventListener('pointerleave', leave);
		}
	};
}

export interface CountUpOptions {
	/** Number of decimal places to keep while counting. */
	decimals?: number;
	duration?: number;
}

/**
 * Counts the first number in an element's text up from zero when it scrolls
 * into view, keeping any surrounding text such as "4+ years".
 *
 * Usage: `<dd use:countUp>{value}</dd>`
 */
export function countUp(node: HTMLElement, options: CountUpOptions = {}) {
	if (prefersReducedMotion()) return {};
	const text = node.textContent ?? '';
	const match = text.match(/\d+(\.\d+)?/);
	if (!match || match.index === undefined) return {};
	const { decimals = match[1] ? match[1].length - 1 : 0, duration = 1400 } = options;
	const before = text.slice(0, match.index);
	const after = text.slice(match.index + match[0].length);
	const target = parseFloat(match[0]);
	const counter = { v: 0 };
	node.textContent = before + (0).toFixed(decimals) + after;
	let fallback: ReturnType<typeof setTimeout> | undefined;

	const observer = new IntersectionObserver((entries) => {
		if (!entries.some((e) => e.isIntersecting)) return;
		observer.disconnect();
		const animation = animate(counter, {
			v: target,
			duration,
			ease: 'outExpo',
			onUpdate: () => {
				node.textContent = before + counter.v.toFixed(decimals) + after;
			}
		});
		fallback = setTimeout(() => {
			animation.cancel();
			node.textContent = text;
		}, duration + 500);
	});
	observer.observe(node);

	return {
		destroy() {
			observer.disconnect();
			clearTimeout(fallback);
		}
	};
}
