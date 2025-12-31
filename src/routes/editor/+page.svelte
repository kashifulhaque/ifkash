<script lang="ts">
	import { auth } from "$lib/stores/auth";
	import { goto } from "$app/navigation";
	import { browser } from "$app/environment";
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
		Sparkles,
		Undo2,
		X,
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

	// AI State
	let isAiModalOpen = false;
	let isAiProcessing = false;
	let jobDescription = "";
	let selectedModel = "anthropic/claude-3.5-sonnet";
	let previousTypstContent = "";
	let canUndo = false;

	const AI_MODELS = [
		{ id: "anthropic/claude-3.5-sonnet", name: "Claude 3.5 Sonnet" },
		{ id: "google/gemini-flash-1.5", name: "Gemini 1.5 Flash" },
		{ id: "openai/gpt-4o-mini", name: "GPT-4o Mini" },
		{ id: "meta-llama/llama-3.1-70b-instruct", name: "Llama 3.1 70B" },
	];

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
			// Load preference for model
			if (browser) {
				const savedModel = localStorage.getItem("preferred_ai_model");
				if (savedModel) selectedModel = savedModel;
			}
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

			setTimeout(() => {
				alert(`Successfully uploaded version ${newVersion}!`);
				window.location.href = "/editor";
			}, 500);
		} catch (e: any) {
			clearInterval(progressInterval);
			uploadProgress = 0;
			console.error("Upload error:", e);
			alert("Upload failed: " + e.message);
			uploading = false;
		}
	}

	function handleUndo() {
		if (canUndo && previousTypstContent) {
			typstContent = previousTypstContent;
			previousTypstContent = "";
			canUndo = false;
			hasUnsavedChanges = true;
			alert("Restored previous version!");
		}
	}

	async function handleAiRewrite() {
		if (!jobDescription.trim()) {
			alert("Please enter a Job Description");
			return;
		}

		isAiProcessing = true;
		// Save current content for undo
		previousTypstContent = typstContent;

		try {
			// Save model preference
			if (browser) {
				localStorage.setItem("preferred_ai_model", selectedModel);
			}

			const token = auth.getToken();
			const apiUrl =
				typeof window !== "undefined" &&
				window.location.hostname === "localhost"
					? "http://localhost:8787/api/ai/rewrite"
					: "https://ifkash.dev/api/ai/rewrite";

			const response = await fetch(apiUrl, {
				method: "POST",
				headers: {
					Authorization: `Bearer ${token}`,
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					code: typstContent,
					job_description: jobDescription,
					model: selectedModel,
				}),
			});

			if (!response.ok) {
				const errorText = await response.text();
				throw new Error(errorText || "AI processing failed");
			}

			const data = await response.json();
			if (data.success && data.code) {
				typstContent = data.code;
				hasUnsavedChanges = true;
				canUndo = true;
				isAiModalOpen = false;
				alert(
					"Resume tailored! Please review changes. Click Undo icon to revert.",
				);
			} else {
				throw new Error(data.message || "Unknown error");
			}
		} catch (e: any) {
			console.error("AI Error:", e);
			alert("AI tailoring failed: " + e.message);
		} finally {
			isAiProcessing = false;
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
				{#if canUndo}
					<button
						on:click={handleUndo}
						class="p-2 text-yellow-500 hover:bg-yellow-500/10 rounded-lg transition-colors"
						title="Undo AI Changes"
					>
						<Undo2 size={20} />
					</button>
				{/if}

				<!-- AI Magic -->
				<button
					on:click={() => (isAiModalOpen = true)}
					disabled={compiling || uploading || isAiProcessing}
					class="p-2 text-purple-400 hover:text-purple-300 hover:bg-purple-400/10 rounded-lg transition-colors"
					title="Tailor with AI"
				>
					<Sparkles size={20} />
				</button>

				<div class="h-6 w-px bg-[var(--color-border)] mx-1"></div>

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

	<!-- AI Modal -->
	{#if isAiModalOpen}
		<div
			class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm"
		>
			<div
				class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl shadow-2xl w-full max-w-lg flex flex-col max-h-[90vh]"
			>
				<div
					class="flex items-center justify-between p-4 border-b border-[var(--color-border)]"
				>
					<h3
						class="text-lg font-bold text-[var(--color-headline)] flex items-center gap-2"
					>
						<Sparkles size={18} class="text-purple-400" /> Tailor Resume
					</h3>
					<button
						on:click={() => (isAiModalOpen = false)}
						class="text-[var(--color-paragraph)] hover:text-[var(--color-headline)]"
					>
						<X size={20} />
					</button>
				</div>

				<div class="p-4 flex flex-col gap-4 overflow-y-auto">
					<div>
						<label
							class="block text-sm font-medium text-[var(--color-headline)] mb-1"
						>
							AI Model
						</label>
						<select
							bind:value={selectedModel}
							class="w-full bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-[var(--color-paragraph)] focus:outline-none focus:border-[var(--color-secondary)]"
						>
							{#each AI_MODELS as model}
								<option value={model.id}>{model.name}</option>
							{/each}
						</select>
					</div>

					<div>
						<label
							class="block text-sm font-medium text-[var(--color-headline)] mb-1"
						>
							Job Description
						</label>
						<textarea
							bind:value={jobDescription}
							placeholder="Paste the job description here..."
							class="w-full h-40 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-[var(--color-paragraph)] focus:outline-none focus:border-[var(--color-secondary)] resize-none"
						></textarea>
					</div>

					<div class="bg-blue-500/10 p-3 rounded-lg">
						<p class="text-xs text-blue-400">
							<strong>Note:</strong> The AI will rewrite your summary
							and bullet points to match the JD keywords. Your resume
							structure will be preserved.
						</p>
					</div>
				</div>

				<div
					class="p-4 border-t border-[var(--color-border)] flex justify-end gap-2"
				>
					<button
						on:click={() => (isAiModalOpen = false)}
						class="px-4 py-2 text-sm text-[var(--color-paragraph)] hover:bg-[var(--color-background)] rounded-lg transition-colors"
					>
						Cancel
					</button>
					<button
						on:click={handleAiRewrite}
						disabled={isAiProcessing || !jobDescription.trim()}
						class="px-4 py-2 text-sm font-semibold bg-purple-600 hover:bg-purple-700 text-white rounded-lg transition-colors disabled:opacity-50 flex items-center gap-2"
					>
						{#if isAiProcessing}
							<Loader2 size={16} class="animate-spin" /> Processing...
						{:else}
							<Sparkles size={16} /> Tailor Resume
						{/if}
					</button>
				</div>
			</div>
		</div>
	{/if}

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
