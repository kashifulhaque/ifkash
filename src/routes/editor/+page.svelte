<script lang="ts">
	import { auth } from "$lib/stores/auth";
	import { goto } from "$app/navigation";
	import MonacoEditor from "$lib/components/MonacoEditor.svelte";
	import type { PageData } from "./$types";
	import {
		Save,
		LogOut,
		UploadCloud,
		FileCode,
		FileDown,
		History,
		ChevronDown,
		CheckCircle,
		AlertTriangle,
		Play,
		Loader2,
	} from "lucide-svelte";

	export let data: PageData;

	let typstContent = "";
	let pdfUrl = "";
	let loading = true;
	let compiling = false;
	let uploading = false;
	let uploadProgress = 0;
	let error = "";
	let compilationError = "";
	let hasUnsavedChanges = false;
	let isDownloadOpen = false;

	$: resume = data.resume;

	// Fetch Typst file content
	async function loadTypstContent() {
		try {
			const response = await fetch(resume.typst_url);
			if (!response.ok) throw new Error("Failed to fetch Typst file");
			typstContent = await response.text();
			pdfUrl = resume.pdf_url;
		} catch (e) {
			error = "Failed to load Typst file";
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
		goto("/");
	}

	async function handleCompile() {
		compilationError = "";
		compiling = true;

		try {
			const token = auth.getToken();
			const apiUrl =
				typeof window !== "undefined" &&
				window.location.hostname === "localhost"
					? "http://localhost:8787/api/typst/compile"
					: "https://ifkash.dev/api/typst/compile";

			const response = await fetch(apiUrl, {
				method: "POST",
				headers: {
					Authorization: `Bearer ${token}`,
					"Content-Type": "application/json",
				},
				body: JSON.stringify({ code: typstContent }),
			});

			if (!response.ok) {
				const errorText = await response.text();
				throw new Error(errorText || "Compilation failed");
			}

			// Get PDF blob and create object URL
			const blob = await response.blob();

			// Revoke old URL if exists
			if (pdfUrl && pdfUrl.startsWith("blob:")) {
				URL.revokeObjectURL(pdfUrl);
			}

			pdfUrl = URL.createObjectURL(blob);
			hasUnsavedChanges = false;
		} catch (e: any) {
			compilationError = e.message;
			console.error("Compilation error:", e);
		} finally {
			compiling = false;
		}
	}

	function handleEditorChange(newValue: string) {
		typstContent = newValue;
		hasUnsavedChanges = true;
	}

	async function downloadPDF() {
		isDownloadOpen = false;
		if (pdfUrl.startsWith("blob:")) {
			const a = document.createElement("a");
			a.href = pdfUrl;
			a.download = "Kashiful_Haque.pdf";
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
		} else {
			window.open(resume.pdf_url, "_blank");
		}
	}

	async function downloadTypst() {
		isDownloadOpen = false;
		const blob = new Blob([typstContent], { type: "text/plain" });
		const url = window.URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = url;
		a.download = resume.typst_filename || "resume.typ";
		document.body.appendChild(a);
		a.click();
		window.URL.revokeObjectURL(url);
		document.body.removeChild(a);
	}

	async function handleUpload() {
		// 1. Ensure everything is compiled
		if (hasUnsavedChanges || (pdfUrl && !pdfUrl.startsWith("blob:"))) {
			// If unsaved OR pdfUrl is the remote one (meaning we haven't compiled locally yet for a re-upload scenario potentially, though if text didn't change it's fine. But safer to compile.)
			// Actually, if we just loaded a version and want to re-upload it as 'new' (restore case), we might not have 'unsaved changes' flag, but we should generate a fresh PDF to be sure.
			// Let's rely on unsaved changes flag mostly, but if user explicitly clicks upload, let's trigger a compile just to be safe and ensure fresh PDF blob.
			await handleCompile();
			if (compilationError) {
				alert(
					"Compilation failed. Please fix errors before uploading.",
				);
				return;
			}
		}

		if (!confirm("Upload as new version to homepage?")) {
			return;
		}

		uploading = true;
		uploadProgress = 5;
		error = "";

		const progressInterval = setInterval(() => {
			if (uploadProgress < 90) {
				uploadProgress += Math.random() * 5;
			}
		}, 300);

		try {
			const token = auth.getToken();
			const apiUrl =
				typeof window !== "undefined" &&
				window.location.hostname === "localhost"
					? "http://localhost:8787/api/resume/upload"
					: "https://ifkash.dev/api/resume/upload";

			// Fetch the PDF blob
			const pdfBlob = await fetch(pdfUrl).then((r) => r.blob());

			const formData = new FormData();
			formData.append(
				"typst_file",
				new Blob([typstContent], { type: "text/plain" }),
				resume.typst_filename || "resume.typ",
			);
			formData.append("pdf_file", pdfBlob, "resume.pdf");

			// Increment version from current loaded
			// If current is v5, we upload as v6 presumably, or backend handles it?
			// The backend just takes 'version' string. Our logic increments current.
			// If we restored v2, and current latest is v10, this will upload as v3?
			// That might be confusing. Ideally we should know the absolute latest version to increment properly.
			// But for now, let's just use what we have. It's an MVP.
			// Actually, if I load v2, `resume.version` is 2. I upload as 3.
			// But v3 might exist.
			// A better approach would be to let user specify version or just "Auto-increment" on backend.
			// Currently backend stores what we send.
			// Let's just strip version logic and let it match created date? No, version string is useful.
			// Let's parse int, but maybe we should allow user to edit it?
			// I'll stick to simple increment for now.
			const currentVersion = parseInt(resume.version || "0");
			const newVersion = (currentVersion + 1).toString();
			formData.append("version", newVersion);

			const response = await fetch(apiUrl, {
				method: "POST",
				headers: {
					Authorization: `Bearer ${token}`,
				},
				body: formData,
			});

			if (!response.ok) {
				const errorText = await response.text();
				let message = errorText;
				try {
					const json = JSON.parse(errorText);
					message = json.message || errorText;
				} catch (e) {}
				throw new Error(message || "Upload failed");
			}

			uploadProgress = 100;
			clearInterval(progressInterval);

			// Small delay to show completion
			setTimeout(() => {
				alert(`Successfully uploaded version ${newVersion}!`);
				window.location.href = "/editor"; // Clear ID and reload latest
			}, 500);
		} catch (e: any) {
			clearInterval(progressInterval);
			uploadProgress = 0;
			console.error("Upload error:", e);
			alert("Upload failed: " + e.message);
			uploading = false; // Only reset if failed
		}
	}

	// Auto-compile on Ctrl/Cmd + S
	function handleKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === "s") {
			e.preventDefault();
			handleCompile();
		}
	}
</script>

<svelte:head>
	<title>Resume Editor — Kashif</title>
</svelte:head>

<svelte:window on:keydown={handleKeydown} />

<div
	class="h-screen flex flex-col bg-[var(--color-background)] overflow-hidden"
>
	<!-- Header / Toolbar -->
	<header
		class="border-b border-[var(--color-border)] bg-[var(--color-surface)] z-20 shadow-sm relative"
	>
		<div class="w-full px-4 h-16 flex items-center justify-between gap-4">
			<!-- Left: Title & Status -->
			<div class="flex items-center gap-4">
				<div class="flex flex-col">
					<h1
						class="text-lg font-bold text-[var(--color-headline)] flex items-center gap-2"
					>
						Resume Editor
						{#if resume.version}
							<span
								class="text-xs font-normal px-2 py-0.5 rounded-full bg-[var(--color-secondary)]/10 text-[var(--color-secondary)] border border-[var(--color-secondary)]/20"
							>
								v{resume.version}
							</span>
						{/if}
					</h1>
					<div class="flex items-center gap-2 text-xs">
						{#if uploading}
							<span class="text-blue-500 flex items-center gap-1">
								<Loader2 size={12} class="animate-spin" /> Uploading...
							</span>
						{:else if compiling}
							<span
								class="text-[var(--color-secondary)] flex items-center gap-1"
							>
								<Loader2 size={12} class="animate-spin" /> Compiling...
							</span>
						{:else if hasUnsavedChanges}
							<span
								class="text-amber-500 flex items-center gap-1"
							>
								<AlertTriangle size={12} /> Unsaved changes
							</span>
						{:else}
							<span
								class="text-emerald-500 flex items-center gap-1"
							>
								<CheckCircle size={12} /> Saved
							</span>
						{/if}
					</div>
				</div>
			</div>

			<!-- Center: Progress Bar (Only when uploading) -->
			{#if uploading}
				<div class="flex-1 max-w-sm mx-4">
					<div
						class="h-2 w-full bg-gray-200 rounded-full overflow-hidden"
					>
						<div
							class="h-full bg-blue-500 transition-all duration-300 ease-out"
							style="width: {uploadProgress}%"
						></div>
					</div>
				</div>
			{/if}

			<!-- Right: Actions -->
			<div class="flex items-center gap-2">
				<!-- History -->
				<a
					href="/editor/history"
					class="p-2 text-[var(--color-paragraph)] hover:text-[var(--color-headline)] hover:bg-[var(--color-background)] rounded-lg transition-colors"
					title="Version History"
				>
					<History size={20} />
				</a>

				<div class="h-6 w-px bg-[var(--color-border)] mx-1"></div>

				<!-- Compile -->
				<button
					on:click={handleCompile}
					disabled={compiling || uploading}
					class="flex items-center gap-2 px-3 py-2 text-sm font-medium text-[var(--color-headline)] hover:bg-[var(--color-background)] rounded-lg transition-colors disabled:opacity-50"
					title="Compile (Cmd+S)"
				>
					<Play
						size={16}
						class={compiling
							? "text-gray-400"
							: "text-green-500 fill-green-500"}
					/>
					<span class="hidden sm:inline">Compile</span>
				</button>

				<!-- Download Dropdown -->
				<div class="relative">
					<button
						on:click={() => (isDownloadOpen = !isDownloadOpen)}
						class="flex items-center gap-2 px-3 py-2 text-sm font-medium text-[var(--color-headline)] hover:bg-[var(--color-background)] rounded-lg transition-colors"
					>
						<FileDown size={18} />
						<span class="hidden sm:inline">Download</span>
						<ChevronDown size={14} class="opacity-50" />
					</button>

					{#if isDownloadOpen}
						<div
							class="absolute top-full right-0 mt-1 w-48 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg shadow-lg py-1 z-50"
						>
							<button
								on:click={downloadPDF}
								class="w-full text-left px-4 py-2 text-sm text-[var(--color-headline)] hover:bg-[var(--color-background)] flex items-center gap-2"
							>
								<FileCode size={16} class="text-red-500" /> PDF Document
							</button>
							<button
								on:click={downloadTypst}
								class="w-full text-left px-4 py-2 text-sm text-[var(--color-headline)] hover:bg-[var(--color-background)] flex items-center gap-2"
							>
								<FileCode size={16} class="text-blue-500" /> Typst
								Source
							</button>
						</div>
						<!-- Backdrop to close -->
						<div
							class="fixed inset-0 z-40 bg-transparent"
							on:click={() => (isDownloadOpen = false)}
						></div>
					{/if}
				</div>

				<!-- Upload -->
				<button
					on:click={handleUpload}
					disabled={uploading || compiling}
					class="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-[var(--color-secondary)] text-white rounded-lg hover:bg-[var(--color-secondary)]/90 transition-all disabled:opacity-50 shadow-sm"
				>
					{#if uploading}
						<Loader2 size={16} class="animate-spin" />
					{:else}
						<UploadCloud size={16} />
					{/if}
					<span class="hidden sm:inline">Upload</span>
				</button>

				<div class="h-6 w-px bg-[var(--color-border)] mx-1"></div>

				<!-- Logout -->
				<button
					on:click={handleLogout}
					class="p-2 text-red-500 hover:bg-red-50 rounded-lg transition-colors"
					title="Log out"
				>
					<LogOut size={20} />
				</button>
			</div>
		</div>
	</header>

	<!-- Error Messages -->
	{#if compilationError}
		<div
			class="bg-red-500/10 border-b border-red-500/20 px-4 py-2 flex items-center gap-2 text-red-600 text-sm"
		>
			<AlertTriangle size={16} />
			<span class="font-semibold">Compilation Error:</span>
			<span class="font-mono text-xs">{compilationError}</span>
		</div>
	{/if}

	<!-- Main Content: Split View -->
	<div class="flex-1 flex flex-col md:flex-row overflow-hidden relative">
		<!-- Left: Typst Editor -->
		<div
			class="w-full md:w-1/2 border-b md:border-b-0 md:border-r border-[var(--color-border)] flex flex-col h-1/2 md:h-full"
		>
			<div class="flex-1 relative">
				{#if loading}
					<div
						class="absolute inset-0 flex items-center justify-center bg-[var(--color-surface)] z-10"
					>
						<div
							class="flex flex-col items-center gap-2 text-[var(--color-paragraph)]"
						>
							<Loader2 size={24} class="animate-spin" />
							<span>Loading editor...</span>
						</div>
					</div>
				{/if}
				<MonacoEditor
					bind:value={typstContent}
					language="plaintext"
					theme="vs-dark"
					onChange={handleEditorChange}
				/>
			</div>
		</div>

		<!-- Right: PDF Preview -->
		<div
			class="w-full md:w-1/2 flex flex-col bg-gray-900 h-1/2 md:h-full relative"
		>
			{#if compiling && !uploading}
				<div
					class="absolute inset-0 z-10 bg-black/50 backdrop-blur-sm flex items-center justify-center"
				>
					<div
						class="bg-[var(--color-surface)] p-4 rounded-xl shadow-xl flex items-center gap-3"
					>
						<Loader2
							size={24}
							class="animate-spin text-[var(--color-secondary)]"
						/>
						<span class="text-[var(--color-headline)] font-medium"
							>Compiling PDF...</span
						>
					</div>
				</div>
			{/if}

			<div class="flex-1 overflow-hidden h-full">
				{#if pdfUrl}
					<iframe
						src={pdfUrl}
						title="Resume PDF"
						class="w-full h-full border-0"
					/>
				{:else}
					<div
						class="flex items-center justify-center h-full text-gray-500"
					>
						<p>
							No PDF available. Compile successfully to preview.
						</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>
