import { json, error } from '@sveltejs/kit';
import { githubFetch, getDefaultBranch } from '$lib/server/github';
import type { RequestHandler } from './$types';

// Lightweight listing: one recursive git-tree call returns paths + blob shas
// only — no clone, no file contents. Blobs are fetched lazily per file.
export const GET: RequestHandler = async () => {
	const branch = await getDefaultBranch();
	const res = await githubFetch(`/git/trees/${encodeURIComponent(branch)}?recursive=1`);
	if (!res.ok) throw error(res.status, `GitHub: failed to list tree (${res.status})`);
	const data = await res.json();

	const files = (data.tree as Array<{ path: string; type: string; size?: number; sha: string }>)
		.filter((e) => e.type === 'blob')
		.map((e) => ({ path: e.path, size: e.size ?? 0, sha: e.sha }));

	return json({ branch, truncated: Boolean(data.truncated), files });
};
