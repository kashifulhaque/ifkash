import { redirect } from '@sveltejs/kit';
import { browser } from '$app/environment';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url }) => {
	// Check authentication
	const token = browser ? localStorage.getItem('auth_token') : null;

	if (!token) {
		throw redirect(307, '/login');
	}

	try {
		const resumeId = url.searchParams.get('id');
		const endpoint = resumeId ? `/api/resume/record/${resumeId}` : '/api/resume/latest';

		// Determine API URL based on environment
		const baseUrl = browser && window.location.hostname === 'localhost'
			? 'http://localhost:8787'
			: 'https://ifkash.dev';

		const response = await fetch(`${baseUrl}${endpoint}`, {
			headers: {
				'Authorization': `Bearer ${token}`
			}
		});

		if (!response.ok) {
			// If unauthorized, redirect to login
			if (response.status === 401) {
				// Clear invalid token
				if (browser) {
					localStorage.removeItem('auth_token');
					localStorage.removeItem('auth_user');
				}
				throw redirect(307, '/login');
			}
			throw new Error('Failed to fetch resume data');
		}

		const data = await response.json();

		return {
			resume: {
				id: data.id,
				version: data.version,
				pdf_url: data.pdf_url,
				typst_url: data.typst_url,
				typst_filename: data.typst_filename,
				updated: data.updated,
				created: data.created
			}
		};
	} catch (error) {
		// If it's already a redirect, re-throw it
		if (error instanceof Response && error.status === 307) {
			throw error;
		}

		console.error('Error loading resume:', error);
		throw error;
	}
};

// Disable SSR for this page since it requires browser APIs
export const ssr = false;
