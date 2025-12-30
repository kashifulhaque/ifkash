<script lang="ts">
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import MonacoEditor from '$lib/components/MonacoEditor.svelte';
	import type { PageData } from './$types';

	export let data: PageData;

	let typstContent = '';
	let pdfUrl = '';
	let loading = true;
	let compiling = false;
	let error = '';
	let compilationError = '';
	let hasUnsavedChanges = false;

	$: resume = data.resume;

	// Fetch Typst file content
	async function loadTypstContent() {
		try {
			const response = await fetch(resume.typst_url);
			if (!response.ok) throw new Error('Failed to fetch Typst file');
			typstContent = await response.text();
			pdfUrl = resume.pdf_url;
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

	async function handleCompile() {
		compilationError = '';
		compiling = true;

		try {
			const token = auth.getToken();
			const apiUrl =
				typeof window !== 'undefined' && window.location.hostname === 'localhost'
					? 'http://localhost:8787/api/typst/compile'
					: 'https://ifkash.dev/api/typst/compile';

			const response = await fetch(apiUrl, {
				method: 'POST',
				headers: {
					Authorization: `Bearer ${token}`,
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({ code: typstContent })
			});

			if (!response.ok) {
				const errorText = await response.text();
				throw new Error(errorText || 'Compilation failed');
			}

			// Get PDF blob and create object URL
			const blob = await response.blob();
			
			// Revoke old URL if exists
			if (pdfUrl && pdfUrl.startsWith('blob:')) {
				URL.revokeObjectURL(pdfUrl);
			}
			
			pdfUrl = URL.createObjectURL(blob);
			hasUnsavedChanges = false;
		} catch (e: any) {
			compilationError = e.message;
			console.error('Compilation error:', e);
		} finally {
			compiling = false;
		}
	}

	function handleEditorChange(newValue: string) {
		typstContent = newValue;
		hasUnsavedChanges = true;
	}

	async function downloadPDF() {
		if (pdfUrl.startsWith('blob:')) {
			// Download the compiled PDF
			const a = document.createElement('a');
			a.href = pdfUrl;
			a.download = 'Kashiful_Haque.pdf';
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
		} else {
			// Download from PocketBase
			window.open(resume.pdf_url, '_blank');
		}
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
		if (confirm('⚠️ Upload New Version?\n\nThis will open PocketBase admin panel. Make sure you have compiled and tested your changes before uploading.\n\nContinue?')) {
			window.open('https://pb.ifkash.dev/_/', '_blank');
		}
	}

	// Auto-compile on Ctrl/Cmd + S
	function handleKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 's') {
			e.preventDefault();
			handleCompile();
		}
	}
</script>

<svelte:head>
	<title>Resume Editor — Kashif</title>
</svelte:head>

<svelte:window on:keydown={handleKeydown} />

<div class="min-h-screen flex flex-col">
	<!-- Header -->
	<header class="border-b border-[var(--color-border)] bg-[var(--color-surface)]">
		<div class="max-w-full px-2 sm:px-4 py-3 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3">
			<div class="flex items-center gap-2 sm:gap-4 flex-wrap">
				<h1 class="text-lg sm:text-xl font-bold text-[var(--color-headline)]">Resume Editor</h1>
				{#if resume.version}
					<span
						class="text-xs sm:text-sm text-[var(--color-paragraph)] bg-[var(--color-background)] px-2 py-1 rounded"
					>
						v{resume.version}
					</span>
				{/if}
				{#if hasUnsavedChanges}
					<span class="text-xs sm:text-sm text-yellow-500">● Unsaved</span>
				{/if}
			</div>
			<div class="flex items-center gap-2 flex-wrap w-full sm:w-auto">
				<button
					on:click={handleCompile}
					disabled={compiling}
					class="px-3 sm:px-4 py-2 text-xs sm:text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-semibold flex-1 sm:flex-none"
				>
					{compiling ? 'Compiling...' : 'Compile'}
				</button>
				<button
					on:click={downloadTypst}
					class="px-3 sm:px-4 py-2 text-xs sm:text-sm bg-gray-600 hover:bg-gray-700 text-white rounded-lg transition-colors hidden sm:block"
				>
					Typst
				</button>
				<button
					on:click={downloadPDF}
					class="px-3 sm:px-4 py-2 text-xs sm:text-sm bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors flex-1 sm:flex-none"
				>
					PDF
				</button>
				<button
					on:click={openPocketBase}
					class="px-3 sm:px-4 py-2 text-xs sm:text-sm bg-orange-600 hover:bg-orange-700 text-white rounded-lg transition-colors font-semibold hidden sm:block"
				>
					Upload
				</button>
				<button
					on:click={handleLogout}
					class="px-3 sm:px-4 py-2 text-xs sm:text-sm bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
				>
					Logout
				</button>
			</div>
		</div>
	</header>

	<!-- Error Messages -->
	{#if compilationError}
		<div class="bg-red-500/10 border-b border-red-500/20 px-4 py-3">
			<p class="text-red-500 text-sm">
				<strong>Compilation Error:</strong>
				{compilationError}
			</p>
		</div>
	{/if}

	<!-- Main Content -->
	<div class="flex-1 flex flex-col md:flex-row overflow-hidden">
		<!-- Left: Typst Editor -->
		<div class="w-full md:w-1/2 border-b md:border-b-0 md:border-r border-[var(--color-border)] flex flex-col h-[50vh] md:h-auto">
			<div class="px-4 py-3 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
				<h2 class="text-sm font-semibold text-[var(--color-headline)]">Typst Source</h2>
			</div>
			<div class="flex-1 overflow-hidden">
				{#if loading}
					<div class="flex items-center justify-center h-full">
						<p class="text-[var(--color-paragraph)]">Loading...</p>
					</div>
				{:else if error}
					<div class="flex items-center justify-center h-full">
						<p class="text-red-500">{error}</p>
					</div>
				{:else}
					<MonacoEditor
						bind:value={typstContent}
						language="plaintext"
						theme="vs-dark"
						onChange={handleEditorChange}
					/>
				{/if}
			</div>
		</div>

		<!-- Right: PDF Preview -->
		<div class="w-full md:w-1/2 flex flex-col bg-[var(--color-background)] h-[50vh] md:h-auto">
			<div class="px-4 py-3 border-b border-[var(--color-border)] bg-[var(--color-surface)]">
				<h2 class="text-sm font-semibold text-[var(--color-headline)]">PDF Preview</h2>
			</div>
			<div class="flex-1 overflow-auto">
				{#if compiling}
					<div class="flex items-center justify-center h-full">
						<div class="text-center">
							<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-[var(--color-secondary)] mx-auto mb-4"></div>
							<p class="text-[var(--color-paragraph)]">Compiling...</p>
						</div>
					</div>
				{:else if pdfUrl}
					<iframe src={pdfUrl} title="Resume PDF" class="w-full h-full border-0" />
				{:else}
					<div class="flex items-center justify-center h-full">
						<p class="text-[var(--color-paragraph)] text-sm px-4 text-center">Click "Compile" to preview PDF</p>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- Footer -->
	<footer class="border-t border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3">
		<div class="max-w-full flex items-center justify-between text-sm">
			<p class="text-[var(--color-paragraph)]">
				Last updated: {new Date(resume.updated).toLocaleString()}
			</p>
			<p class="text-[var(--color-paragraph)]">
				Press <kbd class="bg-[var(--color-background)] px-1 rounded">⌘S</kbd> or
				<kbd class="bg-[var(--color-background)] px-1 rounded">Ctrl+S</kbd> to compile
			</p>
		</div>
	</footer>
</div>
