import { protectedFetch } from '$lib/access';
import type { PageLoad } from './$types';

// Protected by Cloudflare Access (same as /editor). Login-redirect handling
// lives in $lib/access; localhost bypasses the check for local development.
export const load: PageLoad = async ({ fetch }) => {
	const response = await protectedFetch(fetch, '/api/resume/history');
	const history = await response.json();
	return { history };
};

export const ssr = false;
