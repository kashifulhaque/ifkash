import { browser } from '$app/environment';
import type { PageLoad } from './$types';

// `/editor` is protected by Cloudflare Access at the edge, so this load only
// runs for authenticated users. The same-origin fetch carries the Access
// cookie automatically; on localhost the Worker bypasses the check for dev.
export const load: PageLoad = async ({ fetch, url }) => {
	const resumeId = url.searchParams.get('id');
	const endpoint = resumeId ? `/api/resume/record/${resumeId}` : '/api/resume/latest';

	const baseUrl =
		browser && window.location.hostname === 'localhost' ? 'http://localhost:8787' : 'https://ifkash.dev';

	const response = await fetch(`${baseUrl}${endpoint}`);

	if (!response.ok) {
		throw new Error(`Failed to load resume (${response.status})`);
	}

	const data = await response.json();

	return {
		resume: {
			id: data.id,
			version: data.version,
			pdf_url: data.pdf_url,
			typst_source: data.typst_source,
			typst_filename: data.typst_filename,
			updated: data.updated,
			created: data.created
		}
	};
};

// Editor relies on browser APIs (Monaco, WASM), so disable SSR.
export const ssr = false;
