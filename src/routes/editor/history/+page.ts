import { browser } from '$app/environment';
import type { PageLoad } from './$types';

// Protected by Cloudflare Access (same as /editor). Same-origin fetch carries
// the Access cookie; localhost bypasses the check for local development.
export const load: PageLoad = async ({ fetch }) => {
	const baseUrl =
		browser && window.location.hostname === 'localhost' ? 'http://localhost:8787' : 'https://ifkash.dev';

	const target = `${baseUrl}/api/resume/history`;

	// Cloudflare Access answers an unauthenticated request with a cross-origin 302
	// to its login page. A `fetch` can't follow that (CORS), so detect it and do a
	// full-page navigation instead, which lets the browser run the Access login flow.
	let response: Response;
	try {
		response = await fetch(target, { redirect: 'manual' });
	} catch {
		if (browser) {
			window.location.href = target;
			throw new Error('Redirecting to sign in…');
		}
		throw new Error('Failed to load history');
	}

	if (response.type === 'opaqueredirect' || response.status === 0) {
		if (browser) {
			window.location.href = target;
			throw new Error('Redirecting to sign in…');
		}
		throw new Error('Authentication required');
	}

	if (!response.ok) {
		throw new Error(`Failed to load history (${response.status})`);
	}

	const history = await response.json();
	return { history };
};

export const ssr = false;
