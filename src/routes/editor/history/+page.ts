import { browser } from '$app/environment';
import type { PageLoad } from './$types';

// Protected by Cloudflare Access (same as /editor). Same-origin fetch carries
// the Access cookie; localhost bypasses the check for local development.
export const load: PageLoad = async ({ fetch }) => {
	const baseUrl =
		browser && window.location.hostname === 'localhost' ? 'http://localhost:8787' : 'https://ifkash.dev';

	const response = await fetch(`${baseUrl}/api/resume/history`);

	if (!response.ok) {
		throw new Error(`Failed to load history (${response.status})`);
	}

	const history = await response.json();
	return { history };
};

export const ssr = false;
