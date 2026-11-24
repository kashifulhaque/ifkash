/**
 * A game-style vertical menu. The container holds links and one
 * `.selector` element. This action:
 * - slides the selector to the hovered or focused link, and back to the
 *   current page (`aria-current`) when the pointer or focus leaves;
 * - moves focus with the arrow keys, Home, and End;
 * - lets an arrow key start navigation when nothing is focused.
 */
export interface MenuNavParams {
	/** Changes when the active link may have changed, such as the path. */
	key?: unknown;
	/** Reacts to arrow keys pressed while nothing has focus. */
	global?: boolean;
	/** Called with the link that is highlighted, or null at rest. */
	onPick?: (link: HTMLElement | null) => void;
}

export function menuNav(node: HTMLElement, params: MenuNavParams = {}) {
	let opts = params;
	const selector = node.querySelector<HTMLElement>('.selector');
	const links = () => Array.from(node.querySelectorAll<HTMLElement>('a[href]'));
	let raf = 0;

	function place(target: HTMLElement | null, instant = false) {
		if (!selector) return;
		const el = target ?? node.querySelector<HTMLElement>('[aria-current="page"]');
		opts.onPick?.(target);
		if (!el || el.offsetParent === null) {
			selector.style.opacity = '0';
			return;
		}
		const nr = node.getBoundingClientRect();
		const r = el.getBoundingClientRect();
		if (instant) selector.dataset.instant = '';
		selector.style.setProperty('--sy', `${r.top - nr.top}px`);
		selector.style.setProperty('--sh', `${r.height}px`);
		selector.style.opacity = '';
		selector.dataset.state = target ? 'pick' : 'rest';
		if (instant) requestAnimationFrame(() => delete selector.dataset.instant);
	}

	const settle = (instant = false) => {
		cancelAnimationFrame(raf);
		raf = requestAnimationFrame(() => place(null, instant));
	};

	function linkFrom(event: Event): HTMLElement | null {
		return (event.target as Element | null)?.closest<HTMLElement>('a[href]') ?? null;
	}

	const onOver = (e: Event) => {
		const a = linkFrom(e);
		if (a && node.contains(a)) place(a);
	};
	const onLeave = () => {
		if (!node.contains(document.activeElement) || !document.activeElement?.matches(':focus-visible')) {
			place(null);
		}
	};
	const onFocusOut = (e: FocusEvent) => {
		if (!node.contains(e.relatedTarget as Node | null)) place(null);
	};

	function step(from: number, delta: number) {
		const list = links().filter((a) => a.offsetParent !== null);
		if (!list.length) return;
		const next = from < 0 ? (delta > 0 ? 0 : list.length - 1) : (from + delta + list.length) % list.length;
		list[next].focus();
	}

	function handleKey(e: KeyboardEvent, scopeActive: boolean) {
		if (e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
		const list = links().filter((a) => a.offsetParent !== null);
		const idx = list.indexOf(document.activeElement as HTMLElement);
		if (!scopeActive && idx < 0) return;
		switch (e.key) {
			case 'ArrowDown':
			case 'ArrowRight':
				e.preventDefault();
				step(idx, 1);
				break;
			case 'ArrowUp':
			case 'ArrowLeft':
				e.preventDefault();
				step(idx, -1);
				break;
			case 'Home':
				if (idx < 0) return;
				e.preventDefault();
				list[0]?.focus();
				break;
			case 'End':
				if (idx < 0) return;
				e.preventDefault();
				list[list.length - 1]?.focus();
				break;
		}
	}

	const onNodeKey = (e: KeyboardEvent) => handleKey(e, false);
	const onWindowKey = (e: KeyboardEvent) => {
		if (!opts.global || node.offsetParent === null) return;
		const a = document.activeElement;
		const idle = !a || a === document.body;
		if (!idle) return;
		// Only vertical keys start navigation, so the page can still scroll sideways.
		if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return;
		// A menu inside an open overlay takes priority over the page menus.
		const overlay = document.querySelector('.ember-overlay');
		if (overlay && !overlay.contains(node)) return;
		handleKey(e, true);
	};

	node.addEventListener('mouseover', onOver);
	node.addEventListener('focusin', onOver);
	node.addEventListener('mouseleave', onLeave);
	node.addEventListener('focusout', onFocusOut);
	node.addEventListener('keydown', onNodeKey);
	window.addEventListener('keydown', onWindowKey);
	const onResize = () => settle(true);
	window.addEventListener('resize', onResize);
	document.fonts?.ready.then(() => settle(true));
	settle(true);

	return {
		update(next: MenuNavParams = {}) {
			opts = next;
			settle();
		},
		destroy() {
			cancelAnimationFrame(raf);
			node.removeEventListener('mouseover', onOver);
			node.removeEventListener('focusin', onOver);
			node.removeEventListener('mouseleave', onLeave);
			node.removeEventListener('focusout', onFocusOut);
			node.removeEventListener('keydown', onNodeKey);
			window.removeEventListener('keydown', onWindowKey);
			window.removeEventListener('resize', onResize);
		},
	};
}
