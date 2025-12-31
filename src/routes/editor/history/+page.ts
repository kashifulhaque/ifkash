import { redirect } from '@sveltejs/kit';
import { browser } from '$app/environment';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
    const token = browser ? localStorage.getItem('auth_token') : null;

    if (!token) {
        throw redirect(307, '/login');
    }

    try {
        const baseUrl = browser && window.location.hostname === 'localhost'
            ? 'http://localhost:8787'
            : 'https://ifkash.dev';

        const response = await fetch(`${baseUrl}/api/resume/history`, {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (!response.ok) {
            if (response.status === 401) {
                if (browser) {
                    localStorage.removeItem('auth_token');
                    localStorage.removeItem('auth_user');
                }
                throw redirect(307, '/login');
            }
            throw new Error('Failed to fetch history');
        }

        const history = await response.json();
        return { history };
    } catch (error) {
        if (error instanceof Response && error.status === 307) {
            throw error;
        }
        console.error('Error loading history:', error);
        throw error;
    }
};

export const ssr = false;
