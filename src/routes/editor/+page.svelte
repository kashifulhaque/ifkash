<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import type { PageData } from './$types';

	export let data: PageData;

	let typstContent = '';
	let loading = true;
	let error = '';

	$: resume = data.resume;

	// Fetch Typst file content
	async function loadTypstContent() {
		try {
			const response = await fetch(resume.typst_url);
			if (!response.ok) throw new Error('Failed to fetch Typst file');
			typstContent = await response.text();
		} catch (e) {
			error = 'Failed to load Typst file';
			console.error(e);
		} finally {
			loading = false;
		}
	}

	$: if (resume) {
		loadTypstContent();
	}

	function handleLogout() {
		auth.logout();
		goto('/');
	}

	async function downloadPDF() {
		window.open(resume.pdf_url, '_blank');
	}

	async function downloadTypst() {
		const blob = new Blob([typstContent], { type: 'text/plain' });
		const url = window.URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = resume.typst_filename || 'resume.typ';
		document.body.appendChild(a);
		a.click();
		window.URL.revokeObjectURL(url);
		document.body.removeChild(a);
	}

	function openPocketBase() {
		window.open('https://pb.ifkash.dev/_/', '_blank');
	}
</script>

<svelte:head>
	<title>Resume Editor — Kashif</title>
</svelte:head>

<div class="min-h-screen flex flex-col">
	<!-- Header -->
	<header class="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
		<div class="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
			<div class="flex items-center gap-4">
				<h1 class="text-xl font-bold text-[var(--color-headline)]">Resume Editor</h1>
				{#if resume.version}
					<span class="text-sm text-[var(--color-paragraph)] bg-[var(--color-background)] px-2 py-1 rounded">
						v{resume.version}
					</span>
				{/if}
			</div>
			<div class="flex items-center gap-3">
				<button
					on:click={downloadTypst}
					class="px-4 py-2 text-sm border border-[var(--color-border)] rounded-lg hover:border-[var(--color-secondary)] transition-colors text-[var(--color-headline)]"
				>
					Download Typst
				</button>
				<button
					on:click={downloadPDF}
					class="px-4 py-2 text-sm bg-[var(--color-secondary)] hover:bg-[var(--color-highlight)] text-white rounded-lg transition-colors"
				>
					Download PDF
				</button>
				<button
					on:click={openPocketBase}
					class="px-4 py-2 text-sm border border-[var(--color-border)] rounded-lg hover:border-[var(--color-secondary)] transition-colors text-[var(--color-headline)]"
				>
					Upload New Version
				</button>
				<button
					on:click={handleLogout}
					class="px-4 py-2 text-sm text-[var(--color-paragraph)] hover:text-[var(--color-headline)] transition-colors"
				>
					Logout
				</button>
			</div>
		</div>
	</header>

	<!-- Main Content -->
	<div class="flex-1 flex overflow-hidden">
		<!-- Left: Typst Editor -->
		<div class="w-1/2 border-r border-[var(--color-border)] flex flex-col">
			<div class="px-4 py-3 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
				<h2 class="text-sm font-semibold text-[var(--color-headline)]">Typst Source</h2>
			</div>
			<div class="flex-1 overflow-auto p-4 bg-[var(--color-background)]">
				{#if loading}
					<div class="flex items-center justify-center h-full">
						<p class="text-[var(--color-paragraph)]">Loading...</p>
					</div>
				{:else if error}
					<div class="flex items-center justify-center h-full">
						<p class="text-red-500">{error}</p>
					</div>
				{:else}
					<pre
						class="text-sm font-mono text-[var(--color-headline)] whitespace-pre-wrap break-words">{typstContent}</pre>
				{/if}
			</div>
		</div>

		<!-- Right: PDF Preview -->
		<div class="w-1/2 flex flex-col">
			<div class="px-4 py-3 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
				<h2 class="text-sm font-semibold text-[var(--color-headline)]">PDF Preview</h2>
			</div>
			<div class="flex-1 overflow-auto bg-[var(--color-background)]">
				{#if resume.pdf_url}
					<iframe
						src={resume.pdf_url}
						title="Resume PDF"
						class="w-full h-full border-0"
					/>
				{:else}
					<div class="flex items-center justify-center h-full">
						<p class="text-[var(--color-paragraph)]">No PDF available</p>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- Footer -->
	<footer class="border-t border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3">
		<div class="max-w-7xl mx-auto flex items-center justify-between text-sm">
			<p class="text-[var(--color-paragraph)]">
				Last updated: {new Date(resume.updated).toLocaleString()}
			</p>
			<p class="text-[var(--color-paragraph)]">
				To edit: Download Typst → Edit locally → Compile with <code
					class="bg-[var(--color-background)] px-1 rounded">typst compile</code
				> → Upload via PocketBase
			</p>
		</div>
	</footer>
</div>
