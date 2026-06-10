import { browser } from '$app/environment';

/**
 * Central client-side companion to the Cloudflare Access protection on the
 * admin/editor pages. All protected loads go through `protectedFetch` so the
 * login-redirect handling lives in one place.
 *
 * Cloudflare Access answers an unauthenticated request with a cross-origin
 * 302 to its login page. A `fetch` can't follow that (CORS), so we detect it
 * and do a full-page navigation instead, which lets the browser run the
 * Access login flow. While that navigation is happening we suspend the load
 * forever instead of throwing — throwing would flash SvelteKit's 500 error
 * page for the instant before the browser leaves.
 */

export function protectedApiBase(): string {
	return browser && window.location.hostname === 'localhost'
		? 'http://localhost:8787'
		: 'https://ifkash.dev';
}

/** Navigate to the Access-protected URL and never resolve (we're leaving the page). */
function redirectToLogin(target: string): Promise<never> {
	window.location.href = target;
	return new Promise<never>(() => {});
}

/**
 * Fetch an Access-protected API endpoint. If the user isn't authenticated,
 * triggers the Access login flow via full-page navigation and never returns.
 * `endpoint` is a path like `/api/resume/latest`.
 */
export async function protectedFetch(
	fetchFn: typeof fetch,
	endpoint: string
): Promise<Response> {
	const target = `${protectedApiBase()}${endpoint}`;

	let response: Response;
	try {
		response = await fetchFn(target, { redirect: 'manual' });
	} catch {
		if (browser) return redirectToLogin(target);
		throw new Error(`Failed to load ${endpoint}`);
	}

	if (response.type === 'opaqueredirect' || response.status === 0) {
		if (browser) return redirectToLogin(target);
		throw new Error('Authentication required');
	}

	// 401 means Access let the request through but the Worker's OWNER_EMAIL pin
	// rejected it — redirecting to login again would just loop.
	if (response.status === 401) {
		throw new Error('This account is not authorized to access this page.');
	}

	if (!response.ok) {
		throw new Error(`Failed to load ${endpoint} (${response.status})`);
	}

	return response;
}
