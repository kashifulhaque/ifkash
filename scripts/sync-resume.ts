#!/usr/bin/env bun

/**
 * Sync resume files from PocketBase to static/assets
 * This ensures the static fallback is always up-to-date
 */

const POCKETBASE_URL = 'https://pb.ifkash.dev';
const POCKETBASE_EMAIL = process.env.POCKETBASE_EMAIL || 'haque.kashiful7@gmail.com';
const POCKETBASE_PASSWORD = process.env.POCKETBASE_PASSWORD || 'pocketbase-ifkash-password';

async function authenticateWithPocketBase() {
	const response = await fetch(
		`${POCKETBASE_URL}/api/collections/_superusers/auth-with-password`,
		{
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				identity: POCKETBASE_EMAIL,
				password: POCKETBASE_PASSWORD
			})
		}
	);

	if (!response.ok) {
		const error = await response.text();
		throw new Error(`Authentication failed: ${error}`);
	}

	const data = await response.json();
	return data.token;
}

async function getLatestResume(token) {
	const response = await fetch(
		`${POCKETBASE_URL}/api/collections/resumes/records?sort=-created&perPage=1`,
		{
			headers: {
				Authorization: `Bearer ${token}`
			}
		}
	);

	if (!response.ok) {
		throw new Error(`Failed to fetch resume: ${response.statusText}`);
	}

	const data = await response.json();
	if (!data.items || data.items.length === 0) {
		throw new Error('No resume found in PocketBase');
	}

	return data.items[0];
}

async function downloadFile(url, outputPath) {
	console.log(`Downloading ${url}...`);
	const response = await fetch(url);

	if (!response.ok) {
		throw new Error(`Failed to download file: ${response.statusText}`);
	}

	const arrayBuffer = await response.arrayBuffer();
	await Bun.write(outputPath, new Uint8Array(arrayBuffer));
	console.log(`✓ Saved to ${outputPath}`);
}

async function main() {
	try {
		console.log('🔐 Authenticating with PocketBase...');
		const token = await authenticateWithPocketBase();

		console.log('📥 Fetching latest resume...');
		const resume = await getLatestResume(token);

		console.log(`📄 Latest resume: v${resume.version || 'unknown'} (${resume.created})`);

		// Construct file URLs
		const pdfUrl = `${POCKETBASE_URL}/api/files/${resume.collectionId}/${resume.id}/${resume.pdf_file}`;
		const typstUrl = `${POCKETBASE_URL}/api/files/${resume.collectionId}/${resume.id}/${resume.typst_file}`;

		// Download files
		await downloadFile(pdfUrl, './static/assets/Kashiful_Haque.pdf');
		await downloadFile(typstUrl, `./static/assets/${resume.typst_file}`);

		console.log('\n✅ Resume files synced successfully!');
		console.log(`   Version: ${resume.version || 'N/A'}`);
		console.log(`   Updated: ${resume.updated}`);
	} catch (error) {
		console.error('❌ Error syncing resume:', error.message);
		process.exit(1);
	}
}

main();
