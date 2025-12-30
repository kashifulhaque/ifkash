import { writable } from 'svelte/store';
import { browser } from '$app/environment';

interface AuthState {
	isAuthenticated: boolean;
	token: string | null;
	user: {
		id: string;
		email: string;
	} | null;
}

const initialState: AuthState = {
	isAuthenticated: false,
	token: null,
	user: null
};

// Load from localStorage on init
function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>(initialState);

	// Initialize from localStorage if in browser
	if (browser) {
		const token = localStorage.getItem('auth_token');
		const userStr = localStorage.getItem('auth_user');
		if (token && userStr) {
			try {
				const user = JSON.parse(userStr);
				set({ isAuthenticated: true, token, user });
			} catch (e) {
				// Invalid data, clear it
				localStorage.removeItem('auth_token');
				localStorage.removeItem('auth_user');
			}
		}
	}

	return {
		subscribe,
		login: async (email: string, password: string) => {
			const apiUrl =
				browser && window.location.hostname === 'localhost'
					? 'http://localhost:8787/api/auth/login'
					: 'https://ifkash.dev/api/auth/login';

			const response = await fetch(apiUrl, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ email, password })
			});

			if (!response.ok) {
				throw new Error('Login failed');
			}

			const data = await response.json();

			if (browser) {
				localStorage.setItem('auth_token', data.token);
				localStorage.setItem('auth_user', JSON.stringify(data.user));
			}

			set({
				isAuthenticated: true,
				token: data.token,
				user: data.user
			});

			return data;
		},
		logout: () => {
			if (browser) {
				localStorage.removeItem('auth_token');
				localStorage.removeItem('auth_user');
			}
			set(initialState);
		},
		getToken: (): string | null => {
			if (browser) {
				return localStorage.getItem('auth_token');
			}
			return null;
		}
	};
}

export const auth = createAuthStore();
