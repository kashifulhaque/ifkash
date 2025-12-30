import { auth } from '$lib/stores/auth';
import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	// Check if user is authenticated
	const token = auth.getToken();
	
	if (!token) {
		throw redirect(302, '/login');
	}

	// Fetch latest resume data
	const apiUrl =
		typeof window !== 'undefined' && window.location.hostname === 'localhost'
			? 'http://localhost:8787/api/resume/latest'
			: 'https://ifkash.dev/api/resume/latest';

	try {
		const response = await fetch(apiUrl, {
			headers: {
				Authorization: `Bearer ${token}`
			}
		});

		if (!response.ok) {
			if (response.status === 401) {
				// Token invalid, redirect to login
				throw redirect(302, '/login');
			}
			throw new Error('Failed to fetch resume');
		}

		const data = await response.json();
		return { resume: data };
	} catch (error) {
		console.error('Error loading resume:', error);
		throw redirect(302, '/login');
	}
};
