import { error } from '@sveltejs/kit';
import { githubFetch, getDefaultBranch } from '$lib/server/github';
import type { RequestHandler } from './$types';

const CONTENT_TYPES: Record<string, string> = {
	png: 'image/png',
	jpg: 'image/jpeg',
	jpeg: 'image/jpeg',
	gif: 'image/gif',
	webp: 'image/webp',
	svg: 'image/svg+xml',
	avif: 'image/avif',
	bmp: 'image/bmp',
	ico: 'image/x-icon',
	pdf: 'application/pdf',
	mp3: 'audio/mpeg',
	mp4: 'video/mp4',
	webm: 'video/webm'
};

// Serve a file's raw bytes (images and other attachments embedded in notes).
export const GET: RequestHandler = async ({ url }) => {
	const path = url.searchParams.get('path');
	if (!path) throw error(400, 'path is required');

	const branch = await getDefaultBranch();
	const encoded = path.split('/').map(encodeURIComponent).join('/');
	const res = await githubFetch(`/contents/${encoded}?ref=${encodeURIComponent(branch)}`, {
		headers: { Accept: 'application/vnd.github.raw+json' }
	});
	if (res.status === 404) throw error(404, 'File not found');
	if (!res.ok) throw error(res.status, `GitHub: failed to read file (${res.status})`);

	const ext = path.split('.').pop()?.toLowerCase() ?? '';
	const filename = path.split('/').pop() ?? 'file';
	const headers: Record<string, string> = {
		'Content-Type': CONTENT_TYPES[ext] ?? 'application/octet-stream',
		'Cache-Control': 'private, max-age=300'
	};
	if (url.searchParams.get('download')) {
		headers['Content-Disposition'] =
			`attachment; filename*=UTF-8''${encodeURIComponent(filename)}`;
	}
	return new Response(res.body, { headers });
};
