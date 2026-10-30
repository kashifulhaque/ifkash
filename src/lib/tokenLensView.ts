/** Pure display helpers for the Token Lens page. */

const SPECIAL = /^<\|[^|]*\|>$/;

/** True for tokens like `<|im_end|>`, which render as a pill instead of text. */
export function isSpecialToken(text: string): boolean {
	return SPECIAL.test(text);
}

/** Makes whitespace visible, for the candidate list. */
export function visualizeWhitespace(text: string): string {
	if (text === '') return '∅';
	return text
		.replace(/ /g, '␣')
		.replace(/\n/g, '↵')
		.replace(/\t/g, '⇥')
		// A lone byte of a multi-byte character (emoji, CJK) can't decode on its own.
		.replace(/\uFFFD/g, '‹byte›');
}

/** Formats a probability as a percentage, keeping small values readable. */
export function formatPercent(p: number): string {
	const pct = p * 100;
	if (pct >= 10) return `${Math.round(pct)}%`;
	if (pct >= 1) return `${pct.toFixed(1)}%`;
	if (pct > 0) return '<1%';
	return '0%';
}

/** Maps the 0..1 speed slider to the pause, in milliseconds, held on each step. */
export function holdForSpeed(speed: number): number {
	const s = Math.min(1, Math.max(0, speed));
	return Math.round((1 - s) * (1 - s) * 2400);
}

/**
 * Text for a token button in the output. A button lays out as an inline block,
 * so a newline inside it can't break the paragraph: show it as ↵ and let the
 * caller add a real line break after the button.
 */
export function inlineText(text: string): string {
	return text.replace(/[ \t]*\n/g, '↵');
}
