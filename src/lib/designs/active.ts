import { derived, writable } from 'svelte/store';
import { DEFAULT_DESIGN } from './metas';
import { design } from '$lib/stores/design';

/**
 * Set to true by the root layout after mount. The server render and the
 * hydration pass always use the default design, so the saved design
 * takes over only after hydration. That keeps the markup consistent.
 */
export const hydrated = writable(false);

/** The design whose Shell and Home should render right now. */
export const activeDesign = derived([design, hydrated], ([$design, $hydrated]) =>
	$hydrated ? $design : DEFAULT_DESIGN,
);
