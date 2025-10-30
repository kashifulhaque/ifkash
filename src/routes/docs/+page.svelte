<script>
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	onMount(async () => {
		if (browser) {
			// Import Swagger UI dynamically in the browser
			const SwaggerUIBundle = (await import('swagger-ui-dist/swagger-ui-bundle.js')).default;
			const SwaggerUIStandalonePreset = (await import('swagger-ui-dist/swagger-ui-standalone-preset.js')).default;

			SwaggerUIBundle({
				url: '/api/openapi',
				dom_id: '#swagger-ui',
				deepLinking: true,
				presets: [
					SwaggerUIBundle.presets.apis,
					SwaggerUIStandalonePreset
				],
				plugins: [
					SwaggerUIBundle.plugins.DownloadUrl
				],
				layout: "StandaloneLayout",
				validatorUrl: null,
				tryItOutEnabled: true,
				requestInterceptor: (request) => {
					// Modify request headers if needed
					return request;
				},
				responseInterceptor: (response) => {
					// Handle response if needed
					return response;
				}
			});
		}
	});
</script>

<svelte:head>
	<title>API Docs</title>
	<meta name="description" content="Interactive API documentation for ifkash.dev APIs" />

	<!-- Swagger UI CSS -->
	<link rel="stylesheet" type="text/css" href="/swagger-ui-dist/swagger-ui.css" />

	<style>
		/* Custom styling to match your site theme */
		.swagger-ui .topbar {
			display: none;
		}

		.swagger-ui .info {
			margin-bottom: 30px;
		}

		.swagger-ui .scheme-container {
			background: transparent;
			border: none;
			box-shadow: none;
		}

		/* Dark theme styling for Swagger UI */
		.swagger-ui {
			background: transparent;
			color: #e5e7eb;
		}

		.swagger-ui .wrapper {
			background: transparent;
		}

		.swagger-ui .opblock {
			background: #1f2937;
			border: 1px solid #374151;
		}

		.swagger-ui .opblock .opblock-summary {
			background: #111827;
			border-color: #374151;
		}

		.swagger-ui .opblock .opblock-summary:hover {
			background: #1f2937;
		}

		.swagger-ui .opblock.opblock-get .opblock-summary-method {
			background: #059669;
		}

		.swagger-ui .opblock.opblock-post .opblock-summary-method {
			background: #dc2626;
		}

		.swagger-ui .btn.authorize {
			background: #374151;
			border-color: #4b5563;
			color: #e5e7eb;
		}

		.swagger-ui .btn.try-out__btn {
			background: #374151;
			border-color: #4b5563;
			color: #e5e7eb;
		}

		.swagger-ui input, .swagger-ui textarea, .swagger-ui select {
			background: #111827;
			border-color: #374151;
			color: #e5e7eb;
		}

		/* Fix contrast issues in dark theme */
		.swagger-ui .opblock-description-wrapper p,
		.swagger-ui .opblock-description-wrapper,
		.swagger-ui .opblock-summary-description {
			color: #d1d5db !important;
		}

		.swagger-ui .parameter__name,
		.swagger-ui .parameter__type {
			color: #f3f4f6 !important;
		}

		.swagger-ui .parameter__in {
			color: #9ca3af !important;
		}

		.swagger-ui .response-col_status {
			color: #f3f4f6 !important;
		}

		.swagger-ui .response-col_description {
			color: #d1d5db !important;
		}

		.swagger-ui .model-title,
		.swagger-ui .model {
			color: #e5e7eb !important;
		}

		.swagger-ui table thead tr th,
		.swagger-ui table thead tr td {
			color: #f3f4f6 !important;
			border-color: #374151 !important;
		}

		.swagger-ui .parameters-col_description p,
		.swagger-ui .parameters-col_description {
			color: #d1d5db !important;
		}

		.swagger-ui .prop-type,
		.swagger-ui .prop-format {
			color: #9ca3af !important;
		}

		.swagger-ui .renderedMarkdown p,
		.swagger-ui .renderedMarkdown code {
			color: #e5e7eb !important;
		}

		.swagger-ui .info .title,
		.swagger-ui .info h1,
		.swagger-ui .info h2,
		.swagger-ui .info h3 {
			color: #f9fafb !important;
		}

		.swagger-ui .info .description,
		.swagger-ui .info p {
			color: #d1d5db !important;
		}

		/* Additional contrast fixes for labels and sections */
		.swagger-ui .opblock-section-header h4,
		.swagger-ui .opblock-section-header label {
			color: #e5e7eb !important;
		}

		.swagger-ui .opblock-body pre,
		.swagger-ui .microlight {
			background: #0f172a !important;
			color: #e5e7eb !important;
		}

		.swagger-ui .responses-inner h4,
		.swagger-ui .responses-inner h5 {
			color: #d1d5db !important;
		}

		.swagger-ui .tab li {
			color: #9ca3af !important;
		}

		.swagger-ui .opblock-control-arrow {
			fill: #9ca3af !important;
		}

		/* Code highlighting improvements */
		.swagger-ui .highlight-code > .microlight code {
			color: #e5e7eb !important;
		}

		.swagger-ui .model-box,
		.swagger-ui .model-box-control {
			background: #111827 !important;
		}

		/* Fix specific low-contrast labels */
		.swagger-ui .responses-header td,
		.swagger-ui .response-col_links {
			color: #d1d5db !important;
		}

		.swagger-ui .opblock-body .opblock-section-header,
		.swagger-ui .opblock-section-header > label,
		.swagger-ui .opblock-section-header > h4 {
			color: #d1d5db !important;
		}

		.swagger-ui .scheme-container .schemes > label {
			color: #d1d5db !important;
		}

		/* Tiny affordance for the arrow chip */
		.i-tab { transition: transform 150ms ease; }
		a.group:hover .i-tab { transform: translateY(-1px); }

		/* Mobile menu animation */
		@keyframes fade-in {
			from {
				opacity: 0;
				transform: translateY(-8px);
			}
			to {
				opacity: 1;
				transform: translateY(0);
			}
		}

		.animate-fade-in {
			animation: fade-in 200ms ease-out;
		}
	</style>
</svelte:head>

<!-- Page container -->
<div class="min-h-screen bg-neutral-950 text-neutral-200 selection:bg-neutral-800 selection:text-white">
	<div class="mx-auto max-w-5xl px-5 sm:px-6 pb-24">
		<!-- Header / Nav -->
    <header class="sticky top-0 z-30 -mx-5 sm:-mx-6 backdrop-blur supports-[backdrop-filter]:bg-neutral-950/60">
      <div class="mx-auto max-w-3xl px-5 sm:px-6">
        <nav class="flex items-center justify-between py-4">
          <a href="/" class="font-semibold tracking-tight text-neutral-100">ifkash.dev</a>
          <a href="/" class="rounded-full px-3 py-1 text-sm hover:bg-neutral-800/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-neutral-600">Home</a>
        </nav>
      </div>
      <div class="h-px w-full bg-gradient-to-r from-transparent via-neutral-800 to-transparent"></div>
    </header>

		<!-- Main content -->
		<main class="pt-14 sm:pt-20">
			<div class="mb-8">
				<h1 class="text-4xl sm:text-5xl font-semibold leading-tight tracking-tight text-neutral-100 mb-4">
					API Documentation
				</h1>
				<p class="text-neutral-400 max-w-2xl">
					Interactive documentation for the ifkash.dev APIs. Test endpoints directly from this page.
				</p>
			</div>

			<div class="bg-neutral-900/50 backdrop-blur border border-neutral-800 rounded-lg p-6">
				<div id="swagger-ui"></div>
			</div>
		</main>
	</div>
</div>
