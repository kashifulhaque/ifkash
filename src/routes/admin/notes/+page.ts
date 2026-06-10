import { protectedFetch } from '$lib/access';
import type { PageLoad } from './$types';

// Notes editor — gated by Cloudflare Access like the rest of /admin.
export const load: PageLoad = async ({ fetch }) => {
	const response = await protectedFetch(fetch, '/api/whoami');
	const { email } = await response.json();
	return { email };
};

export const ssr = false;
