import { redirect } from '@sveltejs/kit';
import { browser } from '$app/environment';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	// Check if user is already authenticated
	const token = browser ? localStorage.getItem('auth_token') : null;
	
	if (token) {
		// Already logged in, redirect to editor
		throw redirect(307, '/editor');
	}

	return {};
};

// Disable SSR for this page since it requires browser APIs
export const ssr = false;
