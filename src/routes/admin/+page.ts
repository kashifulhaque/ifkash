import { protectedFetch } from '$lib/access';
import type { PageLoad } from './$types';

// Admin dashboard — protected by Cloudflare Access at the edge. The whoami
// call both gates the page (unauthenticated users get bounced to the Access
// login by $lib/access) and tells us who is signed in.
export const load: PageLoad = async ({ fetch }) => {
	const response = await protectedFetch(fetch, '/api/whoami');
	const { email } = await response.json();
	return { email };
};

export const ssr = false;
