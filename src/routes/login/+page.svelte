<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';

	let email = '';
	let password = '';
	let error = '';
	let loading = false;

	async function handleLogin() {
		error = '';
		loading = true;

		try {
			await auth.login(email, password);
			goto('/editor');
		} catch (e) {
			error = 'Invalid email or password';
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Login — Kashif</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center px-4">
	<div class="max-w-md w-full">
		<div class="text-center mb-8">
			<h1 class="text-3xl font-bold text-[var(--color-headline)] mb-2">Resume Editor</h1>
			<p class="text-[var(--color-paragraph)]">Sign in to edit your resume</p>
		</div>

		<form
			on:submit|preventDefault={handleLogin}
			class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg p-8"
		>
			{#if error}
				<div class="mb-4 p-3 bg-red-500/10 border border-red-500/20 rounded text-red-500 text-sm">
					{error}
				</div>
			{/if}

			<div class="mb-4">
				<label for="email" class="block text-sm font-medium text-[var(--color-headline)] mb-2">
					Email
				</label>
				<input
					type="email"
					id="email"
					bind:value={email}
					required
					class="w-full px-4 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg focus:outline-none focus:border-[var(--color-secondary)] text-[var(--color-headline)]"
					placeholder="your@email.com"
				/>
			</div>

			<div class="mb-6">
				<label for="password" class="block text-sm font-medium text-[var(--color-headline)] mb-2">
					Password
				</label>
				<input
					type="password"
					id="password"
					bind:value={password}
					required
					class="w-full px-4 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg focus:outline-none focus:border-[var(--color-secondary)] text-[var(--color-headline)]"
					placeholder="••••••••"
				/>
			</div>

			<button
				type="submit"
				disabled={loading}
				class="w-full bg-[var(--color-secondary)] hover:bg-[var(--color-highlight)] text-white font-semibold py-2 px-4 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
			>
				{loading ? 'Signing in...' : 'Sign In'}
			</button>
		</form>

		<div class="mt-6 text-center">
			<a href="/" class="text-sm text-[var(--color-secondary)] hover:text-[var(--color-highlight)]">
				← Back to home
			</a>
		</div>
	</div>
</div>
