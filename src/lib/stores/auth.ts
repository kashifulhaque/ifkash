import { writable } from 'svelte/store';
import { browser } from '$app/environment';

interface User {
	id: string;
	email: string;
}

interface AuthState {
	isAuthenticated: boolean;
	token: string | null;
	user: User | null;
}

// Initialize from localStorage if in browser
function getInitialState(): AuthState {
	if (browser) {
		const token = localStorage.getItem('auth_token');
		const userStr = localStorage.getItem('auth_user');
		
		if (token && userStr) {
			try {
				const user = JSON.parse(userStr);
				return {
					isAuthenticated: true,
					token,
					user
				};
			} catch (e) {
				// Invalid stored data, clear it
				localStorage.removeItem('auth_token');
				localStorage.removeItem('auth_user');
			}
		}
	}
	
	return {
		isAuthenticated: false,
		token: null,
		user: null
	};
}

const authStore = writable<AuthState>(getInitialState());

export const auth = {
	subscribe: authStore.subscribe,
	
	login: async (email: string, password: string) => {
		try {
			const apiUrl = browser && window.location.hostname === 'localhost'
				? 'http://localhost:8787/api/auth/login'
				: 'https://ifkash.dev/api/auth/login';

			const response = await fetch(apiUrl, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ email, password })
			});

			if (!response.ok) {
				const error = await response.text();
				throw new Error(error || 'Login failed');
			}

			const data = await response.json();

			// Store in localStorage
			if (browser) {
				localStorage.setItem('auth_token', data.token);
				localStorage.setItem('auth_user', JSON.stringify(data.user));
			}

			// Update store
			authStore.set({
				isAuthenticated: true,
				token: data.token,
				user: data.user
			});

			return data;
		} catch (error) {
			throw error;
		}
	},

	logout: () => {
		// Clear localStorage
		if (browser) {
			localStorage.removeItem('auth_token');
			localStorage.removeItem('auth_user');
		}

		// Reset store
		authStore.set({
			isAuthenticated: false,
			token: null,
			user: null
		});
	},

	getToken: (): string | null => {
		if (browser) {
			return localStorage.getItem('auth_token');
		}
		return null;
	},

	// Check if token is valid (optional - for future use)
	verifyToken: async (): Promise<boolean> => {
		const token = auth.getToken();
		if (!token) return false;

		try {
			const apiUrl = browser && window.location.hostname === 'localhost'
				? 'http://localhost:8787/api/auth/verify'
				: 'https://ifkash.dev/api/auth/verify';

			const response = await fetch(apiUrl, {
				headers: {
					'Authorization': `Bearer ${token}`
				}
			});

			return response.ok;
		} catch {
			return false;
		}
	}
};
