import { readable } from 'svelte/store';
import { browser } from '$app/environment';

/**
 * Local time in Bangalore as `HH:MM`, for the footer coordinate line.
 * The value is an empty string on the server and before the first tick,
 * so render it only when it is non-empty. The timer runs while at least
 * one component subscribes.
 */
export const clock = readable('', (set) => {
	if (!browser) return;
	const fmt = new Intl.DateTimeFormat('en-GB', {
		hour: '2-digit',
		minute: '2-digit',
		timeZone: 'Asia/Kolkata',
	});
	const tick = () => set(fmt.format(new Date()));
	tick();
	const id = setInterval(tick, 15_000);
	return () => clearInterval(id);
});
