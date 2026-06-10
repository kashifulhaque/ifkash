import { json, error } from '@sveltejs/kit';
import { githubFetch, getDefaultBranch, COMMIT_AUTHOR } from '$lib/server/github';
import type { RequestHandler } from './$types';

function decodeBase64Utf8(b64: string): string {
	const bin = atob(b64.replace(/\n/g, ''));
	const bytes = new Uint8Array(bin.length);
	for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
	return new TextDecoder().decode(bytes);
}

function encodeUtf8Base64(text: string): string {
	const bytes = new TextEncoder().encode(text);
	let bin = '';
	const chunk = 0x8000;
	for (let i = 0; i < bytes.length; i += chunk) {
		bin += String.fromCharCode(...bytes.subarray(i, i + chunk));
	}
	return btoa(bin);
}

// Fetch a single file's content + blob sha on demand.
export const GET: RequestHandler = async ({ url }) => {
	const path = url.searchParams.get('path');
	if (!path) throw error(400, 'path is required');

	const branch = await getDefaultBranch();
	const encoded = path.split('/').map(encodeURIComponent).join('/');
	const res = await githubFetch(`/contents/${encoded}?ref=${encodeURIComponent(branch)}`);
	if (res.status === 404) throw error(404, 'File not found');
	if (!res.ok) throw error(res.status, `GitHub: failed to read file (${res.status})`);
	const data = await res.json();

	let content: string;
	if (data.content) {
		content = decodeBase64Utf8(data.content);
	} else {
		// Files over 1MB: the JSON response omits content; re-fetch raw.
		const raw = await githubFetch(`/contents/${encoded}?ref=${encodeURIComponent(branch)}`, {
			headers: { Accept: 'application/vnd.github.raw+json' }
		});
		if (!raw.ok) throw error(raw.status, `GitHub: failed to read large file (${raw.status})`);
		content = await raw.text();
	}

	return json({ path, sha: data.sha, content });
};

// Save (or create, when sha is omitted) a file as a commit on the default
// branch. GitHub rejects a stale sha with 409/422, which the UI surfaces as
// "changed upstream — reload" instead of producing a merge conflict.
export const PUT: RequestHandler = async ({ request }) => {
	const { path, content, sha, encoding } = (await request.json()) as {
		path?: string;
		content?: string;
		sha?: string;
		encoding?: 'utf-8' | 'base64';
	};
	if (!path || typeof content !== 'string') throw error(400, 'path and content are required');

	const branch = await getDefaultBranch();
	const encoded = path.split('/').map(encodeURIComponent).join('/');
	const res = await githubFetch(`/contents/${encoded}`, {
		method: 'PUT',
		body: JSON.stringify({
			message: `vault: ${sha ? 'update' : 'create'} ${path}`,
			// base64 encoding is used for binary uploads (images, PDFs, …)
			content: encoding === 'base64' ? content : encodeUtf8Base64(content),
			branch,
			...(sha ? { sha } : {}),
			committer: COMMIT_AUTHOR,
			author: COMMIT_AUTHOR
		})
	});

	if (res.status === 409 || res.status === 422) {
		const body = await res.json().catch(() => ({}));
		return json(
			{ conflict: true, message: body?.message ?? 'File changed upstream' },
			{ status: 409 }
		);
	}
	if (!res.ok) throw error(res.status, `GitHub: failed to save file (${res.status})`);
	const data = await res.json();

	return json({
		path,
		sha: data.content?.sha,
		commit: { sha: data.commit?.sha, url: data.commit?.html_url }
	});
};

// Delete a file as a commit on the default branch.
export const DELETE: RequestHandler = async ({ request }) => {
	const { path, sha } = (await request.json()) as { path?: string; sha?: string };
	if (!path || !sha) throw error(400, 'path and sha are required');

	const branch = await getDefaultBranch();
	const encoded = path.split('/').map(encodeURIComponent).join('/');
	const res = await githubFetch(`/contents/${encoded}`, {
		method: 'DELETE',
		body: JSON.stringify({
			message: `vault: delete ${path}`,
			sha,
			branch,
			committer: COMMIT_AUTHOR,
			author: COMMIT_AUTHOR
		})
	});

	if (res.status === 409 || res.status === 422) {
		return json({ conflict: true, message: 'File changed upstream' }, { status: 409 });
	}
	if (!res.ok) throw error(res.status, `GitHub: failed to delete file (${res.status})`);
	const data = await res.json();

	return json({ path, commit: { sha: data.commit?.sha } });
};
