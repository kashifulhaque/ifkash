import { env } from '$env/dynamic/private';
import { error } from '@sveltejs/kit';

// Notes live in this repo; commits are made via the GitHub REST API with a
// fine-grained PAT (Cloudflare secret GITHUB_TOKEN) — Workers have no git/ssh.
const REPO = 'kashifulhaque/obsidian-sync';
const API = 'https://api.github.com';

export const COMMIT_AUTHOR = {
	name: 'kashifulhaque',
	email: 'haque.kashiful7@gmail.com'
};

export async function githubFetch(path: string, init: RequestInit = {}): Promise<Response> {
	const token = env.GITHUB_TOKEN;
	if (!token) throw error(500, 'GITHUB_TOKEN is not configured');

	return fetch(`${API}/repos/${REPO}${path}`, {
		...init,
		headers: {
			Authorization: `Bearer ${token}`,
			Accept: 'application/vnd.github+json',
			'X-GitHub-Api-Version': '2022-11-28',
			'User-Agent': 'ifkash-notes-editor',
			...init.headers
		}
	});
}

export async function getDefaultBranch(): Promise<string> {
	const res = await githubFetch('');
	if (!res.ok) throw error(res.status, `GitHub: failed to read repo (${res.status})`);
	const repo = await res.json();
	return repo.default_branch as string;
}
